// use deno_core::{v8, JsRuntime, RuntimeOptions};
// use pyo3::prelude::*;
// use parking_lot::RwLock;
// use pyo3::types::PyTuple;
// use pyo3_async_runtimes::tokio::future_into_py;
// use std::collections::HashMap;
// use std::sync::Arc;
// use pyo3::exceptions::{PyRuntimeError, PyKeyError};
// use crate::types::convert::{js_to_py, py_to_js};
// use crate::types::error::JsError;
// use std::future::Future;
// use std::pin::Pin;
// use futures::future::BoxFuture;

// #[pyclass(unsendable)]
// #[derive(Clone)]
// pub struct AsyncEngine {
//     runtime: Arc<RwLock<JsRuntime>>,
//     task_sender: tokio::sync::mpsc::UnboundedSender<BoxFuture<'static, ()>>,
// }


// #[pyclass(unsendable)]
// pub struct PyContext {
//     engine: Arc<AsyncEngine>,
//     global_snapshot: RwLock<HashMap<String, v8::Global<v8::Value>>>,
// }

// impl AsyncEngine {
//     async fn run_in_main_thread<F, Fut, R>(&self, f: F) -> R
//     where
//         F: FnOnce() -> Fut + Send + 'static,
//         Fut: Future<Output = R> + Send + 'static,
//         R: Send + 'static,
//     {
//         let (sender, receiver) = tokio::sync::oneshot::channel();
        
//         // 创建异步任务
//         let future = async move {
//             let result = f().await;
//             let _ = sender.send(result);
//         };

//         // 发送异步任务到执行线程
//         self.task_sender
//             .send(Box::pin(future))
//             .expect("Failed to send task");

//         receiver.await.expect("Failed to receive result")
//     }
// }


// #[pymethods]
// impl AsyncEngine {
//     #[new]
//     pub fn new() -> Self {
//         let (task_sender, mut task_receiver) = tokio::sync::mpsc::unbounded_channel::<BoxFuture<'static, ()>>();
//         std::thread::spawn(move || {
//             let rt = tokio::runtime::Builder::new_current_thread()
//                 .enable_all()
//                 .build()
//                 .unwrap();
            
//                 rt.block_on(async {
//                     while let Some(fut) = task_receiver.recv().await {
//                         // 直接执行异步任务
//                         fut.await;
//                     }
//                 });
//         });

//         Self {
//             runtime: Arc::new(RwLock::new(JsRuntime::new(RuntimeOptions::default()))),
//             task_sender,
//         }
//     }


//     pub fn compile(&self, _py: Python<'_>, code: String) -> PyResult<PyContext> {
//         let engine_arc = Arc::new(self.clone());
//         let mut runtime = self.runtime.write();
//         let scope = &mut runtime.handle_scope();

//         // 编译脚本
//         let source = v8::String::new(scope, &code)
//             .ok_or_else(|| JsError::ExecutionError("Invalid code".to_string()))?;

//         let script = v8::Script::compile(scope, source, None).ok_or_else(|| PyRuntimeError::new_err("Failed to compile script"))?;
//         script.run(scope);

//         // 收集函数
//         let global = scope.get_current_context().global(scope);
//         let mut global_snapshot  = HashMap::new();
//         let names = global
//             .get_property_names(scope, v8::GetPropertyNamesArgs::default())
//             .ok_or_else(|| PyRuntimeError::new_err("Failed to get property names"))?;
//         for i in 0..names.length() {
//             let key = names.get_index(scope, i).unwrap();
//             let key_str = key.to_rust_string_lossy(scope);
//             let value = global.get(scope, key).unwrap();
//             // 统一存储所有属性
//             global_snapshot.insert(key_str, v8::Global::new(scope, value));
//         }

//         Ok(PyContext {
//             engine: engine_arc,
//             global_snapshot: RwLock::new(global_snapshot),
//         })
//     }

// }

// #[pymethods]
// impl PyContext {
//     #[pyo3(signature = (name, *args))]
//     fn call_async(&self, py: Python<'_>, name: String,  args: &Bound<'_, PyTuple>) -> PyResult<Bound<'_, PyAny>>{
//         let global_snapshot = self.global_snapshot.read();
//         let property = global_snapshot.get(&name)
//             .ok_or_else(|| PyKeyError::new_err(format!("Property {} not found", name)))?;
//         let mut rt = self.engine.runtime.write();
//         let scope = &mut rt.handle_scope();
//         let mut try_catch = v8::TryCatch::new(&mut *scope);
//         let scope = &mut try_catch;
//         let context = scope.get_current_context();
//         let this = {
//             let receiver_name = v8::String::new(scope, "this").ok_or_else(|| PyRuntimeError::new_err("Failed to create receiver name"))?;
//             context.global(scope).get(scope, receiver_name.into()).ok_or_else(|| PyRuntimeError::new_err("Failed to get this binding"))?
//         };
//         // let global = context.global(scope);
//         let local_func = match v8::Local::<v8::Function>::try_from(v8::Local::new(scope, property)) {
//             Ok(f) if f.is_function() => f,
//             _ => return Err(PyRuntimeError::new_err(format!("{} is not a function", name))),
//         };
//         let mut v8_args = Vec::with_capacity(args.len());
//         for item in args.iter() {
//             v8_args.push(py_to_js(scope, &item)?);
//         }
//         // println!("v8_args: {:?}", v8_args);
//         // 调用函数并处理错误
//         let result = match local_func.call(scope, this.into(), &v8_args) {
//             Some(result) => result,
//             None => {
//                 if let Some(exception) = scope.exception() {
//                     let message = exception.to_string(scope)
//                         .map(|msg| msg.to_rust_string_lossy(scope))
//                         .unwrap_or_else(|| "Unknown JavaScript error".to_string());
//                     return Err(PyRuntimeError::new_err(message))
//                 } else {
//                     return Err(PyRuntimeError::new_err("Failed to call function"))
//                 }
//             }
//         };
//         if !result.is_promise(){
//             return future_into_py(py, async move {
//                 Ok(())
//             })
//         }else {
//             return future_into_py(py, async move {
//                 let (sender, receiver) = tokio::sync::oneshot::channel();
//                 self.engine.run_in_main_thread(|| async move {
//                     let mut rt = self.engine.runtime.write();
//                     let scope = &mut rt.handle_scope();
//                     let mut try_catch = v8::TryCatch::new(&mut *scope);
//                     let scope = &mut try_catch;
//                     let context = scope.get_current_context();
//                     let promise = v8::Local::<v8::Promise>::try_from(result);
//                     match promise {
//                         Ok(promise) => {
//                             while promise.state() == v8::PromiseState::Pending {
//                                 scope.perform_microtask_checkpoint();
//                                 tokio::task::yield_now().await;
//                             }
//                             match promise.state() {
//                                 v8::PromiseState::Fulfilled => {
//                                     let value = promise.result(scope);
//                                     let py_value = js_to_py(py, scope, value)?;
//                                     return Ok(py_value);
//                                 },
//                                 v8::PromiseState::Rejected => {
//                                     let reason = promise.result(scope);
//                                     let message = reason.to_string(scope).unwrap().to_rust_string_lossy(scope);
//                                     return Err(PyRuntimeError::new_err(message));

//                                 }
//                                 _ => {
//                                     return Err(PyRuntimeError::new_err("Failed to get promise state"))
//                                 }

//                             }
//                         },
//                         Err(_) => {
//                             return Err(PyRuntimeError::new_err("Failed to convert result to promise"))
//                         }
//                     }
//                 }).await;
//                 receiver.await.unwrap()
                
//             })
//         }
//     }



// }
