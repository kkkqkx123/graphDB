//! 表达式系统适配器
//!
//! 提供新旧表达式系统之间的兼容性桥接

use crate::core::Value;
use crate::core::error::{DBError, DBResult};
use super::expr_type::Expression as OldExpression;
use super::evaluator_v2::{ExpressionContext, ExpressionEvaluator, DefaultExpressionEvaluator};
use super::expression_v2::Expression as NewExpression;

/// 旧表达式到新表达式的转换器
pub struct ExpressionConverter;

impl ExpressionConverter {
    /// 将旧表达式转换为新表达式
    pub fn convert_old_to_new(old_expr: &OldExpression) -> NewExpression {
        match old_expr {
            OldExpression::Constant(value) => {
                NewExpression::literal(match value {
                    Value::Bool(b) => super::expression_v2::LiteralValue::Bool(*b),
                    Value::Int(i) => super::expression_v2::LiteralValue::Int(*i),
                    Value::Float(f) => super::expression_v2::LiteralValue::Float(*f),
                    Value::String(s) => super::expression_v2::LiteralValue::String(s.clone()),
                    Value::Null(_) => super::expression_v2::LiteralValue::Null,
                    _ => super::expression_v2::LiteralValue::Null, // 其他复杂类型暂时转为Null
                })
            }
            OldExpression::Variable(name) => {
                NewExpression::variable(name.clone())
            }
            OldExpression::Property(name) => {
                NewExpression::property(NewExpression::variable("vertex"), name.clone())
            }
            OldExpression::Function(name, args) => {
                let new_args: Vec<NewExpression> = args.iter()
                    .map(|arg| Self::convert_old_to_new(arg))
                    .collect();
                NewExpression::function(name.clone(), new_args)
            }
            OldExpression::BinaryOp(left, op, right) => {
                let new_left = Self::convert_old_to_new(left);
                let new_right = Self::convert_old_to_new(right);
                let new_op = Self::convert_binary_operator(op);
                NewExpression::binary(new_left, new_op, new_right)
            }
            OldExpression::UnaryOp(op, operand) => {
                let new_operand = Self::convert_old_to_new(operand);
                let new_op = Self::convert_unary_operator(op);
                NewExpression::unary(new_op, new_operand)
            }
            OldExpression::List(items) => {
                let new_items: Vec<NewExpression> = items.iter()
                    .map(|item| Self::convert_old_to_new(item))
                    .collect();
                NewExpression::list(new_items)
            }
            OldExpression::Map(pairs) => {
                let new_pairs: Vec<(String, NewExpression)> = pairs.iter()
                    .map(|(key, value)| (key.clone(), Self::convert_old_to_new(value)))
                    .collect();
                NewExpression::map(new_pairs)
            }
            OldExpression::Aggregate { func, arg, distinct } => {
                let new_arg = Self::convert_old_to_new(arg);
                let new_func = Self::convert_aggregate_function(func);
                NewExpression::aggregate(new_func, new_arg, *distinct)
            }
            OldExpression::Case { conditions, default } => {
                let new_conditions: Vec<(NewExpression, NewExpression)> = conditions.iter()
                    .map(|(cond, value)| {
                        let new_cond = Self::convert_old_to_new(cond);
                        let new_value = Self::convert_old_to_new(value);
                        (new_cond, new_value)
                    })
                    .collect();
                let new_default = default.as_ref()
                    .map(|def| Self::convert_old_to_new(def));
                NewExpression::case(new_conditions, new_default)
            }
            OldExpression::TypeCasting { expr, target_type } => {
                let new_expr = Self::convert_old_to_new(expr);
                let new_type = Self::convert_data_type(target_type);
                NewExpression::cast(new_expr, new_type)
            }
            OldExpression::Subscript { collection, index } => {
                let new_collection = Self::convert_old_to_new(collection);
                let new_index = Self::convert_old_to_new(index);
                NewExpression::subscript(new_collection, new_index)
            }
            OldExpression::SubscriptRange { collection, start, end } => {
                let new_collection = Self::convert_old_to_new(collection);
                let new_start = start.as_ref().map(|s| Self::convert_old_to_new(s));
                let new_end = end.as_ref().map(|e| Self::convert_old_to_new(e));
                NewExpression::range(new_collection, new_start, new_end)
            }
            OldExpression::Label(name) => {
                NewExpression::label(name.clone())
            }
            OldExpression::PathBuild(items) => {
                let new_items: Vec<NewExpression> = items.iter()
                    .map(|item| Self::convert_old_to_new(item))
                    .collect();
                NewExpression::path(new_items)
            }
            // 其他表达式类型的转换...
            _ => {
                // 对于暂时不支持的表达式类型，返回一个空值表达式
                NewExpression::null()
            }
        }
    }
    
