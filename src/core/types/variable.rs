//! 变量相关基础类型

/// 变量信息
///
/// 统一变量信息结构，用于存储查询中的变量元数据
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub variable_name: String,
    pub variable_type: String,
    pub source_clause: String,
    pub is_aggregated: bool,
    pub properties: Vec<String>,
}

impl VariableInfo {
    pub fn new(variable_name: String, variable_type: String) -> Self {
        Self {
            variable_name,
            variable_type,
            source_clause: String::new(),
            is_aggregated: false,
            properties: Vec::new(),
        }
    }

    pub fn with_source_clause(mut self, source_clause: String) -> Self {
        self.source_clause = source_clause;
        self
    }

    pub fn with_properties(mut self, properties: Vec<String>) -> Self {
        self.properties = properties;
        self
    }

    pub fn with_aggregated(mut self, is_aggregated: bool) -> Self {
        self.is_aggregated = is_aggregated;
        self
    }
}

/// 起始顶点类型 - 强类型枚举替代String
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FromType {
    /// 瞬时表达式
    InstantExpression,
    /// 变量引用
    Variable,
    /// 管道输入
    Pipe,
}

impl Default for FromType {
    fn default() -> Self {
        FromType::InstantExpression
    }
}

impl From<FromType> for String {
    fn from(t: FromType) -> Self {
        match t {
            FromType::InstantExpression => "instant_expression".to_string(),
            FromType::Variable => "variable".to_string(),
            FromType::Pipe => "pipe".to_string(),
        }
    }
}

impl From<&str> for FromType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "instant_expression" => FromType::InstantExpression,
            "variable" => FromType::Variable,
            "pipe" => FromType::Pipe,
            _ => FromType::InstantExpression,
        }
    }
}
