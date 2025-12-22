//! 统一上下文基础接口
//!
//! 定义所有上下文类型的基础接口和共同功能

use crate::core::Value;
use std::collections::HashMap;

/// 上下文基础特征
///
/// 所有上下文类型都必须实现此基础特征，提供通用的上下文操作
pub trait ContextBase: std::any::Any + std::fmt::Debug {
    /// 获取上下文ID
    fn id(&self) -> &str;

    /// 获取上下文类型
    fn context_type(&self) -> ContextType;

    /// 获取父上下文（如果存在）
    fn parent(&self) -> Option<&dyn ContextBase> {
        None
    }

    /// 获取上下文深度（默认为0）
    fn depth(&self) -> usize {
        0
    }

    /// 获取Any引用，用于类型转换
    fn as_any(&self) -> &dyn std::any::Any;

    /// 获取创建时间
    fn created_at(&self) -> std::time::SystemTime;

    /// 获取最后更新时间
    fn updated_at(&self) -> std::time::SystemTime;

    /// 检查上下文是否有效
    fn is_valid(&self) -> bool;

    /// 获取自定义属性（返回克隆值）
    fn get_attribute(&self, key: &str) -> Option<Value>;

    /// 设置自定义属性
    fn set_attribute(&mut self, key: String, value: Value);

    /// 获取所有属性键
    fn attribute_keys(&self) -> Vec<String>;

    /// 移除属性
    fn remove_attribute(&mut self, key: &str) -> Option<Value>;

    /// 清空所有属性
    fn clear_attributes(&mut self);

    /// 克隆上下文（深拷贝）
    fn clone_context(&self) -> Box<dyn ContextBase>;
}

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
    /// 创建子上下文
    fn create_child(&self, context_type: ContextType) -> Box<dyn ContextBase>;
}

/// 上下文容器特征
///
/// 支持存储键值对的上下文特征
pub trait ContextContainer: ContextBase {
    /// 获取值
    fn get(&self, key: &str) -> Option<&Value>;

    /// 设置值
    fn set(&mut self, key: String, value: Value);

    /// 检查键是否存在
    fn contains_key(&self, key: &str) -> bool;

    /// 移除键值对
    fn remove(&mut self, key: &str) -> Option<Value>;

    /// 获取所有键
    fn keys(&self) -> Vec<&str>;

    /// 获取所有值
    fn values(&self) -> Vec<&Value>;

    /// 获取所有键值对
    fn iter(&self) -> std::collections::hash_map::Iter<'_, String, Value>;

    /// 获取键值对数量
    fn len(&self) -> usize;

    /// 检查是否为空
    fn is_empty(&self) -> bool;

    /// 清空所有键值对
    fn clear(&mut self);

    /// 批量设置键值对
    fn extend(&mut self, other: HashMap<String, Value>);
}

/// 上下文管理器特征
///
/// 管理上下文生命周期的特征
pub trait ContextManager {
    /// 创建上下文
    fn create_context(&mut self, context_type: ContextType) -> Box<dyn ContextBase>;

    /// 获取上下文
    fn get_context(&self, id: &str) -> Option<&dyn ContextBase>;

    /// 获取可变上下文
    fn get_context_mut(&mut self, id: &str) -> Option<Box<dyn ContextBase>>;

    /// 移除上下文
    fn remove_context(&mut self, id: &str) -> Option<Box<dyn ContextBase>>;

    /// 清理过期上下文
    fn cleanup_expired_contexts(&mut self);

    /// 获取所有上下文ID
    fn context_ids(&self) -> Vec<String>;

    /// 获取上下文数量
    fn context_count(&self) -> usize;
}

/// 上下文事件类型
#[derive(Debug, Clone)]
pub enum ContextEvent {
    /// 上下文创建
    Created {
        id: String,
        context_type: ContextType,
        timestamp: std::time::SystemTime,
    },
    /// 上下文更新
    Updated {
        id: String,
        timestamp: std::time::SystemTime,
    },
    /// 上下文销毁
    Destroyed {
        id: String,
        timestamp: std::time::SystemTime,
    },
    /// 属性变更
    AttributeChanged {
        id: String,
        key: String,
        old_value: Option<Value>,
        new_value: Value,
        timestamp: std::time::SystemTime,
    },
}

/// 上下文事件监听器特征
pub trait ContextEventListener: std::fmt::Debug {
    /// 处理上下文事件
    fn on_event(&self, event: &ContextEvent);
}

/// 上下文统计信息
#[derive(Debug, Clone, Default)]
pub struct ContextStatistics {
    /// 创建的上下文总数
    pub total_created: usize,
    /// 销毁的上下文总数
    pub total_destroyed: usize,
    /// 当前活跃的上下文数量
    pub active_contexts: usize,
    /// 按类型分组的上下文数量
    pub contexts_by_type: HashMap<ContextType, usize>,
    /// 平均上下文生命周期（毫秒）
    pub average_lifetime_ms: f64,
    /// 最大上下文深度
    pub max_depth: usize,
}

impl ContextStatistics {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录上下文创建
    pub fn record_created(&mut self, context_type: ContextType) {
        self.total_created += 1;
        self.active_contexts += 1;
        *self.contexts_by_type.entry(context_type).or_insert(0) += 1;
    }

    /// 记录上下文销毁
    pub fn record_destroyed(&mut self, context_type: ContextType, lifetime_ms: u64) {
        self.total_destroyed += 1;
        self.active_contexts = self.active_contexts.saturating_sub(1);

        // 更新平均生命周期
        let total_completed = self.total_destroyed as f64;
        let current_avg = self.average_lifetime_ms;
        self.average_lifetime_ms =
            (current_avg * (total_completed - 1.0) + lifetime_ms as f64) / total_completed;

        // 更新类型计数
        if let Some(count) = self.contexts_by_type.get_mut(&context_type) {
            *count = count.saturating_sub(1);
        }
    }

    /// 更新最大深度
    pub fn update_max_depth(&mut self, depth: usize) {
        if depth > self.max_depth {
            self.max_depth = depth;
        }
    }

    /// 重置统计信息
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// 上下文配置
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// 最大上下文深度
    pub max_depth: usize,
    /// 上下文超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    /// 是否启用自动清理
    pub enable_auto_cleanup: bool,
    /// 清理间隔（秒）
    pub cleanup_interval_seconds: u64,
    /// 最大活跃上下文数量
    pub max_active_contexts: Option<usize>,
    /// 是否启用事件监听
    pub enable_event_listening: bool,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_depth: 100,
            timeout_ms: Some(300000), // 5分钟
            enable_auto_cleanup: true,
            cleanup_interval_seconds: 60, // 1分钟
            max_active_contexts: Some(10000),
            enable_event_listening: false,
        }
    }
}
