# Explain/Profile功能在测试验证中的应用

## 1. 当前Explain/Profile功能概述

### 1.1 Explain功能

**文件位置**: `src/query/validator/utility/explain_validator.rs`

**主要功能**:
- 分析查询结构而不实际执行
- 生成查询计划描述
- 支持多种输出格式（Table, Dot）
- 验证内部语句的语法和语义

**输出列**:
- `id` - 节点ID
- `name` - 节点名称
- `dependencies` - 依赖关系
- `profiling_data` - 性能数据
- `operator info` - 操作符信息

**局限性**:
- 不执行实际查询
- 无法验证数据变更效果
- 无法验证查询结果正确性

### 1.2 Profile功能

**文件位置**: 同一文件中的 `ProfileValidator`

**主要功能**:
- 实际执行查询
- 收集性能统计信息

**性能统计** (`ProfilingStats`):
```rust
pub struct ProfilingStats {
    pub rows: i64,                    // 处理的行数
    pub exec_duration_in_us: i64,     // 执行时间（微秒）
    pub total_duration_in_us: i64,    // 总时间（微秒）
    pub other_stats: HashMap<String, String>,
}
```

**可用于测试验证**:
- 执行时间是否符合预期
- 处理的行数是否正确
- 查询计划是否使用了预期的索引

## 2. 将Explain/Profile集成到测试框架

### 2.1 扩展现有测试场景

```rust
impl TestScenario {
    /// 执行EXPLAIN并返回计划描述
    pub fn explain(&mut self, query: &str) -> &mut Self {
        let explain_query = format!("EXPLAIN {}", query);
        match self.pipeline.execute_query(&explain_query) {
            Ok(result) => {
                self.last_result = Some(result);
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("{:?}", e));
                self.last_result = None;
            }
        }
        self
    }

    /// 执行PROFILE并返回性能统计
    pub fn profile(&mut self, query: &str) -> &mut Self {
        let profile_query = format!("PROFILE {}", query);
        match self.pipeline.execute_query(&profile_query) {
            Ok(result) => {
                self.last_result = Some(result);
                self.last_error = None;
            }
            Err(e) => {
                self.last_error = Some(format!("{:?}", e));
                self.last_result = None;
            }
        }
        self
    }
}
```

### 2.2 添加Explain/Profile专用断言

```rust
impl TestScenario {
    /// 断言查询计划包含特定操作符
    pub fn assert_plan_contains(&self, operator: &str) -> &Self {
        if let Some(ref result) = self.last_result {
            let plan_text = self.extract_plan_text(result);
            assert!(
                plan_text.contains(operator),
                "Expected plan to contain '{}', but got: {}",
                operator,
                plan_text
            );
        } else {
            panic!("No explain result to check");
        }
        self
    }

    /// 断言查询计划使用了索引
    pub fn assert_plan_uses_index(&self, index_name: &str) -> &Self {
        self.assert_plan_contains(&format!("IndexScan: {}", index_name))
    }

    /// 断言查询计划使用了全表扫描
    pub fn assert_plan_uses_full_scan(&self) -> &Self {
        self.assert_plan_contains("FullScan")
    }

    /// 断言执行时间小于阈值（毫秒）
    pub fn assert_execution_time_less_than(&self, threshold_ms: i64) -> &Self {
        if let Some(ref result) = self.last_result {
            if let Some(duration) = self.extract_execution_time(result) {
                assert!(
                    duration < threshold_ms * 1000, // 转换为微秒
                    "Expected execution time < {}ms, but got {}μs",
                    threshold_ms,
                    duration
                );
            }
        } else {
            panic!("No profile result to check");
        }
        self
    }

    /// 断言处理的行数
    pub fn assert_rows_processed(&self, expected: i64) -> &Self {
        if let Some(ref result) = self.last_result {
            if let Some(rows) = self.extract_rows_processed(result) {
                assert_eq!(
                    rows, expected,
                    "Expected {} rows processed, but got {}",
                    expected, rows
                );
            }
        } else {
            panic!("No profile result to check");
        }
        self
    }
}
```

## 3. 测试用例示例

### 3.1 验证查询计划使用索引

```rust
#[test]
fn test_query_uses_index() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_ddl("CREATE TAG INDEX idx_person_age ON Person(age)")
        .assert_success()
        // 验证索引扫描被使用
        .explain("MATCH (p:Person) WHERE p.age == 25 RETURN p.name")
        .assert_plan_uses_index("idx_person_age")
        // 对比：无索引时的全表扫描
        .explain("MATCH (p:Person) WHERE p.name == 'Alice' RETURN p.age")
        .assert_plan_uses_full_scan();
}
```

### 3.2 验证查询性能

