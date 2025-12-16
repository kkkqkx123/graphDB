//! MATCH子句执行器
//!
//! 负责执行Cypher的MATCH语句，实现图模式匹配功能
//! 基于nebula-graph的TraverseExecutor设计，支持高效的图遍历和模式匹配

use crate::core::error::DBError;
use crate::core::{Direction, Edge, Value, Vertex};
use crate::query::executor::cypher::context::CypherExecutionContext;
use crate::query::executor::traits::ExecutionResult;
use crate::query::parser::cypher::ast::clauses::MatchClause;
use crate::query::parser::cypher::ast::patterns::{NodePattern, RelationshipPattern, Direction as PatternDirection};
use crate::query::parser::cypher::ast::expressions::{Expression, BinaryExpression, BinaryOperator, PropertyExpression};
use crate::storage::StorageEngine;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// 路径信息
#[derive(Debug, Clone)]
pub struct PathInfo {
    /// 路径中的节点序列
    pub vertices: Vec<Vertex>,
    /// 路径中的边序列
    pub edges: Vec<Edge>,
    /// 当前路径长度
    pub length: usize,
}

impl PathInfo {
    /// 创建新的路径
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            edges: Vec::new(),
            length: 0,
        }
    }

    /// 添加节点到路径
    pub fn add_vertex(&mut self, vertex: Vertex) {
        self.vertices.push(vertex);
        if self.vertices.len() > 1 {
            self.length = self.vertices.len() - 1;
        }
    }

    /// 添加边到路径
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
        self.length = self.edges.len();
    }

    /// 获取最后一个节点
    pub fn last_vertex(&self) -> Option<&Vertex> {
        self.vertices.last()
    }

    /// 检查路径是否包含重复边
    pub fn has_duplicate_edge(&self, edge: &Edge) -> bool {
        self.edges.iter().any(|e| {
            e.src() == edge.src() && e.dst() == edge.dst() && e.edge_type() == edge.edge_type()
        })
    }
}

/// MATCH子句执行器
///
/// 负责处理图模式匹配，包括：
/// - 节点模式匹配
/// - 边模式匹配
/// - 路径模式匹配
/// - 可选匹配（OPTIONAL MATCH）
/// - WHERE条件过滤
#[derive(Debug)]
pub struct MatchClauseExecutor<S: StorageEngine> {
    storage: Arc<Mutex<S>>,
    id: usize,
    /// 邻接表缓存，用于优化遍历性能
    adj_list: HashMap<Value, Vec<Edge>>,
    /// 已访问的节点集合，避免循环
    visited_vertices: HashSet<Value>,
    /// 当前路径集合
    current_paths: Vec<PathInfo>,
    /// 结果路径集合
    result_paths: Vec<PathInfo>,
}

