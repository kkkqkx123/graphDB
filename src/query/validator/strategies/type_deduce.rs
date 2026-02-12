//! 类型推导验证器
//!
//! 本模块实现表达式类型推导功能，类似于 nebula-graph 的 DeduceTypeVisitor。
//! 用于在验证阶段推导表达式的返回类型，并检查类型兼容性。

use crate::core::types::expression::Expression;
use crate::core::types::expression::visitor::ExpressionVisitor;
use crate::core::types::operators::{AggregateFunction, BinaryOperator, UnaryOperator};
use crate::core::types::DataType;
use crate::core::Value;
use crate::query::validator::{ValidationError, ValidationErrorType};

/// 类型推导验证器
///
/// 用于推导表达式的返回类型，并检查类型兼容性。
pub struct TypeDeduceValidator {
    current_type: Option<DataType>,
    error: Option<ValidationError>,
}

impl TypeDeduceValidator {
    pub fn new() -> Self {
        Self {
            current_type: None,
            error: None,
        }
    }

    pub fn deduce_type(&mut self, expression: &Expression) -> Result<DataType, ValidationError> {
        self.visit_expression(expression);
        if let Some(error) = self.error.take() {
            Err(error)
        } else {
            Ok(self.current_type.clone().unwrap_or(DataType::Empty))
        }
    }

    fn set_type(&mut self, data_type: DataType) {
        self.current_type = Some(data_type);
    }

    fn set_error(&mut self, error: ValidationError) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    fn is_numeric_type(&self, data_type: &DataType) -> bool {
        matches!(
            data_type,
            DataType::Int | DataType::Float | DataType::Double
        )
    }

    fn is_integer_type(&self, data_type: &DataType) -> bool {
        matches!(data_type, DataType::Int)
    }

    fn is_valid_for_arithmetic(&self, left: &DataType, right: &DataType, op: &BinaryOperator) -> bool {
        match op {
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply => {
                self.is_numeric_type(left) && self.is_numeric_type(right)
            }
            BinaryOperator::Divide | BinaryOperator::Modulo => {
                self.is_numeric_type(left) && self.is_numeric_type(right)
            }
            BinaryOperator::Exponent => {
                self.is_numeric_type(left) && self.is_numeric_type(right)
            }
            _ => true,
        }
    }

    fn deduce_binary_result_type(&self, left: &DataType, right: &DataType, op: &BinaryOperator) -> DataType {
        match op {
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply => {
                if *left == DataType::Double || *right == DataType::Double {
                    DataType::Double
                } else if *left == DataType::Float || *right == DataType::Float {
                    DataType::Float
                } else {
                    DataType::Int
                }
            }
            BinaryOperator::Divide => {
                DataType::Float
            }
            BinaryOperator::Modulo => {
                DataType::Int
            }
            BinaryOperator::Exponent => {
                if *left == DataType::Double || *right == DataType::Double {
                    DataType::Double
                } else if *left == DataType::Float || *right == DataType::Float {
                    DataType::Float
                } else {
                    DataType::Int
                }
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
            BinaryOperator::Subscript | BinaryOperator::Attribute => DataType::Empty,
            BinaryOperator::Union | BinaryOperator::Intersect | BinaryOperator::Except => DataType::List,
        }
    }
}

impl ExpressionVisitor for TypeDeduceValidator {
    type Result = ();

    fn visit_literal(&mut self, value: &Value) {
        let data_type = match value {
            Value::Null(_) => DataType::Null,
            Value::Bool(_) => DataType::Bool,
            Value::Int(_) => DataType::Int,
            Value::Float(_) => DataType::Float,
            Value::String(_) => DataType::String,
            Value::Date(_) => DataType::Date,
            Value::Time(_) => DataType::Time,
            Value::DateTime(_) => DataType::DateTime,
            Value::Vertex(_) => DataType::Vertex,
            Value::Edge(_) => DataType::Edge,
            Value::Path(_) => DataType::Path,
            Value::List(_) => DataType::List,
            Value::Map(_) => DataType::Map,
            Value::Set(_) => DataType::Set,
            Value::Geography(_) => DataType::Geography,
            Value::Duration(_) => DataType::Duration,
            Value::DataSet(_) => DataType::DataSet,
            Value::Empty => DataType::Empty,
        };
        self.set_type(data_type);
    }

