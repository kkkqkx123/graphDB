# 集成测试现状分析与扩展方案

## 1. 现有测试分析

### 1.1 测试文件概览

当前项目包含以下集成测试文件：

| 文件 | 测试类型 | 当前测试数量 | 主要问题 |
|------|----------|--------------|----------|
| `integration_cypher_create.rs` | Cypher CREATE语句 | ~20个 | 仅验证解析，执行测试无实际断言 |
| `integration_dcl.rs` | 数据控制语言 | ~15个 | 执行测试仅打印结果，无状态验证 |
| `integration_ddl.rs` | 数据定义语言 | ~25个 | 缺乏Schema变更后的验证 |
| `integration_dml.rs` | 数据操作语言 | ~20个 | INSERT/UPDATE/DELETE无数据验证 |
| `integration_dql.rs` | 数据查询语言 | ~30个 | 查询结果无内容验证 |

### 1.2 当前测试模式分析

#### 1.2.1 解析测试模式（普遍存在）

```rust
#[test]
fn test_create_tag_parser_basic() {
    let query = "CREATE TAG Person(name: STRING, age: INT)";
    let mut parser = Parser::new(query);
    let result = parser.parse();
    assert!(result.is_ok(), "...");
    let stmt = result.expect("...");
    assert_eq!(stmt.ast.stmt.kind(), "CREATE");
}
```

**局限性**：
- 仅验证语法解析是否成功
- 不验证语义正确性
- 不验证执行结果

#### 1.2.2 执行测试模式（普遍存在）

```rust
#[test]
fn test_create_tag_execution_basic() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    // ... 创建pipeline_manager
    let query = "CREATE TAG Person(name: STRING, age: INT)";
    let result = pipeline_manager.execute_query(query);
    println!("CREATE TAG基础执行结果: {:?}", result);
    assert!(result.is_ok() || result.is_err()); // 无实际断言
}
```

**局限性**：
- 执行结果仅打印，无验证
- `assert!(result.is_ok() || result.is_err())` 是永真式
- 不验证数据是否实际写入存储
- 不验证返回结果的内容

### 1.3 测试基础设施分析

#### 1.3.1 现有基础设施 (`tests/common/`)

| 模块 | 功能 | 问题 |
|------|------|------|
| `mod.rs` | TestStorage封装 | 仅提供存储实例，无数据验证工具 |
| `assertions.rs` | 基础断言函数 | 过于简单，无数据库专用断言 |
| `data_fixtures.rs` | 测试数据生成 | 仅创建内存对象，无数据导入功能 |
| `storage_helpers.rs` | 存储辅助 | 需要查看实际内容 |

#### 1.3.2 执行结果类型分析

`ExecutionResult`枚举定义在 `src/query/executor/base/execution_result.rs`：

```rust
pub enum ExecutionResult {
    Values(Vec<Value>),
    Vertices(Vec<Vertex>),
    Edges(Vec<Edge>),
    DataSet(DataSet),
    Result(CoreResult),
    Empty,
    Success,
    Error(String),
    Count(usize),
    Paths(Vec<Path>),
}
```

**问题**：测试代码未充分利用这些变体进行验证。

### 1.4 Explain/Profile功能分析

#### 1.4.1 Explain功能

位置：`src/query/validator/utility/explain_validator.rs`

功能：
- 分析查询结构
- 支持多种输出格式（Table, Dot）
- 验证内部语句

**局限性**：
- 仅用于查询计划分析
- 不收集实际执行数据
- 无法用于验证数据变更

#### 1.4.2 Profile功能

位置：与Explain同一文件

功能：
- 实际执行查询
- 收集性能统计

**可用于测试验证**：
- 执行时间统计
- 行数统计
- 但无法验证数据正确性

## 2. 核心问题总结

### 2.1 测试覆盖问题

1. **存在性检查 vs 正确性检查**
   - 当前：验证语句能否解析/执行
   - 缺失：验证执行结果是否正确

2. **状态验证缺失**
   - 创建后无法验证对象存在
   - 更新后无法验证属性变更
   - 删除后无法验证对象消失

3. **数据流验证缺失**
   - 插入数据后无法查询验证
   - 更新操作无法验证前后状态
   - 复杂查询无法验证结果集

### 2.2 测试可维护性问题

1. **测试代码重复**
   - 每个测试都重复创建TestStorage和PipelineManager
   - 无统一的测试数据准备和清理

2. **断言弱**
   - `assert!(result.is_ok() || result.is_err())` 无意义
   - 缺少领域特定的断言宏

3. **测试隔离性**
   - 部分测试依赖执行顺序
   - 缺少事务回滚机制

## 3. 扩展方案设计

### 3.1 目标

