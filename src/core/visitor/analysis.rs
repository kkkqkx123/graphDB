//! 分析类访问者
//!
//! 这个模块提供了用于分析 Value 类型的访问者实现

use crate::core::value::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
};
use crate::core::vertex_edge_path::{Edge, Path, Vertex};
use crate::core::visitor::core::{
    DefaultVisitorState, ValueVisitor, VisitorConfig, VisitorContext, VisitorCore, VisitorResult,
    VisitorState,
};
use std::collections::HashMap;

/// Value 类型分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeCategory {
    Empty,
    Null,
    Bool,
    Numeric,
    String,
    Temporal,
    GraphElement,
    Collection,
    Geography,
    Dataset,
}

/// 类型检查访问者 - 用于确定 Value 的类型分类
#[derive(Debug)]
pub struct TypeCheckerVisitor {
    categories: Vec<TypeCategory>,
    context: VisitorContext,
    state: DefaultVisitorState,
}

impl TypeCheckerVisitor {
    pub fn new() -> Self {
        Self {
            categories: Vec::new(),
            context: VisitorContext::new(VisitorConfig::default()),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn with_config(config: VisitorConfig) -> Self {
        Self {
            categories: Vec::new(),
            context: VisitorContext::new(config),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn categories(&self) -> &[TypeCategory] {
        &self.categories
    }

    pub fn has_category(&self, category: TypeCategory) -> bool {
        self.categories.contains(&category)
    }

    pub fn get_primary_category(&self) -> Option<TypeCategory> {
        self.categories.first().copied()
    }

    pub fn get_type_name(&self) -> &'static str {
        match self.get_primary_category() {
            Some(TypeCategory::Empty) => "Empty",
            Some(TypeCategory::Null) => "Null",
            Some(TypeCategory::Bool) => "Bool",
            Some(TypeCategory::Numeric) => "Numeric",
            Some(TypeCategory::String) => "String",
            Some(TypeCategory::Temporal) => "Temporal",
            Some(TypeCategory::GraphElement) => "GraphElement",
            Some(TypeCategory::Collection) => "Collection",
            Some(TypeCategory::Geography) => "Geography",
            Some(TypeCategory::Dataset) => "Dataset",
            None => "Unknown",
        }
    }

    pub fn reset(&mut self) {
        self.categories.clear();
        self.state.reset();
    }

    fn add_category(&mut self, category: TypeCategory) {
        if !self.categories.contains(&category) {
            self.categories.push(category);
        }
    }
}

impl ValueVisitor for TypeCheckerVisitor {
    type Result = ();

    fn visit_bool(&mut self, _value: bool) -> Self::Result {
        self.add_category(TypeCategory::Bool);
    }

    fn visit_int(&mut self, _value: i64) -> Self::Result {
        self.add_category(TypeCategory::Numeric);
    }

    fn visit_float(&mut self, _value: f64) -> Self::Result {
        self.add_category(TypeCategory::Numeric);
    }

    fn visit_string(&mut self, _value: &str) -> Self::Result {
        self.add_category(TypeCategory::String);
    }

    fn visit_date(&mut self, _value: &DateValue) -> Self::Result {
        self.add_category(TypeCategory::Temporal);
    }

    fn visit_time(&mut self, _value: &TimeValue) -> Self::Result {
        self.add_category(TypeCategory::Temporal);
    }

    fn visit_datetime(&mut self, _value: &DateTimeValue) -> Self::Result {
        self.add_category(TypeCategory::Temporal);
    }

    fn visit_vertex(&mut self, _value: &Vertex) -> Self::Result {
        self.add_category(TypeCategory::GraphElement);
    }

    fn visit_edge(&mut self, _value: &Edge) -> Self::Result {
        self.add_category(TypeCategory::GraphElement);
    }

    fn visit_path(&mut self, _value: &Path) -> Self::Result {
        self.add_category(TypeCategory::GraphElement);
    }

    fn visit_list(&mut self, _value: &[Value]) -> Self::Result {
        self.add_category(TypeCategory::Collection);
    }

    fn visit_map(&mut self, _value: &HashMap<String, Value>) -> Self::Result {
        self.add_category(TypeCategory::Collection);
    }

    fn visit_set(&mut self, _value: &std::collections::HashSet<Value>) -> Self::Result {
        self.add_category(TypeCategory::Collection);
    }

    fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
        self.add_category(TypeCategory::Geography);
    }

    fn visit_duration(&mut self, _value: &DurationValue) -> Self::Result {
        self.add_category(TypeCategory::Temporal);
    }

    fn visit_dataset(&mut self, _value: &DataSet) -> Self::Result {
        self.add_category(TypeCategory::Dataset);
    }

    fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
        self.add_category(TypeCategory::Null);
    }

    fn visit_empty(&mut self) -> Self::Result {
        self.add_category(TypeCategory::Empty);
    }
}

impl VisitorCore for TypeCheckerVisitor {
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

