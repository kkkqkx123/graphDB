# GraphDB E2E 测试实现指南

## 1. 测试框架搭建

### 1.1 目录结构创建

```
tests/e2e/
├── common/
│   ├── mod.rs
│   ├── scenarios.rs
│   ├── data_generators.rs
│   └── assertions.rs
├── scenarios/
│   ├── mod.rs
│   ├── social_network.rs
│   ├── e_commerce.rs
│   ├── knowledge_graph.rs
│   └── recommendation.rs
├── workflows/
│   ├── mod.rs
│   ├── schema_evolution.rs
│   ├── data_migration.rs
│   └── backup_restore.rs
├── performance/
│   ├── mod.rs
│   ├── bulk_insert.rs
│   ├── concurrent_queries.rs
│   └── large_graph.rs
└── regression/
    ├── mod.rs
    └── bug_fixes.rs
```

### 1.2 基础测试基础设施

#### 1.2.1 E2E 测试上下文

E2E 测试需要维护完整的应用状态，包括：

- 服务实例（GraphService）
- 会话管理（SessionManager）
- 存储引擎（Storage）
- 配置管理（Config）

#### 1.2.2 数据准备工具

需要实现以下数据生成工具：

- **SocialNetworkDataGenerator**: 生成社交网络测试数据
- **ECommerceDataGenerator**: 生成电商测试数据
- **KnowledgeGraphDataGenerator**: 生成知识图谱测试数据
- **PerformanceDataGenerator**: 生成大规模性能测试数据

#### 1.2.3 断言工具

扩展断言工具以支持：

- 查询结果集比较
- 图结构验证
- 性能指标检查
- 数据一致性验证

---

## 2. 测试用例实现规范

### 2.1 测试用例结构

每个 E2E 测试用例应遵循以下结构：

```rust
#[tokio::test]
async fn test_case_id_description() {
    // 1. 测试准备
    let ctx = E2eTestContext::new().await;
    let data_gen = SocialNetworkDataGenerator::new(&ctx);
    
    // 2. 数据准备
    data_gen.generate_base_schema().await;
    data_gen.generate_test_data(100).await;
    
    // 3. 执行测试步骤
    let result = ctx.execute_query("MATCH (n) RETURN n").await;
    
    // 4. 验证结果
    assert!(result.is_ok());
    assert_result_count(&result, 100);
    
    // 5. 清理（自动）
}
```

### 2.2 命名规范

- **测试文件**: `场景名_类型.rs`（如 `social_network_scenario.rs`）
- **测试函数**: `test_模块_功能_场景`（如 `test_sns_friend_discovery_mutual`）
- **测试数据**: `场景名_数据类型_编号`（如 `sns_person_base_001`）

### 2.3 文档规范

每个测试用例必须包含：

```rust
/// 测试用例: TC-SN-01
/// 名称: 用户注册与好友添加流程
/// 优先级: P0
/// 
/// # 前置条件
/// - 空数据库
/// 
/// # 执行步骤
/// 1. 创建图空间
/// 2. 创建标签和边类型
/// 3. 插入用户数据
/// 4. 建立好友关系
/// 
/// # 预期结果
/// - 所有操作成功
/// - 查询返回正确结果
#[tokio::test]
async fn test_sns_user_registration_and_friend_addition() {
    // ...
}
```

---

## 3. 场景测试实现

### 3.1 社交网络场景

#### 3.1.1 数据模型定义

```rust
// 顶点类型定义
pub struct Person {
    pub id: i64,
    pub name: String,
    pub age: i32,
    pub city: String,
    pub created_at: DateTime,
}

pub struct Post {
    pub id: i64,
    pub content: String,
    pub created_at: DateTime,
    pub likes: i32,
}

// 边类型定义
pub struct Knows {
    pub since: Date,
    pub strength: f64,
}

pub struct Follows {
    pub since: Date,
}
```

#### 3.1.2 测试数据生成

