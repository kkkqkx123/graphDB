//! 表达式系统适配器
//!
//! 提供新旧表达式系统之间的兼容性桥接

use crate::core::error::{DBError, DBResult};
use crate::core::Value;
use crate::graph::expression::evaluator_v2::ExpressionContext;
use crate::graph::expression::ExpressionV1;
use crate::graph::expression::ExpressionV2;

/// 旧表达式到新表达式的转换器
pub struct ExpressionConverter;

impl ExpressionConverter {
    /// 将旧表达式转换为新表达式
    pub fn convert_old_to_new(old_expr: &ExpressionV1) -> ExpressionV2 {
        match old_expr {
            ExpressionV1::Constant(value) => {
                ExpressionV2::literal(match value {
                    Value::Bool(b) => {
                        crate::graph::expression::expression_v2::LiteralValue::Bool(*b)
                    }
                    Value::Int(i) => crate::graph::expression::expression_v2::LiteralValue::Int(*i),
                    Value::Float(f) => {
                        crate::graph::expression::expression_v2::LiteralValue::Float(*f)
                    }
                    Value::String(s) => {
                        crate::graph::expression::expression_v2::LiteralValue::String(s.clone())
                    }
                    Value::Null(_) => crate::graph::expression::expression_v2::LiteralValue::Null,
                    _ => crate::graph::expression::expression_v2::LiteralValue::Null, // 其他复杂类型暂时转为Null
                })
            }
            ExpressionV1::Variable(name) => ExpressionV2::variable(name.clone()),
            ExpressionV1::Property(name) => {
                ExpressionV2::property(ExpressionV2::variable("vertex"), name.clone())
            }
            ExpressionV1::Function(name, args) => {
                let new_args: Vec<ExpressionV2> = args
                    .iter()
                    .map(|arg| Self::convert_old_to_new(arg))
                    .collect();
                ExpressionV2::function(name.clone(), new_args)
            }
            ExpressionV1::BinaryOp(left, op, right) => {
                let new_left = Self::convert_old_to_new(left);
                let new_right = Self::convert_old_to_new(right);
                let new_op = Self::convert_binary_operator(op);
                ExpressionV2::binary(new_left, new_op, new_right)
            }
            ExpressionV1::UnaryOp(op, operand) => {
                let new_operand = Self::convert_old_to_new(operand);
                let new_op = Self::convert_unary_operator(op);
                ExpressionV2::unary(new_op, new_operand)
            }
            ExpressionV1::List(items) => {
                let new_items: Vec<ExpressionV2> = items
                    .iter()
                    .map(|item| Self::convert_old_to_new(item))
                    .collect();
                ExpressionV2::list(new_items)
            }
            ExpressionV1::Map(pairs) => {
                let new_pairs: Vec<(String, ExpressionV2)> = pairs
                    .iter()
                    .map(|(key, value)| (key.clone(), Self::convert_old_to_new(value)))
                    .collect();
                ExpressionV2::map(new_pairs)
            }
            ExpressionV1::Aggregate {
                func,
                arg,
                distinct,
            } => {
                let new_arg = Self::convert_old_to_new(arg);
                let new_func = Self::convert_aggregate_function(func);
                ExpressionV2::aggregate(new_func, new_arg, *distinct)
            }
            ExpressionV1::Case {
                conditions,
                default,
            } => {
                let new_conditions: Vec<(ExpressionV2, ExpressionV2)> = conditions
                    .iter()
                    .map(|(cond, value)| {
                        let new_cond = Self::convert_old_to_new(cond);
                        let new_value = Self::convert_old_to_new(value);
                        (new_cond, new_value)
                    })
                    .collect();
                let new_default = default.as_ref().map(|def| Self::convert_old_to_new(def));
                ExpressionV2::case(new_conditions, new_default)
            }
            ExpressionV1::TypeCasting { expr, target_type } => {
                let new_expr = Self::convert_old_to_new(expr);
                let new_type = Self::convert_data_type(target_type);
                ExpressionV2::cast(new_expr, new_type)
            }
            ExpressionV1::Subscript { collection, index } => {
                let new_collection = Self::convert_old_to_new(collection);
                let new_index = Self::convert_old_to_new(index);
                ExpressionV2::subscript(new_collection, new_index)
            }
            ExpressionV1::SubscriptRange {
                collection,
                start,
                end,
            } => {
                let new_collection = Self::convert_old_to_new(collection);
                let new_start = start.as_ref().map(|s| Self::convert_old_to_new(s));
                let new_end = end.as_ref().map(|e| Self::convert_old_to_new(e));
                ExpressionV2::range(new_collection, new_start, new_end)
            }
            ExpressionV1::Label(name) => ExpressionV2::label(name.clone()),
            ExpressionV1::PathBuild(items) => {
                let new_items: Vec<ExpressionV2> = items
                    .iter()
                    .map(|item| Self::convert_old_to_new(item))
                    .collect();
                ExpressionV2::path(new_items)
            }
            // 其他表达式类型的转换...
            _ => {
                // 对于暂时不支持的表达式类型，返回一个空值表达式
                ExpressionV2::null()
            }
        }
    }