    fn pre_visit(&mut self) -> VisitorResult<()> {
        self.state.inc_visit_count();
        if self.state.depth() > self.context.config().max_depth {
            return Err(crate::core::visitor::core::VisitorError::Validation(
                format!("访问深度超过限制: {}", self.context.config().max_depth),
            ));
        }
        Ok(())
    }

    fn post_visit(&mut self) -> VisitorResult<()> {
        Ok(())
    }
}

/// 复杂度分析访问者 - 分析 Value 的复杂度
#[derive(Debug)]
pub struct ComplexityAnalyzerVisitor {
    depth: usize,
    max_depth: usize,
    total_nodes: usize,
    container_nodes: usize,
    primitive_nodes: usize,
    context: VisitorContext,
    state: DefaultVisitorState,
}

impl ComplexityAnalyzerVisitor {
    pub fn new() -> Self {
        Self {
            depth: 0,
            max_depth: 0,
            total_nodes: 0,
            container_nodes: 0,
            primitive_nodes: 0,
            context: VisitorContext::new(VisitorConfig::default()),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn with_config(config: VisitorConfig) -> Self {
        Self {
            depth: 0,
            max_depth: 0,
            total_nodes: 0,
            container_nodes: 0,
            primitive_nodes: 0,
            context: VisitorContext::new(config),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn analyze(&self) -> ComplexityMetrics {
        ComplexityMetrics {
            depth: self.depth,
            total_nodes: self.total_nodes,
            container_nodes: self.container_nodes,
            primitive_nodes: self.primitive_nodes,
            complexity_ratio: if self.total_nodes > 0 {
                self.container_nodes as f64 / self.total_nodes as f64
            } else {
                0.0
            },
        }
    }

    pub fn reset(&mut self) {
        self.depth = 0;
        self.max_depth = 0;
        self.total_nodes = 0;
        self.container_nodes = 0;
        self.primitive_nodes = 0;
        self.state.reset();
    }

    fn update_depth(&mut self, new_depth: usize) {
        if new_depth > self.depth {
            self.depth = new_depth;
        }
    }
}

impl ValueVisitor for ComplexityAnalyzerVisitor {
    type Result = ();

    fn visit_bool(&mut self, _value: bool) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_int(&mut self, _value: i64) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_float(&mut self, _value: f64) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_string(&mut self, value: &str) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
        // 字符串的复杂度基于长度
        if value.len() > 100 {
            self.max_depth += 1;
        }
    }

    fn visit_date(&mut self, _value: &DateValue) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_time(&mut self, _value: &TimeValue) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_datetime(&mut self, _value: &DateTimeValue) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
        // 顶点的复杂度基于标签和属性数量
        let complexity = value.tags().len() + value.vertex_properties().len();
        if complexity > 5 {
            self.max_depth += 1;
        }
    }

    fn visit_edge(&mut self, value: &Edge) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
        // 边的复杂度基于属性数量
        if value.property_count() > 3 {
            self.max_depth += 1;
        }
    }

    fn visit_path(&mut self, value: &Path) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
        // 路径的复杂度基于长度
        if value.len() > 10 {
            self.max_depth += 1;
        }
    }

    fn visit_list(&mut self, value: &[Value]) -> Self::Result {
        self.container_nodes += 1;
        self.total_nodes += 1;
        self.update_depth(value.len());
    }

    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
        self.container_nodes += 1;
        self.total_nodes += 1;
        self.update_depth(value.len());
    }

    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
        self.container_nodes += 1;
        self.total_nodes += 1;
        self.update_depth(value.len());
    }

    fn visit_geography(&mut self, _value: &GeographyValue) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_duration(&mut self, _value: &DurationValue) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
        self.container_nodes += 1;
        self.total_nodes += 1;
        self.update_depth(value.rows.len());
    }

    fn visit_null(&mut self, _null_type: &NullType) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }

    fn visit_empty(&mut self) -> Self::Result {
        self.primitive_nodes += 1;
        self.total_nodes += 1;
    }
}

