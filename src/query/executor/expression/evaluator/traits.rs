//! 表达式求值上下文 trait 定义
//!
//! 为图数据库表达式求值提供统一的上下文接口
//!
//! **注意：** 此 trait 用于运行时表达式求值。
//! 编译时分析请使用 `ExpressionAnalysisContext`。

use crate::core::Value;
use crate::query::executor::expression::functions::FunctionRef;

/// 表达式求值上下文 trait
///
/// 为图数据库表达式求值提供统一的上下文接口
///
/// **注意：** 此 trait 用于运行时表达式求值。
/// 编译时分析请使用 `ExpressionAnalysisContext`。
pub trait ExpressionContext {
    /// 获取变量值
    fn get_variable(&self, name: &str) -> Option<Value>;

    /// 设置变量值
    fn set_variable(&mut self, name: String, value: Value);

    /// 获取函数引用
    fn get_function(&self, name: &str) -> Option<FunctionRef> {
        let _ = name;
        None
    }

    /// 检查上下文是否支持缓存
    fn supports_cache(&self) -> bool {
        false
    }

    /// 获取缓存管理器（如果支持）
    ///
    /// 注意：缓存功能已移除，返回None
    fn get_cache(&mut self) -> Option<&mut ()> {
        None
    }
}
