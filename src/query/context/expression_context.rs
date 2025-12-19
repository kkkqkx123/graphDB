//! 表达式上下文 - 表达式求值
//!
//! 表达式求值上下文
//! 对应原C++中的QueryExpressionContext.h/cpp

use crate::core::Value;
use crate::query::context::{QueryContext, ExecutionContext};
use crate::storage::iterator::IteratorEnum;
use std::collections::HashMap;

/// 表达式求值上下文
pub struct ExpressionContext<'a> {
    /// 查询上下文
    pub query_context: &'a QueryContext,
    /// 执行上下文（可选）
    pub execution_context: Option<&'a ExecutionContext>,
    /// 当前行（可选）
    pub current_row: Option<&'a crate::storage::iterator::Row>,
    /// 当前迭代器（可选）
    pub current_iterator: Option<&'a IteratorEnum>,
    /// 局部变量
    local_variables: HashMap<String, Value>,
}

impl<'a> ExpressionContext<'a> {
    /// 创建新的表达式上下文
    pub fn new(query_context: &'a QueryContext) -> Self {
        Self {
            query_context,
            execution_context: None,
            current_row: None,
            current_iterator: None,
            local_variables: HashMap::new(),
        }
    }

    /// 设置执行上下文
    pub fn with_execution_context(mut self, ctx: &'a ExecutionContext) -> Self {
        self.execution_context = Some(ctx);
        self
    }

    /// 设置当前行
    pub fn with_current_row(mut self, row: &'a crate::storage::iterator::Row) -> Self {
        self.current_row = Some(row);
        self
    }

    /// 设置当前迭代器
    pub fn with_current_iterator(mut self, iterator: &'a IteratorEnum) -> Self {
        self.current_iterator = Some(iterator);
        self
    }

    /// 获取变量值
    ///
    /// 查找顺序：
    /// 1. 局部变量
    /// 2. 执行上下文变量
    /// 3. 查询上下文变量
    /// 4. 查询参数
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        // 1. 检查局部变量
        if let Some(value) = self.local_variables.get(name) {
            return Some(value);
        }

        // 2. 检查执行上下文变量
        if let Some(exec_ctx) = self.execution_context {
            if let Some(value) = exec_ctx.get_variable(name) {
                return Some(value);
            }
        }

        // 3. 检查查询上下文变量
        if let Some(value) = self.query_context.get_variable(name) {
            return Some(value);
        }

        // 4. 检查查询参数
        if let Some(value) = self.query_context.get_parameter(name) {
            return Some(value);
        }

