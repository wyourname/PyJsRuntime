// use deno_core::anyhow::Ok;
use deno_core::{v8, JsRuntime, RuntimeOptions};
use pyo3::prelude::*;
use pyo3::types::PyList;
use parking_lot::RwLock;
use pyo3::exceptions::{PyValueError, PyRuntimeError};
use std::{collections::HashMap, result};
use std::sync::Arc;
use crate::types::convert::{js_to_py, py_to_js};


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
    functions: RwLock<HashMap<String, v8::Global<v8::Function>>>,
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
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
        
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
            .ok_or_else(|| PyValueError::new_err("Invalid code"))?;

        let script = v8::Script::compile(scope, source, None).ok_or_else(|| PyRuntimeError::new_err("Failed to compile script"))?;
        script.run(scope);

        // 收集函数
        let global = scope.get_current_context().global(scope);
        let mut functions = HashMap::new();
        let names = global
            .get_property_names(scope, v8::GetPropertyNamesArgs::default())
            .ok_or_else(|| PyRuntimeError::new_err("Failed to get property names"))?;
        for i in 0..names.length() {
            let key = names
                .get_index(scope, i)
                .ok_or_else(|| PyRuntimeError::new_err("Failed to get property key"))?;

            // 检查键是否为字符串类型
            if !key.is_string() {
                continue; // 跳过非字符串的键
            }

            let key_str = key.to_rust_string_lossy(scope);

            let value = global
                .get(scope, key)
                .ok_or_else(|| PyRuntimeError::new_err("Failed to get property value"))?;

            if let Ok(func) = v8::Local::<v8::Function>::try_from(value) {
                // println!("Function: {}", key_str);
                // let function = func.unwrap();
                functions.insert(key_str, v8::Global::new(scope, func));
            }
        }

        Ok(PyContext {
            engine: engine_arc,
            functions: RwLock::new(functions),
        })
    }
}

#[pymethods]
impl PyContext {
    #[pyo3(signature = (name, args))]
    fn call(&self, py: Python<'_>,name: String, args: &Bound<'_, PyList>) -> PyResult<PyObject> { 
        let functions = self.functions.read();
        let func = functions.get(&name)
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(format!("Function {} not found", name)))?;

        let mut rt = self.engine.runtime.write();
        // let scope = &mut rt.handle_scope();
        let mut hs = rt.handle_scope();
        
        // 创建 TryCatch 作用域
        let mut try_catch = v8::TryCatch::new(&mut hs);
        let scope = &mut try_catch;
        let context = scope.get_current_context();

        // 准备参数
        let mut v8_args = Vec::with_capacity(args.len());
        for item in args.iter() {
            v8_args.push(py_to_js(scope, &item)?);
        }
        // 调用函数
        let local_func = v8::Local::new(scope, func);
        let global = context.global(scope);
        let result = local_func.call(scope, global.into(), &v8_args).ok_or_else(|| {
            if let Some(exception) = scope.exception() {
                let message = exception.to_string(scope);
                if let Some(message) = message {
                    return PyRuntimeError::new_err(message.to_rust_string_lossy(scope));
                }else {
                    return PyRuntimeError::new_err("Failed to call function");
                }
                // let message = exception.to_string(scope);
                // let message = message.to_rust_string_lossy(scope);
                // PyRuntimeError::new_err(message.to_rust_string_lossy(scope))
            }else {
                PyRuntimeError::new_err("Failed to call function")
            }
        });
        if let Ok(result) = result {
            js_to_py(py, scope, result)
        }else {
            Err(result.unwrap_err())
        }
    }
}