//! 简化的访问者工厂系统
//!
//! 这个模块提供了一个更简单、更高效的访问者创建和管理机制

use crate::core::visitor::core::{
    DefaultVisitorState, VisitorConfig, VisitorContext, VisitorCore, VisitorError, VisitorResult,
    VisitorState,
};
use std::collections::HashMap;

/// 简化的访问者工厂trait
pub trait VisitorFactory: std::fmt::Debug + Send + Sync {
    /// 创建访问者实例
    fn create(&self, config: VisitorConfig) -> VisitorResult<Box<dyn VisitorCore<Result = ()>>>;

    /// 获取工厂名称
    fn name(&self) -> &str;

    /// 获取描述
    fn description(&self) -> &str {
        "访问者工厂"
    }
}

/// 简化的访问者注册表
#[derive(Debug)]
pub struct VisitorRegistry {
    factories: HashMap<String, Box<dyn VisitorFactory>>,
    default_config: VisitorConfig,
}

impl VisitorRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            default_config: VisitorConfig::default(),
        }
    }

    /// 创建带默认配置的注册表
    pub fn with_config(default_config: VisitorConfig) -> Self {
        Self {
            factories: HashMap::new(),
            default_config,
        }
    }

    /// 注册工厂
    pub fn register<F>(&mut self, factory: F) -> VisitorResult<()>
    where
        F: VisitorFactory + 'static,
    {
        let name = factory.name().to_string();
        if self.factories.contains_key(&name) {
            return Err(VisitorError::Validation(format!("工厂 {} 已经注册", name)));
        }

        self.factories.insert(name, Box::new(factory));
        Ok(())
    }

    /// 创建访问者
    pub fn create(&self, name: &str) -> VisitorResult<Box<dyn VisitorCore<Result = ()>>> {
        let factory = self
            .factories
            .get(name)
            .ok_or_else(|| VisitorError::Validation(format!("工厂 {} 未找到", name)))?;

        factory.create(self.default_config.clone())
    }

    /// 创建带配置的访问者
    pub fn create_with_config(
        &self,
        name: &str,
        config: VisitorConfig,
    ) -> VisitorResult<Box<dyn VisitorCore<Result = ()>>> {
        let factory = self
            .factories
            .get(name)
            .ok_or_else(|| VisitorError::Validation(format!("工厂 {} 未找到", name)))?;

        factory.create(config)
    }

    /// 获取工厂列表
    pub fn list_factories(&self) -> Vec<&str> {
        self.factories.keys().map(|s| s.as_str()).collect()
    }

    /// 检查工厂是否存在
    pub fn has_factory(&self, name: &str) -> bool {
        self.factories.contains_key(name)
    }

    /// 获取默认配置
    pub fn default_config(&self) -> &VisitorConfig {
        &self.default_config
    }

    /// 设置默认配置
    pub fn set_default_config(&mut self, config: VisitorConfig) {
        self.default_config = config;
    }
}

impl Default for VisitorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 通用访问者工厂实现
pub struct GenericVisitorFactory<T, F> {
    name: String,
    description: String,
    constructor: F,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, F> std::fmt::Debug for GenericVisitorFactory<T, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericVisitorFactory")
            .field("name", &self.name)
            .field("description", &self.description)
            .finish()
    }
}

impl<T, F> GenericVisitorFactory<T, F> {
    /// 创建新的通用工厂
    pub fn new(name: String, constructor: F) -> Self {
        Self {
            name,
            description: "通用访问者工厂".to_string(),
            constructor,
            _phantom: std::marker::PhantomData,
        }
    }

    /// 设置描述
    pub fn with_description(mut self, description: String) -> Self {
        self.description = description;
        self
    }
}

