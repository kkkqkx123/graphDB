//! 表达式类型定义 V2 - 优化版本
//!
//! 使用枚举变体减少装箱，优化内存使用和性能

use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::{NullType, Value};
use serde::{Deserialize, Serialize};

/// 优化后的表达式类型
///
/// 使用枚举变体减少装箱，提高内存效率和性能
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    // 字面量
    Literal(Value),

    // 变量和属性
    Variable(String),
    Property {
        object: Box<Expression>,
        property: String,
    },

    // 二元操作
    Binary {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },

    // 一元操作
    Unary {
        op: UnaryOperator,
        operand: Box<Expression>,
    },

    // 函数调用
    Function {
        name: String,
        args: Vec<Expression>,
    },

    // 聚合函数
    Aggregate {
        func: AggregateFunction,
        arg: Box<Expression>,
        distinct: bool,
    },

    // 容器类型
    List(Vec<Expression>),
    Map(Vec<(String, Expression)>),

    // 条件表达式
    Case {
        conditions: Vec<(Expression, Expression)>,
        default: Option<Box<Expression>>,
    },

    // 类型转换
    TypeCast {
        expr: Box<Expression>,
        target_type: DataType,
    },

    // 下标访问
    Subscript {
        collection: Box<Expression>,
        index: Box<Expression>,
    },

    // 范围访问
    Range {
        collection: Box<Expression>,
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
    },

    // 路径构建
    Path(Vec<Expression>),

    // 标签
    Label(String),

    // 图数据库特有表达式类型
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

    // 文本搜索表达式
    ESQuery(String),

    // UUID表达式
    UUID,

    // 匹配路径模式表达式
    MatchPathPattern {
        path_alias: String,
        patterns: Vec<Expression>,
    },
}

// 二元操作符、一元操作符和聚合函数现在从 crate::core::types::operators 导入
// 以避免重复定义，参见 operators.rs

/// 数据类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Bool,
    Int,
    Float,
    String,
    List,
    Map,
    Vertex,
    Edge,
    Path,
    DateTime,
    Date, // 日期类型
    Time, // 时间类型
    Duration, // 期间类型
          // 可以根据需要添加更多类型
}

/// 表达式类型分类
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpressionType {
    Literal,
    Variable,
    Property,
    Binary,
    Unary,
    Function,
    Aggregate,
    List,
    Map,
    Case,
    TypeCast,
    Subscript,
    Range,
    Path,
    Label,
    TagProperty,
    EdgeProperty,
    InputProperty,
    VariableProperty,
    SourceProperty,
    DestinationProperty,
}

impl Expression {
    /// 创建字面量表达式
    pub fn literal(value: impl Into<Value>) -> Self {
        Expression::Literal(value.into())
    }

    /// 创建变量表达式
    pub fn variable(name: impl Into<String>) -> Self {
        Expression::Variable(name.into())
    }

    /// 创建属性访问表达式
    pub fn property(object: Expression, property: impl Into<String>) -> Self {
        Expression::Property {
            object: Box::new(object),
            property: property.into(),
        }
    }

