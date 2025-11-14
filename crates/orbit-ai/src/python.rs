use pyo3::prelude::*;
use crate::langchain_py;

pub fn register_python_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    langchain_py::register_langchain_module(_py, m)?;
    Ok(())
}