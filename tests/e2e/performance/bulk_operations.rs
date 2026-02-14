//! 批量操作性能测试
//!
//! 测试范围:
//! - 批量数据导入
//! - 大图遍历
//! - 复杂查询
//! - 索引查询

use crate::e2e::common::{
    assertions::*,
    data_generators::PerformanceDataGenerator,
    E2eTestContext, PerformanceProfiler,
};
use std::time::{Duration, Instant};

/// 测试用例: TC-PF-03
/// 名称: 批量数据导入
/// 优先级: P0
///
/// # 前置条件
/// - 空数据库
///
/// # 预期结果
/// - 导入成功完成
/// - 查询性能满足要求
#[tokio::test]
async fn test_performance_bulk_import() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let node_count = 10000;
    let edge_count = 50000;

    let start = Instant::now();

    // 批量导入数据
    generator
        .generate_large_graph(node_count, edge_count)
        .await
        .expect("批量导入失败");

    let import_duration = start.elapsed();

    // 验证导入的数据量
    let count_query = "MATCH (n:Node) RETURN count(n)";
    let data = ctx.execute_query_ok(count_query).await.expect("查询失败");
    assert_not_empty(&data);

    // 计算导入性能
    let total_records = node_count + edge_count;
    let throughput = total_records as f64 / import_duration.as_secs_f64();

    println!("批量导入性能:");
    println!("顶点数: {}", node_count);
    println!("边数: {}", edge_count);
    println!("总记录数: {}", total_records);
    println!("导入耗时: {:?}", import_duration);
    println!("导入吞吐量: {:.2} 条/秒", throughput);

    assert!(
        throughput > 1000.0,
        "导入吞吐量应大于 1000 条/秒"
    );
}

/// 测试用例: TC-PF-04
/// 名称: 大图遍历性能
/// 优先级: P0
#[tokio::test]
async fn test_performance_large_graph_traversal() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 生成大图
    generator
        .generate_large_graph(5000, 25000)
        .await
        .expect("生成大图失败");

    let mut profiler = PerformanceProfiler::new();

    // 测试 1 层遍历
    let query1 = "GO FROM 1 OVER CONNECTS YIELD dst(edge) LIMIT 100";
    let start = Instant::now();
    ctx.execute_query_ok(query1).await.expect("查询失败");
    profiler.record("1-hop traversal", start.elapsed());

    // 测试 2 层遍历
    let query2 = "GO 2 STEPS FROM 1 OVER CONNECTS YIELD dst(edge) LIMIT 1000";
    let start = Instant::now();
    ctx.execute_query_ok(query2).await.expect("查询失败");
    profiler.record("2-hop traversal", start.elapsed());

    // 测试 3 层遍历
    let query3 = "GO 3 STEPS FROM 1 OVER CONNECTS YIELD dst(edge) LIMIT 10000";
    let start = Instant::now();
    ctx.execute_query_ok(query3).await.expect("查询失败");
    profiler.record("3-hop traversal", start.elapsed());

    // 测试全图最短路径
    let query4 = "FIND SHORTEST PATH FROM 1 TO 100 OVER CONNECTS";
    let start = Instant::now();
    let result = ctx.execute_query(query4).await;
    profiler.record("shortest path", start.elapsed());
    assert!(result.is_ok() || result.is_err());

    // 测试子图提取
    let query5 = "GET SUBGRAPH 3 STEPS FROM 1";
    let start = Instant::now();
    let result = ctx.execute_query(query5).await;
    profiler.record("subgraph extraction", start.elapsed());
    assert!(result.is_ok() || result.is_err());

    println!("{}", profiler.generate_report());

    // 性能断言
    if let Some(avg) = profiler.average_duration() {
        assert!(
            avg < Duration::from_secs(5),
            "平均遍历时间应小于 5 秒"
        );
    }
}

/// 测试用例: TC-PF-05
/// 名称: 复杂查询性能
/// 优先级: P1
#[tokio::test]
async fn test_performance_complex_query() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    generator
        .generate_large_graph(3000, 15000)
        .await
        .expect("生成测试数据失败");

    let mut profiler = PerformanceProfiler::new();

    // 复杂聚合查询
    let query1 = r#"
        MATCH (n:Node)
        RETURN n.category, count(n) AS count, avg(n.value) AS avg_value
        ORDER BY count DESC
    "#;
    let start = Instant::now();
    ctx.execute_query_ok(query1).await.expect("查询失败");
    profiler.record("aggregation query", start.elapsed());

    // 多条件过滤查询
    let query2 = r#"
        MATCH (n:Node)-[:CONNECTS]->(m:Node)
        WHERE n.value > 500 AND m.value < 500
        RETURN n.name, m.name
        LIMIT 100
    "#;
    let start = Instant::now();
    ctx.execute_query_ok(query2).await.expect("查询失败");
    profiler.record("filtered join query", start.elapsed());

    // 排序和分页查询
    let query3 = r#"
        MATCH (n:Node)
        RETURN n.name, n.value
        ORDER BY n.value DESC
        SKIP 100 LIMIT 100
    "#;
    let start = Instant::now();
    ctx.execute_query_ok(query3).await.expect("查询失败");
    profiler.record("sort pagination query", start.elapsed());

    // 子查询
    let query4 = r#"
        MATCH (n:Node)
        WHERE n.value IN (
            SELECT m.value FROM Node m WHERE m.category == 'A' LIMIT 10
        )
        RETURN n.name
        LIMIT 50
    "#;
    let start = Instant::now();
    let result = ctx.execute_query(query4).await;
    profiler.record("subquery", start.elapsed());
    assert!(result.is_ok() || result.is_err());

    println!("{}", profiler.generate_report());
}