    /// 创建二元操作表达式
    pub fn binary(left: Expression, op: BinaryOperator, right: Expression) -> Self {
        Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    /// 创建一元操作表达式
    pub fn unary(op: UnaryOperator, operand: Expression) -> Self {
        Expression::Unary {
            op,
            operand: Box::new(operand),
        }
    }

    /// 创建函数调用表达式
    pub fn function(name: impl Into<String>, args: Vec<Expression>) -> Self {
        Expression::Function {
            name: name.into(),
            args,
        }
    }

    /// 创建聚合函数表达式
    pub fn aggregate(func: AggregateFunction, arg: Expression, distinct: bool) -> Self {
        Expression::Aggregate {
            func,
            arg: Box::new(arg),
            distinct,
        }
    }

    /// 创建列表表达式
    pub fn list(items: Vec<Expression>) -> Self {
        Expression::List(items)
    }

    /// 创建映射表达式
    pub fn map(pairs: Vec<(impl Into<String>, Expression)>) -> Self {
        Expression::Map(pairs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    /// 创建条件表达式
    pub fn case(conditions: Vec<(Expression, Expression)>, default: Option<Expression>) -> Self {
        Expression::Case {
            conditions,
            default: default.map(Box::new),
        }
    }

    /// 创建类型转换表达式
    pub fn cast(expr: Expression, target_type: DataType) -> Self {
        Expression::TypeCast {
            expr: Box::new(expr),
            target_type,
        }
    }

    /// 创建下标访问表达式
    pub fn subscript(collection: Expression, index: Expression) -> Self {
        Expression::Subscript {
            collection: Box::new(collection),
            index: Box::new(index),
        }
    }

    /// 创建范围访问表达式
    pub fn range(
        collection: Expression,
        start: Option<Expression>,
        end: Option<Expression>,
    ) -> Self {
        Expression::Range {
            collection: Box::new(collection),
            start: start.map(Box::new),
            end: end.map(Box::new),
        }
    }

    /// 创建路径表达式
    pub fn path(items: Vec<Expression>) -> Self {
        Expression::Path(items)
    }

    /// 创建标签表达式
    pub fn label(name: impl Into<String>) -> Self {
        Expression::Label(name.into())
    }

    /// 获取表达式的子表达式
    pub fn children(&self) -> Vec<&Expression> {
        match self {
            Expression::Literal(_) => vec![],
            Expression::Variable(_) => vec![],
            Expression::Property { object, .. } => vec![object.as_ref()],
            Expression::Binary { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            Expression::Unary { operand, .. } => vec![operand.as_ref()],
            Expression::Function { args, .. } => args.iter().collect(),
            Expression::Aggregate { arg, .. } => vec![arg.as_ref()],
            Expression::List(items) => items.iter().collect(),
            Expression::Map(pairs) => pairs.iter().map(|(_, expr)| expr).collect(),
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
                    children.push(def);
                }
                children
            }
            Expression::TypeCast { expr, .. } => vec![expr.as_ref()],
            Expression::Subscript { collection, index } => {
                vec![collection.as_ref(), index.as_ref()]
            }
            Expression::Range {
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
            Expression::Path(items) => items.iter().collect(),
            Expression::Label(_) => vec![],

            // 图数据库特有表达式
            Expression::TagProperty { .. } => vec![],
            Expression::EdgeProperty { .. } => vec![],
            Expression::InputProperty(_) => vec![],
            Expression::VariableProperty { .. } => vec![],
            Expression::SourceProperty { .. } => vec![],
            Expression::DestinationProperty { .. } => vec![],

            // 一元操作扩展
            Expression::UnaryPlus(expr) => vec![expr.as_ref()],
            Expression::UnaryNegate(expr) => vec![expr.as_ref()],
            Expression::UnaryNot(expr) => vec![expr.as_ref()],
            Expression::UnaryIncr(expr) => vec![expr.as_ref()],
            Expression::UnaryDecr(expr) => vec![expr.as_ref()],
            Expression::IsNull(expr) => vec![expr.as_ref()],
            Expression::IsNotNull(expr) => vec![expr.as_ref()],
            Expression::IsEmpty(expr) => vec![expr.as_ref()],
            Expression::IsNotEmpty(expr) => vec![expr.as_ref()],

            // 列表推导
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

            // 谓词表达式
            Expression::Predicate { list, condition } => {
                vec![list.as_ref(), condition.as_ref()]
            }

            // 归约表达式
            Expression::Reduce {
                list,
                initial,
                expr,
                ..
            } => {
                vec![list.as_ref(), initial.as_ref(), expr.as_ref()]
            }

            // 文本搜索表达式
            Expression::ESQuery(_) => vec![],

            // UUID表达式
            Expression::UUID => vec![],

            // 匹配路径模式表达式
            Expression::MatchPathPattern { patterns, .. } => patterns.iter().collect(),
        }
    }

    /// 获取表达式的类型
    pub fn expression_type(&self) -> ExpressionType {
        match self {
            Expression::Literal(_) => ExpressionType::Literal,
            Expression::Variable(_) => ExpressionType::Variable,
            Expression::Property { .. } => ExpressionType::Property,
            Expression::Binary { .. } => ExpressionType::Binary,
            Expression::Unary { .. } => ExpressionType::Unary,
            Expression::Function { .. } => ExpressionType::Function,
            Expression::Aggregate { .. } => ExpressionType::Aggregate,
            Expression::List(_) => ExpressionType::List,
            Expression::Map(_) => ExpressionType::Map,
            Expression::Case { .. } => ExpressionType::Case,
            Expression::TypeCast { .. } => ExpressionType::TypeCast,
            Expression::Subscript { .. } => ExpressionType::Subscript,
            Expression::Range { .. } => ExpressionType::Range,
            Expression::Path(_) => ExpressionType::Path,
            Expression::Label(_) => ExpressionType::Label,

            // 图数据库特有表达式
            Expression::TagProperty { .. } => ExpressionType::Property,
            Expression::EdgeProperty { .. } => ExpressionType::Property,
            Expression::InputProperty(_) => ExpressionType::Property,
            Expression::VariableProperty { .. } => ExpressionType::Property,
            Expression::SourceProperty { .. } => ExpressionType::Property,
            Expression::DestinationProperty { .. } => ExpressionType::Property,

            // 一元操作扩展
            Expression::UnaryPlus(_) => ExpressionType::Unary,
            Expression::UnaryNegate(_) => ExpressionType::Unary,
            Expression::UnaryNot(_) => ExpressionType::Unary,
            Expression::UnaryIncr(_) => ExpressionType::Unary,
            Expression::UnaryDecr(_) => ExpressionType::Unary,
            Expression::IsNull(_) => ExpressionType::Unary,
            Expression::IsNotNull(_) => ExpressionType::Unary,
            Expression::IsEmpty(_) => ExpressionType::Unary,
            Expression::IsNotEmpty(_) => ExpressionType::Unary,

            // 列表推导
            Expression::ListComprehension { .. } => ExpressionType::List,

            // 谓词表达式
            Expression::Predicate { .. } => ExpressionType::Property,

            // 归约表达式
            Expression::Reduce { .. } => ExpressionType::Aggregate,

            // 文本搜索表达式
            Expression::ESQuery(_) => ExpressionType::Function,

            // UUID表达式
            Expression::UUID => ExpressionType::Literal,

            // 匹配路径模式表达式
            Expression::MatchPathPattern { .. } => ExpressionType::Path,
        }
    }

    /// 检查表达式是否为常量
    pub fn is_constant(&self) -> bool {
        match self {
            Expression::Literal(_) => true,
            Expression::List(items) => items.iter().all(|e| e.is_constant()),
            Expression::Map(pairs) => pairs.iter().all(|(_, e)| e.is_constant()),
            _ => false,
        }
    }

    /// 检查表达式是否包含聚合函数
    pub fn contains_aggregate(&self) -> bool {
        match self {
            Expression::Aggregate { .. } => true,
            _ => self.children().iter().any(|e| e.contains_aggregate()),
        }
    }

    /// 获取表达式中使用的所有变量
    pub fn get_variables(&self) -> Vec<String> {
        let mut variables = Vec::new();
        self.collect_variables(&mut variables);
        variables.sort();
        variables.dedup();
        variables
    }

    fn collect_variables(&self, variables: &mut Vec<String>) {
        match self {
            Expression::Variable(name) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
            }
            _ => {
                for child in self.children() {
                    child.collect_variables(variables);
                }
            }
        }
    }
}

// 便捷的构建器方法
impl Expression {
    /// 创建布尔字面量
    pub fn bool(value: bool) -> Self {
        Expression::Literal(Value::Bool(value))
    }

    /// 创建整数字面量
    pub fn int(value: i64) -> Self {
        Expression::Literal(Value::Int(value))
    }

    /// 创建浮点数字面量
    pub fn float(value: f64) -> Self {
        Expression::Literal(Value::Float(value))
    }

    /// 创建字符串字面量
    pub fn string(value: impl Into<String>) -> Self {
        Expression::Literal(Value::String(value.into()))
    }

    /// 创建空值
    pub fn null() -> Self {
        Expression::Literal(Value::Null(NullType::Null))
    }

    /// 创建等于比较
    pub fn eq(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Equal, right)
    }

    /// 创建不等于比较
    pub fn ne(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::NotEqual, right)
    }

    /// 创建小于比较
    pub fn lt(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::LessThan, right)
    }

    /// 创建小于等于比较
    pub fn le(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::LessThanOrEqual, right)
    }

    /// 创建大于比较
    pub fn gt(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::GreaterThan, right)
    }

    /// 创建大于等于比较
    pub fn ge(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::GreaterThanOrEqual, right)
    }

    /// 创建加法
    pub fn add(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Add, right)
    }

    /// 创建减法
    pub fn sub(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Subtract, right)
    }

    /// 创建乘法
    pub fn mul(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Multiply, right)
    }

    /// 创建除法
    pub fn div(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Divide, right)
    }

    /// 创建逻辑与
    pub fn and(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::And, right)
    }

    /// 创建逻辑或
    pub fn or(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Or, right)
    }

    /// 创建逻辑非
    pub fn not(expr: Expression) -> Self {
        Self::unary(UnaryOperator::Not, expr)
    }

    /// 创建空值检查
    pub fn is_null(expr: Expression) -> Self {
        Self::unary(UnaryOperator::IsNull, expr)
    }

    /// 创建非空值检查
    pub fn is_not_null(expr: Expression) -> Self {
        Self::unary(UnaryOperator::IsNotNull, expr)
    }
}
