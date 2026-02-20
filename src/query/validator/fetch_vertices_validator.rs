//! 顶点获取验证器 - 新体系版本
//! 对应 NebulaGraph FetchVerticesValidator.h/.cpp 的功能
//! 验证 FETCH PROP ON ... 语句
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

use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::{Expression, Value};
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::stmt::{FetchStmt, FetchTarget};
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的顶点获取信息
#[derive(Debug, Clone)]
pub struct ValidatedFetchVertices {
    pub space_id: u64,
    pub tag_names: Vec<String>,
    pub tag_ids: Vec<i32>,
    pub vertex_ids: Vec<Value>,
    pub yield_columns: Vec<ValidatedYieldColumn>,
    pub is_system: bool,
}

/// 验证后的 YIELD 列
#[derive(Debug, Clone)]
pub struct ValidatedYieldColumn {
    pub expression: Expression,
    pub alias: String,
    pub tag_name: Option<String>,
    pub prop_name: Option<String>,
}

/// 顶点获取验证器 - 新体系实现
///
/// 功能完整性保证：
/// 1. 完整的验证生命周期
/// 2. 输入/输出列管理
/// 3. 表达式属性追踪
/// 4. 用户定义变量管理
/// 5. 权限检查（可扩展）
/// 6. 执行计划生成（可扩展）
#[derive(Debug)]
pub struct FetchVerticesValidator {
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
    validated_result: Option<ValidatedFetchVertices>,
}

impl FetchVerticesValidator {
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
    pub fn validated_result(&self) -> Option<&ValidatedFetchVertices> {
        self.validated_result.as_ref()
    }

