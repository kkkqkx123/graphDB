//! 标签过滤器处理器
//!
//! 提供对顶点标签的高级过滤功能，支持复杂的表达式求值

use crate::core::vertex_edge_path::Vertex;
use crate::core::Expression;
use crate::core::Value;
use crate::core::value::dataset::List;
use crate::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::expression::evaluator::traits::ExpressionContext;
use crate::expression::DefaultExpressionContext;

/// 标签过滤器处理器
///
/// 使用 unit struct 模式，零开销
#[derive(Debug)]
pub struct TagFilterProcessor;

impl TagFilterProcessor {
    /// 处理标签过滤表达式
    pub fn process_tag_filter(filter_expression: &Expression, vertex: &Vertex) -> bool {
        // 创建包含顶点标签的上下文
        let mut context = Self::create_tag_context(vertex);

        // 评估表达式
        match ExpressionEvaluator::evaluate(filter_expression, &mut context) {
            Ok(value) => Self::value_to_bool(&value),
            Err(e) => {
                eprintln!("标签过滤表达式评估失败: {}", e);
                false // 默认排除
            }
        }
    }

    /// 创建包含标签信息的评估上下文
    fn create_tag_context(vertex: &Vertex) -> DefaultExpressionContext {
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
        context.set_variable("tags".to_string(), Value::List(List::from(tag_names)));

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

        context
    }

    /// 将值转换为布尔值
    fn value_to_bool(value: &Value) -> bool {
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
    pub fn parse_tag_filter(filter_str: &str) -> Result<Expression, String> {
        // 尝试解析为完整表达式
        match crate::query::parser::parse_expression_meta_from_string(filter_str) {
            Ok(meta) => Ok(meta.into()),
            Err(_) => {
                // 如果解析失败，尝试作为简单的标签列表处理
                Self::parse_simple_tag_list(filter_str)
            }
        }
    }

    /// 解析简单的标签列表（逗号分隔）
    fn parse_simple_tag_list(filter_str: &str) -> Result<Expression, String> {
        let tags: Vec<String> = filter_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if tags.is_empty() {
            return Err("空的标签列表".to_string());
        }

        // 创建表达式：tags CONTAINS tag1 OR tags CONTAINS tag2 OR ...
        let mut expression = None;
        for tag in tags {
            let tag_expression = Expression::binary(
                Expression::variable("tags".to_string()),
                crate::core::types::operators::BinaryOperator::In,
                Expression::list(vec![Expression::literal(tag)]),
            );

            expression = match expression {
                None => Some(tag_expression),
                Some(existing) => Some(Expression::binary(
                    existing,
                    crate::core::types::operators::BinaryOperator::Or,
                    tag_expression,
                )),
            };
        }

        expression.ok_or_else(|| "无法创建标签过滤表达式".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::BinaryOperator;
    use crate::core::vertex_edge_path::{Tag, Vertex};

    #[test]
    fn test_process_tag_filter_with_contains() {
        // 创建测试顶点
        let vertex = Vertex::new(
            Value::Int(1),
            vec![
                Tag::new("user".to_string(), std::collections::HashMap::new()),
                Tag::new("admin".to_string(), std::collections::HashMap::new()),
            ],
        );

        // 测试包含标签的表达式 - "user" IN tags
        let expression = Expression::binary(
            Expression::literal("user".to_string()),
            BinaryOperator::In,
            Expression::variable("tags".to_string()),
        );

        assert!(TagFilterProcessor::process_tag_filter(&expression, &vertex));
    }

    #[test]
    fn test_process_tag_filter_with_count() {
        // 创建测试顶点
        let vertex = Vertex::new(
            Value::Int(1),
            vec![
                Tag::new("user".to_string(), std::collections::HashMap::new()),
                Tag::new("admin".to_string(), std::collections::HashMap::new()),
            ],
        );

        // 测试标签数量表达式
        let expression = Expression::binary(
            Expression::variable("tag_count".to_string()),
            BinaryOperator::GreaterThan,
            Expression::literal(1i64),
        );

        assert!(TagFilterProcessor::process_tag_filter(&expression, &vertex));
    }

    #[test]
    fn test_parse_simple_tag_list() {
        let result = TagFilterProcessor::parse_simple_tag_list("user, admin, moderator");
        assert!(result.is_ok());

        let expression = result.expect("Expected Ok result for simple tag list parsing");
        // 验证表达式结构（这里简化测试）
        match expression {
            Expression::Binary { op, .. } => {
                assert_eq!(op, BinaryOperator::Or);
            }
            _ => panic!("Expected binary expression"),
        }
    }

    #[test]
    fn test_parse_empty_tag_list() {
        let result = TagFilterProcessor::parse_simple_tag_list("");
        assert!(result.is_err());
    }
}
