use anyhow::{anyhow, Result};
use pyo3::prelude::*;
use pyo3::types::PyModule;
use tauri::{AppHandle, Emitter};

/// A wrapper for calling Python bot from Rust/Tauri
/// Uses a singleton pattern on the Python side to maintain state
pub struct PythonBotWrapper {
    // We don't store the bot here anymore - it lives in Python
}

impl PythonBotWrapper {
    pub fn new() -> Result<Self> {
        // This is now lightweight - just ensures Python paths are set
        // The actual bot initialization happens in the singleton manager
        Python::with_gil(|py| {
            // Only set up paths if not already done
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;

            #[cfg(debug_assertions)]
            let module_paths = {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                // In dev, CWD is likely .../crates/orbit-app, so project root is parent of parent.
                let project_root = if cwd.ends_with("crates/orbit-app") {
                    cwd.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap_or(cwd.clone())
                } else {
                    cwd.clone()
                };
                vec![
                    project_root.join("crates/orbit-ai/python").to_string_lossy().to_string(),
                    project_root.join("crates/orbit-ai").to_string_lossy().to_string(),
                ]
            };

            #[cfg(not(debug_assertions))]
            let module_paths = {
                let exe_path =
                    std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
                vec![
                    exe_dir.join("python").to_string_lossy().to_string(),
                    exe_dir.to_string_lossy().to_string(),
                ]
            };

            // Check if path already contains our module dirs
            let path_list: Vec<String> = path.extract()?;
            for module_path in &module_paths {
                if !path_list
                    .iter()
                    .any(|p| p.contains("orbit-ai/python") || p.ends_with(module_path))
                {
                    path.call_method1("append", (module_path,))?;
                }
            }

            Ok::<_, PyErr>(())
        })
        .map_err(|e: PyErr| anyhow!("Failed to set Python paths: {}", e))?;

        Ok(Self {})
    }

    pub fn initialize() -> Result<()> {
        // This method actually initializes the singleton
        Python::with_gil(|py| {
            // Set up paths
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;

            // For development, use relative paths from project root
            // For production, use paths relative to the executable
            #[cfg(debug_assertions)]
            let module_paths = {
                let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                // In dev, CWD is likely .../crates/orbit-app, so project root is parent of parent.
                let project_root = if cwd.ends_with("crates/orbit-app") {
                    cwd.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf()).unwrap_or(cwd.clone())
                } else {
                    cwd.clone()
                };
                vec![
                    project_root.join("crates/orbit-ai/python").to_string_lossy().to_string(),
                    project_root.join("crates/orbit-ai").to_string_lossy().to_string(),
                ]
            };

            #[cfg(not(debug_assertions))]
            let module_paths = {
                let exe_path =
                    std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
                vec![
                    exe_dir.join("python").to_string_lossy().to_string(),
                    exe_dir.to_string_lossy().to_string(),
                ]
            };

            for module_path in &module_paths {
                path.call_method1("append", (module_path,))?;
            }

            // Import and initialize singleton
            let singleton_module = PyModule::import(py, "orbit_ai.singleton_manager")?;
            let get_instance = singleton_module.getattr("get_bot_instance")?;
            let get_loop = singleton_module.getattr("get_event_loop")?;

            let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();

            // Create the singleton instance
            let _ = get_instance.call1((api_key,))?;
            let _ = get_loop.call0()?;

            Ok::<_, PyErr>(())
        })
        .map_err(|e: PyErr| anyhow!("Failed to initialize Python bot singleton: {}", e))
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

                // Run it on the persistent event loop with timeout protection
                match loop_obj.call_method1("run_until_complete", (coro,)) {
                    Ok(result) => result.extract::<String>(),
                    Err(e) => {
                        eprintln!("Python ask error: {}", e);
                        if let Ok(exception_type) = e.get_type(py).name() {
                            if exception_type.contains("TimeoutError") || exception_type.contains("asyncio") {
                                Err(PyErr::new::<pyo3::exceptions::PyTimeoutError, _>("LangGraph agent timed out"))
                            } else {
                                Err(e)
                            }
                        } else {
                            Err(e)
                        }
                    }
                }
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

                // Run it on the persistent event loop with timeout protection
                match loop_obj.call_method1("run_until_complete", (coro,)) {
                    Ok(result) => result.extract::<String>(),
                    Err(e) => {
                        eprintln!("Python ask_with_image error: {}", e);
                        if let Ok(exception_type) = e.get_type(py).name() {
                            if exception_type.contains("TimeoutError") || exception_type.contains("asyncio") {
                                Err(PyErr::new::<pyo3::exceptions::PyTimeoutError, _>("LangGraph agent timed out"))
                            } else {
                                Err(e)
                            }
                        } else {
                            Err(e)
                        }
                    }
                }
            })
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))?
        .map_err(|e: PyErr| anyhow!("Python error: {}", e))?;

        Ok(result)
    }

    pub async fn ask_orbit_stream(&self, question: &str, app_handle: AppHandle) -> Result<()> {
        let question = question.to_string();

        tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| {
                // Import singleton manager
                let singleton_module = PyModule::import(py, "orbit_ai.singleton_manager")?;

                // Get the persistent event loop
                let get_loop = singleton_module.getattr("get_event_loop")?;
                let loop_obj = get_loop.call0()?;

                // Get the streaming async function
                let ask_stream_async = singleton_module.getattr("ask_bot_stream_async")?;
                let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();

                // Create the coroutine (generator)
                let coro = ask_stream_async.call1((question, api_key))?;

                // Iterate through the stream
                loop {
                    // Get the next chunk from the async generator
                    let anext = py.eval("anext", None, None)?;
                    let chunk_future = anext.call1((coro,))?;

                    match loop_obj.call_method1("run_until_complete", (chunk_future,)) {
                        Ok(chunk) => {
                            let text: String = chunk.extract()?;
                            // Emit the chunk through Tauri
                            let _ = app_handle.emit("stream_chunk", text);
                        }
                        Err(e) => {
                            // Check if this is a StopAsyncIteration (normal end) or a real error
                            if let Ok(exception_type) = e.get_type(py).name() {
                                if exception_type == "StopAsyncIteration" {
                                    // Normal end of stream
                                    break;
                                } else {
                                    // Real Python exception - log it and emit error
                                    eprintln!("Python streaming error: {} - {}", exception_type, e);
                                    let error_msg = format!("Streaming error: {}", e);
                                    let _ = app_handle.emit("stream_error", error_msg);
                                    break;
                                }
                            } else {
                                // Unknown error type - treat as end of stream but log it
                                eprintln!("Unknown Python error during streaming: {}", e);
                                let _ = app_handle.emit("stream_error", format!("Unknown error: {}", e));
                                break;
                            }
                        }
                    }
                }

                // Emit stream done event
                let _ = app_handle.emit("stream_done", ());
                Ok::<_, PyErr>(())
            })
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))?
        .map_err(|e: PyErr| anyhow!("Python streaming error: {}", e))?;

        Ok(())
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
