//! GO 语句验证器
//! 对应 NebulaGraph GoValidator.h/.cpp 的功能
//! 验证 GO FROM ... OVER ... WHERE ... YIELD ... 语句

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::{
    DataType, Expression,
};
use crate::core::types::EdgeDirection;
use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的 GO 语句信息
#[derive(Debug, Clone)]
pub struct ValidatedGo {
    pub space_id: u64,
    pub from_source: Option<GoSource>,
    pub over_edges: Vec<OverEdge>,
    pub where_filter: Option<Expression>,
    pub yield_columns: Vec<GoYieldColumn>,
    pub step_range: Option<StepRange>,
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

#[derive(Debug)]
pub struct GoValidator {
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expression_props: ExpressionProps,
    user_defined_vars: Vec<String>,
    validated_result: Option<ValidatedGo>,
    schema_manager: Option<Arc<dyn SchemaManager>>,
}

impl GoValidator {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            expression_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validated_result: None,
            schema_manager: None,
        }
    }

    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    /// 验证 FROM 子句
    fn validate_from_clause(&mut self, from_vertices: &[Expression]) -> Result<GoSource, ValidationError> {
        // 取第一个顶点表达式作为源
        let from_expr = from_vertices.first().ok_or_else(|| ValidationError::new(
            "FROM 子句不能为空".to_string(),
            ValidationErrorType::SemanticError,
        ))?;

        let source_type = match from_expr {
            Expression::Variable(var_name) => {
                if var_name == "$-" {
                    GoSourceType::Expression
                } else {
                    self.user_defined_vars.push(var_name.clone());
                    GoSourceType::Variable
                }
            }
            Expression::Literal(_) => GoSourceType::VertexId,
            Expression::Parameter(_) => GoSourceType::Parameter,
            _ => GoSourceType::Expression,
        };

        Ok(GoSource {
            source_type: source_type.clone(),
            expression: from_expr.clone(),
            is_variable: matches!(source_type, GoSourceType::Variable),
            variable_name: if let Expression::Variable(name) = from_expr {
                Some(name.clone())
            } else {
                None
            },
        })
    }

    /// 验证 OVER 子句
    fn validate_over_clause(&mut self, edge_names: &[String]) -> Result<Vec<OverEdge>, ValidationError> {
        if edge_names.is_empty() {
            return Err(ValidationError::new(
                "OVER 子句必须指定至少一条边".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let mut over_edges = Vec::new();
        for edge_name in edge_names {
            if edge_name.is_empty() {
                return Err(ValidationError::new(
                    "边名称不能为空".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }

            over_edges.push(OverEdge {
                edge_name: edge_name.clone(),
                edge_type: None,
                direction: EdgeDirection::Out,
                props: Vec::new(),
                is_reversible: false,
                is_all: edge_name == "*",
            });
        }

        Ok(over_edges)
    }

    /// 验证 WHERE 子句
    fn validate_where_clause(&mut self, filter: &Option<Expression>) -> Result<Option<Expression>, ValidationError> {
        if let Some(ref expr) = filter {
            self.validate_expression(expr)?;
            
            // WHERE 子句应该返回布尔类型
            // 简化处理：假设表达式有效
            Ok(Some(expr.clone()))
        } else {
            Ok(None)
        }
    }

    /// 验证 YIELD 子句
    fn validate_yield_clause(&mut self, items: &[(Expression, Option<String>)]) -> Result<Vec<GoYieldColumn>, ValidationError> {
        let mut column_names = HashMap::new();
        let mut yield_columns = Vec::new();

        for (i, (expr, alias)) in items.iter().enumerate() {
            self.validate_expression(expr)?;

            let col_alias = alias.clone().unwrap_or_else(|| format!("column_{}", i));
            
            if column_names.contains_key(&col_alias) {
                return Err(ValidationError::new(
                    format!("YIELD 列别名 '{}' 重复出现", col_alias),
                    ValidationErrorType::DuplicateKey,
                ));
            }
            column_names.insert(col_alias.clone(), true);

            yield_columns.push(GoYieldColumn {
                expression: expr.clone(),
                alias: col_alias,
                is_distinct: false,
            });
        }

        Ok(yield_columns)
    }

    /// 验证步数范围
    fn validate_step_range(&mut self, steps: &crate::query::parser::ast::stmt::Steps) -> Result<Option<StepRange>, ValidationError> {
        match steps {
            crate::query::parser::ast::stmt::Steps::Fixed(n) => {
                let n_i32 = *n as i32;
                if n_i32 < 0 {
                    return Err(ValidationError::new(
                        "步数不能为负".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(Some(StepRange {
                    step_from: n_i32,
                    step_to: n_i32,
                }))
            }
            crate::query::parser::ast::stmt::Steps::Range { min, max } => {
                let min_i32 = *min as i32;
                let max_i32 = *max as i32;
                if min_i32 < 0 {
                    return Err(ValidationError::new(
                        "步数范围起始值不能为负".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if max_i32 < min_i32 {
                    return Err(ValidationError::new(
                        "步数范围结束值不能小于起始值".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(Some(StepRange {
                    step_from: min_i32,
                    step_to: max_i32,
                }))
            }
            crate::query::parser::ast::stmt::Steps::Variable(_) => {
                // 变量步数，运行时确定
                Ok(None)
            }
        }
    }

    /// 验证表达式
    fn validate_expression(&mut self, expression: &Expression) -> Result<(), ValidationError> {
        match expression {
            Expression::Literal(_) => Ok(()),
            Expression::Variable(name) => {
                if name != "$-" && !self.user_defined_vars.contains(name) {
                    self.user_defined_vars.push(name.clone());
                }
                Ok(())
            }
            Expression::Property { object, .. } => {
                self.validate_expression(object)
            }
            Expression::Binary { left, right, .. } => {
                self.validate_expression(left)?;
                self.validate_expression(right)
            }
            Expression::Unary { operand, .. } => {
                self.validate_expression(operand)
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.validate_expression(arg)?;
                }
                Ok(())
            }
            Expression::Aggregate { arg, .. } => {
                self.validate_expression(arg)
            }
            Expression::List(items) => {
                for item in items {
                    self.validate_expression(item)?;
                }
                Ok(())
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    self.validate_expression(value)?;
                }
                Ok(())
            }
            Expression::Case { test_expr, conditions, default } => {
                if let Some(test) = test_expr {
                    self.validate_expression(test)?;
                }
                for (cond, result) in conditions {
                    self.validate_expression(cond)?;
                    self.validate_expression(result)?;
                }
                if let Some(def) = default {
                    self.validate_expression(def)?;
                }
                Ok(())
            }
            Expression::TypeCast { expression, .. } => {
                self.validate_expression(expression)
            }
            Expression::Subscript { collection, index } => {
                self.validate_expression(collection)?;
                self.validate_expression(index)
            }
            Expression::Range { collection, start, end } => {
                self.validate_expression(collection)?;
                if let Some(s) = start {
                    self.validate_expression(s)?;
                }
                if let Some(e) = end {
                    self.validate_expression(e)?;
                }
                Ok(())
            }
            Expression::Path(items) => {
                for item in items {
                    self.validate_expression(item)?;
                }
                Ok(())
            }
            Expression::Label(_) => Ok(()),
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
                self.validate_expression(mapping)
            }
            Expression::PathBuild(exprs) => {
                for expr in exprs {
                    self.validate_expression(expr)?;
                }
                Ok(())
            }
            Expression::Parameter(_) => Ok(()),
        }
    }

    /// 构建输出列
    fn build_outputs(&mut self, yield_columns: &[GoYieldColumn]) {
        self.outputs.clear();
        for col in yield_columns {
            self.outputs.push(ColumnDef {
                name: col.alias.clone(),
                type_: ValueType::String,
            });
        }
    }
}

impl Default for GoValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for GoValidator {
    fn validate(
        &mut self,
        stmt: &Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        // 1. 检查是否需要空间
        if !self.is_global_statement() && qctx.space_id().is_none() {
            return Err(ValidationError::new(
                "未选择图空间，请先执行 USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 获取 GO 语句
        let go_stmt = match stmt {
            Stmt::Go(go_stmt) => go_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected GO statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 3. 验证 FROM 子句
        let from_source = self.validate_from_clause(&go_stmt.from.vertices)?;

        // 4. 验证 OVER 子句
        let edge_names: Vec<String> = go_stmt.over.as_ref()
            .map(|over| over.edge_types.clone())
            .unwrap_or_default();
        let over_edges = self.validate_over_clause(&edge_names)?;

        // 5. 验证 WHERE 子句
        let where_filter = self.validate_where_clause(&go_stmt.where_clause)?;

        // 6. 验证 YIELD 子句
        let yield_items: Vec<(Expression, Option<String>)> = go_stmt.yield_clause.as_ref()
            .map(|yield_clause| {
                yield_clause.items.iter()
                    .map(|item| (item.expression.clone(), item.alias.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let yield_columns = self.validate_yield_clause(&yield_items)?;

        // 7. 验证步数范围
        let step_range = self.validate_step_range(&go_stmt.steps)?;

        // 8. 构建输出列
        self.build_outputs(&yield_columns);

        // 9. 获取 space_id
        let space_id = qctx.space_id().unwrap_or(0);

        // 10. 创建验证结果
        let validated = ValidatedGo {
            space_id,
            from_source: Some(from_source),
            over_edges,
            where_filter,
            yield_columns: yield_columns.clone(),
            step_range,
            is_truncate: false,
            truncate_columns: Vec::new(),
        };

        self.validated_result = Some(validated);

        // 11. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Go
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // GO 不是全局语句，需要预先选择空间
        false
    }

    fn expression_props(&self) -> &ExpressionProps {
        &self.expression_props
    }

    fn user_defined_vars(&self) -> &[String] {
        &self.user_defined_vars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::query::parser::ast::stmt::{GoStmt, FromClause, OverClause, Steps};
    use crate::query::parser::ast::Span;
    use crate::api::session::{RequestContext, RequestParams};
    use std::sync::Arc;

    /// 创建测试用的 QueryContext，带有有效的 space_id
    fn create_test_query_context() -> Arc<QueryContext> {
        let request_params = RequestParams::new("TEST".to_string());
        let rctx = Arc::new(RequestContext::new(None, request_params));
        let mut qctx = QueryContext::new();
        qctx.set_rctx(rctx);
        Arc::new(qctx)
    }

    fn create_go_stmt(from_expr: Expression, edge_types: Vec<String>) -> GoStmt {
        GoStmt {
            span: Span::default(),
            steps: Steps::Fixed(1),
            from: FromClause {
                span: Span::default(),
                vertices: vec![from_expr],
            },
            over: Some(OverClause {
                span: Span::default(),
                edge_types,
                direction: crate::core::types::EdgeDirection::Out,
            }),
            where_clause: None,
            yield_clause: None,
        }
    }

    #[test]
    fn test_go_validator_basic() {
        let mut validator = GoValidator::new();
        
        let go_stmt = create_go_stmt(
            Expression::literal("vid1"),
            vec!["friend".to_string()],
        );
        
        let qctx = create_test_query_context();
        let result = validator.validate(&Stmt::Go(go_stmt), qctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_go_validator_empty_edges() {
        let mut validator = GoValidator::new();

        let go_stmt = create_go_stmt(
            Expression::literal("vid1"),
            vec![],
        );

        let qctx = create_test_query_context();
        let result = validator.validate(&Stmt::Go(go_stmt), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("OVER 子句必须指定至少一条边"));
    }

    #[test]
    fn test_go_validator_with_yield() {
        let mut validator = GoValidator::new();

        let mut go_stmt = create_go_stmt(
            Expression::literal("vid1"),
            vec!["friend".to_string()],
        );

        go_stmt.yield_clause = Some(crate::query::parser::ast::stmt::YieldClause {
            span: Span::default(),
            items: vec![
                crate::query::parser::ast::stmt::YieldItem {
                    expression: Expression::Variable("$$".to_string()),
                    alias: Some("dst".to_string()),
                },
            ],
            where_clause: None,
            order_by: None,
            limit: None,
            skip: None,
            sample: None,
        });

        let qctx = create_test_query_context();
        let result = validator.validate(&Stmt::Go(go_stmt), qctx);
        assert!(result.is_ok());
        
        let outputs = validator.outputs();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].name, "dst");
    }

    #[test]
    fn test_go_validator_duplicate_alias() {
        let mut validator = GoValidator::new();

        let mut go_stmt = create_go_stmt(
            Expression::literal("vid1"),
            vec!["friend".to_string()],
        );

        go_stmt.yield_clause = Some(crate::query::parser::ast::stmt::YieldClause {
            span: Span::default(),
            items: vec![
                crate::query::parser::ast::stmt::YieldItem {
                    expression: Expression::Variable("$$".to_string()),
                    alias: Some("same".to_string()),
                },
                crate::query::parser::ast::stmt::YieldItem {
                    expression: Expression::Variable("$^".to_string()),
                    alias: Some("same".to_string()),
                },
            ],
            where_clause: None,
            order_by: None,
            limit: None,
            skip: None,
            sample: None,
        });

        let qctx = create_test_query_context();
        let result = validator.validate(&Stmt::Go(go_stmt), qctx);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("重复出现"));
    }

    #[test]
    fn test_go_validator_trait_interface() {
        let validator = GoValidator::new();
        
        assert_eq!(validator.statement_type(), StatementType::Go);
        assert!(validator.inputs().is_empty());
        assert!(validator.user_defined_vars().is_empty());
    }
}
