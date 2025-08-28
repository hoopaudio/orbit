use std::env;
use std::sync::Arc;

use crate::models::{
    DEEPSEEK_R1, GEMINI_FLASH_2_FREE, GEMMA_3_27B_IT_FREE, KIMI_K2, KIMI_VL_A3B_THINKING_FREE,
    MISTRAL_SMALL_3DOT2_24B_FREE, OPENROUTER_BASE_URL, QWEN2DOT5_VL_72B_FREE,
};
use crate::system_prompt::ORBIT_SYSTEM_PROMPT;
use crate::tools::ScreenshotTool;
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
use tauri::{AppHandle, Emitter};

const RATE_LIMIT_ERROR: &str = "🚀 You're using Orbit like a pro! You've hit your free usage limit.\n\nUpgrade to Orbit Pro for unlimited AI conversations, faster responses, and premium features.\n\nTry again in a few minutes or upgrade now at orbit.app/pro";
const API_PARSE_ERROR_PREFIX: &str = "API response parsing error: ";
const AGENT_BUILD_ERROR_PREFIX: &str = "Failed to build agent: ";
const AGENT_INVOCATION_ERROR_PREFIX: &str = "Agent invocation failed: ";
const AGENT_EXECUTION_ERROR_PREFIX: &str = "Agent execution failed: ";

const FALLBACK_MODELS: &[&str] = &[
    GEMINI_FLASH_2_FREE,
    DEEPSEEK_R1,
    KIMI_K2,
    MISTRAL_SMALL_3DOT2_24B_FREE,
    QWEN2DOT5_VL_72B_FREE,
    GEMMA_3_27B_IT_FREE,
    KIMI_VL_A3B_THINKING_FREE,
];

pub struct LangChainChatBot {
    agent_executor: AgentExecutor<OpenAiToolAgent>,
    streaming_agent: AgentExecutor<ConversationalAgent>,
}

impl LangChainChatBot {
    fn create_llm_with_model(model: &str) -> Result<OpenAI<OpenAIConfig>> {
        let key = env::var("OPENROUTER_API_KEY")
            .map_err(|e| anyhow!("OPENROUTER_API_KEY environment variable not found. Error: {}. Please ensure it's set and exported in your shell.", e))?;

        let llm = OpenAI::default()
            .with_config(
                OpenAIConfig::new()
                    .with_api_base(OPENROUTER_BASE_URL)
                    .with_api_key(key),
            )
            .with_model(model);

        Ok(llm)
    }

    pub fn new<R: tauri::Runtime>(app_handle: AppHandle<R>) -> Result<Self> {
        let llm = Self::create_llm_with_model(GEMINI_FLASH_2_FREE)?;

        let memory = SimpleMemory::new();
        let shared_memory = Arc::new(tokio::sync::Mutex::new(memory));
        let screenshot_tool = ScreenshotTool::new(app_handle.clone());

        let agent = OpenAiToolAgentBuilder::new()
            .tools(&[Arc::new(screenshot_tool)])
            .options(ChainCallOptions::new().with_max_tokens(2000))
            .build(llm.clone())
            .map_err(|e| anyhow!("{}{:?}", AGENT_BUILD_ERROR_PREFIX, e))?;

        let agent_executor = AgentExecutor::from_agent(agent).with_memory(shared_memory.clone());

        // Create streaming-capable conversational agent
        let streaming_agent = ConversationalAgentBuilder::new()
            .tools(&[Arc::new(ScreenshotTool::new(app_handle))])
            .options(ChainCallOptions::new().with_max_tokens(2000))
            .build(llm.clone())
            .map_err(|e| anyhow!("{}streaming agent: {:?}", AGENT_BUILD_ERROR_PREFIX, e))?;

        let streaming_executor =
            AgentExecutor::from_agent(streaming_agent).with_memory(shared_memory);

        Ok(Self {
            agent_executor,
            streaming_agent: streaming_executor,
        })
    }

