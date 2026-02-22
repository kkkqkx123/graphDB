//! CREATE 语句验证器（Cypher 风格）- 新体系版本
//! 对应 Cypher CREATE (n:Label {prop: value}) 语法的验证
//! 支持自动 Schema 推断和创建
//! 
//! 本文件已按照新的 trait + 枚举 验证器体系重构：
//! 1. 实现了 StatementValidator trait，统一接口
//! 2. 保留了 base_validator.rs 的完整功能：
//!    - 验证生命周期管理
//!    - 输入/输出列管理
//!    - 表达式属性追踪
//!    - 用户定义变量管理
//!    - 权限检查
//!    - 执行计划生成
//! 3. 移除了生命周期参数，使用 Arc 管理 SchemaManager
//! 4. 使用 QueryContext 统一管理上下文

use std::sync::Arc;

use crate::core::error::{ValidationError, ValidationErrorType};
use crate::core::types::EdgeDirection;
use crate::core::Value;
use crate::query::context::QueryContext;
use crate::storage::metadata::schema_manager::SchemaManager;
use crate::query::parser::ast::stmt::{CreateStmt, CreateTarget};
use crate::query::parser::ast::pattern::{Pattern, NodePattern, EdgePattern, PathPattern, PathElement};
use crate::query::validator::validator_trait::{
    StatementType, StatementValidator, ValidationResult, ColumnDef, ValueType,
    ExpressionProps,
};

/// 验证后的创建信息
#[derive(Debug, Clone)]
pub struct ValidatedCreate {
    pub space_id: u64,
    pub space_name: String,
    pub patterns: Vec<ValidatedPattern>,
    pub auto_create_schema: bool,
    pub missing_tags: Vec<String>,
    pub missing_edge_types: Vec<String>,
}

/// 验证后的模式
#[derive(Debug, Clone)]
pub enum ValidatedPattern {
    Node(ValidatedNodeCreate),
    Edge(ValidatedEdgeCreate),
    Path(ValidatedPathCreate),
}

/// 验证后的节点创建
#[derive(Debug, Clone)]
pub struct ValidatedNodeCreate {
    pub variable: Option<String>,
    pub labels: Vec<String>,
    pub properties: Vec<(String, Value)>,
}

/// 验证后的边创建
#[derive(Debug, Clone)]
pub struct ValidatedEdgeCreate {
    pub variable: Option<String>,
    pub edge_type: String,
    pub src: Value,
    pub dst: Value,
    pub properties: Vec<(String, Value)>,
    pub direction: EdgeDirection,
}

/// 验证后的路径创建
#[derive(Debug, Clone)]
pub struct ValidatedPathCreate {
    pub nodes: Vec<ValidatedNodeCreate>,
    pub edges: Vec<ValidatedEdgeCreate>,
}

/// CREATE 语句验证器 - 新体系实现
/// 
/// 功能完整性保证：
/// 1. 完整的验证生命周期（参考 base_validator.rs）
/// 2. 输入/输出列管理
/// 3. 表达式属性追踪
/// 4. 用户定义变量管理
/// 5. 权限检查（可扩展）
/// 6. 执行计划生成（可扩展）
#[derive(Debug)]
pub struct CreateValidator {
    // Schema 管理
    schema_manager: Option<Arc<dyn SchemaManager>>,
    // 是否自动创建 Schema
    auto_create_schema: bool,
    // 输入列定义
    inputs: Vec<ColumnDef>,
    // 输出列定义
    outputs: Vec<ColumnDef>,
    // 表达式属性
    expr_props: ExpressionProps,
    // 用户定义变量
    user_defined_vars: Vec<String>,
    // 缓存验证结果
    validated_result: Option<ValidatedCreate>,
    // 验证错误列表
    validation_errors: Vec<ValidationError>,
    // 是否不需要空间（用于 CREATE SPACE）
    no_space_required: bool,
}

impl CreateValidator {
    /// 创建新的验证器实例
    pub fn new() -> Self {
        Self {
            schema_manager: None,
            auto_create_schema: true,
            inputs: Vec::new(),
            outputs: Vec::new(),
            expr_props: ExpressionProps::default(),
            user_defined_vars: Vec::new(),
            validated_result: None,
            validation_errors: Vec::new(),
            no_space_required: false,
        }
    }

