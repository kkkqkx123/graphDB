//! Cypher执行器上下文
//!
//! 提供Cypher查询执行过程中的上下文管理，
//! 包括变量管理、结果缓存、执行状态等

use crate::core::vertex_edge_path::{Edge, Vertex};
use crate::core::Value;
use crate::query::context::ast::CypherAstContext;
use crate::query::executor::ExecutionContext;
use std::collections::HashMap;

/// Cypher执行器上下文
///
/// 扩展了基础的执行上下文，添加了Cypher特有的功能：
/// - 变量生命周期管理
/// - 模式匹配结果缓存
/// - 表达式求值上下文
/// - 查询参数管理
#[derive(Debug, Clone)]
pub struct CypherExecutionContext {
    /// 基础执行上下文
    base_context: ExecutionContext,
    /// Cypher AST上下文
    ast_context: CypherAstContext,
    /// 变量映射表
    variables: HashMap<String, CypherVariable>,
    /// 模式匹配结果
    pattern_results: HashMap<String, Vec<Value>>,
    /// 查询参数
    parameters: HashMap<String, Value>,
    /// 执行状态
    execution_state: ExecutionState,
    /// 当前作用域
    current_scope: Vec<String>,
    /// 当前顶点（用于表达式求值）
    current_vertex: Option<Vertex>,
    /// 当前边（用于表达式求值）
    current_edge: Option<Edge>,
    /// 路径信息（用于表达式求值）
    paths: HashMap<String, crate::core::vertex_edge_path::Path>,
}

/// Cypher变量信息
#[derive(Debug, Clone)]
pub struct CypherVariable {
    /// 变量名
    pub name: String,
    /// 变量类型
    pub var_type: CypherVariableType,
    /// 变量值
    pub value: Option<Value>,
    /// 变量作用域
    pub scope: String,
    /// 是否为导入变量
    pub is_imported: bool,
}

/// Cypher变量类型
#[derive(Debug, Clone, PartialEq)]
pub enum CypherVariableType {
    /// 节点
    Node,
    /// 边
    Relationship,
    /// 路径
    Path,
    /// 属性
    Property,
    /// 标量值
    Scalar,
    /// 列表
    List,
    /// 映射
    Map,
    /// 未知类型
    Unknown,
}

/// 执行状态
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionState {
    /// 初始状态
    Initial,
    /// 解析中
    Parsing,
    /// 规划中
    Planning,
    /// 执行中
    Executing,
    /// 已完成
    Completed,
    /// 错误状态
    Error(String),
}

impl CypherExecutionContext {
    /// 创建新的Cypher执行上下文
    pub fn new() -> Self {
        Self {
            base_context: ExecutionContext::new(),
            ast_context: CypherAstContext::new(""),
            variables: HashMap::new(),
            pattern_results: HashMap::new(),
            parameters: HashMap::new(),
            execution_state: ExecutionState::Initial,
            current_scope: Vec::new(),
            current_vertex: None,
            current_edge: None,
            paths: HashMap::new(),
        }
    }

    /// 从查询文本创建上下文
    pub fn from_query(query_text: &str) -> Self {
        Self {
            base_context: ExecutionContext::new(),
            ast_context: CypherAstContext::new(query_text),
            variables: HashMap::new(),
            pattern_results: HashMap::new(),
            parameters: HashMap::new(),
            execution_state: ExecutionState::Initial,
            current_scope: Vec::new(),
            current_vertex: None,
            current_edge: None,
            paths: HashMap::new(),
        }
    }

    /// 设置执行状态
    pub fn set_state(&mut self, state: ExecutionState) {
        self.execution_state = state;
    }

    /// 获取执行状态
    pub fn state(&self) -> &ExecutionState {
        &self.execution_state
    }

    /// 添加变量
    pub fn add_variable(&mut self, var: CypherVariable) {
        self.variables.insert(var.name.clone(), var);
    }

    /// 获取变量
    pub fn get_variable(&self, name: &str) -> Option<&CypherVariable> {
        self.variables.get(name)
    }

