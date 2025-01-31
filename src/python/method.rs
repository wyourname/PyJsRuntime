use pyo3::prelude::*;
use super::class::JsRuntime;
use crate::engine::v8engine::{PyContext, JsEngine};

#[pymethods]
impl JsRuntime {
    #[new]
    fn new(py: Python) -> PyResult<Self> {
        Ok(Self {
            // 创建Python对象而不是纯Rust对象
            engine: Py::new(py, JsEngine::new())?,
        })
    }

    fn eval(&self, py: Python<'_>, code: String) -> PyResult<PyObject> {
        // 通过Python对象调用方法
        self.engine.borrow(py).eval(py, code)
    }

    fn compile(&self, py: Python<'_>, code: String) -> PyResult<PyContext> {
        let engine_ref = self.engine.borrow(py);
        engine_ref.compile(py, code)
    }
}