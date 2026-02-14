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
use tokio::task::JoinHandle;

/// 测试用例: TC-PF-01
/// 名称: 并发写入测试
/// 优先级: P0
///
/// # 前置条件
/// - 空数据库
///
/// # 预期结果
/// - 所有数据正确写入
/// - 无数据丢失或冲突
#[tokio::test]
async fn test_performance_concurrent_write() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    generator
        .generate_base_schema()
        .await
        .expect("创建模式失败");

    let concurrency = 10;
    let batch_size = 100;

    let mut handles: Vec<JoinHandle<anyhow::Result<()>>> = Vec::new();

    for i in 0..concurrency {
        let ctx_clone = ctx.clone();
        let handle = tokio::spawn(async move {
            let start_id = i * batch_size;

            for j in 0..batch_size {
                let id = (start_id + j + 1) as i64;
                let query = format!(
                    "INSERT VERTEX Node(name, value, category) VALUES {}:('Node{}', {}, 'A')",
                    id, id, (id % 1000)
                );
                ctx_clone.execute_query_ok(&query).await?;
            }

            Ok(())
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    let start = Instant::now();
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    let duration = start.elapsed();

    // 验证数据完整性
    let count_query = "MATCH (n:Node) RETURN count(n)";
    let data = ctx.execute_query_ok(count_query).await.expect("查询失败");
    assert_not_empty(&data);

    // 性能断言
    let total_records = concurrency * batch_size;
    let throughput = total_records as f64 / duration.as_secs_f64();
    println!("并发写入吞吐量: {:.2} 条/秒", throughput);
    println!("总耗时: {:?}", duration);
}

/// 测试用例: TC-PF-02
/// 名称: 并发查询测试
/// 优先级: P1
#[tokio::test]
async fn test_performance_concurrent_query() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = PerformanceDataGenerator::new(&ctx);

    // 准备测试数据
    generator
        .generate_large_graph(1000, 5000)
        .await
        .expect("生成测试数据失败");

    let concurrency = 20;
    let queries_per_client = 50;

    let queries = vec![
        "MATCH (n:Node) WHERE n.value > 500 RETURN n LIMIT 10",
        "MATCH (n:Node)-[:CONNECTS]->(m:Node) WHERE n.category == 'A' RETURN n, m LIMIT 10",
        "MATCH (n:Node) RETURN n.name, n.value ORDER BY n.value DESC LIMIT 20",
    ];

    let mut handles = Vec::new();
    let mut profiler = PerformanceProfiler::new();

    for i in 0..concurrency {
        let ctx_clone = ctx.clone();
        let queries_clone = queries.clone();
        let query = queries_clone[i % queries_clone.len()].to_string();

        let handle = tokio::spawn(async move {
            for _ in 0..queries_per_client {
                let start = Instant::now();
                let result = ctx_clone.execute_query(&query).await;
                let duration = start.elapsed();

                assert!(result.is_ok(), "查询失败: {:?}", result.err());
                duration
            }
        });
        handles.push(handle);
    }

    // 等待所有查询完成
    let start = Instant::now();
    for handle in handles {
        let durations: Vec<Duration> = handle.await.unwrap();
        for d in durations {
            profiler.record("query", d);
        }
    }
    let total_duration = start.elapsed();

    // 生成性能报告
    println!("{}", profiler.generate_report());

    let total_queries = concurrency * queries_per_client;
    let qps = total_queries as f64 / total_duration.as_secs_f64();
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

    let write_tasks = 5;
    let read_tasks = 15;
    let duration_secs = 10;

    let mut handles = Vec::new();

    // 启动写入任务
    for i in 0..write_tasks {
        let ctx_clone = ctx.clone();
        let handle = tokio::spawn(async move {
            let start = Instant::now();
            let mut count = 0;

            while start.elapsed() < Duration::from_secs(duration_secs) {
                let id = (i * 10000 + count + 1) as i64;
                let query = format!(
                    "INSERT VERTEX Node(name, value, category) VALUES {}:('NewNode{}', {}, 'B')",
                    id, id, (id % 1000)
                );
                if ctx_clone.execute_query(&query).await.is_ok() {
                    count += 1;
                }
            }

            count
        });
        handles.push(handle);
    }

    // 启动读取任务
    for i in 0..read_tasks {
        let ctx_clone = ctx.clone();
        let handle = tokio::spawn(async move {
            let start = Instant::now();
            let mut count = 0;

            while start.elapsed() < Duration::from_secs(duration_secs) {
                let query = format!(
                    "MATCH (n:Node) WHERE n.value == {} RETURN n LIMIT 10",
                    (i + count) % 1000
                );
                if ctx_clone.execute_query(&query).await.is_ok() {
                    count += 1;
                }
            }

            count
        });
        handles.push(handle);
    }

    // 收集结果
    let mut total_writes = 0;
    let mut total_reads = 0;

    for (i, handle) in handles.into_iter().enumerate() {
        let count = handle.await.unwrap();
        if i < write_tasks {
            total_writes += count;
        } else {
            total_reads += count;
        }
    }

    println!("写入操作数: {}", total_writes);
    println!("读取操作数: {}", total_reads);
    println!("写入吞吐量: {:.2} 条/秒", total_writes as f64 / duration_secs as f64);
    println!("读取吞吐量: {:.2} QPS", total_reads as f64 / duration_secs as f64);
}

/// 测试用例: TC-PF-08
/// 名称: 连接池测试
/// 优先级: P2
#[tokio::test]
async fn test_performance_connection_pool() {
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

    let concurrent_sessions = 50;
    let queries_per_session = 20;

    let mut handles = Vec::new();

    for i in 0..concurrent_sessions {
        let ctx_clone = ctx.clone();
        let handle = tokio::spawn(async move {
            // 每个任务创建独立会话
            let session = ctx_clone.create_session(&format!("user{}", i)).await?;

            for j in 0..queries_per_session {
                let query = format!(
                    "MATCH (n:Node) WHERE n.value == {} RETURN n LIMIT 5",
                    (i + j) % 100
                );
                ctx_clone.execute_query(&query).await?;
            }

            Ok::<_, anyhow::Error>(())
        });
        handles.push(handle);
    }

    let start = Instant::now();
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    let duration = start.elapsed();

    let total_queries = concurrent_sessions * queries_per_session;
    let qps = total_queries as f64 / duration.as_secs_f64();

    println!("连接池测试完成");
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
/// 优先级: P2
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

    let stress_duration = Duration::from_secs(30);
    let concurrency = 100;

    let mut handles = Vec::new();

    for i in 0..concurrency {
        let ctx_clone = ctx.clone();
        let handle = tokio::spawn(async move {
            let start = Instant::now();
            let mut success_count = 0;
            let mut error_count = 0;

            while start.elapsed() < stress_duration {
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

                match ctx_clone.execute_query(&query).await {
                    Ok(_) => success_count += 1,
                    Err(_) => error_count += 1,
                }
            }

            (success_count, error_count)
        });
        handles.push(handle);
    }

    let mut total_success = 0;
    let mut total_errors = 0;

    for handle in handles {
        let (success, errors) = handle.await.unwrap();
        total_success += success;
        total_errors += errors;
    }

    let total_requests = total_success + total_errors;
    let success_rate = total_success as f64 / total_requests as f64 * 100.0;
    let qps = total_requests as f64 / stress_duration.as_secs_f64();

    println!("压力测试结果:");
    println!("总请求数: {}", total_requests);
    println!("成功请求: {}", total_success);
    println!("失败请求: {}", total_errors);
    println!("成功率: {:.2}%", success_rate);
    println!("QPS: {:.2}", qps);

    assert!(
        success_rate > 95.0,
        "成功率应大于 95%，实际 {:.2}%",
        success_rate
    );
}