impl<T, F> VisitorFactory for GenericVisitorFactory<T, F>
where
    T: VisitorCore<Result = ()> + Send + Sync + 'static,
    F: Fn(VisitorConfig) -> T + Send + Sync + 'static,
{
    fn create(&self, config: VisitorConfig) -> VisitorResult<Box<dyn VisitorCore<Result = ()>>> {
        let visitor = (self.constructor)(config);
        Ok(Box::new(visitor))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// 访问者构建器 - 简化版本
#[derive(Debug)]
pub struct VisitorBuilder {
    registry: VisitorRegistry,
    factory_name: Option<String>,
    config: VisitorConfig,
}

impl VisitorBuilder {
    /// 创建新的构建器
    pub fn new() -> Self {
        Self {
            registry: VisitorRegistry::new(),
            factory_name: None,
            config: VisitorConfig::default(),
        }
    }

    /// 使用现有注册表创建构建器
    pub fn with_registry(registry: VisitorRegistry) -> Self {
        Self {
            registry,
            factory_name: None,
            config: VisitorConfig::default(),
        }
    }

    /// 注册工厂
    pub fn register<F>(mut self, factory: F) -> VisitorResult<Self>
    where
        F: VisitorFactory + 'static,
    {
        self.registry.register(factory)?;
        Ok(self)
    }

    /// 设置工厂名称
    pub fn factory(mut self, name: String) -> Self {
        self.factory_name = Some(name);
        self
    }

    /// 设置配置
    pub fn config(mut self, config: VisitorConfig) -> Self {
        self.config = config;
        self
    }

    /// 设置最大深度
    pub fn max_depth(mut self, max_depth: usize) -> Self {
        self.config.max_depth = max_depth;
        self
    }

    /// 设置严格模式
    pub fn strict_mode(mut self, strict: bool) -> Self {
        self.config.strict_mode = strict;
        self
    }

    /// 构建访问者
    pub fn build(self) -> VisitorResult<Box<dyn VisitorCore<Result = ()>>> {
        let factory_name = self
            .factory_name
            .ok_or_else(|| VisitorError::Validation("工厂名称未设置".to_string()))?;

        self.registry.create_with_config(&factory_name, self.config)
    }
}

impl Default for VisitorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷函数：创建简单的访问者工厂
pub fn simple_factory<T, F>(name: &str, constructor: F) -> GenericVisitorFactory<T, F>
where
    T: VisitorCore<Result = ()> + Send + Sync + 'static,
    F: Fn(VisitorConfig) -> T + Send + Sync + 'static,
{
    GenericVisitorFactory::new(name.to_string(), constructor)
}

/// 便捷函数：创建带配置的访问者
pub fn create_visitor<T, F>(
    name: &str,
    config: VisitorConfig,
    constructor: F,
) -> VisitorResult<Box<dyn VisitorCore<Result = ()>>>
where
    T: VisitorCore<Result = ()> + Send + Sync + 'static,
    F: FnOnce(VisitorConfig) -> T,
{
    let visitor = constructor(config);
    Ok(Box::new(visitor))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;
    use crate::core::visitor::core::ValueVisitor;

    #[derive(Debug)]
    struct TestVisitor {
        count: usize,
        context: VisitorContext,
        state: DefaultVisitorState,
    }

    impl TestVisitor {
        fn new(config: VisitorConfig) -> Self {
            Self {
                count: 0,
                context: VisitorContext::new(config),
                state: DefaultVisitorState::new(),
            }
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

        fn visit_date(&mut self, _value: &crate::core::value::DateValue) -> Self::Result {
            self.count += 1;
        }

        fn visit_time(&mut self, _value: &crate::core::value::TimeValue) -> Self::Result {
            self.count += 1;
        }

        fn visit_datetime(&mut self, _value: &crate::core::value::DateTimeValue) -> Self::Result {
            self.count += 1;
        }

        fn visit_vertex(&mut self, _value: &crate::core::vertex_edge_path::Vertex) -> Self::Result {
            self.count += 1;
        }

        fn visit_edge(&mut self, _value: &crate::core::vertex_edge_path::Edge) -> Self::Result {
            self.count += 1;
        }

        fn visit_path(&mut self, _value: &crate::core::vertex_edge_path::Path) -> Self::Result {
            self.count += 1;
        }

        fn visit_list(&mut self, _value: &[Value]) -> Self::Result {
            self.count += 1;
        }

        fn visit_map(&mut self, _value: &std::collections::HashMap<String, Value>) -> Self::Result {
            self.count += 1;
        }

        fn visit_set(&mut self, _value: &std::collections::HashSet<Value>) -> Self::Result {
            self.count += 1;
        }

        fn visit_geography(&mut self, _value: &crate::core::value::GeographyValue) -> Self::Result {
            self.count += 1;
        }

        fn visit_duration(&mut self, _value: &crate::core::value::DurationValue) -> Self::Result {
            self.count += 1;
        }

        fn visit_dataset(&mut self, _value: &crate::core::value::DataSet) -> Self::Result {
            self.count += 1;
        }

        fn visit_null(&mut self, _null_type: &crate::core::value::NullType) -> Self::Result {
            self.count += 1;
        }

        fn visit_empty(&mut self) -> Self::Result {
            self.count += 1;
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

    #[test]
    fn test_registry() {
        let mut registry = VisitorRegistry::new();

        // 注册工厂
        let factory = simple_factory("test_factory", TestVisitor::new);
        assert!(registry.register(factory).is_ok());

        // 创建访问者
        let visitor = registry.create("test_factory");
        assert!(visitor.is_ok());

        // 检查工厂列表
        let factories = registry.list_factories();
        assert!(factories.contains(&"test_factory"));
    }

    #[test]
    fn test_visitor_builder() {
        let factory = simple_factory("builder_test", TestVisitor::new);

        let visitor = VisitorBuilder::new()
            .register(factory)
            .unwrap()
            .factory("builder_test".to_string())
            .max_depth(50)
            .strict_mode(true)
            .build();

        assert!(visitor.is_ok());
    }

    #[test]
    fn test_create_visitor_convenience() {
        let config = VisitorConfig::new().with_max_depth(25);
        let visitor = create_visitor("test_visitor", config, TestVisitor::new);

        assert!(visitor.is_ok());
    }
}
