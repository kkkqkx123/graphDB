//! 自定义断言辅助模块
//!
//! 提供测试中的常用断言函数

#![allow(dead_code)]

/// 断言结果成功，返回内部值
pub fn assert_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    result.expect("操作应该成功")
}

/// 断言集合包含指定数量的元素
pub fn assert_count<T>(collection: &[T], expected: usize, item_name: &str) {
    assert_eq!(
        collection.len(),
        expected,
        "{}数量不匹配: 期望 {}, 实际 {}",
        item_name,
        expected,
        collection.len()
    );
}

/// 断言 Option 是 Some 并返回内部值
pub fn assert_some<T>(opt: &Option<T>) -> &T {
    opt.as_ref().expect("值应该是 Some")
}

/// 断言 Option 是 None
pub fn assert_none<T>(opt: &Option<T>) {
    assert!(opt.is_none(), "值应该是 None");
}
