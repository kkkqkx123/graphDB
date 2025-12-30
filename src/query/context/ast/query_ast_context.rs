//! 查询AST上下文 - 用于NGQL查询的AST上下文

use super::base::AstContext;
use std::collections::HashMap;

/// 查询AST上下文
///
/// 专门用于NGQL查询的AST上下文
/// 包含NGQL查询特有的语法元素和语义信息
/// 注意：执行计划相关的概念应该在独立的模块中管理（如 src/query/planner/plan/）
#[derive(Debug, Clone)]
pub struct QueryAstContext {
    base: AstContext,
    dependencies: HashMap<String, Vec<String>>, // AST节点间的依赖关系
    query_variables: HashMap<String, QueryVariableInfo>, // 查询变量信息
    expression_contexts: Vec<ExpressionContext>, // 表达式上下文
}

/// 查询变量信息
#[derive(Debug, Clone)]
pub struct QueryVariableInfo {
    pub variable_name: String,   // 变量名
    pub variable_type: String,   // 变量类型（如 "vertex", "edge", "path"）
    pub source_clause: String,   // 来源子句（如 "MATCH", "GO"）
    pub is_aggregated: bool,     // 是否聚合变量
    pub properties: Vec<String>, // 访问的属性列表
}

/// 表达式上下文
#[derive(Debug, Clone)]
pub struct ExpressionContext {
    pub expression_id: String,             // 表达式ID
    pub expression_text: String,           // 表达式文本
    pub referenced_variables: Vec<String>, // 引用的变量
    pub expression_type: String,           // 表达式类型（如 "predicate", "projection"）
}

impl QueryAstContext {
    /// 创建新的查询AST上下文
    pub fn new(query_text: &str) -> Self {
        Self {
            base: AstContext::from_strings("QUERY", query_text),
            dependencies: HashMap::new(),
            query_variables: HashMap::new(),
            expression_contexts: Vec::new(),
        }
    }

    /// 添加查询变量信息
    pub fn add_query_variable(&mut self, var_name: String, var_info: QueryVariableInfo) {
        self.query_variables.insert(var_name, var_info);
    }

    /// 添加表达式上下文
    pub fn add_expression_context(&mut self, expr_context: ExpressionContext) {
        self.expression_contexts.push(expr_context);
    }

    /// 添加AST节点间的依赖关系
    pub fn add_dependency(&mut self, node_name: String, dependencies: Vec<String>) {
        self.dependencies.insert(node_name, dependencies);
    }

    /// 获取查询变量信息
    pub fn query_variables(&self) -> &HashMap<String, QueryVariableInfo> {
        &self.query_variables
    }

    /// 获取表达式上下文列表
    pub fn expression_contexts(&self) -> &[ExpressionContext] {
        &self.expression_contexts
    }

    /// 获取依赖关系
    pub fn dependencies(&self) -> &HashMap<String, Vec<String>> {
        &self.dependencies
    }

    /// 获取基础AST上下文
    pub fn base_context(&self) -> &AstContext {
        &self.base
    }

    /// 获取特定查询变量信息
    pub fn get_query_variable(&self, var_name: &str) -> Option<&QueryVariableInfo> {
        self.query_variables.get(var_name)
    }

    /// 获取特定类型的查询变量
    pub fn get_variables_by_type(&self, var_type: &str) -> Vec<&QueryVariableInfo> {
        self.query_variables
            .values()
            .filter(|v| v.variable_type == var_type)
            .collect()
    }

    /// 获取特定来源子句的变量
    pub fn get_variables_by_clause(&self, clause: &str) -> Vec<&QueryVariableInfo> {
        self.query_variables
            .values()
            .filter(|v| v.source_clause == clause)
            .collect()
    }

    /// 获取特定类型的表达式上下文
    pub fn get_expression_contexts_by_type(&self, expr_type: &str) -> Vec<&ExpressionContext> {
        self.expression_contexts
            .iter()
            .filter(|e| e.expression_type == expr_type)
            .collect()
    }

