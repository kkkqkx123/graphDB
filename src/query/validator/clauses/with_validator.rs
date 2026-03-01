//! With 语句验证器
//! 用于验证 WITH 语句（Cypher 风格的管道子句）
//! 参考 nebula-graph MatchValidator.cpp 中的 With 子句验证

use std::sync::Arc;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::types::expression::contextual::ContextualExpression;
use crate::core::Expression;
use crate::query::QueryContext;
use crate::query::parser::ast::stmt::{WithStmt, ReturnItem};
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};

/// With 语句验证器
#[derive(Debug)]
pub struct WithValidator {
    items: Vec<ReturnItem>,
    where_clause: Option<ContextualExpression>,
    distinct: bool,
    order_by: Option<crate::query::parser::ast::stmt::OrderByClause>,
    skip: Option<usize>,
    limit: Option<usize>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl WithValidator {
    /// 创建新的 With 验证器
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            where_clause: None,
            distinct: false,
            order_by: None,
            skip: None,
            limit: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    /// 验证返回项
    fn validate_return_item(&self, item: &ReturnItem) -> Result<ColumnDef, ValidationError> {
        match item {
            ReturnItem::All => {
                // WITH * 传递所有可用变量
                Ok(ColumnDef {
                    name: "*".to_string(),
                    type_: ValueType::Map,
                })
            }
            ReturnItem::Expression { expression, alias } => {
                // 验证表达式
                self.validate_expression(expression)?;

                // 确定列名
                let name = alias.clone()
                    .or_else(|| self.infer_column_name(expression))
                    .unwrap_or_else(|| "column".to_string());

                // 推断类型
                let type_ = self.infer_expression_type(expression);

                Ok(ColumnDef { name, type_ })
            }
        }
    }

    /// 验证表达式
    fn validate_expression(
        &self,
        expr: &ContextualExpression,
    ) -> Result<(), ValidationError> {
        if let Some(e) = expr.expression() {
            self.validate_expression_internal(&e)
        } else {
            Err(ValidationError::new(
                "表达式无效".to_string(),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    /// 内部方法：验证表达式
    fn validate_expression_internal(
        &self,
        expr: &crate::core::types::expression::Expression,
    ) -> Result<(), ValidationError> {
        use crate::core::types::expression::Expression;

        match expr {
            Expression::Literal(_) => Ok(()),
            Expression::Variable(var) => {
                // 检查变量是否来自输入
                if !self.inputs.iter().any(|c| &c.name == var) && !self.user_defined_vars.iter().any(|v| v == var) {
                    return Err(ValidationError::new(
                        format!("Variable '{}' not available in WITH clause", var),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(())
            }
            Expression::Property { object, property } => {
                self.validate_expression_internal(object)?;
                if property.is_empty() {
                    return Err(ValidationError::new(
                        "Property name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(())
            }
            Expression::Function { name, args } => {
                self.validate_function_call_internal(name, args)
            }
            Expression::Binary { left, right, .. } => {
                self.validate_expression_internal(left)?;
                self.validate_expression_internal(right)
            }
            Expression::Unary { operand, .. } => {
                self.validate_expression_internal(operand)
            }
            _ => Ok(()),
        }
    }

    /// 验证函数调用
    fn validate_function_call(
        &self,
        name: &str,
        args: &[ContextualExpression],
    ) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::new(
                "Function name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for arg in args {
            self.validate_expression(arg)?;
        }

        Ok(())
    }

    /// 内部方法：验证函数调用
    fn validate_function_call_internal(
        &self,
        name: &str,
        args: &[crate::core::types::expression::Expression],
    ) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::new(
                "Function name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for arg in args {
            self.validate_expression_internal(arg)?;
        }

        Ok(())
    }

    /// 验证 WHERE 子句
    fn validate_where_clause(
        &self,
        where_clause: &ContextualExpression,
    ) -> Result<(), ValidationError> {
        self.validate_expression(where_clause)?;

        // WHERE 子句必须是布尔类型或可转换为布尔类型
        if let Some(e) = where_clause.expression() {
            use crate::core::types::expression::Expression;
            match e {
                Expression::Literal(_) |
                Expression::Variable(_) |
                Expression::Binary { .. } |
                Expression::Unary { .. } |
                Expression::Function { .. } => Ok(()),
                _ => Err(ValidationError::new(
                    "WHERE clause must be a boolean expression".to_string(),
                    ValidationErrorType::TypeError,
                )),
            }
        } else {
            Err(ValidationError::new(
                "WHERE 表达式无效".to_string(),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    /// 推断列名
    fn infer_column_name(
        &self,
        expr: &ContextualExpression,
    ) -> Option<String> {
        if let Some(e) = expr.expression() {
            self.infer_column_name_internal(&e)
        } else {
            None
        }
    }

    /// 内部方法：推断列名
    fn infer_column_name_internal(
        &self,
        expr: &crate::core::types::expression::Expression,
    ) -> Option<String> {
        use crate::core::types::expression::Expression;

        match expr {
            Expression::Variable(name) => Some(name.clone()),
            Expression::Property { property, .. } => Some(property.clone()),
            Expression::Function { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    /// 推断表达式类型
    fn infer_expression_type(
        &self,
        expr: &ContextualExpression,
    ) -> ValueType {
        if let Some(e) = expr.expression() {
            self.infer_expression_type_internal(&e)
        } else {
            ValueType::Unknown
        }
    }

    /// 内部方法：推断表达式类型
    fn infer_expression_type_internal(
        &self,
        expr: &crate::core::types::expression::Expression,
    ) -> ValueType {
        use crate::core::types::expression::Expression;
        use crate::core::Value;

        match expr {
            Expression::Literal(value) => match value {
                Value::Null(_) => ValueType::Null,
                Value::Bool(_) => ValueType::Bool,
                Value::Int(_) => ValueType::Int,
                Value::Float(_) => ValueType::Float,
                Value::String(_) => ValueType::String,
                Value::Date(_) => ValueType::Date,
                Value::Time(_) => ValueType::Time,
                Value::DateTime(_) => ValueType::DateTime,
                Value::Vertex(_) => ValueType::Vertex,
                Value::Edge(_) => ValueType::Edge,
                Value::Path(_) => ValueType::Path,
                Value::List(_) => ValueType::List,
                Value::Map(_) => ValueType::Map,
                Value::Set(_) => ValueType::Set,
                _ => ValueType::Unknown,
            },
            _ => ValueType::Unknown,
        }
    }

    /// 验证 ORDER BY 子句
    fn validate_order_by(
        &self,
        order_by: &crate::query::parser::ast::stmt::OrderByClause,
    ) -> Result<(), ValidationError> {
        for item in &order_by.items {
            self.validate_expression(&item.expression)?;
        }
        Ok(())
    }

    /// 验证 SKIP 和 LIMIT
    fn validate_skip_limit(&self, skip: Option<usize>, limit: Option<usize>) -> Result<(), ValidationError> {
        if let Some(s) = skip {
            if s > 1_000_000 {
                return Err(ValidationError::new(
                    format!("SKIP value {} exceeds maximum allowed (1000000)", s),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        if let Some(l) = limit {
            if l > 1_000_000 {
                return Err(ValidationError::new(
                    format!("LIMIT value {} exceeds maximum allowed (1000000)", l),
                    ValidationErrorType::SemanticError,
                ));
            }
        }

        Ok(())
    }

    fn validate_impl(&mut self, stmt: &WithStmt) -> Result<(), ValidationError> {
        // 验证返回项
        if stmt.items.is_empty() {
            return Err(ValidationError::new(
                "WITH clause must have at least one item".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for item in &stmt.items {
            let col = self.validate_return_item(item)?;
            self.outputs.push(col);
        }

        // 验证 WHERE 子句
        if let Some(ref where_clause) = stmt.where_clause {
            self.validate_where_clause(where_clause)?;
        }

        // 验证 ORDER BY
        if let Some(ref order_by) = stmt.order_by {
            self.validate_order_by(order_by)?;
        }

        // 验证 SKIP 和 LIMIT
        self.validate_skip_limit(stmt.skip, stmt.limit)?;

        // 保存信息
        self.items = stmt.items.clone();
        self.where_clause = stmt.where_clause.clone();
        self.distinct = stmt.distinct;
        self.order_by = stmt.order_by.clone();
        self.skip = stmt.skip;
        self.limit = stmt.limit;

        // 更新用户定义变量为输出列
        self.user_defined_vars = self.outputs.iter().map(|c| c.name.clone()).collect();

        Ok(())
    }

    /// 设置输入列（从上游传递的列）
    pub fn set_inputs(&mut self, inputs: Vec<ColumnDef>) {
        // 初始时，用户定义变量来自输入
        self.user_defined_vars = inputs.iter().map(|c| c.name.clone()).collect();
        self.inputs = inputs;
    }
}

impl Default for WithValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for WithValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let with_stmt = match stmt {
            crate::query::parser::ast::Stmt::With(with_stmt) => with_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected WITH statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        self.validate_impl(with_stmt)?;

        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::With
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // WITH 不是全局语句
        false
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::expression::Expression;
    use crate::core::Value;

    #[test]
    fn test_with_validator_new() {
        let validator = WithValidator::new();
        assert_eq!(validator.statement_type(), StatementType::With);
        assert!(!validator.is_global_statement());
    }

    #[test]
    fn test_validate_return_item_all() {
        let validator = WithValidator::new();
        let item = ReturnItem::All;
        let col = validator.validate_return_item(&item).expect("Failed to validate return item");
        assert_eq!(col.name, "*");
        assert_eq!(col.type_, ValueType::Map);
    }

    #[test]
    fn test_validate_where_clause() {
        let validator = WithValidator::new();

        // 有效的 WHERE 子句
        let where_expr = Expression::Literal(Value::Bool(true));
        assert!(validator.validate_where_clause(&where_expr).is_ok());

        // 二元操作符
        let _where_expr = Expression::Binary {
            left: Box::new(Expression::Variable("n".to_string())),
            op: crate::core::types::operators::BinaryOperator::Equal,
            right: Box::new(Expression::Literal(Value::Int(1))),
        };
        // 这会失败，因为变量 n 不在输入中
        // assert!(validator.validate_where_clause(&_where_expr).is_err());
    }

    #[test]
    fn test_validate_skip_limit() {
        let validator = WithValidator::new();
        
        // 有效值
        assert!(validator.validate_skip_limit(Some(10), Some(100)).is_ok());
        
        // 超过最大值
        assert!(validator.validate_skip_limit(Some(2_000_000), None).is_err());
    }
}
