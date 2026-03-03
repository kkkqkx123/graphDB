//! 表达式分析器
//!
//! 统一的表达式分析接口，负责：
//! 1. 类型推导 - 将结果存储到 ExpressionContext
//! 2. 常量折叠 - 将结果存储到 ExpressionContext
//! 3. 表达式验证 - 验证表达式合法性
//!
//! 设计原则：
//! - 所有分析结果存储到 ExpressionContext，避免分散存储
//! - 利用缓存避免重复分析
//! - 支持增量分析

use std::sync::Arc;

use crate::core::types::expression::{ContextualExpression, Expression};
use crate::core::types::DataType;
use crate::core::Value;
use crate::core::error::{ValidationError, ValidationErrorType};

/// 表达式分析结果
#[derive(Debug, Clone)]
pub struct ExpressionAnalysisResult {
    /// 推导出的类型
    pub data_type: DataType,
    /// 是否为常量
    pub is_constant: bool,
    /// 常量值（如果是常量）
    pub constant_value: Option<Value>,
    /// 包含的变量列表
    pub variables: Vec<String>,
    /// 是否包含聚合函数
    pub has_aggregate: bool,
}

impl ExpressionAnalysisResult {
    /// 创建新的分析结果
    pub fn new(data_type: DataType) -> Self {
        Self {
            data_type,
            is_constant: false,
            constant_value: None,
            variables: Vec::new(),
            has_aggregate: false,
        }
    }

    /// 创建常量分析结果
    pub fn constant(data_type: DataType, value: Value) -> Self {
        Self {
            data_type,
            is_constant: true,
            constant_value: Some(value),
            variables: Vec::new(),
            has_aggregate: false,
        }
    }
}

/// 表达式分析器
///
/// 统一的表达式分析接口，整合类型推导、常量折叠和验证功能。
/// 所有分析结果存储到 ExpressionContext，确保数据一致性。
pub struct ExpressionAnalyzer;

impl ExpressionAnalyzer {
    /// 创建新的表达式分析器
    pub fn new() -> Self {
        Self
    }