    /// 将新表达式转换为旧表达式
    pub fn convert_new_to_old(new_expr: &ExpressionV2) -> ExpressionV1 {
        match new_expr {
            ExpressionV2::Literal(lit) => {
                let value = match lit {
                    crate::graph::expression::expression_v2::LiteralValue::Bool(b) => {
                        Value::Bool(*b)
                    }
                    crate::graph::expression::expression_v2::LiteralValue::Int(i) => Value::Int(*i),
                    crate::graph::expression::expression_v2::LiteralValue::Float(f) => {
                        Value::Float(*f)
                    }
                    crate::graph::expression::expression_v2::LiteralValue::String(s) => {
                        Value::String(s.clone())
                    }
                    crate::graph::expression::expression_v2::LiteralValue::Null => {
                        Value::Null(crate::core::NullType::Null)
                    }
                };
                ExpressionV1::Constant(value)
            }
            ExpressionV2::Variable(name) => ExpressionV1::Variable(name.clone()),
            ExpressionV2::Property {
                object: _,
                property,
            } => {
                // 简化处理：将属性访问转换为变量访问
                ExpressionV1::Property(property.clone())
            }
            ExpressionV2::Binary { left, op, right } => {
                let old_left = Self::convert_new_to_old(left);
                let old_right = Self::convert_new_to_old(right);
                let old_op = Self::convert_binary_operator_back(op);
                ExpressionV1::BinaryOp(Box::new(old_left), old_op, Box::new(old_right))
            }
            ExpressionV2::Unary { op, operand } => {
                let old_operand = Self::convert_new_to_old(operand);
                let old_op = Self::convert_unary_operator_back(op);
                ExpressionV1::UnaryOp(old_op, Box::new(old_operand))
            }
            ExpressionV2::Function { name, args } => {
                let old_args: Vec<ExpressionV1> = args
                    .iter()
                    .map(|arg| Self::convert_new_to_old(arg))
                    .collect();
                ExpressionV1::Function(name.clone(), old_args)
            }
            ExpressionV2::Aggregate {
                func,
                arg,
                distinct,
            } => {
                let old_arg = Self::convert_new_to_old(arg);
                let old_func = Self::convert_aggregate_function_back(func);
                ExpressionV1::Aggregate {
                    func: old_func,
                    arg: Box::new(old_arg),
                    distinct: *distinct,
                }
            }
            ExpressionV2::List(items) => {
                let old_items: Vec<ExpressionV1> = items
                    .iter()
                    .map(|item| Self::convert_new_to_old(item))
                    .collect();
                ExpressionV1::List(old_items)
            }
            ExpressionV2::Map(pairs) => {
                let old_pairs: Vec<(String, ExpressionV1)> = pairs
                    .iter()
                    .map(|(key, value)| (key.clone(), Self::convert_new_to_old(value)))
                    .collect();
                ExpressionV1::Map(old_pairs)
            }
            ExpressionV2::Case {
                conditions,
                default,
            } => {
                let old_conditions: Vec<(ExpressionV1, ExpressionV1)> = conditions
                    .iter()
                    .map(|(cond, value)| {
                        let old_cond = Self::convert_new_to_old(cond);
                        let old_value = Self::convert_new_to_old(value);
                        (old_cond, old_value)
                    })
                    .collect();
                let old_default = default
                    .as_ref()
                    .map(|def| Box::new(Self::convert_new_to_old(def)));
                ExpressionV1::Case {
                    conditions: old_conditions,
                    default: old_default,
                }
            }
            ExpressionV2::TypeCast { expr, target_type } => {
                let old_expr = Self::convert_new_to_old(expr);
                let old_type = Self::convert_data_type_back(target_type);
                ExpressionV1::TypeCasting {
                    expr: Box::new(old_expr),
                    target_type: old_type,
                }
            }
            ExpressionV2::Subscript { collection, index } => {
                let old_collection = Self::convert_new_to_old(collection);
                let old_index = Self::convert_new_to_old(index);
                ExpressionV1::Subscript {
                    collection: Box::new(old_collection),
                    index: Box::new(old_index),
                }
            }
            ExpressionV2::Range {
                collection,
                start,
                end,
            } => {
                let old_collection = Self::convert_new_to_old(collection);
                let old_start = start
                    .as_ref()
                    .map(|s| Box::new(Self::convert_new_to_old(s)));
                let old_end = end.as_ref().map(|e| Box::new(Self::convert_new_to_old(e)));
                ExpressionV1::SubscriptRange {
                    collection: Box::new(old_collection),
                    start: old_start,
                    end: old_end,
                }
            }
            ExpressionV2::Label(name) => ExpressionV1::Label(name.clone()),
            ExpressionV2::Path(items) => {
                let old_items: Vec<ExpressionV1> = items
                    .iter()
                    .map(|item| Self::convert_new_to_old(item))
                    .collect();
                ExpressionV1::PathBuild(old_items)
            }
        }
    }

