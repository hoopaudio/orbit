use std::env;
use std::io::{stdout, Write};

use crate::services::chatbot::models::{DEEPSEEK_R1, OPENROUTER_BASE_URL};
use crate::services::chatbot::service::ChatBotTrait;
use anyhow::{anyhow, Result};
use futures_util::StreamExt;
use langchain_rust::llm::OpenAIConfig;
use langchain_rust::{
    chain::{builder::ConversationalChainBuilder, Chain},
    llm::openai::{OpenAI, OpenAIModel},
    memory::SimpleMemory,
    prompt_args,
};

pub struct LangChainChatBot {
    chain: Box<dyn Chain>,
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
            .with_model(DEEPSEEK_R1);

        let memory = SimpleMemory::new();

        let chain = ConversationalChainBuilder::new()
            .llm(llm)
            .memory(memory.into())
            .build()
            .map_err(|e| anyhow!("Failed to build ConversationalChain: {}", e))?;

        Ok(Self {
            chain: Box::new(chain),
        })
    }
}

impl ChatBotTrait for LangChainChatBot {
    async fn ask_orbit(&self, question: &str) -> Result<String> {
        let input_variables = prompt_args! {
            "input" => question,
        };

        self.chain
            .invoke(input_variables)
            .await
            .map_err(|e| anyhow!("Chain invocation failed: {}", e))
    }
}
