//! 批量操作测试
//!
//! 测试范围：
//! - 批量添加
//! - 批量更新
//! - 批量删除

use inversearch_service::search::search;

use crate::common::{basic_search_options, create_empty_index};

/// 测试批量添加文档
#[test]
fn test_batch_add() {
    let mut index = create_empty_index();

    // 批量添加文档
    let docs: Vec<(u64, &str)> = (1..=100).map(|i| (i, "batch test content")).collect();

    for (id, content) in docs {
        index.add(id, content, false).unwrap();
    }

    // 验证可以搜索到
    let options = basic_search_options("batch");
    let result = search(&index, &options).unwrap();
    assert_eq!(result.results.len(), 100, "应该返回 100 个结果");
}

/// 测试批量删除文档
#[test]
fn test_batch_remove() {
    let mut index = create_empty_index();

    // 先添加文档
    for i in 1..=50 {
        index.add(i, &format!("Document {}", i), false).unwrap();
    }

    // 批量删除偶数 ID 的文档
    for i in (2..=50).step_by(2) {
        index.remove(i, false).unwrap();
    }

    // 验证奇数 ID 的文档仍然存在
    for i in (1..=50).step_by(2) {
        assert!(index.contains(i), "文档 {} 应该仍然存在", i);
    }

    // 验证偶数 ID 的文档已被删除
    for i in (2..=50).step_by(2) {
        assert!(!index.contains(i), "文档 {} 应该已被删除", i);
    }
}

/// 测试大批量操作性能
#[test]
fn test_large_batch_performance() {
    let mut index = create_empty_index();

    // 添加大量文档
    let start = std::time::Instant::now();
    for i in 1..=1000 {
        index
            .add(i, &format!("Performance test document number {}", i), false)
            .unwrap();
    }
    let add_duration = start.elapsed();

    // 搜索 - 使用较大的 limit 来获取所有结果
    let start = std::time::Instant::now();
    let mut options = basic_search_options("Performance");
    options.limit = Some(1000); // 显式设置 limit 为 1000 以获取所有结果
    let result = search(&index, &options).unwrap();
    let search_duration = start.elapsed();

    assert_eq!(result.results.len(), 1000, "应该返回 1000 个结果");

    // 输出性能信息（仅用于参考，不作为断言）
    println!("添加 1000 个文档耗时: {:?}", add_duration);
    println!("搜索耗时: {:?}", search_duration);
}

/// 测试混合批量操作
#[test]
fn test_mixed_batch_operations() {
    let mut index = create_empty_index();

    // 添加文档
    for i in 1..=20 {
        index.add(i, &format!("Original {}", i), false).unwrap();
    }

    // 更新部分文档
    for i in 1..=10 {
        index.update(i, &format!("Updated {}", i)).unwrap();
    }

    // 删除部分文档
    for i in 11..=15 {
        index.remove(i, false).unwrap();
    }

    // 验证更新后的文档
    for i in 1..=10 {
        let options = basic_search_options(&format!("Updated {}", i));
        let result = search(&index, &options).unwrap();
        assert!(result.results.contains(&i), "应该找到更新后的文档 {}", i);
    }

    // 验证删除的文档
    for i in 11..=15 {
        assert!(!index.contains(i), "文档 {} 应该已被删除", i);
    }

    // 验证未修改的文档
    for i in 16..=20 {
        assert!(index.contains(i), "文档 {} 应该仍然存在", i);
    }
}