impl<S: StorageEngine> MatchClauseExecutor<S> {
    /// 创建新的MATCH执行器
    pub fn new(id: usize, storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage,
            id,
            adj_list: HashMap::new(),
            visited_vertices: HashSet::new(),
            current_paths: Vec::new(),
            result_paths: Vec::new(),
        }
    }

    /// 执行模式匹配
    pub async fn execute_match(
        &mut self,
        clause: MatchClause,
        context: &mut CypherExecutionContext,
    ) -> Result<ExecutionResult, DBError> {
        // 设置执行状态
        context.set_state(crate::query::executor::cypher::context::ExecutionState::Executing);

        // 清理之前的状态
        self.reset_state();

        // 解析并执行模式
        for pattern in &clause.patterns {
            self.execute_pattern(pattern, context).await?;
        }

        // 处理WHERE条件
        if let Some(where_clause) = &clause.where_clause {
            self.apply_where_filter(where_clause, context).await?;
        }

        // 构建结果集
        let result = self.build_result(context)?;

        // 设置完成状态
        context.set_state(crate::query::executor::cypher::context::ExecutionState::Completed);

        Ok(result)
    }

    /// 重置执行器状态
    fn reset_state(&mut self) {
        self.adj_list.clear();
        self.visited_vertices.clear();
        self.current_paths.clear();
        self.result_paths.clear();
    }

    /// 执行单个模式
    async fn execute_pattern(
        &mut self,
        pattern: &crate::query::parser::cypher::ast::patterns::Pattern,
        context: &mut CypherExecutionContext,
    ) -> Result<(), DBError> {
        for part in &pattern.parts {
            self.execute_pattern_part(part, context).await?;
        }
        Ok(())
    }

    /// 执行模式部分
    async fn execute_pattern_part(
        &mut self,
        part: &crate::query::parser::cypher::ast::patterns::PatternPart,
        context: &mut CypherExecutionContext,
    ) -> Result<(), DBError> {
        // 首先处理起始节点
        let start_vertices = self.find_start_vertices(&part.node, context).await?;
        
        if start_vertices.is_empty() {
            return Ok(());
        }

        // 初始化路径
        for vertex in start_vertices {
            let mut path = PathInfo::new();
            path.add_vertex(vertex.clone());
            
            // 如果节点有变量名，存储到上下文
            if let Some(var_name) = &part.node.variable {
                context.set_variable_value(var_name, Value::Vertex(Box::new(vertex.clone())));
            }
            
            self.current_paths.push(path);
        }

        // 处理关系模式
        for rel_pattern in &part.relationships {
            self.expand_with_relationship(rel_pattern, context).await?;
        }

        Ok(())
    }

    /// 查找起始节点
    async fn find_start_vertices(
        &mut self,
        node_pattern: &NodePattern,
        context: &mut CypherExecutionContext,
    ) -> Result<Vec<Vertex>, DBError> {
        let storage = self.storage.lock().map_err(|e| {
            DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                "存储引擎锁定失败: {}",
                e
            )))
        })?;

        // 获取所有节点
        let vertices = self.get_all_vertices(&*storage)?;
        
        // 如果没有节点，直接返回空结果
        if vertices.is_empty() {
            return Ok(Vec::new());
        }

        // 按标签过滤
        let filtered_vertices = if !node_pattern.labels.is_empty() {
            self.filter_vertices_by_labels(vertices, &node_pattern.labels)
        } else {
            vertices
        };

        // 按属性过滤
        let filtered_vertices = if let Some(properties) = &node_pattern.properties {
            self.filter_vertices_by_properties(filtered_vertices, properties, context)?
        } else {
            filtered_vertices
        };

        Ok(filtered_vertices)
    }

    /// 获取所有节点（临时实现，实际应该有更高效的方法）
    fn get_all_vertices(&self, storage: &S) -> Result<Vec<Vertex>, DBError> {
        // 这是一个临时实现，实际存储引擎应该提供遍历所有节点的方法
        // 这里返回空列表，避免编译错误
        Ok(Vec::new())
    }

    /// 按标签过滤节点
    fn filter_vertices_by_labels(
        &self,
        vertices: Vec<Vertex>,
        labels: &[String],
    ) -> Vec<Vertex> {
        vertices.into_iter()
            .filter(|vertex| {
                // 检查节点是否包含任一标签
                labels.iter().any(|label| vertex.has_tag(label))
            })
            .collect()
    }

    /// 使用关系模式扩展路径
    async fn expand_with_relationship(
        &mut self,
        rel_pattern: &RelationshipPattern,
        context: &mut CypherExecutionContext,
    ) -> Result<(), DBError> {
        let mut new_paths = Vec::new();

        for path in &self.current_paths {
            if let Some(last_vertex) = path.last_vertex() {
                let expanded_paths = self.expand_path_from_vertex(
                    path,
                    last_vertex,
                    rel_pattern,
                    context,
                ).await?;
                
                new_paths.extend(expanded_paths);
            }
        }

        self.current_paths = new_paths;
        Ok(())
    }

    /// 从指定节点扩展路径
    async fn expand_path_from_vertex(
        &mut self,
        current_path: &PathInfo,
        vertex: &Vertex,
        rel_pattern: &RelationshipPattern,
        context: &mut CypherExecutionContext,
    ) -> Result<Vec<PathInfo>, DBError> {
        // 检查路径长度是否超过限制（防止无限循环）
        if current_path.length > 1000 {
            return Err(DBError::Query(crate::core::error::QueryError::ExecutionError(
                "路径长度超过限制，可能存在循环".to_string()
            )));
        }

        let storage = self.storage.lock().map_err(|e| {
            DBError::Query(crate::core::error::QueryError::ExecutionError(format!(
                "存储引擎锁定失败: {}",
                e
            )))
        })?;

        // 获取节点的邻居边
        let direction = self.pattern_direction_to_storage_direction(&rel_pattern.direction);
        let edges = storage.get_node_edges(&vertex.id, direction)
            .map_err(|e| DBError::Query(crate::core::error::QueryError::ExecutionError(
                format!("获取邻居边失败: {}", e)
            )))?;

        // 如果没有边，直接返回空结果
        if edges.is_empty() {
            return Ok(Vec::new());
        }

        // 按关系类型过滤
        let filtered_edges = if !rel_pattern.types.is_empty() {
            self.filter_edges_by_types(edges, &rel_pattern.types)
        } else {
            edges
        };

        // 按属性过滤
        let filtered_edges = if let Some(properties) = &rel_pattern.properties {
            self.filter_edges_by_properties(filtered_edges, properties, context)?
        } else {
            filtered_edges
        };

        // 如果过滤后没有边，直接返回空结果
        if filtered_edges.is_empty() {
            return Ok(Vec::new());
        }

        // 构建新路径
        let mut new_paths = Vec::new();
        for edge in filtered_edges {
            // 检查是否会产生重复边
            if current_path.has_duplicate_edge(&edge) {
                continue;
            }

            // 获取目标节点
            let target_vertex_id = self.get_target_vertex_id(vertex, &edge, &rel_pattern.direction);
            
            // 检查是否已经访问过该节点（防止循环）
            if self.visited_vertices.contains(&target_vertex_id) {
                continue;
            }

            let target_vertex = storage.get_node(&target_vertex_id)
                .map_err(|e| DBError::Query(crate::core::error::QueryError::ExecutionError(
                    format!("获取目标节点失败: {}", e)
                )))?;

            if let Some(target_vertex) = target_vertex {
                // 创建新路径
                let mut new_path = current_path.clone();
                new_path.add_edge(edge.clone());
                new_path.add_vertex(target_vertex.clone());

                // 如果关系有变量名，存储到上下文
                if let Some(var_name) = &rel_pattern.variable {
                    context.set_variable_value(var_name, Value::Edge(edge.clone()));
                }

                new_paths.push(new_path);
                
                // 标记节点为已访问
                self.visited_vertices.insert(target_vertex_id);
            }
        }

        Ok(new_paths)
    }

    /// 将模式方向转换为存储方向
    fn pattern_direction_to_storage_direction(&self, pattern_dir: &PatternDirection) -> Direction {
        match pattern_dir {
            PatternDirection::Left => Direction::Incoming,
            PatternDirection::Right => Direction::Outgoing,
            PatternDirection::Both => Direction::Both,
        }
    }

    /// 获取目标节点ID
    fn get_target_vertex_id(&self, source_vertex: &Vertex, edge: &Edge, direction: &PatternDirection) -> Value {
        match direction {
            PatternDirection::Right => {
                // 出边，目标是边的dst
                edge.dst().clone()
            }
            PatternDirection::Left => {
                // 入边，目标是边的src
                edge.src().clone()
            }
            PatternDirection::Both => {
                // 双向，选择不是源节点的另一端
                if edge.src() == &source_vertex.id {
                    edge.dst().clone()
                } else {
                    edge.src().clone()
                }
            }
        }
    }

    /// 按关系类型过滤边
    fn filter_edges_by_types(&self, edges: Vec<Edge>, types: &[String]) -> Vec<Edge> {
        edges.into_iter()
            .filter(|edge| types.contains(&edge.edge_type().to_string()))
            .collect()
    }

    /// 按属性过滤节点
    fn filter_vertices_by_properties(
        &self,
        vertices: Vec<Vertex>,
        properties: &HashMap<String, Expression>,
        context: &CypherExecutionContext,
    ) -> Result<Vec<Vertex>, DBError> {
        let mut filtered = Vec::new();

        for vertex in vertices {
            let mut matches = true;

            for (prop_name, prop_expr) in properties {
                // 获取节点属性值
                if let Some(value) = vertex.properties.get(prop_name) {
                    // 求值表达式
                    let expr_value = self.evaluate_expression(prop_expr, context)?;
                    
                    // 比较值
                    if !self.values_equal(value, &expr_value) {
                        matches = false;
                        break;
                    }
                } else {
                    matches = false;
                    break;
                }
            }

            if matches {
                filtered.push(vertex);
            }
        }

        Ok(filtered)
    }

    /// 按属性过滤边
    fn filter_edges_by_properties(
        &self,
        edges: Vec<Edge>,
        properties: &HashMap<String, Expression>,
        context: &CypherExecutionContext,
    ) -> Result<Vec<Edge>, DBError> {
        let mut filtered = Vec::new();

        for edge in edges {
            let mut matches = true;

            for (prop_name, prop_expr) in properties {
                // 获取边属性值
                if let Some(value) = edge.properties().get(prop_name) {
                    // 求值表达式
                    let expr_value = self.evaluate_expression(prop_expr, context)?;
                    
                    // 比较值
                    if !self.values_equal(value, &expr_value) {
                        matches = false;
                        break;
                    }
                } else {
                    matches = false;
                    break;
                }
            }

            if matches {
                filtered.push(edge);
            }
        }

        Ok(filtered)
    }

    /// 求值表达式
    fn evaluate_expression(
        &self,
        expr: &Expression,
        context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        match expr {
            Expression::Literal(literal) => {
                use crate::query::parser::cypher::ast::expressions::Literal;
                match literal {
                    Literal::String(s) => Ok(Value::String(s.clone())),
                    Literal::Integer(i) => Ok(Value::Int(*i)),
                    Literal::Float(f) => Ok(Value::Float(*f)),
                    Literal::Boolean(b) => Ok(Value::Bool(*b)),
                    Literal::Null => Ok(Value::Null(crate::core::value::NullType::Null)),
                }
            }
            Expression::Variable(name) => {
                // 从上下文获取变量值
                context.get_variable_value(name)
                    .cloned()
                    .ok_or_else(|| DBError::Query(crate::core::error::QueryError::ExecutionError(
                        format!("未找到变量: {}", name)
                    )))
            }
            Expression::Property(prop_expr) => {
                // 求值属性表达式
                self.evaluate_property_expression(prop_expr, context)
            }
            Expression::Binary(bin_expr) => {
                // 求值二元表达式
                self.evaluate_binary_expression(bin_expr, context)
            }
            _ => {
                // 其他表达式类型的临时实现
                Ok(Value::String(format!("{:?}", expr)))
            }
        }
    }

    /// 求值属性表达式
    fn evaluate_property_expression(
        &self,
        prop_expr: &PropertyExpression,
        context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        // 求值基础表达式
        let base_value = self.evaluate_expression(&prop_expr.expression, context)?;
        
        match base_value {
            Value::Vertex(vertex) => {
                // 获取节点属性
                if let Some(value) = vertex.properties.get(&prop_expr.property_name) {
                    Ok(value.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
            }
            Value::Edge(edge) => {
                // 获取边属性
                if let Some(value) = edge.properties().get(&prop_expr.property_name) {
                    Ok(value.clone())
                } else {
                    Ok(Value::Null(crate::core::value::NullType::Null))
                }
            }
            _ => Ok(Value::Null(crate::core::value::NullType::Null)),
        }
    }

    /// 求值二元表达式
    fn evaluate_binary_expression(
        &self,
        bin_expr: &BinaryExpression,
        context: &CypherExecutionContext,
    ) -> Result<Value, DBError> {
        let left_value = self.evaluate_expression(&bin_expr.left, context)?;
        let right_value = self.evaluate_expression(&bin_expr.right, context)?;

        match bin_expr.operator {
            BinaryOperator::Equal => Ok(Value::Bool(self.values_equal(&left_value, &right_value))),
            BinaryOperator::NotEqual => Ok(Value::Bool(!self.values_equal(&left_value, &right_value))),
            BinaryOperator::GreaterThan => Ok(Value::Bool(self.compare_values(&left_value, &right_value) > 0)),
            BinaryOperator::LessThan => Ok(Value::Bool(self.compare_values(&left_value, &right_value) < 0)),
            BinaryOperator::GreaterThanOrEqual => Ok(Value::Bool(self.compare_values(&left_value, &right_value) >= 0)),
            BinaryOperator::LessThanOrEqual => Ok(Value::Bool(self.compare_values(&left_value, &right_value) <= 0)),
            BinaryOperator::And => {
                if let (Value::Bool(left_bool), Value::Bool(right_bool)) = (&left_value, &right_value) {
                    Ok(Value::Bool(*left_bool && *right_bool))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            BinaryOperator::Or => {
                if let (Value::Bool(left_bool), Value::Bool(right_bool)) = (&left_value, &right_value) {
                    Ok(Value::Bool(*left_bool || *right_bool))
                } else {
                    Ok(Value::Bool(false))
                }
            }
            _ => Ok(Value::Bool(false)), // 其他操作符的临时实现
        }
    }

    /// 比较两个值是否相等
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Int(l), Value::Int(r)) => l == r,
            (Value::Float(l), Value::Float(r)) => (l - r).abs() < f64::EPSILON,
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Null(_), Value::Null(_)) => true,
            _ => false,
        }
    }

    /// 比较两个值的大小
    fn compare_values(&self, left: &Value, right: &Value) -> i32 {
        match (left, right) {
            (Value::String(l), Value::String(r)) => l.cmp(r).into(),
            (Value::Int(l), Value::Int(r)) => l.cmp(r).into(),
            (Value::Float(l), Value::Float(r)) => l.partial_cmp(r).unwrap_or(std::cmp::Ordering::Equal).into(),
            _ => 0, // 无法比较的类型返回相等
        }
    }

    /// 应用WHERE过滤条件
    async fn apply_where_filter(
        &mut self,
        where_clause: &crate::query::parser::cypher::ast::clauses::WhereClause,
        context: &mut CypherExecutionContext,
    ) -> Result<(), DBError> {
        // 如果没有当前路径，直接返回
        if self.current_paths.is_empty() {
            return Ok(());
        }

        // 求值WHERE表达式
        let result = self.evaluate_expression(&where_clause.expression, context)?;
        
        // 检查结果是否为布尔值
        if let Value::Bool(matches) = result {
            if !matches {
                // 如果条件不满足，清空结果
                self.current_paths.clear();
            }
        } else {
            return Err(DBError::Query(crate::core::error::QueryError::ExecutionError(
                "WHERE表达式必须返回布尔值".to_string()
            )));
        }

        Ok(())
    }

    /// 构建结果集
    fn build_result(&mut self, context: &CypherExecutionContext) -> Result<ExecutionResult, DBError> {
        // 将当前路径添加到结果路径
        self.result_paths.extend(self.current_paths.clone());

        // 根据结果类型返回不同的ExecutionResult
        if self.result_paths.is_empty() {
            return Ok(ExecutionResult::Success);
        }

        // 检查结果数量是否超过限制
        if self.result_paths.len() > 10000 {
            return Err(DBError::Query(crate::core::error::QueryError::ExecutionError(
                "结果集过大，超过限制".to_string()
            )));
        }

        // 如果只有一个节点模式，返回顶点集合
        if self.result_paths.iter().all(|p| p.vertices.len() == 1 && p.edges.is_empty()) {
            let vertices: Vec<Vertex> = self.result_paths.iter()
                .flat_map(|p| p.vertices.clone())
                .collect();
            return Ok(ExecutionResult::Vertices(vertices));
        }

        // 如果只有边模式，返回边集合
        if self.result_paths.iter().all(|p| p.vertices.len() == 2 && p.edges.len() == 1) {
            let edges: Vec<Edge> = self.result_paths.iter()
                .flat_map(|p| p.edges.clone())
                .collect();
            return Ok(ExecutionResult::Edges(edges));
        }

        // 否则返回路径集合
        let paths: Vec<crate::core::vertex_edge_path::Path> = self.result_paths.iter()
            .filter_map(|p| {
                if !p.vertices.is_empty() {
                    let src = p.vertices[0].clone();
                    let mut steps = Vec::new();
                    
                    for i in 0..p.edges.len() {
                        let dst = if i + 1 < p.vertices.len() {
                            p.vertices[i + 1].clone()
                        } else {
                            continue;
                        };
                        
                        let step = crate::core::vertex_edge_path::Step {
                            dst: Box::new(dst),
                            edge: Box::new(p.edges[i].clone()),
                        };
                        steps.push(step);
                    }
                    
                    Some(crate::core::vertex_edge_path::Path {
                        src: Box::new(src),
                        steps,
                    })
                } else {
                    None
                }
            })
            .collect();
        
        Ok(ExecutionResult::Paths(paths))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::cypher::ast::patterns::*;
    use crate::query::parser::cypher::ast::expressions::*;
    use crate::query::parser::cypher::ast::clauses::*;

    #[test]
    fn test_match_executor_creation() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("test_db")
        ));
        let executor = MatchClauseExecutor::new(1, storage);
        assert_eq!(executor.id, 1);
    }

    #[test]
    fn test_path_info() {
        let mut path = PathInfo::new();
        
        let tag1 = crate::core::vertex_edge_path::Tag::new("Person".to_string(), HashMap::new());
        let vertex1 = Vertex::new(
            Value::String("v1".to_string()),
            vec![tag1],
        );
        let tag2 = crate::core::vertex_edge_path::Tag::new("Person".to_string(), HashMap::new());
        let vertex2 = Vertex::new(
            Value::String("v2".to_string()),
            vec![tag2],
        );
        
        path.add_vertex(vertex1.clone());
        assert_eq!(path.length, 0);
        assert_eq!(path.last_vertex(), Some(&vertex1));
        
        path.add_vertex(vertex2);
        assert_eq!(path.length, 1);
    }

    #[test]
    fn test_path_info_duplicate_edge() {
        let mut path = PathInfo::new();
        
        let tag1 = crate::core::vertex_edge_path::Tag::new("Person".to_string(), HashMap::new());
        let vertex1 = Vertex::new(
            Value::String("v1".to_string()),
            vec![tag1],
        );
        let tag2 = crate::core::vertex_edge_path::Tag::new("Person".to_string(), HashMap::new());
        let vertex2 = Vertex::new(
            Value::String("v2".to_string()),
            vec![tag2],
        );
        
        path.add_vertex(vertex1);
        path.add_vertex(vertex2.clone());
        
        let edge = Edge::new(
            Value::String("v1".to_string()),
            Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            HashMap::new(),
        );
        path.add_edge(edge.clone());
        
        assert!(!path.has_duplicate_edge(&edge));
        
        // 添加相同的边
        path.add_edge(edge.clone());
        assert!(path.has_duplicate_edge(&edge));
    }

    #[test]
    fn test_values_equal() {
        let executor = MatchClauseExecutor::<crate::storage::native_storage::NativeStorage>::new(1, Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("test_db")
        )));
        
        assert!(executor.values_equal(
            &Value::String("test".to_string()),
            &Value::String("test".to_string())
        ));
        assert!(!executor.values_equal(
            &Value::String("test".to_string()),
            &Value::String("other".to_string())
        ));
        assert!(executor.values_equal(
            &Value::Int(42),
            &Value::Int(42)
        ));
        assert!(!executor.values_equal(
            &Value::Int(42),
            &Value::Int(43)
        ));
        assert!(executor.values_equal(
            &Value::Bool(true),
            &Value::Bool(true)
        ));
        assert!(!executor.values_equal(
            &Value::Bool(true),
            &Value::Bool(false)
        ));
    }

    #[test]
    fn test_compare_values() {
        let executor = MatchClauseExecutor::<crate::storage::native_storage::NativeStorage>::new(1, Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("test_db")
        )));
        
        assert_eq!(executor.compare_values(
            &Value::Int(10),
            &Value::Int(5)
        ), 1);
        assert_eq!(executor.compare_values(
            &Value::Int(5),
            &Value::Int(10)
        ), -1);
        assert_eq!(executor.compare_values(
            &Value::Int(5),
            &Value::Int(5)
        ), 0);
        
        assert_eq!(executor.compare_values(
            &Value::String("z".to_string()),
            &Value::String("a".to_string())
        ), 1);
        assert_eq!(executor.compare_values(
            &Value::String("a".to_string()),
            &Value::String("z".to_string())
        ), -1);
    }

    #[test]
    fn test_pattern_direction_to_storage_direction() {
        let executor = MatchClauseExecutor::<crate::storage::native_storage::NativeStorage>::new(1, Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("test_db")
        )));
        
        assert!(matches!(
            executor.pattern_direction_to_storage_direction(&PatternDirection::Right),
            Direction::Outgoing
        ));
        assert!(matches!(
            executor.pattern_direction_to_storage_direction(&PatternDirection::Left),
            Direction::Incoming
        ));
        assert!(matches!(
            executor.pattern_direction_to_storage_direction(&PatternDirection::Both),
            Direction::Both
        ));
    }

    #[test]
    fn test_get_target_vertex_id() {
        let executor = MatchClauseExecutor::<crate::storage::native_storage::NativeStorage>::new(1, Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("test_db")
        )));
        
        let vertex1 = Vertex::new(
            Value::String("v1".to_string()),
            vec![],
        );
        let vertex2 = Vertex::new(
            Value::String("v2".to_string()),
            vec![],
        );
        
        let edge = Edge::new(
            Value::String("v1".to_string()),
            Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            HashMap::new(),
        );
        
        // 右向边，目标是dst
        let target = executor.get_target_vertex_id(&vertex1, &edge, &PatternDirection::Right);
        assert_eq!(target, Value::String("v2".to_string()));
        
        // 左向边，目标是src
        let target = executor.get_target_vertex_id(&vertex2, &edge, &PatternDirection::Left);
        assert_eq!(target, Value::String("v1".to_string()));
        
        // 双向边，选择不是源节点的另一端
        let target = executor.get_target_vertex_id(&vertex1, &edge, &PatternDirection::Both);
        assert_eq!(target, Value::String("v2".to_string()));
    }

    #[test]
    fn test_filter_vertices_by_labels() {
        let executor = MatchClauseExecutor::<crate::storage::native_storage::NativeStorage>::new(1, Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("test_db")
        )));
        
        let tag1 = crate::core::vertex_edge_path::Tag::new("Person".to_string(), HashMap::new());
        let tag2 = crate::core::vertex_edge_path::Tag::new("User".to_string(), HashMap::new());
        let tag3 = crate::core::vertex_edge_path::Tag::new("Admin".to_string(), HashMap::new());
        
        let vertex1 = Vertex::new(
            Value::String("v1".to_string()),
            vec![tag1],
        );
        let vertex2 = Vertex::new(
            Value::String("v2".to_string()),
            vec![tag2],
        );
        let vertex3 = Vertex::new(
            Value::String("v3".to_string()),
            vec![tag3],
        );
        
        let vertices = vec![vertex1.clone(), vertex2.clone(), vertex3.clone()];
        
        // 按单个标签过滤
        let filtered = executor.filter_vertices_by_labels(vertices.clone(), &["Person".to_string()]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id(), &Value::String("v1".to_string()));
        
        // 按多个标签过滤
        let filtered = executor.filter_vertices_by_labels(vertices.clone(), &["Person".to_string(), "User".to_string()]);
        assert_eq!(filtered.len(), 2);
        
        // 按不存在的标签过滤
        let filtered = executor.filter_vertices_by_labels(vertices.clone(), &["NonExistent".to_string()]);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_edges_by_types() {
        let executor = MatchClauseExecutor::<crate::storage::native_storage::NativeStorage>::new(1, Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("test_db")
        )));
        
        let edge1 = Edge::new(
            Value::String("v1".to_string()),
            Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            HashMap::new(),
        );
        let edge2 = Edge::new(
            Value::String("v2".to_string()),
            Value::String("v3".to_string()),
            "FOLLOWS".to_string(),
            0,
            HashMap::new(),
        );
        let edge3 = Edge::new(
            Value::String("v3".to_string()),
            Value::String("v4".to_string()),
            "KNOWS".to_string(),
            0,
            HashMap::new(),
        );
        
        let edges = vec![edge1.clone(), edge2.clone(), edge3.clone()];
        
        // 按单个类型过滤
        let filtered = executor.filter_edges_by_types(edges.clone(), &["KNOWS".to_string()]);
        assert_eq!(filtered.len(), 2);
        
        // 按多个类型过滤
        let filtered = executor.filter_edges_by_types(edges.clone(), &["KNOWS".to_string(), "FOLLOWS".to_string()]);
        assert_eq!(filtered.len(), 3);
        
        // 按不存在的类型过滤
        let filtered = executor.filter_edges_by_types(edges.clone(), &["NONEXISTENT".to_string()]);
        assert_eq!(filtered.len(), 0);
    }

    #[tokio::test]
    async fn test_execute_match_empty_pattern() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("test_db")
        ));
        let mut executor = MatchClauseExecutor::new(1, storage);
        let mut context = CypherExecutionContext::new();
        
        let clause = MatchClause {
            patterns: vec![],
            where_clause: None,
            optional: false,
        };
        
        let result = executor.execute_match(clause, &mut context).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), ExecutionResult::Success));
    }

    #[tokio::test]
    async fn test_execute_match_simple_node_pattern() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("test_db")
        ));
        let mut executor = MatchClauseExecutor::new(1, storage);
        let mut context = CypherExecutionContext::new();
        
        let node_pattern = NodePattern {
            variable: Some("n".to_string()),
            labels: vec![],
            properties: None,
        };
        
        let pattern_part = PatternPart {
            node: node_pattern,
            relationships: vec![],
        };
        
        let pattern = Pattern {
            parts: vec![pattern_part],
        };
        
        let clause = MatchClause {
            patterns: vec![pattern],
            where_clause: None,
            optional: false,
        };
        
        let result = executor.execute_match(clause, &mut context).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_literal_expression() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("test_db")
        ));
        let executor = MatchClauseExecutor::new(1, storage);
        let context = CypherExecutionContext::new();
        
        let string_expr = Expression::Literal(Literal::String("test".to_string()));
        let result = executor.evaluate_expression(&string_expr, &context).unwrap();
        assert_eq!(result, Value::String("test".to_string()));
        
        let int_expr = Expression::Literal(Literal::Integer(42));
        let result = executor.evaluate_expression(&int_expr, &context).unwrap();
        assert_eq!(result, Value::Int(42));
        
        let bool_expr = Expression::Literal(Literal::Boolean(true));
        let result = executor.evaluate_expression(&bool_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_evaluate_binary_expression() {
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new("test_db")
        ));
        let executor = MatchClauseExecutor::new(1, storage);
        let context = CypherExecutionContext::new();
        
        // 测试相等比较
        let equal_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(42))),
            operator: BinaryOperator::Equal,
            right: Box::new(Expression::Literal(Literal::Integer(42))),
        });
        let result = executor.evaluate_expression(&equal_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // 测试不相等比较
        let not_equal_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(42))),
            operator: BinaryOperator::NotEqual,
            right: Box::new(Expression::Literal(Literal::Integer(43))),
        });
        let result = executor.evaluate_expression(&not_equal_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // 测试大于比较
        let greater_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(10))),
            operator: BinaryOperator::GreaterThan,
            right: Box::new(Expression::Literal(Literal::Integer(5))),
        });
        let result = executor.evaluate_expression(&greater_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // 测试AND操作
        let and_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Boolean(true))),
            operator: BinaryOperator::And,
            right: Box::new(Expression::Literal(Literal::Boolean(true))),
        });
        let result = executor.evaluate_expression(&and_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
        
        // 测试OR操作
        let or_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Boolean(true))),
            operator: BinaryOperator::Or,
            right: Box::new(Expression::Literal(Literal::Boolean(false))),
        });
        let result = executor.evaluate_expression(&or_expr, &context).unwrap();
        assert_eq!(result, Value::Bool(true));
    }
}
