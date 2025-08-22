use std::env;
use std::sync::Arc;

use crate::services::chatbot::models::{GEMINI_FLASH_2_FREE, OPENROUTER_BASE_URL};
use crate::services::chatbot::system_prompt::ORBIT_SYSTEM_PROMPT;
use crate::services::chatbot::tools::ScreenshotTool;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use langchain_rust::agent::{
    AgentExecutor, ConversationalAgent, ConversationalAgentBuilder, OpenAiToolAgent,
    OpenAiToolAgentBuilder,
};
use langchain_rust::chain::{options::ChainCallOptions, Chain};
use langchain_rust::llm::openai::{OpenAI, OpenAIConfig};
use langchain_rust::memory::SimpleMemory;
use langchain_rust::prompt_args;
use langchain_rust::schemas::BaseMemory;
use tauri::{AppHandle, Emitter};

pub struct LangChainChatBot {
    agent_executor: AgentExecutor<OpenAiToolAgent>,
    streaming_agent: AgentExecutor<ConversationalAgent>,
}

impl LangChainChatBot {
    pub fn new<R: tauri::Runtime>(app_handle: AppHandle<R>) -> Result<Self> {
        let key = env::var("OPENROUTER_API_KEY")
            .map_err(|e| anyhow!("OPENROUTER_API_KEY environment variable not found. Error: {}. Please ensure it's set and exported in your shell.", e))?;

        let llm = OpenAI::default()
            .with_config(
                OpenAIConfig::new()
                    .with_api_base(OPENROUTER_BASE_URL)
                    .with_api_key(key),
            )
            .with_model(GEMINI_FLASH_2_FREE);

        let memory = SimpleMemory::new();
        let shared_memory = Arc::new(tokio::sync::Mutex::new(memory));
        let screenshot_tool = ScreenshotTool::new(app_handle.clone());

        let agent = OpenAiToolAgentBuilder::new()
            .tools(&[Arc::new(screenshot_tool)])
            .options(ChainCallOptions::new().with_max_tokens(2000))
            .build(llm.clone())
            .map_err(|e| anyhow!("Failed to build agent: {:?}", e))?;

        let agent_executor = AgentExecutor::from_agent(agent).with_memory(shared_memory.clone());

        // Create streaming-capable conversational agent
        let streaming_agent = ConversationalAgentBuilder::new()
            .tools(&[Arc::new(ScreenshotTool::new(app_handle))])
            .options(ChainCallOptions::new().with_max_tokens(2000))
            .build(llm.clone())
            .map_err(|e| anyhow!("Failed to build streaming agent: {:?}", e))?;

        let streaming_executor =
            AgentExecutor::from_agent(streaming_agent).with_memory(shared_memory);

        Ok(Self {
            agent_executor,
            streaming_agent: streaming_executor,
        })
    }

    #[cfg(test)]
    pub fn new_for_test() -> Result<Self> {
        let key = env::var("OPENROUTER_API_KEY")
            .map_err(|e| anyhow!("OPENROUTER_API_KEY environment variable not found. Error: {}. Please ensure it's set and exported in your shell.", e))?;

        let llm = OpenAI::default()
            .with_config(
                OpenAIConfig::new()
                    .with_api_base(OPENROUTER_BASE_URL)
                    .with_api_key(key),
            )
            .with_model(GEMINI_FLASH_2_FREE);

        let memory = SimpleMemory::new();
        let shared_memory = Arc::new(tokio::sync::Mutex::new(memory));

        // Create agents without tools for tests (no AppHandle needed)
        let agent = OpenAiToolAgentBuilder::new()
            .tools(&[])
            .options(ChainCallOptions::new().with_max_tokens(2000))
            .build(llm.clone())
            .map_err(|e| anyhow!("Failed to build agent: {:?}", e))?;

        let agent_executor = AgentExecutor::from_agent(agent).with_memory(shared_memory.clone());

        let streaming_agent = ConversationalAgentBuilder::new()
            .tools(&[])
            .options(ChainCallOptions::new().with_max_tokens(2000))
            .build(llm.clone())
            .map_err(|e| anyhow!("Failed to build streaming agent: {:?}", e))?;
        
        let streaming_executor = AgentExecutor::from_agent(streaming_agent).with_memory(shared_memory);

        Ok(Self {
            agent_executor,
            streaming_agent: streaming_executor,
        })
    }