    /// 将新表达式转换为旧表达式
    pub fn convert_new_to_old(new_expr: &NewExpression) -> OldExpression {
        match new_expr {
            NewExpression::Literal(lit) => {
                let value = match lit {
                    super::expression_v2::LiteralValue::Bool(b) => Value::Bool(*b),
                    super::expression_v2::LiteralValue::Int(i) => Value::Int(*i),
                    super::expression_v2::LiteralValue::Float(f) => Value::Float(*f),
                    super::expression_v2::LiteralValue::String(s) => Value::String(s.clone()),
                    super::expression_v2::LiteralValue::Null => Value::Null(crate::core::NullType::Null),
                };
                OldExpression::Constant(value)
            }
            NewExpression::Variable(name) => {
                OldExpression::Variable(name.clone())
            }
            NewExpression::Property { object: _, property } => {
                // 简化处理：将属性访问转换为变量访问
                OldExpression::Property(property.clone())
            }
            NewExpression::Binary { left, op, right } => {
                let old_left = Self::convert_new_to_old(left);
                let old_right = Self::convert_new_to_old(right);
                let old_op = Self::convert_binary_operator_back(op);
                OldExpression::BinaryOp(Box::new(old_left), old_op, Box::new(old_right))
            }
            NewExpression::Unary { op, operand } => {
                let old_operand = Self::convert_new_to_old(operand);
                let old_op = Self::convert_unary_operator_back(op);
                OldExpression::UnaryOp(old_op, Box::new(old_operand))
            }
            NewExpression::Function { name, args } => {
                let old_args: Vec<OldExpression> = args.iter()
                    .map(|arg| Self::convert_new_to_old(arg))
                    .collect();
                OldExpression::Function(name.clone(), old_args)
            }
            NewExpression::Aggregate { func, arg, distinct } => {
                let old_arg = Self::convert_new_to_old(arg);
                let old_func = Self::convert_aggregate_function_back(func);
                OldExpression::Aggregate {
                    func: old_func,
                    arg: Box::new(old_arg),
                    distinct: *distinct,
                }
            }
            NewExpression::List(items) => {
                let old_items: Vec<OldExpression> = items.iter()
                    .map(|item| Self::convert_new_to_old(item))
                    .collect();
                OldExpression::List(old_items)
            }
            NewExpression::Map(pairs) => {
                let old_pairs: Vec<(String, OldExpression)> = pairs.iter()
                    .map(|(key, value)| (key.clone(), Self::convert_new_to_old(value)))
                    .collect();
                OldExpression::Map(old_pairs)
            }
            NewExpression::Case { conditions, default } => {
                let old_conditions: Vec<(OldExpression, OldExpression)> = conditions.iter()
                    .map(|(cond, value)| {
                        let old_cond = Self::convert_new_to_old(cond);
                        let old_value = Self::convert_new_to_old(value);
                        (old_cond, old_value)
                    })
                    .collect();
                let old_default = default.as_ref()
                    .map(|def| Box::new(Self::convert_new_to_old(def)));
                OldExpression::Case {
                    conditions: old_conditions,
                    default: old_default,
                }
            }
            NewExpression::TypeCast { expr, target_type } => {
                let old_expr = Self::convert_new_to_old(expr);
                let old_type = Self::convert_data_type_back(target_type);
                OldExpression::TypeCasting {
                    expr: Box::new(old_expr),
                    target_type: old_type,
                }
            }
            NewExpression::Subscript { collection, index } => {
                let old_collection = Self::convert_new_to_old(collection);
                let old_index = Self::convert_new_to_old(index);
                OldExpression::Subscript {
                    collection: Box::new(old_collection),
                    index: Box::new(old_index),
                }
            }
            NewExpression::Range { collection, start, end } => {
                let old_collection = Self::convert_new_to_old(collection);
                let old_start = start.as_ref().map(|s| Box::new(Self::convert_new_to_old(s)));
                let old_end = end.as_ref().map(|e| Box::new(Self::convert_new_to_old(e)));
                OldExpression::SubscriptRange {
                    collection: Box::new(old_collection),
                    start: old_start,
                    end: old_end,
                }
            }
            NewExpression::Label(name) => {
                OldExpression::Label(name.clone())
            }
            NewExpression::Path(items) => {
                let old_items: Vec<OldExpression> = items.iter()
                    .map(|item| Self::convert_new_to_old(item))
                    .collect();
                OldExpression::PathBuild(old_items)
            }
        }
    }
    
