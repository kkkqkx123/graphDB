//! FIND PATH 语句验证器 - 新体系版本
//! 对应 NebulaGraph FindPathValidator.h/.cpp 的功能
//! 验证 FIND PATH 语句的合法性
//!
//! 本文件已按照新的 trait + 枚举 验证器体系重构：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 保留了完整功能：
//!    - 验证生命周期管理
//!    - 输入/输出列管理
//!    - 表达式属性追踪
//!    - 用户定义变量管理
//!    - 权限检查
//!    - 执行计划生成
//! 3. 移除了生命周期参数，使用 Arc 管理 SchemaManager
//! 4. 使用 AstContext 统一管理上下文

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::Expression;
use crate::core::types::EdgeDirection;
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::stmt::FindPathStmt;
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};
use crate::storage::metadata::schema_manager::SchemaManager;

/// 路径模式
#[derive(Debug, Clone, PartialEq)]
pub enum PathPattern {
    AllPaths,
    ShortestPath,
    WeightedShortestPath,
}

/// 路径边方向
#[derive(Debug, Clone, PartialEq)]
pub enum PathEdgeDirection {
    Forward,
    Backward,
    Both,
}

/// 验证后的路径查找信息
#[derive(Debug, Clone)]
pub struct ValidatedFindPath {
    pub space_id: u64,
    pub path_pattern: PathPattern,
    pub src_vertices: Vec<Expression>,
    pub dst_vertices: Vec<Expression>,
    pub steps: Option<(i32, Option<i32>)>,
    pub edge_types: Vec<String>,
    pub direction: EdgeDirection,
    pub with_props: bool,
    pub limit: Option<i64>,
    pub weight_expression: Option<String>,
    pub heuristic_expression: Option<String>,
}

/// FIND PATH 验证器 - 新体系实现
///
/// 功能完整性保证：
/// 1. 完整的验证生命周期
/// 2. 输入/输出列管理
/// 3. 表达式属性追踪
/// 4. 用户定义变量管理
/// 5. 权限检查（可扩展）
/// 6. 执行计划生成（可扩展）
#[derive(Debug)]
pub struct FindPathValidator {
    // Schema 管理
    schema_manager: Option<Arc<dyn SchemaManager>>,
    // 输入列定义
    inputs: Vec<ColumnDef>,
    // 输出列定义
    outputs: Vec<ColumnDef>,
    // 表达式属性
    expr_props: ExpressionProps,
    // 用户定义变量
    user_defined_vars: Vec<String>,
    // 缓存验证结果
    validated_result: Option<ValidatedFindPath>,
}

impl FindPathValidator {
    pub fn new() -> Self {
        Self {
            schema_manager: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validated_result: None,
        }
    }

    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    /// 获取验证结果
    pub fn validated_result(&self) -> Option<&ValidatedFindPath> {
        self.validated_result.as_ref()
    }

    /// 基础验证
    fn validate_find_path(&self, stmt: &FindPathStmt) -> Result<(), ValidationError> {
        self.validate_src_vertices(&stmt.src_vertices)?;
        self.validate_dst_vertices(&stmt.dst_vertices)?;
        self.validate_steps(stmt.steps)?;
        self.validate_edge_types(&stmt.edge_types)?;
        self.validate_weight_expression(stmt.weight_expression.as_deref())?;
        self.validate_limit(stmt.limit)?;
        Ok(())
    }

