//! 上下文基础定义
//!
//! 提供上下文管理器等高级功能

use crate::core::Value;

// 重新导出上下文类型
pub use super::ContextType;

/// 上下文管理器特征
///
/// 管理上下文生命周期的特征，使用类型安全的方法
pub trait ContextManager {
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

/// 简单的事件监听器
#[derive(Debug, Clone)]
pub struct SimpleEventListener {
    /// 事件历史
    events: Vec<ContextEvent>,
}

impl SimpleEventListener {
    /// 创建新的简单事件监听器
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// 获取事件历史
    pub fn get_events(&self) -> &[ContextEvent] {
        &self.events
    }

    /// 清空事件历史
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// 获取事件数量
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// 添加事件
    pub fn add_event(&mut self, event: ContextEvent) {
        self.events.push(event);
    }
}

impl Default for SimpleEventListener {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextEventListener for SimpleEventListener {
    fn on_event(&self, _event: &ContextEvent) {
        // 简单监听器只记录事件，实际处理由外部调用者决定
        // 这里可以改为使用内部可变性或者返回事件供外部处理
    }
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
    pub contexts_by_type: std::collections::HashMap<ContextType, usize>,
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