    fn visit_variable(&mut self, _name: &str) {
        self.set_type(DataType::Empty);
    }

    fn visit_property(&mut self, object: &Expression, _property: &str) {
        self.visit_expression(object);
    }

    fn visit_binary(&mut self, left: &Expression, op: &BinaryOperator, right: &Expression) {
        self.visit_expression(left);
        let left_type = self.current_type.clone().unwrap_or(DataType::Empty);
        
        self.visit_expression(right);
        let right_type = self.current_type.clone().unwrap_or(DataType::Empty);

        if !self.is_valid_for_arithmetic(&left_type, &right_type, op) {
            self.set_error(ValidationError::new(
                format!(
                    "二元操作符 {} 的操作数类型不兼容: {:?} 和 {:?}",
                    op.name(),
                    left_type,
                    right_type
                ),
                ValidationErrorType::TypeError,
            ));
            return;
        }

        let result_type = self.deduce_binary_result_type(&left_type, &right_type, op);
        self.set_type(result_type);
    }

    fn visit_unary(&mut self, op: &UnaryOperator, operand: &Expression) {
        self.visit_expression(operand);
        let operand_type = self.current_type.clone().unwrap_or(DataType::Empty);

        let result_type = match op {
            UnaryOperator::Plus | UnaryOperator::Minus => {
                if !self.is_numeric_type(&operand_type) && operand_type != DataType::Empty {
                    self.set_error(ValidationError::new(
                        format!(
                            "一元操作符 {} 需要数值类型参数，但得到: {:?}",
                            op.name(),
                            operand_type
                        ),
                        ValidationErrorType::TypeError,
                    ));
                    return;
                }
                operand_type
            }
            UnaryOperator::Not => {
                DataType::Bool
            }
            UnaryOperator::IsNull | UnaryOperator::IsNotNull | UnaryOperator::IsEmpty | UnaryOperator::IsNotEmpty => {
                DataType::Bool
            }
        };
        self.set_type(result_type);
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) {
        let mut arg_types = Vec::new();
        for arg in args {
            self.visit_expression(arg);
            arg_types.push(self.current_type.clone().unwrap_or(DataType::Empty));
        }

        let result_type = match name.to_uppercase().as_str() {
            "ID" | "SRC" | "DST" => DataType::Int,
            "LENGTH" | "SIZE" => DataType::Int,
            "UPPER" | "LOWER" | "TRIM" | "LTRIM" | "RTRIM" => DataType::String,
            "ABS" | "CEIL" | "FLOOR" | "ROUND" | "SQRT" | "EXP" | "LOG" | "LOG10" => {
                if arg_types.is_empty() || !self.is_numeric_type(&arg_types[0]) {
                    self.set_error(ValidationError::new(
                        format!("函数 {} 需要数值类型参数", name),
                        ValidationErrorType::TypeError,
                    ));
                    return;
                }
                DataType::Float
            }
            "SUBSTRING" => DataType::String,
            "CONCAT" => DataType::String,
            "COALESCE" => {
                if arg_types.is_empty() {
                    DataType::Null
                } else {
                    arg_types[0].clone()
                }
            }
            "NOW" => DataType::DateTime,
            _ => DataType::Empty,
        };
        self.set_type(result_type);
    }

