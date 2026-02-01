//! ID生成器模块 - 提供唯一ID生成功能
//!
//! 提供两种ID生成策略：
//! - IdGenerator: 基于原子计数器的顺序ID生成
//! - generate_id: 基于时间戳的唯一ID生成

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 基于原子计数器的ID生成器
///
/// 线程安全的顺序ID生成器，适用于需要递增ID的场景
#[derive(Debug)]
pub struct IdGenerator {
    counter: AtomicI64,
}

impl IdGenerator {
    /// 创建新的ID生成器，使用指定的初始值
    pub fn new(init: i64) -> Self {
        Self {
            counter: AtomicI64::new(init),
        }
    }

    /// 生成下一个ID
    pub fn id(&self) -> i64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }

    /// 重置计数器到指定值
    pub fn reset(&self, value: i64) {
        self.counter.store(value, Ordering::SeqCst);
    }

    /// 获取当前计数值
    pub fn current_value(&self) -> i64 {
        self.counter.load(Ordering::SeqCst)
    }
}

impl Clone for IdGenerator {
    fn clone(&self) -> Self {
        Self {
            counter: AtomicI64::new(self.current_value()),
        }
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new(0)
    }
}

/// 执行计划ID生成器 - 单例实现
///
/// 用于生成执行计划相关的唯一ID
pub struct EPIdGenerator {
    generator: IdGenerator,
}

impl EPIdGenerator {
    /// 获取单例实例
    pub fn instance() -> &'static Self {
        use std::sync::OnceLock;
        static INSTANCE: OnceLock<EPIdGenerator> = OnceLock::new();
        INSTANCE.get_or_init(|| EPIdGenerator {
            generator: IdGenerator::new(0),
        })
    }

    /// 生成下一个执行计划ID
    pub fn id(&self) -> i64 {
        self.generator.id()
    }

    /// 重置计数器
    pub fn reset(&self, value: i64) {
        self.generator.reset(value);
    }
}

/// 基于时间戳的唯一ID生成
///
/// 使用纳秒级时间戳生成唯一ID，适用于分布式场景或需要全局唯一的ID
pub fn generate_id() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos() as u64
}

/// 验证ID是否有效
///
/// 有效ID必须大于0
pub fn is_valid_id(id: u64) -> bool {
    id != 0
}

/// 无效ID常量
pub const INVALID_ID: i64 = -1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_generator() {
        let gen = IdGenerator::new(0);

        assert_eq!(gen.id(), 0);
        assert_eq!(gen.id(), 1);
        assert_eq!(gen.id(), 2);

        gen.reset(100);
        assert_eq!(gen.current_value(), 100);
        assert_eq!(gen.id(), 100);
    }

    #[test]
    fn test_ep_id_generator() {
        let gen = EPIdGenerator::instance();

        let first_id = gen.id();
        let second_id = gen.id();

        assert_eq!(second_id, first_id + 1);
    }

    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();

        assert_ne!(id1, id2);
        assert!(is_valid_id(id1));
        assert!(is_valid_id(id2));
    }

    #[test]
    fn test_is_valid_id() {
        assert!(is_valid_id(1));
        assert!(is_valid_id(42));
        assert!(is_valid_id(u64::MAX));
        assert!(!is_valid_id(0));
    }
}