    fn convert_binary_operator(old_op: &super::binary::BinaryOperator) -> super::expression_v2::BinaryOperator {
        use super::binary::BinaryOperator as Old;
        use super::expression_v2::BinaryOperator as New;

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
            Old::Xor => New::Or, // Approximate mapping
            Old::In => New::Equal, // Approximate mapping
            Old::NotIn => New::NotEqual, // Approximate mapping
            Old::Subscript => New::Equal, // Approximate mapping
            Old::Attribute => New::Equal, // Approximate mapping
            Old::Contains => New::Equal, // Approximate mapping
            Old::StartsWith => New::Equal, // Approximate mapping
            Old::EndsWith => New::Equal, // Approximate mapping
        }
    }
    
    fn convert_binary_operator_back(new_op: &super::expression_v2::BinaryOperator) -> super::binary::BinaryOperator {
        use super::binary::BinaryOperator as Old;
        use super::expression_v2::BinaryOperator as New;

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
            New::Like => Old::Eq, // Approximate mapping
            New::In => Old::In, // Direct mapping
            New::Union => Old::Add, // Approximate mapping
            New::Intersect => Old::And, // Approximate mapping
            New::Except => Old::Sub, // Approximate mapping
        }
    }
    
    fn convert_unary_operator(old_op: &super::unary::UnaryOperator) -> super::expression_v2::UnaryOperator {
        use super::unary::UnaryOperator as Old;
        use super::expression_v2::UnaryOperator as New;

        match old_op {
            Old::Plus => New::Plus,
            Old::Minus => New::Minus,
            Old::Negate => New::Minus, // Negate 和 Minus 在新系统中合并
            Old::Not => New::Not,
            Old::Increment => New::Increment,
            Old::Decrement => New::Decrement,
        }
    }
    
    fn convert_unary_operator_back(new_op: &super::expression_v2::UnaryOperator) -> super::unary::UnaryOperator {
        use super::unary::UnaryOperator as Old;
        use super::expression_v2::UnaryOperator as New;

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
    
    fn convert_aggregate_function(old_func: &str) -> super::expression_v2::AggregateFunction {
        match old_func {
            "count" => super::expression_v2::AggregateFunction::Count,
            "sum" => super::expression_v2::AggregateFunction::Sum,
            "avg" => super::expression_v2::AggregateFunction::Avg,
            "min" => super::expression_v2::AggregateFunction::Min,
            "max" => super::expression_v2::AggregateFunction::Max,
            _ => super::expression_v2::AggregateFunction::Count, // 默认
        }
    }
    
    fn convert_aggregate_function_back(new_func: &super::expression_v2::AggregateFunction) -> String {
        match new_func {
            super::expression_v2::AggregateFunction::Count => "count".to_string(),
            super::expression_v2::AggregateFunction::Sum => "sum".to_string(),
            super::expression_v2::AggregateFunction::Avg => "avg".to_string(),
            super::expression_v2::AggregateFunction::Min => "min".to_string(),
            super::expression_v2::AggregateFunction::Max => "max".to_string(),
            super::expression_v2::AggregateFunction::Collect => "collect".to_string(),
            super::expression_v2::AggregateFunction::Distinct => "distinct".to_string(),
        }
    }
    
    fn convert_data_type(old_type: &str) -> super::expression_v2::DataType {
        match old_type {
            "bool" => super::expression_v2::DataType::Bool,
            "int" => super::expression_v2::DataType::Int,
            "float" => super::expression_v2::DataType::Float,
            "string" => super::expression_v2::DataType::String,
            "list" => super::expression_v2::DataType::List,
            "map" => super::expression_v2::DataType::Map,
            _ => super::expression_v2::DataType::String, // 默认
        }
    }
    
    fn convert_data_type_back(new_type: &super::expression_v2::DataType) -> String {
        match new_type {
            super::expression_v2::DataType::Bool => "bool".to_string(),
            super::expression_v2::DataType::Int => "int".to_string(),
            super::expression_v2::DataType::Float => "float".to_string(),
            super::expression_v2::DataType::String => "string".to_string(),
            super::expression_v2::DataType::List => "list".to_string(),
            super::expression_v2::DataType::Map => "map".to_string(),
            super::expression_v2::DataType::Vertex => "vertex".to_string(),
            super::expression_v2::DataType::Edge => "edge".to_string(),
            super::expression_v2::DataType::Path => "path".to_string(),
            super::expression_v2::DataType::DateTime => "datetime".to_string(),
        }
    }
}

