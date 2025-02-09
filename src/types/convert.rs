use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyString, PyFloat, PyDict, PyList, PyDateTime, PyBytes, PySet};
use deno_core::v8;
use std::convert::TryFrom;
use crate::types::error::TypeConversionError;

// 简化 ValueExt trait
trait ValueExt {
    fn is_null_or_undefined(&self) -> bool;
    fn to_rust_string_lossy_if_string<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<String>;
    fn as_number<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<f64>;
    fn as_bigint<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<v8::Local<'s, v8::BigInt>>;
    fn as_date<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<v8::Local<'s, v8::Date>>;
    fn as_array<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<v8::Local<'s, v8::Array>>;
    fn as_object<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<v8::Local<'s, v8::Object>>;
}

impl ValueExt for v8::Local<'_, v8::Value> {
    #[inline]
    fn is_null_or_undefined(&self) -> bool {
        self.is_null() || self.is_undefined()
    }

    #[inline]
    fn to_rust_string_lossy_if_string<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<String> {
        self.is_string().then(|| self.to_string(scope).unwrap().to_rust_string_lossy(scope))
    }

    #[inline]
    fn as_number<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<f64> {
        self.is_number().then(|| self.number_value(scope)).flatten()
    }

    #[inline]
    fn as_bigint<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<v8::Local<'s, v8::BigInt>> {
        self.is_big_int().then(|| {
            v8::Local::<v8::BigInt>::try_from(*self)
                .ok()
                .map(|raw| v8::Local::new(scope, raw))
        }).flatten()
    }

    #[inline]
    fn as_date<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<v8::Local<'s, v8::Date>> {
        self.is_date().then(|| {
            v8::Local::<v8::Date>::try_from(*self)
                .ok()
                .map(|raw| v8::Local::new(scope, raw))
        }).flatten()
    }

    #[inline]
    fn as_array<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<v8::Local<'s, v8::Array>> {
        self.is_array().then(|| {
            v8::Local::<v8::Array>::try_from(*self)
                .ok()
                .map(|raw| v8::Local::new(scope, raw))
        }).flatten()
    }

    #[inline]
    fn as_object<'s>(&self, scope: &mut v8::HandleScope<'s>) -> Option<v8::Local<'s, v8::Object>> {
        self.is_object().then(|| {
            v8::Local::<v8::Object>::try_from(*self)
                .ok()
                .map(|raw| v8::Local::new(scope, raw))
        }).flatten()
    }
}

#[allow(deprecated)]
pub fn js_to_py<'a>(
    py: Python<'_>,
    scope: &mut v8::HandleScope<'a>,
    value: v8::Local<'a, v8::Value>,
) -> PyResult<PyObject> {
    if value.is_null_or_undefined() {
        return Ok(py.None());
    }

    // 字符串处理
    if value.is_string_object() || value.is_string() {
        if let Some(s) = value.to_rust_string_lossy_if_string(scope) {
            return Ok(PyString::new(py, &s).into());
        }
    }

    // 数字处理
    if let Some(num) = value.as_number(scope) {
        if num.is_nan() {
            return Ok(f64::NAN.into_pyobject(py)?.into());
        }
        if num.is_infinite() {
            return Ok((if num.is_sign_positive() { f64::INFINITY } else { f64::NEG_INFINITY }).into_pyobject(py)?.into());
        }
        return handle_number(py, num);
    }

    // 布尔处理
    if value.is_boolean_object() || value.is_boolean() {
        let bool = value.boolean_value(scope);
        return Ok(bool.to_object(py));
    }

    // BigInt 处理
    if let Some(bigint) = value.as_bigint(scope) {
        return Ok(bigint.i64_value().0.into_pyobject(py)?.into());
    }

    // 日期处理
    if let Some(date) = value.as_date(scope) {
        return handle_date(py, date);
    }

    // 数组处理
    if let Some(array) = value.as_array(scope) {
        return handle_array(py, scope, array);
    }

    // TypedArray 处理
    if value.is_typed_array() {
        return handle_typed_array(py, scope, value);
    }

    // Set 处理
    if value.is_set() {
        let set = v8::Local::<v8::Set>::try_from(value)
            .map_err(|_| TypeConversionError::InvalidValue("Failed to convert to Set".to_string()))?;
        return handle_set(py, scope, set);
    }

    // Map 处理
    if value.is_map() {
        let map = v8::Local::<v8::Map>::try_from(value)
            .map_err(|_| TypeConversionError::InvalidValue("Failed to convert to Map".to_string()))?;
        return handle_map(py, scope, map);
    }

    // 函数处理
    // if value.is_function() {
    //     return handle_function(py, scope, value);
    // }

    // Promise 处理
    // if value.is_promise() {
    //     return handle_promise(py, scope, value);
    // }

    // 正则表达式处理 未实现

    // ArrayBuffer/ArrayBufferView 处理
    if value.is_array_buffer() || value.is_array_buffer_view() {
        return handle_array_buffer(py, scope, value);
    }

    // 普通对象处理
    if let Some(obj) = value.as_object(scope) {
        return handle_object(py, scope, obj);
    }

    Err(PyTypeError::new_err(format!(
        "Unsupported JS type: {}",
        value.type_of(scope).to_rust_string_lossy(scope)
    )))
}