    /// 获取变量的值
    pub fn get_variable_value(&self, name: &str) -> Option<&Value> {
        self.variables.get(name).and_then(|v| v.value.as_ref())
    }

    /// 设置变量值
    pub fn set_variable_value(&mut self, name: &str, value: Value) {
        if let Some(var) = self.variables.get_mut(name) {
            var.value = Some(value);
        }
    }

    /// 检查变量是否存在
    pub fn has_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// 添加模式匹配结果
    pub fn add_pattern_result(&mut self, pattern_name: String, results: Vec<Value>) {
        self.pattern_results.insert(pattern_name, results);
    }

    /// 获取模式匹配结果
    pub fn get_pattern_result(&self, pattern_name: &str) -> Option<&Vec<Value>> {
        self.pattern_results.get(pattern_name)
    }

    /// 添加查询参数
    pub fn add_parameter(&mut self, name: String, value: Value) {
        self.parameters.insert(name, value);
    }

    /// 获取查询参数
    pub fn get_parameter(&self, name: &str) -> Option<&Value> {
        self.parameters.get(name)
    }

    /// 进入新的作用域
    pub fn enter_scope(&mut self, scope: String) {
        self.current_scope.push(scope);
    }

    /// 退出当前作用域
    pub fn exit_scope(&mut self) {
        self.current_scope.pop();
    }

    /// 获取当前作用域
    pub fn current_scope(&self) -> Option<&String> {
        self.current_scope.last()
    }

    /// 清理上下文
    pub fn clear(&mut self) {
        self.variables.clear();
        self.pattern_results.clear();
        self.parameters.clear();
        self.current_scope.clear();
        self.execution_state = ExecutionState::Initial;
        self.current_vertex = None;
        self.current_edge = None;
        self.paths.clear();
    }

    /// 获取基础执行上下文的引用
    pub fn base_context(&self) -> &ExecutionContext {
        &self.base_context
    }

    /// 获取基础执行上下文的可变引用
    pub fn base_context_mut(&mut self) -> &mut ExecutionContext {
        &mut self.base_context
    }

    /// 获取AST上下文的引用
    pub fn ast_context(&self) -> &CypherAstContext {
        &self.ast_context
    }

    /// 获取AST上下文的可变引用
    pub fn ast_context_mut(&mut self) -> &mut CypherAstContext {
        &mut self.ast_context
    }

    /// 获取所有变量
    pub fn variables(&self) -> &HashMap<String, CypherVariable> {
        &self.variables
    }

    /// 获取所有模式结果
    pub fn pattern_results(&self) -> &HashMap<String, Vec<Value>> {
        &self.pattern_results
    }

    /// 获取所有参数
    pub fn parameters(&self) -> &HashMap<String, Value> {
        &self.parameters
    }

    /// 设置当前顶点
    pub fn set_current_vertex(&mut self, vertex: Vertex) {
        self.current_vertex = Some(vertex);
    }

    /// 获取当前顶点
    pub fn current_vertex(&self) -> Option<&Vertex> {
        self.current_vertex.as_ref()
    }

    /// 设置当前边
    pub fn set_current_edge(&mut self, edge: Edge) {
        self.current_edge = Some(edge);
    }

    /// 获取当前边
    pub fn current_edge(&self) -> Option<&Edge> {
        self.current_edge.as_ref()
    }

    /// 添加路径
    pub fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path) {
        self.paths.insert(name, path);
    }

    /// 获取路径
    pub fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path> {
        self.paths.get(name)
    }

    /// 获取所有路径
    pub fn paths(&self) -> &HashMap<String, crate::core::vertex_edge_path::Path> {
        &self.paths
    }
}

impl Default for CypherExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CypherVariable {
    /// 创建新的变量
    pub fn new(name: String, var_type: CypherVariableType) -> Self {
        Self {
            name,
            var_type,
            value: None,
            scope: "global".to_string(),
            is_imported: false,
        }
    }

    /// 带值创建变量
    pub fn with_value(name: String, var_type: CypherVariableType, value: Value) -> Self {
        Self {
            name,
            var_type,
            value: Some(value),
            scope: "global".to_string(),
            is_imported: false,
        }
    }

