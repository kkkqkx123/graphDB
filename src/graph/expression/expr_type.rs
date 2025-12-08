use super::binary::BinaryOperator;
use super::unary::UnaryOperator;
use crate::core::Value;
use serde::{Deserialize, Serialize};

/// Represents an expression in a query
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Expression {
    Constant(Value),
    Property(String),                  // Property name to access
    Function(String, Vec<Expression>), // Function name and arguments
    BinaryOp(Box<Expression>, BinaryOperator, Box<Expression>),
    UnaryOp(UnaryOperator, Box<Expression>),

    // 新增的表达式类型，以匹配NebulaGraph
    // 属性相关表达式
    TagProperty {
        tag: String,
        prop: String,
    }, // tagName.propName
    EdgeProperty {
        edge: String,
        prop: String,
    }, // edgeName.propName
    InputProperty(String), // $-.propName
    VariableProperty {
        var: String,
        prop: String,
    }, // $varName.propName
    SourceProperty {
        tag: String,
        prop: String,
    }, // $^.tagName.propName
    DestinationProperty {
        tag: String,
        prop: String,
    }, // $$.tagName.propName

    // 一元操作扩展
    UnaryPlus(Box<Expression>),
    UnaryNegate(Box<Expression>),
    UnaryNot(Box<Expression>),
    UnaryIncr(Box<Expression>),
    UnaryDecr(Box<Expression>),
    IsNull(Box<Expression>),
    IsNotNull(Box<Expression>),
    IsEmpty(Box<Expression>),
    IsNotEmpty(Box<Expression>),

    // 容器表达式
    List(Vec<Expression>),
    Set(Vec<Expression>),
    Map(Vec<(String, Expression)>),

    // 类型转换
    TypeCasting {
        expr: Box<Expression>,
        target_type: String,
    },

    // 条件表达式
    Case {
        conditions: Vec<(Expression, Expression)>,
        default: Option<Box<Expression>>,
    },

    // 聚合表达式
    Aggregate {
        func: String,
        arg: Box<Expression>,
        distinct: bool,
    },

    // 列表推导
    ListComprehension {
        generator: Box<Expression>,
        condition: Option<Box<Expression>>,
    },

    // 谓词表达式
    Predicate {
        list: Box<Expression>,
        condition: Box<Expression>,
    },

    // 归约表达式
    Reduce {
        list: Box<Expression>,
        var: String,
        initial: Box<Expression>,
        expr: Box<Expression>,
    },

    // 路径构建表达式
    PathBuild(Vec<Expression>),

    // 文本搜索表达式
    ESQuery(String),

    // UUID表达式
    UUID,

    // 变量表达式
    Variable(String),

    // 下标表达式
    Subscript {
        collection: Box<Expression>,
        index: Box<Expression>,
    },

    // 下标范围表达式
    SubscriptRange {
        collection: Box<Expression>,
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
    },

    // 标签表达式
    Label(String),

    // 匹配路径模式表达式
    MatchPathPattern {
        path_alias: String,
        patterns: Vec<Expression>,
    },
}

impl Expression {
    /// Get the kind of this expression
    pub fn kind(&self) -> ExpressionKind {
        match self {
            Expression::Constant(_) => ExpressionKind::Constant,
            Expression::Property(_) => ExpressionKind::Variable,
            Expression::Function(name, _) => {
                // Could be more specific based on function name, but for now we'll use FunctionCall
                ExpressionKind::FunctionCall
            }
            Expression::BinaryOp(_, _, _) => ExpressionKind::Arithmetic, // Could be more specific based on operator
            Expression::UnaryOp(op, _) => match op {
                crate::graph::expression::unary::UnaryOperator::Plus => ExpressionKind::UnaryPlus,
                crate::graph::expression::unary::UnaryOperator::Minus => {
                    ExpressionKind::UnaryNegate
                }
                crate::graph::expression::unary::UnaryOperator::Not => ExpressionKind::UnaryNot,
                crate::graph::expression::unary::UnaryOperator::Increment => {
                    ExpressionKind::UnaryInvert
                }
                crate::graph::expression::unary::UnaryOperator::Decrement => {
                    ExpressionKind::UnaryInvert
                }
            },

            // 新增表达式类型对应的kind
            Expression::TagProperty { .. } => ExpressionKind::TagProperty,
            Expression::EdgeProperty { .. } => ExpressionKind::EdgeProperty,
            Expression::InputProperty(_) => ExpressionKind::InputProperty,
            Expression::VariableProperty { .. } => ExpressionKind::VariableProperty,
            Expression::SourceProperty { .. } => ExpressionKind::SourceProperty,
            Expression::DestinationProperty { .. } => ExpressionKind::DestinationProperty,

            Expression::UnaryPlus(_) => ExpressionKind::UnaryPlus,
            Expression::UnaryNegate(_) => ExpressionKind::UnaryNegate,
            Expression::UnaryNot(_) => ExpressionKind::UnaryNot,
            Expression::UnaryIncr(_) => ExpressionKind::UnaryInvert,
            Expression::UnaryDecr(_) => ExpressionKind::UnaryInvert,
            Expression::IsNull(_) => ExpressionKind::UnaryNot,
            Expression::IsNotNull(_) => ExpressionKind::UnaryNot,
            Expression::IsEmpty(_) => ExpressionKind::UnaryNot,
            Expression::IsNotEmpty(_) => ExpressionKind::UnaryNot,

            Expression::List(_) => ExpressionKind::List,
            Expression::Set(_) => ExpressionKind::Set,
            Expression::Map(_) => ExpressionKind::Map,

            Expression::TypeCasting { .. } => ExpressionKind::TypeCasting,

            Expression::Case { .. } => ExpressionKind::Relational,

            Expression::Aggregate { .. } => ExpressionKind::Aggregate,

            Expression::ListComprehension { .. } => ExpressionKind::Container,

            Expression::Predicate { .. } => ExpressionKind::Logical,

            Expression::Reduce { .. } => ExpressionKind::Logical,

            Expression::PathBuild(_) => ExpressionKind::Relational,

            Expression::ESQuery(_) => ExpressionKind::Relational,

            Expression::UUID => ExpressionKind::Constant,

            Expression::Variable(_) => ExpressionKind::Variable,

            Expression::Subscript { .. } => ExpressionKind::Relational,

            Expression::SubscriptRange { .. } => ExpressionKind::Relational,

            Expression::Label(_) => ExpressionKind::Label,

            Expression::MatchPathPattern { .. } => ExpressionKind::Relational,
        }
    }

