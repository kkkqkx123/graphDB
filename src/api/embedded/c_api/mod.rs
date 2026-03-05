//! C API 模块
//!
//! 提供 GraphDB 的 C 语言接口，允许 C/C++ 程序调用 GraphDB 功能

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::missing_safety_doc)]

pub mod types;
pub mod error;

pub use types::*;
pub use error::*;