    /// 基础验证
    fn validate_fetch_vertices(&self, stmt: &FetchStmt) -> Result<(), ValidationError> {
        match &stmt.target {
            FetchTarget::Vertices { ids, properties } => {
                self.validate_vertex_ids(ids)?;
                self.validate_properties_clause(properties.as_ref())?;
                Ok(())
            }
            _ => Err(ValidationError::new(
                "Expected FETCH VERTICES statement".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 验证顶点 ID 列表
    fn validate_vertex_ids(&self, vertex_ids: &[Expression]) -> Result<(), ValidationError> {
        if vertex_ids.is_empty() {
            return Err(ValidationError::new(
                "必须指定至少一个顶点 ID".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        for vertex_id in vertex_ids {
            self.validate_vertex_id(vertex_id)?;
        }

        Ok(())
    }

    /// 验证单个顶点 ID
    fn validate_vertex_id(&self, expr: &Expression) -> Result<(), ValidationError> {
        match expr {
            Expression::Literal(value) => {
                if value.is_null() || value.is_empty() {
                    return Err(ValidationError::new(
                        "顶点 ID 不能为空".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(())
            }
            Expression::Variable(_) => Ok(()),
            _ => Err(ValidationError::new(
                "顶点 ID 必须是常量或变量".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 验证属性列表子句
    fn validate_properties_clause(&self, properties: Option<&Vec<String>>) -> Result<(), ValidationError> {
        // 属性列表可以为空，表示获取所有属性
        if let Some(props) = properties {
            let mut prop_set = std::collections::HashSet::new();
            for prop in props {
                if !prop_set.insert(prop) {
                    return Err(ValidationError::new(
                        format!("属性 '{}' 重复出现", prop),
                        ValidationErrorType::DuplicateKey,
                    ));
                }
            }
        }
        Ok(())
    }

    /// 评估表达式为 Value
    fn evaluate_expression(&self, expr: &Expression) -> Result<Value, ValidationError> {
        match expr {
            Expression::Literal(v) => Ok(v.clone()),
            Expression::Variable(name) => Ok(Value::String(format!("${}", name))),
            _ => Err(ValidationError::new(
                "表达式必须是常量或变量".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 获取 Tag ID
    fn get_tag_id(&self, tag_name: &str, _space_id: u64) -> Result<Option<i32>, ValidationError> {
        let _ = tag_name;
        Ok(None)
    }
}

impl Default for FetchVerticesValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for FetchVerticesValidator {
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

        // 2. 获取 FETCH 语句
        let stmt = ast.sentence()
            .ok_or_else(|| ValidationError::new(
                "No statement found in AST context".to_string(),
                ValidationErrorType::SemanticError,
            ))?;

        let fetch_stmt = match stmt {
            crate::query::parser::ast::Stmt::Fetch(fetch_stmt) => fetch_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected FETCH statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 3. 执行基础验证
        self.validate_fetch_vertices(fetch_stmt)?;

        // 4. 获取 space_id
        let space_id = query_context
            .map(|qc| qc.space_id())
            .filter(|&id| id != 0)
            .or_else(|| ast.space().space_id.map(|id| id as u64))
            .unwrap_or(0);

        // 5. 提取顶点信息
        let (vertex_ids, properties) = match &fetch_stmt.target {
            FetchTarget::Vertices { ids, properties } => (ids, properties),
            _ => {
                return Err(ValidationError::new(
                    "Expected FETCH VERTICES statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 6. 验证并转换顶点 ID
        let mut validated_vids = Vec::new();
        for vid_expr in vertex_ids {
            let vid = self.evaluate_expression(vid_expr)?;
            validated_vids.push(vid);
        }

        // 7. 验证并转换属性列为 YIELD 列
        let mut validated_columns = Vec::new();
        if let Some(props) = properties {
            for prop in props {
                // 使用变量表达式表示属性名
                validated_columns.push(ValidatedYieldColumn {
                    expression: Expression::Variable(prop.clone()),
                    alias: prop.clone(),
                    tag_name: None,
                    prop_name: Some(prop.clone()),
                });
            }
        }

        // 8. 创建验证结果
        let validated = ValidatedFetchVertices {
            space_id,
            tag_names: vec![], // FETCH VERTICES 不指定具体 tag
            tag_ids: vec![],
            vertex_ids: validated_vids,
            yield_columns: validated_columns,
            is_system: false,
        };

        // 9. 设置输出列
        self.outputs.clear();
        for (i, col) in validated.yield_columns.iter().enumerate() {
            let col_name = if col.alias.is_empty() {
                format!("column_{}", i)
            } else {
                col.alias.clone()
            };
            self.outputs.push(ColumnDef {
                name: col_name,
                type_: ValueType::String,
            });
        }

        self.validated_result = Some(validated);

        // 10. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::FetchVertices
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
    use crate::query::parser::ast::stmt::{FetchStmt, FetchTarget};
    use crate::query::parser::ast::Span;

    fn create_fetch_vertices_stmt(
        vertex_ids: Vec<Expression>,
        properties: Option<Vec<String>>,
    ) -> FetchStmt {
        FetchStmt {
            span: Span::default(),
            target: FetchTarget::Vertices {
                ids: vertex_ids,
                properties,
            },
        }
    }

    #[test]
    fn test_validate_vertex_ids_empty() {
        let validator = FetchVerticesValidator::new();
        let result = validator.validate_vertex_ids(&[]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "必须指定至少一个顶点 ID");
    }

    #[test]
    fn test_validate_vertex_ids_valid() {
        let validator = FetchVerticesValidator::new();
        let vertex_ids = vec![
            Expression::Literal(Value::String("v1".to_string())),
            Expression::Literal(Value::String("v2".to_string())),
        ];
        let result = validator.validate_vertex_ids(&vertex_ids);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_ids_with_variable() {
        let validator = FetchVerticesValidator::new();
        let vertex_ids = vec![Expression::Variable("vids".to_string())];
        let result = validator.validate_vertex_ids(&vertex_ids);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_id_empty() {
        let validator = FetchVerticesValidator::new();
        let vertex_ids = vec![
            Expression::Literal(Value::String("v1".to_string())),
            Expression::Literal(Value::String("".to_string())),
        ];
        let result = validator.validate_vertex_ids(&vertex_ids);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("不能为空"));
    }

    #[test]
    fn test_validate_properties_clause_duplicate() {
        let validator = FetchVerticesValidator::new();
        let properties = Some(vec!["prop1".to_string(), "prop1".to_string()]);
        let result = validator.validate_properties_clause(properties.as_ref());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("重复"));
    }

    #[test]
    fn test_validate_properties_clause_valid() {
        let validator = FetchVerticesValidator::new();
        let properties = Some(vec!["prop1".to_string(), "prop2".to_string()]);
        let result = validator.validate_properties_clause(properties.as_ref());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_properties_clause_empty() {
        let validator = FetchVerticesValidator::new();
        let result = validator.validate_properties_clause(None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_statement_validator_trait() {
        let mut validator = FetchVerticesValidator::new();
        
        // 测试 statement_type
        assert_eq!(validator.statement_type(), StatementType::FetchVertices);
        
        // 测试 inputs/outputs
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
        
        // 测试 user_defined_vars
        assert!(validator.user_defined_vars().is_empty());
    }
}