    #[cfg(test)]
    pub fn new_for_test() -> Result<Self> {
        let llm = Self::create_llm_with_model(GEMINI_FLASH_2_FREE)?;

        let memory = SimpleMemory::new();
        let shared_memory = Arc::new(tokio::sync::Mutex::new(memory));

        // Create agents without tools for tests (no AppHandle needed)
        let agent = OpenAiToolAgentBuilder::new()
            .tools(&[])
            .options(ChainCallOptions::new().with_max_tokens(2000))
            .build(llm.clone())
            .map_err(|e| anyhow!("{}{:?}", AGENT_BUILD_ERROR_PREFIX, e))?;

        let agent_executor = AgentExecutor::from_agent(agent).with_memory(shared_memory.clone());

        let streaming_agent = ConversationalAgentBuilder::new()
            .tools(&[])
            .options(ChainCallOptions::new().with_max_tokens(2000))
            .build(llm.clone())
            .map_err(|e| anyhow!("{}streaming agent: {:?}", AGENT_BUILD_ERROR_PREFIX, e))?;

        let streaming_executor =
            AgentExecutor::from_agent(streaming_agent).with_memory(shared_memory);

        Ok(Self {
            agent_executor,
            streaming_agent: streaming_executor,
        })
    }

    pub async fn ask_orbit(&self, question: &str) -> Result<String> {
        let input_with_system_prompt = format!("{}\n\nUser: {}", ORBIT_SYSTEM_PROMPT, question);

        for (i, model) in FALLBACK_MODELS.iter().enumerate() {
            let llm = Self::create_llm_with_model(model)?;
            let memory = SimpleMemory::new();
            let shared_memory = Arc::new(tokio::sync::Mutex::new(memory));

            // Create agent with the current model
            let agent = OpenAiToolAgentBuilder::new()
                .tools(&[]) // No tools for simplicity in fallback
                .options(ChainCallOptions::new().with_max_tokens(2000))
                .build(llm)
                .map_err(|e| anyhow!("{}{:?}", AGENT_BUILD_ERROR_PREFIX, e))?;

            let executor = AgentExecutor::from_agent(agent).with_memory(shared_memory);

            let input_variables = prompt_args! {
                "input" => input_with_system_prompt.clone()
            };

            match executor.invoke(input_variables).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let error_str = format!("{:?}", e);
                    let is_rate_limited = error_str.contains("429")
                        || error_str.contains(
                            "failed to deserialize api response: invalid type: integer `429`",
                        );

                    log::error!("Model {} failed: {}", model, error_str);

                    if is_rate_limited {
                        log::warn!("Rate limited on model {}, trying next model", model);
                        if i < FALLBACK_MODELS.len() - 1 {
                            // Rate limited and more models available, try next model
                            continue;
                        } else {
                            // Rate limited and no more models to try
                            log::error!("All models rate limited, no more fallbacks available");
                            return Err(anyhow!(RATE_LIMIT_ERROR));
                        }
                    } else {
                        // Not rate limited - return the actual error immediately
                        log::error!("Non-rate-limit error on model {}: {}", model, error_str);
                        if error_str.contains("deserialize") && error_str.contains("integer") {
                            return Err(anyhow!("{}{}", API_PARSE_ERROR_PREFIX, error_str));
                        } else {
                            return Err(anyhow!("{}{}", AGENT_INVOCATION_ERROR_PREFIX, error_str));
                        }
                    }
                }
            }
        }

        Err(anyhow!(RATE_LIMIT_ERROR))
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
                if error_str.contains("429")
                    || error_str
                        .contains("failed to deserialize api response: invalid type: integer `429`")
                {
                    anyhow!(RATE_LIMIT_ERROR)
                } else if error_str.contains("deserialize") && error_str.contains("integer") {
                    anyhow!("{}{}", API_PARSE_ERROR_PREFIX, error_str)
                } else {
                    anyhow!("{}{}", AGENT_EXECUTION_ERROR_PREFIX, error_str)
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
                if error_str.contains("429")
                    || error_str
                        .contains("failed to deserialize api response: invalid type: integer `429`")
                {
                    anyhow!(RATE_LIMIT_ERROR)
                } else if error_str.contains("deserialize") && error_str.contains("integer") {
                    anyhow!("{}{}", API_PARSE_ERROR_PREFIX, error_str)
                } else {
                    anyhow!("{}{}", AGENT_EXECUTION_ERROR_PREFIX, error_str)
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
                if error_str.contains("429")
                    || error_str
                        .contains("failed to deserialize api response: invalid type: integer `429`")
                {
                    anyhow!(RATE_LIMIT_ERROR)
                } else if error_str.contains("deserialize") && error_str.contains("integer") {
                    anyhow!("{}{}", API_PARSE_ERROR_PREFIX, error_str)
                } else {
                    anyhow!("{}{}", AGENT_INVOCATION_ERROR_PREFIX, error_str)
                }
            })
    }
}