    /// 带作用域创建变量
    pub fn with_scope(name: String, var_type: CypherVariableType, scope: String) -> Self {
        Self {
            name,
            var_type,
            value: None,
            scope,
            is_imported: false,
        }
    }

    /// 设置变量值
    pub fn set_value(&mut self, value: Value) {
        self.value = Some(value);
    }

    /// 获取变量值
    pub fn value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    /// 检查变量是否有值
    pub fn has_value(&self) -> bool {
        self.value.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cypher_execution_context_creation() {
        let context = CypherExecutionContext::new();
        assert_eq!(context.state(), &ExecutionState::Initial);
        assert!(context.variables().is_empty());
        assert!(context.pattern_results().is_empty());
    }

    #[test]
    fn test_cypher_execution_context_from_query() {
        let query = "MATCH (n:Person) RETURN n.name";
        let context = CypherExecutionContext::from_query(query);
        assert_eq!(
            context.ast_context().base_context().statement_type(),
            "CYPHER"
        );
    }

    #[test]
    fn test_variable_management() {
        let mut context = CypherExecutionContext::new();

        // 添加变量
        let var = CypherVariable::new("n".to_string(), CypherVariableType::Node);
        context.add_variable(var);

        // 检查变量存在
        assert!(context.has_variable("n"));

        // 设置变量值
        let value = Value::String("test".to_string());
        context.set_variable_value("n", value.clone());

        // 获取变量值
        assert_eq!(context.get_variable_value("n"), Some(&value));
    }

    #[test]
    fn test_scope_management() {
        let mut context = CypherExecutionContext::new();

        // 进入作用域
        context.enter_scope("scope1".to_string());
        assert_eq!(context.current_scope(), Some(&"scope1".to_string()));

        // 进入嵌套作用域
        context.enter_scope("scope2".to_string());
        assert_eq!(context.current_scope(), Some(&"scope2".to_string()));

        // 退出作用域
        context.exit_scope();
        assert_eq!(context.current_scope(), Some(&"scope1".to_string()));

        // 退出最后作用域
        context.exit_scope();
        assert_eq!(context.current_scope(), None);
    }

    #[test]
    fn test_parameter_management() {
        let mut context = CypherExecutionContext::new();

        // 添加参数
        let param = Value::Int(42);
        context.add_parameter("param1".to_string(), param.clone());

        // 获取参数
        assert_eq!(context.get_parameter("param1"), Some(&param));

        // 检查不存在的参数
        assert_eq!(context.get_parameter("nonexistent"), None);
    }

    #[test]
    fn test_pattern_results() {
        let mut context = CypherExecutionContext::new();

        // 添加模式结果
        let results = vec![
            Value::String("result1".to_string()),
            Value::String("result2".to_string()),
        ];
        context.add_pattern_result("pattern1".to_string(), results.clone());

        // 获取模式结果
        assert_eq!(context.get_pattern_result("pattern1"), Some(&results));
    }

    #[test]
    fn test_execution_state() {
        let mut context = CypherExecutionContext::new();

        // 设置执行状态
        context.set_state(ExecutionState::Parsing);
        assert_eq!(context.state(), &ExecutionState::Parsing);

        context.set_state(ExecutionState::Executing);
        assert_eq!(context.state(), &ExecutionState::Executing);

        context.set_state(ExecutionState::Completed);
        assert_eq!(context.state(), &ExecutionState::Completed);
    }

    #[test]
    fn test_context_clear() {
        let mut context = CypherExecutionContext::new();

        // 添加一些数据
        context.add_variable(CypherVariable::new(
            "n".to_string(),
            CypherVariableType::Node,
        ));
        context.add_parameter("param".to_string(), Value::Int(42));
        context.set_state(ExecutionState::Executing);

        // 清理上下文
        context.clear();

        // 验证清理结果
        assert!(context.variables().is_empty());
        assert!(context.parameters().is_empty());
        assert_eq!(context.state(), &ExecutionState::Initial);
    }
}
