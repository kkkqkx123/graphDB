//! 统一上下文特征定义
//!
//! 这个模块提供了统一的Context trait，合并了原有的多个上下文特征
//! 包括ContextBase、MutableContext、HierarchicalContext和AttributeSupport

use crate::core::Value;

/// 统一上下文特征
///
/// 这个trait合并了原有的ContextBase、MutableContext、HierarchicalContext和AttributeSupport
/// 提供了所有上下文类型需要的统一接口
///
/// 设计原则：
/// - 必须实现的方法：基础功能（id、context_type、created_at、updated_at、is_valid）
/// - 可变功能：提供默认实现，子类可以覆盖
/// - 层次化功能：提供默认实现，子类可以覆盖
/// - 属性功能：提供默认实现，子类可以覆盖
pub trait BaseContext: std::fmt::Debug + Send + Sync {
    /// 获取上下文ID
    fn id(&self) -> &str;

    /// 获取上下文类型
    fn context_type(&self) -> super::ContextType;

    /// 获取创建时间
    fn created_at(&self) -> std::time::SystemTime;

    /// 获取最后更新时间
    fn updated_at(&self) -> std::time::SystemTime;

    /// 检查上下文是否有效
    fn is_valid(&self) -> bool;

    /// 更新最后更新时间
    ///
    /// 默认实现：将updated_at设置为当前时间
    fn touch(&mut self) {
        let _ = std::time::SystemTime::now();
    }

    /// 标记上下文为无效
    ///
    /// 默认实现：空实现，子类可以覆盖
    fn invalidate(&mut self) {}

    /// 重新验证上下文
    ///
    /// 默认实现：返回true，子类可以覆盖
    fn revalidate(&mut self) -> bool {
        true
    }

    /// 获取父上下文ID（如果存在）
    ///
    /// 默认实现：返回None，子类可以覆盖
    fn parent_id(&self) -> Option<&str> {
        None
    }

    /// 获取上下文深度
    ///
    /// 默认实现：返回0，子类可以覆盖
    fn depth(&self) -> usize {
        0
    }

    /// 获取自定义属性
    ///
    /// 默认实现：返回None，子类可以覆盖
    fn get_attribute(&self, _key: &str) -> Option<Value> {
        None
    }

    /// 设置自定义属性
    ///
    /// 默认实现：空实现，子类可以覆盖
    fn set_attribute(&mut self, _key: String, _value: Value) {}

    /// 获取所有属性键
    ///
    /// 默认实现：返回空向量，子类可以覆盖
    fn attribute_keys(&self) -> Vec<String> {
        Vec::new()
    }

    /// 移除属性
    ///
    /// 默认实现：返回None，子类可以覆盖
    fn remove_attribute(&mut self, _key: &str) -> Option<Value> {
        None
    }

    /// 清空所有属性
    ///
    /// 默认实现：空实现，子类可以覆盖
    fn clear_attributes(&mut self) {}
}

/// 上下文辅助trait - 提供额外的实用方法
///
/// 这个trait为BaseContext提供额外的实用方法，不需要强制实现
pub trait ContextExt: BaseContext {
    /// 检查上下文是否过期
    ///
    /// 默认实现：检查updated_at是否超过指定时间
    fn is_expired(&self, timeout_seconds: u64) -> bool {
        if let Ok(elapsed) = self.updated_at().elapsed() {
            elapsed.as_secs() > timeout_seconds
        } else {
            true
        }
    }

    /// 获取上下文持续时间
    ///
    /// 默认实现：返回从created_at到现在的持续时间
    fn duration(&self) -> Option<std::time::Duration> {
        self.created_at().elapsed().ok()
    }

    /// 检查是否是根上下文（没有父上下文）
    ///
    /// 默认实现：检查parent_id是否为None
    fn is_root(&self) -> bool {
        self.parent_id().is_none()
    }

    /// 获取上下文路径（从根到当前上下文）
    ///
    /// 默认实现：返回包含当前ID的向量
    fn path(&self) -> Vec<String> {
        vec![self.id().to_string()]
    }
}

/// 为所有实现了BaseContext的类型自动实现ContextExt
impl<T: BaseContext> ContextExt for T {}

/// 上下文验证trait - 提供上下文验证功能
///
/// 这个trait为上下文提供验证功能
pub trait ContextValidation: BaseContext {
    /// 验证上下文
    ///
    /// 返回验证结果和错误消息列表
    fn validate(&self) -> (bool, Vec<String>) {
        if self.is_valid() {
            (true, Vec::new())
        } else {
            (false, vec!["上下文无效".to_string()])
        }
    }
}

/// 为所有实现了BaseContext的类型自动实现ContextValidation
impl<T: BaseContext> ContextValidation for T {}
