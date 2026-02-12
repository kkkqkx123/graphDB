# GraphDB 集成测试设计方案

> 本文档基于 Rust 集成测试最佳实践（参考 Rust Book、Cargo 官方文档及社区实践）

## 1. 项目现状分析

### 1.1 已完成测试
- **单元测试（lib tests）**: 已完成，覆盖核心模块的基础功能
- 包含测试文件：
  - `src/query/executor/admin/*/tests.rs` - 管理操作执行器测试
  - `src/query/executor/data_processing/graph_traversal/tests.rs` - 图遍历测试
  - 各模块内嵌的 `#[cfg(test)]` 单元测试（100+ 文件）

### 1.2 项目架构概览

```
graphDB
├── src/
│   ├── api/          # API接口层（服务、会话管理）
│   ├── common/       # 通用工具（ID生成、内存管理、线程）
│   ├── config/       # 配置管理
│   ├── core/         # 核心类型系统、错误处理、表达式
│   ├── expression/   # 表达式求值、函数注册
│   ├── index/        # 索引系统
│   ├── query/        # 查询引擎（解析器、规划器、优化器、执行器）
│   ├── services/     # 业务服务层
│   ├── storage/      # 存储引擎（Redb/RocksDB）
│   └── utils/        # 工具函数
```

### 1.3 关键集成点

1. **查询全流程**: Parser → Validator → Planner → Optimizer → Executor
2. **存储接口**: StorageClient 与 RedbStorage 的集成
3. **会话管理**: ClientSession ↔ GraphService ↔ QueryPipelineManager
4. **事务协调**: TransactionManager ↔ Storage 事务接口
5. **索引集成**: IndexManager ↔ Storage 元数据管理

---

## 2. Rust 集成测试最佳实践

### 2.1 目录结构标准（Cargo 约定）

