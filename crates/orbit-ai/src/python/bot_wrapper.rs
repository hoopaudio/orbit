use anyhow::{anyhow, Result};
use pyo3::prelude::*;
use pyo3::types::PyModule;

/// A wrapper for calling Python bot from Rust/Tauri
/// Uses a singleton pattern on the Python side to maintain state
pub struct PythonBotWrapper {
    // We don't store the bot here anymore - it lives in Python
}

impl PythonBotWrapper {
    pub fn new() -> Result<Self> {
        // Initialize the Python singleton if needed
        Python::with_gil(|py| {
            // Add the python directory to sys.path
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;

            // Use absolute path to the Python module
            let module_dir = "/Users/cuthlehoop/projects/orbit/crates/orbit-ai/python";
            path.call_method1("append", (module_dir,))?;

            // Also add parent directory in case module is there
            path.call_method1("append", ("/Users/cuthlehoop/projects/orbit/crates/orbit-ai",))?;

            // Import our singleton manager
            let singleton_module = PyModule::import(py, "orbit_ai.singleton_manager")?;
            let get_instance = singleton_module.getattr("get_bot_instance")?;
            let get_loop = singleton_module.getattr("get_event_loop")?;

            let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();

            // This will either create or reuse the existing bot instance
            let _ = get_instance.call1((api_key,))?;

            // Also initialize the event loop
            let _ = get_loop.call0()?;

            Ok::<_, PyErr>(())
        })
        .map_err(|e: PyErr| anyhow!("Failed to initialize Python bot singleton: {}", e))?;

        Ok(Self {})
    }

    pub async fn ask_orbit(&self, question: &str) -> Result<String> {
        let question = question.to_string();

        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| {
                // Import singleton manager
                let singleton_module = PyModule::import(py, "orbit_ai.singleton_manager")?;

                // Get the persistent event loop
                let get_loop = singleton_module.getattr("get_event_loop")?;
                let loop_obj = get_loop.call0()?;

                // Get the async function
                let ask_async = singleton_module.getattr("ask_bot_async")?;
                let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();

                // Create the coroutine
                let coro = ask_async.call1((question, api_key))?;

                // Run it on the persistent event loop
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

        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| {
                // Import singleton manager
                let singleton_module = PyModule::import(py, "orbit_ai.singleton_manager")?;

                // Get the persistent event loop
                let get_loop = singleton_module.getattr("get_event_loop")?;
                let loop_obj = get_loop.call0()?;

                // Get the async function
                let ask_async = singleton_module.getattr("ask_bot_with_image_async")?;
                let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();

                // Create the coroutine
                let coro = ask_async.call1((question, image_path, api_key))?;

                // Run it on the persistent event loop
                let result = loop_obj.call_method1("run_until_complete", (coro,))?;

                result.extract::<String>()
            })
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))?
        .map_err(|e: PyErr| anyhow!("Python error: {}", e))?;

        Ok(result)
    }

    pub async fn clear_memory(&self) -> Result<String> {
        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| {
                // Reset the singleton instance
                let singleton_module = PyModule::import(py, "orbit_ai.singleton_manager")?;
                let reset_instance = singleton_module.getattr("reset_bot_instance")?;
                reset_instance.call0()?;

                Ok::<_, PyErr>("Memory cleared and bot reset".to_string())
            })
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))?
        .map_err(|e: PyErr| anyhow!("Python error: {}", e))?;

        Ok(result)
    }
}