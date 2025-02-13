use deno_core::{v8, JsRuntime, RuntimeOptions};
use pyo3::prelude::*;
use parking_lot::RwLock;
use pyo3::types::PyTuple;
use std::collections::HashMap;
use std::sync::Arc;
use pyo3::exceptions::{PyRuntimeError, PyKeyError};
use crate::types::convert::{js_to_py, py_to_js};
use crate::types::error::JsError;

// 标记为不可跨线程的Python类
#[pyclass(unsendable)]
#[derive(Clone)]
pub struct JsEngine {
    runtime: Arc<RwLock<JsRuntime>>,
}

// 上下文结构体，持有引擎引用和函数缓存
#[pyclass(unsendable)]
pub struct PyContext {
    engine: Arc<JsEngine>,
    global_snapshot: RwLock<HashMap<String, v8::Global<v8::Value>>>,
}

#[pymethods]
impl JsEngine {
    #[new]
    pub fn new() -> Self {
        Self {
            runtime: Arc::new(RwLock::new(
                JsRuntime::new(RuntimeOptions::default())
            )),
        }
    }

    pub fn eval(&self, py: Python<'_>, code: String) -> PyResult<PyObject> {
        let mut runtime = self.runtime.write();
        let result = runtime.execute_script("<eval>", code)
            .map_err(|e| JsError::ExecutionError(e.to_string()))?;
        
        let scope = &mut runtime.handle_scope();
        let local = v8::Local::new(scope, result);
        js_to_py(py, scope, local)
    }

    pub fn compile(&self, _py: Python<'_>, code: String) -> PyResult<PyContext> {
        let engine_arc = Arc::new(self.clone());

        // 获取可写的 runtime 引用
        let mut runtime = self.runtime.write();
        let scope = &mut runtime.handle_scope();

        // 编译脚本
        let source = v8::String::new(scope, &code)
            .ok_or_else(|| JsError::ExecutionError("Invalid code".to_string()))?;

        let script = v8::Script::compile(scope, source, None).ok_or_else(|| PyRuntimeError::new_err("Failed to compile script"))?;
        script.run(scope);

        // 收集函数
        let global = scope.get_current_context().global(scope);
        let mut global_snapshot  = HashMap::new();
        let names = global
            .get_property_names(scope, v8::GetPropertyNamesArgs::default())
            .ok_or_else(|| PyRuntimeError::new_err("Failed to get property names"))?;
        for i in 0..names.length() {
            let key = names.get_index(scope, i).unwrap();
            let key_str = key.to_rust_string_lossy(scope);
            let value = global.get(scope, key).unwrap();
            // 统一存储所有属性
            global_snapshot.insert(key_str, v8::Global::new(scope, value));
        }

        Ok(PyContext {
            engine: engine_arc,
            global_snapshot: RwLock::new(global_snapshot),
        })
    }
}

#[pymethods]
impl PyContext {
    #[pyo3(signature = (name, *args))]
    fn call_function(&self, py: Python<'_>, name: String, args: &Bound<'_, PyTuple>) -> PyResult<PyObject> { 
        let global_snapshot = self.global_snapshot.read();
        let property = global_snapshot.get(&name)
            .ok_or_else(|| PyKeyError::new_err(format!("Property {} not found", name)))?;
        let mut rt = self.engine.runtime.write();
        let scope = &mut rt.handle_scope();
        let mut try_catch = v8::TryCatch::new(&mut *scope);
        let scope = &mut try_catch;
        let context = scope.get_current_context();
        let this = {
            let receiver_name = v8::String::new(scope, "this").ok_or_else(|| PyRuntimeError::new_err("Failed to create receiver name"))?;
            context.global(scope).get(scope, receiver_name.into()).ok_or_else(|| PyRuntimeError::new_err("Failed to get this binding"))?
        };
        // let global = context.global(scope);
        let local_func = match v8::Local::<v8::Function>::try_from(v8::Local::new(scope, property)) {
            Ok(f) if f.is_function() => f,
            _ => return Err(PyRuntimeError::new_err(format!("{} is not a function", name))),
        };
        let mut v8_args = Vec::with_capacity(args.len());
        for item in args.iter() {
            v8_args.push(py_to_js(scope, &item)?);
        }

        // 调用函数并处理错误
        let result = match local_func.call(scope, this.into(), &v8_args) {
            Some(result) => result,
            None => {
                if let Some(exception) = scope.exception() {
                    let message = exception.to_string(scope)
                        .map(|msg| msg.to_rust_string_lossy(scope))
                        .unwrap_or_else(|| "Unknown JavaScript error".to_string());
                    return Err(PyRuntimeError::new_err(message))
                } else {
                    return Err(PyRuntimeError::new_err("Failed to call function"))
                }
            }
        };
        if result.is_promise(){
            return Err(PyRuntimeError::new_err("Promise not supported, please use async_call_function instead"))
        }
        js_to_py(py, scope, result)
    }

    fn get_property(&self, py: Python<'_>, expr: String) -> PyResult<PyObject> {
        let snapshot = self.global_snapshot.read();
        let value = snapshot.get(&expr)
        .ok_or_else(|| PyKeyError::new_err(format!("Property {} not found", expr)))?;
        let mut rt = self.engine.runtime.write();
        let scope = &mut rt.handle_scope();
        let local_value = v8::Local::new(scope, value);
        js_to_py(py, scope, local_value)
    }
}