根据 [Cargo 官方文档](https://doc.rust-lang.org/cargo/reference/cargo-targets.html) 和 [Rust Book](https://doc.rust-lang.org/book/ch11-03-test-organization.html)：

```
project/
├── Cargo.toml
├── src/
│   └── lib.rs          # 必须是 lib.rs 才能支持集成测试
└── tests/              # 集成测试目录（与 src/ 同级）
    ├── common/         # 共享测试工具模块
    │   └── mod.rs      # 使用 tests/common/mod.rs 模式
    ├── integration_storage.rs    # 每个文件是一个独立的测试 crate
    ├── integration_core.rs
    ├── integration_query.rs
    └── integration_e2e.rs
```

**关键要点**：
- `tests/` 目录下的每个 `.rs` 文件会被编译为独立的测试 crate
- 共享代码放在 `tests/common/mod.rs`，通过 `mod common;` 引用
- **不要**创建 `tests/common.rs`，否则会被当作测试文件执行
- 测试的工作目录设置为包根目录，可使用相对路径访问资源文件

### 2.2 共享模块使用模式

```rust
// tests/common/mod.rs
use std::sync::Arc;
use tempfile::TempDir;
use graphdb::storage::DefaultStorage;

pub struct TestStorage {
    storage: Arc<DefaultStorage>,
    _temp_dir: TempDir,  // 生命周期绑定，自动清理
}

impl TestStorage {
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let storage = Arc::new(DefaultStorage::new_with_path(temp_dir.path())?);
        Ok(Self { 
            storage, 
            _temp_dir: temp_dir 
        })
    }
    
    pub fn storage(&self) -> Arc<DefaultStorage> {
        self.storage.clone()
    }
}
```

```rust
// tests/integration_storage.rs
mod common;
use common::TestStorage;

#[tokio::test]
async fn test_storage_basic_operations() {
    let test_storage = TestStorage::new().expect("创建测试存储失败");
    let storage = test_storage.storage();
    // 测试逻辑...
}
```

### 2.3 测试命名规范

```rust
// 格式: test_<被测组件>_<场景>_<预期结果>
#[tokio::test]
async fn test_redb_storage_create_space_success() {}

#[tokio::test]
async fn test_redb_storage_transaction_rollback_isolation() {}

#[tokio::test]
async fn test_redb_iterator_filter_predicate_match() {}
```

### 2.4 临时资源管理

使用 `tempfile` crate 确保测试隔离：

```rust
use tempfile::TempDir;

#[tokio::test]
async fn test_with_isolated_storage() {
    let temp_dir = tempfile::tempdir().expect("创建临时目录失败");
    let storage = DefaultStorage::new_with_path(temp_dir.path())
        .expect("初始化存储失败");
    
    // 测试执行...
    
    // temp_dir 在作用域结束时自动清理
}
```

---

## 3. 集成测试分阶段方案

### 阶段一：存储层集成测试（基础层）

**目标**: 验证存储引擎的核心功能与接口正确性

**测试范围**:
| 模块 | 测试内容 | 优先级 |
|------|----------|--------|
| `storage::redb_storage` | 数据库初始化、表创建、基础CRUD | P0 |
| `storage::operations` | 读写操作、批量操作、错误处理 | P0 |
| `storage::transaction` | 事务开始/提交/回滚、隔离性 | P0 |
| `storage::metadata` | Schema管理、元数据持久化 | P1 |
| `storage::iterator` | 迭代器组合、谓词过滤、属性访问 | P1 |
| `storage::index` | 索引创建、查询、维护 | P1 |

**测试文件**: `tests/integration_storage.rs`

**依赖关系**: 无（最底层）

---

### 阶段二：核心类型与表达式集成测试

**目标**: 验证核心类型系统和表达式求值的正确性

**测试范围**:
| 模块 | 测试内容 | 优先级 |
|------|----------|--------|
| `core::value` | 值类型转换、比较、运算 | P0 |
| `core::types` | 类型系统兼容性检查 | P0 |
| `expression::evaluator` | 表达式求值、上下文访问 | P0 |
| `expression::functions` | 内置函数注册与调用 | P1 |
| `expression::context` | 上下文链、缓存管理 | P1 |

**测试文件**: `tests/integration_core.rs`

**依赖关系**: 依赖阶段一的存储测试基础设施

---

### 阶段三：查询引擎组件集成测试

**目标**: 验证查询处理各阶段的协同工作

**测试范围**:
| 模块 | 测试内容 | 优先级 |
|------|----------|--------|
| `query::parser` | SQL/NGQL解析、AST生成 | P0 |
| `query::validator` | 语义验证、类型推导 | P0 |
| `query::planner` | 执行计划生成 | P0 |
| `query::optimizer` | 计划优化、规则应用 | P1 |
| `query::executor` | 执行器调度、结果返回 | P0 |

**测试文件**: `tests/integration_query.rs`

**测试场景**:
```rust
// 示例：完整查询流程测试
#[tokio::test]
async fn test_complete_query_flow() {
    // 1. 初始化存储
    // 2. 创建测试数据
    // 3. 解析查询
    // 4. 验证语义
    // 5. 生成计划
    // 6. 优化计划
    // 7. 执行查询
    // 8. 验证结果
}
```

**依赖关系**: 依赖阶段一、二

---

### 阶段四：API服务层集成测试

**目标**: 验证服务层与查询引擎、存储层的集成

**测试范围**:
| 模块 | 测试内容 | 优先级 |
|------|----------|--------|
| `api::service` | GraphService 完整流程 | P0 |
| `api::session` | 会话生命周期管理 | P0 |
| `api::service::auth` | 认证流程 | P1 |
| `api::service::permission` | 权限检查 | P1 |

**测试文件**: `tests/integration_api.rs`

**测试场景**:
- 会话创建与销毁
- 查询执行完整流程
- 事务边界管理
- 错误传播与处理

**依赖关系**: 依赖阶段一、二、三

---

### 阶段五：端到端场景集成测试

**目标**: 验证典型业务场景的完整功能

**测试范围**:
| 场景 | 描述 | 优先级 |
|------|------|--------|
| 图空间管理 | CREATE SPACE / USE / DROP | P0 |
| Schema管理 | CREATE TAG / EDGE / INDEX | P0 |
| 数据操作 | INSERT / UPDATE / DELETE | P0 |
| 查询操作 | MATCH / GO / LOOKUP / FETCH | P0 |
| 图遍历 | 多跳遍历、路径查询 | P1 |
| 聚合查询 | COUNT / SUM / GROUP BY | P1 |
| 事务场景 | 多语句事务、回滚 | P1 |

**测试文件**: `tests/integration_e2e.rs`

**依赖关系**: 依赖所有前置阶段

---

## 4. 测试基础设施设计

### 4.1 目录结构（遵循 Cargo 标准）

```
tests/
├── common/                   # 共享测试工具（必须是目录形式）
│   ├── mod.rs               # 主模块导出
│   ├── storage_helpers.rs   # 存储相关辅助函数
│   ├── data_fixtures.rs     # 测试数据生成
│   └── assertions.rs        # 自定义断言
├── fixtures/                # 静态测试数据文件
│   ├── schemas/
│   │   ├── person_tag.json
│   │   └── knows_edge.json
│   └── datasets/
│       └── social_network.csv
├── integration_storage.rs   # 阶段一：存储层集成测试
├── integration_core.rs      # 阶段二：核心层集成测试
├── integration_query.rs     # 阶段三：查询引擎测试
├── integration_api.rs       # 阶段四：API层测试
└── integration_e2e.rs       # 阶段五：端到端测试
```

### 4.2 共享测试工具

```rust
// tests/common/mod.rs
pub mod storage_helpers;
pub mod data_fixtures;
pub mod assertions;

use std::sync::Arc;
use tempfile::TempDir;
use graphdb::storage::DefaultStorage;

/// 测试存储实例包装器
/// 
/// 使用 tempfile 确保每个测试有独立的存储环境，
/// 测试结束后自动清理临时目录
pub struct TestStorage {
    storage: Arc<DefaultStorage>,
    _temp_dir: TempDir,
}

impl TestStorage {
    /// 创建新的测试存储实例
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let storage = Arc::new(DefaultStorage::new_with_path(temp_dir.path())?);
        Ok(Self { 
            storage, 
            _temp_dir: temp_dir 
        })
    }
    
    /// 获取存储实例引用
    pub fn storage(&self) -> Arc<DefaultStorage> {
        self.storage.clone()
    }
    
    /// 获取存储实例（用于需要直接访问的场景）
    pub fn storage_ref(&self) -> &DefaultStorage {
        &self.storage
    }
}

/// 测试上下文，包含常用测试资源
pub struct TestContext {
    pub storage: TestStorage,
}

impl TestContext {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            storage: TestStorage::new()?,
        })
    }
}
```

### 4.3 测试配置

```toml
# Cargo.toml [dev-dependencies] 补充
tokio-test = "0.4.4"
tempfile = "3.23.0"
serde_json = "1.0.145"
```

---

## 5. 执行策略

### 5.1 分阶段执行命令

```bash
# 阶段一：存储层测试
cargo test --test integration_storage

# 阶段二：核心层测试
cargo test --test integration_core

# 阶段三：查询引擎测试
cargo test --test integration_query

# 阶段四：API层测试
cargo test --test integration_api

# 阶段五：端到端测试
cargo test --test integration_e2e

# 全部集成测试
cargo test --test 'integration_*'
```

### 5.2 CI/CD 集成建议

```yaml
# 示例 GitHub Actions 配置
jobs:
  integration-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run Phase 1 - Storage Tests
        run: cargo test --test integration_storage
        
      - name: Run Phase 2 - Core Tests
        run: cargo test --test integration_core
        
      - name: Run Phase 3 - Query Engine Tests
        run: cargo test --test integration_query
        
      - name: Run Phase 4 - API Tests
        run: cargo test --test integration_api
        
      - name: Run Phase 5 - E2E Tests
        run: cargo test --test integration_e2e
```

### 5.3 测试执行顺序约束

| 阶段 | 前置依赖 | 失败处理 |
|------|----------|----------|
| 阶段一 | 无 | 阻塞后续所有阶段 |
| 阶段二 | 阶段一通过 | 阻塞后续阶段 |
| 阶段三 | 阶段一、二通过 | 阻塞后续阶段 |
| 阶段四 | 阶段一至三通过 | 阻塞阶段五 |
| 阶段五 | 所有前置阶段通过 | 仅报告失败 |

---

## 6. 测试用例设计原则

### 6.1 测试命名规范

```rust
// 格式: test_<模块>_<场景>_<预期结果>
#[tokio::test]
async fn test_storage_transaction_commit_success() { }

#[tokio::test]
async fn test_storage_transaction_rollback_data_integrity() { }

#[tokio::test]
async fn test_query_match_single_vertex_filter() { }
```

### 6.2 测试数据管理

- **fixtures 模式**: 静态测试数据存放于 `tests/fixtures/`
- **工厂模式**: 动态生成测试数据
- **每个测试独立**: 使用临时目录，测试后清理

```rust
// tests/common/data_fixtures.rs
use graphdb::storage::{Schema, ColumnDef, DataType};

/// 创建人员标签 Schema
pub fn person_tag_schema() -> Schema {
    Schema::new("Person")
        .with_column(ColumnDef::new("name", DataType::String))
        .with_column(ColumnDef::new("age", DataType::Int32))
}

/// 创建认识关系 Schema
pub fn knows_edge_schema() -> Schema {
    Schema::new("KNOWS")
        .with_column(ColumnDef::new("since", DataType::Date))
}
```

### 6.3 断言策略

```rust
// tests/common/assertions.rs
use graphdb::core::DBResult;

/// 断言操作成功
pub fn assert_ok<T>(result: DBResult<T>) -> T {
    result.expect("操作应该成功")
}

/// 断言操作失败并匹配错误类型
pub fn assert_err_with<T>(result: DBResult<T>, expected_msg: &str) {
    let err = result.expect_err("操作应该失败");
    let err_str = err.to_string();
    assert!(
        err_str.contains(expected_msg),
        "错误消息应包含 '{}', 实际是 '{}'",
        expected_msg,
        err_str
    );
}

/// 断言存储包含指定数据
pub async fn assert_storage_has_vertex(
    storage: &DefaultStorage,
    space: &str,
    vertex_id: i64
) {
    // 实现验证逻辑
}
```

---

## 7. 风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 存储测试慢 | 执行时间长 | 使用内存模式、并行执行 |
| 测试数据依赖 | 测试不稳定 | 每个测试独立初始化数据 |
| 资源泄漏 | 磁盘空间耗尽 | 使用 TempDir 自动清理 |
| 并发冲突 | 测试随机失败 | 避免共享状态、使用隔离存储 |

---

## 8. 实施计划

### 8.1 优先级排序

1. **立即实施**: 阶段一（存储层）- 基础设施
2. **第一迭代**: 阶段二（核心层）+ 阶段三基础部分
3. **第二迭代**: 阶段三完整 + 阶段四（API层）
4. **第三迭代**: 阶段五（端到端场景）

### 8.2 工作量估算

| 阶段 | 预估用例数 | 预估工作量 |
|------|------------|------------|
| 阶段一 | 20-30 | 3-4 天 |
| 阶段二 | 15-20 | 2-3 天 |
| 阶段三 | 30-40 | 5-7 天 |
| 阶段四 | 15-20 | 3-4 天 |
| 阶段五 | 25-35 | 5-7 天 |
| **总计** | **105-145** | **18-25 天** |

---

## 9. 附录

### 9.1 参考文档

- [Rust Book - Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Cargo Targets - Integration Tests](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests)
- [Cargo Project Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html)
- [Rust Users Forum - Integration Test Organization](https://users.rust-lang.org/t/integration-test-common-module-questions/96638)

### 9.2 相关代码文件

- [src/lib.rs](file:///d:/项目/database/graphDB/src/lib.rs) - 库入口
- [src/query/query_pipeline_manager.rs](file:///d:/项目/database/graphDB/src/query/query_pipeline_manager.rs) - 查询管道
- [src/api/service/graph_service.rs](file:///d:/项目/database/graphDB/src/api/service/graph_service.rs) - 图服务
- [src/storage/redb_storage.rs](file:///d:/项目/database/graphDB/src/storage/redb_storage.rs) - 存储实现

### 9.3 最佳实践要点总结

1. **必须是 lib.rs**: 项目必须有 `src/lib.rs` 才能创建集成测试
2. **common 模块**: 共享代码放在 `tests/common/mod.rs`，不是 `tests/common.rs`
3. **独立 crate**: `tests/` 下每个 `.rs` 文件编译为独立测试 crate
4. **工作目录**: 测试工作目录是包根目录，可用相对路径访问资源
5. **资源清理**: 使用 `tempfile` crate 确保测试资源自动清理
6. **命名规范**: 测试函数使用 `test_<组件>_<场景>_<结果>` 格式
