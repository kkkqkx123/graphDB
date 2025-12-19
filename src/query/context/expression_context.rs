//! 表达式求值上下文 - 为表达式提供运行时上下文
//!
//! 对应原C++中的QueryExpressionContext.h/cpp
//! 提供：
//! - 变量访问（从ExecutionContext）
//! - 列访问（从当前迭代器行）
//! - 属性访问（标签、边、顶点）
//! - 表达式内部变量管理

use super::QueryExecutionContext;
use crate::core::Value;
use crate::storage::iterator::IteratorEnum;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

/// 表达式求值上下文
///
/// 为表达式求值提供：
/// 1. 变量值访问（来自QueryExecutionContext）
/// 2. 当前行列数据访问（来自迭代器）
/// 3. 属性访问（标签、边、顶点属性）
/// 4. 表达式内部变量（用于列表解析等）
#[derive(Clone)]
pub struct QueryExpressionContext {
    // 查询执行上下文（变量值来源）
    ectx: Arc<QueryExecutionContext>,

    // 当前迭代器（用于访问行数据）
    iter: Arc<Mutex<Option<IteratorEnum>>>,

    // 表达式内部变量（例如列表解析中的变量）
    expr_value_map: Arc<RwLock<HashMap<String, Value>>>,
}

impl std::fmt::Debug for QueryExpressionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let has_iterator = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned")
            .is_some();
        let expr_vars_count = self.expr_value_map.read()
            .expect("ExpressionContext expression value map lock should not be poisoned")
            .len();
        
        f.debug_struct("QueryExpressionContext")
            .field("has_iterator", &has_iterator)
            .field("expr_vars", &expr_vars_count)
            .finish()
    }
}