impl VisitorCore for ComplexityAnalyzerVisitor {
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

    fn pre_visit(&mut self) -> VisitorResult<()> {
        self.state.inc_visit_count();
        self.state.inc_depth();
        if self.state.depth() > self.context.config().max_depth {
            return Err(crate::core::visitor::core::VisitorError::Validation(
                format!("访问深度超过限制: {}", self.context.config().max_depth),
            ));
        }
        Ok(())
    }

    fn post_visit(&mut self) -> VisitorResult<()> {
        self.state.dec_depth();
        Ok(())
    }
}

/// 复杂度级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComplexityLevel {
    Simple,
    Moderate,
    Complex,
}

/// 复杂度指标
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    pub depth: usize,
    pub total_nodes: usize,
    pub container_nodes: usize,
    pub primitive_nodes: usize,
    pub complexity_ratio: f64,
}

impl ComplexityMetrics {
    pub fn is_simple(&self) -> bool {
        self.complexity_ratio < 0.3 && self.depth < 5
    }

    pub fn is_complex(&self) -> bool {
        self.complexity_ratio > 0.7 || self.depth > 10
    }

    pub fn is_moderate(&self) -> bool {
        !self.is_simple() && !self.is_complex()
    }

    pub fn get_level(&self) -> ComplexityLevel {
        if self.is_simple() {
            ComplexityLevel::Simple
        } else if self.is_complex() {
            ComplexityLevel::Complex
        } else {
            ComplexityLevel::Moderate
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_type_checker_visitor() {
        let mut visitor = TypeCheckerVisitor::new();

        let int_value = Value::Int(42);
        int_value.accept(&mut visitor);
        assert!(visitor.has_category(TypeCategory::Numeric));
        assert_eq!(visitor.get_type_name(), "Numeric");

        visitor.reset();
        let string_value = Value::String("test".to_string());
        string_value.accept(&mut visitor);
        assert!(visitor.has_category(TypeCategory::String));
        assert_eq!(visitor.get_type_name(), "String");
    }

    #[test]
    fn test_complexity_analyzer() {
        let mut visitor = ComplexityAnalyzerVisitor::new();

        let simple_value = Value::Int(42);
        simple_value.accept(&mut visitor);
        let metrics = visitor.analyze();
        assert!(metrics.is_simple());

        visitor.reset();
        let complex_value = Value::List(vec![
            Value::Int(1),
            Value::String("test".to_string()),
            Value::Map(std::collections::HashMap::from([(
                "key".to_string(),
                Value::Bool(true),
            )])),
        ]);
        complex_value.accept(&mut visitor);
        let metrics = visitor.analyze();
        assert!(metrics.is_moderate());
    }

    #[test]
    fn test_visitor_core_integration() {
        let config = VisitorConfig::new().with_max_depth(5);
        let mut visitor = TypeCheckerVisitor::with_config(config);

        // 测试VisitorCore方法
        assert!(visitor.should_continue());
        assert_eq!(visitor.state().depth(), 0);

        visitor.state_mut().inc_depth();
        assert_eq!(visitor.state().depth(), 1);

        visitor.reset();
        assert_eq!(visitor.state().depth(), 0);

        // 测试原始ValueVisitor功能
        let value = Value::Int(42);
        value.accept(&mut visitor);
        assert!(visitor.has_category(TypeCategory::Numeric));
    }

    #[test]
    fn test_complexity_analyzer_with_config() {
        let config = VisitorConfig::new().with_max_depth(3);
        let mut visitor = ComplexityAnalyzerVisitor::with_config(config);

        assert_eq!(visitor.context().config().max_depth, 3);

        let simple_value = Value::Int(42);
        simple_value.accept(&mut visitor);
        let metrics = visitor.analyze();
        assert!(metrics.is_simple());
        assert_eq!(visitor.state().visit_count(), 1);
    }
}
