//! GroupBy 语句验证器
//! 对应 NebulaGraph GroupByValidator 的功能
//! 验证 GROUP BY 语句的合法性
//!
//! 设计原则：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 验证分组键和聚合表达式的合法性
//! 3. 支持 HAVING 子句验证

use std::sync::Arc;
use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::types::expression::contextual::ContextualExpression;
use crate::query::QueryContext;
use crate::query::parser::ast::stmt::GroupByStmt;
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};

/// 验证后的 GroupBy 信息
#[derive(Debug, Clone)]
pub struct ValidatedGroupBy {
    pub group_keys: Vec<ContextualExpression>,
    pub group_items: Vec<ContextualExpression>,
    pub output_col_names: Vec<String>,
    pub need_gen_project: bool,
}

/// GroupBy 验证器
#[derive(Debug)]
pub struct GroupByValidator {
    group_keys: Vec<ContextualExpression>,
    group_items: Vec<ContextualExpression>,
    agg_output_col_names: Vec<String>,
    need_gen_project: bool,
    #[allow(dead_code)]
    proj_cols: Vec<ContextualExpression>,
    yield_cols: Vec<ContextualExpression>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl GroupByValidator {
    pub fn new() -> Self {
        Self {
            group_keys: Vec::new(),
            group_items: Vec::new(),
            agg_output_col_names: Vec::new(),
            need_gen_project: false,
            proj_cols: Vec::new(),
            yield_cols: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    fn validate_impl(&mut self, stmt: &GroupByStmt) -> Result<(), ValidationError> {
        // 验证分组键
        self.validate_group_keys(&stmt.group_items)?;
        
        // 验证 YIELD 子句
        self.validate_yield(&stmt.yield_clause)?;
        
        // 语义检查
        self.group_clause_semantic_check()?;
        
        // 验证 HAVING 子句
        if let Some(ref having) = stmt.having_clause {
            self.validate_having(having)?;
        }

        self.setup_outputs();
        Ok(())
    }

    fn validate_group_keys(&mut self, group_items: &[ContextualExpression]) -> Result<(), ValidationError> {
        if group_items.is_empty() {
            return Err(ValidationError::new(
                "GROUP BY clause must have at least one key".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for item in group_items {
            // 验证分组键必须是有效的表达式
            self.validate_group_key(item)?;
            self.group_keys.push(item.clone());
        }

        Ok(())
    }

    fn validate_group_key(&self, expr: &ContextualExpression) -> Result<(), ValidationError> {
        if let Some(e) = expr.expression() {
            self.validate_group_key_internal(&e)
        } else {
            Err(ValidationError::new(
                "分组键表达式无效".to_string(),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    /// 内部方法：验证分组键
    fn validate_group_key_internal(&self, expr: &crate::core::types::expression::Expression) -> Result<(), ValidationError> {
        // 分组键可以是：
        // 1. 列引用
        // 2. 属性访问
        // 3. 简单的表达式
        match expr {
            Expression::Variable(_) | Expression::Property { .. } => Ok(()),
            Expression::Function { name, .. } => {
                // 分组键中不允许聚合函数
                if Self::is_aggregate_function(name) {
                    Err(ValidationError::new(
                        format!("Aggregate function {} cannot be used in GROUP BY key", name),
                        ValidationErrorType::SemanticError,
                    ))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    fn validate_yield(&mut self, yield_clause: &crate::query::parser::ast::stmt::YieldClause) -> Result<(), ValidationError> {
        for item in &yield_clause.items {
            let expr = &item.expression;
            
            // 检查表达式中的聚合函数
            if Self::contains_aggregate(expr) {
                self.agg_output_col_names.push(
                    item.alias.clone()
                        .unwrap_or_else(|| Self::expr_to_string(expr))
                );
            }
            
            self.group_items.push(expr.clone());
            
            // 保存 yield 列用于语义检查
            self.yield_cols.push(expr.clone());
        }

        Ok(())
    }

    fn validate_having(&self, having: &ContextualExpression) -> Result<(), ValidationError> {
        // HAVING 子句中的表达式必须是有效的布尔表达式
        // 且可以包含聚合函数
        if let Some(e) = having.expression() {
            self.validate_having_expr_internal(&e)
        } else {
            Err(ValidationError::new(
                "HAVING 表达式无效".to_string(),
                ValidationErrorType::SemanticError,
            ))
        }
    }

    /// 内部方法：验证 HAVING 表达式
    fn validate_having_expr_internal(&self, expr: &crate::core::types::expression::Expression) -> Result<(), ValidationError> {
        match expr {
            Expression::Binary { op, left, right } => {
                self.validate_having_expr_internal(left)?;
                self.validate_having_expr_internal(right)?;
                
                // 验证比较操作符
                match op {
                    crate::core::types::operators::BinaryOperator::Equal |
                    crate::core::types::operators::BinaryOperator::NotEqual |
                    crate::core::types::operators::BinaryOperator::LessThan |
                    crate::core::types::operators::BinaryOperator::GreaterThan |
                    crate::core::types::operators::BinaryOperator::LessThanOrEqual |
                    crate::core::types::operators::BinaryOperator::GreaterThanOrEqual |
                    crate::core::types::operators::BinaryOperator::And |
                    crate::core::types::operators::BinaryOperator::Or => Ok(()),
                    _ => Err(ValidationError::new(
                        format!("Invalid operator in HAVING clause: {:?}", op),
                        ValidationErrorType::SemanticError,
                    )),
                }
            }
            Expression::Unary { op, operand } => {
                self.validate_having_expr_internal(operand)?;
                match op {
                    crate::core::types::operators::UnaryOperator::Not => Ok(()),
                    _ => Err(ValidationError::new(
                        format!("Invalid unary operator in HAVING clause: {:?}", op),
                        ValidationErrorType::SemanticError,
                    ))
                }
            }
            _ => Ok(()),
        }
    }

    fn group_clause_semantic_check(&self) -> Result<(), ValidationError> {
        // 检查 YIELD 中的非聚合表达式是否都在 GROUP BY 中
        for yield_col in &self.yield_cols {
            if !Self::contains_aggregate(yield_col) {
                // 非聚合表达式必须在 GROUP BY 中
                let found = self.group_keys.iter().any(|key| {
                    Self::expr_equivalent(key, yield_col)
                });
                
                if !found {
                    return Err(ValidationError::new(
                        format!(
                            "Expression '{}' must appear in GROUP BY clause or be used in an aggregate function",
                            Self::expr_to_string(yield_col)
                        ),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        Ok(())
    }

    fn setup_outputs(&mut self) {
        // 根据 YIELD 子句设置输出列
        self.outputs = self.group_items.iter()
            .zip(&self.agg_output_col_names)
            .map(|(_, name)| ColumnDef {
                name: name.clone(),
                type_: ValueType::Unknown,
            })
            .collect();
    }

    fn is_aggregate_function(name: &str) -> bool {
        matches!(
            name.to_uppercase().as_str(),
            "COUNT" | "SUM" | "AVG" | "MAX" | "MIN" | "COLLECT" | "STDDEV"
        )
    }

    fn contains_aggregate(expr: &ContextualExpression) -> bool {
        if let Some(e) = expr.expression() {
            Self::contains_aggregate_internal(&e)
        } else {
            false
        }
    }

    /// 内部方法：检查表达式是否包含聚合函数
    fn contains_aggregate_internal(expr: &crate::core::types::expression::Expression) -> bool {
        match expr {
            Expression::Function { name, .. } => {
                if Self::is_aggregate_function(name) {
                    return true;
                }
                false
            }
            Expression::Binary { left, right, .. } => {
                Self::contains_aggregate_internal(left) || Self::contains_aggregate_internal(right)
            }
            Expression::Unary { operand, .. } => {
                Self::contains_aggregate_internal(operand)
            }
            _ => false,
        }
    }

    fn expr_equivalent(a: &ContextualExpression, b: &ContextualExpression) -> bool {
        // 简化实现：比较字符串表示
        Self::expr_to_string(a) == Self::expr_to_string(b)
    }

    fn expr_to_string(expr: &ContextualExpression) -> String {
        if let Some(e) = expr.expression() {
            format!("{:?}", e)
        } else {
            "InvalidExpression".to_string()
        }
    }

    pub fn validated_result(&self) -> ValidatedGroupBy {
        ValidatedGroupBy {
            group_keys: self.group_keys.clone(),
            group_items: self.group_items.clone(),
            output_col_names: self.agg_output_col_names.clone(),
            need_gen_project: self.need_gen_project,
        }
    }

    pub fn group_keys(&self) -> &[ContextualExpression] {
        &self.group_keys
    }

    pub fn group_items(&self) -> &[ContextualExpression] {
        &self.group_items
    }

    pub fn need_gen_project(&self) -> bool {
        self.need_gen_project
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for GroupByValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let group_by_stmt = match stmt {
            crate::query::parser::ast::Stmt::GroupBy(group_by_stmt) => group_by_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected GROUP BY statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };
        
        self.validate_impl(group_by_stmt)?;
        
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::GroupBy
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        false
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expr_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

impl Default for GroupByValidator {
    fn default() -> Self {
        Self::new()
    }
}
