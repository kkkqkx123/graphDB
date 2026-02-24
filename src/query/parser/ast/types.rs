//! AST 基础类型定义
//!
//! 本模块定义查询 AST 特有的类型，包括标签、属性引用、子句结构等。
//! 同时重导出 core 模块的类型以方便使用。

pub use crate::core::types::operators::AggregateFunction as CoreAggregateFunction;
pub use crate::core::types::{EdgeDirection, OrderDirection};

pub use crate::core::types::Span;

pub type BinaryOp = crate::core::types::operators::BinaryOperator;
pub type UnaryOp = crate::core::types::operators::UnaryOperator;
pub type DataType = crate::core::types::DataType;
pub type AggregateFunction = CoreAggregateFunction;

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PropertyRef {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitClause {
    pub span: Span,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SkipClause {
    pub span: Span,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampleClause {
    pub span: Span,
    pub count: usize,
    pub percentage: Option<f64>,
}
