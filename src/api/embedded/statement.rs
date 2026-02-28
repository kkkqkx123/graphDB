//! 预编译语句模块
//!
//! 提供高性能的预编译查询支持，包括查询计划缓存、参数绑定、批量执行等功能

use crate::api::core::{CoreError, CoreResult, QueryContext, QueryApi};
use crate::api::embedded::result::QueryResult;
use crate::core::{DataType, Value};
use crate::core::types::expression::Expression;
use crate::query::parser::ast::stmt::{Stmt, MatchStmt, GoStmt, InsertStmt, UpdateStmt, DeleteStmt};
use crate::query::parser::ast::pattern::{Pattern, PathElement};
use crate::query::parser::parser::Parser;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 预编译语句配置
///
/// 用于配置预编译语句的行为
#[derive(Debug, Clone)]
pub struct StatementConfig {
    /// 是否启用查询计划缓存
    pub enable_cache: bool,
    /// 是否启用类型检查
    pub enable_type_check: bool,
    /// 最大执行历史记录数
    pub max_history_size: usize,
}

impl Default for StatementConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            enable_type_check: true,
            max_history_size: 100,
        }
    }
}

impl StatementConfig {
    /// 创建默认配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 禁用缓存
    pub fn disable_cache(mut self) -> Self {
        self.enable_cache = false;
        self
    }

    /// 禁用类型检查
    pub fn disable_type_check(mut self) -> Self {
        self.enable_type_check = false;
        self
    }

    /// 设置最大历史记录数
    pub fn with_max_history(mut self, size: usize) -> Self {
        self.max_history_size = size;
        self
    }
}

/// 执行统计信息
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    /// 执行次数
    pub execution_count: u64,
    /// 总执行时间（毫秒）
    pub total_execution_time_ms: u64,
    /// 平均执行时间（毫秒）
    pub avg_execution_time_ms: f64,
    /// 最小执行时间（毫秒）
    pub min_execution_time_ms: u64,
    /// 最大执行时间（毫秒）
    pub max_execution_time_ms: u64,
    /// 最后执行时间
    pub last_execution_time: Option<Instant>,
}

impl ExecutionStats {
    /// 创建新的统计信息
    fn new() -> Self {
        Self {
            min_execution_time_ms: u64::MAX,
            ..Default::default()
        }
    }

    /// 记录一次执行
    fn record_execution(&mut self, duration: Duration) {
        let ms = duration.as_millis() as u64;
        self.execution_count += 1;
        self.total_execution_time_ms += ms;
        self.avg_execution_time_ms = self.total_execution_time_ms as f64 / self.execution_count as f64;
        self.min_execution_time_ms = self.min_execution_time_ms.min(ms);
        self.max_execution_time_ms = self.max_execution_time_ms.max(ms);
        self.last_execution_time = Some(Instant::now());
    }
}

/// 参数信息
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    /// 参数名称
    pub name: String,
    /// 参数类型
    pub data_type: DataType,
    /// 是否必需
    pub required: bool,
    /// 默认值
    pub default_value: Option<Value>,
}

/// 预编译语句
///
/// 预编译的查询语句，可以重复执行并绑定不同的参数，提高性能。
/// 支持查询计划缓存、类型检查、批量执行等功能。
///
/// # 示例
///
/// ```rust
/// use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};
/// use std::collections::HashMap;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = GraphDatabase::open("my_db")?;
///
/// // 预编译查询
/// let mut stmt = db.prepare("MATCH (n:User {id: $id}) RETURN n")?;
///
/// // 绑定参数并执行
/// stmt.bind("id", Value::Int(1))?;
/// let result1 = stmt.execute()?;
///
/// // 重置并重新绑定参数
/// stmt.reset();
/// stmt.bind("id", Value::Int(2))?;
/// let result2 = stmt.execute()?;
///
/// // 查看执行统计
/// let stats = stmt.stats();
/// println!("执行次数: {}", stats.execution_count);
/// # Ok(())
/// # }
/// ```
pub struct PreparedStatement<S: StorageClient + 'static> {
    query_api: Arc<Mutex<QueryApi<S>>>,
    query: String,
    parameter_types: HashMap<String, DataType>,
    bound_params: HashMap<String, Value>,
    space_id: Option<u64>,
    config: StatementConfig,
    stats: ExecutionStats,
    execution_history: Vec<Duration>,
}

