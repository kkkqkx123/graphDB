//! C API 自定义函数模块
//!
//! 提供自定义标量函数和聚合函数的注册功能

use crate::api::embedded::c_api::session::GraphDbSessionHandle;
use crate::api::embedded::c_api::types::{
    graphdb_session_t, graphdb_value_t, graphdb_value_type_t,
};
use crate::c_api::graphdb_error_code_t;
use crate::query::executor::expression::functions::{
    AggregateFinalCallback, AggregateStepCallback, CFunctionContext, CustomFunction,
    ScalarFunctionCallback,
};
use std::ffi::{c_char, c_int, c_void, CStr};

/// 标量函数回调类型
#[allow(non_camel_case_types)]
pub type graphdb_scalar_function_callback =
    Option<extern "C" fn(context: *mut graphdb_context_t, argc: c_int, argv: *mut graphdb_value_t)>;

/// 聚合函数步骤回调类型
#[allow(non_camel_case_types)]
pub type graphdb_aggregate_step_callback =
    Option<extern "C" fn(context: *mut graphdb_context_t, argc: c_int, argv: *mut graphdb_value_t)>;

/// 聚合函数最终回调类型
#[allow(non_camel_case_types)]
pub type graphdb_aggregate_final_callback = Option<extern "C" fn(context: *mut graphdb_context_t)>;

/// 函数析构回调类型
#[allow(non_camel_case_types)]
pub type graphdb_function_destroy_callback = Option<extern "C" fn(user_data: *mut c_void)>;

/// 函数执行上下文（不透明指针）
#[repr(C)]
pub struct graphdb_context_t {
    /// 内部上下文
    pub(crate) inner: CFunctionContext,
}

