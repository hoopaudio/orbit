use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyModule};

#[pyclass]
pub struct OrbitAgent {
    py_agent: PyObject,
}

impl OrbitAgent {
    pub fn new() -> Result<Self> {
        Python::with_gil(|py| {
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;

            // Get the directory where our Python module is located
            // CARGO_MANIFEST_DIR is crates/orbit-ai, so we go up to root then into packages
            let module_dir = concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../packages/orbit-agent/src"
            );
            path.call_method1("append", (module_dir,))?;

            // Import our Python OrbitAgent module
            let module = PyModule::import(py, "orbit_agent")?;
            let agent_class = module.getattr("OrbitAgent")?;
            let context_class = module.getattr("OrbitContext")?;

            // Create OrbitContext with default settings
            let context = context_class.call0()?;

            // Create an instance of the OrbitAgent
            let agent = agent_class.call1((context,))?;

            Ok(Self {
                py_agent: agent.into(),
            })
        })
        .map_err(|e: PyErr| anyhow::anyhow!("Failed to create OrbitAgent: {}", e))
    }

    /// Stream a response from the agent (simplified - collects all chunks)
    pub async fn stream(&self, message: &str, thread_id: Option<&str>) -> Result<Vec<String>> {
        let thread_id = thread_id.unwrap_or("default");

        Python::with_gil(|py| {
            // Create event loop and run the async Python function
            let asyncio = py.import("asyncio")?;
            let loop_obj = asyncio.call_method0("new_event_loop")?;
            asyncio.call_method1("set_event_loop", (loop_obj,))?;

            // Create kwargs for thread_id
            let kwargs = [("thread_id", thread_id.to_object(py))].into_py_dict(py);

            let stream_coro = self
                .py_agent
                .call_method(py, "stream", (message,), Some(kwargs))?;

            // Collect all chunks from the Python stream
            let async_fn = py.eval(
                r#"
async def collect_chunks(stream_coro):
    chunks = []
    async for chunk in stream_coro:
        chunks.append(chunk)
    return chunks
                "#,
                None,
                None,
            )?;

            let collect_coro = async_fn.call1((stream_coro,))?;
            let chunks = loop_obj.call_method1("run_until_complete", (collect_coro,))?;

            chunks.extract::<Vec<String>>()
        })
        .map_err(|e: PyErr| anyhow::anyhow!("Failed to stream: {}", e))
    }

    /// Run the agent and return the complete response
    pub async fn run(&self, message: &str, thread_id: Option<&str>) -> Result<String> {
        let thread_id = thread_id.unwrap_or("default");

        Python::with_gil(|py| {
            // Create event loop and run the async Python function
            let asyncio = py.import("asyncio")?;
            let loop_obj = asyncio.call_method0("new_event_loop")?;
            asyncio.call_method1("set_event_loop", (loop_obj,))?;

            // Create kwargs for thread_id
            let kwargs = [("thread_id", thread_id.to_object(py))].into_py_dict(py);

            let coro = self
                .py_agent
                .call_method(py, "run", (message,), Some(kwargs))?;
            let result = loop_obj.call_method1("run_until_complete", (coro,))?;

            result.extract::<String>()
        })
        .map_err(|e: PyErr| anyhow::anyhow!("Failed to run agent: {}", e))
    }
}