impl QueryExpressionContext {
    /// 创建新的表达式上下文
    ///
    /// # 参数
    /// - `ectx`: 查询执行上下文的Arc指针
    pub fn new(ectx: Arc<QueryExecutionContext>) -> Self {
        Self {
            ectx,
            iter: Arc::new(Mutex::new(None)),
            expr_value_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 设置当前迭代器（用于行数据访问）
    ///
    /// # 参数
    /// - `iter`: 迭代器
    ///
    /// # 返回
    /// 更新后的上下文（链式调用）
    pub fn with_iterator(self, iter: IteratorEnum) -> Self {
        *self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned") = Some(iter);
        self
    }

    // ===== 变量访问 =====

    /// 获取变量值（最新版本）
    ///
    /// # 参数
    /// - `var`: 变量名
    ///
    /// # 返回
    /// - Ok(Value): 变量值
    /// - Err(String): 如果变量不存在
    pub fn get_var(&self, var: &str) -> Result<Value, String> {
        self.ectx.get_value(var)
    }

    /// 获取指定版本的变量值
    ///
    /// # 参数
    /// - `var`: 变量名
    /// - `version`: 版本号（0=最新，-1=前一个）
    pub fn get_versioned_var(&self, var: &str, version: i64) -> Result<Value, String> {
        match self.ectx.get_versioned_result(var, version) {
            Ok(result) => Ok(result.value().clone()),
            Err(e) => Err(e),
        }
    }

    /// 设置变量值
    ///
    /// # 参数
    /// - `var`: 变量名
    /// - `value`: 变量值
    pub fn set_var(&self, var: &str, value: Value) -> Result<(), String> {
        self.ectx.set_value(var, value)
    }

    // ===== 表达式内部变量 =====

    /// 设置表达式内部变量（不持久化到执行上下文）
    ///
    /// 用于列表解析、临时变量等场景
    pub fn set_inner_var(&self, var: &str, value: Value) {
        self.expr_value_map
            .write()
            .expect("ExpressionContext expression value map write lock should not be poisoned")
            .insert(var.to_string(), value);
    }

    /// 获取表达式内部变量
    pub fn get_inner_var(&self, var: &str) -> Option<Value> {
        self.expr_value_map.read()
            .expect("ExpressionContext expression value map read lock should not be poisoned")
            .get(var).cloned()
    }

    /// 清除所有表达式内部变量
    pub fn clear_inner_vars(&self) {
        self.expr_value_map.write()
            .expect("ExpressionContext expression value map write lock should not be poisoned")
            .clear();
    }

    // ===== 列访问 =====

    /// 获取列值（从当前行）
    ///
    /// # 参数
    /// - `col`: 列名
    ///
    /// # 返回
    /// - Ok(Value): 列值
    /// - Err(String): 如果列不存在或没有迭代器
    pub fn get_column(&self, col: &str) -> Result<Value, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => iter
                .get_column(col)
                .ok_or_else(|| format!("列 {} 不存在", col))
                .map(|v| v.clone()),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 按索引获取列值
    ///
    /// # 参数
    /// - `index`: 列索引（支持负数）
    pub fn get_column_by_index(&self, index: i32) -> Result<Value, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => iter
                .get_column_by_index(index)
                .ok_or_else(|| format!("列索引 {} 不存在", index))
                .map(|v| v.clone()),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取列索引
    ///
    /// # 参数
    /// - `col`: 列名
    pub fn get_column_index(&self, col: &str) -> Result<usize, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => iter
                .get_column_index(col)
                .ok_or_else(|| format!("列 {} 的索引不存在", col)),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取所有列名
    pub fn get_col_names(&self) -> Result<Vec<String>, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => Ok(iter.get_col_names()),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    // ===== 属性访问 =====

    /// 获取变量属性值（$a.prop_name）
    ///
    /// # 参数
    /// - `var`: 变量名
    /// - `prop`: 属性名
    pub fn get_var_prop(&self, var: &str, prop: &str) -> Result<Value, String> {
        // 获取变量值
        let var_val = self.get_var(var)?;

        // 根据Value的类型提取属性
        match &var_val {
            Value::Vertex(vertex) => {
                // 从顶点中获取属性（需要指定标签名，这里使用默认标签）
                // 在实际使用中，可能需要指定标签名，这里先使用第一个标签
                if let Some(tag) = vertex.tags.first() {
                    tag.properties
                        .get(prop)
                        .ok_or_else(|| format!("顶点变量 {} 的属性 {} 不存在", var, prop))
                        .map(|v| v.clone())
                } else {
                    Err(format!("顶点变量 {} 没有标签", var))
                }
            }
            Value::Edge(edge) => {
                // 从边中获取属性
                edge.props
                    .get(prop)
                    .ok_or_else(|| format!("边变量 {} 的属性 {} 不存在", var, prop))
                    .map(|v| v.clone())
            }
            Value::Map(map) => {
                // 从Map中获取属性
                map.get(prop)
                    .ok_or_else(|| format!("Map变量 {} 的属性 {} 不存在", var, prop))
                    .map(|v| v.clone())
            }
            Value::DataSet(dataset) => {
                // 从DataSet中获取列
                let col_idx = dataset
                    .col_names
                    .iter()
                    .position(|c| c == prop)
                    .ok_or_else(|| format!("DataSet变量 {} 的列 {} 不存在", var, prop))?;

                // 获取当前行的值（如果有迭代器）
                let iter_guard = self.iter.lock()
                    .expect("ExpressionContext iterator lock should not be poisoned");
                match iter_guard.as_ref() {
                    Some(iter) => {
                        if iter.valid() {
                            iter.get_column_by_index(col_idx as i32)
                                .ok_or_else(|| {
                                    format!("DataSet变量 {} 的列 {} 值不存在", var, prop)
                                })
                                .map(|v| v.clone())
                        } else {
                            Err("迭代器无效".to_string())
                        }
                    }
                    None => Err("没有设置迭代器".to_string()),
                }
            }
            _ => Err(format!("变量 {} 的类型不支持属性访问", var)),
        }
    }

    /// 获取标签属性（tag.prop_name）
    ///
    /// # 参数
    /// - `tag`: 标签名
    /// - `prop`: 属性名
    pub fn get_tag_prop(&self, tag: &str, prop: &str) -> Result<Value, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => iter
                .get_tag_prop(tag, prop)
                .ok_or_else(|| format!("标签 {} 的属性 {} 不存在", tag, prop)),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取边属性（edge.prop_name）
    ///
    /// # 参数
    /// - `edge`: 边名
    /// - `prop`: 属性名
    pub fn get_edge_prop(&self, edge: &str, prop: &str) -> Result<Value, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => iter
                .get_edge_prop(edge, prop)
                .ok_or_else(|| format!("边 {} 的属性 {} 不存在", edge, prop)),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    // ===== 属性访问（补充缺失的接口） =====

    /// 获取源顶点属性（$^.prop_name）
    ///
    /// # 参数
    /// - `tag`: 标签名
    /// - `prop`: 属性名
    pub fn get_src_prop(&self, tag: &str, prop: &str) -> Result<Value, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => {
                // 从当前顶点获取源属性
                // 这需要迭代器支持源顶点属性访问
                // 暂时委托给 get_tag_prop
                iter.get_tag_prop(tag, prop)
                    .ok_or_else(|| format!("源顶点标签 {} 的属性 {} 不存在", tag, prop))
            }
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取目标顶点属性（$$.prop_name）
    ///
    /// # 参数
    /// - `tag`: 标签名
    /// - `prop`: 属性名
    pub fn get_dst_prop(&self, tag: &str, prop: &str) -> Result<Value, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => {
                // 从当前边获取目标顶点属性
                // 这需要迭代器支持目标顶点属性访问
                // 暂时委托给 get_tag_prop
                iter.get_tag_prop(tag, prop)
                    .ok_or_else(|| format!("目标顶点标签 {} 的属性 {} 不存在", tag, prop))
            }
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取输入属性（$-.prop_name）
    ///
    /// # 参数
    /// - `prop`: 属性名
    pub fn get_input_prop(&self, prop: &str) -> Result<Value, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => {
                // 从输入行获取属性
                iter.get_column(prop)
                    .ok_or_else(|| format!("输入属性 {} 不存在", prop))
                    .map(|v| v.clone())
            }
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取输入属性索引
    ///
    /// # 参数
    /// - `prop`: 属性名
    pub fn get_input_prop_index(&self, prop: &str) -> Result<usize, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => iter
                .get_column_index(prop)
                .ok_or_else(|| format!("输入属性 {} 的索引不存在", prop)),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    // ===== 对象获取 =====

    /// 获取顶点
    ///
    /// # 参数
    /// - `name`: 顶点名（可选）
    pub fn get_vertex(&self, name: &str) -> Result<Value, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => iter
                .get_vertex(name)
                .ok_or_else(|| format!("顶点 {} 不存在", name)),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    /// 获取边
    pub fn get_edge(&self) -> Result<Value, String> {
        let iter_guard = self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned");
        match iter_guard.as_ref() {
            Some(iter) => iter.get_edge().ok_or_else(|| "边不存在".to_string()),
            None => Err("没有设置迭代器".to_string()),
        }
    }

    // ===== 迭代器管理 =====

    /// 检查是否有迭代器
    pub fn has_iterator(&self) -> bool {
        self.iter.lock()
            .expect("ExpressionContext iterator lock should not be poisoned")
            .is_some()
    }

    /// 获取当前迭代器的有效性
    pub fn is_iter_valid(&self) -> bool {
        self.iter
            .lock()
            .expect("ExpressionContext iterator lock should not be poisoned")
            .as_ref()
            .map(|iter| iter.valid())
            .unwrap_or(false)
    }

    /// 获取查询执行上下文的引用
    pub fn ectx(&self) -> &Arc<QueryExecutionContext> {
        &self.ectx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inner_var_management() {
        let qectx = Arc::new(QueryExecutionContext::new());
        let qctx = QueryExpressionContext::new(qectx);

        qctx.set_inner_var("temp", Value::Int(100));
        assert_eq!(qctx.get_inner_var("temp"), Some(Value::Int(100)));

        qctx.clear_inner_vars();
        assert_eq!(qctx.get_inner_var("temp"), None);
    }

    #[test]
    fn test_var_access() {
        let qectx = Arc::new(QueryExecutionContext::new());
        qectx.set_value("x", Value::Int(42))
            .expect("Failed to set test value");

        let qctx = QueryExpressionContext::new(qectx);
        let val = qctx.get_var("x")
            .expect("Failed to get test value");
        assert_eq!(val, Value::Int(42));
    }

    #[test]
    fn test_nonexistent_var() {
        let qectx = Arc::new(QueryExecutionContext::new());
        let qctx = QueryExpressionContext::new(qectx);

        let result = qctx.get_var("undefined");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_var() {
        let qectx = Arc::new(QueryExecutionContext::new());
        let qctx = QueryExpressionContext::new(qectx);

        qctx.set_var("y", Value::String("hello".to_string()))
            .expect("Failed to set test variable");
        assert_eq!(
            qctx.get_var("y")
                .expect("Failed to get test variable"),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_iterator_without_iterator_set() {
        let qectx = Arc::new(QueryExecutionContext::new());
        let qctx = QueryExpressionContext::new(qectx);

        assert!(!qctx.has_iterator());
        assert!(qctx.get_column("col").is_err());
    }

    #[test]
    fn test_clone() {
        let qectx = Arc::new(QueryExecutionContext::new());
        qectx.set_value("x", Value::Int(42))
            .expect("Failed to set test value");

        let qctx = QueryExpressionContext::new(qectx);
        qctx.set_inner_var("temp", Value::Int(100));

        let cloned = qctx.clone();
        assert_eq!(cloned.get_var("x")
            .expect("Failed to get test value"), Value::Int(42));
        assert_eq!(cloned.get_inner_var("temp"), Some(Value::Int(100)));
    }
}
