//! 内置函数实现模块
//!
//! 提供所有内置函数的具体实现，按功能分类组织

pub mod math;
pub mod string;
pub mod conversion;
pub mod regex;
pub mod datetime;
pub mod aggregate;

use super::registry::FunctionRegistry;

/// 注册所有内置函数
pub fn register_all(registry: &mut FunctionRegistry) {
    math::register_all(registry);
    string::register_all(registry);
    conversion::register_all(registry);
    regex::register_all(registry);
    datetime::register_all(registry);
}
