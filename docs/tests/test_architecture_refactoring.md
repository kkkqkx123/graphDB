# GraphDB 测试架构重构方案

## 1. 概述

本文档定义 GraphDB 项目测试架构的重构方案，解决当前测试文件过长、组织混乱的问题，建立清晰的目录结构和模块化测试体系。

## 2. 设计原则

### 2.1 核心原则

1. **目录模块化** - 按功能领域组织测试，每个领域一个目录
2. **统一导出** - `tests/` 目录下的文件作为构建入口，导出子模块测试
3. **单一职责** - 每个测试文件专注单一功能点，避免过长
4. **共享基础设施** - `tests/common/` 提供统一的测试工具

### 2.2 文件大小限制

| 类型         | 建议行数 | 最大行数 |
| ------------ | -------- | -------- |
| 单个测试文件 | 200-400  | 500      |
| 模块导出文件 | 50-100   | 150      |
| 测试辅助模块 | 100-300  | 400      |

## 3. 新目录结构

```
tests/
├── common/                           # 共享测试基础设施
│   ├── mod.rs                        # 模块导出
│   ├── assertions.rs                 # 断言扩展
│   ├── data_fixtures.rs              # 测试数据集
│   ├── debug_helpers.rs              # 调试辅助
│   ├── fulltext_helpers.rs           # 全文索引辅助
│   ├── query_helpers.rs              # 查询辅助
│   ├── storage_helpers.rs            # 存储辅助
│   ├── sync_helpers.rs               # 同步辅助
│   ├── test_scenario.rs              # 流式测试API
│   ├── transaction_helpers.rs        # 事务辅助
│   └── validation_helpers.rs         # 验证辅助
│
├── dcl/                              # 数据控制语言测试
│   ├── mod.rs                        # 模块导出
│   ├── user_management.rs            # 用户管理测试
│   ├── permission.rs                 # 权限控制测试
│   └── role.rs                       # 角色管理测试
│
├── ddl/                              # 数据定义语言测试
│   ├── mod.rs                        # 模块导出
│   ├── tag_basic.rs                  # Tag基础操作
│   ├── tag_alter.rs                  # Tag修改操作
│   ├── edge_basic.rs                 # Edge基础操作
│   ├── edge_alter.rs                 # Edge修改操作
│   ├── schema_evolution.rs           # Schema演化
│   └── constraints.rs                # 约束测试
│
├── dml/                              # 数据操作语言测试
│   ├── mod.rs                        # 模块导出
│   ├── insert_vertex.rs              # 插入点
│   ├── insert_edge.rs                # 插入边
│   ├── update.rs                     # 更新操作
│   ├── delete.rs                     # 删除操作
│   ├── upsert.rs                     # Upsert操作
│   └── batch_operations.rs           # 批量操作
│
├── dql/                              # 数据查询语言测试
│   ├── mod.rs                        # 模块导出
│   ├── match_basic.rs                # MATCH基础
│   ├── match_pattern.rs              # MATCH模式
│   ├── go_basic.rs                   # GO基础遍历
│   ├── go_advanced.rs                # GO高级遍历
│   ├── lookup.rs                     # LOOKUP查询
│   ├── fetch.rs                      # FETCH查询
│   ├── find_path.rs                  # 路径查找
│   ├── subgraph.rs                   # 子图查询
│   ├── aggregation.rs                # 聚合函数
│   └── pipe.rs                       # 管道操作
│
├── transaction/                      # 事务测试
│   ├── mod.rs
│   ├── basic.rs
│   ├── isolation.rs
│   └── concurrent.rs
│
├── index/                            # 索引测试
│   ├── mod.rs
│   ├── tag_index.rs
│   ├── edge_index.rs
│   └── fulltext.rs
│
├── storage/                          # 存储层测试
│   ├── mod.rs
│   ├── basic.rs
│   └── concurrent.rs
│
├── api/                              # API测试
│   ├── mod.rs
│   ├── embedded.rs
│   └── http.rs
│
├── functions/                        # 函数测试
│   ├── mod.rs
│   ├── string.rs
│   ├── numeric.rs
│   ├── datetime.rs
│   └── aggregate.rs
│
├── fulltext/                         # 全文索引测试
│   ├── mod.rs
│   ├── basic.rs
│   ├── concurrent.rs
│   └── edge_cases.rs
│
├── vector/                           # 向量搜索测试
│   ├── mod.rs
│   ├── basic.rs
│   └── transaction.rs
│
├── sync/                             # 同步测试
│   ├── mod.rs
│   ├── basic.rs
│   ├── 2pc.rs
│   └── fault_tolerance.rs
│
├── e2e/                              # 端到端测试
│   ├── data/
│   ├── hurl/
│   └── python/
│
├── c_api/                            # C API测试
│   └── ...
│
├── integration_dcl.rs                # DCL测试入口 (导出dcl模块)
├── integration_ddl.rs                # DDL测试入口 (导出ddl模块)
├── integration_dml.rs                # DML测试入口 (导出dml模块)
├── integration_dql.rs                # DQL测试入口 (导出dql模块)
├── integration_transaction.rs        # 事务测试入口
├── integration_index.rs              # 索引测试入口
├── integration_storage.rs            # 存储测试入口
├── integration_api.rs                # API测试入口
├── integration_functions.rs          # 函数测试入口
├── integration_fulltext.rs           # 全文索引测试入口
├── integration_vector_search.rs      # 向量搜索测试入口
└── integration_sync.rs               # 同步测试入口
```

