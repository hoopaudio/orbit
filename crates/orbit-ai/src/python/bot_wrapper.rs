use anyhow::{anyhow, Result};
use pyo3::prelude::*;
use pyo3::types::PyModule;

/// A simple wrapper for calling Python bot from Rust/Tauri
/// This replaces the need for app_handle
pub struct PythonBotWrapper {
    py_bot: PyObject,
}

impl PythonBotWrapper {
    pub fn new() -> Result<Self> {
        Python::with_gil(|py| {
            // Add the python directory to sys.path
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;

            // Use absolute path to the Python module
            let module_dir = "/Users/cuthlehoop/projects/orbit/crates/orbit-ai/python";
            path.call_method1("append", (module_dir,))?;

            // Also add parent directory in case module is there
            path.call_method1("append", ("/Users/cuthlehoop/projects/orbit/crates/orbit-ai",))?;

            // Import our Python bot module
            let module = PyModule::import(py, "orbit_ai.dummy_bot")?;
            let bot_class = module.getattr("DummyLangChainBot")?;

            let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();
            let bot = bot_class.call1((api_key,))?;

            Ok(Self {
                py_bot: bot.into(),
            })
        })
        .map_err(|e: PyErr| anyhow!("Failed to create Python bot: {}", e))
    }

    pub async fn ask_orbit(&self, question: &str) -> Result<String> {
        let question = question.to_string();
        let py_bot = self.py_bot.clone();

        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let asyncio = py.import("asyncio")?;
                let loop_obj = asyncio.call_method0("new_event_loop")?;
                asyncio.call_method1("set_event_loop", (loop_obj,))?;

                let coro = py_bot.call_method1(py, "ask", (question,))?;
                let result = loop_obj.call_method1("run_until_complete", (coro,))?;

                result.extract::<String>()
            })
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))?
        .map_err(|e: PyErr| anyhow!("Python error: {}", e))?;

        Ok(result)
    }

    pub async fn ask_orbit_with_image(&self, question: &str, image_path: &str) -> Result<String> {
        let question = question.to_string();
        let image_path = image_path.to_string();
        let py_bot = self.py_bot.clone();

        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let asyncio = py.import("asyncio")?;
                let loop_obj = asyncio.call_method0("new_event_loop")?;
                asyncio.call_method1("set_event_loop", (loop_obj,))?;

                let coro = py_bot.call_method1(py, "ask_with_image", (question, image_path))?;
                let result = loop_obj.call_method1("run_until_complete", (coro,))?;

                result.extract::<String>()
            })
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))?
        .map_err(|e: PyErr| anyhow!("Python error: {}", e))?;

        Ok(result)
    }
}