//! FIND PATH 语句验证器
//! 对应 NebulaGraph FindPathValidator.h/.cpp 的功能
//! 验证 FIND PATH 语句的合法性

use super::base_validator::Validator;
use super::ValidationContext;
use crate::core::Expression;
use crate::core::types::EdgeDirection;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum PathPattern {
    AllPaths,
    ShortestPath,
    WeightedShortestPath,
}

#[derive(Debug, Clone)]
pub struct FindPathConfig {
    pub path_pattern: PathPattern,
    pub src_vertices: Vec<Expression>,
    pub dst_vertices: Vec<Expression>,
    pub steps: Option<(i32, Option<i32>)>,
    pub edge_types: Vec<String>,
    pub direction: EdgeDirection,
    pub with_props: bool,
    pub limit: Option<i64>,
    pub yield_columns: Vec<super::structs::YieldColumn>,
    pub weight_expression: Option<String>,
    pub heuristic_expression: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathEdgeDirection {
    Forward,
    Backward,
    Both,
}

pub struct FindPathValidator {
    config: FindPathConfig,
}

impl FindPathValidator {
    pub fn new(_context: ValidationContext) -> Self {
        Self {
            config: FindPathConfig {
                path_pattern: PathPattern::AllPaths,
                src_vertices: Vec::new(),
                dst_vertices: Vec::new(),
                steps: None,
                edge_types: Vec::new(),
                direction: crate::core::types::EdgeDirection::Out,
                with_props: false,
                limit: None,
                yield_columns: Vec::new(),
                weight_expression: None,
                heuristic_expression: None,
            },
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_src_vertices()?;
        self.validate_dst_vertices()?;
        self.validate_steps()?;
        self.validate_edge_types()?;
        self.validate_weight_expression()?;
        self.validate_limit()?;
        self.validate_yields()?;
        Ok(())
    }

    fn validate_src_vertices(&self) -> Result<(), ValidationError> {
        if self.config.src_vertices.is_empty() {
            return Err(ValidationError::new(
                "FIND PATH must specify source vertices".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_dst_vertices(&self) -> Result<(), ValidationError> {
        if self.config.dst_vertices.is_empty() {
            return Err(ValidationError::new(
                "FIND PATH must specify destination vertices".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }
        Ok(())
    }

    fn validate_steps(&self) -> Result<(), ValidationError> {
        if let Some((min, max)) = &self.config.steps {
            if *min < 0 {
                return Err(ValidationError::new(
                    "Steps cannot be negative".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
            if let Some(max_steps) = max {
                if *max_steps < *min {
                    return Err(ValidationError::new(
                        "Maximum steps cannot be less than minimum steps".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_edge_types(&self) -> Result<(), ValidationError> {
        for edge_type in &self.config.edge_types {
            if edge_type.is_empty() {
                return Err(ValidationError::new(
                    "Edge type name cannot be empty".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_weight_expression(&self) -> Result<(), ValidationError> {
        if let Some(ref weight_expr) = self.config.weight_expression {
            // 验证权重表达式格式
            // 支持: "ranking" 或属性名
            let expr_lower = weight_expr.to_lowercase();
            if expr_lower != "ranking" && expr_lower.is_empty() {
                return Err(ValidationError::new(
                    "Weight expression must be 'ranking' or a valid property name".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_limit(&self) -> Result<(), ValidationError> {
        if let Some(limit) = self.config.limit {
            if limit <= 0 {
                return Err(ValidationError::new(
                    "LIMIT must be positive".to_string(),
                    ValidationErrorType::SemanticError,
                ));
            }
        }
        Ok(())
    }

    fn validate_yields(&self) -> Result<(), ValidationError> {
        if self.config.yield_columns.is_empty() && !self.config.with_props {
            return Err(ValidationError::new(
                "FIND PATH must have YIELD clause".to_string(),
                ValidationErrorType::SemanticError,
            ));
        }

        let mut seen_names: HashMap<String, usize> = HashMap::new();
        for col in &self.config.yield_columns {
            let name = col.name();
            let count = seen_names.entry(name.to_string()).or_insert(0);
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

    pub fn set_path_pattern(&mut self, pattern: PathPattern) {
        self.config.path_pattern = pattern;
    }

    pub fn set_src_vertices(&mut self, vertices: Vec<Expression>) {
        self.config.src_vertices = vertices;
    }

    pub fn set_dst_vertices(&mut self, vertices: Vec<Expression>) {
        self.config.dst_vertices = vertices;
    }

    pub fn set_steps(&mut self, min: i32, max: Option<i32>) {
        self.config.steps = Some((min, max));
    }

    pub fn set_edge_types(&mut self, types: Vec<String>) {
        self.config.edge_types = types;
    }

    pub fn set_direction(&mut self, direction: EdgeDirection) {
        self.config.direction = direction;
    }

    pub fn set_with_props(&mut self, with_props: bool) {
        self.config.with_props = with_props;
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.config.limit = Some(limit);
    }

    pub fn add_yield_column(&mut self, col: super::structs::YieldColumn) {
        self.config.yield_columns.push(col);
    }

    pub fn set_weight_expression(&mut self, expression: String) {
        self.config.weight_expression = Some(expression);
    }

    pub fn set_heuristic_expression(&mut self, expression: String) {
        self.config.heuristic_expression = Some(expression);
    }

    pub fn weight_expression(&self) -> Option<&String> {
        self.config.weight_expression.as_ref()
    }

    pub fn heuristic_expression(&self) -> Option<&String> {
        self.config.heuristic_expression.as_ref()
    }
}

impl Validator {
    pub fn validate_find_path(
        &mut self,
        config: FindPathConfig,
    ) -> Result<(), ValidationError> {
        let mut validator = FindPathValidator::new(self.context().clone());
        validator.config = config;
        validator.validate()
    }
}