```rust
pub struct SocialNetworkDataGenerator {
    ctx: Arc<E2eTestContext>,
}

impl SocialNetworkDataGenerator {
    /// 生成基础模式
    pub async fn generate_base_schema(&self) -> Result<()> {
        let schema_queries = vec![
            "CREATE SPACE IF NOT EXISTS social_network",
            "USE social_network",
            "CREATE TAG IF NOT EXISTS Person(name STRING, age INT, city STRING, created_at TIMESTAMP)",
            "CREATE TAG IF NOT EXISTS Post(content STRING, created_at TIMESTAMP, likes INT)",
            "CREATE EDGE IF NOT EXISTS KNOWS(since DATE, strength DOUBLE)",
            "CREATE EDGE IF NOT EXISTS FOLLOWS(since DATE)",
            // ...
        ];
        
        for query in schema_queries {
            self.ctx.execute_query(query).await?;
        }
        Ok(())
    }
    
    /// 生成社交网络数据
    pub async fn generate_social_graph(&self, user_count: usize) -> Result<SocialGraph> {
        // 生成用户
        // 生成好友关系
        // 生成动态
        // 返回图数据引用
    }
}
```

#### 3.1.3 核心测试用例

**TC-SN-01: 用户注册与好友添加**

```rust
#[tokio::test]
async fn test_sns_user_registration_and_friend_addition() {
    let ctx = E2eTestContext::new().await.expect("创建上下文失败");
    let generator = SocialNetworkDataGenerator::new(&ctx);
    
    // 准备基础模式
    generator.generate_base_schema().await.expect("创建模式失败");
    
    // 创建用户
    let create_users = r#"
        INSERT VERTEX Person(name, age, city) 
        VALUES 1:('Alice', 25, 'Beijing'), 2:('Bob', 28, 'Shanghai')
    "#;
    let result = ctx.execute_query(create_users).await;
    assert!(result.is_ok(), "创建用户失败: {:?}", result.err());
    
    // 建立好友关系
    let create_friendship = r#"
        INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2024-01-01')
    "#;
    let result = ctx.execute_query(create_friendship).await;
    assert!(result.is_ok(), "创建好友关系失败: {:?}", result.err());
    
    // 验证好友关系
    let query = "GO FROM 1 OVER KNOWS YIELD dst(edge)";
    let result = ctx.execute_query(query).await.expect("查询失败");
    
    // 验证结果
    let rows = result.data.unwrap().rows;
    assert_eq!(rows.len(), 1, "应该返回一个好友");
    assert_eq!(rows[0].values[0], Value::Int(2));
}
```

**TC-SN-02: 多层好友关系查询**

```rust
#[tokio::test]
async fn test_sns_multi_level_friend_query() {
    let ctx = E2eTestContext::new().await.unwrap();
    let generator = SocialNetworkDataGenerator::new(&ctx);
    
    // 生成 3 层好友网络
    let graph = generator.generate_social_graph(100).await.unwrap();
    graph.create_friendship_network(3).await.unwrap();
    
    // 测试 2 层好友查询
    let query = "GO 2 STEPS FROM 1 OVER KNOWS YIELD dst(edge)";
    let result = ctx.execute_query(query).await.expect("查询失败");
    
    // 验证返回了 2 层好友
    assert!(!result.data.unwrap().rows.is_empty());
    
    // 测试路径查找
    let path_query = "FIND ALL PATH FROM 1 TO 50 OVER KNOWS";
    let path_result = ctx.execute_query(path_query).await;
    assert!(path_result.is_ok());
}
```

**TC-SN-03: 动态发布与互动**

