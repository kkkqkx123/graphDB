//! Graph utility module - 提供图相关的工具函数
//! 对应原C++中graph/util目录下的工具类

pub mod id_utils;
pub use id_utils::{generate_id, is_valid_id};

pub mod id_generator;
pub use id_generator::{EPIdGenerator, IdGenerator, INVALID_ID};
