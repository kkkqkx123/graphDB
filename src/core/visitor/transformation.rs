//! 转换类访问者
//!
//! 这个模块提供了用于转换 Value 的访问者实现

use crate::core::value::{
    DataSet, DateTimeValue, DateValue, DurationValue, GeographyValue, NullType, TimeValue, Value,
};
use crate::core::vertex_edge_path::{Edge, Path, Step, Tag, Vertex};
use crate::core::visitor::core::{utils, ValueVisitor, VisitorCore, VisitorContext, VisitorConfig, DefaultVisitorState, VisitorState, VisitorResult, VisitorError};
use std::collections::HashMap;

/// 深度克隆访问者 - 创建 Value 的深度副本
#[derive(Debug)]
pub struct DeepCloneVisitor {
    max_depth: usize,
    context: VisitorContext,
    state: DefaultVisitorState,
}

impl DeepCloneVisitor {
    pub fn new() -> Self {
        Self {
            max_depth: 100, // 默认最大深度限制
            context: VisitorContext::new(VisitorConfig::default()),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            max_depth,
            context: VisitorContext::new(VisitorConfig::new().with_max_depth(max_depth)),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn with_config(config: VisitorConfig) -> Self {
        Self {
            max_depth: config.max_depth,
            context: VisitorContext::new(config),
            state: DefaultVisitorState::new(),
        }
    }

    pub fn clone_value(value: &Value) -> Result<Value, TransformationError> {
        value.accept(&mut Self::new())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TransformationError {
    #[error("转换错误: {0}")]
    Transformation(String),
    #[error("递归深度超过限制")]
    MaxDepthExceeded,
}

impl ValueVisitor for DeepCloneVisitor {
    type Result = Result<Value, TransformationError>;

    fn visit_bool(&mut self, value: bool) -> Self::Result {
        Ok(Value::Bool(value))
    }

    fn visit_int(&mut self, value: i64) -> Self::Result {
        Ok(Value::Int(value))
    }

    fn visit_float(&mut self, value: f64) -> Self::Result {
        Ok(Value::Float(value))
    }

    fn visit_string(&mut self, value: &str) -> Self::Result {
        Ok(Value::String(value.to_string()))
    }

    fn visit_date(&mut self, value: &DateValue) -> Self::Result {
        Ok(Value::Date(value.clone()))
    }

    fn visit_time(&mut self, value: &TimeValue) -> Self::Result {
        Ok(Value::Time(value.clone()))
    }

    fn visit_datetime(&mut self, value: &DateTimeValue) -> Self::Result {
        Ok(Value::DateTime(value.clone()))
    }

    fn visit_vertex(&mut self, value: &Vertex) -> Self::Result {
        let mut cloned_tags = Vec::with_capacity(value.tags().len());
        for tag in value.tags() {
            let mut cloned_props = HashMap::new();
            for (name, prop_value) in &tag.properties {
                cloned_props.insert(name.clone(), Self::clone_value(prop_value)?);
            }
            cloned_tags.push(Tag::new(tag.name.clone(), cloned_props));
        }

        let mut cloned_vertex_props = HashMap::new();
        for (name, prop_value) in value.vertex_properties() {
            cloned_vertex_props.insert(name.clone(), Self::clone_value(prop_value)?);
        }

        Ok(Value::Vertex(Box::new(Vertex::new_with_properties(
            value.id().clone(),
            cloned_tags,
            cloned_vertex_props,
        ))))
    }

    fn visit_edge(&mut self, value: &Edge) -> Self::Result {
        let mut cloned_props = HashMap::new();
        for (name, prop_value) in value.get_all_properties() {
            cloned_props.insert(name.clone(), Self::clone_value(prop_value)?);
        }

        Ok(Value::Edge(Edge::new(
            (*value.src).clone(),
            (*value.dst).clone(),
            value.edge_type().to_string(),
            value.ranking(),
            cloned_props,
        )))
    }

    fn visit_path(&mut self, value: &Path) -> Self::Result {
        let cloned_src = Self::clone_value(&Value::Vertex(Box::new(value.src.as_ref().clone())))?;
        let mut cloned_steps = Vec::with_capacity(value.steps.len());

        for step in &value.steps {
            let cloned_dst =
                Self::clone_value(&Value::Vertex(Box::new(step.dst.as_ref().clone())))?;
            let cloned_edge = Self::clone_value(&Value::Edge(step.edge.as_ref().clone()))?;
            cloned_steps.push(Step {
                dst: Box::new(match cloned_dst {
                    Value::Vertex(v) => *v,
                    _ => {
                        return Err(TransformationError::Transformation(
                            "Expected vertex".to_string(),
                        ))
                    }
                }),
                edge: Box::new(match cloned_edge {
                    Value::Edge(e) => e,
                    _ => {
                        return Err(TransformationError::Transformation(
                            "Expected edge".to_string(),
                        ))
                    }
                }),
            });
        }

        Ok(Value::Path(Path {
            src: Box::new(match cloned_src {
                Value::Vertex(v) => *v,
                _ => {
                    return Err(TransformationError::Transformation(
                        "Expected vertex".to_string(),
                    ))
                }
            }),
            steps: cloned_steps,
        }))
    }

    fn visit_list(&mut self, value: &[Value]) -> Self::Result {
        let cloned_list: Vec<Value> = value
            .iter()
            .map(|v| Self::clone_value(v))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::List(cloned_list))
    }

    fn visit_map(&mut self, value: &HashMap<String, Value>) -> Self::Result {
        let mut cloned_map = HashMap::new();
        for (k, v) in value {
            cloned_map.insert(k.clone(), Self::clone_value(v)?);
        }
        Ok(Value::Map(cloned_map))
    }

    fn visit_set(&mut self, value: &std::collections::HashSet<Value>) -> Self::Result {
        let cloned_set: std::collections::HashSet<Value> = value
            .iter()
            .map(|v| Self::clone_value(v))
            .collect::<Result<std::collections::HashSet<_>, _>>()?;
        Ok(Value::Set(cloned_set))
    }

    fn visit_geography(&mut self, value: &GeographyValue) -> Self::Result {
        Ok(Value::Geography(value.clone()))
    }

    fn visit_duration(&mut self, value: &DurationValue) -> Self::Result {
        Ok(Value::Duration(value.clone()))
    }

    fn visit_dataset(&mut self, value: &DataSet) -> Self::Result {
        let cloned_col_names = value.col_names.clone();
        let mut cloned_rows = Vec::with_capacity(value.rows.len());

        for row in &value.rows {
            let cloned_row = row
                .iter()
                .map(|v| Self::clone_value(v))
                .collect::<Result<Vec<_>, _>>()?;
            cloned_rows.push(cloned_row);
        }

        Ok(Value::DataSet(DataSet {
            col_names: cloned_col_names,
            rows: cloned_rows,
        }))
    }

    fn visit_null(&mut self, null_type: &NullType) -> Self::Result {
        Ok(Value::Null(null_type.clone()))
    }

    fn visit_empty(&mut self) -> Self::Result {
        Ok(Value::Empty)
    }
}

// Implement From trait for error conversion
impl From<utils::RecursionError> for TransformationError {
    fn from(err: utils::RecursionError) -> Self {
        match err {
            utils::RecursionError::MaxDepthExceeded => TransformationError::MaxDepthExceeded,
        }
    }
}

impl VisitorCore for DeepCloneVisitor {
    type Result = Result<Value, TransformationError>;
    
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
            return Err(VisitorError::Validation(
                format!("访问深度超过限制: {}", self.context.config().max_depth)
            ));
        }
        Ok(())
    }
    
    fn post_visit(&mut self) -> VisitorResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::Value;

    #[test]
    fn test_deep_clone_visitor() {
        let original = Value::List(vec![
            Value::Int(42),
            Value::String("test".to_string()),
            Value::Map(std::collections::HashMap::from([(
                "key".to_string(),
                Value::Bool(true),
            )])),
        ]);

        let cloned = DeepCloneVisitor::clone_value(&original).expect("DeepCloneVisitor::clone_value should succeed in test");
        assert_eq!(original, cloned);
    }
    
    #[test]
    fn test_visitor_core_integration() {
        let config = VisitorConfig::new().with_max_depth(5);
        let mut visitor = DeepCloneVisitor::with_config(config);
        
        // 测试VisitorCore方法
        assert!(visitor.should_continue());
        assert_eq!(visitor.state().depth(), 0);
        
        visitor.state_mut().inc_depth();
        assert_eq!(visitor.state().depth(), 1);
        
        visitor.reset().unwrap();
        assert_eq!(visitor.state().depth(), 0);
        
        // 测试原始ValueVisitor功能
        let value = crate::core::value::Value::Int(42);
        let result = value.accept(&mut visitor);
        assert!(result.is_ok());
    }
}