//! 模式匹配引擎模块
//!
//! 提供节点和关系模式的匹配逻辑，包括标签和属性过滤

use crate::core::error::DBError;
use crate::core::{Edge, Value, Vertex};
use crate::query::executor::cypher::clauses::match_path::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::cypher::context::CypherExecutionContext;
use crate::query::parser::cypher::ast::patterns::NodePattern;
use crate::storage::StorageEngine;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 模式匹配器
#[derive(Debug)]
pub struct PatternMatcher<S: StorageEngine> {
    storage: Arc<Mutex<S>>,
}

impl<S: StorageEngine> PatternMatcher<S> {
    /// 创建新的模式匹配器
    pub fn new(storage: Arc<Mutex<S>>) -> Self {
        Self { storage }
    }

    /// 查找起始节点
    pub async fn find_start_vertices(
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

    /// 按标签过滤节点
    pub fn filter_vertices_by_labels(
        &self,
        vertices: Vec<Vertex>,
        labels: &[String],
    ) -> Vec<Vertex> {
        vertices
            .into_iter()
            .filter(|vertex| {
                // 检查节点是否包含任一标签
                labels.iter().any(|label| vertex.has_tag(label))
            })
            .collect()
    }

    /// 按属性过滤节点
    pub fn filter_vertices_by_properties(
        &self,
        vertices: Vec<Vertex>,
        properties: &HashMap<String, crate::query::parser::cypher::ast::expressions::Expression>,
        context: &CypherExecutionContext,
    ) -> Result<Vec<Vertex>, DBError> {
        let mut filtered = Vec::new();

        for vertex in vertices {
            let mut matches = true;

            for (prop_name, prop_expr) in properties {
                // 获取节点属性值
                if let Some(value) = vertex.get_property_any(prop_name) {
                    // 求值表达式
                    let expr_value = ExpressionEvaluator::evaluate(prop_expr, context)?;

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

    /// 按关系类型过滤边
    pub fn filter_edges_by_types(&self, edges: Vec<Edge>, types: &[String]) -> Vec<Edge> {
        edges
            .into_iter()
            .filter(|edge| types.contains(&edge.edge_type().to_string()))
            .collect()
    }

    /// 按属性过滤边
    pub fn filter_edges_by_properties(
        &self,
        edges: Vec<Edge>,
        properties: &HashMap<String, crate::query::parser::cypher::ast::expressions::Expression>,
        context: &CypherExecutionContext,
    ) -> Result<Vec<Edge>, DBError> {
        let mut filtered = Vec::new();

        for edge in edges {
            let mut matches = true;

            for (prop_name, prop_expr) in properties {
                // 获取边属性值
                if let Some(value) = edge.get_property(prop_name) {
                    // 求值表达式
                    let expr_value = ExpressionEvaluator::evaluate(prop_expr, context)?;

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

    /// 获取所有节点（临时实现，实际应该有更高效的方法）
    fn get_all_vertices(&self, _storage: &S) -> Result<Vec<Vertex>, DBError> {
        // 这是一个临时实现，实际存储引擎应该提供遍历所有节点的方法
        // 这里返回空列表，避免编译错误
        Ok(Vec::new())
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_config::test_config;
    use crate::core::vertex_edge_path::Tag;
    use crate::query::executor::cypher::context::CypherExecutionContext;
    use crate::query::parser::cypher::ast::expressions::*;

    #[test]
    fn test_filter_vertices_by_labels() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("Failed to create storage"),
        ));
        let matcher = PatternMatcher::new(storage);

        let tag1 = Tag::new("Person".to_string(), HashMap::new());
        let tag2 = Tag::new("User".to_string(), HashMap::new());
        let tag3 = Tag::new("Admin".to_string(), HashMap::new());

        let vertex1 = Vertex::new(Value::String("v1".to_string()), vec![tag1]);
        let vertex2 = Vertex::new(Value::String("v2".to_string()), vec![tag2]);
        let vertex3 = Vertex::new(Value::String("v3".to_string()), vec![tag3]);

        let vertices = vec![vertex1.clone(), vertex2.clone(), vertex3.clone()];

        // 按单个标签过滤
        let filtered = matcher.filter_vertices_by_labels(vertices.clone(), &["Person".to_string()]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].vid(), &Value::String("v1".to_string()));

        // 按多个标签过滤
        let filtered = matcher.filter_vertices_by_labels(
            vertices.clone(),
            &["Person".to_string(), "User".to_string()],
        );
        assert_eq!(filtered.len(), 2);

        // 按不存在的标签过滤
        let filtered =
            matcher.filter_vertices_by_labels(vertices.clone(), &["NonExistent".to_string()]);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_edges_by_types() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("Failed to create storage"),
        ));
        let matcher = PatternMatcher::new(storage);

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
        let filtered = matcher.filter_edges_by_types(edges.clone(), &["KNOWS".to_string()]);
        assert_eq!(filtered.len(), 2);

        // 按多个类型过滤
        let filtered = matcher
            .filter_edges_by_types(edges.clone(), &["KNOWS".to_string(), "FOLLOWS".to_string()]);
        assert_eq!(filtered.len(), 3);

        // 按不存在的类型过滤
        let filtered = matcher.filter_edges_by_types(edges.clone(), &["NONEXISTENT".to_string()]);
        assert_eq!(filtered.len(), 0);
    }

    #[tokio::test]
    async fn test_filter_vertices_by_properties() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("Failed to create storage"),
        ));
        let matcher = PatternMatcher::new(storage);
        let context = CypherExecutionContext::new();

        // 创建带有属性的节点
        let mut props1 = HashMap::new();
        props1.insert("name".to_string(), Value::String("Alice".to_string()));
        props1.insert("age".to_string(), Value::Int(30));

        let tag1 = Tag::new("Person".to_string(), HashMap::new());
        let vertex1 =
            Vertex::new_with_properties(Value::String("v1".to_string()), vec![tag1], props1);

        let mut props2 = HashMap::new();
        props2.insert("name".to_string(), Value::String("Bob".to_string()));
        props2.insert("age".to_string(), Value::Int(25));

        let tag2 = Tag::new("Person".to_string(), HashMap::new());
        let vertex2 =
            Vertex::new_with_properties(Value::String("v2".to_string()), vec![tag2], props2);

        let vertices = vec![vertex1, vertex2];

        // 创建属性过滤条件
        let mut properties = HashMap::new();
        properties.insert("age".to_string(), Expression::Literal(Literal::Integer(30)));

        let filtered = matcher
            .filter_vertices_by_properties(vertices, &properties, &context)
            .expect("Failed to get next");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].vid(), &Value::String("v1".to_string()));
    }

    #[tokio::test]
    async fn test_filter_edges_by_properties() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("Failed to create storage"),
        ));
        let matcher = PatternMatcher::new(storage);
        let context = CypherExecutionContext::new();

        // 创建带有属性的边
        let mut props1 = HashMap::new();
        props1.insert("weight".to_string(), Value::Float(0.5));
        props1.insert("since".to_string(), Value::Int(2020));

        let edge1 = Edge::new(
            Value::String("v1".to_string()),
            Value::String("v2".to_string()),
            "KNOWS".to_string(),
            0,
            props1,
        );

        let mut props2 = HashMap::new();
        props2.insert("weight".to_string(), Value::Float(0.8));
        props2.insert("since".to_string(), Value::Int(2021));

        let edge2 = Edge::new(
            Value::String("v2".to_string()),
            Value::String("v3".to_string()),
            "KNOWS".to_string(),
            0,
            props2,
        );

        let edges = vec![edge1, edge2];

        // 创建属性过滤条件
        let mut properties = HashMap::new();
        properties.insert(
            "weight".to_string(),
            Expression::Literal(Literal::Float(0.5)),
        );

        let filtered = matcher
            .filter_edges_by_properties(edges, &properties, &context)
            .expect("Failed to get next");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].src(), &Value::String("v1".to_string()));
    }

    #[test]
    fn test_values_equal() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("Failed to create storage"),
        ));
        let matcher = PatternMatcher::new(storage);

        assert!(matcher.values_equal(
            &Value::String("test".to_string()),
            &Value::String("test".to_string())
        ));
        assert!(!matcher.values_equal(
            &Value::String("test".to_string()),
            &Value::String("other".to_string())
        ));
        assert!(matcher.values_equal(&Value::Int(42), &Value::Int(42)));
        assert!(!matcher.values_equal(&Value::Int(42), &Value::Int(43)));
        assert!(matcher.values_equal(&Value::Bool(true), &Value::Bool(true)));
        assert!(!matcher.values_equal(&Value::Bool(true), &Value::Bool(false)));
    }
}
