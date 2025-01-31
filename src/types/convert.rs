use pyo3::prelude::*;
use pyo3::exceptions::{PyException, PyTypeError};
use pyo3::types::{PyString, PyFloat, PyBool, PyDict, PyList, PyDateTime, PyBytes};
use deno_core::v8;
use std::convert::TryFrom;

trait ValueExt {
    fn is_null_or_undefined(&self) -> bool;
    
    fn to_rust_string_lossy_if_string<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Option<String>;

    fn as_number<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<f64>;
    
    // fn as_boolean<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<bool>;
    
    fn as_bigint<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Option<v8::Local<'s, v8::BigInt>>;
    
    fn as_date<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Option<v8::Local<'s, v8::Date>>;
    
    fn as_array<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Option<v8::Local<'s, v8::Array>>;
    
    fn as_object<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Option<v8::Local<'s, v8::Object>>;
    
    // fn as_array_buffer<'s>(
    //     &self,
    //     scope: &mut v8::HandleScope<'s>,
    // ) -> Option<v8::Local<'s, v8::ArrayBuffer>>;
    
    // fn as_array_buffer_view<'s>(
    //     &self,
    //     scope: &mut v8::HandleScope<'s>,
    // ) -> Option<v8::Local<'s, v8::ArrayBufferView>>;
}

impl ValueExt for v8::Local<'_, v8::Value> {
    fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.is_undefined()
    }

    fn to_rust_string_lossy_if_string<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Option<String> {
        if self.is_string() {
            Some(self.to_string(scope).unwrap().to_rust_string_lossy(scope))
        } else {
            None
        }
    }

    fn as_number<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<f64> {
        if self.is_number() {
            Some(self.number_value(scope)?)
        } else {
            None
        }
    }

    // fn as_boolean<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<bool> {
    //     if self.is_boolean() {
    //         Some(self.boolean_value(scope))
    //     } else {
    //         None
    //     }
    // }

    fn as_bigint<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Option<v8::Local<'s, v8::BigInt>> {
        if self.is_big_int() {
            let raw = v8::Local::<v8::BigInt>::try_from(*self).ok()?;
            Some(v8::Local::new(scope, raw))
        } else {
            None
        }
    }

    fn as_date<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Option<v8::Local<'s, v8::Date>> {
        if self.is_date() {
            let raw = v8::Local::<v8::Date>::try_from(*self).ok()?;
            Some(v8::Local::new(scope, raw))
        } else {
            None
        }
    }

    fn as_array<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Option<v8::Local<'s, v8::Array>> {
        if self.is_array() {
            let raw = v8::Local::<v8::Array>::try_from(*self).ok()?;
            Some(v8::Local::new(scope, raw))
        } else {
            None
        }
    }

    fn as_object<'s>(
        &self,
        scope: &mut v8::HandleScope<'s>,
    ) -> Option<v8::Local<'s, v8::Object>> {
        if self.is_object() {
            let raw = v8::Local::<v8::Object>::try_from(*self).ok()?;
            Some(v8::Local::new(scope, raw))
        } else {
            None
        }
    }

    // fn as_array_buffer<'s>(
    //     &self,
    //     scope: &mut v8::HandleScope<'s>,
    // ) -> Option<v8::Local<'s, v8::ArrayBuffer>> {
    //     if self.is_array_buffer() {
    //         let raw = v8::Local::<v8::ArrayBuffer>::try_from(*self).ok()?;
    //         Some(v8::Local::new(scope, raw))
    //         // v8::Local::<v8::ArrayBuffer>::try_from(*self).ok()
    //     } else {
    //         None
    //     }
    // }

    // fn as_array_buffer_view<'s>(
    //     &self,
    //     scope: &mut v8::HandleScope<'s>,
    // ) -> Option<v8::Local<'s, v8::ArrayBufferView>> {
    //     if self.is_array_buffer_view() {
    //         let raw = v8::Local::<v8::ArrayBufferView>::try_from(*self).ok()?;
    //         Some(v8::Local::new(scope, raw))
    //         // v8::Local::<v8::ArrayBufferView>::try_from(*self).ok()
    //     } else {
    //         None
    //     }
    // }
}


pub fn js_to_py<'a>(
    py: Python<'_>,
    scope: &mut v8::HandleScope<'a>,
    value: v8::Local<'a, v8::Value>,
) -> PyResult<PyObject> {
    if value.is_null_or_undefined() {
        return Ok(py.None());
    }

    // String
    if let Some(s) = value.to_rust_string_lossy_if_string(scope) {
        return Ok(PyString::new(py, &s).into());
    }

    // Number
    if let Some(num) = value.as_number(scope) {
        return handle_number(py, num);
    }

    // Boolean
    if value.is_boolean() {
        // let val = value.boolean_value(scope);
        return Ok(PyBool::new(py, value.boolean_value(scope)).into_py(py));
    }

    // BigInt
    if let Some(bigint) = value.as_bigint(scope) {
        return Ok(bigint.i64_value().0.into_py(py));
    }

    // Date
    if let Some(date) = value.as_date(scope) {
        return handle_date(py, date);
    }

    // Array
    if let Some(array) = value.as_array(scope) {
        return handle_array(py, scope, array);
    }

    // Object
    if let Some(obj) = value.as_object(scope) {
        return handle_object(py, scope, obj);
    }

    // ArrayBuffer/ArrayBufferView
    if value.is_array_buffer() || value.is_array_buffer_view() {
        return handle_array_buffer(py, scope, value);
    }

    Err(PyTypeError::new_err(format!(
        "Unsupported JS type: {}",
        value.type_of(scope).to_rust_string_lossy(scope)
    )))
}