#[inline]
fn handle_number(py: Python<'_>, num: f64) -> PyResult<PyObject> {
    if num.fract() == 0.0 && num <= i64::MAX as f64 && num >= i64::MIN as f64 {
        Ok((num as i64).into_pyobject(py)?.into())
    } else {
        Ok(PyFloat::new(py, num).into())
    }
}

#[inline]
fn handle_date(py: Python<'_>, date: v8::Local<v8::Date>) -> PyResult<PyObject> {
    let timestamp = date.value_of() / 1000.0;
    Ok(PyDateTime::from_timestamp(py, timestamp, None)?.into())
}

#[inline]
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

#[inline]
fn handle_object(
    py: Python<'_>,
    scope: &mut v8::HandleScope,
    obj: v8::Local<v8::Object>,
) -> PyResult<PyObject> {
    let py_dict = PyDict::new(py);
    if let Some(keys) = obj.get_own_property_names(scope, v8::GetPropertyNamesArgs::default()) {
        for i in 0..keys.length() {
            if let (Some(key), Some(value)) = (
                keys.get_index(scope, i),
                keys.get_index(scope, i).and_then(|k| obj.get(scope, k))
            ) {
                py_dict.set_item(
                    key.to_rust_string_lossy(scope),
                    js_to_py(py, scope, value)?
                )?;
            }
        }
    }
    Ok(py_dict.into())
}

#[inline]
fn handle_array_buffer(
    py: Python<'_>,
    scope: &mut v8::HandleScope,
    value: v8::Local<v8::Value>,
) -> PyResult<PyObject> {
    let (store, offset, len) = if value.is_array_buffer() {
        let buffer = v8::Local::<v8::ArrayBuffer>::try_from(value)
            .map_err(|_| TypeConversionError::InvalidValue("Failed to convert to ArrayBuffer".to_string()))?;
        (buffer.get_backing_store(), 0, buffer.byte_length())
    } else {
        let view = v8::Local::<v8::ArrayBufferView>::try_from(value)
            .map_err(|_| TypeConversionError::InvalidValue("Failed to convert to ArrayBufferView".to_string()))?;
        let buffer = view.buffer(scope)
            .ok_or_else(|| TypeConversionError::InvalidValue("Detached ArrayBufferView".to_string()))?;
        (buffer.get_backing_store(), view.byte_offset(), view.byte_length())
    };

    let data_ptr = store.data()
        .ok_or_else(|| TypeConversionError::InvalidValue("Invalid backing store".to_string()))?;
    
    let bytes = unsafe {
        std::slice::from_raw_parts(
            data_ptr.as_ptr().add(offset) as *const u8,
            len
        )
    };

    Ok(PyBytes::new(py, bytes).into())
}

#[inline]
fn handle_typed_array(
    py: Python<'_>,
    scope: &mut v8::HandleScope,
    value: v8::Local<v8::Value>,
) -> PyResult<PyObject> {
    let view = v8::Local::<v8::TypedArray>::try_from(value)
        .map_err(|_| TypeConversionError::InvalidValue("Failed to convert to TypedArray".to_string()))?;
    
    let buffer = view.buffer(scope)
        .ok_or_else(|| TypeConversionError::InvalidValue("Detached TypedArray".to_string()))?;
    
    let store = buffer.get_backing_store();
    let data = store.data()
        .ok_or_else(|| TypeConversionError::InvalidValue("Invalid backing store".to_string()))?;
    
    let bytes = unsafe {
        std::slice::from_raw_parts(
            (data.as_ptr() as *const u8).add(view.byte_offset()),
            view.byte_length()
        )
    };

    Ok(PyBytes::new(py, bytes).into())
}

#[inline]
fn handle_set(
    py: Python<'_>,
    scope: &mut v8::HandleScope,
    set: v8::Local<v8::Set>,
) -> PyResult<PyObject> {
    let py_set = PySet::empty(py).map_err(|e| TypeConversionError::InvalidValue(e.to_string()))?;
    let array = set.as_array(scope);
    
    for i in 0..array.length() {
        if let Some(item) = array.get_index(scope, i) {
            py_set.add(js_to_py(py, scope, item)?)?;
        }
    }
    
    Ok(py_set.into())
}

