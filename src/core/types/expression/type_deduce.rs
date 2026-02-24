//! 表达式类型推导
//!
//! 提供表达式类型推导功能。

use crate::core::types::expression::Expression;
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::types::DataType;
use crate::core::Value;

impl Expression {
    /// 推导表达式的数据类型
    ///
    /// 根据表达式的结构和操作符推导其返回类型。
    /// 如果无法确定类型，返回 DataType::Empty。
    pub fn deduce_type(&self) -> DataType {
        match self {
            Expression::Literal(value) => Self::deduce_value_type(value),
            Expression::Variable(_) => DataType::Empty,
            Expression::Property { .. } => DataType::Empty,
            Expression::Binary { op, left, right } => {
                Self::deduce_binary_type(op, left, right)
            }
            Expression::Unary { op, operand } => {
                Self::deduce_unary_type(op, operand)
            }
            Expression::Function { name, args } => {
                Self::deduce_function_type(name, args)
            }
            Expression::Aggregate { func, .. } => {
                Self::deduce_aggregate_type(func)
            }
            Expression::List(_) => DataType::List,
            Expression::Map(_) => DataType::Map,
            Expression::Case { conditions, default, .. } => {
                Self::deduce_case_type(conditions, default.as_deref())
            }
            Expression::TypeCast { target_type, .. } => target_type.clone(),
            Expression::Subscript { collection, .. } => {
                Self::deduce_subscript_type(collection)
            }
            Expression::Range { .. } => DataType::List,
            Expression::Path(_) => DataType::Path,
            Expression::Label(_) => DataType::String,
            Expression::ListComprehension { .. } => DataType::List,
            Expression::LabelTagProperty { .. } => DataType::Empty,
            Expression::TagProperty { .. } => DataType::Empty,
            Expression::EdgeProperty { .. } => DataType::Empty,
            Expression::Predicate { .. } => DataType::Bool,
            Expression::Reduce { .. } => DataType::Empty,
            Expression::PathBuild(_) => DataType::Path,
            Expression::Parameter(_) => DataType::Empty,
        }
    }

    /// 推导值类型
    fn deduce_value_type(value: &Value) -> DataType {
        match value {
            Value::Null(_) => DataType::Null,
            Value::Bool(_) => DataType::Bool,
            Value::Int(_) => DataType::Int,
            Value::Float(_) => DataType::Float,
            Value::String(_) => DataType::String,
            Value::List(_) => DataType::List,
            Value::Map(_) => DataType::Map,
            Value::Vertex(_) => DataType::Vertex,
            Value::Edge(_) => DataType::Edge,
            Value::Path(_) => DataType::Path,
            Value::Date(_) => DataType::Date,
            Value::Time(_) => DataType::Time,
            Value::DateTime(_) => DataType::DateTime,
            Value::Duration(_) => DataType::Duration,
            Value::Empty => DataType::Empty,
            _ => DataType::Empty,
        }
    }

    /// 推导二元运算类型
    fn deduce_binary_type(op: &BinaryOperator, left: &Expression, right: &Expression) -> DataType {
        match op {
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo
            | BinaryOperator::Exponent => {
                let left_type = left.deduce_type();
                let right_type = right.deduce_type();
                Self::deduce_arithmetic_type(&left_type, &right_type)
            }
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::And
            | BinaryOperator::Or
            | BinaryOperator::Xor
            | BinaryOperator::Like
            | BinaryOperator::In
            | BinaryOperator::NotIn
            | BinaryOperator::Contains
            | BinaryOperator::StartsWith
            | BinaryOperator::EndsWith => DataType::Bool,
            BinaryOperator::StringConcat => DataType::String,
            _ => DataType::Empty,
        }
    }

    /// 推导算术运算结果类型
    fn deduce_arithmetic_type(left: &DataType, right: &DataType) -> DataType {
        match (left, right) {
            (DataType::Float, _) | (_, DataType::Float) => DataType::Float,
            (DataType::Int, DataType::Int) => DataType::Int,
            _ => DataType::Empty,
        }
    }

