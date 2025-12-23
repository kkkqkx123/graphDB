//! 访问者模式核心定义
//!
//! 这个模块提供了统一的访问者基础设施，支持零成本抽象

use crate::core::error::DBError;
use crate::core::value::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
};
use crate::core::vertex_edge_path::{Edge, Path, Vertex};
use std::collections::HashMap;

// 导入 visitor_state_enum 模块
pub use crate::core::visitor_state_enum;

/// 统一的访问者错误类型
pub type VisitorError = DBError;
pub type VisitorResult<T> = Result<T, VisitorError>;

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

/// 访问者核心trait - 所有访问者的基础
pub trait VisitorCore<T>: std::fmt::Debug {
    /// 访问者结果类型
    type Result;

    /// 访问目标对象
    fn visit(&mut self, target: &T) -> Self::Result;

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
    fn state(&self) -> &visitor_state_enum::VisitorStateEnum;

    /// 获取可变访问者状态
    fn state_mut(&mut self) -> &mut visitor_state_enum::VisitorStateEnum;

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

/// Value 访问者 trait - 用于访问Value类型的各个变体
pub trait ValueVisitor: VisitorCore<Value> {
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

/// Value 访问者接受器 trait - 为Value类型提供接受访问者的能力
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

/// 表达式访问者 trait - 用于访问Expression类型的各个变体
pub trait ExpressionVisitor: VisitorCore<crate::core::Expression> {
    fn visit_literal(&mut self, value: &crate::core::LiteralValue) -> Self::Result;
    fn visit_variable(&mut self, name: &str) -> Self::Result;
    fn visit_property(&mut self, object: &crate::core::Expression, property: &str) -> Self::Result;
    fn visit_binary(
        &mut self,
        left: &crate::core::Expression,
        op: &crate::core::BinaryOperator,
        right: &crate::core::Expression,
    ) -> Self::Result;
    fn visit_unary(
        &mut self,
        op: &crate::core::UnaryOperator,
        operand: &crate::core::Expression,
    ) -> Self::Result;
    fn visit_function(&mut self, name: &str, args: &[crate::core::Expression]) -> Self::Result;
    fn visit_aggregate(
        &mut self,
        func: &crate::core::AggregateFunction,
        arg: &crate::core::Expression,
        distinct: bool,
    ) -> Self::Result;
    fn visit_list(&mut self, items: &[crate::core::Expression]) -> Self::Result;
    fn visit_map(&mut self, pairs: &[(String, crate::core::Expression)]) -> Self::Result;
    fn visit_case(
        &mut self,
        conditions: &[(crate::core::Expression, crate::core::Expression)],
        default: &Option<crate::core::Expression>,
    ) -> Self::Result;
    fn visit_type_cast(
        &mut self,
        expr: &crate::core::Expression,
        target_type: &crate::core::DataType,
    ) -> Self::Result;
    fn visit_subscript(
        &mut self,
        collection: &crate::core::Expression,
        index: &crate::core::Expression,
    ) -> Self::Result;
    fn visit_range(
        &mut self,
        collection: &crate::core::Expression,
        start: &Option<crate::core::Expression>,
        end: &Option<crate::core::Expression>,
    ) -> Self::Result;
    fn visit_path(&mut self, items: &[crate::core::Expression]) -> Self::Result;
    fn visit_label(&mut self, name: &str) -> Self::Result;
    fn visit_tag_property(&mut self, tag: &str, prop: &str) -> Self::Result;
    fn visit_edge_property(&mut self, edge: &str, prop: &str) -> Self::Result;
    fn visit_input_property(&mut self, prop: &str) -> Self::Result;
    fn visit_variable_property(&mut self, var: &str, prop: &str) -> Self::Result;
    fn visit_source_property(&mut self, tag: &str, prop: &str) -> Self::Result;
    fn visit_destination_property(&mut self, tag: &str, prop: &str) -> Self::Result;
}

/// 表达式访问者接受器 trait - 为Expression类型提供接受访问者的能力
pub trait ExpressionAcceptor {
    /// 接受访问者进行访问
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result;
}

impl ExpressionAcceptor for crate::core::Expression {
    fn accept<V: ExpressionVisitor>(&self, visitor: &mut V) -> V::Result {
        use crate::core::Expression;

        match self {
            Expression::Literal(value) => visitor.visit_literal(value),
            Expression::Variable(name) => visitor.visit_variable(name),
            Expression::Property { object, property } => visitor.visit_property(object, property),
            Expression::Binary { left, op, right } => visitor.visit_binary(left, op, right),
            Expression::Unary { op, operand } => visitor.visit_unary(op, operand),
            Expression::Function { name, args } => visitor.visit_function(name, args),
            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => visitor.visit_aggregate(func, arg, *distinct),
            Expression::List(items) => visitor.visit_list(items),
            Expression::Map(pairs) => visitor.visit_map(pairs),
            Expression::Case {
                conditions,
                default,
            } => {
                let default_cloned = default.as_ref().map(|b| b.as_ref().clone());
                visitor.visit_case(conditions, &default_cloned)
            }
            Expression::TypeCast { expr, target_type } => {
                visitor.visit_type_cast(expr, target_type)
            }
            Expression::Subscript { collection, index } => {
                visitor.visit_subscript(collection, index)
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                let start_cloned = start.as_ref().map(|b| b.as_ref().clone());
                let end_cloned = end.as_ref().map(|b| b.as_ref().clone());
                visitor.visit_range(collection, &start_cloned, &end_cloned)
            }
            Expression::Path(items) => visitor.visit_path(items),
            Expression::Label(name) => visitor.visit_label(name),
            Expression::TagProperty { tag, prop } => visitor.visit_tag_property(tag, prop),
            Expression::EdgeProperty { edge, prop } => visitor.visit_edge_property(edge, prop),
            Expression::InputProperty(prop) => visitor.visit_input_property(prop),
            Expression::VariableProperty { var, prop } => {
                visitor.visit_variable_property(var, prop)
            }
            Expression::SourceProperty { tag, prop } => visitor.visit_source_property(tag, prop),
            Expression::DestinationProperty { tag, prop } => {
                visitor.visit_destination_property(tag, prop)
            }

            // 处理新增的表达式类型
            Expression::UnaryPlus(expr) => {
                visitor.visit_unary(&crate::core::UnaryOperator::Plus, expr)
            }
            Expression::UnaryNegate(expr) => {
                visitor.visit_unary(&crate::core::UnaryOperator::Minus, expr)
            }
            Expression::UnaryNot(expr) => {
                visitor.visit_unary(&crate::core::UnaryOperator::Not, expr)
            }
            Expression::UnaryIncr(expr) => {
                visitor.visit_unary(&crate::core::UnaryOperator::Increment, expr)
            }
            Expression::UnaryDecr(expr) => {
                visitor.visit_unary(&crate::core::UnaryOperator::Decrement, expr)
            }
            Expression::IsNull(expr) => {
                visitor.visit_unary(&crate::core::UnaryOperator::IsNull, expr)
            }
            Expression::IsNotNull(expr) => {
                visitor.visit_unary(&crate::core::UnaryOperator::IsNotNull, expr)
            }
            Expression::IsEmpty(expr) => {
                visitor.visit_unary(&crate::core::UnaryOperator::IsEmpty, expr)
            }
            Expression::IsNotEmpty(expr) => {
                visitor.visit_unary(&crate::core::UnaryOperator::IsNotEmpty, expr)
            }

            Expression::TypeCasting { expr, target_type } => {
                visitor.visit_type_cast(expr, &crate::core::DataType::String)
            }
            Expression::ListComprehension {
                generator,
                condition,
            } => {
                // 简化为函数调用
                let cond_expr = condition
                    .as_ref()
                    .map(|c| c.as_ref().clone())
                    .unwrap_or_else(|| crate::core::Expression::bool(true));
                visitor.visit_function(
                    "list_comprehension",
                    &[generator.as_ref().clone(), cond_expr],
                )
            }
            Expression::Predicate { list, condition } => visitor.visit_function(
                "predicate",
                &[list.as_ref().clone(), condition.as_ref().clone()],
            ),
            Expression::Reduce {
                list,
                initial,
                expr,
                ..
            } => visitor.visit_function(
                "reduce",
                &[
                    list.as_ref().clone(),
                    initial.as_ref().clone(),
                    expr.as_ref().clone(),
                ],
            ),
            Expression::PathBuild(items) => visitor.visit_path(items),
            Expression::ESQuery(query) => {
                visitor.visit_function("es_query", &[crate::core::Expression::string(query)])
            }
            Expression::UUID => visitor.visit_function("uuid", &[]),
            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                let start_cloned = start.as_ref().map(|b| b.as_ref().clone());
                let end_cloned = end.as_ref().map(|b| b.as_ref().clone());
                visitor.visit_range(collection.as_ref(), &start_cloned, &end_cloned)
            }
            Expression::MatchPathPattern { patterns, .. } => visitor.visit_list(patterns),
        }
    }
}

/// 默认访问者实现 - 提供基础的访问者功能
#[derive(Debug)]
pub struct DefaultVisitor<T: std::fmt::Debug> {
    context: VisitorContext,
    state: visitor_state_enum::VisitorStateEnum,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: std::fmt::Debug> DefaultVisitor<T> {
    /// 创建新的默认访问者
    pub fn new() -> Self {
        Self {
            context: VisitorContext::new(VisitorConfig::new()),
            state: visitor_state_enum::VisitorStateEnum::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// 创建带配置的默认访问者
    pub fn with_config(config: VisitorConfig) -> Self {
        Self {
            context: VisitorContext::new(config),
            state: visitor_state_enum::VisitorStateEnum::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// 创建带初始深度的默认访问者
    pub fn with_depth(depth: usize) -> Self {
        Self {
            context: VisitorContext::new(VisitorConfig::new()),
            state: visitor_state_enum::VisitorStateEnum::with_depth(depth),
            _phantom: std::marker::PhantomData,
        }
    }

    /// 创建带配置和初始深度的默认访问者
    pub fn with_config_and_depth(config: VisitorConfig, depth: usize) -> Self {
        Self {
            context: VisitorContext::new(config),
            state: visitor_state_enum::VisitorStateEnum::with_depth(depth),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: std::fmt::Debug> Default for DefaultVisitor<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: std::fmt::Debug> VisitorCore<T> for DefaultVisitor<T> {
    type Result = ();

    fn visit(&mut self, _target: &T) -> Self::Result {
        // 默认实现什么也不做
    }

    fn context(&self) -> &VisitorContext {
        &self.context
    }

    fn context_mut(&mut self) -> &mut VisitorContext {
        &mut self.context
    }

    fn state(&self) -> &visitor_state_enum::VisitorStateEnum {
        &self.state
    }

    fn state_mut(&mut self) -> &mut visitor_state_enum::VisitorStateEnum {
        &mut self.state
    }
}
