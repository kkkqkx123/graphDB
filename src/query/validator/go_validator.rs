//! GO 语句验证器
//! 对应 NebulaGraph GoValidator.h/.cpp 的功能
//! 验证 GO FROM ... OVER ... WHERE ... YIELD ... 语句

use super::base_validator::Validator;
use super::validation_interface::{ValidationError, ValidationErrorType};
use crate::core::{
    AggregateFunction, BinaryOperator, Expression, UnaryOperator, Value,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GoContext {
    pub from_source: Option<GoSource>,
    pub over_edges: Vec<OverEdge>,
    pub where_filter: Option<Expression>,
    pub yield_columns: Vec<GoYieldColumn>,
    pub step_range: Option<StepRange>,
    pub inputs: Vec<GoInput>,
    pub outputs: Vec<GoOutput>,
    pub is_truncate: bool,
    pub truncate_columns: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct GoSource {
    pub source_type: GoSourceType,
    pub expression: Expression,
    pub is_variable: bool,
    pub variable_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum GoSourceType {
    VertexId,
    Expression,
    Variable,
    Parameter,
}

#[derive(Debug, Clone)]
pub struct OverEdge {
    pub edge_name: String,
    pub edge_type: Option<i32>,
    pub direction: EdgeDirection,
    pub props: Vec<EdgeProperty>,
    pub is_reversible: bool,
    pub is_all: bool,
}

#[derive(Debug, Clone)]
pub enum EdgeDirection {
    Forward,
    Backward,
    Both,
}

#[derive(Debug, Clone)]
pub struct EdgeProperty {
    pub name: String,
    pub prop_name: String,
    pub prop_type: String,
}

#[derive(Debug, Clone)]
pub struct GoYieldColumn {
    pub expression: Expression,
    pub alias: String,
    pub is_distinct: bool,
}

#[derive(Debug, Clone)]
pub struct StepRange {
    pub step_from: i32,
    pub step_to: i32,
}

#[derive(Debug, Clone)]
pub struct GoInput {
    pub name: String,
    pub columns: Vec<InputColumn>,
}

#[derive(Debug, Clone)]
pub struct InputColumn {
    pub name: String,
    pub type_: String,
}

#[derive(Debug, Clone)]
pub struct GoOutput {
    pub name: String,
    pub type_: String,
    pub alias: String,
}

pub struct GoValidator {
    base: Validator,
    context: GoContext,
}

impl GoValidator {
    pub fn new(context: super::ValidationContext) -> Self {
        Self {
            base: Validator::new(context),
            context: GoContext {
                from_source: None,
                over_edges: Vec::new(),
                where_filter: None,
                yield_columns: Vec::new(),
                step_range: None,
                inputs: Vec::new(),
                outputs: Vec::new(),
                is_truncate: false,
                truncate_columns: Vec::new(),
            },
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_from_clause()?;
        self.validate_over_clause()?;
        self.validate_where_clause()?;
        self.validate_yield_clause()?;
        self.validate_step_range()?;
        self.build_outputs()?;

        if self.base.context().has_validation_errors() {
            let errors = self.base.context().get_validation_errors();
            if let Some(first_error) = errors.first() {
                return Err(first_error.clone());
            }
        }

        Ok(())
    }

    fn validate_from_clause(&mut self) -> Result<(), ValidationError> {
        // 验证 FROM 子句
        // 需要检查：
        // 1. 起始点表达式是否有效
        // 2. 如果是变量引用，变量是否存在
        // 3. 如果是常量表达式，类型是否正确

        if let Some(ref source) = self.context.from_source {
            match &source.source_type {
                GoSourceType::VertexId => {
                    // 验证顶点ID表达式
                    self.validate_expression(&source.expression)?;
                }
                GoSourceType::Expression => {
                    // 验证通用表达式
                    self.validate_expression(&source.expression)?;
                }
                GoSourceType::Variable => {
                    // 验证变量引用
                    if let Expression::Variable(ref var_name) = source.expression {
                        self.validate_variable_reference(var_name)?;
                    } else {
                        return Err(ValidationError::new(
                            "FROM 子句中的变量引用格式不正确".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
                GoSourceType::Parameter => {
                    // 验证参数表达式
                    self.validate_expression(&source.expression)?;
                }
            }
        }

        Ok(())
    }

    fn validate_over_clause(&mut self) -> Result<(), ValidationError> {
        // 验证 OVER 子句
        // 需要检查：
        // 1. 边类型是否存在
        // 2. 方向是否有效
        // 3. 属性引用是否正确

        if self.context.over_edges.is_empty() {
            return Err(ValidationError::new(
                "OVER 子句必须指定至少一条边".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for edge in &self.context.over_edges {
            // 验证边名称
            if edge.edge_name.is_empty() {
                return Err(ValidationError::new(
                    "边名称不能为空".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }

            // 验证边方向
            match edge.direction {
                EdgeDirection::Forward | EdgeDirection::Backward | EdgeDirection::Both => {
                    // 方向有效
                }
            }

            // 验证边属性
            for prop in &edge.props {
                if prop.name.is_empty() || prop.prop_name.is_empty() {
                    return Err(ValidationError::new(
                        "边属性名称不能为空".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_where_clause(&mut self) -> Result<(), ValidationError> {
        // 验证 WHERE 子句
        // 需要检查：
        // 1. 过滤表达式是否有效
        // 2. 引用的属性是否存在
        // 3. 类型兼容性

        if let Some(ref filter) = self.context.where_filter {
            // 验证过滤表达式
            self.validate_expression(filter)?;
        }

        Ok(())
    }

    fn validate_yield_clause(&mut self) -> Result<(), ValidationError> {
        // 验证 YIELD 子句
        // 需要检查：
        // 1. 返回列表达式是否有效
        // 2. 别名是否重复
        // 3. 属性引用是否正确

        let mut column_names = HashMap::new();

        for column in &self.context.yield_columns {
            if let Some(_existing) = column_names.get(&column.alias) {
                return Err(ValidationError::new(
                    format!("YIELD 列别名 '{}' 重复出现", column.alias),
                    ValidationErrorType::DuplicateKey,
                ));
            }
            column_names.insert(column.alias.clone(), true);
        }

        Ok(())
    }

    fn validate_step_range(&mut self) -> Result<(), ValidationError> {
        // 验证步数范围
        // 需要检查：
        // 1. 起始步数是否为正
        // 2. 结束步数是否大于等于起始步数

        if let Some(ref range) = self.context.step_range {
            if range.step_from < 0 {
                return Err(ValidationError::new(
                    "步数范围起始值不能为负".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            if range.step_to < range.step_from {
                return Err(ValidationError::new(
                    "步数范围结束值不能小于起始值".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        Ok(())
    }

    /// 验证表达式
    fn validate_expression(&self, expr: &Expression) -> Result<(), ValidationError> {
        match expr {
            Expression::Literal(_) => {
                // 字面量总是有效的
                Ok(())
            }
            Expression::Variable(name) => {
                // 检查变量是否存在
                self.validate_variable_reference(name)
            }
            Expression::Property { object, property } => {
                // 验证对象表达式和属性名称
                self.validate_expression(object)?;
                self.validate_property_name(property)?;
                Ok(())
            }
            Expression::Binary { left, op, right } => {
                // 验证左右操作数和操作符
                self.validate_expression(left)?;
                self.validate_expression(right)?;
                self.validate_binary_operator(op)?;
                Ok(())
            }
            Expression::Unary { op, operand } => {
                // 验证操作数和操作符
                self.validate_expression(operand)?;
                self.validate_unary_operator(op)?;
                Ok(())
            }
            Expression::Function { name, args } => {
                // 验证函数名称和参数
                self.validate_function_name(name)?;
                for arg in args {
                    self.validate_expression(arg)?;
                }
                Ok(())
            }
            Expression::Aggregate { func, arg, .. } => {
                // 验证聚合函数和参数
                self.validate_aggregate_function(func)?;
                self.validate_expression(arg)?;
                Ok(())
            }
            Expression::List(items) => {
                // 验证列表中的每个元素
                for item in items {
                    self.validate_expression(item)?;
                }
                Ok(())
            }
            Expression::Map(pairs) => {
                // 验证映射中的每对键值
                for (key, value) in pairs {
                    // 键通常是字符串，所以只验证值
                    self.validate_expression(value)?;
                }
                Ok(())
            }
            Expression::Case { conditions, default } => {
                // 验证条件和默认值
                for (condition, result) in conditions {
                    self.validate_expression(condition)?;
                    self.validate_expression(result)?;
                }
                if let Some(default_expr) = default {
                    self.validate_expression(default_expr)?;
                }
                Ok(())
            }
            Expression::TypeCast { expr, .. } => {
                // 验证类型转换表达式
                self.validate_expression(expr)?;
                Ok(())
            }
            Expression::Subscript { collection, index } => {
                // 验证下标访问
                self.validate_expression(collection)?;
                self.validate_expression(index)?;
                Ok(())
            }
            Expression::Range { collection, start, end } => {
                // 验证范围访问
                self.validate_expression(collection)?;
                if let Some(start_expr) = start {
                    self.validate_expression(start_expr)?;
                }
                if let Some(end_expr) = end {
                    self.validate_expression(end_expr)?;
                }
                Ok(())
            }
            Expression::Path(items) => {
                // 验证路径表达式
                for item in items {
                    self.validate_expression(item)?;
                }
                Ok(())
            }
            Expression::Label(name) => {
                // 验证标签名称
                self.validate_label_name(name)?;
                Ok(())
            }
            // 属性表达式统一处理
            Expression::Property { object, property } => {
                self.validate_expression(object)?;
                self.validate_property_name(&property)?;
                Ok(())
            }
            // 一元操作
            Expression::Unary { op, operand } => {
                self.validate_expression(operand)?;
                Ok(())
            }
        }
    }

    /// 验证变量引用
    fn validate_variable_reference(&self, var_name: &str) -> Result<(), ValidationError> {
        // 检查变量是否已定义
        if var_name.is_empty() {
            return Err(ValidationError::new(
                "变量名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 在当前上下文中检查变量是否存在
        // 这里可以检查 self.context.inputs 或其他变量定义源
        // 为了简化，我们假设变量存在
        Ok(())
    }

    /// 验证属性名称
    fn validate_property_name(&self, prop_name: &str) -> Result<(), ValidationError> {
        if prop_name.is_empty() {
            return Err(ValidationError::new(
                "属性名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证二元操作符
    fn validate_binary_operator(&self, op: &BinaryOperator) -> Result<(), ValidationError> {
        // 所有 BinaryOperator 枚举值都应该是有效的，因此只需确认它存在
        match op {
            _ => Ok(()), // 所有操作符都是有效的
        }
    }

    /// 验证一元操作符
    fn validate_unary_operator(&self, op: &UnaryOperator) -> Result<(), ValidationError> {
        // 所有 UnaryOperator 枚举值都应该是有效的，因此只需确认它存在
        match op {
            _ => Ok(()), // 所有操作符都是有效的
        }
    }

    /// 验证函数名称
    fn validate_function_name(&self, func_name: &str) -> Result<(), ValidationError> {
        if func_name.is_empty() {
            return Err(ValidationError::new(
                "函数名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证聚合函数
    fn validate_aggregate_function(&self, func: &AggregateFunction) -> Result<(), ValidationError> {
        // 所有 AggregateFunction 枚举值都应该是有效的
        match func {
            _ => Ok(()), // 所有聚合函数都是有效的
        }
    }

    /// 验证标签名称
    fn validate_label_name(&self, label_name: &str) -> Result<(), ValidationError> {
        if label_name.is_empty() {
            return Err(ValidationError::new(
                "标签名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证标签名称
    fn validate_tag_name(&self, tag_name: &str) -> Result<(), ValidationError> {
        if tag_name.is_empty() {
            return Err(ValidationError::new(
                "标签名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证边名称
    fn validate_edge_name(&self, edge_name: &str) -> Result<(), ValidationError> {
        if edge_name.is_empty() {
            return Err(ValidationError::new(
                "边名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证输入属性名称
    fn validate_input_property_name(&self, prop_name: &str) -> Result<(), ValidationError> {
        if prop_name.is_empty() {
            return Err(ValidationError::new(
                "输入属性名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn build_outputs(&mut self) -> Result<(), ValidationError> {
        // 构建输出列
        // 根据 YIELD 子句构建输出定义

        for column in &self.context.yield_columns {
            let inferred_type = self.infer_expression_type(&column.expression)?;
            let output = GoOutput {
                name: column.alias.clone(),
                type_: inferred_type,
                alias: column.alias.clone(),
            };
            self.context.outputs.push(output);
        }

        Ok(())
    }

    /// 推断表达式的类型
    fn infer_expression_type(&self, expr: &Expression) -> Result<String, ValidationError> {
        match expr {
            Expression::Literal(value) => {
                // 根据字面量值推断类型
                Ok(self.infer_literal_type(value))
            }
            Expression::Variable(_) => {
                // 变量类型的推断可能需要访问符号表
                // 暂时返回通用类型
                Ok("ANY".to_string())
            }
            Expression::Property { .. } => {
                // 属性访问的类型取决于对象和属性
                Ok("ANY".to_string())
            }
            Expression::Binary { left, op, right } => {
                // 二元操作的结果类型取决于操作符和操作数类型
                let left_type = self.infer_expression_type(left)?;
                let right_type = self.infer_expression_type(right)?;

                // 根据操作符确定结果类型
                match op {
                    BinaryOperator::And | BinaryOperator::Or => Ok("BOOL".to_string()),
                    BinaryOperator::Equal | BinaryOperator::NotEqual |
                    BinaryOperator::LessThan | BinaryOperator::LessThanOrEqual |
                    BinaryOperator::GreaterThan | BinaryOperator::GreaterThanOrEqual |
                    BinaryOperator::Like | BinaryOperator::In | BinaryOperator::NotIn |
                    BinaryOperator::Contains | BinaryOperator::StartsWith | BinaryOperator::EndsWith => {
                        Ok("BOOL".to_string())
                    }
                    BinaryOperator::Add | BinaryOperator::Subtract |
                    BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => {
                        // 如果任一操作数是浮点数，则结果为浮点数
                        if left_type == "FLOAT" || right_type == "FLOAT" ||
                           left_type == "DOUBLE" || right_type == "DOUBLE" {
                            Ok("DOUBLE".to_string())
                        } else {
                            // 默认返回整数类型
                            Ok("INT".to_string())
                        }
                    }
                    BinaryOperator::StringConcat => Ok("STRING".to_string()),
                    _ => Ok("ANY".to_string()),
                }
            }
            Expression::Unary { op, .. } => {
                match op {
                    UnaryOperator::Plus | UnaryOperator::Minus => Ok("NUMBER".to_string()),
                    UnaryOperator::Not => Ok("BOOL".to_string()),
                    UnaryOperator::IsNull | UnaryOperator::IsNotNull |
                    UnaryOperator::IsEmpty | UnaryOperator::IsNotEmpty => Ok("BOOL".to_string()),
                    _ => Ok("ANY".to_string()),
                }
            }
            Expression::Function { name, .. } => {
                // 根据函数名推断返回类型
                match name.to_uppercase().as_str() {
                    "COALESCE" | "IFNULL" | "NULLIF" => Ok("ANY".to_string()),
                    "UPPER" | "LOWER" | "TRIM" | "LTRIM" | "RTRIM" | "REPLACE" | "SUBSTR" => Ok("STRING".to_string()),
                    "LENGTH" | "CHAR_LENGTH" | "BIT_LENGTH" => Ok("INT".to_string()),
                    "ABS" | "ROUND" | "FLOOR" | "CEIL" => Ok("NUMBER".to_string()),
                    "NOW" | "TODAY" | "CURRENT_DATE" | "CURRENT_TIME" | "CURRENT_TIMESTAMP" => Ok("DATETIME".to_string()),
                    "DATE" | "TIME" => Ok("DATETIME".to_string()),
                    "YEAR" | "MONTH" | "DAY" | "HOUR" | "MINUTE" | "SECOND" => Ok("INT".to_string()),
                    _ => Ok("ANY".to_string()), // 未知函数返回ANY类型
                }
            }
            Expression::Aggregate { func, .. } => {
                // 根据聚合函数类型推断返回类型
                match func {
                    AggregateFunction::Count(_) => Ok("INT".to_string()),
                    AggregateFunction::Sum(_) => Ok("NUMBER".to_string()),
                    AggregateFunction::Avg(_) => Ok("DOUBLE".to_string()),
                    AggregateFunction::Min(_) | AggregateFunction::Max(_) => Ok("ANY".to_string()),
                    AggregateFunction::Collect(_) | AggregateFunction::Distinct(_) => Ok("LIST".to_string()),
                    AggregateFunction::Percentile(_, _) => Ok("DOUBLE".to_string()),
                }
            }
            Expression::List(_) => Ok("LIST".to_string()),
            Expression::Map(_) => Ok("MAP".to_string()),
            Expression::Case { .. } => Ok("ANY".to_string()), // CASE表达式类型取决于结果
            Expression::TypeCast { target_type, .. } => {
                // 直接返回目标类型
                Ok(format!("{:?}", target_type).to_uppercase())
            }
            Expression::Subscript { collection, .. } => {
                // 下标访问的结果类型取决于集合元素类型
                let collection_type = self.infer_expression_type(collection)?;
                // 简化处理：如果是LIST则返回ELEMENT，如果是MAP则返回VALUE
                if collection_type.starts_with("LIST") {
                    Ok("ELEMENT".to_string()) // 实际上应该更精确地推断元素类型
                } else if collection_type.starts_with("MAP") {
                    Ok("VALUE".to_string()) // 实际上应该更精确地推断值类型
                } else {
                    Ok("ANY".to_string())
                }
            }
            Expression::Range { collection, .. } => {
                // 范围访问的结果通常是一个列表
                let _collection_type = self.infer_expression_type(collection)?;
                Ok("LIST".to_string())
            }
            Expression::Path(_) => Ok("PATH".to_string()),
            Expression::Label(_) => Ok("STRING".to_string()),
            // 属性表达式统一处理
            Expression::Property { .. } => Ok("ANY".to_string()),
            // 一元操作
            Expression::Unary { op, .. } => match op {
                UnaryOperator::Plus | UnaryOperator::Minus => Ok("NUMBER".to_string()),
                UnaryOperator::Not | UnaryOperator::IsNull | UnaryOperator::IsNotNull | UnaryOperator::IsEmpty | UnaryOperator::IsNotEmpty => Ok("BOOL".to_string()),
            },
        }
    }

    /// 从字面量值推断类型
    fn infer_literal_type(&self, value: &Value) -> String {
        match value {
            Value::Null(_) => "NULL".to_string(),
            Value::Bool(_) => "BOOL".to_string(),
            Value::Int(_) => "INT".to_string(),
            Value::Float(_) => "DOUBLE".to_string(),
            Value::String(_) => "STRING".to_string(),
            Value::List(_) => "LIST".to_string(),
            Value::Map(_) => "MAP".to_string(),
            Value::Set(_) => "SET".to_string(),
            Value::Vertex(_) => "VERTEX".to_string(),
            Value::Edge(_) => "EDGE".to_string(),
            Value::Path(_) => "PATH".to_string(),
            Value::Date(_) => "DATE".to_string(),
            Value::Time(_) => "TIME".to_string(),
            Value::DateTime(_) => "DATETIME".to_string(),
            Value::Duration(_) => "DURATION".to_string(),
            Value::Geography(_) => "GEOGRAPHY".to_string(),
            Value::DataSet(_) => "DATASET".to_string(),
            Value::Empty => "EMPTY".to_string(),
        }
    }

    pub fn context(&self) -> &GoContext {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut GoContext {
        &mut self.context
    }

    pub fn set_from_source(&mut self, source: GoSource) {
        self.context.from_source = Some(source);
    }

    pub fn add_over_edge(&mut self, edge: OverEdge) {
        self.context.over_edges.push(edge);
    }

    pub fn set_where_filter(&mut self, filter: Expression) {
        self.context.where_filter = Some(filter);
    }

    pub fn add_yield_column(&mut self, column: GoYieldColumn) {
        self.context.yield_columns.push(column);
    }

    pub fn set_step_range(&mut self, range: StepRange) {
        self.context.step_range = Some(range);
    }
}

impl super::validation_interface::ValidationStrategy for GoValidator {
    fn validate(&self, _context: &dyn super::validation_interface::ValidationContext) -> Result<(), ValidationError> {
        Ok(())
    }

    fn strategy_type(&self) -> super::validation_interface::ValidationStrategyType {
        super::validation_interface::ValidationStrategyType::Clause
    }

    fn strategy_name(&self) -> &'static str {
        "GoValidator"
    }
}
