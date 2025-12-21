//! 访问者模式核心定义
//!
//! 这个模块提供了访问者模式的核心 trait 和基础实现

use crate::core::error::DBError;
use crate::core::value::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
};
use crate::core::vertex_edge_path::{Edge, Path, Vertex};
use std::collections::HashMap;

/// Value 访问者 trait
///
/// 这个 trait 定义了访问者模式的核心接口，允许对 Value 类型进行操作而不修改其结构
///
/// # 示例
/// ```
/// use graphdb::core::visitor::{ValueVisitor, ValueAcceptor};
///
/// struct MyVisitor {
///     count: usize,
/// }
///
/// impl ValueVisitor for MyVisitor {
///     type Result = ();
///     
///     fn visit_int(&mut self, _value: i64) -> Self::Result {
///         self.count += 1;
///     }
///     
///     fn visit_string(&mut self, _value: &str) -> Self::Result {
///         self.count += 1;
///     }
///     
///     // ... 其他 visit 方法
/// }
///
/// let value = Value::Int(42);
/// let mut visitor = MyVisitor { count: 0 };
/// value.accept(&mut visitor);
/// assert_eq!(visitor.count, 1);
/// ```
pub trait ValueVisitor {
    type Result;

    fn visit_bool(&mut self, value: bool) -> Self::Result;
    fn visit_int(&mut self, value: i64) -> Self::Result;
    fn visit_float(&mut self, value: f64) -> Self::Result;
    fn visit_string(&mut self, value: &str) -> Self::Result;
    fn visit_date(&mut self, value: &DateValue) -> Self::Result;
    fn visit_time(&mut self, value: &TimeValue) -> Self::Result;
    fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result;
    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result;
    fn visit_edge(&mut self, value: &Edge) -> Self::Result;
    fn visit_path(&mut self, value: &Path) -> Self::Result;
    fn visit_list(&mut self, value: &[Value]) -> Self::Result;
    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result;
    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result;
    fn visit_geography(&mut self, value: &GeographyValue) -> Self::Result;
    fn visit_duration(&mut self, value: &DurationValue) -> Self::Result;
    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result;
    fn visit_null(&mut self, null_type: &NullType) -> Self::Result;
    fn visit_empty(&mut self) -> Self::Result;
}

/// Value 访问者接受器 trait
///
/// 这个 trait 为 Value 类型提供了接受访问者的能力，实现了访问者模式的"可访问性"部分
pub trait ValueAcceptor {
    /// 接受访问者进行访问
    fn accept<V: ValueVisitor>(&self, visitor: &mut V) -> V::Result;
}

impl ValueAcceptor for Value {
    fn accept<V: ValueVisitor>(&self, visitor: &mut V) -> V::Result {
        match self {
            Value::Bool(b) => visitor.visit_bool(*b),
            Value::Int(i) => visitor.visit_int(*i),
            Value::Float(f) => visitor.visit_float(*f),
            Value::String(s) => visitor.visit_string(s),
            Value::Date(d) => visitor.visit_date(d),
            Value::Time(t) => visitor.visit_time(t),
            Value::DateTime(dt) => visitor.visit_datetime(dt),
            Value::Vertex(v) => visitor.visit_vertex(v),
            Value::Edge(e) => visitor.visit_edge(e),
            Value::Path(p) => visitor.visit_path(p),
            Value::List(l) => visitor.visit_list(l),
            Value::Map(m) => visitor.visit_map(m),
            Value::Set(s) => visitor.visit_set(s),
            Value::Geography(g) => visitor.visit_geography(g),
            Value::Duration(d) => visitor.visit_duration(d),
            Value::DataSet(ds) => visitor.visit_dataset(ds),
            Value::Null(nt) => visitor.visit_null(nt),
            Value::Empty => visitor.visit_empty(),
        }
    }
}

/// 统一的访问者错误类型（复用core层的错误系统）
pub type VisitorError = DBError;
pub type VisitorResult<T> = Result<T, VisitorError>;

/// 访问者核心trait - 所有访问者的基础
pub trait VisitorCore: std::fmt::Debug {
    /// 访问者结果类型
    type Result;

    /// 预访问钩子 - 在访问开始前调用
    fn pre_visit(&mut self) -> VisitorResult<()> {
        Ok(())
    }

    /// 后访问钩子 - 在访问结束后调用
    fn post_visit(&mut self) -> VisitorResult<()> {
        Ok(())
    }

    /// 获取访问者上下文
    fn context(&self) -> &VisitorContext;

    /// 获取可变访问者上下文
    fn context_mut(&mut self) -> &mut VisitorContext;

    /// 获取访问者状态
    fn state(&self) -> &dyn VisitorState;

    /// 获取可变访问者状态
    fn state_mut(&mut self) -> &mut dyn VisitorState;