    /// 设置 SchemaManager
    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    /// 设置是否自动创建 Schema
    pub fn with_auto_create_schema(mut self, auto_create: bool) -> Self {
        self.auto_create_schema = auto_create;
        self
    }

    /// 获取验证结果
    pub fn validated_result(&self) -> Option<&ValidatedCreate> {
        self.validated_result.as_ref()
    }

    /// 获取验证错误列表
    pub fn validation_errors(&self) -> &[ValidationError] {
        &self.validation_errors
    }

    /// 添加验证错误
    fn add_error(&mut self, error: ValidationError) {
        self.validation_errors.push(error);
    }

    /// 清空验证错误
    fn clear_errors(&mut self) {
        self.validation_errors.clear();
    }

    /// 检查是否有验证错误
    fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }

    /// 验证 CREATE 语句（传统方式，保持向后兼容）
    pub fn validate_create(
        &mut self,
        stmt: &CreateStmt,
        space_name: &str,
    ) -> Result<ValidatedCreate, ValidationError> {
        let schema_manager = self.schema_manager.as_ref().ok_or_else(|| {
            ValidationError::new(
                "Schema manager not initialized".to_string(),
                ValidationErrorType::SemanticError,
            )
        })?;

        // 获取空间信息
        let space = schema_manager
            .get_space(space_name)
            .map_err(|e| {
                ValidationError::new(
                    format!("Failed to get space '{}': {}", space_name, e),
                    ValidationErrorType::SemanticError,
                )
            })?
            .ok_or_else(|| {
                ValidationError::new(
                    format!("Space '{}' does not exist", space_name),
                    ValidationErrorType::SemanticError,
                )
            })?;

        let space_id = space.space_id;
        let mut missing_tags = Vec::new();
        let mut missing_edge_types = Vec::new();

        // 验证目标
        let patterns = match &stmt.target {
            CreateTarget::Path { patterns } => {
                self.validate_patterns(patterns, space_name, schema_manager.as_ref(), &mut missing_tags, &mut missing_edge_types)?
            }
            CreateTarget::Node { variable, labels, properties } => {
                vec![self.validate_single_node(variable, labels, properties, space_name, schema_manager.as_ref(), &mut missing_tags)?]
            }
            CreateTarget::Edge { variable, edge_type, src, dst, properties, direction } => {
                vec![self.validate_single_edge(variable, edge_type, src, dst, properties, direction, space_name, schema_manager.as_ref(), &mut missing_edge_types)?]
            }
            _ => {
                return Err(ValidationError::new(
                    "Unsupported CREATE target type".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        let result = ValidatedCreate {
            space_id,
            space_name: space_name.to_string(),
            patterns,
            auto_create_schema: self.auto_create_schema,
            missing_tags,
            missing_edge_types,
        };

        // 缓存结果
        self.validated_result = Some(result.clone());

        Ok(result)
    }

    /// 验证模式列表
    fn validate_patterns(
        &self,
        patterns: &[Pattern],
        space_name: &str,
        schema_manager: &(dyn SchemaManager + Send + Sync),
        missing_tags: &mut Vec<String>,
        missing_edge_types: &mut Vec<String>,
    ) -> Result<Vec<ValidatedPattern>, ValidationError> {
        let mut validated = Vec::new();

        for pattern in patterns {
            let validated_pattern = match pattern {
                Pattern::Node(node) => {
                    ValidatedPattern::Node(self.validate_node_pattern(node, space_name, schema_manager, missing_tags)?)
                }
                Pattern::Edge(edge) => {
                    ValidatedPattern::Edge(self.validate_edge_pattern(edge, space_name, schema_manager, missing_edge_types)?)
                }
                Pattern::Path(path) => {
                    ValidatedPattern::Path(self.validate_path_pattern(path, space_name, schema_manager, missing_tags, missing_edge_types)?)
                }
                Pattern::Variable(_) => {
                    return Err(ValidationError::new(
                        "Variable pattern is not supported in CREATE statement".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            };
            validated.push(validated_pattern);
        }

        Ok(validated)
    }

    /// 验证节点模式
    fn validate_node_pattern(
        &self,
        node: &NodePattern,
        space_name: &str,
        schema_manager: &(dyn SchemaManager + Send + Sync),
        missing_tags: &mut Vec<String>,
    ) -> Result<ValidatedNodeCreate, ValidationError> {
        // 验证标签
        for label in &node.labels {
            if let Ok(None) = schema_manager.get_tag(space_name, label) {
                if !self.auto_create_schema {
                    return Err(ValidationError::new(
                        format!("Tag '{}' does not exist", label),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if !missing_tags.contains(label) {
                    missing_tags.push(label.clone());
                }
            }
        }

        // 提取属性
        let props = if let Some(ref props_expr) = node.properties {
            self.extract_properties(props_expr)?
        } else {
            Vec::new()
        };

        Ok(ValidatedNodeCreate {
            variable: node.variable.clone(),
            labels: node.labels.clone(),
            properties: props,
        })
    }

    /// 验证边模式
    fn validate_edge_pattern(
        &self,
        edge: &EdgePattern,
        space_name: &str,
        schema_manager: &(dyn SchemaManager + Send + Sync),
        missing_edge_types: &mut Vec<String>,
    ) -> Result<ValidatedEdgeCreate, ValidationError> {
        // 验证边类型（取第一个边类型）
        let edge_type = edge.edge_types.first()
            .ok_or_else(|| ValidationError::new(
                "Edge must specify at least one edge type".to_string(),
                ValidationErrorType::SemanticError,
            ))?;
        
        if let Ok(None) = schema_manager.get_edge_type(space_name, edge_type) {
            if !self.auto_create_schema {
                return Err(ValidationError::new(
                    format!("Edge type '{}' does not exist", edge_type),
                    ValidationErrorType::SemanticError,
                ));
            }
            if !missing_edge_types.contains(edge_type) {
                missing_edge_types.push(edge_type.clone());
            }
        }

        // 提取属性
        let props = if let Some(ref props_expr) = edge.properties {
            self.extract_properties(props_expr)?
        } else {
            Vec::new()
        };

        Ok(ValidatedEdgeCreate {
            variable: edge.variable.clone(),
            edge_type: edge_type.clone(),
            src: Value::Null(crate::core::NullType::Null),
            dst: Value::Null(crate::core::NullType::Null),
            properties: props,
            direction: edge.direction.clone(),
        })
    }

    /// 验证路径模式
    fn validate_path_pattern(
        &self,
        path: &PathPattern,
        space_name: &str,
        schema_manager: &(dyn SchemaManager + Send + Sync),
        missing_tags: &mut Vec<String>,
        missing_edge_types: &mut Vec<String>,
    ) -> Result<ValidatedPathCreate, ValidationError> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for element in &path.elements {
            match element {
                PathElement::Node(node) => {
                    nodes.push(self.validate_node_pattern(node, space_name, schema_manager, missing_tags)?);
                }
                PathElement::Edge(edge) => {
                    edges.push(self.validate_edge_pattern(edge, space_name, schema_manager, missing_edge_types)?);
                }
                PathElement::Alternative(_) => {
                    return Err(ValidationError::new(
                        "Alternative pattern is not supported in CREATE statement".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                PathElement::Optional(_) => {
                    return Err(ValidationError::new(
                        "Optional pattern is not supported in CREATE statement".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
                PathElement::Repeated(_, _) => {
                    return Err(ValidationError::new(
                        "Repeated pattern is not supported in CREATE statement".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }

        Ok(ValidatedPathCreate { nodes, edges })
    }

    /// 验证单个节点创建（简化版）
    fn validate_single_node(
        &self,
        variable: &Option<String>,
        labels: &[String],
        properties: &Option<crate::core::types::expression::Expression>,
        space_name: &str,
        schema_manager: &(dyn SchemaManager + Send + Sync),
        missing_tags: &mut Vec<String>,
    ) -> Result<ValidatedPattern, ValidationError> {
        // 验证标签
        for label in labels {
            if let Ok(None) = schema_manager.get_tag(space_name, label) {
                if !self.auto_create_schema {
                    return Err(ValidationError::new(
                        format!("Tag '{}' does not exist", label),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if !missing_tags.contains(label) {
                    missing_tags.push(label.clone());
                }
            }
        }

        // 提取属性
        let props = if let Some(ref props_expr) = properties {
            self.extract_properties(props_expr)?
        } else {
            Vec::new()
        };

        Ok(ValidatedPattern::Node(ValidatedNodeCreate {
            variable: variable.clone(),
            labels: labels.to_vec(),
            properties: props,
        }))
    }

    /// 验证单个边创建（简化版）
    fn validate_single_edge(
        &self,
        variable: &Option<String>,
        edge_type: &str,
        _src: &crate::core::types::expression::Expression,
        _dst: &crate::core::types::expression::Expression,
        properties: &Option<crate::core::types::expression::Expression>,
        direction: &EdgeDirection,
        space_name: &str,
        schema_manager: &(dyn SchemaManager + Send + Sync),
        missing_edge_types: &mut Vec<String>,
    ) -> Result<ValidatedPattern, ValidationError> {
        // 验证边类型
        if let Ok(None) = schema_manager.get_edge_type(space_name, edge_type) {
            if !self.auto_create_schema {
                return Err(ValidationError::new(
                    format!("Edge type '{}' does not exist", edge_type),
                    ValidationErrorType::SemanticError,
                ));
            }
            if !missing_edge_types.contains(&edge_type.to_string()) {
                missing_edge_types.push(edge_type.to_string());
            }
        }

        // 提取属性
        let props = if let Some(ref props_expr) = properties {
            self.extract_properties(props_expr)?
        } else {
            Vec::new()
        };

        Ok(ValidatedPattern::Edge(ValidatedEdgeCreate {
            variable: variable.clone(),
            edge_type: edge_type.to_string(),
            src: Value::Null(crate::core::NullType::Null),
            dst: Value::Null(crate::core::NullType::Null),
            properties: props,
            direction: direction.clone(),
        }))
    }

    /// 从表达式中提取属性键值对
    fn extract_properties(
        &self,
        expr: &crate::core::types::expression::Expression,
    ) -> Result<Vec<(String, Value)>, ValidationError> {
        use crate::core::types::expression::Expression;

        match expr {
            Expression::Map(entries) => {
                let mut props = Vec::new();
                for (key, value_expr) in entries {
                    let value = self.evaluate_expression(value_expr)?;
                    props.push((key.clone(), value));
                }
                Ok(props)
            }
            _ => Err(ValidationError::new(
                "Expected Map expression for properties".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 求值表达式（简化版）
    fn evaluate_expression(
        &self,
        expr: &crate::core::types::expression::Expression,
    ) -> Result<Value, ValidationError> {
        use crate::core::types::expression::Expression;

        match expr {
            Expression::Literal(value) => Ok(value.clone()),
            _ => Err(ValidationError::new(
                format!("Unsupported expression type in CREATE: {:?}", expr),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 验证具体语句（参考 base_validator.rs 的 validate_impl）
    fn validate_impl(&mut self, create_stmt: &CreateStmt, space_name: &str) -> Result<(), ValidationError> {
        // 根据 CreateTarget 类型处理
        match &create_stmt.target {
            // CREATE SPACE: 是全局语句，不需要空间
            CreateTarget::Space { .. } => {
                self.no_space_required = true;
                // TODO: 实现 CREATE SPACE 的验证逻辑
                return Err(ValidationError::new(
                    "CREATE SPACE 验证尚未实现".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            // CREATE TAG/EDGE/INDEX: 需要空间，但当前验证器不支持
            CreateTarget::Tag { .. } | CreateTarget::EdgeType { .. } | CreateTarget::Index { .. } => {
                return Err(ValidationError::new(
                    "CreateValidator 不支持 CREATE TAG/EDGE/INDEX，请使用 DDL 验证器".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            // CREATE Node/Edge/Path: 需要空间，执行 DML 验证
            CreateTarget::Node { .. } | CreateTarget::Edge { .. } | CreateTarget::Path { .. } => {
                if space_name.is_empty() {
                    return Err(ValidationError::new(
                        "CREATE 语句需要预先选择图空间，请先执行 USE <space_name>".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }

                // 执行验证
                let result = self.validate_create(create_stmt, space_name)?;

                // 设置输出列 - 根据实际类型设置
                self.outputs.clear();
                for (i, pattern) in result.patterns.iter().enumerate() {
                    let (col_name, col_type) = match pattern {
                        ValidatedPattern::Node(node) => {
                            let name = node.variable.clone()
                                .unwrap_or_else(|| format!("node_{}", i));
                            (name, ValueType::Vertex)
                        }
                        ValidatedPattern::Edge(edge) => {
                            let name = edge.variable.clone()
                                .unwrap_or_else(|| format!("edge_{}", i));
                            (name, ValueType::Edge)
                        }
                        ValidatedPattern::Path(_) => {
                            (format!("path_{}", i), ValueType::Path)
                        }
                    };
                    self.outputs.push(ColumnDef {
                        name: col_name,
                        type_: col_type,
                    });
                }

                // 缓存验证结果
                self.validated_result = Some(result);
            }
        }

        Ok(())
    }
}

impl Default for CreateValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// 实现 StatementValidator trait
/// 
/// 完整实现验证生命周期（参考 base_validator.rs）：
/// 1. 检查是否需要空间（is_global_statement）
/// 2. 执行具体验证逻辑（validate_impl）
/// 3. 权限检查（check_permission）
/// 4. 生成执行计划（to_plan）
/// 5. 同步输入/输出到 AstContext
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for CreateValidator {
    fn validate(
        &mut self,
        stmt: &crate::query::parser::ast::Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        // 清空之前的状态
        self.outputs.clear();
        self.inputs.clear();
        self.expr_props = ExpressionProps::default();
        self.user_defined_vars.clear();
        self.clear_errors();
        self.no_space_required = false;

        // 获取 CREATE 语句
        let create_stmt = match stmt {
            crate::query::parser::ast::Stmt::Create(create_stmt) => create_stmt,
            _ => {
                return Err(ValidationError::new(
                    "预期CREATE语句".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        // 步骤 1: 检查是否需要空间
        let is_global = match &create_stmt.target {
            CreateTarget::Space { .. } => true,
            _ => false,
        };

        if !is_global && qctx.space_id().is_none() {
            return Err(ValidationError::new(
                "未选择图空间。请先执行 `USE <space>` 选择图空间。".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        // 步骤 2: 获取空间名称
        let space_name = qctx.space_name()
            .unwrap_or_default();

        // 步骤 3: 执行具体验证逻辑
        if let Err(e) = self.validate_impl(create_stmt, &space_name) {
            self.add_error(e);
        }

        // 如果有验证错误，返回失败结果
        if self.has_errors() {
            let errors = self.validation_errors.clone();
            return Ok(ValidationResult::failure(errors));
        }

        // 返回成功的验证结果
        Ok(ValidationResult::success(
            self.inputs.clone(),
            self.outputs.clone(),
        ))
    }

    fn statement_type(&self) -> StatementType {
        StatementType::Create
    }

    fn inputs(&self) -> &[ColumnDef] {
        &self.inputs
    }

    fn outputs(&self) -> &[ColumnDef] {
        &self.outputs
    }

    fn is_global_statement(&self) -> bool {
        // CREATE 不是全局语句，需要预先选择空间
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
    use crate::query::validator::validator_trait::StatementValidator;

    #[test]
    fn test_create_validator_new() {
        let validator = CreateValidator::new();
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
        assert!(validator.validated_result().is_none());
        assert!(validator.validation_errors().is_empty());
    }

    #[test]
    fn test_create_validator_default() {
        let validator: CreateValidator = Default::default();
        assert!(validator.inputs().is_empty());
        assert!(validator.outputs().is_empty());
    }

    #[test]
    fn test_statement_validator_trait() {
        let validator = CreateValidator::new();
        
        // 测试 trait 方法
        assert_eq!(validator.statement_type(), StatementType::Create);
        assert_eq!(validator.validator_name(), "CREATEValidator");
    }
}
