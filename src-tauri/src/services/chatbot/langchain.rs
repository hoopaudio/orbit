use std::env;

use crate::services::chatbot::models::{KIMI_K2, OPENROUTER_BASE_URL};
use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use langchain_rust::chain::{Chain, LLMChainBuilder};
use langchain_rust::prompt::HumanMessagePromptTemplate;
use langchain_rust::schemas::Message;
use langchain_rust::{
    fmt_message, fmt_template,
    language_models::llm::LLM,
    llm::openai::{OpenAI, OpenAIConfig},
    message_formatter, prompt_args, template_fstring,
};
use tauri::{AppHandle, Emitter};

pub struct LangChainChatBot {
    llm: OpenAI<OpenAIConfig>,
}

impl LangChainChatBot {
    pub fn new() -> Result<Self> {
        let key = env::var("OPENROUTER_API_KEY")
            .map_err(|e| anyhow!("OPENROUTER_API_KEY environment variable not found. Error: {}. Please ensure it's set and exported in your shell.", e))?;

        let llm = OpenAI::default()
            .with_config(
                OpenAIConfig::new()
                    .with_api_base(OPENROUTER_BASE_URL)
                    .with_api_key(key),
            )
            .with_model(KIMI_K2);

        Ok(Self { llm })
    }

    pub async fn ask_orbit(&self, question: &str) -> Result<String> {
        let prompt = format!(
            "You are Orbit, a helpful assistant. User: {}\nOrbit:",
            question
        );

        self.llm
            .invoke(&prompt)
            .await
            .map_err(|e| anyhow!("LLM invocation failed: {}", e))
    }
    pub async fn ask_orbit_stream(&self, question: &str, app_handle: AppHandle) -> Result<()> {
        let prompt = message_formatter![
            fmt_message!(Message::new_system_message(
                "You are Orbit, a helpful assistant.",
            )),
            fmt_template!(HumanMessagePromptTemplate::new(template_fstring!(
                "{input}", "input"
            )))
        ];

        let chain = LLMChainBuilder::new()
            .prompt(prompt)
            .llm(self.llm.clone())
            .build()
            .map_err(|e| anyhow!("Failed to build chain: {:?}", e))?;

        let mut stream = chain
            .stream(prompt_args! {
                "input" => question
            })
            .await
            .map_err(|e| anyhow!("Failed to start stream: {:?}", e))?;

        while let Some(result) = stream.next().await {
            match result {
                Ok(value) => {
                    let chunk_text = format!("{}", value.content);

                    app_handle
                        .emit("stream_chunk", chunk_text)
                        .map_err(|e| anyhow!("Failed to emit chunk: {}", e))?;
                }
                Err(e) => {
                    app_handle
                        .emit("stream_error", format!("Stream error: {:?}", e))
                        .map_err(|e| anyhow!("Failed to emit error: {}", e))?;
                    return Err(anyhow!("Streaming failed: {:?}", e));
                }
            }
        }

        app_handle
            .emit("stream_done", ())
            .map_err(|e| anyhow!("Failed to emit done: {}", e))?;

        Ok(())
    }
}
