//! Update 语句验证器（增强版）
//! 对应 NebulaGraph UpdateValidator 的功能
//! 验证 UPDATE 语句的语义正确性

use std::sync::Arc;

use crate::core::error::{DBResult, ValidationError, ValidationError as CoreValidationError, ValidationErrorType};
use crate::core::{Expression, Value};
use crate::query::QueryContext;
use crate::query::parser::ast::stmt::{SetClause, UpdateStmt, UpdateTarget};
use crate::query::validator::validator_trait::{StatementValidator, StatementType, ValidationResult, ColumnDef, ExpressionProps, ValueType};
use crate::query::validator::schema_validator::SchemaValidator;
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的更新信息
#[derive(Debug, Clone)]
pub struct ValidatedUpdate {
    pub space_id: u64,
    pub target_type: UpdateTargetType,
    pub tag_or_edge_id: Option<i32>,
    pub tag_or_edge_name: Option<String>,
    pub assignments: Vec<ValidatedAssignment>,
    pub where_clause: Option<Expression>,
    pub is_upsert: bool,
    pub yield_columns: Option<Vec<String>>,
}

/// 更新目标类型
#[derive(Debug, Clone)]
pub enum UpdateTargetType {
    Vertex(Value),
    Edge {
        src: Value,
        dst: Value,
        edge_type: String,
        rank: i64,
    },
    Tag(String, Value),
}

/// 验证后的赋值
#[derive(Debug, Clone)]
pub struct ValidatedAssignment {
    pub property: String,
    pub value: Value,
    pub prop_id: Option<i32>,
}

#[derive(Debug)]
pub struct UpdateValidator {
    schema_validator: Option<SchemaValidator>,
    inputs: Vec<ColumnDef>,
    outputs: Vec<ColumnDef>,
    expr_props: ExpressionProps,
    user_defined_vars: Vec<String>,
}

