use crate::core::ExpressionError;
use crate::core::Value;
use crate::expression::{Expression, ExpressionContext};

/// 评估属性表达式
pub fn evaluate_property_expression(
    expr: &Expression,
    context: &ExpressionContext,
) -> Result<Value, ExpressionError> {
    match expr {
        Expression::TagProperty { tag, prop } => {
            // 在顶点的标签中查找属性
            if let Some(vertex) = context.get_vertex() {
                for vertex_tag in &vertex.tags {
                    if &vertex_tag.name == tag {
                        if let Some(value) = vertex_tag.properties.get(prop) {
                            return Ok(value.clone());
                        }
                    }
                }
            }
            Err(ExpressionError::PropertyNotFound(format!(
                "{}.{}",
                tag, prop
            )))
        }

        Expression::EdgeProperty { edge, prop } => {
            // 在边中查找属性
            if let Some(edge_obj) = context.get_edge() {
                // 检查边类型是否匹配
                if edge_obj.edge_type == *edge {
                    if let Some(value) = edge_obj.props.get(prop) {
                        return Ok(value.clone());
                    }
                }
            }
            Err(ExpressionError::PropertyNotFound(format!(
                "{}.{}",
                edge, prop
            )))
        }

        Expression::InputProperty(prop) => {
            // 从输入中查找属性
            if let Some(vertex) = context.get_vertex() {
                for tag in &vertex.tags {
                    if let Some(value) = tag.properties.get(prop) {
                        return Ok(value.clone());
                    }
                }
            }

            if let Some(edge) = context.get_edge() {
                if let Some(value) = edge.props.get(prop) {
                    return Ok(value.clone());
                }
            }

            if let Some(value) = context.get_variable(prop) {
                return Ok(value);
            }

            Err(ExpressionError::PropertyNotFound(format!("$-.{}", prop)))
        }

        Expression::VariableProperty { var, prop } => {
            // 从变量中查找属性
            if let Some(value) = context.get_variable(var) {
                match value {
                    Value::Map(map) => match map.get(prop) {
                        Some(prop_value) => Ok(prop_value.clone()),
                        None => Err(ExpressionError::PropertyNotFound(format!(
                            "{}.{}",
                            var, prop
                        ))),
                    },
                    _ => Err(ExpressionError::TypeError(format!(
                        "Variable '{}' is not a map type, cannot access property '{}'",
                        var, prop
                    ))),
                }
            } else {
                Err(ExpressionError::PropertyNotFound(format!(
                    "${}.{}",
                    var, prop
                )))
            }
        }

        Expression::SourceProperty { tag, prop } => {
            // 查找源顶点的属性
            if let Some(edge) = context.get_edge() {
                // 在源顶点中查找标签和属性
                if let Value::Vertex(src_vertex) = &*edge.src {
                    for vertex_tag in &src_vertex.tags {
                        if &vertex_tag.name == tag {
                            if let Some(value) = vertex_tag.properties.get(prop) {
                                return Ok(value.clone());
                            }
                        }
                    }
                }
            }
            Err(ExpressionError::PropertyNotFound(format!(
                "$^.{}.{}",
                tag, prop
            )))
        }

        Expression::DestinationProperty { tag, prop } => {
            // 查找目标顶点的属性
            if let Some(edge) = context.get_edge() {
                // 在目标顶点中查找标签和属性
                if let Value::Vertex(dst_vertex) = &*edge.dst {
                    for vertex_tag in &dst_vertex.tags {
                        if &vertex_tag.name == tag {
                            if let Some(value) = vertex_tag.properties.get(prop) {
                                return Ok(value.clone());
                            }
                        }
                    }
                }
            }
            Err(ExpressionError::PropertyNotFound(format!(
                "$$.{}.{}",
                tag, prop
            )))
        }

        _ => Err(ExpressionError::TypeError(
            "Expression is not a property expression".to_string(),
        )),
    }
}
