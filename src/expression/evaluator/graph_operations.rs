//! 图操作求值
//!
//! 提供图相关的表达式求值功能，包括标签、顶点属性、边属性等

use crate::core::error::ExpressionError;
use crate::core::Value;
use crate::expression::evaluator::traits::ExpressionContext;

/// 图操作求值器
pub struct GraphOperationEvaluator;

impl GraphOperationEvaluator {
    /// 求值标签表达式
    pub fn eval_label_expression<C: ExpressionContext>(
        &self,
        label_name: &str,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        if let Some(vertex) = context.get_vertex() {
            let label_list: Vec<Value> = vertex
                .tags
                .iter()
                .map(|tag| Value::String(tag.name.clone()))
                .collect();
            Ok(Value::List(label_list))
        } else {
            Err(ExpressionError::runtime_error(format!(
                "标签表达式需要顶点上下文: {}",
                label_name
            )))
        }
    }

    /// 求值标签属性表达式
    pub fn eval_tag_property<C: ExpressionContext>(
        &self,
        tag_name: &str,
        prop_name: &str,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        if let Some(vertex) = context.get_vertex() {
            for tag in &vertex.tags {
                if tag.name == tag_name {
                    if let Some(value) = tag.properties.get(prop_name) {
                        return Ok(value.clone());
                    }
                }
            }
            Err(ExpressionError::runtime_error(format!(
                "标签属性不存在: {}.{}",
                tag_name, prop_name
            )))
        } else {
            Err(ExpressionError::runtime_error(format!(
                "标签属性表达式需要顶点上下文: {}.{}",
                tag_name, prop_name
            )))
        }
    }

    /// 求值边属性表达式
    pub fn eval_edge_property<C: ExpressionContext>(
        &self,
        edge_name: &str,
        prop_name: &str,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        if let Some(edge) = context.get_edge() {
            if edge_name.is_empty() || edge.edge_type() == edge_name {
                if let Some(value) = edge.properties().get(prop_name) {
                    return Ok(value.clone());
                }
                Err(ExpressionError::runtime_error(format!(
                    "边属性不存在: {}.{}",
                    edge_name, prop_name
                )))
            } else {
                Err(ExpressionError::runtime_error(format!(
                    "边名称不匹配: 期望 '{}', 实际 '{}'",
                    edge_name,
                    edge.edge_type()
                )))
            }
        } else {
            Err(ExpressionError::runtime_error(format!(
                "边属性表达式需要边上下文: {}.{}",
                edge_name, prop_name
            )))
        }
    }

    /// 求值变量属性表达式
    pub fn eval_variable_property<C: ExpressionContext>(
        &self,
        var_name: &str,
        prop_name: &str,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        if let Some(value) = context.get_variable(var_name) {
            match value {
                Value::Vertex(vertex) => {
                    if let Some(prop_value) = vertex.properties.get(prop_name) {
                        Ok(prop_value.clone())
                    } else {
                        Err(ExpressionError::runtime_error(format!(
                            "顶点属性不存在: {}.{}",
                            var_name, prop_name
                        )))
                    }
                }
                Value::Edge(edge) => {
                    if let Some(prop_value) = edge.properties().get(prop_name) {
                        Ok(prop_value.clone())
                    } else {
                        Err(ExpressionError::runtime_error(format!(
                            "边属性不存在: {}.{}",
                            var_name, prop_name
                        )))
                    }
                }
                Value::Map(map) => {
                    if let Some(prop_value) = map.get(prop_name) {
                        Ok(prop_value.clone())
                    } else {
                        Err(ExpressionError::runtime_error(format!(
                            "映射属性不存在: {}.{}",
                            var_name, prop_name
                        )))
                    }
                }
                _ => Err(ExpressionError::type_error(format!(
                    "变量属性访问需要顶点、边或映射类型: {}",
                    var_name
                ))),
            }
        } else {
            Err(ExpressionError::undefined_variable(var_name))
        }
    }

    /// 求值源属性表达式
    pub fn eval_source_property<C: ExpressionContext>(
        &self,
        tag_name: &str,
        prop_name: &str,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        if let Some(edge) = context.get_edge() {
            let source_var = format!("_src_{}", edge.src());
            if let Some(value) = context.get_variable(&source_var) {
                if let Value::Vertex(vertex) = value {
                    for tag in &vertex.tags {
                        if tag.name == tag_name {
                            if let Some(prop_value) = tag.properties.get(prop_name) {
                                return Ok(prop_value.clone());
                            }
                        }
                    }
                    Err(ExpressionError::runtime_error(format!(
                        "源标签属性不存在: $^.{}.{}",
                        tag_name, prop_name
                    )))
                } else {
                    Err(ExpressionError::type_error(format!(
                        "源属性表达式需要顶点类型: $^.{}.{}",
                        tag_name, prop_name
                    )))
                }
            } else {
                Err(ExpressionError::undefined_variable(&source_var))
            }
        } else {
            Err(ExpressionError::runtime_error(format!(
                "源属性表达式需要边上下文: $^.{}.{}",
                tag_name, prop_name
            )))
        }
    }

    /// 求值目的属性表达式
    pub fn eval_destination_property<C: ExpressionContext>(
        &self,
        tag_name: &str,
        prop_name: &str,
        context: &mut C,
    ) -> Result<Value, ExpressionError> {
        if let Some(edge) = context.get_edge() {
            let dest_var = format!("_dst_{}", edge.dst());
            if let Some(value) = context.get_variable(&dest_var) {
                if let Value::Vertex(vertex) = value {
                    for tag in &vertex.tags {
                        if tag.name == tag_name {
                            if let Some(prop_value) = tag.properties.get(prop_name) {
                                return Ok(prop_value.clone());
                            }
                        }
                    }
                    Err(ExpressionError::runtime_error(format!(
                        "目的标签属性不存在: $$.{}.{}",
                        tag_name, prop_name
                    )))
                } else {
                    Err(ExpressionError::type_error(format!(
                        "目的属性表达式需要顶点类型: $$.{}.{}",
                        tag_name, prop_name
                    )))
                }
            } else {
                Err(ExpressionError::undefined_variable(&dest_var))
            }
        } else {
            Err(ExpressionError::runtime_error(format!(
                "目的属性表达式需要边上下文: $$.{}.{}",
                tag_name, prop_name
            )))
        }
    }
}