impl UpdateValidator {
    pub fn new() -> Self {
        Self {
            schema_validator: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
        }
    }

    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_validator = Some(SchemaValidator::new(schema_manager));
        self
    }

    /// 验证 UPDATE 语句并返回验证后的信息
    pub fn validate_with_schema(
        &mut self,
        stmt: &UpdateStmt,
        space_name: &str,
    ) -> Result<ValidatedUpdate, CoreValidationError> {
        // 基础验证（不依赖 schema_validator 的可变借用）
        self.validate_update_stmt(stmt)?;

        let schema_validator = self.schema_validator.as_ref().ok_or_else(|| {
            CoreValidationError::new(
                "Schema validator not initialized".to_string(),
                ValidationErrorType::SemanticError,
            )
        })?;

        let space = schema_validator
            .get_schema_manager()
            .get_space(space_name)
            .map_err(|e| {
                CoreValidationError::new(
                    format!("Failed to get space '{}': {}", space_name, e),
                    ValidationErrorType::SemanticError,
                )
            })?
            .ok_or_else(|| {
                CoreValidationError::new(
                    format!("Space '{}' not found", space_name),
                    ValidationErrorType::SemanticError,
                )
            })?;

        // 验证并转换目标
        let target_type = self.validate_and_convert_target_with_schema(
            &stmt.target,
            &space.vid_type,
            schema_validator,
        )?;

        // 根据目标类型获取 Schema 信息
        let (tag_or_edge_id, tag_or_edge_name, schema_props) = match &target_type {
            UpdateTargetType::Tag(tag_name, _) => {
                let tag_info = schema_validator
                    .get_tag(space_name, tag_name)
                    .map_err(|e| {
                        CoreValidationError::new(
                            format!("Failed to get tag '{}': {}", tag_name, e),
                            ValidationErrorType::SemanticError,
                        )
                    })?
                    .ok_or_else(|| {
                        CoreValidationError::new(
                            format!("Tag '{}' not found in space '{}'", tag_name, space_name),
                            ValidationErrorType::SemanticError,
                        )
                    })?;
                (
                    Some(tag_info.tag_id),
                    Some(tag_name.clone()),
                    tag_info.properties,
                )
            }
            UpdateTargetType::Edge { edge_type, .. } => {
                let edge_info = schema_validator
                    .get_edge_type(space_name, edge_type)
                    .map_err(|e| {
                        CoreValidationError::new(
                            format!("Failed to get edge type '{}': {}", edge_type, e),
                            ValidationErrorType::SemanticError,
                        )
                    })?
                    .ok_or_else(|| {
                        CoreValidationError::new(
                            format!(
                                "Edge type '{}' not found in space '{}'",
                                edge_type, space_name
                            ),
                            ValidationErrorType::SemanticError,
                        )
                    })?;
                (
                    Some(edge_info.edge_type_id),
                    Some(edge_type.clone()),
                    edge_info.properties,
                )
            }
            _ => (None, None, vec![]),
        };

        // 验证并转换赋值
        // 对于 Vertex 目标，跳过属性 Schema 验证（因为 Vertex 可能关联多个 Tag）
        let validated_assignments = match &target_type {
            UpdateTargetType::Vertex(_) => {
                // Vertex 更新：仅验证赋值语法，不验证属性是否存在
                self.validate_and_convert_assignments_without_schema(&stmt.set_clause, schema_validator)?
            }
            _ => {
                // Tag 或 Edge 更新：验证属性存在于 Schema 中
                self.validate_and_convert_assignments(
                    &stmt.set_clause,
                    &schema_props,
                    schema_validator,
                )?
            }
        };

        // 提取 YIELD 列名
        let yield_columns = stmt.yield_clause.as_ref().map(|yc| {
            yc.items.iter().map(|item| item.alias.clone().unwrap_or_else(|| format!("{:?}", item.expression))).collect()
        });

        Ok(ValidatedUpdate {
            space_id: space.space_id,
            target_type,
            tag_or_edge_id,
            tag_or_edge_name,
            assignments: validated_assignments,
            where_clause: stmt.where_clause.clone(),
            is_upsert: stmt.is_upsert,
            yield_columns,
        })
    }

    /// 基础验证（不依赖 Schema）
    pub fn validate_update_stmt(&mut self, stmt: &UpdateStmt) -> Result<(), CoreValidationError> {
        self.validate_target(&stmt.target)?;
        self.validate_set_clause(&stmt.set_clause)?;
        self.validate_where_clause(stmt.where_clause.as_ref())?;
        self.validate_assignments(&stmt.set_clause)?;
        Ok(())
    }

    /// 完整验证（包含 AST 上下文）
    pub fn validate_with_ast(
        &mut self,
        stmt: &UpdateStmt,
        qctx: Arc<QueryContext>,
    ) -> DBResult<()> {
        self.validate_space_chosen(qctx)?;
        self.validate_update_stmt(stmt)?;
        self.generate_output_columns();
        Ok(())
    }

    fn validate_target(&self, target: &UpdateTarget) -> Result<(), CoreValidationError> {
        match target {
            UpdateTarget::Vertex(vid_expr) => {
                self.validate_vertex_id(vid_expr, "vertex")?;
            }
            UpdateTarget::Edge { src, dst, edge_type, rank } => {
                self.validate_vertex_id(src, "source")?;
                self.validate_vertex_id(dst, "destination")?;
                if let Some(rank_expr) = rank {
                    self.validate_rank(rank_expr)?;
                }
                if let Some(et) = edge_type {
                    if et.is_empty() {
                        return Err(CoreValidationError::new(
                            "Edge type name cannot be empty".to_string(),
                            ValidationErrorType::SemanticError,
                        ));
                    }
                }
            }
            UpdateTarget::Tag(tag_name) => {
                if tag_name.is_empty() {
                    return Err(CoreValidationError::new(
                        "Tag name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
            UpdateTarget::TagOnVertex { vid, tag_name } => {
                self.validate_vertex_id(vid, "vertex")?;
                if tag_name.is_empty() {
                    return Err(CoreValidationError::new(
                        "Tag name cannot be empty".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }
        Ok(())
    }

    /// 验证并转换目标（使用 Schema）
    fn validate_and_convert_target_with_schema(
        &self,
        target: &UpdateTarget,
        vid_type: &crate::core::types::DataType,
        schema_validator: &SchemaValidator,
    ) -> Result<UpdateTargetType, CoreValidationError> {
        match target {
            UpdateTarget::Vertex(vid_expr) => {
                let vid = self.validate_and_evaluate_vid(vid_expr, vid_type, schema_validator, "vertex")?;
                Ok(UpdateTargetType::Vertex(vid))
            }
            UpdateTarget::Edge { src, dst, edge_type, rank } => {
                let src_vid = self.validate_and_evaluate_vid(src, vid_type, schema_validator, "source")?;
                let dst_vid = self.validate_and_evaluate_vid(dst, vid_type, schema_validator, "destination")?;
                let rank_val = if let Some(rank_expr) = rank {
                    self.evaluate_rank(rank_expr, schema_validator)?
                } else {
                    0
                };
                let et = edge_type.as_ref().ok_or_else(|| {
                    CoreValidationError::new(
                        "Edge type is required for edge update".to_string(),
                        ValidationErrorType::SemanticError,
                    )
                })?;
                Ok(UpdateTargetType::Edge {
                    src: src_vid,
                    dst: dst_vid,
                    edge_type: et.clone(),
                    rank: rank_val,
                })
            }
            UpdateTarget::Tag(_tag_name) => {
                // Tag update requires a vertex ID from the context
                // For now, we return an error as we need the VID from elsewhere
                return Err(CoreValidationError::new(
                    "Tag update requires vertex ID context".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            UpdateTarget::TagOnVertex { vid, tag_name } => {
                let vid_val = self.validate_and_evaluate_vid(vid, vid_type, schema_validator, "vertex")?;
                Ok(UpdateTargetType::Tag(tag_name.clone(), vid_val))
            }
        }
    }

    /// 验证顶点 ID
    /// 优先使用 SchemaValidator 的统一验证方法
    fn validate_vertex_id(&self, expr: &Expression, role: &str) -> Result<(), CoreValidationError> {
        // 如果有 schema_validator，使用统一的验证方法
        if let Some(ref schema_validator) = self.schema_validator {
            // 获取 space 的 vid_type，默认为 String
            let vid_type = crate::core::types::DataType::String;
            return schema_validator.validate_vid_expr(expr, &vid_type, role);
        }
        
        // 没有 schema_validator 时进行基本验证
        Self::basic_validate_vertex_id(expr, role)
    }
    
    /// 基本顶点 ID 验证（无 SchemaValidator 时）
    fn basic_validate_vertex_id(expr: &Expression, role: &str) -> Result<(), CoreValidationError> {
        match expr {
            Expression::Literal(crate::core::Value::String(s)) => {
                if s.is_empty() {
                    return Err(CoreValidationError::new(
                        format!("{} vertex ID cannot be empty", role),
                        ValidationErrorType::SemanticError,
                    ));
                }
                Ok(())
            }
            Expression::Literal(crate::core::Value::Int(_)) => Ok(()),
            Expression::Variable(_) => Ok(()),
            _ => Err(CoreValidationError::new(
                format!("{} vertex ID must be a string constant or variable", role),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 验证并评估 VID
    fn validate_and_evaluate_vid(
        &self,
        vid_expr: &Expression,
        vid_type: &crate::core::types::DataType,
        schema_validator: &SchemaValidator,
        role: &str,
    ) -> Result<Value, CoreValidationError> {
        let vid = schema_validator
            .evaluate_expression(vid_expr)
            .map_err(|e| {
                CoreValidationError::new(
                    format!("Failed to evaluate {} vertex ID: {}", role, e.message),
                    e.error_type,
                )
            })?;

        schema_validator
            .validate_vid(&vid, vid_type)
            .map_err(|e| {
                CoreValidationError::new(
                    format!("Invalid {} vertex ID: {}", role, e.message),
                    e.error_type,
                )
            })?;

        Ok(vid)
    }

    fn validate_rank(&self, expr: &Expression) -> Result<(), CoreValidationError> {
        match expr {
            Expression::Literal(crate::core::Value::Int(_)) => Ok(()),
            Expression::Variable(_) => Ok(()),
            _ => Err(CoreValidationError::new(
                "Rank must be an integer constant or variable".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 评估 rank 表达式
    fn evaluate_rank(
        &self,
        expr: &Expression,
        schema_validator: &SchemaValidator,
    ) -> Result<i64, CoreValidationError> {
        let value = schema_validator
            .evaluate_expression(expr)
            .map_err(|e| {
                CoreValidationError::new(
                    format!("Failed to evaluate rank: {}", e.message),
                    e.error_type,
                )
            })?;

        match value {
            Value::Int(i) => Ok(i),
            _ => Err(CoreValidationError::new(
                "Rank must be an integer".to_string(),
                ValidationErrorType::TypeMismatch,
            )),
        }
    }

    fn validate_set_clause(&self, set_clause: &SetClause) -> Result<(), CoreValidationError> {
        if set_clause.assignments.is_empty() {
            return Err(CoreValidationError::new(
                "UPDATE statement must have at least one SET clause".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_assignments(&self, set_clause: &SetClause) -> Result<(), CoreValidationError> {
        let mut seen = std::collections::HashSet::new();
        for assignment in &set_clause.assignments {
            if !seen.insert(assignment.property.clone()) {
                return Err(CoreValidationError::new(
                    format!("Duplicate property assignment for '{}'", assignment.property),
                    ValidationErrorType::SemanticError,
                ));
            }
            self.validate_property_value(&assignment.value)?;
        }
        Ok(())
    }

    /// 验证并转换赋值
    fn validate_and_convert_assignments(
        &self,
        set_clause: &SetClause,
        schema_props: &[crate::core::types::PropertyDef],
        schema_validator: &SchemaValidator,
    ) -> Result<Vec<ValidatedAssignment>, CoreValidationError> {
        let mut result = Vec::new();

        for assignment in &set_clause.assignments {
            // 检查属性是否存在于 Schema 中
            let prop_def = schema_validator
                .get_property_def(&assignment.property, schema_props)
                .ok_or_else(|| {
                    CoreValidationError::new(
                        format!(
                            "Property '{}' does not exist in schema",
                            assignment.property
                        ),
                        ValidationErrorType::SemanticError,
                    )
                })?;

            // 评估表达式
            let value = schema_validator
                .evaluate_expression(&assignment.value)
                .map_err(|e| {
                    CoreValidationError::new(
                        format!(
                            "Failed to evaluate property '{}': {}",
                            assignment.property, e.message
                        ),
                        e.error_type,
                    )
                })?;

            // 验证类型
            schema_validator
                .validate_property_type(&assignment.property, &prop_def.data_type, &value)
                .map_err(|e| {
                    CoreValidationError::new(
                        format!(
                            "Property '{}': {}",
                            assignment.property, e.message
                        ),
                        e.error_type,
                    )
                })?;

            result.push(ValidatedAssignment {
                property: assignment.property.clone(),
                value,
                prop_id: None, // 可以后续填充
            });
        }

        Ok(result)
    }

    /// 不验证 Schema 的情况下转换赋值（用于 Vertex 更新）
    fn validate_and_convert_assignments_without_schema(
        &self,
        set_clause: &SetClause,
        schema_validator: &SchemaValidator,
    ) -> Result<Vec<ValidatedAssignment>, CoreValidationError> {
        let mut result = Vec::new();

        for assignment in &set_clause.assignments {
            // 评估表达式
            let value = schema_validator
                .evaluate_expression(&assignment.value)
                .map_err(|e| {
                    CoreValidationError::new(
                        format!(
                            "Failed to evaluate property '{}': {}",
                            assignment.property, e.message
                        ),
                        e.error_type,
                    )
                })?;

            result.push(ValidatedAssignment {
                property: assignment.property.clone(),
                value,
                prop_id: None,
            });
        }

        Ok(result)
    }

    fn validate_property_value(&self, value: &Expression) -> Result<(), CoreValidationError> {
        match value {
            Expression::Literal(_) => Ok(()),
            Expression::Variable(_) => Ok(()),
            Expression::Function { args, .. } => {
                if args.is_empty() {
                    return Err(CoreValidationError::new(
                        "Function call must have arguments".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                self.validate_function_args(args)?;
                Ok(())
            }
            Expression::Unary { op: _, operand } => {
                self.validate_property_value(operand)?;
                Ok(())
            }
            Expression::Binary { left, right, .. } => {
                self.validate_property_value(left)?;
                self.validate_property_value(right)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn validate_function_args(&self, args: &[Expression]) -> Result<(), CoreValidationError> {
        for arg in args {
            self.validate_property_value(arg)?;
        }
        Ok(())
    }

    fn validate_where_clause(
        &self,
        where_clause: Option<&Expression>,
    ) -> Result<(), CoreValidationError> {
        if let Some(where_expr) = where_clause {
            self.validate_expression(where_expr)?;
        }
        Ok(())
    }

    fn validate_expression(&self, expr: &Expression) -> Result<(), CoreValidationError> {
        match expr {
            Expression::Literal(_) => Ok(()),
            Expression::Variable(_) => Ok(()),
            Expression::Property { .. } => Ok(()),
            Expression::Function { args, .. } => {
                for arg in args {
                    self.validate_expression(arg)?;
                }
                Ok(())
            }
            Expression::Unary { operand, .. } => self.validate_expression(operand),
            Expression::Binary { left, right, .. } => {
                self.validate_expression(left)?;
                self.validate_expression(right)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn generate_output_columns(&mut self) {
        self.outputs.push(ColumnDef {
            name: "UPDATED".to_string(),
            type_: ValueType::Bool,
        });
    }

    fn validate_space_chosen(&self, qctx: Arc<QueryContext>) -> Result<(), CoreValidationError> {
        if qctx.space_id().is_none() {
            return Err(CoreValidationError::new(
                "No space selected. Use `USE <space>` to select a graph space first.".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }
}

/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for UpdateValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        // 1. 检查是否需要空间
        if !self.is_global_statement() && qctx.space_id().is_none() {
            return Err(ValidationError::new(
                "未选择图空间，请先执行 USE <space>".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 2. 获取 UPDATE 语句
        let update_stmt = match stmt {
            crate::query::parser::ast::Stmt::Update(u) => u,
            _ => {
                return Err(ValidationError::new(
                    "期望 UPDATE 语句".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 3. 验证 UPDATE 语句
        if let Err(e) = self.validate_update_stmt(update_stmt) {
            return Err(ValidationError::new(
                format!("UPDATE 验证失败: {}", e),
                ValidationErrorType::SemanticError,
            ));
        }

        // 4. 生成输出列
        self.generate_output_columns();

        // 5. 返回验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Update
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

impl Default for UpdateValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::core::types::{DataType, PropertyDef, TagInfo};
    use crate::query::parser::ast::stmt::{UpdateTarget, SetClause, Assignment};
    use crate::query::parser::ast::Span;

    fn create_update_stmt(target: UpdateTarget, assignments: Vec<Assignment>, where_clause: Option<Expression>) -> UpdateStmt {
        UpdateStmt {
            span: Span::default(),
            target,
            set_clause: SetClause {
                span: Span::default(),
                assignments,
            },
            where_clause,
            is_upsert: false,
            yield_clause: None,
        }
    }

    // 模拟 SchemaManager 用于测试
    #[derive(Debug)]
    struct MockSchemaManager;

    impl SchemaManager for MockSchemaManager {
        fn create_space(&self, _space: &crate::core::types::SpaceInfo) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn drop_space(&self, _space_name: &str) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn get_space(&self, _space_name: &str) -> crate::storage::StorageResult<Option<crate::core::types::SpaceInfo>> {
            Ok(Some(crate::core::types::SpaceInfo {
                space_id: 1,
                space_name: "test_space".to_string(),
                vid_type: DataType::String,
                tags: vec![],
                edge_types: vec![],
                version: crate::core::types::MetadataVersion {
                    version: 1,
                    timestamp: 0,
                    description: String::new(),
                },
                comment: None,
            }))
        }
        fn get_space_by_id(&self, _space_id: u64) -> crate::storage::StorageResult<Option<crate::core::types::SpaceInfo>> {
            Ok(None)
        }
        fn list_spaces(&self) -> crate::storage::StorageResult<Vec<crate::core::types::SpaceInfo>> {
            Ok(vec![])
        }
        fn create_tag(&self, _space: &str, _tag: &TagInfo) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn get_tag(&self, _space: &str, tag_name: &str) -> crate::storage::StorageResult<Option<TagInfo>> {
            if tag_name == "person" {
                Ok(Some(TagInfo {
                    tag_id: 1,
                    tag_name: "person".to_string(),
                    properties: vec![
                        PropertyDef::new("name".to_string(), DataType::String).with_nullable(false),
                        PropertyDef::new("age".to_string(), DataType::Int).with_nullable(true),
                    ],
                    comment: None,
                    ttl_duration: None,
                    ttl_col: None,
                }))
            } else {
                Ok(None)
            }
        }
        fn list_tags(&self, _space: &str) -> crate::storage::StorageResult<Vec<TagInfo>> {
            Ok(vec![])
        }
        fn drop_tag(&self, _space: &str, _tag_name: &str) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn create_edge_type(&self, _space: &str, _edge: &crate::core::types::EdgeTypeInfo) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn get_edge_type(&self, _space: &str, _edge_type_name: &str) -> crate::storage::StorageResult<Option<crate::core::types::EdgeTypeInfo>> {
            Ok(None)
        }
        fn list_edge_types(&self, _space: &str) -> crate::storage::StorageResult<Vec<crate::core::types::EdgeTypeInfo>> {
            Ok(vec![])
        }
        fn drop_edge_type(&self, _space: &str, _edge_type_name: &str) -> crate::storage::StorageResult<bool> {
            Ok(true)
        }
        fn get_tag_schema(&self, _space: &str, _tag: &str) -> crate::storage::StorageResult<crate::storage::Schema> {
            Ok(crate::storage::Schema::new("test".to_string(), 1))
        }
        fn get_edge_type_schema(&self, _space: &str, _edge: &str) -> crate::storage::StorageResult<crate::storage::Schema> {
            Ok(crate::storage::Schema::new("test".to_string(), 1))
        }
    }

    #[test]
    fn test_validate_vertex_target_valid() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![Assignment {
                property: "name".to_string(),
                value: Expression::literal("new_name"),
            }],
            None,
        );
        let result = validator.validate_update_stmt(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_target_variable() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::variable("$vid")),
            vec![Assignment {
                property: "name".to_string(),
                value: Expression::literal("new_name"),
            }],
            None,
        );
        let result = validator.validate_update_stmt(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_vertex_id_empty() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("")),
            vec![Assignment {
                property: "name".to_string(),
                value: Expression::literal("new_name"),
            }],
            None,
        );
        let result = validator.validate_update_stmt(&stmt);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("vertex ID cannot be empty"));
    }

    #[test]
    fn test_validate_with_schema() {
        let mock = Arc::new(MockSchemaManager);
        let mut validator = UpdateValidator::new().with_schema_manager(mock);

        // 使用 Vertex 目标类型进行测试（Tag 类型需要额外的 VID 上下文）
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![
                Assignment {
                    property: "name".to_string(),
                    value: Expression::literal("new_name"),
                },
            ],
            None,
        );

        let result = validator.validate_with_schema(&stmt, "test_space");
        // Vertex 类型更新不需要 Schema 属性验证，所以应该成功
        assert!(result.is_ok());

        let validated = result.expect("Failed to validate update statement");
        assert_eq!(validated.space_id, 1);
    }

    #[test]
    fn test_validate_duplicate_assignment() {
        let mut validator = UpdateValidator::new();
        let stmt = create_update_stmt(
            UpdateTarget::Vertex(Expression::literal("v1")),
            vec![
                Assignment {
                    property: "name".to_string(),
                    value: Expression::literal("name1"),
                },
                Assignment {
                    property: "name".to_string(),
                    value: Expression::literal("name2"),
                },
            ],
            None,
        );
        let result = validator.validate_update_stmt(&stmt);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Duplicate"));
    }
}