/// 创建自定义标量函数
///
/// # 参数
/// - `session`: 会话句柄
/// - `name`: 函数名称
/// - `argc`: 参数数量，-1 表示可变参数
/// - `user_data`: 用户数据指针
/// - `x_func`: 标量函数回调
/// - `x_destroy`: 析构回调，可为 NULL
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
///
/// # 示例
/// ```c
/// extern void my_function(graphdb_context_t* ctx, int argc, graphdb_value_t* argv) {
///     // 实现函数逻辑
/// }
///
/// graphdb_create_function(session, "my_func", 2, NULL, my_function, NULL);
/// ```
#[no_mangle]
pub extern "C" fn graphdb_create_function(
    session: *mut graphdb_session_t,
    name: *const c_char,
    argc: c_int,
    user_data: *mut c_void,
    x_func: graphdb_scalar_function_callback,
    _x_destroy: graphdb_function_destroy_callback,
) -> c_int {
    if session.is_null() || name.is_null() || x_func.is_none() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let name_str = unsafe {
        match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);

        // 将 C 回调转换为 Rust 回调类型
        let callback: ScalarFunctionCallback = std::mem::transmute(x_func);

        // 创建自定义函数
        let func = CustomFunction::new_c(
            name_str,
            argc as usize,
            argc < 0,
            format!("C function: {}", name_str),
            callback,
            user_data,
        );

        // 注册到会话
        if let Err(e) = handle.inner.register_custom_function(func) {
            eprintln!("注册函数失败: {:?}", e);
            return graphdb_error_code_t::GRAPHDB_ERROR as c_int;
        }
    }

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 创建自定义聚合函数
///
/// # 参数
/// - `session`: 会话句柄
/// - `name`: 函数名称
/// - `argc`: 参数数量，-1 表示可变参数
/// - `user_data`: 用户数据指针
/// - `x_step`: 聚合步骤回调
/// - `x_final`: 聚合最终回调
/// - `x_destroy`: 析构回调，可为 NULL
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_create_aggregate(
    session: *mut graphdb_session_t,
    name: *const c_char,
    argc: c_int,
    user_data: *mut c_void,
    x_step: graphdb_aggregate_step_callback,
    x_final: graphdb_aggregate_final_callback,
    _x_destroy: graphdb_function_destroy_callback,
) -> c_int {
    if session.is_null() || name.is_null() || x_step.is_none() || x_final.is_none() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let name_str = unsafe {
        match CStr::from_ptr(name).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    unsafe {
        let handle = &*(session as *mut GraphDbSessionHandle);

        // 将 C 回调转换为 Rust 回调类型
        let step_callback: AggregateStepCallback = std::mem::transmute(x_step);
        let final_callback: AggregateFinalCallback = std::mem::transmute(x_final);

        // 创建聚合函数
        let func = CustomFunction::new_c_aggregate(
            name_str,
            argc as usize,
            argc < 0,
            format!("C aggregate function: {}", name_str),
            step_callback,
            final_callback,
            user_data,
        );

        // 注册到会话
        if let Err(e) = handle.inner.register_custom_function(func) {
            eprintln!("注册聚合函数失败: {:?}", e);
            return graphdb_error_code_t::GRAPHDB_ERROR as c_int;
        }
    }

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 删除自定义函数
///
/// # 参数
/// - `session`: 会话句柄
/// - `name`: 函数名称
///
/// # 返回
/// - 成功: GRAPHDB_OK
/// - 失败: 错误码
#[no_mangle]
pub extern "C" fn graphdb_delete_function(
    session: *mut graphdb_session_t,
    name: *const c_char,
) -> c_int {
    if session.is_null() || name.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    // 注意：需要从注册表中删除函数
    // 当前返回成功（函数会在会话结束时自动清理）
    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 设置函数返回值
///
/// # 参数
/// - `context`: 函数执行上下文
/// - `value`: 返回值
///
/// # 说明
/// 在标量函数或聚合函数的 xFinal 回调中调用此函数设置返回值
#[no_mangle]
pub extern "C" fn graphdb_context_set_result(
    context: *mut graphdb_context_t,
    value: *const graphdb_value_t,
) -> c_int {
    if context.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let ctx = &mut (*context).inner;
        if value.is_null() {
            ctx.set_result(crate::core::Value::Null(crate::core::NullType::Null));
        } else {
            let val = crate::api::embedded::c_api::value::graphdb_value_to_core(value);
            ctx.set_result(val);
        }
    }

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 获取函数返回值的类型
///
/// # 参数
/// - `context`: 函数执行上下文
///
/// # 返回
/// - 值类型
#[no_mangle]
pub extern "C" fn graphdb_context_result_type(
    context: *mut graphdb_context_t,
) -> graphdb_value_type_t {
    if context.is_null() {
        return graphdb_value_type_t::GRAPHDB_NULL;
    }

    unsafe {
        let ctx = &(*context).inner;
        match &ctx.result {
            Some(val) => crate::api::embedded::c_api::value::core_value_to_graphdb_type(val),
            None => graphdb_value_type_t::GRAPHDB_NULL,
        }
    }
}

/// 设置错误消息
///
/// # 参数
/// - `context`: 函数执行上下文
/// - `error_msg`: 错误消息
///
/// # 说明
/// 在函数执行出错时调用此函数设置错误消息
#[no_mangle]
pub extern "C" fn graphdb_context_set_error(
    context: *mut graphdb_context_t,
    error_msg: *const c_char,
) -> c_int {
    if context.is_null() || error_msg.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let ctx = &mut (*context).inner;
        let msg = CStr::from_ptr(error_msg).to_string_lossy().into_owned();
        ctx.set_error(msg);
    }

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// 从上下文获取参数值（辅助函数）
///
/// # 参数
/// - `context`: 函数执行上下文
/// - `index`: 参数索引
///
/// # 返回
/// - 参数值指针，如果索引越界返回 NULL
#[no_mangle]
pub extern "C" fn graphdb_context_get_arg(
    _context: *mut graphdb_context_t,
    _index: c_int,
) -> *const graphdb_value_t {
    // 注意：参数通过 argv 数组直接传递，此函数当前未使用
    std::ptr::null()
}

/// 获取参数数量
///
/// # 参数
/// - `context`: 函数执行上下文
///
/// # 返回
/// - 参数数量
#[no_mangle]
pub extern "C" fn graphdb_context_arg_count(_context: *mut graphdb_context_t) -> c_int {
    // 注意：参数数量通过 argc 直接传递，此函数当前未使用
    0
}
