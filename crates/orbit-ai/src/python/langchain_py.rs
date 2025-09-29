use pyo3::prelude::*;
use pyo3::types::PyModule;

/// A bot that calls Python LangChain implementation directly
/// This doesn't require app_handle and serves as an MVP
#[pyclass]
pub struct PyLangChainBot {
    py_bot: PyObject,
}

#[pymethods]
impl PyLangChainBot {
    #[new]
    fn new() -> PyResult<Self> {
        Python::with_gil(|py| {
            // Add the python directory to sys.path
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;

            // Get the directory where our Python module is located
            let module_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/python");
            path.call_method1("append", (module_dir,))?;

            // Import our Python bot module (using the dummy for MVP)
            let module = PyModule::import(py, "orbit_ai.dummy_bot")?;
            let bot_class = module.getattr("DummyLangChainBot")?;

            // Create an instance of the bot
            let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();
            let bot = bot_class.call1((api_key,))?;

            Ok(Self { py_bot: bot.into() })
        })
    }

    fn ask(&self, py: Python, question: String) -> PyResult<String> {
        // Create event loop and run the async Python function
        let asyncio = py.import("asyncio")?;
        let loop_obj = asyncio.call_method0("new_event_loop")?;
        asyncio.call_method1("set_event_loop", (loop_obj,))?;

        let coro = self.py_bot.call_method1(py, "ask", (question,))?;
        let result = loop_obj.call_method1("run_until_complete", (coro,))?;

        result.extract::<String>()
    }

    fn ask_with_image(
        &self,
        py: Python,
        question: String,
        image_path: String,
    ) -> PyResult<String> {
        let asyncio = py.import("asyncio")?;
        let loop_obj = asyncio.call_method0("new_event_loop")?;
        asyncio.call_method1("set_event_loop", (loop_obj,))?;

        let coro = self.py_bot.call_method1(py, "ask_with_image", (question, image_path))?;
        let result = loop_obj.call_method1("run_until_complete", (coro,))?;

        result.extract::<String>()
    }
}

#[pyfunction]
fn set_openrouter_api_key(api_key: String) -> PyResult<()> {
    std::env::set_var("OPENROUTER_API_KEY", api_key);
    Ok(())
}

#[pyfunction]
fn get_system_prompt() -> PyResult<String> {
    Ok(crate::system_prompt::ORBIT_SYSTEM_PROMPT.to_string())
}

pub fn register_langchain_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyLangChainBot>()?;
    m.add_function(wrap_pyfunction!(set_openrouter_api_key, m)?)?;
    m.add_function(wrap_pyfunction!(get_system_prompt, m)?)?;
    Ok(())
}