    fn convert_binary_operator(
        old_op: &crate::graph::expression::binary::BinaryOperator,
    ) -> crate::graph::expression::expression_v2::BinaryOperator {
        use crate::graph::expression::binary::BinaryOperator as Old;
        use crate::graph::expression::expression_v2::BinaryOperator as New;

        match old_op {
            Old::Add => New::Add,
            Old::Sub => New::Subtract,
            Old::Mul => New::Multiply,
            Old::Div => New::Divide,
            Old::Mod => New::Modulo,
            Old::Eq => New::Equal,
            Old::Ne => New::NotEqual,
            Old::Lt => New::LessThan,
            Old::Le => New::LessThanOrEqual,
            Old::Gt => New::GreaterThan,
            Old::Ge => New::GreaterThanOrEqual,
            Old::And => New::And,
            Old::Or => New::Or,
            // Map other operators that don't have direct equivalents
            Old::Xor => New::Or,           // Approximate mapping
            Old::In => New::Equal,         // Approximate mapping
            Old::NotIn => New::NotEqual,   // Approximate mapping
            Old::Subscript => New::Equal,  // Approximate mapping
            Old::Attribute => New::Equal,  // Approximate mapping
            Old::Contains => New::Equal,   // Approximate mapping
            Old::StartsWith => New::Equal, // Approximate mapping
            Old::EndsWith => New::Equal,   // Approximate mapping
        }
    }

    fn convert_binary_operator_back(
        new_op: &crate::graph::expression::expression_v2::BinaryOperator,
    ) -> crate::graph::expression::binary::BinaryOperator {
        use crate::graph::expression::binary::BinaryOperator as Old;
        use crate::graph::expression::expression_v2::BinaryOperator as New;

        match new_op {
            New::Add => Old::Add,
            New::Subtract => Old::Sub,
            New::Multiply => Old::Mul,
            New::Divide => Old::Div,
            New::Modulo => Old::Mod,
            New::Equal => Old::Eq,
            New::NotEqual => Old::Ne,
            New::LessThan => Old::Lt,
            New::LessThanOrEqual => Old::Le,
            New::GreaterThan => Old::Gt,
            New::GreaterThanOrEqual => Old::Ge,
            New::And => Old::And,
            New::Or => Old::Or,
            // Handle other new operators with approximate mappings
            New::StringConcat => Old::Add, // Approximate mapping
            New::Like => Old::Eq,          // Approximate mapping
            New::In => Old::In,            // Direct mapping
            New::Union => Old::Add,        // Approximate mapping
            New::Intersect => Old::And,    // Approximate mapping
            New::Except => Old::Sub,       // Approximate mapping
        }
    }

