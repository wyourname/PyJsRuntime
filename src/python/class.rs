use pyo3::prelude::*;
use crate::engine::v8engine::JsEngine;

#[pyclass(name = "JsRuntime", unsendable)]
pub struct JsRuntime {
    pub engine: Py<JsEngine>,
}