    /// 重置访问者状态
    fn reset(&mut self) -> VisitorResult<()> {
        self.state_mut().reset();
        Ok(())
    }

    /// 检查是否应该继续访问
    fn should_continue(&self) -> bool {
        self.state().should_continue()
    }

    /// 停止访问
    fn stop(&mut self) {
        self.state_mut().stop();
    }
}

/// 访问者状态管理trait
pub trait VisitorState: std::fmt::Debug + Send + Sync {
    /// 重置状态
    fn reset(&mut self);

    /// 检查是否应该继续访问
    fn should_continue(&self) -> bool;

    /// 停止访问
    fn stop(&mut self);

    /// 获取访问深度
    fn depth(&self) -> usize;

    /// 设置访问深度
    fn set_depth(&mut self, depth: usize);

    /// 增加访问深度
    fn inc_depth(&mut self);

    /// 减少访问深度
    fn dec_depth(&mut self);

    /// 获取访问计数
    fn visit_count(&self) -> usize;

    /// 增加访问计数
    fn inc_visit_count(&mut self);

    /// 获取自定义状态数据
    fn get_custom_data(&self, key: &str) -> Option<&String>;

    /// 设置自定义状态数据
    fn set_custom_data(&mut self, key: String, value: String);

    /// 移除自定义状态数据
    fn remove_custom_data(&mut self, key: &str) -> Option<String>;
}

/// 默认访问者状态实现
#[derive(Debug, Clone)]
pub struct DefaultVisitorState {
    /// 是否继续访问
    continue_visiting: bool,
    /// 访问深度
    depth: usize,
    /// 访问计数
    visit_count: usize,
    /// 自定义状态数据
    custom_data: HashMap<String, String>,
}

impl DefaultVisitorState {
    /// 创建新的默认状态
    pub fn new() -> Self {
        Self {
            continue_visiting: true,
            depth: 0,
            visit_count: 0,
            custom_data: HashMap::new(),
        }
    }

    /// 创建带初始深度的状态
    pub fn with_depth(depth: usize) -> Self {
        Self {
            continue_visiting: true,
            depth,
            visit_count: 0,
            custom_data: HashMap::new(),
        }
    }
}

impl Default for DefaultVisitorState {
    fn default() -> Self {
        Self::new()
    }
}

impl VisitorState for DefaultVisitorState {
    fn reset(&mut self) {
        self.continue_visiting = true;
        self.depth = 0;
        self.visit_count = 0;
        self.custom_data.clear();
    }

    fn should_continue(&self) -> bool {
        self.continue_visiting
    }

    fn stop(&mut self) {
        self.continue_visiting = false;
    }

    fn depth(&self) -> usize {
        self.depth
    }

    fn set_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    fn inc_depth(&mut self) {
        self.depth += 1;
    }

    fn dec_depth(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    fn visit_count(&self) -> usize {
        self.visit_count
    }

    fn inc_visit_count(&mut self) {
        self.visit_count += 1;
    }

    fn get_custom_data(&self, key: &str) -> Option<&String> {
        self.custom_data.get(key)
    }

    fn set_custom_data(&mut self, key: String, value: String) {
        self.custom_data.insert(key, value);
    }

    fn remove_custom_data(&mut self, key: &str) -> Option<String> {
        self.custom_data.remove(key)
    }
}

/// 访问者上下文 - 包含配置、缓存和错误收集器
#[derive(Debug, Clone)]
pub struct VisitorContext {
    /// 访问者配置
    config: VisitorConfig,
    /// 自定义数据
    custom_data: HashMap<String, String>,
}

impl VisitorContext {
    /// 创建新的访问者上下文
    pub fn new(config: VisitorConfig) -> Self {
        Self {
            config,
            custom_data: HashMap::new(),
        }
    }

    /// 获取配置
    pub fn config(&self) -> &VisitorConfig {
        &self.config
    }

    /// 获取可变配置
    pub fn config_mut(&mut self) -> &mut VisitorConfig {
        &mut self.config
    }

    /// 获取自定义数据
    pub fn get_custom_data(&self, key: &str) -> Option<&String> {
        self.custom_data.get(key)
    }

    /// 设置自定义数据
    pub fn set_custom_data(&mut self, key: String, value: String) {
        self.custom_data.insert(key, value);
    }

