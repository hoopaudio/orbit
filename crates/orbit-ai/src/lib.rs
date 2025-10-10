pub mod langchain;
#[cfg(test)]
pub mod langchain_test;
pub mod models;
pub mod ocr;
pub mod python;
pub mod screenshot;
pub mod service;
pub mod system_prompt;
pub mod tools;

// Re-export commonly used items
pub use langchain::LangChainChatBot;
pub use ocr::OcrService;
pub use python::bot_wrapper::PythonBotWrapper;
pub use screenshot::ScreenshotService;
pub use system_prompt::ORBIT_SYSTEM_PROMPT;

// Re-export PyO3 for other crates to use
pub use pyo3;

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn orbit_ai(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    python::register_python_module(_py, m)?;
    Ok(())
}