    fn convert_unary_operator(
        old_op: &crate::graph::expression::unary::UnaryOperator,
    ) -> crate::graph::expression::expression_v2::UnaryOperator {
        use crate::graph::expression::expression_v2::UnaryOperator as New;
        use crate::graph::expression::unary::UnaryOperator as Old;

        match old_op {
            Old::Plus => New::Plus,
            Old::Minus => New::Minus,
            Old::Negate => New::Minus, // Negate 和 Minus 在新系统中合并
            Old::Not => New::Not,
            Old::Increment => New::Increment,
            Old::Decrement => New::Decrement,
        }
    }

    fn convert_unary_operator_back(
        new_op: &crate::graph::expression::expression_v2::UnaryOperator,
    ) -> crate::graph::expression::unary::UnaryOperator {
        use crate::graph::expression::expression_v2::UnaryOperator as New;
        use crate::graph::expression::unary::UnaryOperator as Old;

        match new_op {
            New::Plus => Old::Plus,
            New::Minus => Old::Minus,
            New::Not => Old::Not,
            New::IsNull => Old::Not, // Approximate mapping - not directly supported in old system
            New::IsNotNull => Old::Not, // Approximate mapping - not directly supported in old system
            New::IsEmpty => Old::Not, // Approximate mapping - not directly supported in old system
            New::IsNotEmpty => Old::Not, // Approximate mapping - not directly supported in old system
            New::Increment => Old::Increment,
            New::Decrement => Old::Decrement,
        }
    }

    fn convert_aggregate_function(
        old_func: &str,
    ) -> crate::graph::expression::expression_v2::AggregateFunction {
        match old_func {
            "count" => crate::graph::expression::expression_v2::AggregateFunction::Count,
            "sum" => crate::graph::expression::expression_v2::AggregateFunction::Sum,
            "avg" => crate::graph::expression::expression_v2::AggregateFunction::Avg,
            "min" => crate::graph::expression::expression_v2::AggregateFunction::Min,
            "max" => crate::graph::expression::expression_v2::AggregateFunction::Max,
            _ => crate::graph::expression::expression_v2::AggregateFunction::Count, // 默认
        }
    }

    fn convert_aggregate_function_back(
        new_func: &crate::graph::expression::expression_v2::AggregateFunction,
    ) -> String {
        match new_func {
            crate::graph::expression::expression_v2::AggregateFunction::Count => {
                "count".to_string()
            }
            crate::graph::expression::expression_v2::AggregateFunction::Sum => "sum".to_string(),
            crate::graph::expression::expression_v2::AggregateFunction::Avg => "avg".to_string(),
            crate::graph::expression::expression_v2::AggregateFunction::Min => "min".to_string(),
            crate::graph::expression::expression_v2::AggregateFunction::Max => "max".to_string(),
            crate::graph::expression::expression_v2::AggregateFunction::Collect => {
                "collect".to_string()
            }
            crate::graph::expression::expression_v2::AggregateFunction::Distinct => {
                "distinct".to_string()
            }
        }
    }

    fn convert_data_type(old_type: &str) -> crate::graph::expression::expression_v2::DataType {
        match old_type {
            "bool" => crate::graph::expression::expression_v2::DataType::Bool,
            "int" => crate::graph::expression::expression_v2::DataType::Int,
            "float" => crate::graph::expression::expression_v2::DataType::Float,
            "string" => crate::graph::expression::expression_v2::DataType::String,
            "list" => crate::graph::expression::expression_v2::DataType::List,
            "map" => crate::graph::expression::expression_v2::DataType::Map,
            _ => crate::graph::expression::expression_v2::DataType::String, // 默认
        }
    }

