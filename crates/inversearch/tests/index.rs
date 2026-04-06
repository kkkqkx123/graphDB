//! 索引操作集成测试入口
//!
//! 统一导出 index 子目录下的所有测试模块

// 首先声明 common 模块
mod common;

mod index {
    mod crud_test;
    mod batch_test;
    mod clear_test;
}