// 辅助函数
fn handle_number(py: Python<'_>, num: f64) -> PyResult<PyObject> {
    if num.fract() == 0.0 && num <= i64::MAX as f64 && num >= i64::MIN as f64 {
        Ok((num as i64).into_py(py))
    } else {
        Ok(PyFloat::new(py, num).into())
    }
}

fn handle_date(py: Python<'_>, date: v8::Local<v8::Date>) -> PyResult<PyObject> {
    let timestamp = date.value_of() / 1000.0;
    Ok(PyDateTime::from_timestamp(py, timestamp, None)?.into())
}

fn handle_array(
    py: Python<'_>,
    scope: &mut v8::HandleScope,
    array: v8::Local<v8::Array>,
) -> PyResult<PyObject> {
    let py_list = PyList::empty(py);
    for i in 0..array.length() {
        if let Some(item) = array.get_index(scope, i) {
            py_list.append(js_to_py(py, scope, item)?)?;
        }
    }
    Ok(py_list.into())
}

fn handle_object(
    py: Python<'_>,
    scope: &mut v8::HandleScope,
    obj: v8::Local<v8::Object>,
) -> PyResult<PyObject> {
    let py_dict = PyDict::new(py);
    let keys = obj.get_own_property_names(scope, v8::GetPropertyNamesArgs::default());
    if let Some(keys) = keys {
        for i in 0..keys.length() {
            let key = keys.get_index(scope, i).unwrap();
            let key_str = key.to_rust_string_lossy(scope);
            let value = obj.get(scope, key).unwrap();
            py_dict.set_item(key_str, js_to_py(py, scope, value)?)?;
        }
    }
    Ok(py_dict.into())
}

fn handle_array_buffer(
    py: Python<'_>,
    scope: &mut v8::HandleScope,
    value: v8::Local<v8::Value>,
) -> PyResult<PyObject> {
    let (store, offset, len) = if value.is_array_buffer() {
        let buffer = v8::Local::<v8::ArrayBuffer>::try_from(value)
            .map_err(|_| PyException::new_err("Failed to convert to ArrayBuffer"))?;
        (buffer.get_backing_store(), 0, buffer.byte_length())
    } else {
        let view = v8::Local::<v8::ArrayBufferView>::try_from(value)
            .map_err(|_| PyException::new_err("Failed to convert to ArrayBufferView"))?;
        // let buffer = view.buffer().ok_or(PyException::new_err("Detached ArrayBufferView"))?;
        let buffer = view.buffer(scope).ok_or(PyException::new_err("Detached ArrayBufferView"))?;
        (
            buffer.get_backing_store(),
            view.byte_offset(),
            view.byte_length()
        )
    };

    let data_ptr = store.data().ok_or(PyException::new_err("Invalid backing store"))?;
    let bytes = unsafe {
        std::slice::from_raw_parts(
            data_ptr.as_ptr().add(offset) as *const u8,
            len
        )
    };

    Ok(PyBytes::new(py, bytes).into())
}



pub fn py_to_js<'a>(scope: &mut v8::HandleScope<'a>, obj: &Bound<'_, PyAny>) -> PyResult<v8::Local<'a, v8::Value>> {
    if let Ok(s) = obj.extract::<String>() {
        Ok(v8::String::new(scope, &s).unwrap().into())
    } else if let Ok(n) = obj.extract::<f64>() {
        Ok(v8::Number::new(scope, n).into())
    } else if let Ok(b) = obj.extract::<bool>() {
        Ok(v8::Boolean::new(scope, b).into())
    } else if obj.is_none() {
        Ok(v8::null(scope).into())
    } else if let Ok(list) = obj.downcast::<PyList>() {
        let array = v8::Array::new(scope, list.len() as i32);
        for (i, item) in list.iter().enumerate() {
            let value = py_to_js(scope, &item)?;
            array.set_index(scope, i as u32, value).unwrap();
        }
        Ok(array.into())
    } else if let Ok(dict) = obj.downcast::<PyDict>() {
        let js_obj = v8::Object::new(scope);
        for (key, value) in dict.iter() {
            let key_str = key.extract::<String>()?;
            let js_key = v8::String::new(scope, &key_str).unwrap();
            let js_value = py_to_js(scope, &value)?;
            js_obj.set(scope, js_key.into(), js_value).unwrap();
        }
        Ok(js_obj.into())
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err("Unsupported Python type"))
    }
}