    fn visit_aggregate(&mut self, func: &AggregateFunction, arg: &Expression, distinct: bool) {
        self.visit_expression(arg);
        let arg_type = self.current_type.clone().unwrap_or(DataType::Empty);

        if distinct {
            if !matches!(func, AggregateFunction::Count(_) | AggregateFunction::Sum(_) | AggregateFunction::Avg(_)) {
                self.set_error(ValidationError::new(
                    format!("聚合函数 {} 不支持 DISTINCT 关键字", func.name()),
                    ValidationErrorType::SyntaxError,
                ));
                return;
            }
        }

        let result_type = match func {
            AggregateFunction::Count(_) => DataType::Int,
            AggregateFunction::Sum(_) | AggregateFunction::Avg(_) => {
                if !self.is_numeric_type(&arg_type) && arg_type != DataType::Empty {
                    self.set_error(ValidationError::new(
                        format!("聚合函数 {} 需要数值类型参数，但得到: {:?}", func.name(), arg_type),
                        ValidationErrorType::TypeError,
                    ));
                    return;
                }
                DataType::Float
            }
            AggregateFunction::Min(_) | AggregateFunction::Max(_) => {
                arg_type
            }
            AggregateFunction::Collect(_) => DataType::List,
            AggregateFunction::CollectSet(_) => DataType::Set,
            AggregateFunction::Distinct(_) => DataType::Empty,
            AggregateFunction::Percentile(_, _) => {
                if !self.is_numeric_type(&arg_type) && arg_type != DataType::Empty {
                    self.set_error(ValidationError::new(
                        format!("聚合函数 {} 需要数值类型参数，但得到: {:?}", func.name(), arg_type),
                        ValidationErrorType::TypeError,
                    ));
                    return;
                }
                DataType::Float
            }
            AggregateFunction::Std(_) => {
                if !self.is_numeric_type(&arg_type) && arg_type != DataType::Empty {
                    self.set_error(ValidationError::new(
                        format!("聚合函数 {} 需要数值类型参数，但得到: {:?}", func.name(), arg_type),
                        ValidationErrorType::TypeError,
                    ));
                    return;
                }
                DataType::Float
            }
            AggregateFunction::BitAnd(_) | AggregateFunction::BitOr(_) => {
                if !self.is_integer_type(&arg_type) && arg_type != DataType::Empty {
                    self.set_error(ValidationError::new(
                        format!("聚合函数 {} 需要整数类型参数，但得到: {:?}", func.name(), arg_type),
                        ValidationErrorType::TypeError,
                    ));
                    return;
                }
                DataType::Int
            }
            AggregateFunction::GroupConcat(_, _) => DataType::String,
        };
        self.set_type(result_type);
    }

    fn visit_list(&mut self, _items: &[Expression]) {
        self.set_type(DataType::List);
    }

    fn visit_map(&mut self, _pairs: &[(String, Expression)]) {
        self.set_type(DataType::Map);
    }

    fn visit_case(
        &mut self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
    ) {
        if let Some(test) = test_expr {
            self.visit_expression(test);
        }

        let mut result_types = Vec::new();
        for (cond, value) in conditions {
            self.visit_expression(cond);
            self.visit_expression(value);
            if let Some(t) = self.current_type.clone() {
                result_types.push(t);
            }
        }

        if let Some(def) = default {
            self.visit_expression(def);
            if let Some(t) = self.current_type.clone() {
                result_types.push(t);
            }
        }

        if result_types.is_empty() {
            self.set_type(DataType::Null);
        } else {
            self.set_type(result_types[0].clone());
        }
    }

    fn visit_type_cast(&mut self, _expression: &Expression, target_type: &DataType) {
        self.set_type(target_type.clone());
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) {
        self.visit_expression(collection);
        let collection_type = self.current_type.clone().unwrap_or(DataType::Empty);

        self.visit_expression(index);
        let index_type = self.current_type.clone().unwrap_or(DataType::Empty);

        match collection_type {
            DataType::List => {
                if !self.is_integer_type(&index_type) && index_type != DataType::Empty {
                    self.set_error(ValidationError::new(
                        format!("列表下标需要整数类型，但得到: {:?}", index_type),
                        ValidationErrorType::TypeError,
                    ));
                    return;
                }
                self.set_type(DataType::Empty);
            }
            DataType::Map => {
                if index_type != DataType::String && index_type != DataType::Empty {
                    self.set_error(ValidationError::new(
                        format!("映射键需要字符串类型，但得到: {:?}", index_type),
                        ValidationErrorType::TypeError,
                    ));
                    return;
                }
                self.set_type(DataType::Empty);
            }
            DataType::Empty => {
                self.set_type(DataType::Empty);
            }
            _ => {
                self.set_error(ValidationError::new(
                    format!("下标操作不支持类型: {:?}", collection_type),
                    ValidationErrorType::TypeError,
                ));
            }
        }
    }