    /// 获取AST节点的依赖关系
    pub fn get_node_dependencies(&self, node_name: &str) -> Option<&Vec<String>> {
        self.dependencies.get(node_name)
    }

    /// 检查是否存在循环依赖
    pub fn has_circular_dependency(&self) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut recursion_stack = std::collections::HashSet::new();

        for node in self.dependencies.keys() {
            if self.has_cycle_helper(node, &mut visited, &mut recursion_stack) {
                return true;
            }
        }
        false
    }

    fn has_cycle_helper(
        &self,
        node: &str,
        visited: &mut std::collections::HashSet<String>,
        recursion_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        if !visited.contains(node) {
            visited.insert(node.to_string());
            recursion_stack.insert(node.to_string());

            if let Some(deps) = self.dependencies.get(node) {
                for dep in deps {
                    if !visited.contains(dep) {
                        if self.has_cycle_helper(dep, visited, recursion_stack) {
                            return true;
                        }
                    } else if recursion_stack.contains(dep) {
                        return true;
                    }
                }
            }
        }
        recursion_stack.remove(node);
        false
    }

    /// 获取所有聚合变量
    pub fn get_aggregated_variables(&self) -> Vec<&QueryVariableInfo> {
        self.query_variables
            .values()
            .filter(|v| v.is_aggregated)
            .collect()
    }

    /// 获取表达式中引用的所有变量
    pub fn get_all_referenced_variables(&self) -> std::collections::HashSet<String> {
        let mut vars = std::collections::HashSet::new();
        for expr in &self.expression_contexts {
            for var in &expr.referenced_variables {
                vars.insert(var.clone());
            }
        }
        vars
    }
}

