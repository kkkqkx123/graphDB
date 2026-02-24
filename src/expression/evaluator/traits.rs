//! 表达式上下文特征定义
//!
//! 为图数据库表达式求值提供统一的上下文接口

use crate::core::Value;
use crate::expression::functions::FunctionRef;

/// 表达式上下文特征
///
/// 为图数据库表达式求值提供统一的上下文接口
pub trait ExpressionContext {
    /// 获取变量值
    fn get_variable(&self, name: &str) -> Option<Value>;

    /// 设置变量值
    fn set_variable(&mut self, name: String, value: Value);

    /// 获取函数引用
    fn get_function(&self, _name: &str) -> Option<FunctionRef> {
        None
    }

    /// 检查上下文是否支持缓存
    fn supports_cache(&self) -> bool {
        false
    }

    /// 获取缓存管理器（如果支持）
    fn get_cache(&mut self) -> Option<&mut crate::expression::context::cache_manager::CacheManager> {
        None
    }
}