    /// 分析表达式
    ///
    /// 执行完整的表达式分析：
    /// 1. 检查缓存，如果已分析则直接返回
    /// 2. 进行类型推导
    /// 3. 进行常量折叠
    /// 4. 收集变量信息
    /// 5. 存储结果到 ExpressionContext
    ///
    /// # 参数
    /// - `expr`: 要分析的上下文表达式
    /// - `variable_types`: 变量类型映射（用于类型推导）
    ///
    /// # 返回
    /// 分析结果，包含类型、常量信息等
    pub fn analyze(
        &self,
        expr: &ContextualExpression,
        variable_types: Option<&std::collections::HashMap<String, DataType>>,
    ) -> Result<ExpressionAnalysisResult, ValidationError> {
        // 检查是否已分析
        if let Some(data_type) = expr.data_type() {
            // 已分析过，直接返回缓存结果
            let constant_value = expr.constant_value();
            let is_constant = constant_value.is_some();

            return Ok(ExpressionAnalysisResult {
                data_type,
                is_constant,
                constant_value,
                variables: expr.get_variables(),
                has_aggregate: expr.contains_aggregate(),
            });
        }

        // 获取表达式
        let inner_expr = match expr.get_expression() {
            Some(e) => e,
            None => {
                return Err(ValidationError::new(
                    "表达式无效或不存在".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 执行分析
        let result = self.analyze_expression(&inner_expr, variable_types)?;

        // 存储结果到 ExpressionContext
        self.store_result(expr, &result)?;

        Ok(result)
    }

    /// 分析表达式（内部方法）
    fn analyze_expression(
        &self,
        expr: &Expression,
        variable_types: Option<&std::collections::HashMap<String, DataType>>,
    ) -> Result<ExpressionAnalysisResult, ValidationError> {
        match expr {
            Expression::Literal(value) => {
                let data_type = value.get_type();
                Ok(ExpressionAnalysisResult::constant(data_type.clone(), value.clone()))
            }

            Expression::Variable(name) => {
                let data_type = variable_types
                    .and_then(|types| types.get(name))
                    .cloned()
                    .unwrap_or(DataType::Empty);

                let mut result = ExpressionAnalysisResult::new(data_type);
                result.variables.push(name.clone());
                Ok(result)
            }

            Expression::Binary { op, left, right } => {
                self.analyze_binary_expression(op, left, right, variable_types)
            }

            Expression::Unary { op, operand } => {
                self.analyze_unary_expression(op, operand, variable_types)
            }

            Expression::Function { name, args } => {
                self.analyze_function_call(name, args, variable_types)
            }

            Expression::Aggregate { func, arg, distinct: _ } => {
                self.analyze_aggregate_expression(func, arg, variable_types)
            }

            Expression::Property { object, property: _ } => {
                // 属性访问表达式，类型取决于对象
                let obj_result = self.analyze_expression(object, variable_types)?;
                let mut result = ExpressionAnalysisResult::new(DataType::Empty);
                result.variables = obj_result.variables;
                Ok(result)
            }

            Expression::Subscript { collection, index } => {
                self.analyze_subscript_expression(collection, index, variable_types)
            }

            Expression::List(elements) => {
                self.analyze_list_expression(elements, variable_types)
            }

            Expression::Map(pairs) => {
                self.analyze_map_expression(pairs, variable_types)
            }

            Expression::Case { test_expr, conditions, default } => {
                self.analyze_case_expression(test_expr.as_deref(), conditions, default.as_deref(), variable_types)
            }

            _ => Ok(ExpressionAnalysisResult::new(DataType::Empty)),
        }
    }

    /// 分析二元表达式
    fn analyze_binary_expression(
        &self,
        op: &crate::core::BinaryOperator,
        left: &Expression,
        right: &Expression,
        variable_types: Option<&std::collections::HashMap<String, DataType>>,
    ) -> Result<ExpressionAnalysisResult, ValidationError> {
        use crate::core::BinaryOperator;

        let left_result = self.analyze_expression(left, variable_types)?;
        let right_result = self.analyze_expression(right, variable_types)?;

        // 推导结果类型
        let data_type = match op {
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::LessThan
            | BinaryOperator::LessThanOrEqual
            | BinaryOperator::GreaterThan
            | BinaryOperator::GreaterThanOrEqual
            | BinaryOperator::And
            | BinaryOperator::Or => DataType::Bool,

            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Modulo
            | BinaryOperator::Exponent => {
                self.deduce_arithmetic_type(&left_result.data_type, &right_result.data_type)
            }

            BinaryOperator::StringConcat => DataType::String,

            _ => DataType::Empty,
        };

        // 尝试常量折叠
        let constant_value = if left_result.is_constant && right_result.is_constant {
            self.fold_binary_constant(op, left_result.constant_value.as_ref(), right_result.constant_value.as_ref())
        } else {
            None
        };

        let mut result = if let Some(value) = constant_value {
            ExpressionAnalysisResult::constant(data_type, value)
        } else {
            ExpressionAnalysisResult::new(data_type)
        };

        // 合并变量列表
        result.variables = left_result.variables;
        result.variables.extend(right_result.variables);
        result.has_aggregate = left_result.has_aggregate || right_result.has_aggregate;

        Ok(result)
    }

    /// 分析一元表达式
    fn analyze_unary_expression(
        &self,
        op: &crate::core::UnaryOperator,
        operand: &Expression,
        variable_types: Option<&std::collections::HashMap<String, DataType>>,
    ) -> Result<ExpressionAnalysisResult, ValidationError> {
        use crate::core::UnaryOperator;

        let operand_result = self.analyze_expression(operand, variable_types)?;

        let data_type = match op {
            UnaryOperator::Not => DataType::Bool,
            UnaryOperator::IsNull
            | UnaryOperator::IsNotNull
            | UnaryOperator::IsEmpty
            | UnaryOperator::IsNotEmpty => DataType::Bool,
            UnaryOperator::Minus | UnaryOperator::Plus => operand_result.data_type.clone(),
        };

        // 尝试常量折叠
        let constant_value = if operand_result.is_constant {
            self.fold_unary_constant(op, operand_result.constant_value.as_ref())
        } else {
            None
        };

        let mut result = if let Some(value) = constant_value {
            ExpressionAnalysisResult::constant(data_type, value)
        } else {
            ExpressionAnalysisResult::new(data_type)
        };

        result.variables = operand_result.variables;
        result.has_aggregate = operand_result.has_aggregate;

        Ok(result)
    }

    /// 分析函数调用
    fn analyze_function_call(
        &self,
        name: &str,
        args: &[Expression],
        variable_types: Option<&std::collections::HashMap<String, DataType>>,
    ) -> Result<ExpressionAnalysisResult, ValidationError> {
        // 分析参数
        let mut all_constant = true;
        let mut variables = Vec::new();

        for arg in args {
            let arg_result = self.analyze_expression(arg, variable_types)?;
            if !arg_result.is_constant {
                all_constant = false;
            }
            variables.extend(arg_result.variables);
        }

        // 推导返回类型
        let data_type = self.deduce_function_return_type(name, args, variable_types);

        let mut result = ExpressionAnalysisResult::new(data_type);
        result.variables = variables;
        result.is_constant = all_constant; // 函数调用通常不是常量，除非是内置纯函数

        Ok(result)
    }

    /// 分析聚合表达式
    fn analyze_aggregate_expression(
        &self,
        func: &crate::core::AggregateFunction,
        arg: &Expression,
        variable_types: Option<&std::collections::HashMap<String, DataType>>,
    ) -> Result<ExpressionAnalysisResult, ValidationError> {
        let arg_result = self.analyze_expression(arg, variable_types)?;

        let data_type = self.deduce_aggregate_return_type(func, &arg_result.data_type);

        let mut result = ExpressionAnalysisResult::new(data_type);
        result.variables = arg_result.variables;
        result.has_aggregate = true;

        Ok(result)
    }

    /// 分析下标表达式
    fn analyze_subscript_expression(
        &self,
        collection: &Expression,
        index: &Expression,
        variable_types: Option<&std::collections::HashMap<String, DataType>>,
    ) -> Result<ExpressionAnalysisResult, ValidationError> {
        let coll_result = self.analyze_expression(collection, variable_types)?;
        let index_result = self.analyze_expression(index, variable_types)?;

        // 下标表达式的类型取决于集合类型
        let data_type = match &coll_result.data_type {
            DataType::List => DataType::Empty, // 无法确定元素类型
            DataType::Map => DataType::Empty,
            _ => DataType::Empty,
        };

        let mut result = ExpressionAnalysisResult::new(data_type);
        result.variables = coll_result.variables;
        result.variables.extend(index_result.variables);

        Ok(result)
    }

    /// 分析列表表达式
    fn analyze_list_expression(
        &self,
        elements: &[Expression],
        variable_types: Option<&std::collections::HashMap<String, DataType>>,
    ) -> Result<ExpressionAnalysisResult, ValidationError> {
        let mut all_constant = true;
        let mut variables = Vec::new();

        for elem in elements {
            let elem_result = self.analyze_expression(elem, variable_types)?;
            if !elem_result.is_constant {
                all_constant = false;
            }
            variables.extend(elem_result.variables);
        }

        let mut result = ExpressionAnalysisResult::new(DataType::List);
        result.variables = variables;
        result.is_constant = all_constant;

        Ok(result)
    }

    /// 分析映射表达式
    fn analyze_map_expression(
        &self,
        pairs: &[(String, Expression)],
        variable_types: Option<&std::collections::HashMap<String, DataType>>,
    ) -> Result<ExpressionAnalysisResult, ValidationError> {
        let mut all_constant = true;
        let mut variables = Vec::new();

        for (_, value) in pairs {
            let value_result = self.analyze_expression(value, variable_types)?;
            if !value_result.is_constant {
                all_constant = false;
            }
            variables.extend(value_result.variables);
        }

        let mut result = ExpressionAnalysisResult::new(DataType::Map);
        result.variables = variables;
        result.is_constant = all_constant;

        Ok(result)
    }

    /// 分析 CASE 表达式
    fn analyze_case_expression(
        &self,
        test_expr: Option<&Expression>,
        conditions: &[(Expression, Expression)],
        default: Option<&Expression>,
        variable_types: Option<&std::collections::HashMap<String, DataType>>,
    ) -> Result<ExpressionAnalysisResult, ValidationError> {
        let mut all_constant = true;
        let mut variables = Vec::new();
        let mut result_type = DataType::Empty;

        // 分析 test 表达式
        if let Some(test) = test_expr {
            let test_result = self.analyze_expression(test, variable_types)?;
            if !test_result.is_constant {
                all_constant = false;
            }
            variables.extend(test_result.variables);
        }

        // 分析条件和结果
        for (condition, value) in conditions {
            let cond_result = self.analyze_expression(condition, variable_types)?;
            let value_result = self.analyze_expression(value, variable_types)?;

            if !cond_result.is_constant || !value_result.is_constant {
                all_constant = false;
            }

            variables.extend(cond_result.variables);
            variables.extend(value_result.variables);

            // 合并结果类型
            if result_type == DataType::Empty {
                result_type = value_result.data_type;
            }
        }

        // 分析 default
        if let Some(default_expr) = default {
            let default_result = self.analyze_expression(default_expr, variable_types)?;
            if !default_result.is_constant {
                all_constant = false;
            }
            variables.extend(default_result.variables);

            if result_type == DataType::Empty {
                result_type = default_result.data_type;
            }
        }

        let mut result = ExpressionAnalysisResult::new(result_type);
        result.variables = variables;
        result.is_constant = all_constant;

        Ok(result)
    }

    /// 存储分析结果到 ExpressionContext
    fn store_result(
        &self,
        expr: &ContextualExpression,
        result: &ExpressionAnalysisResult,
    ) -> Result<(), ValidationError> {
        let context = expr.context();
        let id = expr.id();

        // 存储类型信息
        context.set_type(id, result.data_type.clone());

        // 存储常量值
        if let Some(ref value) = result.constant_value {
            context.set_constant(id, value.clone());
        }

        Ok(())
    }

    /// 推导算术表达式类型
    fn deduce_arithmetic_type(&self, left: &DataType, right: &DataType) -> DataType {
        let left_is_numeric = matches!(left,
            DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
            DataType::Float | DataType::Double
        );
        let right_is_numeric = matches!(right,
            DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
            DataType::Float | DataType::Double
        );

        if !left_is_numeric || !right_is_numeric {
            return DataType::Empty;
        }

        let left_is_float = matches!(left, DataType::Float | DataType::Double);
        let right_is_float = matches!(right, DataType::Float | DataType::Double);

        if left_is_float || right_is_float {
            DataType::Float
        } else {
            DataType::Int
        }
    }

    /// 推导函数返回类型
    fn deduce_function_return_type(
        &self,
        name: &str,
        _args: &[Expression],
        _variable_types: Option<&std::collections::HashMap<String, DataType>>,
    ) -> DataType {
        match name.to_lowercase().as_str() {
            "abs" | "length" | "size" | "round" | "floor" | "ceil" => DataType::Int,
            "sqrt" | "pow" | "sin" | "cos" | "tan" => DataType::Float,
            "concat" | "substring" | "trim" | "ltrim" | "rtrim" | "upper" | "lower" | "type" => {
                DataType::String
            }
            "id" => DataType::Int,
            "properties" => DataType::Map,
            "labels" | "keys" | "values" | "range" | "reverse" => DataType::List,
            _ => DataType::Empty,
        }
    }

    /// 推导聚合函数返回类型
    fn deduce_aggregate_return_type(
        &self,
        func: &crate::core::AggregateFunction,
        arg_type: &DataType,
    ) -> DataType {
        use crate::core::AggregateFunction;

        match func {
            AggregateFunction::Count(_) => DataType::Int,
            AggregateFunction::Sum(_) => DataType::Float,
            AggregateFunction::Avg(_) => DataType::Float,
            AggregateFunction::Max(_) | AggregateFunction::Min(_) => arg_type.clone(),
            AggregateFunction::Collect(_) => DataType::List,
            AggregateFunction::CollectSet(_) => DataType::Set,
            AggregateFunction::Distinct(_) => DataType::Set,
            AggregateFunction::Percentile(_, _) => DataType::Float,
            AggregateFunction::Std(_) => DataType::Float,
            AggregateFunction::BitAnd(_) | AggregateFunction::BitOr(_) => DataType::Int,
            AggregateFunction::GroupConcat(_, _) => DataType::String,
        }
    }

    /// 折叠二元常量表达式
    fn fold_binary_constant(
        &self,
        op: &crate::core::BinaryOperator,
        left: Option<&Value>,
        right: Option<&Value>,
    ) -> Option<Value> {
        use crate::core::BinaryOperator;
        use crate::core::Value;

        let (left, right) = (left?, right?);

        match op {
            BinaryOperator::Add => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Some(Value::Int(l + r)),
                (Value::Float(l), Value::Float(r)) => Some(Value::Float(l + r)),
                (Value::Int(l), Value::Float(r)) => Some(Value::Float(*l as f64 + r)),
                (Value::Float(l), Value::Int(r)) => Some(Value::Float(l + *r as f64)),
                _ => None,
            },

            BinaryOperator::Subtract => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Some(Value::Int(l - r)),
                (Value::Float(l), Value::Float(r)) => Some(Value::Float(l - r)),
                (Value::Int(l), Value::Float(r)) => Some(Value::Float(*l as f64 - r)),
                (Value::Float(l), Value::Int(r)) => Some(Value::Float(l - *r as f64)),
                _ => None,
            },

            BinaryOperator::Multiply => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Some(Value::Int(l * r)),
                (Value::Float(l), Value::Float(r)) => Some(Value::Float(l * r)),
                (Value::Int(l), Value::Float(r)) => Some(Value::Float(*l as f64 * r)),
                (Value::Float(l), Value::Int(r)) => Some(Value::Float(l * *r as f64)),
                _ => None,
            },

            BinaryOperator::Divide => match (left, right) {
                (Value::Int(l), Value::Int(r)) if *r != 0 => Some(Value::Int(l / r)),
                (Value::Float(l), Value::Float(r)) if *r != 0.0 => Some(Value::Float(l / r)),
                (Value::Int(l), Value::Float(r)) if *r != 0.0 => Some(Value::Float(*l as f64 / r)),
                (Value::Float(l), Value::Int(r)) if *r != 0 => Some(Value::Float(l / *r as f64)),
                _ => None,
            },

            BinaryOperator::And => match (left, right) {
                (Value::Bool(l), Value::Bool(r)) => Some(Value::Bool(*l && *r)),
                _ => None,
            },

            BinaryOperator::Or => match (left, right) {
                (Value::Bool(l), Value::Bool(r)) => Some(Value::Bool(*l || *r)),
                _ => None,
            },

            BinaryOperator::Equal => Some(Value::Bool(left == right)),
            BinaryOperator::NotEqual => Some(Value::Bool(left != right)),

            BinaryOperator::LessThan => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Some(Value::Bool(l < r)),
                (Value::Float(l), Value::Float(r)) => Some(Value::Bool(l < r)),
                (Value::Int(l), Value::Float(r)) => Some(Value::Bool((*l as f64) < *r)),
                (Value::Float(l), Value::Int(r)) => Some(Value::Bool(*l < *r as f64)),
                _ => None,
            },

            BinaryOperator::LessThanOrEqual => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Some(Value::Bool(l <= r)),
                (Value::Float(l), Value::Float(r)) => Some(Value::Bool(l <= r)),
                (Value::Int(l), Value::Float(r)) => Some(Value::Bool((*l as f64) <= *r)),
                (Value::Float(l), Value::Int(r)) => Some(Value::Bool(*l <= *r as f64)),
                _ => None,
            },

            BinaryOperator::GreaterThan => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Some(Value::Bool(l > r)),
                (Value::Float(l), Value::Float(r)) => Some(Value::Bool(l > r)),
                (Value::Int(l), Value::Float(r)) => Some(Value::Bool((*l as f64) > *r)),
                (Value::Float(l), Value::Int(r)) => Some(Value::Bool(*l > *r as f64)),
                _ => None,
            },

