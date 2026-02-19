//! CREATE 数据语句验证器（Cypher 风格）
//! 对应 Cypher CREATE (n:Label {prop: value}) 语法的验证
//! 支持自动 Schema 推断和创建

use crate::core::error::{ValidationError as CoreValidationError, ValidationErrorType};
use crate::core::types::EdgeDirection;
use crate::core::Value;
use crate::query::parser::ast::stmt::{CreateStmt, CreateTarget};
use crate::query::parser::ast::pattern::{Pattern, NodePattern, EdgePattern, PathPattern, PathElement};
use crate::query::validator::base_validator::Validator;
use crate::storage::metadata::schema_manager::SchemaManager;

/// 验证后的创建信息
#[derive(Debug, Clone)]
pub struct ValidatedCreate {
    pub space_id: i32,
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

/// CREATE 语句验证器
pub struct CreateValidator<'a> {
    base: Validator,
    schema_manager: Option<&'a dyn SchemaManager>,
    auto_create_schema: bool,
}

impl<'a> CreateValidator<'a> {
    pub fn new() -> Self {
        Self {
            base: Validator::new(),
            schema_manager: None,
            auto_create_schema: true,
        }
    }

    pub fn with_schema_manager(mut self, schema_manager: &'a dyn SchemaManager) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    pub fn with_auto_create_schema(mut self, auto_create: bool) -> Self {
        self.auto_create_schema = auto_create;
        self
    }