    /// 移除自定义数据
    pub fn remove_custom_data(&mut self, key: &str) -> Option<String> {
        self.custom_data.remove(key)
    }
}

/// 访问者配置
#[derive(Debug, Clone)]
pub struct VisitorConfig {
    /// 最大访问深度
    pub max_depth: usize,
    /// 是否启用缓存
    pub enable_cache: bool,
    /// 是否收集错误
    pub collect_errors: bool,
    /// 是否启用性能统计
    pub enable_performance_stats: bool,
    /// 是否严格模式
    pub strict_mode: bool,
    /// 自定义配置
    pub custom_config: HashMap<String, String>,
}

impl Default for VisitorConfig {
    fn default() -> Self {
        Self {
            max_depth: 100,
            enable_cache: true,
            collect_errors: true,
            enable_performance_stats: false,
            strict_mode: false,
            custom_config: HashMap::new(),
        }
    }
}

impl VisitorConfig {
    /// 创建新的配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置最大深度
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// 设置是否启用缓存
    pub fn with_cache(mut self, enable_cache: bool) -> Self {
        self.enable_cache = enable_cache;
        self
    }

    /// 设置是否收集错误
    pub fn with_error_collection(mut self, collect_errors: bool) -> Self {
        self.collect_errors = collect_errors;
        self
    }

    /// 设置是否启用性能统计
    pub fn with_performance_stats(mut self, enable_performance_stats: bool) -> Self {
        self.enable_performance_stats = enable_performance_stats;
        self
    }

    /// 设置是否严格模式
    pub fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.strict_mode = strict_mode;
        self
    }

    /// 添加自定义配置
    pub fn with_custom_config(mut self, key: String, value: String) -> Self {
        self.custom_config.insert(key, value);
        self
    }

    /// 获取自定义配置
    pub fn get_custom_config(&self, key: &str) -> Option<&String> {
        self.custom_config.get(key)
    }
}

/// 访问者模式辅助工具
pub mod utils {
    use super::*;

    /// 递归访问辅助函数，避免栈溢出
    pub fn visit_recursive<V: ValueVisitor>(
        value: &Value,
        visitor: &mut V,
        depth: usize,
        max_depth: usize,
    ) -> Result<V::Result, RecursionError> {
        if depth > max_depth {
            return Err(RecursionError::MaxDepthExceeded);
        }

        match value {
            Value::Bool(b) => Ok(visitor.visit_bool(*b)),
            Value::Int(i) => Ok(visitor.visit_int(*i)),
            Value::Float(f) => Ok(visitor.visit_float(*f)),
            Value::String(s) => Ok(visitor.visit_string(s)),
            Value::Date(d) => Ok(visitor.visit_date(d)),
            Value::Time(t) => Ok(visitor.visit_time(t)),
            Value::DateTime(dt) => Ok(visitor.visit_datetime(dt)),
            Value::Vertex(v) => Ok(visitor.visit_vertex(v)),
            Value::Edge(e) => Ok(visitor.visit_edge(e)),
            Value::Path(p) => Ok(visitor.visit_path(p)),
            Value::List(l) => {
                // For lists, we can't call visitor.visit_list directly on the results
                // We need to transform the list first and then call visit_list
                // This is handled by the specific visitor implementation
                Ok(visitor.visit_list(l))
            }
            Value::Map(m) => Ok(visitor.visit_map(m)),
            Value::Set(s) => Ok(visitor.visit_set(s)),
            Value::Geography(g) => Ok(visitor.visit_geography(g)),
            Value::Duration(d) => Ok(visitor.visit_duration(d)),
            Value::DataSet(ds) => Ok(visitor.visit_dataset(ds)),
            Value::Null(nt) => Ok(visitor.visit_null(nt)),
            Value::Empty => Ok(visitor.visit_empty()),
        }
    }

