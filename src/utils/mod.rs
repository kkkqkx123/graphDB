// 工具模块 - 仅用于导出各个子模块，不包含具体实现

// 对象池模块
pub mod object_pool;
pub use object_pool::ObjectPool;

// ID工具模块

// 日志模块
pub mod logger;
pub use logger::Logger;

// 字符串工具模块
pub mod string_utils;
pub use string_utils::{
    escape_for_query, normalize_identifier, sanitize_input, unescape_for_query,
};

// 类型转换工具模块
pub mod type_utils;
pub use type_utils::{value_to_bool, value_to_f64, value_to_i64, value_to_string};

// 表达式工具模块 - temporarily commented out until expressions module issue is resolved
// pub mod expression_utils;
// pub use expression_utils::{
//     expr_check,
//     expr_rewrite,
//     expr_transform,
//     expr_collect,
//     AliasType,
// };
// pub use crate::expressions::ExpressionKind;
pub use crate::core::Value;

// 匿名变量生成器模块
pub mod anon_var_generator;
pub use anon_var_generator::AnonVarGenerator;

// 错误处理辅助函数模块
pub mod error_handling;
pub use error_handling::{
    expect_arc_mut, expect_first, expect_last, expect_max, expect_min, expect_option,
    expect_result, expect_vec_first, expect_vec_last, safe_lock, safe_read, safe_write,
};
