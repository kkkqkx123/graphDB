//! 表达式构造函数
//!
//! 提供创建各类表达式的工厂方法。

use crate::core::types::expression::Expression;
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::types::DataType;
use crate::core::{NullType, Value};

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

    /// 创建二元运算表达式
    pub fn binary(left: Expression, op: BinaryOperator, right: Expression) -> Self {
        Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    /// 创建一元运算表达式
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
    pub fn case(
        test_expr: Option<Expression>,
        conditions: Vec<(Expression, Expression)>,
        default: Option<Expression>,
    ) -> Self {
        Expression::Case {
            test_expr: test_expr.map(Box::new),
            conditions,
            default: default.map(Box::new),
        }
    }

    /// 创建类型转换表达式
    pub fn cast(expression: Expression, target_type: DataType) -> Self {
        Expression::TypeCast {
            expression: Box::new(expression),
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

    /// 创建范围表达式
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

    /// 创建列表推导表达式
    pub fn list_comprehension(
        variable: impl Into<String>,
        source: Expression,
        filter: Option<Expression>,
        map: Option<Expression>,
    ) -> Self {
        Expression::ListComprehension {
            variable: variable.into(),
            source: Box::new(source),
            filter: filter.map(Box::new),
            map: map.map(Box::new),
        }
    }

    /// 创建标签属性动态访问表达式
    pub fn label_tag_property(tag: Expression, property: impl Into<String>) -> Self {
        Expression::LabelTagProperty {
            tag: Box::new(tag),
            property: property.into(),
        }
    }

    /// 创建标签属性访问表达式
    pub fn tag_property(tag_name: impl Into<String>, property: impl Into<String>) -> Self {
        Expression::TagProperty {
            tag_name: tag_name.into(),
            property: property.into(),
        }
    }

    /// 创建边属性访问表达式
    pub fn edge_property(edge_name: impl Into<String>, property: impl Into<String>) -> Self {
        Expression::EdgeProperty {
            edge_name: edge_name.into(),
            property: property.into(),
        }
    }

    /// 创建谓词表达式
    pub fn predicate(func: impl Into<String>, args: Vec<Expression>) -> Self {
        Expression::Predicate {
            func: func.into(),
            args,
        }
    }

    /// 创建 Reduce 表达式
    pub fn reduce(
        accumulator: impl Into<String>,
        initial: Expression,
        variable: impl Into<String>,
        source: Expression,
        mapping: Expression,
    ) -> Self {
        Expression::Reduce {
            accumulator: accumulator.into(),
            initial: Box::new(initial),
            variable: variable.into(),
            source: Box::new(source),
            mapping: Box::new(mapping),
        }
    }

    /// 创建路径构建表达式
    pub fn path_build(items: Vec<Expression>) -> Self {
        Expression::PathBuild(items)
    }

    /// 创建参数表达式
    pub fn parameter(name: impl Into<String>) -> Self {
        Expression::Parameter(name.into())
    }

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

    /// 创建空值字面量
    pub fn null() -> Self {
        Expression::Literal(Value::Null(NullType::Null))
    }

    /// 创建加法表达式
    pub fn add(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Add, right)
    }

    /// 创建减法表达式
    pub fn sub(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Subtract, right)
    }

    /// 创建乘法表达式
    pub fn mul(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Multiply, right)
    }

    /// 创建除法表达式
    pub fn div(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Divide, right)
    }

    /// 创建取模表达式
    pub fn modulo(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Modulo, right)
    }

    /// 创建等于表达式
    pub fn eq(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Equal, right)
    }

    /// 创建不等于表达式
    pub fn ne(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::NotEqual, right)
    }

    /// 创建小于表达式
    pub fn lt(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::LessThan, right)
    }

    /// 创建小于等于表达式
    pub fn le(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::LessThanOrEqual, right)
    }

    /// 创建大于表达式
    pub fn gt(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::GreaterThan, right)
    }

    /// 创建大于等于表达式
    pub fn ge(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::GreaterThanOrEqual, right)
    }

    /// 创建逻辑与表达式
    pub fn and(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::And, right)
    }

    /// 创建逻辑或表达式
    pub fn or(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Or, right)
    }

    /// 创建逻辑非表达式
    pub fn not(operand: Expression) -> Self {
        Self::unary(UnaryOperator::Not, operand)
    }

    /// 创建负号表达式
    pub fn neg(operand: Expression) -> Self {
        Self::unary(UnaryOperator::Minus, operand)
    }

    /// 创建 IS NULL 表达式
    pub fn is_null(operand: Expression) -> Self {
        Self::unary(UnaryOperator::IsNull, operand)
    }

    /// 创建 IS NOT NULL 表达式
    pub fn is_not_null(operand: Expression) -> Self {
        Self::unary(UnaryOperator::IsNotNull, operand)
    }
}
