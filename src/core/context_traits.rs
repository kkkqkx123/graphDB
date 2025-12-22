//! 上下文特征定义
//!
//! 定义上下文系统的核心特征和类型，避免循环依赖

use crate::core::Value;

/// 上下文类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ContextType {
    /// 会话上下文
    Session,
    /// 查询上下文
    Query,
    /// 执行上下文
    Execution,
    /// 表达式上下文
    Expression,
    /// 请求上下文
    Request,
    /// 运行时上下文
    Runtime,
    /// 验证上下文
    Validation,
    /// 存储上下文
    Storage,
}

/// 上下文基础特征 - 最小化接口
///
/// 只包含所有上下文类型真正需要的基础方法
pub trait ContextBase: std::fmt::Debug {
    /// 获取上下文ID
    fn id(&self) -> &str;

    /// 获取上下文类型
    fn context_type(&self) -> ContextType;

    /// 获取创建时间
    fn created_at(&self) -> std::time::SystemTime;

    /// 获取最后更新时间
    fn updated_at(&self) -> std::time::SystemTime;

    /// 检查上下文是否有效
    fn is_valid(&self) -> bool;
}

/// 可变上下文特征
///
/// 提供可变操作的上下文特征
pub trait MutableContext: ContextBase {
    /// 更新最后更新时间
    fn touch(&mut self);

    /// 标记上下文为无效
    fn invalidate(&mut self);

    /// 重新验证上下文
    fn revalidate(&mut self) -> bool;
}

/// 层次化上下文特征
///
/// 支持层次化结构的上下文特征，用于具有父子关系的上下文
pub trait HierarchicalContext: ContextBase {
    /// 获取父上下文ID（如果存在）
    fn parent_id(&self) -> Option<&str>;

    /// 获取上下文深度
    fn depth(&self) -> usize;
}

/// 属性支持特征
///
/// 为需要属性的上下文提供支持
pub trait AttributeSupport {
    /// 获取自定义属性
    fn get_attribute(&self, key: &str) -> Option<Value>;

    /// 设置自定义属性
    fn set_attribute(&mut self, key: String, value: Value);

    /// 获取所有属性键
    fn attribute_keys(&self) -> Vec<String>;

    /// 移除属性
    fn remove_attribute(&mut self, key: &str) -> Option<Value>;

    /// 清空所有属性
    fn clear_attributes(&mut self);
}