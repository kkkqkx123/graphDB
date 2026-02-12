# GraphDB 集成测试设计方案

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

## 2. 集成测试分阶段方案

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

**测试文件**: `tests/integration/storage/*.rs`

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

**测试文件**: `tests/integration/core/*.rs`

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

**测试文件**: `tests/integration/query/*.rs`

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

**测试文件**: `tests/integration/api/*.rs`

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

**测试文件**: `tests/integration/e2e/*.rs`

**依赖关系**: 依赖所有前置阶段

---

## 3. 测试基础设施设计

### 3.1 目录结构

```
tests/
├── integration/
│   ├── storage/          # 阶段一：存储层测试
│   │   ├── mod.rs
│   │   ├── redb_tests.rs
│   │   ├── transaction_tests.rs
│   │   └── metadata_tests.rs
│   ├── core/             # 阶段二：核心层测试
│   │   ├── mod.rs
│   │   ├── value_tests.rs
│   │   └── expression_tests.rs
│   ├── query/            # 阶段三：查询引擎测试
│   │   ├── mod.rs
│   │   ├── parser_tests.rs
│   │   ├── planner_tests.rs
│   │   └── executor_tests.rs
│   ├── api/              # 阶段四：API层测试
│   │   ├── mod.rs
│   │   ├── service_tests.rs
│   │   └── session_tests.rs
│   └── e2e/              # 阶段五：端到端测试
│       ├── mod.rs
│       ├── space_tests.rs
│       ├── schema_tests.rs
│       ├── data_tests.rs
│       └── query_tests.rs
├── fixtures/             # 测试数据
│   ├── schemas/
│   ├── datasets/
│   └── queries/
└── helpers/              # 测试辅助函数
    ├── mod.rs
    ├── storage_helpers.rs
    ├── query_helpers.rs
    └── assertions.rs
```

### 3.2 共享测试工具

```rust
// tests/helpers/mod.rs
pub mod storage_helpers;
pub mod query_helpers;
pub mod assertions;

use std::sync::Arc;
use tempfile::TempDir;

/// 测试存储实例包装器
pub struct TestStorage {
    storage: Arc<DefaultStorage>,
    temp_dir: TempDir,
}

impl TestStorage {
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let storage = Arc::new(DefaultStorage::new_with_path(temp_dir.path())?);
        Ok(Self { storage, temp_dir })
    }
    
    pub fn storage(&self) -> Arc<DefaultStorage> {
        self.storage.clone()
    }
}

/// 测试数据集加载
pub async fn load_test_dataset(storage: &DefaultStorage, dataset: &str) -> anyhow::Result<()> {
    // 从 fixtures/datasets/ 加载数据
}
```

### 3.3 测试配置

```toml
# Cargo.toml [dev-dependencies] 补充
tokio-test = "0.4.4"
tempfile = "3.23.0"
serde_json = "1.0.145"
```

---

## 4. 执行策略

### 4.1 分阶段执行命令

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

### 4.2 CI/CD 集成建议

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

### 4.3 测试执行顺序约束

| 阶段 | 前置依赖 | 失败处理 |
|------|----------|----------|
| 阶段一 | 无 | 阻塞后续所有阶段 |
| 阶段二 | 阶段一通过 | 阻塞后续阶段 |
| 阶段三 | 阶段一、二通过 | 阻塞后续阶段 |
| 阶段四 | 阶段一至三通过 | 阻塞阶段五 |
| 阶段五 | 所有前置阶段通过 | 仅报告失败 |

---

## 5. 测试用例设计原则

### 5.1 测试命名规范

```rust
// 格式: test_<模块>_<场景>_<预期结果>
#[tokio::test]
async fn test_storage_transaction_commit_success() { }

#[tokio::test]
async fn test_storage_transaction_rollback_data_integrity() { }

#[tokio::test]
async fn test_query_match_single_vertex_filter() { }
```

### 5.2 测试数据管理

- ** fixtures 模式**: 静态测试数据存放于 `tests/fixtures/`
- **工厂模式**: 动态生成测试数据
- **每个测试独立**: 使用临时目录，测试后清理

### 5.3 断言策略

```rust
// 结果断言
assert!(result.is_ok());
assert_eq!(result.unwrap(), expected);

// 状态断言
assert_storage_contains(&storage, key, value).await;
assert_query_returns_rows(&query_result, expected_count).await;

// 错误断言
assert!(matches!(err, QueryError::ValidationError(_)));
```

---

## 6. 风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 存储测试慢 | 执行时间长 | 使用内存模式、并行执行 |
| 测试数据依赖 | 测试不稳定 | 每个测试独立初始化数据 |
| 资源泄漏 | 磁盘空间耗尽 | 使用 TempDir 自动清理 |
| 并发冲突 | 测试随机失败 | 避免共享状态、使用隔离存储 |

---

## 7. 实施计划

### 7.1 优先级排序

1. **立即实施**: 阶段一（存储层）- 基础设施
2. **第一迭代**: 阶段二（核心层）+ 阶段三（查询引擎基础）
3. **第二迭代**: 阶段三完整 + 阶段四（API层）
4. **第三迭代**: 阶段五（端到端场景）

### 7.2 工作量估算

| 阶段 | 预估用例数 | 预估工作量 |
|------|------------|------------|
| 阶段一 | 20-30 | 3-4 天 |
| 阶段二 | 15-20 | 2-3 天 |
| 阶段三 | 30-40 | 5-7 天 |
| 阶段四 | 15-20 | 3-4 天 |
| 阶段五 | 25-35 | 5-7 天 |
| **总计** | **105-145** | **18-25 天** |

---

## 8. 附录

### 8.1 参考文档

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Integration Tests](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests)

### 8.2 相关代码文件

- [src/lib.rs](file:///d:/项目/database/graphDB/src/lib.rs) - 库入口
- [src/query/query_pipeline_manager.rs](file:///d:/项目/database/graphDB/src/query/query_pipeline_manager.rs) - 查询管道
- [src/api/service/graph_service.rs](file:///d:/项目/database/graphDB/src/api/service/graph_service.rs) - 图服务
- [src/storage/redb_storage.rs](file:///d:/项目/database/graphDB/src/storage/redb_storage.rs) - 存储实现