## 4. 模块导出规范

### 4.1 子目录模块导出 (mod.rs)

每个测试子目录包含一个 `mod.rs` 文件，负责导出该目录下的所有测试模块：

```rust
// tests/dcl/mod.rs

mod user_management;
mod permission;
mod role;
```

### 4.2 入口文件导出

`tests/` 目录下的入口文件负责导出对应的子目录模块：

```rust
// tests/integration_dcl.rs

mod dcl;
```

### 4.3 测试文件命名规范

| 类型     | 命名格式                    | 示例                     |
| -------- | --------------------------- | ------------------------ |
| 测试函数 | `test_<feature>_<scenario>` | `test_create_user_basic` |
| 测试文件 | `<feature>_<subfeature>.rs` | `user_management.rs`     |
| 入口文件 | `integration_<domain>.rs`   | `integration_dcl.rs`     |

## 5. 测试拆分策略

### 5.1 DCL测试拆分

原文件：`integration_dcl.rs` (915行, 45个测试)

| 新文件                   | 内容                      | 预计测试数 |
| ------------------------ | ------------------------- | ---------- |
| `dcl/user_management.rs` | CREATE/ALTER/DROP USER    | 20         |
| `dcl/permission.rs`      | GRANT/REVOKE              | 15         |
| `dcl/role.rs`            | SHOW ROLES, DESCRIBE USER | 10         |

### 5.2 DDL测试拆分

原文件：`integration_ddl.rs` (999行, 70个测试)

| 新文件                    | 内容              | 预计测试数 |
| ------------------------- | ----------------- | ---------- |
| `ddl/tag_basic.rs`        | CREATE/DROP TAG   | 20         |
| `ddl/tag_alter.rs`        | ALTER TAG         | 15         |
| `ddl/edge_basic.rs`       | CREATE/DROP EDGE  | 15         |
| `ddl/edge_alter.rs`       | ALTER EDGE        | 10         |
| `ddl/schema_evolution.rs` | Schema演化流程    | 5          |
| `ddl/constraints.rs`      | DEFAULT, NOT NULL | 5          |

### 5.3 DML测试拆分

原文件：`integration_dml.rs` (999行, 67个测试)

| 新文件                    | 内容          | 预计测试数 |
| ------------------------- | ------------- | ---------- |
| `dml/insert_vertex.rs`    | INSERT VERTEX | 15         |
| `dml/insert_edge.rs`      | INSERT EDGE   | 10         |
| `dml/update.rs`           | UPDATE操作    | 15         |
| `dml/delete.rs`           | DELETE操作    | 15         |
| `dml/upsert.rs`           | UPSERT, MERGE | 8          |
| `dml/batch_operations.rs` | 批量操作      | 4          |

