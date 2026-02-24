//! 内置函数实现模块
//!
//! 提供所有内置函数的具体实现，按功能分类组织
//!
//! 注意：函数注册现在通过 FunctionRegistry::register_all_builtin_functions 直接完成
//! 使用静态分发机制，通过 BuiltinFunction 枚举直接调用函数

// 宏模块必须先加载，供其他模块使用
#[macro_use]
pub mod macros;

pub mod math;
pub mod string;
pub mod conversion;
pub mod regex;
pub mod datetime;
pub mod aggregate;
pub mod graph;
pub mod container;
pub mod path;
pub mod utility;
pub mod geography;