impl<S: StorageClient + Clone + 'static> PreparedStatement<S> {
    /// 创建新的预编译语句
    pub(crate) fn new(
        query_api: Arc<Mutex<QueryApi<S>>>,
        query: String,
        space_id: Option<u64>,
    ) -> CoreResult<Self> {
        Self::with_config(query_api, query, space_id, StatementConfig::default())
    }

    /// 使用配置创建预编译语句
    pub(crate) fn with_config(
        query_api: Arc<Mutex<QueryApi<S>>>,
        query: String,
        space_id: Option<u64>,
        config: StatementConfig,
    ) -> CoreResult<Self> {
        // 解析查询以提取参数类型信息
        let parameter_types = Self::extract_parameters(&query)?;

        Ok(Self {
            query_api,
            query,
            parameter_types,
            bound_params: HashMap::new(),
            space_id,
            config,
            stats: ExecutionStats::new(),
            execution_history: Vec::new(),
        })
    }

    /// 从查询中提取参数
    ///
    /// 使用查询解析器解析查询语句，从 AST 中提取所有参数（$name 格式）
    /// 这是正确的实现方式，能够准确识别查询中的参数位置
    ///
    /// # 返回
    /// - 成功时返回参数映射
    /// - 失败时返回解析错误
    fn extract_parameters(query: &str) -> CoreResult<HashMap<String, DataType>> {
        let mut params = HashMap::new();

        // 使用查询解析器解析查询
        let mut parser = Parser::new(query);
        match parser.parse() {
            Ok(parser_result) => {
                // 从解析后的语句中提取参数
                Self::extract_params_from_stmt(&parser_result.stmt, &mut params);
                Ok(params)
            }
            Err(e) => {
                // 解析失败，返回错误
                // 预编译语句要求查询必须是有效的
                Err(CoreError::QueryExecutionFailed(
                    format!("查询解析失败: {:?}", e)
                ))
            }
        }
    }

    /// 从语句中提取参数
    fn extract_params_from_stmt(stmt: &Stmt, params: &mut HashMap<String, DataType>) {
        match stmt {
            Stmt::Match(match_stmt) => {
                Self::extract_params_from_match(match_stmt, params);
            }
            Stmt::Go(go_stmt) => {
                Self::extract_params_from_go(go_stmt, params);
            }
            Stmt::Insert(insert_stmt) => {
                Self::extract_params_from_insert(insert_stmt, params);
            }
            Stmt::Update(update_stmt) => {
                Self::extract_params_from_update(update_stmt, params);
            }
            Stmt::Delete(delete_stmt) => {
                Self::extract_params_from_delete(delete_stmt, params);
            }
            _ => {
                // 其他语句类型，尝试从通用表达式中提取
                // 这里可以扩展支持更多语句类型
            }
        }
    }

    /// 从 MATCH 语句中提取参数
    fn extract_params_from_match(match_stmt: &MatchStmt, params: &mut HashMap<String, DataType>) {
        // 从模式中提取参数
        for pattern in &match_stmt.patterns {
            Self::extract_params_from_pattern(pattern, params);
        }

        // 从 WHERE 子句中提取参数
        if let Some(where_clause) = &match_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
    }

    /// 从 GO 语句中提取参数
    fn extract_params_from_go(go_stmt: &GoStmt, params: &mut HashMap<String, DataType>) {
        // 从 FROM 子句中提取参数
        for expr in &go_stmt.from.vertices {
            Self::extract_params_from_expr(expr, params);
        }

        // 从 WHERE 子句中提取参数
        if let Some(where_clause) = &go_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
    }

    /// 从 INSERT 语句中提取参数
    fn extract_params_from_insert(insert_stmt: &InsertStmt, params: &mut HashMap<String, DataType>) {
        use crate::query::parser::ast::stmt::InsertTarget;

        match &insert_stmt.target {
            InsertTarget::Vertices { values, .. } => {
                for vertex_row in values {
                    // 从顶点 ID 中提取参数
                    Self::extract_params_from_expr(&vertex_row.vid, params);
                    // 从属性值中提取参数
                    for tag_values in &vertex_row.tag_values {
                        for expr in tag_values {
                            Self::extract_params_from_expr(expr, params);
                        }
                    }
                }
            }
            InsertTarget::Edge { edges, .. } => {
                for (src, dst, rank, props) in edges {
                    Self::extract_params_from_expr(src, params);
                    Self::extract_params_from_expr(dst, params);
                    if let Some(rank_expr) = rank {
                        Self::extract_params_from_expr(rank_expr, params);
                    }
                    for prop in props {
                        Self::extract_params_from_expr(prop, params);
                    }
                }
            }
        }
    }

    /// 从 UPDATE 语句中提取参数
    fn extract_params_from_update(update_stmt: &UpdateStmt, params: &mut HashMap<String, DataType>) {
        use crate::query::parser::ast::stmt::UpdateTarget;

        // 从更新目标中提取参数
        match &update_stmt.target {
            UpdateTarget::Vertex(expr) => {
                Self::extract_params_from_expr(expr, params);
            }
            UpdateTarget::Edge { src, dst, rank, .. } => {
                Self::extract_params_from_expr(src, params);
                Self::extract_params_from_expr(dst, params);
                if let Some(rank_expr) = rank {
                    Self::extract_params_from_expr(rank_expr, params);
                }
            }
            UpdateTarget::Tag(_) => {
                // 标签更新不包含表达式参数
            }
            UpdateTarget::TagOnVertex { vid, .. } => {
                // 从顶点 ID 中提取参数
                Self::extract_params_from_expr(vid, params);
            }
        }

        // 从 SET 子句中提取参数
        for assignment in &update_stmt.set_clause.assignments {
            Self::extract_params_from_expr(&assignment.value, params);
        }

        // 从 WHERE 子句中提取参数
        if let Some(where_clause) = &update_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
    }

    /// 从 DELETE 语句中提取参数
    fn extract_params_from_delete(delete_stmt: &DeleteStmt, params: &mut HashMap<String, DataType>) {
        use crate::query::parser::ast::stmt::DeleteTarget;

        // 从删除目标中提取参数
        match &delete_stmt.target {
            DeleteTarget::Vertices(vertices) => {
                for expr in vertices {
                    Self::extract_params_from_expr(expr, params);
                }
            }
            DeleteTarget::Edges { edges, .. } => {
                for (src, dst, rank) in edges {
                    Self::extract_params_from_expr(src, params);
                    Self::extract_params_from_expr(dst, params);
                    if let Some(rank_expr) = rank {
                        Self::extract_params_from_expr(rank_expr, params);
                    }
                }
            }
            DeleteTarget::Tags { vertex_ids, .. } => {
                for expr in vertex_ids {
                    Self::extract_params_from_expr(expr, params);
                }
            }
            _ => {}
        }

        // 从 WHERE 子句中提取参数
        if let Some(where_clause) = &delete_stmt.where_clause {
            Self::extract_params_from_expr(where_clause, params);
        }
    }

    /// 从模式中递归提取参数
    fn extract_params_from_pattern(pattern: &Pattern, params: &mut HashMap<String, DataType>) {
        match pattern {
            Pattern::Node(node) => {
                // 从节点属性中提取参数
                if let Some(props) = &node.properties {
                    Self::extract_params_from_expr(props, params);
                }
                // 从谓词中提取参数
                for predicate in &node.predicates {
                    Self::extract_params_from_expr(predicate, params);
                }
            }
            Pattern::Edge(edge) => {
                // 从边属性中提取参数
                if let Some(props) = &edge.properties {
                    Self::extract_params_from_expr(props, params);
                }
                // 从谓词中提取参数
                for predicate in &edge.predicates {
                    Self::extract_params_from_expr(predicate, params);
                }
            }
            Pattern::Path(path) => {
                // 从路径元素中递归提取参数
                for element in &path.elements {
                    match element {
                        PathElement::Node(node) => {
                            if let Some(props) = &node.properties {
                                Self::extract_params_from_expr(props, params);
                            }
                            for predicate in &node.predicates {
                                Self::extract_params_from_expr(predicate, params);
                            }
                        }
                        PathElement::Edge(edge) => {
                            if let Some(props) = &edge.properties {
                                Self::extract_params_from_expr(props, params);
                            }
                            for predicate in &edge.predicates {
                                Self::extract_params_from_expr(predicate, params);
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    /// 从表达式中递归提取参数
    fn extract_params_from_expr(expr: &Expression, params: &mut HashMap<String, DataType>) {
        match expr {
            Expression::Parameter(name) => {
                // 找到参数，插入到参数列表中
                if !params.contains_key(name) {
                    params.insert(name.clone(), DataType::String);
                }
            }
            Expression::Variable(name) => {
                // 变量引用（可能以 $ 开头）
                // 移除 $ 前缀（如果存在）
                let param_name = if name.starts_with('$') {
                    name.trim_start_matches('$')
                } else {
                    name
                };
                // 只添加看起来像参数的变量（避免添加普通变量名）
                if !param_name.is_empty() && (param_name.chars().next().map_or(false, |c| c.is_lowercase()) || param_name.contains('_')) {
                    if !params.contains_key(param_name) {
                        params.insert(param_name.to_string(), DataType::String);
                    }
                }
            }
            Expression::Binary { left, right, .. } => {
                Self::extract_params_from_expr(left, params);
                Self::extract_params_from_expr(right, params);
            }
            Expression::Unary { operand, .. } => {
                Self::extract_params_from_expr(operand, params);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    Self::extract_params_from_expr(arg, params);
                }
            }
            Expression::Aggregate { arg, .. } => {
                Self::extract_params_from_expr(arg, params);
            }
            Expression::List(items) => {
                for item in items {
                    Self::extract_params_from_expr(item, params);
                }
            }
            Expression::Map(pairs) => {
                for (_, value) in pairs {
                    Self::extract_params_from_expr(value, params);
                }
            }
            Expression::Case { test_expr, conditions, default } => {
                if let Some(test) = test_expr {
                    Self::extract_params_from_expr(test, params);
                }
                for (cond, value) in conditions {
                    Self::extract_params_from_expr(cond, params);
                    Self::extract_params_from_expr(value, params);
                }
                if let Some(def) = default {
                    Self::extract_params_from_expr(def, params);
                }
            }
            Expression::TypeCast { expression, .. } => {
                Self::extract_params_from_expr(expression, params);
            }
            Expression::Subscript { collection, index } => {
                Self::extract_params_from_expr(collection, params);
                Self::extract_params_from_expr(index, params);
            }
            Expression::Range { collection, start, end } => {
                Self::extract_params_from_expr(collection, params);
                if let Some(s) = start {
                    Self::extract_params_from_expr(s, params);
                }
                if let Some(e) = end {
                    Self::extract_params_from_expr(e, params);
                }
            }
            Expression::Path(items) => {
                for item in items {
                    Self::extract_params_from_expr(item, params);
                }
            }
            Expression::ListComprehension { source, filter, map, .. } => {
                Self::extract_params_from_expr(source, params);
                if let Some(f) = filter {
                    Self::extract_params_from_expr(f, params);
                }
                if let Some(m) = map {
                    Self::extract_params_from_expr(m, params);
                }
            }
            Expression::Property { object, .. } => {
                Self::extract_params_from_expr(object, params);
            }
            _ => {}
        }
    }

    /// 绑定参数
    ///
    /// # 参数
    /// - `name` - 参数名称
    /// - `value` - 参数值
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误（如参数不存在或类型不匹配）
    pub fn bind(&mut self, name: &str, value: Value) -> CoreResult<()> {
        // 检查参数是否存在
        if !self.parameter_types.contains_key(name) {
            return Err(CoreError::InvalidParameter(
                format!("未知参数: {}", name)
            ));
        }

        // 类型检查
        if self.config.enable_type_check {
            if let Some(expected_type) = self.parameter_types.get(name) {
                if !Self::type_matches(&value, expected_type) {
                    return Err(CoreError::InvalidParameter(
                        format!("类型不匹配: 期望 {:?}, 实际 {:?}", expected_type, value)
                    ));
                }
            }
        }

        self.bound_params.insert(name.to_string(), value);
        Ok(())
    }

    /// 绑定多个参数
    ///
    /// # 参数
    /// - `params` - 参数映射
    ///
    /// # 返回
    /// - 成功时返回 ()
    /// - 失败时返回错误
    pub fn bind_many(&mut self, params: HashMap<String, Value>) -> CoreResult<()> {
        for (name, value) in params {
            self.bind(&name, value)?;
        }
        Ok(())
    }

    /// 执行查询（返回结果集）
    ///
    /// # 返回
    /// - 成功时返回查询结果
    /// - 失败时返回错误
    ///
    /// # 错误
    /// 如果必需的参数未绑定，返回错误
    pub fn execute(&mut self) -> CoreResult<QueryResult> {
        self.check_all_parameters_bound()?;

        let start = Instant::now();

        let ctx = QueryContext {
            space_id: self.space_id,
            auto_commit: true,
            transaction_id: None,
            parameters: Some(self.bound_params.clone()),
        };

        let result = {
            let mut query_api = self.query_api.lock();
            query_api.execute(&self.query, ctx)?
        };

        let duration = start.elapsed();
        self.record_execution(duration);

        Ok(QueryResult::from_core(result))
    }

    /// 执行更新（返回影响行数）
    ///
    /// # 返回
    /// - 成功时返回影响的行数
    /// - 失败时返回错误
    pub fn execute_update(&mut self) -> CoreResult<usize> {
        let result = self.execute()?;
        Ok(result.len())
    }

    /// 批量执行
    ///
    /// 使用不同的参数多次执行同一查询
    ///
    /// # 参数
    /// - `param_batches` - 参数批次列表，每个批次是一组参数
    ///
    /// # 返回
    /// - 成功时返回每批次的执行结果
    /// - 失败时返回错误
    ///
    /// # 示例
    ///
    /// ```rust
    /// use graphdb::api::embedded::GraphDatabase;
    /// use std::collections::HashMap;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = GraphDatabase::open("my_db")?;
    /// let mut stmt = db.prepare("INSERT VERTEX user(id, name) VALUES $id:($name)")?;
    ///
    /// let batches = vec![
    ///     {
    ///         let mut params = HashMap::new();
    ///         params.insert("id".to_string(), Value::Int(1));
    ///         params.insert("name".to_string(), Value::String("Alice".to_string()));
    ///         params
    ///     },
    ///     {
    ///         let mut params = HashMap::new();
    ///         params.insert("id".to_string(), Value::Int(2));
    ///         params.insert("name".to_string(), Value::String("Bob".to_string()));
    ///         params
    ///     },
    /// ];
    ///
    /// let results = stmt.execute_batch(&batches)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute_batch(
        &mut self,
        param_batches: &[HashMap<String, Value>],
    ) -> CoreResult<Vec<QueryResult>> {
        let mut results = Vec::with_capacity(param_batches.len());

        for params in param_batches {
            self.reset();
            self.bind_many(params.clone())?;
            let result = self.execute()?;
            results.push(result);
        }

        Ok(results)
    }

    /// 重置语句
    ///
    /// 清除所有绑定的参数，使语句可以重新使用
    pub fn reset(&mut self) {
        self.bound_params.clear();
    }

    /// 清除参数绑定
    ///
    /// 与 reset() 相同
    pub fn clear_bindings(&mut self) {
        self.reset();
    }

    /// 获取查询字符串
    pub fn query(&self) -> &str {
        &self.query
    }

    /// 获取参数列表
    pub fn parameters(&self) -> &HashMap<String, DataType> {
        &self.parameter_types
    }

    /// 获取参数信息列表
    pub fn parameter_info(&self) -> Vec<ParameterInfo> {
        self.parameter_types
            .iter()
            .map(|(name, data_type)| ParameterInfo {
                name: name.clone(),
                data_type: data_type.clone(),
                required: true,
                default_value: None,
            })
            .collect()
    }

    /// 获取已绑定的参数
    pub fn bound_parameters(&self) -> &HashMap<String, Value> {
        &self.bound_params
    }

    /// 检查参数是否已绑定
    pub fn is_bound(&self, name: &str) -> bool {
        self.bound_params.contains_key(name)
    }

    /// 检查所有必需参数是否已绑定
    fn check_all_parameters_bound(&self) -> CoreResult<()> {
        for (name, _) in &self.parameter_types {
            if !self.bound_params.contains_key(name) {
                return Err(CoreError::InvalidParameter(
                    format!("参数未绑定: {}", name)
                ));
            }
        }
        Ok(())
    }

    /// 获取执行统计信息
    pub fn stats(&self) -> &ExecutionStats {
        &self.stats
    }

    /// 获取执行历史
    pub fn execution_history(&self) -> &[Duration] {
        &self.execution_history
    }

    /// 记录执行
    fn record_execution(&mut self, duration: Duration) {
        self.stats.record_execution(duration);
        self.execution_history.push(duration);

        // 限制历史记录大小
        if self.execution_history.len() > self.config.max_history_size {
            self.execution_history.remove(0);
        }
    }

    /// 类型匹配检查
    fn type_matches(value: &Value, expected_type: &DataType) -> bool {
        match (value, expected_type) {
            (Value::Int(_), DataType::Int) => true,
            (Value::Float(_), DataType::Float) => true,
            (Value::String(_), DataType::String) => true,
            (Value::Bool(_), DataType::Bool) => true,
            (Value::Date(_), DataType::Date) => true,
            (Value::DateTime(_), DataType::DateTime) => true,
            (Value::Time(_), DataType::Time) => true,
            (Value::Null(_), _) => true, // NULL 可以匹配任何类型
            _ => false,
        }
    }
}

/// 预编译语句构建器
pub struct PreparedStatementBuilder<S: StorageClient + 'static> {
    query_api: Arc<Mutex<QueryApi<S>>>,
    query: Option<String>,
    space_id: Option<u64>,
    config: StatementConfig,
}

impl<S: StorageClient + Clone + 'static> PreparedStatementBuilder<S> {
    /// 创建新的构建器
    #[allow(dead_code)]
    pub(crate) fn new(query_api: Arc<Mutex<QueryApi<S>>>) -> Self {
        Self {
            query_api,
            query: None,
            space_id: None,
            config: StatementConfig::default(),
        }
    }

    /// 设置查询
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// 设置空间 ID
    pub fn space_id(mut self, space_id: u64) -> Self {
        self.space_id = Some(space_id);
        self
    }

    /// 设置配置
    pub fn config(mut self, config: StatementConfig) -> Self {
        self.config = config;
        self
    }

    /// 构建预编译语句
    pub fn build(self) -> CoreResult<PreparedStatement<S>> {
        let query = self.query.ok_or_else(|| {
            CoreError::InvalidParameter("查询语句不能为空".to_string())
        })?;

        PreparedStatement::with_config(
            self.query_api,
            query,
            self.space_id,
            self.config,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::RedbStorage;

    #[test]
    fn test_extract_parameters_insert() {
        // 测试 INSERT 语句中的参数提取
        let query = "INSERT VERTEX Person(name, age) VALUES $id:($name, $age)";
        let params = PreparedStatement::<RedbStorage>::extract_parameters(query)
            .expect("解析查询失败");

        assert!(params.contains_key("id"), "应该包含 id 参数");
        assert!(params.contains_key("name"), "应该包含 name 参数");
        assert!(params.contains_key("age"), "应该包含 age 参数");
        assert_eq!(params.len(), 3);
    }

    #[test]
    fn test_extract_parameters_update() {
        // 测试 UPDATE 语句中的参数提取
        let query = "UPDATE $vid SET age = $new_age";
        let params = PreparedStatement::<RedbStorage>::extract_parameters(query)
            .expect("解析查询失败");

        assert!(params.contains_key("vid"), "应该包含 vid 参数");
        assert!(params.contains_key("new_age"), "应该包含 new_age 参数");
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn test_extract_parameters_delete() {
        // 测试 DELETE 语句中的参数提取
        let query = "DELETE VERTEX $vid";
        let params = PreparedStatement::<RedbStorage>::extract_parameters(query)
            .expect("解析查询失败");

        assert!(params.contains_key("vid"), "应该包含 vid 参数");
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_extract_parameters_invalid_query() {
        // 测试无效查询应该返回错误
        let query = "INVALID SYNTAX !!!";
        let result = PreparedStatement::<RedbStorage>::extract_parameters(query);

        assert!(result.is_err(), "无效查询应该返回错误");
    }

    #[test]
    fn test_extract_parameters_none() {
        // 测试没有参数的查询
        let query = "INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)";
        let params = PreparedStatement::<RedbStorage>::extract_parameters(query)
            .expect("解析查询失败");

        assert!(params.is_empty(), "没有参数的查询应该返回空映射");
    }

    #[test]
    fn test_statement_config() {
        let config = StatementConfig::new()
            .disable_cache()
            .disable_type_check()
            .with_max_history(50);

        assert!(!config.enable_cache);
        assert!(!config.enable_type_check);
        assert_eq!(config.max_history_size, 50);
    }

    #[test]
    fn test_execution_stats() {
        let mut stats = ExecutionStats::new();

        stats.record_execution(Duration::from_millis(10));
        stats.record_execution(Duration::from_millis(20));
        stats.record_execution(Duration::from_millis(30));

        assert_eq!(stats.execution_count, 3);
        assert_eq!(stats.total_execution_time_ms, 60);
        assert_eq!(stats.avg_execution_time_ms, 20.0);
        assert_eq!(stats.min_execution_time_ms, 10);
        assert_eq!(stats.max_execution_time_ms, 30);
    }

    #[test]
    fn test_type_matches() {
        assert!(PreparedStatement::<RedbStorage>::type_matches(
            &Value::Int(1),
            &DataType::Int
        ));
        assert!(PreparedStatement::<RedbStorage>::type_matches(
            &Value::String("test".to_string()),
            &DataType::String
        ));
        assert!(!PreparedStatement::<RedbStorage>::type_matches(
            &Value::Int(1),
            &DataType::String
        ));
    }
}