        None
    }

    /// 设置局部变量
    pub fn set_local_variable(&mut self, name: String, value: Value) {
        self.local_variables.insert(name, value);
    }

    /// 获取局部变量
    pub fn get_local_variable(&self, name: &str) -> Option<&Value> {
        self.local_variables.get(name)
    }

    /// 删除局部变量
    pub fn remove_local_variable(&mut self, name: &str) -> Option<Value> {
        self.local_variables.remove(name)
    }

    /// 清除所有局部变量
    pub fn clear_local_variables(&mut self) {
        self.local_variables.clear();
    }

    /// 获取列值（从当前行或迭代器）
    pub fn get_column(&self, name: &str) -> Option<&Value> {
        // 首先尝试从迭代器获取
        if let Some(iterator) = self.current_iterator {
            return iterator.get_column(name);
        }

        // 如果没有迭代器，尝试从当前行获取（按索引）
        if let Some(row) = self.current_row {
            // 简单实现：假设第一列是name
            if row.len() > 0 {
                return Some(&row[0]);
            }
        }

        None
    }

    /// 按索引获取列值
    pub fn get_column_by_index(&self, index: i32) -> Option<&Value> {
        // 首先尝试从迭代器获取
        if let Some(iterator) = self.current_iterator {
            return iterator.get_column_by_index(index);
        }

        // 如果没有迭代器，尝试从当前行获取
        if let Some(row) = self.current_row {
            let idx = if index < 0 { row.len() as i32 + index } else { index } as usize;
            if idx < row.len() {
                return Some(&row[idx]);
            }
        }

        None
    }

    /// 获取列索引
    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        // 从迭代器获取
        if let Some(iterator) = self.current_iterator {
            return iterator.get_column_index(name);
        }

        // 简单实现：假设只有一列
        if self.current_row.is_some() {
            Some(0)
        } else {
            None
        }
    }

    /// 获取所有列名
    pub fn get_column_names(&self) -> Vec<String> {
        // 从迭代器获取
        if let Some(iterator) = self.current_iterator {
            return iterator.get_col_names();
        }

        // 简单实现：假设只有一列
        if self.current_row.is_some() {
            vec!["column".to_string()]
        } else {
            Vec::new()
        }
    }

    /// 获取变量属性值（$a.prop_name）
    pub fn get_variable_property(&self, var_name: &str, prop_name: &str) -> Option<&Value> {
        if let Some(var_value) = self.get_variable(var_name) {
            match var_value {
                Value::Vertex(vertex) => {
                    // 从顶点中获取属性
                    for tag in &vertex.tags {
                        if let Some(value) = tag.properties.get(prop_name) {
                            return Some(value);
                        }
                    }
                }
                Value::Edge(edge) => {
                    // 从边中获取属性
                    if let Some(value) = edge.props.get(prop_name) {
                        return Some(value);
                    }
                }
                Value::Map(map) => {
                    // 从Map中获取属性
                    if let Some(value) = map.get(prop_name) {
                        return Some(value);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// 获取标签属性（tag.prop_name）
    pub fn get_tag_property(&self, tag_name: &str, prop_name: &str) -> Option<Value> {
        if let Some(iterator) = self.current_iterator {
            return iterator.get_tag_prop(tag_name, prop_name);
        }
        None
    }

    /// 获取边属性（edge.prop_name）
    pub fn get_edge_property(&self, edge_name: &str, prop_name: &str) -> Option<Value> {
        if let Some(iterator) = self.current_iterator {
            return iterator.get_edge_prop(edge_name, prop_name);
        }
        None
    }

    /// 获取源顶点属性（$^.prop_name）
    pub fn get_source_property(&self, tag_name: &str, prop_name: &str) -> Option<Value> {
        // 暂时使用get_tag_prop作为替代
        if let Some(iterator) = self.current_iterator {
            return iterator.get_tag_prop(tag_name, prop_name);
        }
        None
    }

    /// 获取目标顶点属性（$$.prop_name）
    pub fn get_destination_property(&self, tag_name: &str, prop_name: &str) -> Option<Value> {
        // 暂时使用get_tag_prop作为替代
        if let Some(iterator) = self.current_iterator {
            return iterator.get_tag_prop(tag_name, prop_name);
        }
        None
    }

    /// 获取输入属性（$-.prop_name）
    pub fn get_input_property(&self, prop_name: &str) -> Option<Value> {
        // 暂时使用get_column作为替代
        self.get_column(prop_name).cloned()
    }

    /// 获取顶点
    pub fn get_vertex(&self, name: &str) -> Option<Value> {
        if let Some(iterator) = self.current_iterator {
            return iterator.get_vertex(name);
        }
        None
    }

    /// 获取边
    pub fn get_edge(&self) -> Option<Value> {
        if let Some(iterator) = self.current_iterator {
            return iterator.get_edge();
        }
        None
    }

    /// 检查是否有当前行
    pub fn has_current_row(&self) -> bool {
        self.current_row.is_some()
    }

    /// 检查是否有当前迭代器
    pub fn has_current_iterator(&self) -> bool {
        self.current_iterator.is_some()
    }

    /// 检查迭代器是否有效
    pub fn is_iterator_valid(&self) -> bool {
        if let Some(iterator) = self.current_iterator {
            iterator.valid()
        } else {
            false
        }
    }

    /// 获取局部变量数量
    pub fn local_variable_count(&self) -> usize {
        self.local_variables.len()
    }

    /// 获取所有局部变量名
    pub fn local_variable_names(&self) -> Vec<String> {
        self.local_variables.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::managers::r#impl::{
        MockIndexManager, MockMetaClient, MockSchemaManager, MockStorageClient,
    };
    use std::collections::HashMap;

    fn create_test_query_context() -> QueryContext {
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let mut ctx = QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        );

        // 设置一些测试变量和参数
        ctx.set_variable("query_var".to_string(), Value::Int(100));
        ctx.set_parameter("param".to_string(), Value::String("test".to_string()));

        ctx
    }

    #[test]
    fn test_expression_context_creation() {
        let query_ctx = create_test_query_context();
        let expr_ctx = ExpressionContext::new(&query_ctx);

        assert!(!expr_ctx.has_current_row());
        assert!(!expr_ctx.has_current_iterator());
        assert_eq!(expr_ctx.local_variable_count(), 0);
    }

    #[test]
    fn test_variable_resolution() {
        let query_ctx = create_test_query_context();
        let mut expr_ctx = ExpressionContext::new(&query_ctx);

        // 设置局部变量
        expr_ctx.set_local_variable("local_var".to_string(), Value::Int(200));

        // 测试变量解析顺序
        assert_eq!(expr_ctx.get_variable("local_var"), Some(&Value::Int(200)));
        assert_eq!(expr_ctx.get_variable("query_var"), Some(&Value::Int(100)));
        assert_eq!(expr_ctx.get_variable("param"), Some(&Value::String("test".to_string())));
        assert_eq!(expr_ctx.get_variable("nonexistent"), None);
    }

    #[test]
    fn test_local_variable_management() {
        let query_ctx = create_test_query_context();
        let mut expr_ctx = ExpressionContext::new(&query_ctx);

        // 设置局部变量
        expr_ctx.set_local_variable("var1".to_string(), Value::Int(1));
        expr_ctx.set_local_variable("var2".to_string(), Value::String("test".to_string()));

        // 获取局部变量
        assert_eq!(expr_ctx.get_local_variable("var1"), Some(&Value::Int(1)));
        assert_eq!(expr_ctx.get_local_variable("var2"), Some(&Value::String("test".to_string())));

        // 获取变量名列表
        let names = expr_ctx.local_variable_names();
        assert!(names.contains(&"var1".to_string()));
        assert!(names.contains(&"var2".to_string()));
        assert_eq!(names.len(), 2);

        // 删除局部变量
        let removed = expr_ctx.remove_local_variable("var1");
        assert_eq!(removed, Some(Value::Int(1)));
        assert_eq!(expr_ctx.get_local_variable("var1"), None);

        // 清除所有局部变量
        expr_ctx.clear_local_variables();
        assert_eq!(expr_ctx.local_variable_count(), 0);
    }

    #[test]
    fn test_with_methods() {
        let query_ctx = create_test_query_context();
        let expr_ctx = ExpressionContext::new(&query_ctx);

        // 测试with_execution_context
        let schema_manager = Arc::new(MockSchemaManager::new());
        let index_manager = Arc::new(MockIndexManager::new());
        let meta_client = Arc::new(MockMetaClient::new());
        let storage_client = Arc::new(MockStorageClient::new());

        let query_ctx2 = Arc::new(QueryContext::new(
            "session123".to_string(),
            "user456".to_string(),
            schema_manager,
            index_manager,
            meta_client,
            storage_client,
        ));

        let exec_ctx = ExecutionContext::new(query_ctx2);
        let expr_ctx_with_exec = expr_ctx.with_execution_context(&exec_ctx);
        assert!(expr_ctx_with_exec.execution_context.is_some());

        // 注意：由于with_current_row和with_current_iterator需要具体的Row和IteratorEnum实例，
        // 这里只测试方法存在性，实际功能需要更复杂的测试设置
    }

    #[test]
    fn test_variable_property_access() {
        let query_ctx = create_test_query_context();
        let mut expr_ctx = ExpressionContext::new(&query_ctx);

        // 创建一个包含属性的顶点
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), Value::String("Alice".to_string()));
        properties.insert("age".to_string(), Value::Int(30));

        let tag = crate::core::vertex_edge_path::Tag {
            name: "Person".to_string(),
            properties,
        };

        let vertex = Value::Vertex(crate::core::vertex_edge_path::Vertex {
            id: Value::String("v1".to_string()),
            tags: vec![tag],
        });

        // 设置顶点变量
        expr_ctx.set_local_variable("person".to_string(), vertex);

        // 测试属性访问
        assert_eq!(
            expr_ctx.get_variable_property("person", "name"),
            Some(&Value::String("Alice".to_string()))
        );
        assert_eq!(
            expr_ctx.get_variable_property("person", "age"),
            Some(&Value::Int(30))
        );
        assert_eq!(expr_ctx.get_variable_property("person", "nonexistent"), None);
    }

    #[test]
    fn test_map_property_access() {
        let query_ctx = create_test_query_context();
        let mut expr_ctx = ExpressionContext::new(&query_ctx);

        // 创建一个Map
        let mut map = HashMap::new();
        map.insert("key1".to_string(), Value::Int(1));
        map.insert("key2".to_string(), Value::String("value2".to_string()));

        // 设置Map变量
        expr_ctx.set_local_variable("map_var".to_string(), Value::Map(map));

        // 测试属性访问
        assert_eq!(
            expr_ctx.get_variable_property("map_var", "key1"),
            Some(&Value::Int(1))
        );
        assert_eq!(
            expr_ctx.get_variable_property("map_var", "key2"),
            Some(&Value::String("value2".to_string()))
        );
        assert_eq!(expr_ctx.get_variable_property("map_var", "nonexistent"), None);
    }
}