1. **验证实际执行效果**：不仅验证语句执行，还验证数据状态
2. **支持数据流测试**：插入→查询→更新→查询→删除的完整流程
3. **提供丰富的断言工具**：针对图数据库的专用断言
4. **保持测试隔离**：每个测试独立，自动清理

### 3.2 架构设计

```
tests/
├── common/
│   ├── mod.rs                    # 基础TestStorage
│   ├── assertions.rs             # 扩展断言库
│   ├── data_fixtures.rs          # 测试数据集
│   ├── query_helpers.rs          # 查询辅助函数
│   ├── validation_helpers.rs     # 数据验证辅助
│   └── test_scenario.rs          # 测试场景封装
├── integration_cypher_create.rs  # 扩展后的测试
├── integration_dcl.rs
├── integration_ddl.rs
├── integration_dml.rs
├── integration_dql.rs
└── test_data/                    # 预设测试数据
    ├── social_network.cypher
    └── e_commerce.cypher
```

### 3.3 关键组件设计

#### 3.3.1 扩展断言库

```rust
// 顶点断言
pub fn assert_vertex_exists(storage: &TestStorage, vid: i64, tag: &str);
pub fn assert_vertex_props(storage: &TestStorage, vid: i64, expected: HashMap<&str, Value>);
pub fn assert_vertex_count(storage: &TestStorage, tag: &str, expected: usize);

// 边断言
pub fn assert_edge_exists(storage: &TestStorage, src: i64, dst: i64, edge_type: &str);
pub fn assert_edge_props(storage: &TestStorage, src: i64, dst: i64, edge_type: &str, expected: HashMap<&str, Value>);

// 查询结果断言
pub fn assert_result_count(result: &ExecutionResult, expected: usize);
pub fn assert_result_contains(result: &ExecutionResult, expected: Vec<Value>);
pub fn assert_result_columns(result: &ExecutionResult, expected: Vec<&str>);
```

#### 3.3.2 测试场景封装

```rust
pub struct TestScenario {
    storage: TestStorage,
    pipeline: QueryPipelineManager,
}

impl TestScenario {
    // 执行DDL
    pub fn exec_ddl(&mut self, query: &str) -> &mut Self;
    
    // 执行DML并验证
    pub fn exec_dml(&mut self, query: &str) -> Result<ExecutionResult, DBError>;
    
    // 执行查询并验证结果
    pub fn query(&mut self, query: &str) -> Result<ExecutionResult, DBError>;
    
    // 验证顶点存在
    pub fn assert_vertex(&self, vid: i64, tag: &str) -> &Self;
    
    // 验证边存在
    pub fn assert_edge(&self, src: i64, dst: i64, edge_type: &str) -> &Self;
    
    // 清理数据
    pub fn cleanup(&mut self);
}
```

#### 3.3.3 数据流测试支持

```rust
// 完整的数据流测试示例
#[test]
fn test_insert_and_query_flow() {
    TestScenario::new()
        .exec_ddl("CREATE TAG IF NOT EXISTS Person(name STRING, age INT)")
        .exec_ddl("CREATE EDGE IF NOT EXISTS KNOWS(since DATE)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 1:('Alice', 30)")
        .exec_dml("INSERT VERTEX Person(name, age) VALUES 2:('Bob', 25)")
        .exec_dml("INSERT EDGE KNOWS(since) VALUES 1 -> 2:('2024-01-01')")
        .assert_vertex(1, "Person")
        .assert_vertex(2, "Person")
        .assert_edge(1, 2, "KNOWS")
        .query("MATCH (n:Person) RETURN n.name, n.age")
        .expect_result_count(2)
        .expect_result_contains(vec![Value::String("Alice".into()), Value::Int(30)])
        .cleanup();
}
```

## 4. 实施建议

### 4.1 阶段1：基础设施扩展

1. 扩展 `common/assertions.rs` 添加数据库专用断言
2. 创建 `common/test_scenario.rs` 提供测试场景封装
3. 扩展 `common/data_fixtures.rs` 支持数据导入

### 4.2 阶段2：测试重写

1. 重写 `integration_ddl.rs` 添加Schema变更验证
2. 重写 `integration_dml.rs` 添加数据操作验证
3. 重写 `integration_dql.rs` 添加查询结果验证

### 4.3 阶段3：新增测试

1. 添加数据流测试（跨DDL/DML/DQL）
2. 添加边界条件测试
3. 添加错误处理测试

## 5. 参考资源

### 5.1 内部参考

- `src/query/executor/base/execution_result.rs` - 执行结果类型
- `src/core/query_result/result.rs` - 查询结果结构
- `src/core/value/dataset.rs` - DataSet类型
- `src/query/validator/utility/explain_validator.rs` - Explain/Profile

### 5.2 外部参考

- NebulaGraph BDD测试框架
- Neo4j测试最佳实践
- 图数据库等效查询重写测试方法 (GRev)