    /// 递归错误类型
    #[derive(Debug, thiserror::Error)]
    pub enum RecursionError {
        #[error("递归深度超过最大限制")]
        MaxDepthExceeded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_value_acceptor() {
        struct CountVisitor {
            count: usize,
        }

        impl ValueVisitor for CountVisitor {
            type Result = ();

            fn visit_int(&mut self, _value: i64) -> Self::Result {
                self.count += 1;
            }

            fn visit_string(&mut self, _value: &str) -> Self::Result {
                self.count += 1;
            }

            fn visit_bool(&mut self, _value: bool) -> Self::Result {
                self.count += 1;
            }

            fn visit_float(&mut self, _value: f64) -> Self::Result {
                self.count += 1;
            }

            fn visit_date(&mut self, _value: &DateValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_time(&mut self, _value: &TimeValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_datetime(&mut self, _value: &DateTimeValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_vertex(&mut self, _value: &Vertex) -> Self::Result {
                self.count += 1;
            }

            fn visit_edge(&mut self, _value: &Edge) -> Self::Result {
                self.count += 1;
            }

            fn visit_path(&mut self, _value: &Path) -> Self::Result {
                self.count += 1;
            }

            fn visit_list(&mut self, _value: &[Value]) -> Self::Result {
                self.count += 1;
            }

            fn visit_map(&mut self, _value: &HashMap<String, Value>) -> Self::Result {
                self.count += 1;
            }

            fn visit_set(&mut self, _value: &std::collections::HashSet<Value>) -> Self::Result {
                self.count += 1;
            }

            fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_duration(&mut self, _value: &DurationValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_dataset(&mut self, _value: &DataSet) -> Self::Result {
                self.count += 1;
            }

            fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
                self.count += 1;
            }

            fn visit_empty(&mut self) -> Self::Result {
                self.count += 1;
            }
        }

        let value = Value::Int(42);
        let mut visitor = CountVisitor { count: 0 };
        value.accept(&mut visitor);
        assert_eq!(visitor.count, 1);
    }
}

#[cfg(test)]
mod core_tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_visitor_config() {
        let config = VisitorConfig::new()
            .with_max_depth(50)
            .with_cache(false)
            .with_strict_mode(true)
            .with_custom_config("test_key".to_string(), "test_value".to_string());

        assert_eq!(config.max_depth, 50);
        assert!(!config.enable_cache);
        assert!(config.strict_mode);
        assert_eq!(
            config.get_custom_config("test_key"),
            Some(&"test_value".to_string())
        );
    }

    #[test]
    fn test_default_visitor_state() {
        let mut state = DefaultVisitorState::new();

        assert!(state.should_continue());
        assert_eq!(state.depth(), 0);
        assert_eq!(state.visit_count(), 0);

        state.inc_depth();
        assert_eq!(state.depth(), 1);

        state.inc_visit_count();
        assert_eq!(state.visit_count(), 1);

        state.stop();
        assert!(!state.should_continue());

        state.reset();
        assert!(state.should_continue());
        assert_eq!(state.depth(), 0);
        assert_eq!(state.visit_count(), 0);
    }

    #[test]
    fn test_visitor_context() {
        let config = VisitorConfig::new();
        let mut context = VisitorContext::new(config);

        context.set_custom_data("test_key".to_string(), "test_value".to_string());
        assert_eq!(
            context.get_custom_data("test_key"),
            Some(&"test_value".to_string())
        );

        let removed = context.remove_custom_data("test_key");
        assert_eq!(removed, Some("test_value".to_string()));
        assert_eq!(context.get_custom_data("test_key"), None);
    }

    #[test]
    fn test_value_visitor_with_core() {
        struct TestVisitor {
            count: usize,
            context: VisitorContext,
            state: DefaultVisitorState,
        }

        impl std::fmt::Debug for TestVisitor {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "TestVisitor {{ count: {} }}", self.count)
            }
        }

        impl VisitorCore for TestVisitor {
            type Result = ();
            
            fn context(&self) -> &VisitorContext {
                &self.context
            }
            
            fn context_mut(&mut self) -> &mut VisitorContext {
                &mut self.context
            }
            
            fn state(&self) -> &dyn VisitorState {
                &self.state
            }
            
            fn state_mut(&mut self) -> &mut dyn VisitorState {
                &mut self.state
            }
        }

        impl ValueVisitor for TestVisitor {
            type Result = ();

            fn visit_int(&mut self, _value: i64) -> Self::Result {
                self.count += 1;
            }

            fn visit_string(&mut self, _value: &str) -> Self::Result {
                self.count += 1;
            }

            fn visit_bool(&mut self, _value: bool) -> Self::Result {
                self.count += 1;
            }

            fn visit_float(&mut self, _value: f64) -> Self::Result {
                self.count += 1;
            }

            fn visit_date(&mut self, _value: &DateValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_time(&mut self, _value: &TimeValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_datetime(&mut self, _value: &DateTimeValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_vertex(&mut self, _value: &Vertex) -> Self::Result {
                self.count += 1;
            }

            fn visit_edge(&mut self, _value: &Edge) -> Self::Result {
                self.count += 1;
            }

            fn visit_path(&mut self, _value: &Path) -> Self::Result {
                self.count += 1;
            }

            fn visit_list(&mut self, _value: &[Value]) -> Self::Result {
                self.count += 1;
            }

            fn visit_map(&mut self, _value: &HashMap<String, Value>) -> Self::Result {
                self.count += 1;
            }

            fn visit_set(&mut self, _value: &std::collections::HashSet<Value>) -> Self::Result {
                self.count += 1;
            }

            fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_duration(&mut self, _value: &DurationValue) -> Self::Result {
                self.count += 1;
            }

            fn visit_dataset(&mut self, _value: &DataSet) -> Self::Result {
                self.count += 1;
            }

            fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
                self.count += 1;
            }

            fn visit_empty(&mut self) -> Self::Result {
                self.count += 1;
            }
        }

        let mut visitor = TestVisitor {
            count: 0,
            context: VisitorContext::new(VisitorConfig::default()),
            state: DefaultVisitorState::new(),
        };
        let value = Value::Int(42);
    }
}
