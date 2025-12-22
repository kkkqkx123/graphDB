//! 标签过滤器处理器
//!
//! 提供对顶点标签的高级过滤功能，支持复杂的表达式求值

use crate::core::vertex_edge_path::Vertex;
use crate::core::Value;
use crate::core::expressions::ExpressionContextCore;
use crate::core::expressions::{ExpressionContext, DefaultExpressionContext};
use crate::core::{Expression, ExpressionEvaluator};

/// 标签过滤器处理器
pub struct TagFilterProcessor {
    evaluator: ExpressionEvaluator,
}

impl std::fmt::Debug for TagFilterProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TagFilterProcessor")
            .field("evaluator", &"ExpressionEvaluator")
            .finish()
    }
}

impl TagFilterProcessor {
    /// 创建新的标签过滤器处理器
    pub fn new() -> Self {
        Self {
            evaluator: ExpressionEvaluator::new(),
        }
    }

    /// 处理标签过滤表达式
    pub fn process_tag_filter(&self, filter_expr: &Expression, vertex: &Vertex) -> bool {
        // 创建包含顶点标签的上下文
        let context = self.create_tag_context(vertex);

        // 评估表达式
        match self.evaluator.evaluate(filter_expr, &context) {
            Ok(value) => self.value_to_bool(&value),
            Err(e) => {
                eprintln!("标签过滤表达式评估失败: {}", e);
                false // 默认排除
            }
        }
    }

    /// 创建包含标签信息的评估上下文
    fn create_tag_context(&self, vertex: &Vertex) -> ExpressionContext {
        let mut context = DefaultExpressionContext::new();

        // 将顶点作为变量添加
        context.set_variable(
            "vertex".to_string(),
            Value::Vertex(Box::new(vertex.clone())),
        );

        // 添加标签列表
        let tag_names: Vec<Value> = vertex
            .tags
            .iter()
            .map(|tag| Value::String(tag.name.clone()))
            .collect();
        context.set_variable("tags".to_string(), Value::List(tag_names));

        // 添加标签数量
        context.set_variable(
            "tag_count".to_string(),
            Value::Int(vertex.tags.len() as i64),
        );

        // 将每个标签作为单独的变量添加
        for (i, tag) in vertex.tags.iter().enumerate() {
            context.set_variable(format!("tag_{}", tag.name), Value::String(tag.name.clone()));
            context.set_variable(format!("tag_{}", i), Value::String(tag.name.clone()));
        }

        // 添加标签属性
        for tag in &vertex.tags {
            let tag_prefix = format!("tag_{}_", tag.name);
            for (prop_name, prop_value) in &tag.properties {
                context.set_variable(format!("{}{}", tag_prefix, prop_name), prop_value.clone());
            }
        }

        ExpressionContext::Default(context)
    }

    /// 将值转换为布尔值
    fn value_to_bool(&self, value: &Value) -> bool {
        match value {
            Value::Bool(b) => *b,
            Value::Null(_) => false,
            Value::Empty => false,
            Value::Int(0) => false,
            Value::Float(0.0) => false,
            Value::String(s) if s.is_empty() => false,
            Value::List(l) if l.is_empty() => false,
            Value::Map(m) if m.is_empty() => false,
            Value::Set(s) if s.is_empty() => false,
            _ => true, // 非空、非零值视为 true
        }
    }

    /// 解析标签过滤字符串为表达式
    pub fn parse_tag_filter(&self, filter_str: &str) -> Result<Expression, String> {
        // 尝试解析为完整表达式
        match crate::query::parser::expressions::parse_expression_from_string(filter_str) {
            Ok(expr) => Ok(expr),
            Err(_) => {
                // 如果解析失败，尝试作为简单的标签列表处理
                self.parse_simple_tag_list(filter_str)
            }
        }
    }

    /// 解析简单的标签列表（逗号分隔）
    fn parse_simple_tag_list(&self, filter_str: &str) -> Result<Expression, String> {
        let tags: Vec<String> = filter_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if tags.is_empty() {
            return Err("空的标签列表".to_string());
        }

        // 创建表达式：tags CONTAINS tag1 OR tags CONTAINS tag2 OR ...
        let mut expr = None;
        for tag in tags {
            let tag_expr = Expression::binary(
                Expression::variable("tags".to_string()),
                crate::core::types::operators::BinaryOperator::In,
                Expression::list(vec![Expression::literal(tag)]),
            );

            expr = match expr {
                None => Some(tag_expr),
                Some(existing) => Some(Expression::binary(
                    existing,
                    crate::core::types::operators::BinaryOperator::Or,
                    tag_expr,
                )),
            };
        }

        expr.ok_or_else(|| "无法创建标签过滤表达式".to_string())
    }
}

impl Default for TagFilterProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vertex_edge_path::{Tag, Vertex};
    use crate::core::types::operators::BinaryOperator;

    #[test]
    fn test_process_tag_filter_with_contains() {
        let processor = TagFilterProcessor::new();

        // 创建测试顶点
        let vertex = Vertex::new(
            Value::Int(1),
            vec![
                Tag::new("user".to_string(), std::collections::HashMap::new()),
                Tag::new("admin".to_string(), std::collections::HashMap::new()),
            ],
        );

        // 测试包含标签的表达式
        let expr = Expression::binary(
            Expression::variable("tags".to_string()),
            BinaryOperator::In,
            Expression::list(vec![Expression::literal("user".to_string())]),
        );

        assert!(processor.process_tag_filter(&expr, &vertex));
    }

    #[test]
    fn test_process_tag_filter_with_count() {
        let processor = TagFilterProcessor::new();

        // 创建测试顶点
        let vertex = Vertex::new(
            Value::Int(1),
            vec![
                Tag::new("user".to_string(), std::collections::HashMap::new()),
                Tag::new("admin".to_string(), std::collections::HashMap::new()),
            ],
        );

        // 测试标签数量表达式
        let expr = Expression::binary(
            Expression::variable("tag_count".to_string()),
            BinaryOperator::GreaterThan,
            Expression::literal(1i64),
        );

        assert!(processor.process_tag_filter(&expr, &vertex));
    }

    #[test]
    fn test_parse_simple_tag_list() {
        let processor = TagFilterProcessor::new();

        let result = processor.parse_simple_tag_list("user, admin, moderator");
        assert!(result.is_ok());

        let expr = result.expect("Expected Ok result for simple tag list parsing");
        // 验证表达式结构（这里简化测试）
        match expr {
            Expression::Binary { op, .. } => {
                assert_eq!(op, BinaryOperator::Or);
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_empty_tag_list() {
        let processor = TagFilterProcessor::new();

        let result = processor.parse_simple_tag_list("");
        assert!(result.is_err());
    }
}
