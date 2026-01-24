//! 表达式上下文特征定义（拆分版本）
//!
//! 将大的 ExpressionContext Trait 拆分为多个小 Trait，提高可维护性和扩展性

use crate::core::error::ExpressionError;
use crate::core::Value;
use crate::core::Expression;
use crate::expression::functions::FunctionRef;

/// 变量上下文特征
///
/// 提供基本的变量访问和管理功能
pub trait VariableContext {
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

    /// 获取变量数量
    fn variable_count(&self) -> usize;

    /// 获取所有变量
    fn get_all_variables(&self) -> Option<std::collections::HashMap<String, Value>>;

    /// 清空所有变量
    fn clear_variables(&mut self);
}

/// 版本化变量上下文特征
///
/// 提供变量版本管理功能
pub trait VersionedContext {
    /// 获取最新版本 (version = 0)
    fn get_latest_version(&self, name: &str) -> Option<Value>;

    /// 获取指定版本
    ///
    /// version = 0: 最新版本
    /// version = -1: 前一个版本
    /// version = 1: 最老版本
    fn get_versioned_variable(&self, name: &str, version: i64) -> Option<Value>;

    /// 设置新版本（追加到历史）
    fn set_versioned_variable(&mut self, name: String, value: Value);

    /// 获取版本数量
    fn version_count(&self, name: &str) -> usize;
}

/// 图数据库上下文特征
///
/// 提供图数据库特有的顶点、边、路径访问功能
pub trait GraphContext {
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
}

/// 函数上下文特征
///
/// 提供函数注册和调用功能
pub trait FunctionContext {
    /// 获取函数
    fn get_function(&self, name: &str) -> Option<FunctionRef>;

    /// 检查函数是否存在
    fn has_function(&self, name: &str) -> bool {
        self.get_function(name).is_some()
    }

    /// 获取所有函数名
    fn get_function_names(&self) -> Vec<&str>;
}

/// 缓存上下文特征
///
/// 提供缓存管理功能
pub trait CacheContext {
    /// 获取或编译正则表达式
    fn get_regex(&mut self, pattern: &str) -> Option<&regex::Regex>;
}

/// 作用域上下文特征
///
/// 提供嵌套作用域支持
pub trait ScopedContext {
    /// 获取上下文深度
    fn get_depth(&self) -> usize;

    /// 创建子上下文
    fn create_child_context(&self) -> Box<dyn ExpressionContext>;
}

/// 表达式上下文特征（组合版本）
///
/// 组合所有小 Trait，提供完整的上下文功能
pub trait ExpressionContext:
    VariableContext
    + GraphContext
    + FunctionContext
    + CacheContext
    + ScopedContext
{
    /// 检查是否为空上下文
    fn is_empty(&self) -> bool;

    /// 清空所有数据
    fn clear(&mut self);
}

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
        true
    }

    /// 获取求值器名称
    fn name(&self) -> &str;

    /// 获取求值器描述
    fn description(&self) -> &str;

    /// 获取求值器版本
    fn version(&self) -> &str;
}
