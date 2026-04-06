//! 搜索功能集成测试入口
//!
//! 统一导出 search 子目录下的所有测试模块

// 首先声明 common 模块
mod common;

mod search {
    mod basic_test;
    mod pagination_test;
    mod multi_term_test;
    mod edge_case_test;
}
