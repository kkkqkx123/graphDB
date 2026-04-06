//! 字符集集成测试入口
//!
//! 统一导出 charset 子目录下的所有测试模块

// 首先声明 common 模块
mod common;

mod charset {
    mod latin_test;
    mod cjk_test;
    mod mixed_test;
}