    /// 推导一元运算类型
    fn deduce_unary_type(op: &UnaryOperator, operand: &Expression) -> DataType {
        match op {
            UnaryOperator::Not => DataType::Bool,
            UnaryOperator::IsNull | UnaryOperator::IsNotNull => DataType::Bool,
            UnaryOperator::IsEmpty | UnaryOperator::IsNotEmpty => DataType::Bool,
            UnaryOperator::Plus | UnaryOperator::Minus => operand.deduce_type(),
        }
    }

    /// 推导函数返回类型
    fn deduce_function_type(name: &str, args: &[Expression]) -> DataType {
        let name_upper = name.to_uppercase();
        match name_upper.as_str() {
            // 数学函数
            "ABS" | "CEIL" | "FLOOR" | "ROUND" | "SIGN" | "SQRT" | "POW" | "EXP" | "LOG" | "LOG10" | "LOG2" => {
                if let Some(first_arg) = args.first() {
                    first_arg.deduce_type()
                } else {
                    DataType::Empty
                }
            }
            // 字符串函数
            "LENGTH" | "SIZE" => DataType::Int,
            "SUBSTRING" | "REPLACE" | "TRIM" | "LTRIM" | "RTRIM" | "UPPER" | "LOWER" | "CONCAT" => DataType::String,
            // 类型转换函数
            "TOSTRING" => DataType::String,
            "TOINT" => DataType::Int,
            "TOFLOAT" => DataType::Float,
            "TOBOOLEAN" => DataType::Bool,
            // 集合函数
            "HEAD" | "LAST" => {
                if let Some(first_arg) = args.first() {
                    first_arg.deduce_type()
                } else {
                    DataType::Empty
                }
            }
            "TAIL" | "NODES" | "RELATIONSHIPS" | "KEYS" | "LABELS" | "RANGE" => DataType::List,
            // 聚合相关函数
            "COUNT" => DataType::Int,
            "COLLECT" => DataType::List,
            // 图相关函数
            "ID" | "SRC" | "DST" | "TYPE" => DataType::String,
            "STARTNODE" | "ENDNODE" => DataType::Vertex,
            // 时间函数
            "NOW" | "TIMESTAMP" => DataType::DateTime,
            "DATE" => DataType::Date,
            "TIME" => DataType::Time,
            // 条件函数
            "COALESCE" => {
                // 返回第一个非空参数的类型
                for arg in args {
                    let arg_type = arg.deduce_type();
                    if arg_type != DataType::Null && arg_type != DataType::Empty {
                        return arg_type;
                    }
                }
                DataType::Empty
            }
            _ => DataType::Empty,
        }
    }

    /// 推导聚合函数返回类型
    fn deduce_aggregate_type(func: &AggregateFunction) -> DataType {
        match func {
            AggregateFunction::Count(_) => DataType::Int,
            AggregateFunction::Sum(_) => DataType::Float,
            AggregateFunction::Avg(_) => DataType::Float,
            AggregateFunction::Min(_) => DataType::Empty,
            AggregateFunction::Max(_) => DataType::Empty,
            AggregateFunction::Collect(_) => DataType::List,
            AggregateFunction::CollectSet(_) => DataType::List,
            AggregateFunction::Distinct(_) => DataType::List,
            AggregateFunction::Percentile(_, _) => DataType::Float,
            AggregateFunction::Std(_) => DataType::Float,
            AggregateFunction::BitAnd(_) => DataType::Int,
            AggregateFunction::BitOr(_) => DataType::Int,
            AggregateFunction::GroupConcat(_, _) => DataType::String,
        }
    }

    /// 推导条件表达式类型
    fn deduce_case_type(
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) -> DataType {
        // 尝试从条件分支推导类型
        for (_, value) in conditions {
            let value_type = value.deduce_type();
            if value_type != DataType::Empty {
                return value_type;
            }
        }
        // 尝试从默认分支推导类型
        if let Some(def) = default {
            def.deduce_type()
        } else {
            DataType::Empty
        }
    }

    /// 推导下标访问类型
    fn deduce_subscript_type(collection: &Expression) -> DataType {
        let collection_type = collection.deduce_type();
        match collection_type {
            DataType::List => DataType::Empty,
            DataType::Map => DataType::Empty,
            DataType::String => DataType::String,
            DataType::Path => DataType::Vertex,
            _ => DataType::Empty,
        }
    }
}
