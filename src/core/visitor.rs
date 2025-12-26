//! 访问者模式定义
//!
//! 这个模块提供了统一的Visitor trait，合并了原有的多层访问者结构
//! 包括core、query、AST、expression四层访问者

use crate::core::error::DBError;
use crate::core::value::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
};
use crate::core::vertex_edge_path::{Edge, Path, Vertex};
use std::collections::HashMap;

/// 统一的访问者错误类型
pub type VisitorError = DBError;
pub type VisitorResult<T> = Result<T, VisitorError>;

/// 访问者状态
///
/// 替代原有的多层状态管理，提供统一的状态管理机制
#[derive(Debug, Clone)]
pub struct VisitorState {
    /// 是否继续访问
    pub continue_visiting: bool,
    /// 访问深度
    pub depth: usize,
    /// 访问计数
    pub visit_count: usize,
    /// 最大深度限制
    pub max_depth: Option<usize>,
    /// 自定义状态数据
    pub custom_data: HashMap<String, Value>,
}

impl VisitorState {
    /// 创建新的访问者状态
    pub fn new() -> Self {
        Self {
            continue_visiting: true,
            depth: 0,
            visit_count: 0,
            max_depth: None,
            custom_data: HashMap::new(),
        }
    }

    /// 创建带最大深度的访问者状态
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            continue_visiting: true,
            depth: 0,
            visit_count: 0,
            max_depth: Some(max_depth),
            custom_data: HashMap::new(),
        }
    }

    /// 检查是否应该继续访问
    pub fn should_continue(&self) -> bool {
        self.continue_visiting && self.max_depth.map_or(true, |max| self.depth <= max)
    }

    /// 停止访问
    pub fn stop(&mut self) {
        self.continue_visiting = false;
    }

    /// 重置状态
    pub fn reset(&mut self) {
        self.continue_visiting = true;
        self.depth = 0;
        self.visit_count = 0;
        self.custom_data.clear();
    }

    /// 增加访问深度
    pub fn inc_depth(&mut self) {
        self.depth += 1;
    }

    /// 减少访问深度
    pub fn dec_depth(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    /// 增加访问计数
    pub fn inc_visit_count(&mut self) {
        self.visit_count += 1;
    }

    /// 获取访问深度
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// 获取访问计数
    pub fn visit_count(&self) -> usize {
        self.visit_count
    }

    /// 获取自定义数据
    pub fn get_custom_data(&self, key: &str) -> Option<&Value> {
        self.custom_data.get(key)
    }

    /// 设置自定义数据
    pub fn set_custom_data(&mut self, key: String, value: Value) {
        self.custom_data.insert(key, value);
    }

    /// 移除自定义数据
    pub fn remove_custom_data(&mut self, key: &str) -> Option<Value> {
        self.custom_data.remove(key)
    }
}

