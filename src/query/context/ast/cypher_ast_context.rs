//! Cypher AST上下文 - 专门用于Cypher查询的AST上下文

use super::base::AstContext;
use super::common::{AliasType, CypherClauseKind, PatternKind};
use std::collections::{HashMap, HashSet};

/// Cypher查询的AST上下文
///
/// 专门用于处理Cypher查询语言的AST上下文
/// 包含Cypher特有的语法元素和语义信息
#[derive(Debug, Clone)]
pub struct CypherAstContext {
    base: AstContext,
    query_parts: Vec<QueryPart>, // 查询分段（MATCH...WITH...MATCH...RETURN）
    variables: HashMap<String, VariableInfo>, // 变量信息
    expressions: Vec<CypherExpression>, // Cypher表达式
    parameters: HashMap<String, String>, // 查询参数
    clause_references: HashMap<String, Vec<String>>, // 子句间的引用关系
}

/// 查询分段 - Cypher查询的逻辑分段
///
/// Cypher查询可以包含多个分段，每个分段以MATCH、UNWIND、WITH等开始
/// 例如：MATCH (a) WITH a MATCH (b) WHERE b.id = a.id RETURN b
/// 包含两个QueryPart：MATCH (a) WITH a 和 MATCH (b) WHERE b.id = a.id RETURN b
#[derive(Debug, Clone)]
pub struct QueryPart {
    pub part_id: String,                   // 分段ID
    pub clauses: Vec<CypherClause>,        // 该分段包含的子句
    pub input_variables: HashSet<String>,  // 输入变量（来自前一个分段）
    pub output_variables: HashSet<String>, // 输出变量（传递给下一个分段）
}

/// Cypher子句 - 专门化的子句结构
#[derive(Debug, Clone)]
pub struct CypherClause {
    pub clause_kind: CypherClauseKind,   // 子句类型（强类型枚举）
    pub content: String,                 // 子句内容
    pub position: usize,                 // 在查询中的位置
    pub referenced_clauses: Vec<String>, // 引用的其他子句ID
}

/// Cypher模式定义 - 专门化的模式结构
#[derive(Debug, Clone)]
pub struct CypherPattern {
    pub pattern_kind: PatternKind,           // 模式类型（强类型枚举）
    pub variable_name: Option<String>,       // 变量名
    pub labels: HashSet<String>,             // 标签列表（使用HashSet保证唯一性）
    pub properties: HashMap<String, String>, // 属性
    pub alias_type: Option<AliasType>,       // 别名类型（强类型枚举）
}

/// 节点信息 - 专门化的节点结构
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub variable_name: String,               // 变量名
    pub labels: HashSet<String>,             // 标签列表
    pub properties: HashMap<String, String>, // 属性
    pub is_optional: bool,                   // 是否可选
}

/// 边信息 - 专门化的边结构
#[derive(Debug, Clone)]
pub struct EdgeInfo {
    pub variable_name: String,               // 变量名
    pub edge_types: HashSet<String>,         // 边类型列表
    pub direction: String,                   // 方向（"out", "in", "both"）
    pub properties: HashMap<String, String>, // 属性
    pub is_optional: bool,                   // 是否可选
}

/// 路径信息 - 专门化的路径结构
#[derive(Debug, Clone)]
pub struct PathInfo {
    pub variable_name: String, // 变量名
    pub nodes: Vec<String>,    // 节点变量名列表
    pub edges: Vec<String>,    // 边变量名列表
    pub is_shortest: bool,     // 是否最短路径
}

/// Cypher表达式
#[derive(Debug, Clone)]
pub struct CypherExpression {
    pub expression_type: String,       // "property", "function", "literal"
    pub content: String,               // 表达式内容
    pub variable_name: Option<String>, // 关联的变量名
}

/// 变量信息 - 专门化的变量结构
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub var_type: AliasType,                 // 变量类型（强类型枚举）
    pub labels: HashSet<String>,             // 标签列表
    pub properties: HashMap<String, String>, // 属性
    pub is_optional: bool,                   // 是否可选
    pub scope: VariableVisibility,           // 变量可见性
}

/// 变量可见性级别
///
/// 表示变量在查询中的可见性范围，使用符号表管理变量作用域。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableVisibility {
    /// 局部变量（仅在当前QueryPart中可见）
    Local,
    /// 全局变量（在整个查询中可见）
    Global,
}