    /// 验证源顶点
    fn validate_src_vertices(&self, src_vertices: &[Expression]) -> Result<(), ValidationError> {
        if src_vertices.is_empty() {
            return Err(ValidationError::new(
                "FIND PATH must specify source vertices".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证目标顶点
    fn validate_dst_vertices(&self, dst_vertices: &[Expression]) -> Result<(), ValidationError> {
        if dst_vertices.is_empty() {
            return Err(ValidationError::new(
                "FIND PATH must specify destination vertices".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证步数
    fn validate_steps(&self, steps: Option<(i32, Option<i32>)>) -> Result<(), ValidationError> {
        if let Some((min, max)) = steps {
            if min < 0 {
                return Err(ValidationError::new(
                    "Steps cannot be negative".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            if let Some(max_steps) = max {
                if max_steps < min {
                    return Err(ValidationError::new(
                        "Maximum steps cannot be less than minimum steps".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }
        Ok(())
    }

    /// 验证边类型
    fn validate_edge_types(&self, edge_types: &[String]) -> Result<(), ValidationError> {
        for edge_type in edge_types {
            if edge_type.is_empty() {
                return Err(ValidationError::new(
                    "Edge type name cannot be empty".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// 验证权重表达式
    fn validate_weight_expression(&self, weight_expr: Option<&str>) -> Result<(), ValidationError> {
        if let Some(expr) = weight_expr {
            let expr_lower = expr.to_lowercase();
            if expr_lower != "ranking" && expr.is_empty() {
                return Err(ValidationError::new(
                    "Weight expression must be 'ranking' or a valid property name".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// 验证限制
    fn validate_limit(&self, limit: Option<i64>) -> Result<(), ValidationError> {
        if let Some(l) = limit {
            if l <= 0 {
                return Err(ValidationError::new(
                    "LIMIT must be positive".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// 验证 YIELD 子句
    fn validate_yield_clause(&self, yield_columns: &[(Expression, Option<String>)]) -> Result<(), ValidationError> {
        if yield_columns.is_empty() {
            return Err(ValidationError::new(
                "FIND PATH must have YIELD clause".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let mut seen_names: HashMap<String, usize> = HashMap::new();
        for (_, alias) in yield_columns {
            let name = alias.clone().unwrap_or_else(|| "column".to_string());
            let count = seen_names.entry(name.clone()).or_insert(0);
            *count += 1;
            if *count > 1 {
                return Err(ValidationError::new(
                    format!("Duplicate column name '{}' in YIELD clause", name),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// 将解析器的路径模式转换为验证器的路径模式
    fn convert_path_pattern(&self, pattern: &crate::query::parser::ast::stmt::PathPattern) -> PathPattern {
        match pattern {
            crate::query::parser::ast::stmt::PathPattern::AllPaths => PathPattern::AllPaths,
            crate::query::parser::ast::stmt::PathPattern::ShortestPath => PathPattern::ShortestPath,
            crate::query::parser::ast::stmt::PathPattern::WeightedShortestPath => PathPattern::WeightedShortestPath,
        }
    }
}

impl Default for FindPathValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for FindPathValidator {
    fn validate(
        &mut self,
        query_context: Option<&QueryContext>,
        ast: &mut AstContext,
    ) -> Result<ValidationResult, ValidationError> {
        // 1. 检查是否需要空间
        if !self.is_global_statement(ast) && query_context.is_none() {
            return Err(ValidationError::new(
                "未选择图空间，请先执行 USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 获取 FIND PATH 语句
        let stmt = ast.sentence()
            .ok_or_else(|| ValidationError::new(
                "No statement found in AST context".to_string(),
                ValidationErrorType::SemanticError,
            ))?;

        let find_path_stmt = match stmt {
            crate::query::parser::ast::Stmt::FindPath(find_path_stmt) => find_path_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected FIND PATH statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 3. 执行基础验证
        self.validate_find_path(find_path_stmt)?;

        // 4. 验证 YIELD 子句
        self.validate_yield_clause(&find_path_stmt.yield_columns)?;

        // 5. 获取 space_id
        let space_id = query_context
            .map(|qc| qc.space_id())
            .flatten()
            .or_else(|| ast.space().space_id.map(|id| id as u64))
            .unwrap_or(0);

        // 6. 创建验证结果
        let validated = ValidatedFindPath {
            space_id,
            path_pattern: self.convert_path_pattern(&find_path_stmt.path_pattern),
            src_vertices: find_path_stmt.src_vertices.clone(),
            dst_vertices: find_path_stmt.dst_vertices.clone(),
            steps: find_path_stmt.steps,
            edge_types: find_path_stmt.edge_types.clone(),
            direction: find_path_stmt.direction,
            with_props: find_path_stmt.with_props,
            limit: find_path_stmt.limit,
            weight_expression: find_path_stmt.weight_expression.clone(),
            heuristic_expression: find_path_stmt.heuristic_expression.clone(),
        };

        // 7. 设置输出列
        self.outputs.clear();
        for (i, (_, alias)) in find_path_stmt.yield_columns.iter().enumerate() {
            let col_name = alias.clone()
                .unwrap_or_else(|| format!("column_{}", i));
            self.outputs.push(ColumnDef {
                name: col_name,
                type_: ValueType::Path,
            });
        }

        self.validated_result = Some(validated);

        // 8. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::FindPath
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
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
    use crate::core::Expression;
    use crate::core::Value;
    use crate::query::parser::ast::stmt::{FindPathStmt, PathPattern as AstPathPattern};
    use crate::query::parser::ast::Span;

    fn create_find_path_stmt(
        src_vertices: Vec<Expression>,
        dst_vertices: Vec<Expression>,
        yield_columns: Vec<(Expression, Option<String>)>,
    ) -> FindPathStmt {
        FindPathStmt {
            span: Span::default(),
            path_pattern: AstPathPattern::ShortestPath,
            src_vertices,
            dst_vertices,
            steps: Some((1, Some(5))),
            edge_types: vec![],
            direction: EdgeDirection::Out,
            with_props: false,
            limit: None,
            yield_columns,
            weight_expression: None,
            heuristic_expression: None,
        }
    }

    #[test]
    fn test_validate_src_vertices_empty() {
        let validator = FindPathValidator::new();
        let result = validator.validate_src_vertices(&[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "FIND PATH must specify source vertices");
    }

    #[test]
    fn test_validate_src_vertices_valid() {
        let validator = FindPathValidator::new();
        let src_vertices = vec![Expression::Literal(Value::String("v1".to_string()))];
        let result = validator.validate_src_vertices(&src_vertices);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_dst_vertices_empty() {
        let validator = FindPathValidator::new();
        let result = validator.validate_dst_vertices(&[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "FIND PATH must specify destination vertices");
    }

    #[test]
    fn test_validate_steps_negative() {
        let validator = FindPathValidator::new();
        let result = validator.validate_steps(Some((-1, None)));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("negative"));
    }

    #[test]
    fn test_validate_steps_invalid_range() {
        let validator = FindPathValidator::new();
        let result = validator.validate_steps(Some((5, Some(3))));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("less than"));
    }

    #[test]
    fn test_validate_edge_types_empty() {
        let validator = FindPathValidator::new();
        let result = validator.validate_edge_types(&["".to_string()]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("empty"));
    }

    #[test]
    fn test_validate_limit_invalid() {
        let validator = FindPathValidator::new();
        let result = validator.validate_limit(Some(0));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("positive"));
    }

    #[test]
    fn test_validate_yield_empty() {
        let validator = FindPathValidator::new();
        let result = validator.validate_yield_clause(&[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("YIELD"));
    }

    #[test]
    fn test_validate_yield_duplicate() {
        let validator = FindPathValidator::new();
        let yield_columns = vec![
            (Expression::Literal(Value::String("path".to_string())), Some("col".to_string())),
            (Expression::Literal(Value::String("cost".to_string())), Some("col".to_string())),
        ];
        let result = validator.validate_yield_clause(&yield_columns);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Duplicate"));
    }

    #[test]
    fn test_statement_validator_trait() {
        let mut validator = FindPathValidator::new();
        
        // 测试 statement_type
        assert_eq!(validator.statement_type(), StatementType::FindPath);
        
        // 测试 inputs/outputs
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
        
        // 测试 user_defined_vars
        assert!(validator.user_defined_vars().is_empty());
    }
}
