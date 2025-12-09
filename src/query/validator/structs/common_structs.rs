//! 通用数据结构

use crate::query::validator::validation_interface::{ValidationContext, ValidationError};
use crate::query::validator::structs::{AliasType, QueryPart};

/// 验证上下文实现
#[derive(Debug, Clone)]
pub struct ValidationContextImpl {
    pub query_parts: Vec<QueryPart>,
    pub errors: Vec<ValidationError>,
    pub aliases: std::collections::HashMap<String, AliasType>,
    pub current_clause: Option<ClauseType>,
}

impl ValidationContextImpl {
    pub fn new() -> Self {
        Self {
            query_parts: Vec::new(),
            errors: Vec::new(),
            aliases: std::collections::HashMap::new(),
            current_clause: None,
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_errors(&self) -> &[ValidationError] {
        &self.errors
    }
}

impl ValidationContext for ValidationContextImpl {
    fn get_query_parts(&self) -> &[QueryPart] {
        &self.query_parts
    }

    fn get_aliases(&self) -> &std::collections::HashMap<String, AliasType> {
        &self.aliases
    }

    fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn get_errors(&self) -> &[ValidationError] {
        &self.errors
    }
}

/// 子句类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ClauseType {
    Match,
    Return,
    With,
    Unwind,
    Where,
    Yield,
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.is_valid = false;
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        if !other.is_valid {
            self.is_valid = false;
        }
    }
}