    /// 验证 CREATE 语句
    pub fn validate(
        &self,
        stmt: &CreateStmt,
        space_name: &str,
    ) -> Result<ValidatedCreate, CoreValidationError> {
        let schema_manager = self.schema_manager.ok_or_else(|| {
            CoreValidationError::new(
                "Schema manager not initialized".to_string(),
                ValidationErrorType::SemanticError,
            )
        })?;

        let space = schema_manager
            .get_space(space_name)
            .map_err(|e| {
                CoreValidationError::new(
                    format!("Failed to get space '{}': {}", space_name, e),
                    ValidationErrorType::SemanticError,
                )
            })?
            .ok_or_else(|| {
                CoreValidationError::new(
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
                self.validate_patterns(patterns, space_name, schema_manager, &mut missing_tags, &mut missing_edge_types)?
            }
            CreateTarget::Node { variable, labels, properties } => {
                vec![self.validate_single_node(variable, labels, properties, space_name, schema_manager, &mut missing_tags)?]
            }
            CreateTarget::Edge { variable, edge_type, src, dst, properties, direction } => {
                vec![self.validate_single_edge(variable, edge_type, src, dst, properties, direction, space_name, schema_manager, &mut missing_edge_types)?]
            }
            _ => {
                return Err(CoreValidationError::new(
                    "Unsupported CREATE target type".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        };

        Ok(ValidatedCreate {
            space_id,
            space_name: space_name.to_string(),
            patterns,
            auto_create_schema: self.auto_create_schema,
            missing_tags,
            missing_edge_types,
        })
    }

    /// 验证模式列表
    fn validate_patterns(
        &self,
        patterns: &[Pattern],
        space_name: &str,
        schema_manager: &'a dyn SchemaManager,
        missing_tags: &mut Vec<String>,
        missing_edge_types: &mut Vec<String>,
    ) -> Result<Vec<ValidatedPattern>, CoreValidationError> {
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
                _ => {
                    return Err(CoreValidationError::new(
                        format!("Unsupported pattern type: {:?}", pattern),
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
        schema_manager: &'a dyn SchemaManager,
        missing_tags: &mut Vec<String>,
    ) -> Result<ValidatedNodeCreate, CoreValidationError> {
        // 验证标签是否存在，如果不存在且允许自动创建，则记录需要创建
        for label in &node.labels {
            if let Ok(None) = schema_manager.get_tag(space_name, label) {
                if !self.auto_create_schema {
                    return Err(CoreValidationError::new(
                        format!("Tag '{}' does not exist", label),
                        ValidationErrorType::SemanticError,
                    ));
                }
                if !missing_tags.contains(label) {
                    missing_tags.push(label.clone());
                }
            }
        }

        // 验证属性
        let properties = if let Some(ref props_expr) = node.properties {
            self.extract_properties(props_expr)?
        } else {
            Vec::new()
        };

        Ok(ValidatedNodeCreate {
            variable: node.variable.clone(),
            labels: node.labels.clone(),
            properties,
        })
    }

    /// 验证边模式
    fn validate_edge_pattern(
        &self,
        edge: &EdgePattern,
        space_name: &str,
        schema_manager: &'a dyn SchemaManager,
        missing_edge_types: &mut Vec<String>,
    ) -> Result<ValidatedEdgeCreate, CoreValidationError> {
        // 验证边类型是否存在
        let edge_type = edge.edge_types.first()
            .cloned()
            .unwrap_or_else(|| "UNKNOWN".to_string());

        if let Ok(None) = schema_manager.get_edge_type(space_name, &edge_type) {
            if !self.auto_create_schema {
                return Err(CoreValidationError::new(
                    format!("Edge type '{}' does not exist", edge_type),
                    ValidationErrorType::SemanticError,
                ));
            }
            if !missing_edge_types.contains(&edge_type) {
                missing_edge_types.push(edge_type.clone());
            }
        }

        // 验证属性
        let properties = if let Some(ref props_expr) = edge.properties {
            self.extract_properties(props_expr)?
        } else {
            Vec::new()
        };

        // 对于边创建，需要源节点和目标节点的值
        // 这些值通常在执行时才能确定
        Ok(ValidatedEdgeCreate {
            variable: edge.variable.clone(),
            edge_type,
            src: Value::Null(crate::core::NullType::Null),
            dst: Value::Null(crate::core::NullType::Null),
            properties,
            direction: edge.direction.clone(),
        })
    }

    /// 验证路径模式
    fn validate_path_pattern(
        &self,
        path: &PathPattern,
        space_name: &str,
        schema_manager: &'a dyn SchemaManager,
        missing_tags: &mut Vec<String>,
        missing_edge_types: &mut Vec<String>,
    ) -> Result<ValidatedPathCreate, CoreValidationError> {
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
                _ => {
                    return Err(CoreValidationError::new(
                        "Unsupported path element type".to_string(),
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
        schema_manager: &'a dyn SchemaManager,
        missing_tags: &mut Vec<String>,
    ) -> Result<ValidatedPattern, CoreValidationError> {
        // 验证标签
        for label in labels {
            if let Ok(None) = schema_manager.get_tag(space_name, label) {
                if !self.auto_create_schema {
                    return Err(CoreValidationError::new(
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
        schema_manager: &'a dyn SchemaManager,
        missing_edge_types: &mut Vec<String>,
    ) -> Result<ValidatedPattern, CoreValidationError> {
        // 验证边类型
        if let Ok(None) = schema_manager.get_edge_type(space_name, edge_type) {
            if !self.auto_create_schema {
                return Err(CoreValidationError::new(
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
    ) -> Result<Vec<(String, Value)>, CoreValidationError> {
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
            _ => Err(CoreValidationError::new(
                "Expected Map expression for properties".to_string(),
                ValidationErrorType::SemanticError,
            )),
        }
    }

    /// 求值表达式（简化版）
    fn evaluate_expression(
        &self,
        expr: &crate::core::types::expression::Expression,
    ) -> Result<Value, CoreValidationError> {
        use crate::core::types::expression::Expression;

        match expr {
            Expression::Literal(value) => Ok(value.clone()),
            _ => Err(CoreValidationError::new(
                format!("Unsupported expression type in CREATE: {:?}", expr),
                ValidationErrorType::SemanticError,
            )),
        }
    }
}

impl<'a> Default for CreateValidator<'a> {
    fn default() -> Self {
        Self::new()
    }
}