    /// Get child expressions
    pub fn children(&self) -> Vec<&Expression> {
        match self {
            Expression::Constant(_) => vec![],
            Expression::Property(_) => vec![],
            Expression::Function(_, args) => args.iter().collect(),
            Expression::BinaryOp(left, _, right) => {
                vec![left.as_ref(), right.as_ref()]
            }
            Expression::UnaryOp(_, operand) => {
                vec![operand.as_ref()]
            }

            // 新增表达式的子表达式
            Expression::TagProperty { .. } => vec![],
            Expression::EdgeProperty { .. } => vec![],
            Expression::InputProperty(_) => vec![],
            Expression::VariableProperty { .. } => vec![],
            Expression::SourceProperty { .. } => vec![],
            Expression::DestinationProperty { .. } => vec![],

            Expression::UnaryPlus(operand) => vec![operand.as_ref()],
            Expression::UnaryNegate(operand) => vec![operand.as_ref()],
            Expression::UnaryNot(operand) => vec![operand.as_ref()],
            Expression::UnaryIncr(operand) => vec![operand.as_ref()],
            Expression::UnaryDecr(operand) => vec![operand.as_ref()],
            Expression::IsNull(operand) => vec![operand.as_ref()],
            Expression::IsNotNull(operand) => vec![operand.as_ref()],
            Expression::IsEmpty(operand) => vec![operand.as_ref()],
            Expression::IsNotEmpty(operand) => vec![operand.as_ref()],

            Expression::List(items) => items.iter().collect(),
            Expression::Set(items) => items.iter().collect(),
            Expression::Map(items) => items.iter().map(|(_, expr)| expr).collect(),

            Expression::TypeCasting { expr, .. } => vec![expr.as_ref()],

            Expression::Case {
                conditions,
                default,
            } => {
                let mut children = Vec::new();
                for (cond, value) in conditions {
                    children.push(cond);
                    children.push(value);
                }
                if let Some(def) = default {
                    children.push(def.as_ref());
                }
                children
            }

            Expression::Aggregate { arg, .. } => vec![arg.as_ref()],

            Expression::ListComprehension {
                generator,
                condition,
            } => {
                let mut children = vec![generator.as_ref()];
                if let Some(cond) = condition {
                    children.push(cond.as_ref());
                }
                children
            }

            Expression::Predicate { list, condition } => vec![list.as_ref(), condition.as_ref()],

            Expression::Reduce {
                list,
                initial,
                expr,
                ..
            } => vec![list.as_ref(), initial.as_ref(), expr.as_ref()],

            Expression::PathBuild(items) => items.iter().collect(),

            Expression::ESQuery(_) => vec![],

            Expression::UUID => vec![],

            Expression::Variable(_) => vec![],

            Expression::Subscript { collection, index } => {
                vec![collection.as_ref(), index.as_ref()]
            }

            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                let mut children = vec![collection.as_ref()];
                if let Some(s) = start {
                    children.push(s.as_ref());
                }
                if let Some(e) = end {
                    children.push(e.as_ref());
                }
                children
            }

            Expression::Label(_) => vec![],

            Expression::MatchPathPattern { patterns, .. } => patterns.iter().collect(),
        }
    }
}

/// A simplified version of the ExpressionKind enum for expression analysis
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpressionKind {
    // 属性表达式类型
    TagProperty,
    EdgeProperty,
    InputProperty,
    VariableProperty,
    DestinationProperty,
    SourceProperty,

    // 二元表达式类型
    Arithmetic,
    Relational,
    Logical,

    // 一元表达式类型
    UnaryPlus,
    UnaryNegate,
    UnaryNot,
    UnaryInvert,

    // 函数调式
    FunctionCall,

    // 常量
    Constant,

    // 变量
    Variable,

    // 参数 (simplified as Variable for now)
    Parameter,

    // 其他类型
    Aggregate,
    TypeCasting,
    Label,
    Container,

    // 容器类型
    List,
    Set,
    Map,
}