```rust
#[tokio::test]
async fn test_sns_post_and_interaction() {
    let ctx = E2eTestContext::new().await.unwrap();
    let generator = SocialNetworkDataGenerator::new(&ctx);
    
    generator.generate_base_schema().await.unwrap();
    generator.generate_social_graph(10).await.unwrap();
    
    // 创建帖子
    let create_post = r#"
        INSERT VERTEX Post(content, created_at, likes) 
        VALUES 100:('Hello GraphDB!', now(), 0)
    "#;
    ctx.execute_query(create_post).await.unwrap();
    
    // 用户发布帖子
    let post_relation = "INSERT EDGE POSTED VALUES 1 -> 100";
    ctx.execute_query(post_relation).await.unwrap();
    
    // 其他用户点赞
    let like = r#"
        INSERT EDGE LIKES(created_at) VALUES 2 -> 100:(now())
    "#;
    ctx.execute_query(like).await.unwrap();
    
    // 查询帖子及互动
    let query = r#"
        MATCH (p:Person)-[:POSTED]->(post:Post)<-[:LIKES]-(liker:Person) 
        RETURN p.name, post.content, liker.name
    "#;
    let result = ctx.execute_query(query).await.expect("查询失败");
    
    // 验证结果包含作者和点赞者
    let rows = result.data.unwrap().rows;
    assert!(!rows.is_empty(), "应该返回互动记录");
}
```

### 3.2 电商推荐场景

#### 3.2.1 数据模型定义

```rust
pub struct User {
    pub id: i64,
    pub name: String,
    pub age: i32,
    pub gender: String,
    pub city: String,
}

pub struct Product {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub price: f64,
    pub brand: String,
}

pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
}
```

#### 3.2.2 核心测试用例

**TC-EC-03: 相似商品推荐**

```rust
#[tokio::test]
async fn test_ecommerce_similar_product_recommendation() {
    let ctx = E2eTestContext::new().await.unwrap();
    let generator = ECommerceDataGenerator::new(&ctx);
    
    generator.generate_base_schema().await.unwrap();
    generator.generate_products(1000).await.unwrap();
    generator.generate_similarity_relations().await.unwrap();
    
    // 查询相似商品
    let query = r#"
        MATCH (p:Product)-[s:SIMILAR_TO]->(similar:Product)
        WHERE p.id == 100
        RETURN similar.name, s.similarity_score
        ORDER BY s.similarity_score DESC
        LIMIT 10
    "#;
    
    let result = ctx.execute_query(query).await.expect("查询失败");
    let rows = result.data.unwrap().rows;
    
    assert_eq!(rows.len(), 10, "应该返回 10 个相似商品");
    
    // 验证相似度排序
    let scores: Vec<f64> = rows.iter()
        .map(|r| r.values[1].as_f64().unwrap())
        .collect();
    
    for i in 1..scores.len() {
        assert!(scores[i-1] >= scores[i], "相似度应该按降序排列");
    }
}
```

**TC-EC-04: 协同过滤推荐**

```rust
#[tokio::test]
async fn test_ecommerce_collaborative_filtering() {
    let ctx = E2eTestContext::new().await.unwrap();
    let generator = ECommerceDataGenerator::new(&ctx);
    
    generator.generate_base_schema().await.unwrap();
    generator.generate_users(100).await.unwrap();
    generator.generate_products(500).await.unwrap();
    generator.generate_purchase_history(1000).await.unwrap();
    
    // 为目标用户生成推荐
    let target_user_id = 1;
    let query = format!(r#"
        // 找到购买行为相似的用户
        MATCH (u:User)-[:PURCHASED]->(p:Product)<-[:PURCHASED]-(similar:User)
        WHERE u.id == {}
        WITH similar, count(p) AS common_products
        ORDER BY common_products DESC
        LIMIT 10
        
        // 获取相似用户购买但目标用户未购买的商品
        MATCH (similar)-[:PURCHASED]->(rec:Product)
        WHERE NOT (u)-[:PURCHASED]->(rec)
        
        // 按购买频率排序
        RETURN rec.name, count(*) AS score
        ORDER BY score DESC
        LIMIT 10
    "#, target_user_id);
    
    let result = ctx.execute_query(&query).await.expect("查询失败");
    let rows = result.data.unwrap().rows;
    
    assert!(!rows.is_empty(), "应该返回推荐商品");
}
```

