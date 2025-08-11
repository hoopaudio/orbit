use std::env;
use std::sync::Arc;

use crate::services::chatbot::models::{KIMI_K2, OPENROUTER_BASE_URL};
use crate::services::chatbot::tools::ScreenshotTool;
use anyhow::{anyhow, Result};
use langchain_rust::agent::{AgentExecutor, OpenAiToolAgent, OpenAiToolAgentBuilder};
use langchain_rust::chain::{options::ChainCallOptions, Chain};
use langchain_rust::llm::openai::{OpenAI, OpenAIConfig};
use langchain_rust::memory::SimpleMemory;
use langchain_rust::prompt_args;
use tauri::{AppHandle, Emitter};

pub struct LangChainChatBot {
    agent_executor: AgentExecutor<OpenAiToolAgent>,
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

        let memory = SimpleMemory::new();
        let screenshot_tool = ScreenshotTool::new();

        let agent = OpenAiToolAgentBuilder::new()
            .tools(&[Arc::new(screenshot_tool)])
            .options(ChainCallOptions::new().with_max_tokens(2000))
            .build(llm)
            .map_err(|e| anyhow!("Failed to build agent: {:?}", e))?;

        let agent_executor = AgentExecutor::from_agent(agent).with_memory(memory.into());

        Ok(Self { agent_executor })
    }

    pub async fn ask_orbit(&self, question: &str) -> Result<String> {
        let input_variables = prompt_args! {
            "input" => question
        };

        self.agent_executor
            .invoke(input_variables)
            .await
            .map_err(|e| anyhow!("Agent invocation failed: {:?}", e))
    }
    pub async fn ask_orbit_stream(&self, question: &str, app_handle: AppHandle) -> Result<()> {
        let input_variables = prompt_args! {
            "input" => question
        };

        let response = self
            .agent_executor
            .invoke(input_variables)
            .await
            .map_err(|e| anyhow!("Agent execution failed: {:?}", e))?;

        app_handle
            .emit("stream_chunk", response)
            .map_err(|e| anyhow!("Failed to emit response: {}", e))?;

        app_handle
            .emit("stream_done", ())
            .map_err(|e| anyhow!("Failed to emit done: {}", e))?;

        Ok(())
    }
}
