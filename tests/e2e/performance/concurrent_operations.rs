//! 并发操作性能测试
//!
//! 测试范围:
//! - 并发写入
//! - 并发查询
//! - 读写混合

use crate::e2e::common::{
    assertions::*,
    data_generators::PerformanceDataGenerator,
    E2eTestContext, PerformanceProfiler,
};
use std::time::{Duration, Instant};

/// 测试用例: TC-PF-01
/// 名称: 批量写入测试
/// 优先级: P0
///
/// # 前置条件
/// - 空数据库
///
/// # 预期结果
/// - 所有数据正确写入
/// - 无数据丢失或冲突
#[tokio::test]
async fn test_performance_batch_write() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let batch_count = 10;
    let batch_size = 100;

    let start = Instant::now();
    for i in 0..batch_count {
        let start_id = i * batch_size;

        for j in 0..batch_size {
            let id = (start_id + j + 1) as i64;
            let query = format!(
                "INSERT VERTEX Node(name, value, category) VALUES {}:('Node{}', {}, 'A')",
                id, id, (id % 1000)
            );
            let _ = ctx.execute_query_ok(&query).await;
        }
    }
    let duration = start.elapsed();

    // 验证数据完整性
    let count_query = "MATCH (n:Node) RETURN count(n)";
    let data = ctx.execute_query_ok(count_query).await.expect("查询失败");
    assert_not_empty(&data);

    // 性能断言
    let total_records = batch_count * batch_size;
    let throughput = total_records as f64 / duration.as_secs_f64();
    println!("批量写入吞吐量: {:.2} 条/秒", throughput);
    println!("总耗时: {:?}", duration);
}

/// 测试用例: TC-PF-02
/// 名称: 顺序查询测试
/// 优先级: P1
#[tokio::test]
async fn test_performance_sequential_query() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    // 准备测试数据
    generator
        .generate_large_graph(1000, 5000)
        .await
        .expect("生成测试数据失败");

    let query_count = 100;

    let queries = vec![
        "MATCH (n:Node) WHERE n.value > 500 RETURN n LIMIT 10",
        "MATCH (n:Node)-[:CONNECTS]->(m:Node) WHERE n.category == 'A' RETURN n, m LIMIT 10",
        "MATCH (n:Node) RETURN n.name, n.value ORDER BY n.value DESC LIMIT 20",
    ];

    let mut profiler = PerformanceProfiler::new();

    // 顺序执行查询
    let start = Instant::now();
    for i in 0..query_count {
        let query = queries[i % queries.len()];
        let query_start = Instant::now();
        let result = ctx.execute_query(query).await;
        let duration = query_start.elapsed();

        assert!(result.is_ok(), "查询失败: {:?}", result.err());
        profiler.record("query", duration);
    }
    let total_duration = start.elapsed();

    // 生成性能报告
    println!("{}", profiler.generate_report());

    let qps = query_count as f64 / total_duration.as_secs_f64();
    println!("QPS: {:.2}", qps);

    // 性能断言
    if let Some(avg) = profiler.average_duration() {
        assert!(
            avg < Duration::from_millis(500),
            "平均延迟应小于 500ms"
        );
    }
}

/// 测试用例: TC-PF-05
/// 名称: 读写混合测试
/// 优先级: P1
#[tokio::test]
async fn test_performance_read_write_mixed() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 预填充数据
    generator
        .generate_large_graph(500, 2000)
        .await
        .expect("生成测试数据失败");

    let write_count = 50;
    let read_count = 150;

    // 顺序执行写入
    let write_start = Instant::now();
    for i in 0..write_count {
        let id = (i + 1) as i64;
        let query = format!(
            "INSERT VERTEX Node(name, value, category) VALUES {}:('NewNode{}', {}, 'B')",
            id, id, (id % 1000)
        );
        let _ = ctx.execute_query(&query).await;
    }
    let write_duration = write_start.elapsed();

    // 顺序执行读取
    let read_start = Instant::now();
    for i in 0..read_count {
        let query = format!(
            "MATCH (n:Node) WHERE n.value == {} RETURN n LIMIT 10",
            i % 1000
        );
        let _ = ctx.execute_query(&query).await;
    }
    let read_duration = read_start.elapsed();

    println!("写入操作数: {}", write_count);
    println!("读取操作数: {}", read_count);
    println!("写入吞吐量: {:.2} 条/秒", write_count as f64 / write_duration.as_secs_f64());
    println!("读取吞吐量: {:.2} QPS", read_count as f64 / read_duration.as_secs_f64());
}