---

## 4. 性能测试实现

### 4.1 并发写入测试

```rust
#[tokio::test]
async fn test_concurrent_bulk_insert() {
    let ctx = E2eTestContext::new().await.unwrap();
    let generator = PerformanceDataGenerator::new(&ctx);
    
    generator.generate_base_schema().await.unwrap();
    
    let concurrency = 10;
    let batch_size = 1000;
    
    let mut handles = vec![];
    
    for i in 0..concurrency {
        let ctx_clone = ctx.clone();
        let handle = tokio::spawn(async move {
            let start_id = i * batch_size;
            let mut queries = vec![];
            
            for j in 0..batch_size {
                let id = start_id + j;
                queries.push(format!(
                    "INSERT VERTEX Person(name, age) VALUES {}:('User{}', {})",
                    id, id, 20 + (id % 50) as i32
                ));
            }
            
            for query in queries {
                ctx_clone.execute_query(&query).await?;
            }
            
            Ok::<_, anyhow::Error>(())
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
    let count_query = "MATCH (n:Person) RETURN count(n)";
    let result = ctx.execute_query(count_query).await.unwrap();
    let count = result.data.unwrap().rows[0].values[0].as_i64().unwrap();
    
    assert_eq!(count, (concurrency * batch_size) as i64);
    
    // 性能断言
    let throughput = (concurrency * batch_size) as f64 / duration.as_secs_f64();
    println!("插入吞吐量: {:.2} 条/秒", throughput);
    assert!(throughput > 1000.0, "插入吞吐量应大于 1000 条/秒");
}
```

### 4.2 并发查询测试

```rust
#[tokio::test]
async fn test_concurrent_query_performance() {
    let ctx = E2eTestContext::new().await.unwrap();
    let generator = PerformanceDataGenerator::new(&ctx);
    
    // 准备测试数据
    generator.generate_large_graph(100000, 500000).await.unwrap();
    
    let concurrency = 50;
    let queries_per_client = 100;
    
    let queries = vec![
        "MATCH (n:Person) WHERE n.age > 30 RETURN n LIMIT 100",
        "GO FROM 1 OVER KNOWS YIELD dst(edge) LIMIT 50",
        "MATCH (p:Person)-[:KNOWS]->(friend:Person) RETURN p.name, friend.name LIMIT 100",
    ];
    
    let mut handles = vec![];
    let latencies = Arc::new(Mutex::new(Vec::new()));
    
    for i in 0..concurrency {
        let ctx_clone = ctx.clone();
        let queries_clone = queries.clone();
        let latencies_clone = latencies.clone();
        
        let handle = tokio::spawn(async move {
            for _ in 0..queries_per_client {
                let query = &queries_clone[i % queries_clone.len()];
                let start = Instant::now();
                let result = ctx_clone.execute_query(query).await;
                let latency = start.elapsed();
                
                assert!(result.is_ok(), "查询失败: {:?}", result.err());
                latencies_clone.lock().unwrap().push(latency);
            }
        });
        handles.push(handle);
    }
    
    // 等待所有查询完成
    let start = Instant::now();
    for handle in handles {
        handle.await.unwrap();
    }
    let total_duration = start.elapsed();
    
    // 计算性能指标
    let latencies = latencies.lock().unwrap();
    let avg_latency = latencies.iter().sum::<Duration>() / latencies.len() as u32;
    let p99_latency = latencies[latencies.len() * 99 / 100];
    let qps = (concurrency * queries_per_client) as f64 / total_duration.as_secs_f64();
    
    println!("平均延迟: {:?}", avg_latency);
    println!("P99 延迟: {:?}", p99_latency);
    println!("QPS: {:.2}", qps);
    
    assert!(avg_latency < Duration::from_millis(100), "平均延迟应小于 100ms");
    assert!(qps > 100.0, "QPS 应大于 100");
}
```

