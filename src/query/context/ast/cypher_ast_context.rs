//! Cypher AST上下文 - 专门用于Cypher查询的AST上下文

use super::base::AstContext;
use std::collections::HashMap;

/// Cypher查询的AST上下文
///
/// 专门用于处理Cypher查询语言的AST上下文
/// 包含Cypher特有的语法元素和语义信息
#[derive(Debug, Clone)]
pub struct CypherAstContext {
    base: AstContext,
    patterns: Vec<CypherPattern>,             // Cypher模式
    clauses: Vec<CypherClause>,               // Cypher子句
    variables: HashMap<String, VariableInfo>, // 变量信息
    expressions: Vec<CypherExpression>,       // Cypher表达式
    parameters: HashMap<String, String>,      // 查询参数
}

/// Cypher模式定义
#[derive(Debug, Clone)]
pub struct CypherPattern {
    pub pattern_type: String,                // "node", "edge", "path"
    pub variable_name: Option<String>,       // 变量名
    pub labels: Vec<String>,                 // 标签列表
    pub properties: HashMap<String, String>, // 属性
}

/// Cypher子句类型
#[derive(Debug, Clone)]
pub struct CypherClause {
    pub clause_type: String, // "MATCH", "WHERE", "RETURN", etc.
    pub content: String,     // 子句内容
    pub position: usize,     // 在查询中的位置
}

/// Cypher表达式
#[derive(Debug, Clone)]
pub struct CypherExpression {
    pub expression_type: String,       // "property", "function", "literal"
    pub content: String,               // 表达式内容
    pub variable_name: Option<String>, // 关联的变量名
}

/// 变量信息
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub var_type: String,                    // "node", "edge", "path"
    pub labels: Vec<String>,                 // 标签列表
    pub properties: HashMap<String, String>, // 属性
    pub is_optional: bool,                   // 是否可选
}

impl CypherAstContext {
    /// 创建新的Cypher AST上下文
    pub fn new(query_text: &str) -> Self {
        Self {
            base: AstContext::new("CYPHER", query_text),
            patterns: Vec::new(),
            clauses: Vec::new(),
            variables: HashMap::new(),
            expressions: Vec::new(),
            parameters: HashMap::new(),
        }
    }

    /// 添加Cypher模式
    pub fn add_pattern(&mut self, pattern: CypherPattern) {
        self.patterns.push(pattern);
    }

    /// 添加Cypher子句
    pub fn add_clause(&mut self, clause: CypherClause) {
        self.clauses.push(clause);
    }

    /// 添加变量信息
    pub fn add_variable(&mut self, var_name: String, var_info: VariableInfo) {
        self.variables.insert(var_name, var_info);
    }

    /// 添加表达式
    pub fn add_expression(&mut self, expression: CypherExpression) {
        self.expressions.push(expression);
    }

    /// 添加查询参数
    pub fn add_parameter(&mut self, param_name: String, param_value: String) {
        self.parameters.insert(param_name, param_value);
    }

    /// 获取模式列表
    pub fn patterns(&self) -> &[CypherPattern] {
        &self.patterns
    }

    /// 获取子句列表
    pub fn clauses(&self) -> &[CypherClause] {
        &self.clauses
    }

    /// 获取变量信息
    pub fn variables(&self) -> &HashMap<String, VariableInfo> {
        &self.variables
    }

    /// 获取表达式列表
    pub fn expressions(&self) -> &[CypherExpression] {
        &self.expressions
    }

    /// 获取参数列表
    pub fn parameters(&self) -> &HashMap<String, String> {
        &self.parameters
    }

    /// 获取基础AST上下文
    pub fn base_context(&self) -> &AstContext {
        &self.base
    }

    /// 检查是否包含特定类型的子句
    pub fn has_clause_type(&self, clause_type: &str) -> bool {
        self.clauses.iter().any(|c| c.clause_type == clause_type)
    }

    /// 获取特定类型的子句
    pub fn get_clauses_by_type(&self, clause_type: &str) -> Vec<&CypherClause> {
        self.clauses
            .iter()
            .filter(|c| c.clause_type == clause_type)
            .collect()
    }

    /// 获取特定标签的模式
    pub fn get_patterns_by_label(&self, label: &str) -> Vec<&CypherPattern> {
        self.patterns
            .iter()
            .filter(|p| p.labels.contains(&label.to_string()))
            .collect()
    }
}

impl Default for CypherAstContext {
    fn default() -> Self {
        Self {
            base: AstContext::default(),
            patterns: Vec::new(),
            clauses: Vec::new(),
            variables: HashMap::new(),
            expressions: Vec::new(),
            parameters: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cypher_ast_context_creation() {
        let query = "MATCH (n:Person) RETURN n.name";
        let context = CypherAstContext::new(query);

        assert_eq!(context.base_context().statement_type(), "CYPHER");
        assert!(context.patterns().is_empty());
        assert!(context.clauses().is_empty());
    }

    #[test]
    fn test_cypher_ast_context_add_pattern() {
        let mut context = CypherAstContext::new("MATCH (n:Person)");

        let pattern = CypherPattern {
            pattern_type: "node".to_string(),
            variable_name: Some("n".to_string()),
            labels: vec!["Person".to_string()],
            properties: HashMap::new(),
        };

        context.add_pattern(pattern);
        assert_eq!(context.patterns().len(), 1);
    }

    #[test]
    fn test_cypher_ast_context_add_clause() {
        let mut context = CypherAstContext::new("MATCH (n) RETURN n");

        let clause = CypherClause {
            clause_type: "MATCH".to_string(),
            content: "(n)".to_string(),
            position: 0,
        };

        context.add_clause(clause);
        assert_eq!(context.clauses().len(), 1);
        assert!(context.has_clause_type("MATCH"));
    }

    #[test]
    fn test_cypher_ast_context_add_variable() {
        let mut context = CypherAstContext::new("MATCH (n:Person)");

        let var_info = VariableInfo {
            var_type: "node".to_string(),
            labels: vec!["Person".to_string()],
            properties: HashMap::new(),
            is_optional: false,
        };

        context.add_variable("n".to_string(), var_info);
        assert!(context.variables().contains_key("n"));
    }

    #[test]
    fn test_cypher_ast_context_get_clauses_by_type() {
        let mut context = CypherAstContext::new("MATCH (n) WHERE n.age > 30 RETURN n");

        context.add_clause(CypherClause {
            clause_type: "MATCH".to_string(),
            content: "(n)".to_string(),
            position: 0,
        });

        context.add_clause(CypherClause {
            clause_type: "WHERE".to_string(),
            content: "n.age > 30".to_string(),
            position: 1,
        });

        context.add_clause(CypherClause {
            clause_type: "RETURN".to_string(),
            content: "n".to_string(),
            position: 2,
        });

        let match_clauses = context.get_clauses_by_type("MATCH");
        assert_eq!(match_clauses.len(), 1);

        let where_clauses = context.get_clauses_by_type("WHERE");
        assert_eq!(where_clauses.len(), 1);
    }
}