```rust
#[test]
fn test_query_performance() {
    let mut scenario = TestScenario::new();
    scenario
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success();
    
    // 插入大量数据
    for i in 0..1000 {
        scenario.exec_dml(&format!(
            "INSERT VERTEX Person(name, age) VALUES {}:('Person{}', {})",
            i, i, i % 100
        ));
    }
    
    scenario
        // 无索引查询性能
        .profile("MATCH (p:Person) WHERE p.age == 50 RETURN p.name")
        .assert_execution_time_less_than(100) // 应该较慢
        // 创建索引
        .exec_ddl("CREATE TAG INDEX idx_person_age ON Person(age)")
        .assert_success()
        // 有索引查询性能
        .profile("MATCH (p:Person) WHERE p.age == 50 RETURN p.name")
        .assert_execution_time_less_than(10); // 应该更快
}
```

### 3.3 验证查询计划结构

```rust
#[test]
fn test_complex_query_plan() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        .exec_ddl("CREATE EDGE KNOWS(since DATE)")
        .assert_success()
        // 验证JOIN操作符存在
        .explain(r#"
            MATCH (a:Person)-[:KNOWS]->(b:Person)
            WHERE a.age > 25 AND b.age < 40
            RETURN a.name, b.name
        "#)
        .assert_plan_contains("Join")
        .assert_plan_contains("Filter")
        .assert_plan_contains("EdgeScan");
}
```

### 3.4 验证数据变更影响

```rust
#[test]
fn test_profile_shows_affected_rows() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Person(name STRING, age INT)")
        .assert_success()
        // 插入数据
        .exec_dml(r#"
            INSERT VERTEX Person(name, age) VALUES 
                1:('Alice', 30),
                2:('Bob', 25),
                3:('Charlie', 30)
        "#)
        .assert_success()
        // 验证UPDATE影响的行数
        .profile("UPDATE 1 SET age = 31")
        .assert_rows_processed(1)
        // 验证批量UPDATE影响的行数
        .profile("UPDATE 1, 2, 3 SET age = age + 1")
        .assert_rows_processed(3);
}
```

## 4. 与现有测试框架的集成建议

### 4.1 扩展现有模块

在 `tests/common/test_scenario.rs` 中添加：

```rust
// Explain/Profile支持
pub fn explain(&mut self, query: &str) -> &mut Self;
pub fn profile(&mut self, query: &str) -> &mut Self;

// 断言方法
pub fn assert_plan_contains(&self, operator: &str) -> &Self;
pub fn assert_plan_uses_index(&self, index_name: &str) -> &Self;
pub fn assert_execution_time_less_than(&self, threshold_ms: i64) -> &Self;
pub fn assert_rows_processed(&self, expected: i64) -> &Self;
```

### 4.2 创建专用测试文件

```
tests/
├── integration_explain_profile.rs    # Explain/Profile专用测试
└── integration_performance.rs        # 性能测试
```

### 4.3 示例测试文件

```rust
//! Explain/Profile Integration Tests

mod common;
use common::test_scenario::TestScenario;

#[test]
fn test_index_usage_verification() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG User(name STRING, age INT)")
        .assert_success()
        // 无索引时应该使用全表扫描
        .explain("MATCH (u:User) WHERE u.age == 25 RETURN u.name")
        .assert_plan_contains("FullScan")
        // 创建索引
        .exec_ddl("CREATE TAG INDEX idx_user_age ON User(age)")
        .assert_success()
        // 有索引时应该使用索引扫描
        .explain("MATCH (u:User) WHERE u.age == 25 RETURN u.name")
        .assert_plan_contains("IndexScan");
}

#[test]
fn test_query_performance_baseline() {
    TestScenario::new()
        .setup_space("test_space")
        .exec_ddl("CREATE TAG Item(name STRING, price DOUBLE)")
        .assert_success()
        // 插入测试数据
        .exec_dml("INSERT VERTEX Item(name, price) VALUES 1:('Item1', 100.0)")
        .assert_success()
        // 验证简单查询性能
        .profile("MATCH (i:Item) RETURN i.name")
        .assert_execution_time_less_than(100)
        .assert_rows_processed(1);
}
```

## 5. 总结

### 5.1 Explain/Profile在测试中的价值

1. **查询计划验证**
   - 确保查询使用预期的执行策略
   - 验证索引被正确使用
   - 检测意外的全表扫描

2. **性能回归测试**
   - 建立性能基线
   - 检测性能退化
   - 验证优化效果

3. **执行统计验证**
   - 验证DML语句影响的行数
   - 验证查询返回的行数
   - 监控资源使用情况

### 5.2 实现建议

1. **短期**：扩展现有 `TestScenario` 添加Explain/Profile支持
2. **中期**：创建专用的性能测试套件
3. **长期**：集成性能回归检测到CI/CD流程

### 5.3 注意事项

1. **性能测试的稳定性**
   - 在隔离环境中运行
   - 多次执行取平均值
   - 设置合理的阈值范围

2. **Explain输出的可维护性**
   - 避免过度依赖具体计划格式
   - 关注关键操作符而非完整计划
   - 定期更新测试以适应优化器改进