            BinaryOperator::GreaterThanOrEqual => match (left, right) {
                (Value::Int(l), Value::Int(r)) => Some(Value::Bool(l >= r)),
                (Value::Float(l), Value::Float(r)) => Some(Value::Bool(l >= r)),
                (Value::Int(l), Value::Float(r)) => Some(Value::Bool((*l as f64) >= *r)),
                (Value::Float(l), Value::Int(r)) => Some(Value::Bool(*l >= *r as f64)),
                _ => None,
            },

            BinaryOperator::StringConcat => match (left, right) {
                (Value::String(l), Value::String(r)) => Some(Value::String(format!("{}{}", l, r))),
                _ => None,
            },

            _ => None,
        }
    }

    /// 折叠一元常量表达式
    fn fold_unary_constant(
        &self,
        op: &crate::core::UnaryOperator,
        operand: Option<&Value>,
    ) -> Option<Value> {
        use crate::core::UnaryOperator;
        use crate::core::Value;

        let operand = operand?;

        match op {
            UnaryOperator::Not => match operand {
                Value::Bool(b) => Some(Value::Bool(!b)),
                _ => None,
            },

            UnaryOperator::Minus => match operand {
                Value::Int(i) => Some(Value::Int(-i)),
                Value::Float(f) => Some(Value::Float(-f)),
                _ => None,
            },

            UnaryOperator::IsNull => Some(Value::Bool(matches!(operand, Value::Null(_)))),
            UnaryOperator::IsNotNull => Some(Value::Bool(!matches!(operand, Value::Null(_)))),

            _ => None,
        }
    }
}

