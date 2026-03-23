//! C API 值类型转换模块
//!
//! 提供 graphdb_value_t 和 core::Value 之间的转换

use crate::api::embedded::c_api::types::{
    graphdb_string_t, graphdb_value_data_t, graphdb_value_t, graphdb_value_type_t,
};
use crate::core::Value;

/// 将 C API 值类型转换为 Core Value
pub fn graphdb_value_to_core(value: *const graphdb_value_t) -> Value {
    if value.is_null() {
        return Value::Null(crate::core::NullType::Null);
    }

    unsafe {
        let val = &*value;
        match val.type_ {
            graphdb_value_type_t::GRAPHDB_NULL => Value::Null(crate::core::NullType::Null),
            graphdb_value_type_t::GRAPHDB_BOOL => Value::Bool(val.data.boolean),
            graphdb_value_type_t::GRAPHDB_INT => Value::Int(val.data.integer),
            graphdb_value_type_t::GRAPHDB_FLOAT => Value::Float(val.data.floating),
            graphdb_value_type_t::GRAPHDB_STRING => {
                let s = &val.data.string;
                let bytes = std::slice::from_raw_parts(s.data as *const u8, s.len);
                Value::String(String::from_utf8_lossy(bytes).into_owned())
            }
            _ => Value::Null(crate::core::NullType::Null),
        }
    }
}

/// 将 Core Value 转换为 C API 值类型
pub fn core_value_to_graphdb(value: &Value) -> graphdb_value_t {
    match value {
        Value::Null(_) => graphdb_value_t {
            type_: graphdb_value_type_t::GRAPHDB_NULL,
            data: graphdb_value_data_t {
                ptr: std::ptr::null_mut(),
            },
        },
        Value::Bool(b) => graphdb_value_t {
            type_: graphdb_value_type_t::GRAPHDB_BOOL,
            data: graphdb_value_data_t { boolean: *b },
        },
        Value::Int(i) => graphdb_value_t {
            type_: graphdb_value_type_t::GRAPHDB_INT,
            data: graphdb_value_data_t { integer: *i },
        },
        Value::Float(f) => graphdb_value_t {
            type_: graphdb_value_type_t::GRAPHDB_FLOAT,
            data: graphdb_value_data_t { floating: *f },
        },
        Value::String(s) => {
            let string_t = graphdb_string_t {
                data: s.as_ptr() as *const i8,
                len: s.len(),
            };
            graphdb_value_t {
                type_: graphdb_value_type_t::GRAPHDB_STRING,
                data: graphdb_value_data_t { string: string_t },
            }
        }
        _ => graphdb_value_t {
            type_: graphdb_value_type_t::GRAPHDB_NULL,
            data: graphdb_value_data_t {
                ptr: std::ptr::null_mut(),
            },
        },
    }
}

/// 获取 Core Value 的 C API 类型
pub fn core_value_to_graphdb_type(value: &Value) -> graphdb_value_type_t {
    match value {
        Value::Null(_) => graphdb_value_type_t::GRAPHDB_NULL,
        Value::Bool(_) => graphdb_value_type_t::GRAPHDB_BOOL,
        Value::Int(_) => graphdb_value_type_t::GRAPHDB_INT,
        Value::Float(_) => graphdb_value_type_t::GRAPHDB_FLOAT,
        Value::String(_) => graphdb_value_type_t::GRAPHDB_STRING,
        Value::List(_) => graphdb_value_type_t::GRAPHDB_LIST,
        Value::Map(_) => graphdb_value_type_t::GRAPHDB_MAP,
        Value::Vertex(_) => graphdb_value_type_t::GRAPHDB_VERTEX,
        Value::Edge(_) => graphdb_value_type_t::GRAPHDB_EDGE,
        Value::Path(_) => graphdb_value_type_t::GRAPHDB_PATH,
        _ => graphdb_value_type_t::GRAPHDB_NULL,
    }
}