/// 旧上下文到新上下文的适配器
pub struct ContextAdapter<'a> {
    old_context: &'a super::context::EvalContext<'a>,
}

impl<'a> ContextAdapter<'a> {
    pub fn new(old_context: &'a super::context::EvalContext<'a>) -> Self {
        Self { old_context }
    }
}

impl<'a> ExpressionContext for ContextAdapter<'a> {
    fn get_variable(&self, name: &str) -> Option<&Value> {
        self.old_context.vars.get(name)
    }

    fn get_property(&self, object: &Value, property: &str) -> DBResult<&Value> {
        match object {
            Value::Vertex(vertex) => {
                for tag in &vertex.tags {
                    if let Some(value) = tag.properties.get(property) {
                        return Ok(value);
                    }
                }
                Err(DBError::Expression(crate::core::error::ExpressionError::PropertyNotFound(format!("Property '{}' not found in vertex", property))))
            }
            Value::Edge(edge) => {
                edge.props.get(property)
                    .ok_or_else(|| DBError::Expression(crate::core::error::ExpressionError::PropertyNotFound(format!("Property '{}' not found in edge", property))))
            }
            Value::Map(map) => {
                map.get(property)
                    .ok_or_else(|| DBError::Expression(crate::core::error::ExpressionError::PropertyNotFound(format!("Property '{}' not found in map", property))))
            }
            _ => Err(DBError::Expression(crate::core::error::ExpressionError::TypeError(format!("Cannot access property on type {:?}", object)))),
        }
    }

    fn get_function(&self, _name: &str) -> Option<&dyn super::evaluator_v2::Function> {
        // 暂时不支持函数，返回 None
        None
    }
}

/// 兼容性求值器
/// 
/// 使用新的求值器来处理旧的表达式
pub struct CompatibilityEvaluator {
    new_evaluator: DefaultExpressionEvaluator,
}

impl CompatibilityEvaluator {
    pub fn new() -> Self {
        Self {
            new_evaluator: DefaultExpressionEvaluator::new(),
        }
    }
    
    /// 求值旧表达式
    pub fn evaluate_old(&self, old_expr: &OldExpression, old_context: &super::context::EvalContext) -> DBResult<Value> {
        // 转换为新表达式和新上下文
        let new_expr = ExpressionConverter::convert_old_to_new(old_expr);
        let new_context = ContextAdapter::new(old_context);

        // 使用新求值器求值
        self.new_evaluator.evaluate(&new_expr, &new_context)
    }
}

impl Default for CompatibilityEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_expression_conversion() {
        // 测试简单表达式的转换
        let old_expr = OldExpression::Constant(Value::Int(42));
        let new_expr = ExpressionConverter::convert_old_to_new(&old_expr);
        
        assert_eq!(new_expr, NewExpression::int(42));
        
        // 测试转换回去
        let converted_back = ExpressionConverter::convert_new_to_old(&new_expr);
        assert_eq!(old_expr, converted_back);
    }

    #[test]
    fn test_binary_expression_conversion() {
        let old_expr = OldExpression::BinaryOp(
            Box::new(OldExpression::Constant(Value::Int(10))),
            super::binary::BinaryOperator::Add,
            Box::new(OldExpression::Constant(Value::Int(20))),
        );
        
        let new_expr = ExpressionConverter::convert_old_to_new(&old_expr);
        
        if let NewExpression::Binary { left, op, right } = new_expr {
            assert_eq!(*left, NewExpression::int(10));
            assert_eq!(op, super::expression_v2::BinaryOperator::Add);
            assert_eq!(*right, NewExpression::int(20));
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
        
        let context = super::context::EvalContext {
            vertex: None,
            edge: None,
            vars,
        };
        
        // 测试常量求值
        let expr = OldExpression::Constant(Value::Int(42));
        let result = evaluator.evaluate_old(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(42));
        
        // 测试变量求值
        let expr = OldExpression::Variable("x".to_string());
        let result = evaluator.evaluate_old(&expr, &context).unwrap();
        assert_eq!(result, Value::Int(100));
    }
}