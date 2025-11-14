use futures_util::{SinkExt, StreamExt};
use reqwest;
use serde_json::{json, Value};
use std::time::Duration;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

#[derive(Error, Debug)]
pub enum NemoAgentError {
    #[error("Unable to connect to NAT server at {url}. Make sure the server is running with: nat serve --config_file orbit_config.yml")]
    ConnectionError { url: String },
    #[error("Request to NAT server timed out after {timeout} seconds")]
    TimeoutError { timeout: u64 },
    #[error("NAT server returned error: {status} - {message}")]
    HttpError { status: u16, message: String },
    #[error("NAT server returned invalid JSON response")]
    InvalidJsonResponse,
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("URL parse error: {0}")]
    UrlError(#[from] url::ParseError),
}

pub struct NemoAgent {
    base_url: String,
    timeout: Duration,
    client: reqwest::Client,
    ws_connection: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl NemoAgent {
    /// Initialize NemoAgent to communicate with NAT server.
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the NAT server (default: http://localhost:8000)
    /// * `timeout_secs` - Request timeout in seconds (default: 30)
    pub fn new(base_url: Option<String>, timeout_secs: Option<u64>) -> Self {
        let base_url = base_url.unwrap_or_else(|| "http://localhost:8000".to_string());
        let timeout = Duration::from_secs(timeout_secs.unwrap_or(30));

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        println!("Initializing NemoAgent - NAT server at {}", base_url);

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            timeout,
            client,
            ws_connection: None,
        }
    }

    /// Make a chat completion request to the NAT server.
    async fn make_request(&self, payload: Value) -> Result<String, NemoAgentError> {
        let url = format!("{}/chat", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&payload)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    NemoAgentError::ConnectionError {
                        url: self.base_url.clone(),
                    }
                } else if e.is_timeout() {
                    NemoAgentError::TimeoutError {
                        timeout: self.timeout.as_secs(),
                    }
                } else {
                    NemoAgentError::RequestError(e)
                }
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(NemoAgentError::HttpError { status, message });
        }

        let data: Value = response
            .json()
            .await
            .map_err(|_| NemoAgentError::InvalidJsonResponse)?;

        self.parse_response(data)
    }

    /// Send a message to the agent and get a response.
    /// NAT server handles all conversation state management.
    ///
    /// # Arguments
    /// * `message` - User message to send to the agent
    /// * `conversation_id` - Optional conversation ID for multi-tab/session support
    pub async fn chat(
        &self,
        message: &str,
        conversation_id: Option<&str>,
    ) -> Result<String, NemoAgentError> {
        let mut payload = json!({
            "model": "orbit-ai",
            "messages": [{"role": "user", "content": message}],
            "temperature": 0.1,
            "max_tokens": 1000
        });

        // Add conversation_id if provided for session management
        if let Some(conv_id) = conversation_id {
            payload["conversation_id"] = json!(conv_id);
        }

        self.make_request(payload).await
    }

    /// Connect to the NAT server via WebSocket for persistent session.
    pub async fn connect_websocket(&mut self) -> Result<(), NemoAgentError> {
        if self.ws_connection.is_some() {
            return Ok(()); // Already connected
        }

        let ws_url = self
            .base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        // Use the correct NAT WebSocket endpoint from source code
        let ws_url = format!("{}/websocket", ws_url);

        println!("Connecting to WebSocket at: {}", ws_url);
        let (ws_stream, _) = connect_async(&ws_url).await?;
        self.ws_connection = Some(ws_stream);

        Ok(())
    }

    /// Send a message via WebSocket and stream responses in real-time.
    /// Calls the callback for each chunk received.
    pub async fn chat_websocket_stream<F>(
        &mut self,
        message: &str,
        conversation_id: Option<&str>,
        mut on_chunk: F,
    ) -> Result<String, NemoAgentError>
    where
        F: FnMut(&str),
    {
        // Ensure WebSocket connection exists
        if self.ws_connection.is_none() {
            self.connect_websocket().await?;
        }

        // Use proper NAT WebSocket schema format
        let payload = json!({
            "type": "user_message",
            "schema_type": "chat",
            "id": format!("msg-{}", chrono::Utc::now().timestamp_millis()),
            "conversation_id": conversation_id,
            "content": {
                "messages": [{
                    "role": "user",
                    "content": [{
                        "type": "text",
                        "text": message
                    }]
                }]
            },
            "user": {
                "name": "rust-client",
                "email": "default"
            },
            "security": {},
            "error": {},
            "schema_version": "1.0.0",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        if let Some(ws) = &mut self.ws_connection {
            // Send message
            ws.send(Message::Text(payload.to_string().into())).await?;

            let mut final_response = String::new();

            // Stream responses
            while let Some(msg) = ws.next().await {
                match msg? {
                    Message::Text(text) => {
                        if let Ok(data) = serde_json::from_str::<Value>(&text) {
                            // Check if this is a final response
                            if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
                                if status == "complete" {
                                    break;
                                }
                            }

                            // Extract and stream content
                            if let Some(content) = data.get("content") {
                                if let Some(payload) = content.get("payload") {
                                    if let Some(payload_str) = payload.as_str() {
                                        on_chunk(payload_str);
                                        final_response.push_str(payload_str);
                                    }
                                }
                            }
                        }
                    }
                    Message::Close(_) => {
                        self.ws_connection = None;
                        break;
                    }
                    _ => continue,
                }
            }

            return Ok(final_response);
        }

        Err(NemoAgentError::WebSocketError(
            tokio_tungstenite::tungstenite::Error::ConnectionClosed,
        ))
    }

    /// Send a message via WebSocket and receive complete response.
    pub async fn chat_websocket(
        &mut self,
        message: &str,
        conversation_id: Option<&str>,
    ) -> Result<String, NemoAgentError> {
        self.chat_websocket_stream(message, conversation_id, |_| {})
            .await
    }

    /// Parse response from either HTTP or WebSocket.
    fn parse_response(&self, data: Value) -> Result<String, NemoAgentError> {
        // Handle different response formats
        if let Some(choices) = data.get("choices").and_then(|c| c.as_array()) {
            if let Some(choice) = choices.get(0) {
                if let Some(content) = choice
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_str())
                {
                    return Ok(content.to_string());
                }
            }
        }

        // Handle nested value format where choices are inside a "value" object
        if let Some(value) = data.get("value") {
            if let Some(choices) = value.get("choices").and_then(|c| c.as_array()) {
                if let Some(choice) = choices.get(0) {
                    if let Some(content) = choice
                        .get("message")
                        .and_then(|m| m.get("content"))
                        .and_then(|c| c.as_str())
                    {
                        return Ok(content.to_string());
                    }
                }
            }
            // Fallback to string representation if it's not the expected nested format
            if let Some(value_str) = value.as_str() {
                return Ok(value_str.to_string());
            }
        }

        if let Some(response) = data.get("response").and_then(|r| r.as_str()) {
            return Ok(response.to_string());
        }

        Ok(data.to_string())
    }

    /// Disconnect WebSocket connection.
    pub async fn disconnect_websocket(&mut self) -> Result<(), NemoAgentError> {
        if let Some(mut ws) = self.ws_connection.take() {
            ws.close(None).await?;
        }
        Ok(())
    }

    /// Check if the NAT server is running and responsive.
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/health", self.base_url);

        match self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

impl Default for NemoAgent {
    fn default() -> Self {
        Self::new(None, None)
    }
}
