//! 统一表达式类型定义
//!
//! 本模块定义了查询引擎中使用的统一表达式类型 `Expression`。
//!
//! ## 设计说明
//!
//! `Expression` 是统一的表达式类型，结合了以下来源的特点：
//! - **Parser 层 AST**: 提供 `Span` 信息用于错误定位
//! - **Core 层表达式**: 提供序列化支持和聚合函数
//!
//! ## 类型特点
//!
//! - **位置信息**: 可选的 `Span` 字段用于错误报告
//! - **聚合函数**: 支持 `Aggregate` 变体用于聚合查询
//! - **序列化支持**: 通过 `serde` 支持序列化/反序列化
//!
//! ## 变体说明
//!
//! | 变体 | 用途 |
//! |------|------|
//! | `Literal` | 字面量值 |
//! | `Variable` | 变量引用 |
//! | `Property` | 属性访问 |
//! | `Binary` | 二元运算 |
//! | `Unary` | 一元运算 |
//! | `Function` | 函数调用 |
//! | `Aggregate` | 聚合函数 |
//! | `List` | 列表字面量 |
//! | `Map` | 映射字面量 |
//! | `Case` | 条件表达式 |
//! | `TypeCast` | 类型转换 |
//! | `Subscript` | 下标访问 |
//! | `Range` | 范围表达式 |
//! | `Path` | 路径表达式 |
//! | `Label` | 标签表达式 |
//!
//! ## 使用示例
//!
//! ```rust
//! use crate::core::types::expression::Expression;
//! use crate::core::types::operators::{BinaryOperator, AggregateFunction};
//! use crate::core::Value;
//!
//! // 简单字面量
//! let expression = Expression::literal(Value::Int(42));
//!
//! // 二元运算
//! let sum = Expression::add(Expression::variable("a"), Expression::variable("b"));
//!
//! // 聚合函数
//! let count = Expression::aggregate(
//!     AggregateFunction::Count,
//!     Expression::variable("col"),
//!     false
//! );
//! ```
//!
//! ## 上下文说明
//!
//! 本模块定义纯数据类型，不包含任何上下文。
//! 上下文相关的类型定义在 `query` 模块中：
//! - **`query::validator::context::ExpressionAnalysisContext`**: 编译时分析上下文，用于验证、优化器、类型推导等阶段
//! - **`query::executor::expression::evaluation_context::ExpressionContext`**: 运行时求值上下文 trait，用于表达式求值
//!
//! 请根据使用场景选择合适的上下文类型。

// 子模块定义
pub mod common_utils;
mod construction;
pub mod contextual;
mod def;
mod display;
pub mod expression;
mod inspection;
pub mod serializable;
mod traverse;
mod type_deduce;
pub mod utils;
pub mod visitor;
pub mod visitor_checkers;
pub mod visitor_collectors;

// 统一导出
pub use contextual::ContextualExpression;
pub use def::Expression;
pub use expression::{ExpressionId, ExpressionMeta};
pub use serializable::SerializableExpression;
pub use visitor::ExpressionVisitor;
pub use visitor_checkers::{ConstantChecker, PropertyContainsChecker};
pub use visitor_collectors::{
    FunctionCollector, OrConditionCollector, PropertyCollector, PropertyPredicate,
    PropertyPredicateCollector, VariableCollector,
};

// 重新导出工具类型
pub use common_utils::{
    extract_group_info, extract_property_refs, extract_string_from_expr,
    generate_default_alias_from_contextual, is_constant, is_constant_expression,
};
pub use utils::extract_group_suite;
pub use utils::GroupSuite;