### 5.4 DQL测试拆分

原文件：`integration_dql.rs` (1156行, 72个测试)

| 新文件                 | 内容          | 预计测试数 |
| ---------------------- | ------------- | ---------- |
| `dql/match_basic.rs`   | MATCH基础     | 12         |
| `dql/match_pattern.rs` | MATCH模式匹配 | 10         |
| `dql/go_basic.rs`      | GO基础遍历    | 12         |
| `dql/go_advanced.rs`   | GO高级遍历    | 10         |
| `dql/lookup.rs`        | LOOKUP查询    | 8          |
| `dql/fetch.rs`         | FETCH查询     | 8          |
| `dql/find_path.rs`     | FIND PATH     | 6          |
| `dql/subgraph.rs`      | GET SUBGRAPH  | 4          |
| `dql/aggregation.rs`   | 聚合函数      | 5          |
| `dql/pipe.rs`          | 管道操作      | 5          |

## 6. 迁移步骤

### 6.1 第一阶段：创建目录结构

1. 创建 `tests/dcl/`, `tests/ddl/`, `tests/dml/`, `tests/dql/` 目录
2. 创建各目录的 `mod.rs` 文件

### 6.2 第二阶段：拆分测试文件

1. 按功能将测试从原文件移动到新文件
2. 更新 `mod.rs` 导出
3. 更新入口文件

### 6.3 第三阶段：验证与清理

1. 运行 `cargo test` 确保所有测试通过
2. 删除原文件中的冗余代码
3. 更新文档

## 7. 测试分类与优先级

### 7.1 测试类型

| 类型     | 标记       | 用途             |
| -------- | ---------- | ---------------- |
| 单元测试 | `#[test]`  | 快速验证单个功能 |
| 集成测试 | `#[test]`  | 验证模块间交互   |
| 边界测试 | `#[test]`  | 验证边界条件     |
| 性能测试 | `#[bench]` | 性能基准         |

### 7.2 测试属性

```rust
// 忽略测试
#[test]
#[ignore = "需要外部依赖"]
fn test_external_dependency() { }

// 长时间运行测试
#[test]
#[ignore = "长时间运行"]
fn test_long_running() { }
```

## 8. 最佳实践

### 8.1 测试隔离

每个测试使用独立的Space，避免数据污染：

```rust
#[test]
fn test_example() {
    let space_name = format!("test_{}", uuid::Uuid::new_v4());
    TestScenario::new()
        .setup_space(&space_name)
        // ...
}
```

### 8.2 断言规范

使用明确的断言，避免模糊断言：

```rust
// 不推荐
assert!(result.is_ok() || result.is_err());

// 推荐
assert!(result.is_ok(), "Expected success but got: {:?}", result.err());
```

### 8.3 测试数据管理

使用 `data_fixtures.rs` 管理测试数据：

```rust
// tests/common/data_fixtures.rs
pub fn social_network_data() -> Vec<&'static str> {
    vec![
        "CREATE TAG Person(name STRING, age INT)",
        "CREATE EDGE KNOWS(since DATE)",
        // ...
    ]
}
```

## 9. 持续集成

### 9.1 CI配置

```yaml
# .github/workflows/test.yml
- name: Run tests
  run: |
    cargo test --lib -- --nocapture
    cargo test --test integration_dcl -- --nocapture
    cargo test --test integration_ddl -- --nocapture
    cargo test --test integration_dml -- --nocapture
    cargo test --test integration_dql -- --nocapture
```

### 9.2 测试覆盖率

```bash
cargo tarpaulin --out Html --output-dir coverage/
```

## 10. 参考文档

- [Rust测试组织](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [集成测试设计](./integration_test_design.md)
- [调试指南](./debugging_guide.md)
