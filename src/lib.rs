use pyo3::prelude::*;
mod engine;
mod types;
mod python;

/// A Python module implemented in Rust.
#[pymodule]
fn py_js_runtime(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<python::class::JsRuntime>()?;
    // m.add_class::<JsExecutor>()?;
    Ok(())
}