    pub async fn ask_orbit(&self, question: &str) -> Result<String> {
        let input_with_system_prompt = format!("{}\n\nUser: {}", ORBIT_SYSTEM_PROMPT, question);
        let input_variables = prompt_args! {
            "input" => input_with_system_prompt
        };

        self.agent_executor
            .invoke(input_variables)
            .await
            .map_err(|e| {
                let error_str = format!("{:?}", e);
                if error_str.contains("429") || error_str.contains("failed to deserialize api response: invalid type: integer `429`") {
                    anyhow!("🚀 You're using Orbit like a pro! You've hit your free usage limit.\n\nUpgrade to Orbit Pro for unlimited AI conversations, faster responses, and premium features.\n\nTry again in a few minutes or upgrade now at orbit.app/pro")
                } else if error_str.contains("deserialize") && error_str.contains("integer") {
                    anyhow!("API response parsing error: {}", error_str)
                } else {
                    anyhow!("Agent invocation failed: {}", error_str)
                }
            })
    }

    pub async fn ask_orbit_stream(&self, question: &str, app_handle: AppHandle) -> Result<()> {
        let input_with_system_prompt = format!("{}\n\nUser: {}", ORBIT_SYSTEM_PROMPT, question);
        let input_variables = prompt_args! {
            "input" => input_with_system_prompt
        };

        let response = self
            .agent_executor
            .invoke(input_variables)
            .await
            .map_err(|e| {
                let error_str = format!("{:?}", e);
                if error_str.contains("429") || error_str.contains("failed to deserialize api response: invalid type: integer `429`") {
                    anyhow!("🚀 You're using Orbit like a pro! You've hit your free usage limit.\n\nUpgrade to Orbit Pro for unlimited AI conversations, faster responses, and premium features.\n\nTry again in a few minutes or upgrade now at orbit.app/pro")
                } else if error_str.contains("deserialize") && error_str.contains("integer") {
                    anyhow!("API response parsing error: {}", error_str)
                } else {
                    anyhow!("Agent execution failed: {}", error_str)
                }
            })?;

        app_handle
            .emit("stream_chunk", response)
            .map_err(|e| anyhow!("Failed to emit response: {}", e))?;

        app_handle
            .emit("stream_done", ())
            .map_err(|e| anyhow!("Failed to emit done: {}", e))?;

        Ok(())
    }

    pub async fn ask_orbit_stream_conversational(
        &self,
        question: &str,
        app_handle: AppHandle,
    ) -> Result<()> {
        let input_with_system_prompt = format!("{}\n\nUser: {}", ORBIT_SYSTEM_PROMPT, question);
        let input_variables = prompt_args! {
            "input" => input_with_system_prompt
        };

        let response = self
            .streaming_agent
            .invoke(input_variables)
            .await
            .map_err(|e| {
                let error_str = format!("{:?}", e);
                if error_str.contains("429") || error_str.contains("failed to deserialize api response: invalid type: integer `429`") {
                    anyhow!("🚀 You're using Orbit like a pro! You've hit your free usage limit.\n\nUpgrade to Orbit Pro for unlimited AI conversations, faster responses, and premium features.\n\nTry again in a few minutes or upgrade now at orbit.app/pro")
                } else if error_str.contains("deserialize") && error_str.contains("integer") {
                    anyhow!("API response parsing error: {}", error_str)
                } else {
                    anyhow!("Agent execution failed: {}", error_str)
                }
            })?;

        app_handle
            .emit("stream_chunk", response)
            .map_err(|e| anyhow!("Failed to emit response: {}", e))?;

        app_handle
            .emit("stream_done", ())
            .map_err(|e| anyhow!("Failed to emit done: {}", e))?;

        Ok(())
    }

    pub async fn ask_orbit_with_image(&self, question: &str, image_path: &str) -> Result<String> {
        // Read and encode the image
        let image_data =
            std::fs::read(image_path).map_err(|e| anyhow!("Failed to read image file: {}", e))?;
        let image_base64 = BASE64_STANDARD.encode(image_data);

        // Create input with image and system prompt
        let input_with_image = format!(
            "{}\n\nUser: {}\n\nImage: data:image/png;base64,{}",
            ORBIT_SYSTEM_PROMPT, question, image_base64
        );

        let input_variables = prompt_args! {
            "input" => input_with_image
        };

        self.agent_executor
            .invoke(input_variables)
            .await
            .map_err(|e| {
                let error_str = format!("{:?}", e);
                if error_str.contains("429") || error_str.contains("failed to deserialize api response: invalid type: integer `429`") {
                    anyhow!("🚀 You're using Orbit like a pro! You've hit your free usage limit.\n\nUpgrade to Orbit Pro for unlimited AI conversations, faster responses, and premium features.\n\nTry again in a few minutes or upgrade now at orbit.app/pro")
                } else if error_str.contains("deserialize") && error_str.contains("integer") {
                    anyhow!("API response parsing error: {}", error_str)
                } else {
                    anyhow!("Agent invocation failed: {}", error_str)
                }
            })
    }
}
