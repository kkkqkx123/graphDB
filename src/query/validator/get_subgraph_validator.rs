//! GET SUBGRAPH 语句验证器
//! 对应 NebulaGraph GetSubgraphValidator.h/.cpp 的功能
//! 验证 GET SUBGRAPH 语句的合法性

use super::base_validator::{Validator, ValueType};
use super::ValidationContext;
use crate::core::Expression;
use crate::query::validator::ValidationError;
use crate::query::validator::ValidationErrorType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GetSubgraphConfig {
    pub steps: Option<(i32, Option<i32>)>,
    pub vertex_filters: Vec<Expression>,
    pub edge_filters: Vec<Expression>,
    pub edge_types: Vec<String>,
    pub direction: EdgeDirection,
    pub yield_columns: Vec<super::structs::YieldColumn>,
    pub yield_stats: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EdgeDirection {
    Forward,
    Backward,
    Both,
}

pub struct GetSubgraphValidator {
    base: Validator,
    config: GetSubgraphConfig,
}

impl GetSubgraphValidator {
    pub fn new(context: ValidationContext) -> Self {
        Self {
            base: Validator::with_context(context),
            config: GetSubgraphConfig {
                steps: Some((1, None)),
                vertex_filters: Vec::new(),
                edge_filters: Vec::new(),
                edge_types: Vec::new(),
                direction: EdgeDirection::Both,
                yield_columns: Vec::new(),
                yield_stats: false,
            },
        }
    }

    pub fn validate(&mut self) -> Result<(), ValidationError> {
        self.validate_impl()
    }

    fn validate_impl(&mut self) -> Result<(), ValidationError> {
        self.validate_steps()?;
        self.validate_vertex_filters()?;
        self.validate_edge_filters()?;
        self.validate_edge_types()?;
        self.validate_yields()?;
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
                if *max_steps > 100 {
                    return Err(ValidationError::new(
                        "Maximum steps cannot exceed 100".to_string(),
                        ValidationErrorType::SemanticError,
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_vertex_filters(&self) -> Result<(), ValidationError> {
        for filter in &self.config.vertex_filters {
            self.validate_filter_type(filter)?;
        }
        Ok(())
    }

    fn validate_edge_filters(&self) -> Result<(), ValidationError> {
        for filter in &self.config.edge_filters {
            self.validate_filter_type(filter)?;
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

    fn validate_filter_type(&self, filter: &Expression) -> Result<(), ValidationError> {
        let filter_type = self.base.deduce_expr_type(filter);
        match filter_type {
            ValueType::Bool => Ok(()),
            ValueType::Null | ValueType::Unknown => Ok(()),
            _ => Err(ValidationError::new(
                format!("Filter expression must return bool type, actual returns {:?}", filter_type),
                ValidationErrorType::TypeError,
            )),
        }
    }

    fn validate_yields(&self) -> Result<(), ValidationError> {
        if self.config.yield_columns.is_empty() && !self.config.yield_stats {
            return Err(ValidationError::new(
                "GET SUBGRAPH must have YIELD clause".to_string(),
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

    pub fn set_steps(&mut self, min: i32, max: Option<i32>) {
        self.config.steps = Some((min, max));
    }

    pub fn add_vertex_filter(&mut self, filter: Expression) {
        self.config.vertex_filters.push(filter);
    }

    pub fn add_edge_filter(&mut self, filter: Expression) {
        self.config.edge_filters.push(filter);
    }

    pub fn set_edge_types(&mut self, types: Vec<String>) {
        self.config.edge_types = types;
    }

    pub fn set_direction(&mut self, direction: EdgeDirection) {
        self.config.direction = direction;
    }

    pub fn add_yield_column(&mut self, col: super::structs::YieldColumn) {
        self.config.yield_columns.push(col);
    }

    pub fn set_yield_stats(&mut self, yield_stats: bool) {
        self.config.yield_stats = yield_stats;
    }
}

impl Validator {
    pub fn validate_get_subgraph(
        &mut self,
        config: GetSubgraphConfig,
    ) -> Result<(), ValidationError> {
        let mut validator = GetSubgraphValidator::new(self.context().clone());
        validator.config = config;
        validator.validate()
    }
}
