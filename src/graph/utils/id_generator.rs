//! ID生成器模块 - 提供ID生成功能
//! 对应原C++中的IdGenerator.h/cpp

use std::sync::atomic::{AtomicI64, Ordering};

/// ID生成器
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
}
