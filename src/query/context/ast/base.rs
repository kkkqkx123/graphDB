//! 基础AST上下文定义

// 基础AST上下文
#[derive(Debug, Clone)]
pub struct AstContext {
    statement_type: String,
    
    query_text: String,
    contains_path: bool,
}

impl AstContext {
    pub fn new(statement_type: &str, query_text: &str) -> Self {
        Self {
            statement_type: statement_type.to_string(),
            query_text: query_text.to_string(),
            contains_path: query_text.to_lowercase().contains("path"),
        }
    }

    pub fn statement_type(&self) -> &str {
        &self.statement_type
    }

    pub fn contains_path_query(&self) -> bool {
        self.contains_path
    }
}

impl Default for AstContext {
    fn default() -> Self {
        Self {
            statement_type: "UNKNOWN".to_string(),
            query_text: "".to_string(),
            contains_path: false,
        }
    }
}
