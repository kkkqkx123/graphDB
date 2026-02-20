//! GO 语句验证器
//! 对应 NebulaGraph GoValidator.h/.cpp 的功能
//! 验证 GO FROM ... OVER ... WHERE ... YIELD ... 语句

use super::base_validator::Validator;
use super::strategies::expression_rewriter::ExpressionRewriter;
use super::strategies::type_deduce::TypeDeduceValidator;
use super::validation_interface::{ValidationError, ValidationErrorType};
use crate::core::{
    AggregateFunction, BinaryOperator, DataType, Expression, UnaryOperator,
};
use crate::core::types::EdgeDirection;
use crate::query::parser::ast::stmt::GoStmt;
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

#[derive(Debug, Clone, PartialEq)]
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
pub struct EdgeProperty {
    pub name: String,
    pub prop_name: String,
    pub prop_type: DataType,
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
    pub type_: DataType,
}

#[derive(Debug, Clone)]
pub struct GoOutput {
    pub name: String,
    pub type_: DataType,
    pub alias: String,
}

pub struct GoValidator {
    base: Validator,
    context: GoContext,
}

impl GoValidator {
    pub fn new(context: super::ValidationContext) -> Self {
        Self {
            base: Validator::with_context(context),
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
            // EdgeDirection 已统一为 core::types::EdgeDirection (Out, In, Both)
            // 所有变体都是有效的

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
        // 4. 重写表达式以适配语义

        if let Some(ref filter) = self.context.where_filter {
            // 重写表达式：将边属性函数转换为标签属性表达式
            let rewriter = ExpressionRewriter::new();
            let rewritten_filter = rewriter.rewrite_edge_prop_func_to_label_attr(filter);

            // 重写表达式：将标签属性转换为边属性表达式
            let rewritten_filter = rewriter.rewrite_label_attr_to_edge_prop(&rewritten_filter);

            // 验证重写后的表达式
            self.validate_expression(&rewritten_filter)?;

            // 推断表达式类型
            let mut type_validator = TypeDeduceValidator::new();
            let expr_type = type_validator.deduce_type(&rewritten_filter)?;

            // WHERE 子句必须返回布尔类型
            if expr_type != DataType::Bool && expr_type != DataType::Null && expr_type != DataType::Empty {
                return Err(ValidationError::new(
                    format!(
                        "WHERE 子句表达式必须返回布尔类型，但得到: {:?}",
                        expr_type
                    ),
                    ValidationErrorType::TypeError,
                ));
            }
        }

        Ok(())
    }

    fn validate_yield_clause(&mut self) -> Result<(), ValidationError> {
        // 验证 YIELD 子句
        // 需要检查：
        // 1. 返回列表达式是否有效
        // 2. 别名是否重复
        // 3. 属性引用是否正确
        // 4. 重写表达式以适配语义
        // 5. 验证表达式类型
        // 6. 确保边在 OVER 子句中声明

        let mut column_names = HashMap::new();
        let rewriter = ExpressionRewriter::new();

        for column in &self.context.yield_columns {
            if let Some(_existing) = column_names.get(&column.alias) {
                return Err(ValidationError::new(
                    format!("YIELD 列别名 '{}' 重复出现", column.alias),
                    ValidationErrorType::DuplicateKey,
                ));
            }
            column_names.insert(column.alias.clone(), true);

            // 重写表达式：将边属性函数转换为标签属性表达式
            let rewritten_expr = rewriter.rewrite_edge_prop_func_to_label_attr(&column.expression);

            // 重写表达式：将标签属性转换为边属性表达式
            let rewritten_expr = rewriter.rewrite_label_attr_to_edge_prop(&rewritten_expr);

            // 验证重写后的表达式
            self.validate_expression(&rewritten_expr)?;

            // 推断表达式类型
            let mut type_validator = TypeDeduceValidator::new();
            let _expr_type = type_validator.deduce_type(&rewritten_expr)?;
        }

        // 检查所有引用的边是否在 OVER 子句中声明
        // 这里需要收集表达式中使用的边属性，然后检查它们是否在 self.context.over_edges 中
        // 由于当前实现中没有属性收集功能，这里暂时跳过

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
    fn validate_expression(&self, expression: &Expression) -> Result<(), ValidationError> {
        match expression {
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
                for (_key, value) in pairs {
                    // 键通常是字符串，所以只验证值
                    self.validate_expression(value)?;
                }
                Ok(())
            }
            Expression::Case { test_expr, conditions, default } => {
                // 验证条件和默认值
                if let Some(test_expression) = test_expr {
                    self.validate_expression(test_expression)?;
                }
                for (condition, result) in conditions {
                    self.validate_expression(condition)?;
                    self.validate_expression(result)?;
                }
                if let Some(default_expression) = default {
                    self.validate_expression(default_expression)?;
                }
                Ok(())
            }
            Expression::TypeCast { expression, .. } => {
                // 验证类型转换表达式
                self.validate_expression(expression)?;
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
                if let Some(start_expression) = start {
                    self.validate_expression(start_expression)?;
                }
                if let Some(end_expression) = end {
                    self.validate_expression(end_expression)?;
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
            Expression::ListComprehension { .. } => Ok(()),
            Expression::LabelTagProperty { tag, .. } => self.validate_expression(tag),
            Expression::TagProperty { .. } => Ok(()),
            Expression::EdgeProperty { .. } => Ok(()),
            Expression::Predicate { args, .. } => {
                for arg in args {
                    self.validate_expression(arg)?;
                }
                Ok(())
            }
            Expression::Reduce { initial, source, mapping, .. } => {
                self.validate_expression(initial)?;
                self.validate_expression(source)?;
                self.validate_expression(mapping)?;
                Ok(())
            }
            Expression::PathBuild(exprs) => {
                for expr in exprs {
                    self.validate_expression(expr)?;
                }
                Ok(())
            }
        }
    }

    /// 验证变量引用
    /// 
    /// 类似于 nebula-graph 的变量验证逻辑：
    /// 1. 检查变量是否已定义
    /// 2. 检查变量是否在 FROM 子句中声明
    /// 3. 确保变量引用的一致性
    fn validate_variable_reference(&self, var_name: &str) -> Result<(), ValidationError> {
        if var_name.is_empty() {
            return Err(ValidationError::new(
                "变量名不能为空".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 检查是否是输入变量 ($-)
        if var_name == "$-" {
            // 检查 FROM 子句是否使用了管道输入
            if let Some(ref source) = self.context.from_source {
                if source.source_type != GoSourceType::Expression {
                    return Err(ValidationError::new(
                        "$- 必须在 FROM 中使用管道输入".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            return Ok(());
        }

        // 检查是否是用户定义的变量
        if let Some(ref source) = self.context.from_source {
            if source.source_type == GoSourceType::Variable {
                if let Some(ref variable_name) = source.variable_name {
                    if var_name != variable_name {
                        return Err(ValidationError::new(
                            format!(
                                "变量 '{}' 必须在 FROM 中引用，而不是 '{}'",
                                var_name, variable_name
                            ),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            } else {
                // 如果 FROM 子句不是变量类型，则不允许在其他地方引用变量
                return Err(ValidationError::new(
                    "变量必须在 FROM 中引用才能在 WHERE 或 YIELD 中使用".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        // 检查变量是否在 inputs 中定义
        let input_exists = self.context.inputs.iter().any(|input| input.name == var_name);
        if !input_exists {
            return Err(ValidationError::new(
                format!("变量 '{}' 未定义", var_name),
                ValidationErrorType::SemanticError,
            ));
        }

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
    /// 
    /// 使用类型推导验证器来推断表达式类型
    /// 类似于 nebula-graph 的 deduceExprType 函数
    fn infer_expression_type(&self, expression: &Expression) -> Result<DataType, ValidationError> {
        let mut type_validator = TypeDeduceValidator::new();
        type_validator.deduce_type(expression)
    }

    pub fn context(&self) -> &GoContext {
        &self.context
    }

    /// 获取基础验证器的上下文
    pub fn base_context(&self) -> &super::ValidationContext {
        self.base.context()
    }

    /// 获取基础验证器的上下文的可变引用
    pub fn base_context_mut(&mut self) -> &mut super::ValidationContext {
        self.base.context_mut()
    }

    /// 从 GoStmt 构建并验证
    /// 
    /// 这是从 AST 直接验证的入口点
    pub fn validate_from_stmt(&mut self, go_stmt: &GoStmt) -> Result<(), ValidationError> {
        // 1. 构建 FROM 上下文
        self.build_from_context(&go_stmt.from)?;

        // 2. 构建 OVER 上下文
        if let Some(ref over) = go_stmt.over {
            self.build_over_context(over)?;
        }

        // 3. 构建 WHERE 上下文
        self.context.where_filter = go_stmt.where_clause.clone();

        // 4. 构建 YIELD 上下文
        if let Some(ref yield_clause) = go_stmt.yield_clause {
            self.build_yield_context(yield_clause)?;
        }

        // 5. 构建步数范围
        self.build_step_range(&go_stmt.steps)?;

        // 6. 执行验证
        self.validate()
    }

    /// 从 FROM 子句构建上下文
    fn build_from_context(
        &mut self,
        from: &crate::query::parser::ast::stmt::FromClause,
    ) -> Result<(), ValidationError> {
        if from.vertices.is_empty() {
            return Err(ValidationError::new(
                "GO 语句必须指定 FROM 顶点".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 使用第一个顶点表达式作为源
        let first_vertex = from.vertices.first().unwrap();
        
        let source_type = match first_vertex {
            Expression::Variable(_) => GoSourceType::Variable,
            Expression::List(_) => GoSourceType::VertexId,
            _ => GoSourceType::Expression,
        };

        let variable_name = match first_vertex {
            Expression::Variable(name) => Some(name.clone()),
            _ => None,
        };

        self.context.from_source = Some(GoSource {
            source_type,
            expression: first_vertex.clone(),
            is_variable: variable_name.is_some(),
            variable_name,
        });

        Ok(())
    }

    /// 从 OVER 子句构建上下文
    fn build_over_context(
        &mut self,
        over: &crate::query::parser::ast::stmt::OverClause,
    ) -> Result<(), ValidationError> {
        if over.edge_types.is_empty() {
            return Err(ValidationError::new(
                "OVER 子句必须指定至少一条边".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for edge_type in &over.edge_types {
            self.context.over_edges.push(OverEdge {
                edge_name: edge_type.clone(),
                edge_type: None, // 稍后从 Schema 中解析
                direction: over.direction,
                props: Vec::new(),
                is_reversible: over.direction == EdgeDirection::Both,
                is_all: false,
            });
        }

        Ok(())
    }

    /// 从 YIELD 子句构建上下文
    fn build_yield_context(
        &mut self,
        yield_clause: &crate::query::parser::ast::stmt::YieldClause,
    ) -> Result<(), ValidationError> {
        for item in &yield_clause.items {
            let column = GoYieldColumn {
                expression: item.expression.clone(),
                alias: item.alias.clone().unwrap_or_else(|| {
                    // 如果没有别名，尝试从表达式生成
                    format!("col_{}", self.context.yield_columns.len())
                }),
                is_distinct: false, // TODO: 支持 DISTINCT
            };
            self.context.yield_columns.push(column);
        }

        Ok(())
    }

    /// 构建步数范围
    fn build_step_range(
        &mut self,
        steps: &crate::query::parser::ast::stmt::Steps,
    ) -> Result<(), ValidationError> {
        let (from, to) = match steps {
            crate::query::parser::ast::stmt::Steps::Fixed(n) => (*n as i32, *n as i32),
            crate::query::parser::ast::stmt::Steps::Range { min, max } => (*min as i32, *max as i32),
            crate::query::parser::ast::stmt::Steps::Variable(_) => {
                return Err(ValidationError::new(
                    "GO 语句不支持变量步数".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        self.context.step_range = Some(StepRange {
            step_from: from,
            step_to: to,
        });

        Ok(())
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
