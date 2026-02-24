//! 表达式类型定义
//!
//! 本模块定义查询引擎中使用的统一表达式类型 `Expression` 枚举。

use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
pub use crate::core::types::DataType;
use crate::core::Value;
use serde::{Deserialize, Serialize};

/// 统一表达式类型
///
/// 包含位置信息（`span` 字段）的表达式枚举，用于：
/// - Parser 层：错误定位和报告
/// - Core 层：类型检查和执行
/// - 序列化：存储和传输
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    /// 字面量值
    Literal(Value),

    /// 变量引用
    Variable(String),

    /// 属性访问
    Property {
        object: Box<Expression>,
        property: String,
    },

    /// 二元运算
    Binary {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },

    /// 一元运算
    Unary {
        op: UnaryOperator,
        operand: Box<Expression>,
    },

    /// 函数调用
    Function {
        name: String,
        args: Vec<Expression>,
    },

    /// 聚合函数
    Aggregate {
        func: AggregateFunction,
        arg: Box<Expression>,
        distinct: bool,
    },

    /// 列表字面量
    List(Vec<Expression>),

    /// 映射字面量
    Map(Vec<(String, Expression)>),

    /// 条件表达式
    Case {
        test_expr: Option<Box<Expression>>,
        conditions: Vec<(Expression, Expression)>,
        default: Option<Box<Expression>>,
    },

    /// 类型转换
    TypeCast {
        expression: Box<Expression>,
        target_type: DataType,
    },

    /// 下标访问
    Subscript {
        collection: Box<Expression>,
        index: Box<Expression>,
    },

    /// 范围表达式
    Range {
        collection: Box<Expression>,
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
    },

    /// 路径表达式
    Path(Vec<Expression>),

    /// 标签表达式
    Label(String),

    /// 列表推导表达式
    ListComprehension {
        variable: String,
        source: Box<Expression>,
        filter: Option<Box<Expression>>,
        map: Option<Box<Expression>>,
    },

    /// 标签属性动态访问
    ///
    /// 用于动态访问标签属性，如 `tagName.propertyName`
    /// 其中 tagName 是一个变量或标签表达式
    LabelTagProperty {
        tag: Box<Expression>,
        property: String,
    },

    /// 标签属性访问
    ///
    /// 用于访问顶点标签上的属性，如 `tagName.propertyName`
    TagProperty {
        tag_name: String,
        property: String,
    },

    /// 边属性访问
    ///
    /// 用于访问边类型上的属性
    EdgeProperty {
        edge_name: String,
        property: String,
    },

    /// 谓词表达式
    ///
    /// 用于实现 FILTER、ALL、ANY、EXISTS 等谓词函数
    Predicate {
        func: String,
        args: Vec<Expression>,
    },

    /// Reduce 表达式
    ///
    /// 用于实现 REDUCE 函数
    Reduce {
        accumulator: String,
        initial: Box<Expression>,
        variable: String,
        source: Box<Expression>,
        mapping: Box<Expression>,
    },

    /// 路径构建表达式
    ///
    /// 用于构建路径，如 `path(v1, e1, v2)`
    PathBuild(Vec<Expression>),

    /// 查询参数表达式
    ///
    /// 用于表示查询参数，如 `$param`
    Parameter(String),
}