#[inline]
fn handle_map(
    py: Python<'_>,
    scope: &mut v8::HandleScope,
    map: v8::Local<v8::Map>,
) -> PyResult<PyObject> {
    let py_dict = PyDict::new(py);
    let array = map.as_array(scope);
    
    for i in (0..array.length()).step_by(2) {
        if let (Some(key), Some(value)) = (
            array.get_index(scope, i),
            array.get_index(scope, i + 1)
        ) {
            py_dict.set_item(
                js_to_py(py, scope, key)?,
                js_to_py(py, scope, value)?
            )?;
        }
    }
    
    Ok(py_dict.into())
}

// #[inline]
// fn handle_function(
//     py: Python<'_>,
//     scope: &mut v8::HandleScope,
//     value: v8::Local<v8::Value>,
// ) -> PyResult<PyObject> {
//     // 创建一个 Python 函数包装器
//     // 这里需要实现具体的函数调用逻辑
//     Err(PyTypeError::new_err("JavaScript function conversion not implemented"))
// }

// #[inline]
// fn handle_promise(
//     py: Python<'_>,
//     scope: &mut v8::HandleScope,
//     value: v8::Local<v8::Value>,
// ) -> PyResult<PyObject> {
//     // 创建一个 Python Future 对象
//     // 这里需要实现具体的异步处理逻辑
//     Err(PyTypeError::new_err("JavaScript Promise conversion not implemented"))
// }


pub fn py_to_js<'a>(
    scope: &mut v8::HandleScope<'a>,
    obj: &Bound<'_, PyAny>,
) -> PyResult<v8::Local<'a, v8::Value>> {
    if obj.is_none() {
        return Ok(v8::null(scope).into());
    }
    
    // 字符串处理
    if let Ok(s) = obj.extract::<String>() {
        return Ok(v8::String::new(scope, &s).unwrap().into());
    }
    
    // 整数类型处理
    if let Ok(n) = obj.extract::<i64>() {
        return Ok(v8::Number::new(scope, n as f64).into());
    }

    if let Ok(n) = obj.extract::<i32>() {
        return Ok(v8::Number::new(scope, n as f64).into());
    }

    if let Ok(n) = obj.extract::<u32>() {
        return Ok(v8::Number::new(scope, n as f64).into());
    }

    // 浮点数处理
    if let Ok(n) = obj.extract::<f64>() {
        if n.is_nan() {
            return Ok(v8::Number::new(scope, f64::NAN).into());
        }
        if n.is_infinite() {
            return Ok(v8::Number::new(scope, 
                if n.is_sign_positive() { f64::INFINITY } else { f64::NEG_INFINITY }
            ).into());
        }
        return Ok(v8::Number::new(scope, n).into());
    }
    
    // 布尔处理
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(v8::Boolean::new(scope, b).into());
    }
    
    // 字节数据处理
    if let Ok(bytes) = obj.downcast::<PyBytes>() {
        let len = bytes.len()? as usize;
        let buffer = v8::ArrayBuffer::new(scope, len);
        let store = buffer.get_backing_store();
        if let Some(data) = store.data() {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    bytes.as_bytes().as_ptr(),
                    data.as_ptr() as *mut u8,
                    len
                );
            }
            return Ok(buffer.into());
        }
        return Err(TypeConversionError::InvalidValue("Failed to get buffer data".to_string()).into());
    }

    // 日期时间处理
    if let Ok(dt) = obj.downcast::<PyDateTime>() {
        let timestamp = dt.call_method0("timestamp")?.extract::<f64>()?;
        return Ok(v8::Date::new(scope, timestamp * 1000.0).unwrap().into());
    }
    
    // 列表处理
    if let Ok(list) = obj.downcast::<PyList>() {
        let array = v8::Array::new(scope, list.len() as i32);
        for (i, item) in list.iter().enumerate() {
            let js_value = py_to_js(scope, &item)?;
            array.set_index(scope, i as u32, js_value).unwrap();
        }
        return Ok(array.into());
    }
    
    // 字典处理
    if let Ok(dict) = obj.downcast::<PyDict>() {
        let js_obj = v8::Object::new(scope);
        for (key, value) in dict.iter() {
            if let Ok(key_str) = key.extract::<String>() {
                let js_key = v8::String::new(scope, &key_str).unwrap();
                let js_value = py_to_js(scope, &value)?;
                js_obj.set(scope, js_key.into(), js_value).unwrap();
            }
        }
        return Ok(js_obj.into());
    }
    
    Err(TypeConversionError::InvalidValue(format!(
        "Unsupported Python type: {}",
        obj.get_type().to_string()
    )).into())
}