impl Default for ExpressionAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::{ExpressionMeta, ExpressionContext};

    #[test]
    fn test_analyze_literal() {
        let ctx = Arc::new(ExpressionContext::new());
        let expr = Expression::literal(42);
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, ctx.clone());

        let analyzer = ExpressionAnalyzer::new();
        let result = analyzer.analyze(&ctx_expr, None).expect("分析失败");

        assert_eq!(result.data_type, DataType::Int);
        assert!(result.is_constant);
        assert_eq!(result.constant_value, Some(Value::Int(42)));
    }

    #[test]
    fn test_analyze_binary_constant_fold() {
        let ctx = Arc::new(ExpressionContext::new());
        let left = Expression::literal(10);
        let right = Expression::literal(20);
        let expr = Expression::binary(left, crate::core::BinaryOperator::Add, right);
        let meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(meta);
        let ctx_expr = ContextualExpression::new(id, ctx.clone());

        let analyzer = ExpressionAnalyzer::new();
        let result = analyzer.analyze(&ctx_expr, None).expect("分析失败");

        assert_eq!(result.data_type, DataType::Int);
        assert!(result.is_constant);
        assert_eq!(result.constant_value, Some(Value::Int(30)));

        // 验证存储到 ExpressionContext
        assert_eq!(ctx_expr.data_type(), Some(DataType::Int));
        assert_eq!(ctx_expr.constant_value(), Some(Value::Int(30)));
    }
}
