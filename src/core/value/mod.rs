//! Value Module - Graph Database Value Type System
//!
//! This module provides the core value type system in the graph database.
//!
//! ## Module Structure
//!
//! - `null` - NullType 定义
//! - `value` - Value 枚举定义及基础方法
//! - `value_compare` - 比较逻辑 (PartialEq, Eq, Ord, Hash)
//! - `value_arithmetic` - 算术/逻辑/位运算
//! - `value_convert` - 类型转换
//! - `list` - 列表类型
//! - `dataset` - 数据集类型
//! - `date_time` - 日期时间类型
//! - `decimal128` - Decimal128 高精度数值
//! - `geography` - 地理空间类型
//! - `memory` - 内存估算

#[allow(non_snake_case)]
pub mod dataset;
pub mod date_time;
pub mod decimal128;
pub mod geography;
pub mod list;
pub mod memory;
pub mod null;
pub mod value_arithmetic;
pub mod value_compare;
pub mod value_convert;
pub mod value_def;
pub mod vector;

// Re-export all public types
pub use dataset::DataSet;
pub use date_time::{DateTimeValue, DateValue, DurationValue, TimeValue};
pub use decimal128::Decimal128Value;
pub use geography::GeographyValue;
pub use list::List;
pub use null::NullType;
pub use value_def::Value;
pub use vector::VectorValue;
