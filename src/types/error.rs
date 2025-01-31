use std::fmt;
use pyo3::prelude::*;

#[derive(Debug)]
pub enum TypeConversionError {
    UnsupportedType(String),
    InvalidValue(String),
    SerializationError(String),
    DeserializationError(String),
}

#[derive(Debug)]
pub enum JsError {
    RuntimeError(String),
    ExecutionError(String),
    JsonError(String),
}

impl fmt::Display for TypeConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedType(msg) => write!(f, "Unsupported type: {}", msg),
            Self::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
        }
    }
}

impl fmt::Display for JsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            Self::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            Self::JsonError(msg) => write!(f, "JSON error: {}", msg),
        }
    }
}

impl std::error::Error for TypeConversionError {}
impl std::error::Error for JsError {}

impl From<JsError> for PyErr {
    fn from(err: JsError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.to_string())
    }
}

impl From<TypeConversionError> for PyErr {
    fn from(err: TypeConversionError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(err.to_string())
    }
} 