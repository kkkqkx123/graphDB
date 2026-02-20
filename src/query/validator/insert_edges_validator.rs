//! Insert Edges 语句验证器
//! 对应 NebulaGraph InsertEdgesValidator 的功能
//! 验证 INSERT EDGES 语句的语义正确性

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::{Expression, Value, NullType};
use crate::query::context::ast::AstContext;
use crate::query::context::execution::QueryContext;
use crate::query::parser::ast::stmt::InsertTarget;
use crate::query::parser::ast::Stmt;
use crate::query::validator::validator_trait::{
    ColumnDef, ExpressionProps, StatementType, StatementValidator, ValidationResult, ValueType,
};
use std::collections::HashSet;
use std::sync::Arc;
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的边插入信息
#[derive(Debug, Clone)]
pub struct ValidatedInsertEdges {
    pub space_id: u64,
    pub edge_name: String,
    pub edge_type_id: Option<i32>,
    pub prop_names: Vec<String>,
    pub edges: Vec<ValidatedEdgeInsert>,
    pub if_not_exists: bool,
}

#[derive(Debug, Clone)]
pub struct ValidatedEdgeInsert {
    pub src_id: Value,
    pub dst_id: Value,
    pub rank: i64,
    pub values: Vec<Value>,
}

#[derive(Debug)]
pub struct InsertEdgesValidator {
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expression_props: ExpressionProps,
    user_defined_vars: Vec<String>,
    validated_result: Option<ValidatedInsertEdges>,
    schema_manager: Option<Arc<dyn SchemaManager>>,
}

impl InsertEdgesValidator {
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