impl Default for QueryAstContext {
    fn default() -> Self {
        Self {
            base: AstContext::default(),
            dependencies: HashMap::new(),
            query_variables: HashMap::new(),
            expression_contexts: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_ast_context_creation() {
        let query = "GO FROM '1' OVER follow YIELD follow._dst AS dst";
        let context = QueryAstContext::new(query);

        assert_eq!(context.base_context().statement_type(), "QUERY");
        assert!(context.query_variables().is_empty());
        assert!(context.expression_contexts().is_empty());
    }

    #[test]
    fn test_query_ast_context_add_query_variable() {
        let mut context = QueryAstContext::new("GO FROM '1' OVER follow");

        let var_info = QueryVariableInfo {
            variable_name: "dst".to_string(),
            variable_type: "vertex".to_string(),
            source_clause: "GO".to_string(),
            is_aggregated: false,
            properties: vec!["_dst".to_string()],
        };

        context.add_query_variable("dst".to_string(), var_info);
        assert!(context.query_variables().contains_key("dst"));
    }

    #[test]
    fn test_query_ast_context_add_expression_context() {
        let mut context = QueryAstContext::new("GO FROM '1' OVER follow WHERE $$.age > 30");

        let expr_context = ExpressionContext {
            expression_id: "filter1".to_string(),
            expression_text: "$$.age > 30".to_string(),
            referenced_variables: vec!["dst".to_string()],
            expression_type: "predicate".to_string(),
        };

        context.add_expression_context(expr_context);
        assert_eq!(context.expression_contexts().len(), 1);
    }

    #[test]
    fn test_query_ast_context_add_dependency() {
        let mut context = QueryAstContext::new("GO FROM '1' OVER follow YIELD dst");

        let dependencies = vec!["scan_vertex".to_string()];
        context.add_dependency("go_step".to_string(), dependencies.clone());

        assert_eq!(
            context.get_node_dependencies("go_step"),
            Some(&dependencies)
        );
    }

    #[test]
    fn test_query_ast_context_get_variables_by_type() {
        let mut context = QueryAstContext::new("GO FROM '1' OVER follow");

        let var_info1 = QueryVariableInfo {
            variable_name: "dst".to_string(),
            variable_type: "vertex".to_string(),
            source_clause: "GO".to_string(),
            is_aggregated: false,
            properties: vec!["_dst".to_string()],
        };

        let var_info2 = QueryVariableInfo {
            variable_name: "edge".to_string(),
            variable_type: "edge".to_string(),
            source_clause: "GO".to_string(),
            is_aggregated: false,
            properties: vec!["_src".to_string(), "_dst".to_string()],
        };

        context.add_query_variable("dst".to_string(), var_info1);
        context.add_query_variable("edge".to_string(), var_info2);

        let vertex_vars = context.get_variables_by_type("vertex");
        assert_eq!(vertex_vars.len(), 1);
        assert_eq!(vertex_vars[0].variable_name, "dst");
    }

    #[test]
    fn test_query_ast_context_get_variables_by_clause() {
        let mut context = QueryAstContext::new("GO FROM '1' OVER follow");

        let var_info = QueryVariableInfo {
            variable_name: "dst".to_string(),
            variable_type: "vertex".to_string(),
            source_clause: "GO".to_string(),
            is_aggregated: false,
            properties: vec!["_dst".to_string()],
        };

        context.add_query_variable("dst".to_string(), var_info);

        let go_vars = context.get_variables_by_clause("GO");
        assert_eq!(go_vars.len(), 1);
        assert_eq!(go_vars[0].variable_name, "dst");
    }

    #[test]
    fn test_query_ast_context_has_circular_dependency() {
        let mut context = QueryAstContext::new("GO FROM '1' OVER follow");

        context.add_dependency("step1".to_string(), vec!["step2".to_string()]);
        context.add_dependency("step2".to_string(), vec!["step3".to_string()]);
        context.add_dependency("step3".to_string(), vec!["step1".to_string()]);

        assert!(context.has_circular_dependency());
    }

    #[test]
    fn test_query_ast_context_no_circular_dependency() {
        let mut context = QueryAstContext::new("GO FROM '1' OVER follow");

        context.add_dependency("step1".to_string(), vec!["step2".to_string()]);
        context.add_dependency("step2".to_string(), vec!["step3".to_string()]);

        assert!(!context.has_circular_dependency());
    }

    #[test]
    fn test_query_ast_context_get_aggregated_variables() {
        let mut context = QueryAstContext::new("GO FROM '1' OVER follow YIELD COUNT(*) AS count");

        let var_info1 = QueryVariableInfo {
            variable_name: "dst".to_string(),
            variable_type: "vertex".to_string(),
            source_clause: "GO".to_string(),
            is_aggregated: false,
            properties: vec!["_dst".to_string()],
        };

        let var_info2 = QueryVariableInfo {
            variable_name: "count".to_string(),
            variable_type: "integer".to_string(),
            source_clause: "YIELD".to_string(),
            is_aggregated: true,
            properties: vec![],
        };

        context.add_query_variable("dst".to_string(), var_info1);
        context.add_query_variable("count".to_string(), var_info2);

        let aggregated_vars = context.get_aggregated_variables();
        assert_eq!(aggregated_vars.len(), 1);
        assert_eq!(aggregated_vars[0].variable_name, "count");
    }

    #[test]
    fn test_query_ast_context_get_all_referenced_variables() {
        let mut context = QueryAstContext::new("GO FROM '1' OVER follow");

        let expr_context1 = ExpressionContext {
            expression_id: "filter1".to_string(),
            expression_text: "$$.age > 30".to_string(),
            referenced_variables: vec!["dst".to_string()],
            expression_type: "predicate".to_string(),
        };

        let expr_context2 = ExpressionContext {
            expression_id: "proj1".to_string(),
            expression_text: "dst.name".to_string(),
            referenced_variables: vec!["dst".to_string()],
            expression_type: "projection".to_string(),
        };

        context.add_expression_context(expr_context1);
        context.add_expression_context(expr_context2);

        let all_vars = context.get_all_referenced_variables();
        assert!(all_vars.contains("dst"));
        assert_eq!(all_vars.len(), 1);
    }
}