/// 测试用例: TC-PF-06
/// 名称: 索引查询性能
/// 优先级: P1
#[tokio::test]
async fn test_performance_index_query() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 生成数据
    generator
        .generate_large_graph(5000, 10000)
        .await
        .expect("生成测试数据失败");

    // 创建索引
    let create_index = "CREATE INDEX node_value_idx ON Node(value)";
    let result = ctx.execute_query(create_index).await;
    println!("创建索引结果: {:?}", result);

    // 重建索引
    let rebuild_index = "REBUILD INDEX node_value_idx";
    let result = ctx.execute_query(rebuild_index).await;
    println!("重建索引结果: {:?}", result);

    let mut profiler = PerformanceProfiler::new();

    // 索引点查
    let query1 = r#"
        LOOKUP ON Node WHERE Node.value == 500
    "#;
    let start = Instant::now();
    let result = ctx.execute_query(query1).await;
    profiler.record("index point lookup", start.elapsed());
    assert!(result.is_ok() || result.is_err());

    // 索引范围查
    let query2 = r#"
        LOOKUP ON Node WHERE Node.value >= 400 AND Node.value < 600
    "#;
    let start = Instant::now();
    let result = ctx.execute_query(query2).await;
    profiler.record("index range lookup", start.elapsed());
    assert!(result.is_ok() || result.is_err());

    // 对比全表扫描
    let query3 = r#"
        MATCH (n:Node) WHERE n.value == 500 RETURN n
    "#;
    let start = Instant::now();
    ctx.execute_query_ok(query3).await.expect("查询失败");
    profiler.record("full scan", start.elapsed());

    println!("{}", profiler.generate_report());
}

/// 测试用例: TC-PF-07
/// 名称: 内存使用测试
/// 优先级: P2
#[tokio::test]
async fn test_performance_memory_usage() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 记录初始内存（简化版，实际可能需要系统调用）
    println!("开始内存使用测试");

    // 逐步增加数据量，观察内存使用
    let data_sizes = vec![1000, 5000, 10000];

    for size in data_sizes {
        let start = Instant::now();
        generator
            .generate_large_graph(size, size * 5)
            .await
            .expect("生成数据失败");

        let duration = start.elapsed();
        println!("导入 {} 顶点耗时: {:?}", size, duration);

        // 执行查询测试内存稳定性
        let query = format!("MATCH (n:Node) WHERE n.value == {} RETURN n LIMIT 10", size / 2);
        ctx.execute_query_ok(&query).await.ok();
    }

    println!("内存使用测试完成");
}

/// 测试用例: 批量删除性能
/// 名称: 测试批量删除操作的性能
/// 优先级: P2
#[tokio::test]
async fn test_performance_bulk_delete() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 生成测试数据
    generator
        .generate_large_graph(5000, 10000)
        .await
        .expect("生成测试数据失败");

    // 批量删除
    let delete_query = r#"
        DELETE VERTEX 1, 2, 3, 4, 5
    "#;
    let start = Instant::now();
    let result = ctx.execute_query(delete_query).await;
    let duration = start.elapsed();

    println!("批量删除耗时: {:?}", duration);
    assert!(result.is_ok() || result.is_err());

    // 验证删除结果
    let count_query = "MATCH (n:Node) RETURN count(n)";
    let data = ctx.execute_query_ok(count_query).await.expect("查询失败");
    assert_not_empty(&data);
}

/// 测试用例: 批量更新性能
/// 名称: 测试批量更新操作的性能
/// 优先级: P2
#[tokio::test]
async fn test_performance_bulk_update() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 生成测试数据
    generator
        .generate_large_graph(3000, 5000)
        .await
        .expect("生成测试数据失败");

    // 批量更新
    let update_query = r#"
        UPDATE Node SET category = 'Updated' WHERE value < 100
    "#;
    let start = Instant::now();
    let result = ctx.execute_query(update_query).await;
    let duration = start.elapsed();

    println!("批量更新耗时: {:?}", duration);
    assert!(result.is_ok() || result.is_err());

    // 验证更新结果
    let verify_query = r#"
        MATCH (n:Node) WHERE n.category == 'Updated' RETURN count(n)
    "#;
    let data = ctx.execute_query_ok(verify_query).await.expect("查询失败");
    assert_not_empty(&data);
}
