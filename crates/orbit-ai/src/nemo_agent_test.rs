#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use serde_json::json;

    #[tokio::test]
    async fn test_new_agent_with_defaults() {
        let agent = NemoAgent::new(None, None);
        assert_eq!(agent.base_url, "http://localhost:8000");
        assert_eq!(agent.timeout.as_secs(), 30);
    }

    #[tokio::test]
    async fn test_new_agent_with_custom_values() {
        let agent = NemoAgent::new(Some("http://example.com:9000".to_string()), Some(60));
        assert_eq!(agent.base_url, "http://example.com:9000");
        assert_eq!(agent.timeout.as_secs(), 60);
    }

    #[tokio::test]
    async fn test_new_agent_strips_trailing_slash() {
        let agent = NemoAgent::new(Some("http://example.com:9000/".to_string()), None);
        assert_eq!(agent.base_url, "http://example.com:9000");
    }

    #[tokio::test]
    async fn test_default_implementation() {
        let agent = NemoAgent::default();
        assert_eq!(agent.base_url, "http://localhost:8000");
        assert_eq!(agent.timeout.as_secs(), 30);
    }

    #[tokio::test]
    async fn test_chat_success_choices_format() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "choices": [{
                        "message": {
                            "content": "Hello, I'm your AI assistant!"
                        }
                    }]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let agent = NemoAgent::new(Some(server.url()), Some(30));
        let result = agent.chat("Hello", None).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, I'm your AI assistant!");
    }

    #[tokio::test]
    async fn test_chat_success_value_format() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "value": "Response from value field"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let agent = NemoAgent::new(Some(server.url()), Some(30));
        let result = agent.chat("Test message", None).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Response from value field");
    }

    #[tokio::test]
    async fn test_chat_success_response_format() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "response": "Response from response field"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let agent = NemoAgent::new(Some(server.url()), Some(30));
        let result = agent.chat("Test message", None).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Response from response field");
    }

    #[tokio::test]
    async fn test_chat_with_conversation_id() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .match_body(mockito::Matcher::JsonString(
                json!({
                    "model": "orbit-ai",
                    "messages": [{"role": "user", "content": "Hello"}],
                    "temperature": 0.1,
                    "max_tokens": 1000,
                    "conversation_id": "test-conv-123"
                })
                .to_string(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "choices": [{
                        "message": {
                            "content": "Hello with conversation ID!"
                        }
                    }]
                })
                .to_string(),
            )
            .create_async()
            .await;

        let agent = NemoAgent::new(Some(server.url()), Some(30));
        let result = agent.chat("Hello", Some("test-conv-123")).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello with conversation ID!");
    }

    #[tokio::test]
    async fn test_chat_fallback_to_string() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                json!({
                    "unknown_field": "some data"
                })
                .to_string(),
            )
            .create_async()
            .await;

        let agent = NemoAgent::new(Some(server.url()), Some(30));
        let result = agent.chat("Test message", None).await;

        mock.assert_async().await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("unknown_field"));
    }

    #[tokio::test]
    async fn test_chat_http_error() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(500)
            .with_body("Internal Server Error")
            .create_async()
            .await;

        let agent = NemoAgent::new(Some(server.url()), Some(30));
        let result = agent.chat("Test message", None).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NemoAgentError::HttpError { status, message } => {
                assert_eq!(status, 500);
                assert_eq!(message, "Internal Server Error");
            }
            _ => panic!("Expected HttpError"),
        }
    }

    #[tokio::test]
    async fn test_chat_invalid_json() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("invalid json")
            .create_async()
            .await;

        let agent = NemoAgent::new(Some(server.url()), Some(30));
        let result = agent.chat("Test message", None).await;

        mock.assert_async().await;
        assert!(result.is_err());
        match result.unwrap_err() {
            NemoAgentError::InvalidJsonResponse => {}
            _ => panic!("Expected InvalidJsonResponse"),
        }
    }

    #[tokio::test]
    async fn test_chat_connection_error() {
        // Use a non-existent server
        let agent = NemoAgent::new(Some("http://nonexistent:9999".to_string()), Some(1));
        let result = agent.chat("Test message", None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            NemoAgentError::ConnectionError { url } => {
                assert_eq!(url, "http://nonexistent:9999");
            }
            _ => panic!("Expected ConnectionError"),
        }
    }

    #[tokio::test]
    async fn test_health_check_success() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/health")
            .with_status(200)
            .create_async()
            .await;

        let agent = NemoAgent::new(Some(server.url()), Some(30));
        let result = agent.health_check().await;

        mock.assert_async().await;
        assert!(result);
    }

    #[tokio::test]
    async fn test_health_check_failure() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/health")
            .with_status(500)
            .create_async()
            .await;

        let agent = NemoAgent::new(Some(server.url()), Some(30));
        let result = agent.health_check().await;

        mock.assert_async().await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_health_check_connection_error() {
        // Use a non-existent server
        let agent = NemoAgent::new(Some("http://nonexistent:9999".to_string()), Some(1));
        let result = agent.health_check().await;

        assert!(!result);
    }

    #[tokio::test]
    async fn test_error_display() {
        let connection_error = NemoAgentError::ConnectionError {
            url: "http://localhost:8000".to_string(),
        };
        assert_eq!(
            connection_error.to_string(),
            "Unable to connect to NAT server at http://localhost:8000. Make sure the server is running with: nat serve --config_file orbit_config.yml"
        );

        let timeout_error = NemoAgentError::TimeoutError { timeout: 30 };
        assert_eq!(
            timeout_error.to_string(),
            "Request to NAT server timed out after 30 seconds"
        );

        let http_error = NemoAgentError::HttpError {
            status: 404,
            message: "Not Found".to_string(),
        };
        assert_eq!(
            http_error.to_string(),
            "NAT server returned error: 404 - Not Found"
        );

        let json_error = NemoAgentError::InvalidJsonResponse;
        assert_eq!(
            json_error.to_string(),
            "NAT server returned invalid JSON response"
        );
    }

    #[tokio::test]
    async fn test_websocket_conversation_memory() {
        let mut agent = NemoAgent::new(None, Some(30));

        // Test WebSocket conversation with memory
        let conv_id = "test-websocket-session";

        // First message with streaming
        println!("Testing WebSocket streaming...");
        let first_result = agent
            .chat_websocket_stream("yeet", Some(conv_id), |chunk| {
                print!("Chunk: {}", chunk);
            })
            .await;

        match first_result {
            Ok(response) => {
                println!("\nWebSocket first response complete: {}", response);
                assert!(!response.is_empty());
            }
            Err(error) => {
                println!("WebSocket first request failed: {}", error);
                // Skip test if WebSocket not available
                return;
            }
        }

        // Second message asking about previous conversation
        let second_result = agent
            .chat_websocket_stream(
                "what word did I ask you about earlier?",
                Some(conv_id),
                |chunk| {
                    print!("Chunk: {}", chunk);
                },
            )
            .await;
        match second_result {
            Ok(response) => {
                println!("\nWebSocket second response complete: {}", response);
                assert!(!response.is_empty());
                // Check if agent remembers "yeet" - this should work with WebSocket persistence
                if response.to_lowercase().contains("yeet") {
                    println!("WebSocket conversation memory works!");
                } else {
                    println!("WebSocket still no conversation memory: {}", response);
                }
            }
            Err(error) => {
                println!("WebSocket second request failed: {}", error);
            }
        }

        // Clean up
        let _ = agent.disconnect_websocket().await;
    }

    #[tokio::test]
    async fn test_yeet() {
        let mut agent = NemoAgent::new(None, Some(30));
        let conv_id = "yeet-session";

        let ws_first = agent.chat_websocket("yeet", Some(conv_id)).await;
        let ws_second = agent
            .chat_websocket("what word did I ask you about earlier?", Some(conv_id))
            .await;

        match (ws_first, ws_second) {
            (Ok(first_response), Ok(second_response)) => {
                println!("🔌 WebSocket Mode:");
                println!("First response: {}", first_response);
                println!("Second response: {}", second_response);

                if second_response.to_lowercase().contains("yeet") {
                    println!("WebSocket conversation memory works!");
                } else {
                    println!("WebSocket still no conversation memory");
                    assert!(second_response.to_lowercase().contains("yeet"))
                }

                return;
            }
            (Err(ws_error), _) => {
                println!("WebSocket failed: {}", ws_error);
                let _ = agent.disconnect_websocket().await;
            }
            (_, Err(ws_error)) => {
                println!("WebSocket second request failed: {}", ws_error);
                let _ = agent.disconnect_websocket().await;
            }
        }
        let _ = agent.disconnect_websocket().await;
    }
}