impl CypherAstContext {
    /// 创建新的Cypher AST上下文
    pub fn new(query_text: &str) -> Self {
        // 创建一个默认的AstContext
        let base = AstContext::from_strings("CYPHER", query_text);

        Self {
            base,
            query_parts: Vec::new(),
            variables: HashMap::new(),
            expressions: Vec::new(),
            parameters: HashMap::new(),
            clause_references: HashMap::new(),
        }
    }

    /// 添加查询分段
    pub fn add_query_part(&mut self, part: QueryPart) {
        self.query_parts.push(part);
    }

    /// 添加Cypher模式到指定分段
    pub fn add_pattern_to_part(&mut self, part_id: &str, pattern: CypherPattern) {
        if let Some(part) = self.query_parts.iter_mut().find(|p| p.part_id == part_id) {
            // 将模式信息添加到该分段的第一个MATCH子句中
            if let Some(clause) = part
                .clauses
                .iter_mut()
                .find(|c| c.clause_kind == CypherClauseKind::Match)
            {
                clause.content.push_str(&format!(" {:?}", pattern));
            }
        }
    }

    /// 添加Cypher子句到指定分段
    pub fn add_clause_to_part(&mut self, part_id: &str, clause: CypherClause) {
        if let Some(part) = self.query_parts.iter_mut().find(|p| p.part_id == part_id) {
            part.clauses.push(clause);
        }
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

    /// 添加子句引用关系
    pub fn add_clause_reference(&mut self, from_clause: String, to_clause: String) {
        self.clause_references
            .entry(from_clause)
            .or_insert_with(Vec::new)
            .push(to_clause);
    }

    /// 获取查询分段列表
    pub fn query_parts(&self) -> &[QueryPart] {
        &self.query_parts
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

    /// 获取子句引用关系
    pub fn clause_references(&self) -> &HashMap<String, Vec<String>> {
        &self.clause_references
    }

    /// 获取基础AST上下文
    pub fn base_context(&self) -> &AstContext {
        &self.base
    }

    /// 检查是否包含特定类型的子句
    pub fn has_clause_kind(&self, clause_kind: CypherClauseKind) -> bool {
        self.query_parts
            .iter()
            .any(|part| part.clauses.iter().any(|c| c.clause_kind == clause_kind))
    }

    /// 获取特定类型的子句
    pub fn get_clauses_by_kind(&self, clause_kind: CypherClauseKind) -> Vec<&CypherClause> {
        self.query_parts
            .iter()
            .flat_map(|part| {
                part.clauses
                    .iter()
                    .filter(move |c| c.clause_kind == clause_kind)
            })
            .collect()
    }

    /// 获取特定标签的模式
    pub fn get_patterns_by_label(&self, _label: &str) -> Vec<&CypherPattern> {
        // 由于模式现在存储在子句内容中，这里返回空列表
        // 实际实现需要解析子句内容来提取模式信息
        Vec::new()
    }

    /// 获取指定分段的输入变量
    pub fn get_part_input_variables(&self, part_id: &str) -> Option<&HashSet<String>> {
        self.query_parts
            .iter()
            .find(|p| p.part_id == part_id)
            .map(|p| &p.input_variables)
    }

    /// 获取指定分段的输出变量
    pub fn get_part_output_variables(&self, part_id: &str) -> Option<&HashSet<String>> {
        self.query_parts
            .iter()
            .find(|p| p.part_id == part_id)
            .map(|p| &p.output_variables)
    }

    /// 获取子句的引用关系
    pub fn get_clause_references(&self, clause_id: &str) -> Option<&Vec<String>> {
        self.clause_references.get(clause_id)
    }
}

impl Default for CypherAstContext {
    fn default() -> Self {
        Self {
            base: AstContext::default(),
            query_parts: Vec::new(),
            variables: HashMap::new(),
            expressions: Vec::new(),
            parameters: HashMap::new(),
            clause_references: HashMap::new(),
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
        assert!(context.query_parts().is_empty());
        assert!(context.variables().is_empty());
    }

    #[test]
    fn test_cypher_ast_context_add_query_part() {
        let mut context = CypherAstContext::new("MATCH (n:Person)");

        let part = QueryPart {
            part_id: "part1".to_string(),
            clauses: vec![CypherClause {
                clause_kind: CypherClauseKind::Match,
                content: "(n:Person)".to_string(),
                position: 0,
                referenced_clauses: vec![],
            }],
            input_variables: HashSet::new(),
            output_variables: ["n"].iter().map(|s| s.to_string()).collect(),
        };

        context.add_query_part(part);
        assert_eq!(context.query_parts().len(), 1);
    }

    #[test]
    fn test_cypher_ast_context_add_clause_to_part() {
        let mut context = CypherAstContext::new("MATCH (n) RETURN n");

        let part = QueryPart {
            part_id: "part1".to_string(),
            clauses: vec![],
            input_variables: HashSet::new(),
            output_variables: HashSet::new(),
        };

        context.add_query_part(part);

        let clause = CypherClause {
            clause_kind: CypherClauseKind::Match,
            content: "(n)".to_string(),
            position: 0,
            referenced_clauses: vec![],
        };

        context.add_clause_to_part("part1", clause);
        assert_eq!(context.query_parts()[0].clauses.len(), 1);
        assert!(context.has_clause_kind(CypherClauseKind::Match));
    }

    #[test]
    fn test_cypher_ast_context_add_variable() {
        let mut context = CypherAstContext::new("MATCH (n:Person)");

        let var_info = VariableInfo {
            var_type: AliasType::Node,
            labels: ["Person"].iter().map(|s| s.to_string()).collect(),
            properties: HashMap::new(),
            is_optional: false,
            scope: VariableVisibility::Local,
        };

        context.add_variable("n".to_string(), var_info);
        assert!(context.variables().contains_key("n"));
    }

    #[test]
    fn test_cypher_ast_context_get_clauses_by_kind() {
        let mut context = CypherAstContext::new("MATCH (n) WHERE n.age > 30 RETURN n");

        let part = QueryPart {
            part_id: "part1".to_string(),
            clauses: vec![
                CypherClause {
                    clause_kind: CypherClauseKind::Match,
                    content: "(n)".to_string(),
                    position: 0,
                    referenced_clauses: vec![],
                },
                CypherClause {
                    clause_kind: CypherClauseKind::Where,
                    content: "n.age > 30".to_string(),
                    position: 1,
                    referenced_clauses: vec![],
                },
                CypherClause {
                    clause_kind: CypherClauseKind::Return,
                    content: "n".to_string(),
                    position: 2,
                    referenced_clauses: vec![],
                },
            ],
            input_variables: HashSet::new(),
            output_variables: HashSet::new(),
        };

        context.add_query_part(part);

        let match_clauses = context.get_clauses_by_kind(CypherClauseKind::Match);
        assert_eq!(match_clauses.len(), 1);

        let where_clauses = context.get_clauses_by_kind(CypherClauseKind::Where);
        assert_eq!(where_clauses.len(), 1);
    }

    #[test]
    fn test_cypher_ast_context_query_parts_variables() {
        let mut context = CypherAstContext::new("MATCH (a) WITH a MATCH (b) RETURN b");

        let part1 = QueryPart {
            part_id: "part1".to_string(),
            clauses: vec![CypherClause {
                clause_kind: CypherClauseKind::Match,
                content: "(a)".to_string(),
                position: 0,
                referenced_clauses: vec![],
            }],
            input_variables: HashSet::new(),
            output_variables: ["a"].iter().map(|s| s.to_string()).collect(),
        };

        let part2 = QueryPart {
            part_id: "part2".to_string(),
            clauses: vec![CypherClause {
                clause_kind: CypherClauseKind::Match,
                content: "(b)".to_string(),
                position: 1,
                referenced_clauses: vec![],
            }],
            input_variables: ["a"].iter().map(|s| s.to_string()).collect(),
            output_variables: ["b"].iter().map(|s| s.to_string()).collect(),
        };

        context.add_query_part(part1);
        context.add_query_part(part2);

        let part1_input = context.get_part_input_variables("part1");
        assert!(part1_input.is_some());
        assert!(part1_input.expect("part1_input should be Some").is_empty());

        let part1_output = context.get_part_output_variables("part1");
        assert!(part1_output.is_some());
        assert!(part1_output.expect("part1_output should be Some").contains("a"));

        let part2_input = context.get_part_input_variables("part2");
        assert!(part2_input.is_some());
        assert!(part2_input.expect("part2_input should be Some").contains("a"));
    }

    #[test]
    fn test_cypher_ast_context_clause_references() {
        let mut context = CypherAstContext::new("MATCH (n) WHERE n.age > 30 RETURN n");

        context.add_clause_reference("match_clause".to_string(), "where_clause".to_string());
        context.add_clause_reference("where_clause".to_string(), "return_clause".to_string());

        let refs = context.get_clause_references("match_clause");
        assert!(refs.is_some());
        let refs = refs.expect("refs should be Some");
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0], "where_clause");
    }
}