    fn visit_range(
        &mut self,
        collection: &Expression,
        start: Option<&Expression>,
        end: Option<&Expression>,
    ) {
        self.visit_expression(collection);
        
        if let Some(s) = start {
            self.visit_expression(s);
            let start_type = self.current_type.clone().unwrap_or(DataType::Empty);
            if !self.is_integer_type(&start_type) && start_type != DataType::Empty {
                self.set_error(ValidationError::new(
                    format!("范围起始索引需要整数类型，但得到: {:?}", start_type),
                    ValidationErrorType::TypeError,
                ));
                return;
            }
        }

        if let Some(e) = end {
            self.visit_expression(e);
            let end_type = self.current_type.clone().unwrap_or(DataType::Empty);
            if !self.is_integer_type(&end_type) && end_type != DataType::Empty {
                self.set_error(ValidationError::new(
                    format!("范围结束索引需要整数类型，但得到: {:?}", end_type),
                    ValidationErrorType::TypeError,
                ));
                return;
            }
        }

        self.set_type(DataType::List);
    }

    fn visit_path(&mut self, _items: &[Expression]) {
        self.set_type(DataType::Path);
    }

    fn visit_label(&mut self, _name: &str) {
        self.set_type(DataType::Empty);
    }

    fn visit_list_comprehension(
        &mut self,
        _variable: &str,
        source: &Expression,
        filter: Option<&Expression>,
        map: Option<&Expression>,
    ) {
        self.visit_expression(source);
        if let Some(f) = filter {
            self.visit_expression(f);
        }
        if let Some(m) = map {
            self.visit_expression(m);
        }
        self.set_type(DataType::List);
    }

    fn visit_label_tag_property(&mut self, tag: &Expression, _property: &str) {
        self.visit_expression(tag);
    }

    fn visit_tag_property(&mut self, _tag_name: &str, _property: &str) {
        self.set_type(DataType::Empty);
    }

    fn visit_edge_property(&mut self, _edge_name: &str, _property: &str) {
        self.set_type(DataType::Empty);
    }

    fn visit_predicate(&mut self, _func: &str, args: &[Expression]) {
        for arg in args {
            self.visit_expression(arg);
        }
        self.set_type(DataType::Bool);
    }

    fn visit_reduce(
        &mut self,
        _accumulator: &str,
        initial: &Expression,
        _variable: &str,
        source: &Expression,
        mapping: &Expression,
    ) {
        self.visit_expression(initial);
        self.visit_expression(source);
        self.visit_expression(mapping);
        self.set_type(DataType::Empty);
    }

    fn visit_path_build(&mut self, exprs: &[Expression]) {
        for expr in exprs {
            self.visit_expression(expr);
        }
        self.set_type(DataType::Path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_literal_type_deduction() {
        let mut validator = TypeDeduceValidator::new();
        
        let expr = Expression::literal(Value::Int(42));
        assert_eq!(validator.deduce_type(&expr).unwrap(), DataType::Int);
        
        let expr = Expression::literal(Value::String("test".to_string()));
        assert_eq!(validator.deduce_type(&expr).unwrap(), DataType::String);
    }

    #[test]
    fn test_binary_arithmetic_type_deduction() {
        let mut validator = TypeDeduceValidator::new();
        
        let expr = Expression::binary(
            Expression::literal(Value::Int(1)),
            BinaryOperator::Add,
            Expression::literal(Value::Int(2)),
        );
        assert_eq!(validator.deduce_type(&expr).unwrap(), DataType::Int);
        
        let expr = Expression::binary(
            Expression::literal(Value::Int(1)),
            BinaryOperator::Divide,
            Expression::literal(Value::Int(2)),
        );
        assert_eq!(validator.deduce_type(&expr).unwrap(), DataType::Float);
    }

    #[test]
    fn test_aggregate_type_deduction() {
        let mut validator = TypeDeduceValidator::new();
        
        let expr = Expression::aggregate(
            AggregateFunction::Count(None),
            Expression::literal(Value::Int(1)),
            false,
        );
        assert_eq!(validator.deduce_type(&expr).unwrap(), DataType::Int);
        
        let expr = Expression::aggregate(
            AggregateFunction::Sum("x".to_string()),
            Expression::variable("x"),
            false,
        );
        assert_eq!(validator.deduce_type(&expr).unwrap(), DataType::Float);
    }
}
