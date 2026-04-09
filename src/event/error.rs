//! 事件系统错误类型

use thiserror::Error;

/// 事件系统错误
#[derive(Debug, Error)]
pub enum EventError {
    /// 处理器错误
    #[error("Handler error: {0}")]
    HandlerError(String),

    /// 订阅不存在
    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(u64),

    /// 事件队列已满
    #[error("Event queue is full")]
    QueueFull,

    /// 内部错误
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<std::io::Error> for EventError {
    fn from(err: std::io::Error) -> Self {
        EventError::InternalError(err.to_string())
    }
}

/// 事件处理器错误结果
pub type EventHandlerResult<T> = Result<T, EventError>;
