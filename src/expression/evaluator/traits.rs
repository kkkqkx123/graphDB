//! 表达式求值器特征定义
//!
//! 定义表达式求值器的核心接口和特征

use crate::core::error::ExpressionError;
use crate::core::Value;
use crate::expression::Expression;

/// 表达式求值器核心特征
///
/// 使用泛型约束避免动态分发，提高性能
pub trait Evaluator<C: ExpressionContext> {
    /// 求值表达式
    fn evaluate(&self, expression: &Expression, context: &mut C) -> Result<Value, ExpressionError>;

    /// 批量求值表达式
    fn evaluate_batch(
        &self,
        expressions: &[Expression],
        context: &mut C,
    ) -> Result<Vec<Value>, ExpressionError> {
        let mut results = Vec::with_capacity(expressions.len());
        for expression in expressions {
            results.push(self.evaluate(expression, context)?);
        }
        Ok(results)
    }

    /// 检查表达式是否可以求值
    fn can_evaluate(&self, _expression: &Expression, _context: &C) -> bool {
        true // 默认实现：所有表达式都可以求值
    }

    /// 获取求值器名称
    fn name(&self) -> &str;

    /// 获取求值器描述
    fn description(&self) -> &str;

    /// 获取求值器版本
    fn version(&self) -> &str;
}

/// 表达式上下文特征
///
/// 为图数据库表达式求值提供统一的上下文接口
pub trait ExpressionContext {
    /// 获取变量值
    fn get_variable(&self, name: &str) -> Option<Value>;

    /// 设置变量值
    fn set_variable(&mut self, name: String, value: Value);

    /// 获取所有变量名
    fn get_variable_names(&self) -> Vec<&str>;

    /// 检查变量是否存在
    fn has_variable(&self, name: &str) -> bool {
        self.get_variable(name).is_some()
    }

    /// 获取上下文深度
    fn get_depth(&self) -> usize {
        0 // 默认实现
    }

    // 图数据库特有功能

    /// 获取顶点引用
    fn get_vertex(&self) -> Option<&crate::core::Vertex>;

    /// 获取边引用
    fn get_edge(&self) -> Option<&crate::core::Edge>;

    /// 获取路径
    fn get_path(&self, name: &str) -> Option<&crate::core::vertex_edge_path::Path>;

    /// 设置顶点
    fn set_vertex(&mut self, vertex: crate::core::Vertex);

    /// 设置边
    fn set_edge(&mut self, edge: crate::core::Edge);

    /// 添加路径
    fn add_path(&mut self, name: String, path: crate::core::vertex_edge_path::Path);

    /// 检查是否为空上下文
    fn is_empty(&self) -> bool;

    /// 获取变量数量
    fn variable_count(&self) -> usize;

    /// 获取所有变量名（返回String类型）
    fn variable_names(&self) -> Vec<String>;

    /// 获取所有变量
    fn get_all_variables(&self) -> Option<std::collections::HashMap<String, Value>>;

    /// 清空所有数据
    fn clear(&mut self);
}