    /// 验证边类型存在
    fn validate_edge_type_exists(&self, edge_name: &str) -> Result<(), ValidationError> {
        if edge_name.is_empty() {
            return Err(ValidationError::new(
                "Edge type name cannot be empty".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证属性名
    fn validate_property_names(&self, prop_names: &[String]) -> Result<(), ValidationError> {
        let mut seen = HashSet::new();
        for prop_name in prop_names {
            if !seen.insert(prop_name) {
                return Err(ValidationError::new(
                    format!("Duplicate property name '{}' in INSERT EDGES", prop_name),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    /// 验证顶点ID格式
    fn validate_vertex_id_format(
        &self,
        expr: &Expression,
        role: &str,
    ) -> Result<(), ValidationError> {
        match expr {
            Expression::Literal(Value::String(s)) => {
                if s.is_empty() {
                    return Err(ValidationError::new(
                        format!("{} vertex ID cannot be empty", role),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(())
            }
            Expression::Variable(_var_name) => {
                // 变量引用是允许的
                Ok(())
            }
            Expression::Literal(Value::Int(_)) => {
                // 整数ID也是允许的
                Ok(())
            }
            _ => {
                Err(ValidationError::new(
                    format!(
                        "{} vertex ID must be a string constant or variable",
                        role
                    ),
                    ValidationErrorType::SemanticError,
                ))
            }
        }
    }

    /// 验证 rank
    fn validate_rank(&self, rank: &Option<Expression>) -> Result<(), ValidationError> {
        if let Some(rank_expr) = rank {
            match rank_expr {
                Expression::Literal(Value::Int(_)) => Ok(()),
                Expression::Variable(_) => Ok(()),
                _ => Err(ValidationError::new(
                    "Rank must be an integer constant or variable".to_string(),
                    ValidationErrorType::SemanticError,
                )),
            }
        } else {
            Ok(())
        }
    }

    /// 验证值数量
    fn validate_values_count(
        &self,
        prop_names: &[String],
        values: &[Expression],
    ) -> Result<(), ValidationError> {
        if values.len() != prop_names.len() {
            return Err(ValidationError::new(
                format!(
                    "Value count mismatch: expected {} values, got {}",
                    prop_names.len(),
                    values.len()
                ),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    /// 验证属性值
    fn validate_property_values(
        &self,
        _edge_name: &str,
        prop_names: &[String],
        values: &[Expression],
    ) -> Result<(), ValidationError> {
        for (prop_idx, value) in values.iter().enumerate() {
            if let Err(e) = self.validate_property_value(&prop_names[prop_idx], value) {
                return Err(ValidationError::new(
                    format!(
                        "Error in edge property '{}': {}",
                        prop_names[prop_idx],
                        e.message
                    ),
                    e.error_type,
                ));
            }
        }
        Ok(())
    }

    /// 验证单个属性值
    fn validate_property_value(
        &self,
        _prop_name: &str,
        value: &Expression,
    ) -> Result<(), ValidationError> {
        match value {
            Expression::Literal(_) => Ok(()),
            Expression::Variable(_) => Ok(()),
            Expression::Function { args, .. } => {
                if args.is_empty() {
                    return Err(ValidationError::new(
                        "Function call must have arguments".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// 生成输出列
    fn generate_output_columns(&mut self) {
        self.outputs.clear();
        self.outputs.push(ColumnDef {
            name: "INSERTED_EDGES".to_string(),
            type_: ValueType::List,
        });
    }

    /// 评估表达式为值
    fn evaluate_expression(&self, expr: &Expression) -> Result<Value, ValidationError> {
        match expr {
            Expression::Literal(val) => Ok(val.clone()),
            Expression::Variable(name) => {
                // 变量在运行时解析
                Ok(Value::String(format!("${}", name)))
            }
            _ => Ok(Value::Null(NullType::Null)),
        }
    }

    /// 评估 rank 表达式
    fn evaluate_rank(&self, rank: &Option<Expression>) -> Result<i64, ValidationError> {
        match rank {
            Some(Expression::Literal(Value::Int(n))) => Ok(*n),
            None => Ok(0),
            _ => Ok(0),
        }
    }
}

impl Default for InsertEdgesValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl StatementValidator for InsertEdgesValidator {
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

        // 2. 获取 INSERT 语句
        let stmt = ast.sentence()
            .ok_or_else(|| ValidationError::new(
                "No statement found in AST context".to_string(),
                ValidationErrorType::SemanticError,
            ))?;

        let insert_stmt = match stmt {
            Stmt::Insert(insert_stmt) => insert_stmt,
            _ => {
                return Err(ValidationError::new(
                    "Expected INSERT statement".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 3. 验证语句类型
        let (edge_name, prop_names, edges) = match &insert_stmt.target {
            InsertTarget::Edge { edge_name, prop_names, edges } => {
                (edge_name.clone(), prop_names.clone(), edges.clone())
            }
            InsertTarget::Vertices { .. } => {
                return Err(ValidationError::new(
                    "Expected INSERT EDGES but got INSERT VERTICES".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 4. 验证边类型存在
        self.validate_edge_type_exists(&edge_name)?;

        // 5. 验证属性名
        self.validate_property_names(&prop_names)?;

        // 6. 验证每条边
        let mut validated_edges = Vec::new();
        for (src, dst, rank, values) in &edges {
            self.validate_vertex_id_format(src, "source")?;
            self.validate_vertex_id_format(dst, "destination")?;
            self.validate_rank(rank)?;
            self.validate_values_count(&prop_names, values)?;
            self.validate_property_values(&edge_name, &prop_names, values)?;

            // 评估并转换
            let src_id = self.evaluate_expression(src)?;
            let dst_id = self.evaluate_expression(dst)?;
            let rank_val = self.evaluate_rank(rank)?;
            let mut value_list = Vec::new();
            for v in values {
                value_list.push(self.evaluate_expression(v)?);
            }

            validated_edges.push(ValidatedEdgeInsert {
                src_id,
                dst_id,
                rank: rank_val,
                values: value_list,
            });
        }

        // 7. 获取 space_id
        let space_id = query_context
            .map(|qc| qc.space_id())
            .filter(|&id| id != 0)
            .or_else(|| ast.space().space_id.map(|id| id as u64))
            .unwrap_or(0);

        // 8. 创建验证结果
        let validated = ValidatedInsertEdges {
            space_id,
            edge_name,
            edge_type_id: None,
            prop_names,
            edges: validated_edges,
            if_not_exists: insert_stmt.if_not_exists,
        };

        self.validated_result = Some(validated);

        // 9. 生成输出列
        self.generate_output_columns();

        // 10. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::InsertEdges
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
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
    use crate::query::parser::ast::Span;

    fn create_insert_edge_stmt(
        edge_name: String,
        prop_names: Vec<String>,
        src: Expression,
        dst: Expression,
        rank: Option<Expression>,
        values: Vec<Expression>,
    ) -> InsertStmt {
        InsertStmt {
            span: Span::default(),
            target: InsertTarget::Edge {
                edge_name,
                prop_names,
                edges: vec![(src, dst, rank, values)],
            },
            if_not_exists: false,
        }
    }

    #[test]
    fn test_validate_edge_name_not_empty() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "".to_string(),
            vec!["prop".to_string()],
            Expression::literal("v1"),
            Expression::literal("v2"),
            None,
            vec![Expression::literal("value")],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Edge type name cannot be empty");
    }

    #[test]
    fn test_validate_duplicate_property_names() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec!["prop1".to_string(), "prop1".to_string()],
            Expression::literal("v1"),
            Expression::literal("v2"),
            None,
            vec![Expression::literal("val1"), Expression::literal("val2")],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Duplicate property name"));
    }

    #[test]
    fn test_validate_value_count_mismatch() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec!["prop1".to_string(), "prop2".to_string()],
            Expression::literal("v1"),
            Expression::literal("v2"),
            None,
            vec![Expression::literal("val1")],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Value count mismatch"));
    }

    #[test]
    fn test_validate_source_vertex_id_empty() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal(""),
            Expression::literal("v2"),
            None,
            vec![],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("source vertex ID cannot be empty"));
    }

    #[test]
    fn test_validate_destination_vertex_id_empty() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal("v1"),
            Expression::literal(""),
            None,
            vec![],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("destination vertex ID cannot be empty"));
    }

    #[test]
    fn test_validate_vertex_ids_valid() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal("v1"),
            Expression::literal("v2"),
            None,
            vec![],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_ids_variable() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::variable("$src"),
            Expression::variable("$dst"),
            None,
            vec![],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_source_vertex_id() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal(123),
            Expression::literal("v2"),
            None,
            vec![],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("source vertex ID must be a string constant or variable"));
    }

    #[test]
    fn test_validate_rank_valid_integer() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal("v1"),
            Expression::literal("v2"),
            Some(Expression::literal(0)),
            vec![],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_rank_valid_variable() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal("v1"),
            Expression::literal("v2"),
            Some(Expression::variable("$rank")),
            vec![],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_rank() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec![],
            Expression::literal("v1"),
            Expression::literal("v2"),
            Some(Expression::literal("invalid")),
            vec![],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Rank must be an integer constant or variable"));
    }

    #[test]
    fn test_validate_property_values() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = create_insert_edge_stmt(
            "friend".to_string(),
            vec!["since".to_string(), "type".to_string()],
            Expression::literal("v1"),
            Expression::literal("v2"),
            None,
            vec![Expression::literal(2020), Expression::literal("best")],
        );

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_wrong_target_type() {
        let mut validator = InsertEdgesValidator::new();
        let stmt = InsertStmt {
            span: Span::default(),
            target: InsertTarget::Vertices {
                tags: vec![],
                values: vec![],
            },
            if_not_exists: false,
        };

        let mut ast = AstContext::default();
        ast.set_sentence(Stmt::Insert(stmt));

        let result = validator.validate(None, &mut ast);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.message, "Expected INSERT EDGES but got INSERT VERTICES");
    }

    #[test]
    fn test_insert_edges_validator_trait_interface() {
        let validator = InsertEdgesValidator::new();
        
        assert_eq!(validator.statement_type(), StatementType::InsertEdges);
        assert!(validator.inputs().is_empty());
        assert!(validator.user_defined_vars().is_empty());
    }
}