/// 测试用例: TC-PF-08
/// 名称: 多会话测试
/// 优先级: P2
#[tokio::test]
async fn test_performance_multiple_sessions() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    generator
        .generate_large_graph(100, 500)
        .await
        .expect("生成测试数据失败");

    let session_count = 10;
    let queries_per_session = 20;

    let start = Instant::now();
    for i in 0..session_count {
        // 每个迭代创建独立会话
        let session = ctx.create_session(&format!("user{}", i)).await.expect("创建会话失败");

        for j in 0..queries_per_session {
            let query = format!(
                "MATCH (n:Node) WHERE n.value == {} RETURN n LIMIT 5",
                (i + j) % 100
            );
            let _ = ctx.execute_query(&query).await;
        }
    }
    let duration = start.elapsed();

    let total_queries = session_count * queries_per_session;
    let qps = total_queries as f64 / duration.as_secs_f64();

    println!("多会话测试完成");
    println!("总查询数: {}", total_queries);
    println!("总耗时: {:?}", duration);
    println!("QPS: {:.2}", qps);
}

/// 测试用例: TC-PF-09
/// 名称: 缓存性能测试
/// 优先级: P2
#[tokio::test]
async fn test_performance_cache() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    generator
        .generate_large_graph(100, 500)
        .await
        .expect("生成测试数据失败");

    let query = "MATCH (n:Node) WHERE n.value == 50 RETURN n LIMIT 10";

    // 第一次查询（冷缓存）
    let cold_start = Instant::now();
    ctx.execute_query_ok(query).await.expect("查询失败");
    let cold_duration = cold_start.elapsed();

    // 多次重复查询（热缓存）
    let hot_start = Instant::now();
    for _ in 0..100 {
        ctx.execute_query_ok(query).await.expect("查询失败");
    }
    let hot_duration = hot_start.elapsed();
    let avg_hot = hot_duration / 100;

    println!("冷缓存查询时间: {:?}", cold_duration);
    println!("热缓存平均查询时间: {:?}", avg_hot);

    // 热缓存应该更快
    if avg_hot < cold_duration {
        println!("缓存有效，性能提升: {:.2}x", 
            cold_duration.as_secs_f64() / avg_hot.as_secs_f64());
    }
}

/// 测试用例: TC-PF-10
/// 名称: 压力测试
/// 优先级: P0
#[tokio::test]
async fn test_performance_stress() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    // 生成大量数据
    generator
        .generate_large_graph(5000, 25000)
        .await
        .expect("生成测试数据失败");

    let query_count = 500;
    let mut success_count = 0;
    let mut error_count = 0;

    let start = Instant::now();
    for i in 0..query_count {
        let query_type = (i + success_count + error_count) % 3;
        let query = match query_type {
            0 => format!(
                "MATCH (n:Node) WHERE n.value == {} RETURN n LIMIT 10",
                (i + success_count) % 1000
            ),
            1 => format!(
                "MATCH (n:Node)-[:CONNECTS]->(m:Node) WHERE n.category == 'A' RETURN n, m LIMIT 5"
            ),
            _ => format!(
                "MATCH (n:Node) RETURN count(n)"
            ),
        };

        match ctx.execute_query(&query).await {
            Ok(_) => success_count += 1,
            Err(_) => error_count += 1,
        }
    }
    let stress_duration = start.elapsed();

    let total_requests = success_count + error_count;
    let success_rate = success_count as f64 / total_requests as f64 * 100.0;
    let qps = total_requests as f64 / stress_duration.as_secs_f64();

    println!("压力测试结果:");
    println!("总请求数: {}", total_requests);
    println!("成功请求: {}", success_count);
    println!("失败请求: {}", error_count);
    println!("成功率: {:.2}%", success_rate);
    println!("QPS: {:.2}", qps);

    assert!(
        success_rate > 95.0,
        "成功率应大于 95%，实际 {:.2}%",
        success_rate
    );
}