    fn convert_data_type_back(
        new_type: &crate::graph::expression::expression_v2::DataType,
    ) -> String {
        match new_type {
            crate::graph::expression::expression_v2::DataType::Bool => "bool".to_string(),
            crate::graph::expression::expression_v2::DataType::Int => "int".to_string(),
            crate::graph::expression::expression_v2::DataType::Float => "float".to_string(),
            crate::graph::expression::expression_v2::DataType::String => "string".to_string(),
            crate::graph::expression::expression_v2::DataType::List => "list".to_string(),
            crate::graph::expression::expression_v2::DataType::Map => "map".to_string(),
            crate::graph::expression::expression_v2::DataType::Vertex => "vertex".to_string(),
            crate::graph::expression::expression_v2::DataType::Edge => "edge".to_string(),
            crate::graph::expression::expression_v2::DataType::Path => "path".to_string(),
            crate::graph::expression::expression_v2::DataType::DateTime => "datetime".to_string(),
        }
    }
}

/// 旧上下文到新上下文的适配器
pub struct ContextAdapter<'a> {
    old_context: &'a crate::graph::expression::context::EvalContext<'a>,
}

impl<'a> ContextAdapter<'a> {
    pub fn new(old_context: &'a crate::graph::expression::context::EvalContext<'a>) -> Self {
        Self { old_context }
    }
}

impl<'a> ExpressionContext for ContextAdapter<'a> {
    fn get_variable(&self, name: &str) -> Option<&Value> {
        self.old_context.vars.get(name)
    }

    fn get_property(&self, _object: &Value, property: &str) -> DBResult<&Value> {
        // 由于生命周期限制，暂时不支持属性访问
        Err(DBError::Expression(
            crate::core::error::ExpressionError::PropertyNotFound(format!(
                "Property access not supported in compatibility adapter: {}",
                property
            )),
        ))
    }

    fn get_function(
        &self,
        _name: &str,
    ) -> Option<&dyn crate::graph::expression::evaluator_v2::Function> {
        // 暂时不支持函数，返回 None
        None
    }
}

/// 兼容性求值器
///
/// 使用新的求值器来处理旧的表达式
pub struct CompatibilityEvaluator;

impl CompatibilityEvaluator {
    pub fn new() -> Self {
        Self
    }

    /// 求值旧表达式
    pub fn evaluate_old(
        &self,
        old_expr: &OldExpression,
        old_context: &crate::graph::expression::context::EvalContext,
    ) -> DBResult<Value> {
        // 暂时直接使用旧表达式求值逻辑
        // TODO: 实现完整的表达式转换和求值
        old_expr.evaluate(old_context).map_err(|e| {
            DBError::Expression(crate::core::error::ExpressionError::InvalidOperation(
                e.to_string(),
            ))
        })
    }
}

impl Default for CompatibilityEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::graph::expression::*;
    use crate::core::Value;
    use std::collections::HashMap;

    #[test]
    fn test_expression_conversion() {
        // 测试简单表达式的转换
        let old_expr = ExpressionV1::Constant(Value::Int(42));
        let new_expr = ExpressionConverter::convert_old_to_new(&old_expr);

        assert_eq!(new_expr, ExpressionV2::int(42));

        // 测试转换回去
        let converted_back = ExpressionConverter::convert_new_to_old(&new_expr);
        assert_eq!(old_expr, converted_back);
    }

    #[test]
    fn test_binary_expression_conversion() {
        let old_expr = ExpressionV1::BinaryOp(
            Box::new(ExpressionV1::Constant(Value::Int(10))),
            crate::graph::expression::binary::BinaryOperator::Add,
            Box::new(ExpressionV1::Constant(Value::Int(20))),
        );

        let new_expr = ExpressionConverter::convert_old_to_new(&old_expr);

        if let ExpressionV2::Binary { left, op, right } = new_expr {
            assert_eq!(*left, ExpressionV2::int(10));
            assert_eq!(
                op,
                crate::graph::expression::expression_v2::BinaryOperator::Add
            );
            assert_eq!(*right, ExpressionV2::int(20));
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_compatibility_evaluator() {
        let evaluator = CompatibilityEvaluator::new();

        // 创建测试上下文
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), Value::Int(100));

        let context = crate::graph::expression::context::EvalContext {
            vertex: None,
            edge: None,
            vars,
        };

        // 测试常量求值
        let expr = ExpressionV1::Constant(Value::Int(42));
        let result = evaluator.evaluate_old(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(42));

        // 测试变量求值
        let expr = ExpressionV1::Variable("x".to_string());
        let result = evaluator.evaluate_old(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(100));
    }
}