impl Default for VisitorState {
    fn default() -> Self {
        Self::new()
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

/// 统一访问者trait
///
/// 这个trait合并了原有的多层访问者结构，提供统一的访问者接口
///
/// 设计原则：
/// - 泛型T：访问目标类型
/// - 类型参数Result：访问结果类型
/// - 提供默认实现的方法：pre_visit、post_visit、reset、stop
/// - 必须实现的方法：visit、state、state_mut
pub trait Visitor<T>: std::fmt::Debug + Send + Sync {
    /// 访问者结果类型
    type Result;

    /// 访问目标对象
    fn visit(&mut self, target: &T) -> Self::Result;

    /// 预访问钩子 - 在访问开始前调用
    ///
    /// 默认实现：返回Ok(())
    fn pre_visit(&mut self) -> VisitorResult<()> {
        Ok(())
    }

    /// 后访问钩子 - 在访问结束后调用
    ///
    /// 默认实现：返回Ok(())
    fn post_visit(&mut self) -> VisitorResult<()> {
        Ok(())
    }

    /// 获取访问者状态
    fn state(&self) -> &VisitorState;

    /// 获取可变访问者状态
    fn state_mut(&mut self) -> &mut VisitorState;

    /// 重置访问者状态
    fn reset(&mut self) {
        self.state_mut().reset();
    }

    /// 检查是否应该继续访问
    fn should_continue(&self) -> bool {
        self.state().should_continue()
    }

    /// 停止访问
    fn stop(&mut self) {
        self.state_mut().stop();
    }

    /// 获取访问者配置
    ///
    /// 默认实现：返回None，子类可以覆盖
    fn config(&self) -> Option<&VisitorConfig> {
        None
    }

    /// 获取可变访问者配置
    ///
    /// 默认实现：返回None，子类可以覆盖
    fn config_mut(&mut self) -> Option<&mut VisitorConfig> {
        None
    }
}

/// Value访问者trait
///
/// 用于访问Value类型的各个变体
pub trait ValueVisitor: Visitor<Value> {
    /// 访问布尔值
    fn visit_bool(&mut self, value: bool) -> Self::Result;

    /// 访问整数
    fn visit_int(&mut self, value: i64) -> Self::Result;

    /// 访问浮点数
    fn visit_float(&mut self, value: f64) -> Self::Result;

    /// 访问字符串
    fn visit_string(&mut self, value: &str) -> Self::Result;

    /// 访问日期
    fn visit_date(&mut self, value: &DateValue) -> Self::Result;

    /// 访问时间
    fn visit_time(&mut self, value: &TimeValue) -> Self::Result;

    /// 访问日期时间
    fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result;

    /// 访问顶点
    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result;

    /// 访问边
    fn visit_edge(&mut self, value: &Edge) -> Self::Result;

    /// 访问路径
    fn visit_path(&mut self, value: &Path) -> Self::Result;

    /// 访问列表
    fn visit_list(&mut self, value: &[Value]) -> Self::Result;

    /// 访问映射
    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result;

    /// 访问集合
    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result;

    /// 访问地理数据
    fn visit_geography(&mut self, value: &GeographyValue) -> Self::Result;

    /// 访问持续时间
    fn visit_duration(&mut self, value: &DurationValue) -> Self::Result;

    /// 访问数据集
    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result;

    /// 访问空值
    fn visit_null(&mut self, null_type: &NullType) -> Self::Result;

    /// 访问空值
    fn visit_empty(&mut self) -> Self::Result;
}

/// Value访问者接受器trait
///
/// 为Value类型提供接受访问者的能力
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

/// 默认访问者实现
///
/// 提供基础的访问者功能，可以被继承和扩展
#[derive(Debug)]
pub struct DefaultVisitor<T: std::fmt::Debug> {
    state: VisitorState,
    config: VisitorConfig,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: std::fmt::Debug> DefaultVisitor<T> {
    /// 创建新的默认访问者
    pub fn new() -> Self {
        Self {
            state: VisitorState::new(),
            config: VisitorConfig::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// 创建带配置的默认访问者
    pub fn with_config(config: VisitorConfig) -> Self {
        Self {
            state: VisitorState::with_max_depth(config.max_depth),
            config,
            _phantom: std::marker::PhantomData,
        }
    }

    /// 创建带初始深度的默认访问者
    pub fn with_depth(depth: usize) -> Self {
        Self {
            state: VisitorState::with_max_depth(depth),
            config: VisitorConfig::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: std::fmt::Debug + Send + Sync> Default for DefaultVisitor<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: std::fmt::Debug + Send + Sync> Visitor<T> for DefaultVisitor<T> {
    type Result = ();

    fn visit(&mut self, _target: &T) -> Self::Result {}

    fn state(&self) -> &VisitorState {
        &self.state
    }

    fn state_mut(&mut self) -> &mut VisitorState {
        &mut self.state
    }

    fn config(&self) -> Option<&VisitorConfig> {
        Some(&self.config)
    }

    fn config_mut(&mut self) -> Option<&mut VisitorConfig> {
        Some(&mut self.config)
    }
}

/// 访问者构建器trait
///
/// 提供统一的访问者构建接口
pub trait VisitorBuilder: Sized {
    /// 访问者类型
    type Output;

    /// 创建新的构建器
    fn new() -> Self;

    /// 设置最大深度
    fn with_max_depth(self, depth: usize) -> Self;

    /// 设置配置
    fn with_config(self, config: VisitorConfig) -> Self;

    /// 构建访问者
    fn build(self) -> Self::Output;
}

/// 默认访问者构建器实现
///
/// 提供通用的构建器实现
#[derive(Debug, Clone)]
pub struct DefaultVisitorBuilder<T: std::fmt::Debug> {
    config: VisitorConfig,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: std::fmt::Debug> DefaultVisitorBuilder<T> {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            config: VisitorConfig::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// 设置最大深度
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.config.max_depth = depth;
        self
    }

    /// 设置配置
    pub fn with_config(mut self, config: VisitorConfig) -> Self {
        self.config = config;
        self
    }

    /// 构建默认访问者
    pub fn build(self) -> DefaultVisitor<T> {
        DefaultVisitor::with_config(self.config)
    }
}

impl<T: std::fmt::Debug> Default for DefaultVisitorBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: std::fmt::Debug> VisitorBuilder for DefaultVisitorBuilder<T> {
    type Output = DefaultVisitor<T>;

    fn new() -> Self {
        Self::new()
    }

    fn with_max_depth(self, depth: usize) -> Self {
        self.with_max_depth(depth)
    }

    fn with_config(self, config: VisitorConfig) -> Self {
        self.with_config(config)
    }

    fn build(self) -> Self::Output {
        self.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visitor_state() {
        let mut state = VisitorState::new();

        assert_eq!(state.should_continue(), true);
        assert_eq!(state.depth(), 0);
        assert_eq!(state.visit_count(), 0);

        state.inc_depth();
        state.inc_visit_count();
        state.set_custom_data("test".to_string(), Value::Int(42));

        assert_eq!(state.depth(), 1);
        assert_eq!(state.visit_count(), 1);
        assert_eq!(state.get_custom_data("test"), Some(&Value::Int(42)));

        state.stop();
        assert_eq!(state.should_continue(), false);

        state.reset();
        assert_eq!(state.should_continue(), true);
        assert_eq!(state.depth(), 0);
        assert_eq!(state.visit_count(), 0);
        assert_eq!(state.get_custom_data("test"), None);
    }

    #[test]
    fn test_visitor_state_with_max_depth() {
        let state = VisitorState::with_max_depth(3);
        assert_eq!(state.should_continue(), true);

        let mut state = state;
        state.depth = 3;
        assert_eq!(state.should_continue(), false);

        state.depth = 2;
        assert_eq!(state.should_continue(), true);
    }

    #[test]
    fn test_visitor_config() {
        let config = VisitorConfig::new()
            .with_max_depth(50)
            .with_cache(false)
            .with_strict_mode(true);

        assert_eq!(config.max_depth, 50);
        assert_eq!(config.enable_cache, false);
        assert_eq!(config.strict_mode, true);
    }

    #[test]
    fn test_default_visitor() {
        let mut visitor = DefaultVisitor::<()>::new();

        assert_eq!(visitor.should_continue(), true);
        assert_eq!(visitor.state().depth(), 0);

        visitor.visit(&());
        assert_eq!(visitor.state().visit_count(), 0);

        visitor.stop();
        assert_eq!(visitor.should_continue(), false);
    }

    #[test]
    fn test_visitor_builder() {
        let builder = DefaultVisitorBuilder::<()>::new()
            .with_max_depth(10)
            .with_config(VisitorConfig::new().with_strict_mode(true));

        let visitor = builder.build();
        assert_eq!(visitor.config().unwrap().max_depth, 10);
        assert_eq!(visitor.config().unwrap().strict_mode, true);
    }
}