---

## 5. CI/CD 集成

### 5.1 GitHub Actions 配置

```yaml
# .github/workflows/e2e-test.yml
name: E2E Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  e2e-test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Cache cargo dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run E2E tests
      run: |
        cargo test --test e2e -- --nocapture
      env:
        RUST_LOG: info
        E2E_TEST_TIMEOUT: 300
    
    - name: Generate test report
      if: always()
      run: |
        cargo test --test e2e -- --format json > e2e-test-results.json
    
    - name: Upload test results
      if: always()
      uses: actions/upload-artifact@v3
      with:
        name: e2e-test-results
        path: e2e-test-results.json
```

### 5.2 测试选择策略

```rust
// 根据变更选择测试
pub fn select_tests_by_changes(changed_files: &[String]) -> Vec<String> {
    let mut selected_tests = vec![];
    
    for file in changed_files {
        if file.contains("storage") {
            selected_tests.extend(vec![
                "test_storage_*",
                "test_concurrent_*",
            ]);
        }
        if file.contains("query") {
            selected_tests.extend(vec![
                "test_sns_*",
                "test_ecommerce_*",
                "test_kg_*",
            ]);
        }
        // ...
    }
    
    selected_tests
}
```

---

## 6. 调试与故障排查

### 6.1 测试失败诊断

```rust
/// 启用详细日志记录
pub async fn run_with_detailed_logging<F, Fut>(test_fn: F) 
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    // 设置日志级别
    std::env::set_var("RUST_LOG", "debug");
    
    // 记录测试开始
    log::info!("=== 测试开始 ===");
    
    let start = Instant::now();
    let result = test_fn().await;
    let duration = start.elapsed();
    
    // 记录测试结果
    match &result {
        Ok(_) => log::info!("=== 测试通过，耗时: {:?} ===", duration),
        Err(e) => {
            log::error!("=== 测试失败，耗时: {:?} ===", duration);
            log::error!("错误信息: {}", e);
            
            // 保留测试数据用于调试
            log::info!("测试数据已保留在: {}", std::env::temp_dir().display());
        }
    }
    
    result
}
```

### 6.2 性能分析

```rust
/// 性能分析工具
pub struct PerformanceProfiler {
    measurements: Vec<Measurement>,
}

impl PerformanceProfiler {
    pub fn record_query(&mut self, query: &str, duration: Duration) {
        self.measurements.push(Measurement {
            query: query.to_string(),
            duration,
            timestamp: Instant::now(),
        });
    }
    
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== 性能分析报告 ===\n");
        
        // 按查询类型分组统计
        let mut by_type: HashMap<String, Vec<Duration>> = HashMap::new();
        for m in &self.measurements {
            let query_type = self.classify_query(&m.query);
            by_type.entry(query_type).or_default().push(m.duration);
        }
        
        for (query_type, durations) in by_type {
            let avg = durations.iter().sum::<Duration>() / durations.len() as u32;
            let max = durations.iter().max().unwrap();
            let min = durations.iter().min().unwrap();
            
            report.push_str(&format!(
                "{}: 平均={:?}, 最大={:?}, 最小={:?}, 次数={}\n",
                query_type, avg, max, min, durations.len()
            ));
        }
        
        report
    }
}
```

---

## 7. 最佳实践

### 7.1 测试隔离

- 每个测试用例使用独立的图空间
- 测试完成后自动清理数据
- 避免测试间的数据依赖

### 7.2 数据管理

- 使用数据生成器创建一致的测试数据
- 对大数据集使用延迟加载
- 复用静态测试数据

### 7.3 性能考虑

- 设置合理的测试超时
- 并行执行独立的测试
- 使用内存存储加速测试

### 7.4 可维护性

- 测试代码与业务代码同等重要
- 定期重构测试代码
- 保持测试文档同步更新
