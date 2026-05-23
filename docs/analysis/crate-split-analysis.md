# 包拆分分析报告

## 一、现状

当前 `src/` 为一个单一大包 (`graphdb`)，约 **400+ 源文件**、**11 个顶层模块**。

```
src/ 模块概览:
  api/        ~92 文件  (HTTP/gRPC/C-API/Embedded API)
  common/     ~2 文件   (ID 类型)
  config/     ~16 文件  (配置管理)
  core/       ~92 文件  (核心类型、值、错误、统计)
  query/      ~200+ 文件 (解析器、优化器、执行器、规划器)
  search/     ~10 文件  (全文搜索)
  storage/    ~110 文件 (存储引擎)
  sync/       ~25 文件  (同步引擎)
  transaction/~25 文件  (事务管理)
  utils/      ~4 文件   (工具函数)
  c_api.rs    -         (C API 绑定)
```

## 二、单大包的问题

| 问题 | 影响 |
|------|------|
| **增量编译粒度粗** | 修改任意文件 → cargo 必须重新评估整个包 |
| **无工作空间级并行** | 工作空间成员可并行编译，单包只能靠 `codegen-units=64` |
| **特征门控全包膨胀** | 切换 feature 就必须重编全部 |
| **链接体积大** | cdylib + rlib 要链接全部 400+ 文件 |
| **依赖链扁平** | cargo 无法按模块粒度缓存 |

## 三、循环依赖分析

### 依赖矩阵

```
       api  cfg  cor  qry  sch  sto  syn  tra  utl  com
api     ·    ✔   ✔   ✔   ✔   ✔   ✔   ✔    ·    ·
cfg     ·    ·    ·    ·   ✔    ·    ·    ·    ·    ·
cor     ✔    ·    ·   ✔   ✔    ·   ✔   ✔    ·    ·
qry     ✔    ·    ✔    ·   ✔   ✔   ✔    ·   ✔    ·
sch     ·    ·    ✔    ·    ·   ✔   ✔?   ·    ·    ·
sto     ✔    ·    ✔    ·    ·    ·   ✔   ✔   ✔    ·
syn     ·    ·    ✔    ·   ✔    ·    ·    ·    ·    ·
tra     ·    ·    ✔    ·    ·    ·   ✔    ·    ·    ·
utl     ·    ✔    ·    ·    ·    ·    ·    ·    ·    ·
com     ·    ·    ·    ·    ·    ·    ·    ·    ·    ·
```

### 已检测到的循环依赖

#### 1. core ◀──▶ api (直接循环)
- **core→api**: `core/error/mod.rs` re-export `api::server::auth`, `api::server::permission`, `api::server::session` 的错误类型
- **core→api**: `core/error/query.rs` 有 `From<SessionError>`, `From<PermissionError>`
- **api→core**: api 大量使用 core 的类型

#### 2. core ◀──▶ query (直接循环)
- **core→query**: `core/error/mod.rs` re-export `query::executor::expression::ExpressionError`, `query::optimizer` 错误
- **core→query**: `core/error/query.rs` 有 `From<ExpressionError>`
- **core→query**: `core/value/value_def.rs` 引入 `crate::query::DataSet`
- **query→core**: query 大量使用 core 的类型

#### 3. core ◀──▶ search (直接循环)
- **core→search**: `core/error/mod.rs` re-export `search::error::SearchError`
- **core→search**: `core/stats/manager.rs` 引入 `search::error::SearchError`
- **search→core**: search 使用 core 的类型

#### 4. core ◀──▶ sync (直接循环)
- **core→sync**: `core/error/mod.rs` re-export `sync::external_index` 的错误类型
- **sync→core**: sync 使用 core 的类型

#### 5. core ◀──▶ transaction (直接循环)
- **core→transaction**: `core/error/mod.rs` 有 `From<TransactionError>`
- **core→transaction**: `core/error/storage.rs` 有 `From<InsertTransactionError>`
- **transaction→core**: transaction 使用 core 的类型

#### 6. storage ◀──▶ api (直接循环)
- **storage→api**: `storage/engine/graph_storage/context.rs` 引入 `api::server::auth::UserStorage`
- **api→storage**: api 使用 storage 的类型

#### 7. query ◀──▶ api (直接循环)
- **query→api**: `query/executor/expression/functions/mod.rs` 引入 `api::embedded::c_api::value::core_value_to_graphdb`
- **api→query**: api 使用 query 的类型

### 根因分析

**最主要的根因是 `src/core/error/mod.rs`**，它 re-export 了大量上层模块的错误类型并实现了 `From` 转换，导致 `core` 依赖了系统中所有其他模块。

次要根因包括：
- `core/value/value_def.rs` 引入 `DataSet`（query 模块）
- `storage` 使用 `UserStorage`（api 模块）
- `query` 使用 `core_value_to_graphdb`（api 模块）
- `config` 使用 `FulltextConfig`（search 模块）

## 四、重构方案

### 目标依赖图（重构后）

```
common  core  utils  
  │      │      │     
  └──┬───┘      │     
     ▼          │     
  storage       │     
     │          │     
     ▼          ▼     
  transaction  config  
     │          │     
     └─────┬────┘     
           ▼           
         query          
           │            
           ▼            
      sync              
           │            
           ▼            
          api           
```

### 具体修复步骤

| # | 修复内容 | 涉及文件 | 说明 |
|---|---------|---------|------|
| 1 | 移除 `core/error/mod.rs` 中所有对外部模块的 re-export | `core/error/mod.rs` | 删除 33-46 行 |
| 2 | 移除 `core/error/mod.rs` 中所有对外部错误类型的 `From` 实现 | `core/error/mod.rs` | 删除 342-470 行（保留 `std` 和 `serde_json`） |
| 3 | 简化 `core/error/mod.rs` 中的 `ToPublicError` 实现 | `core/error/mod.rs` | 不再 downcast 到上层错误 |
| 4 | 移除 `core/error/query.rs` 中对 api/query 的依赖 | `core/error/query.rs` | 删除 `From<SessionError>`, `From<PermissionError>`, `From<ExpressionError>` |
| 5 | 移除 `core/error/storage.rs` 中对 transaction 的依赖 | `core/error/storage.rs` | 删除 `From<InsertTransactionError>` |
| 6 | 修复 `Value::DataSet` 依赖 | `core/value/value_def.rs`, `query/data_set.rs` | 将 `DataSet` 定义移到 `core` |
| 7 | 移除 `core/stats/manager.rs` 中对 search 的依赖 | `core/stats/manager.rs` | 删除 `classify_search_error` |
| 8 | 解除 `storage → api` 依赖 | `UserStorage` | 将 `UserStorage` 移到 `common` 或 `core` |
| 9 | 解除 `query → api` 依赖 | `core_value_to_graphdb` | 将类型转换函数移到适当位置 |
| 10 | 解除 `config → search` 依赖 | `FulltextConfig` | 将 `FulltextConfig` 定义移到 `config` |

## 五、拆分方案

完成循环依赖解除后，可按以下结构拆分：

```
graphdb-core/        ← core/, common/, utils/
graphdb-storage/     ← storage/
graphdb-transaction/ ← transaction/
graphdb-config/      ← config/
graphdb-query/       ← query/
graphdb-sync/        ← sync/
graphdb-search/      ← search/
graphdb-api/         ← api/
graphdb-server/      ← main.rs, lib.rs (胶水层)
```

### 依赖关系

```
graphdb-core
  ├─ graphdb-storage
  │   ├─ graphdb-transaction
  │   └─ graphdb-query
  ├─ graphdb-config
  │   └─ graphdb-query
  └─ graphdb-search
      └─ graphdb-sync
          └─ graphdb-api
```

### 预期收益

| 场景 | 单包 | 多包 |
|------|------|------|
| 全量编译 | 一次编译全部 | 可并行，总耗时相近 |
| 改 query 一个文件 | 重编 + 链接全包 | 仅 `graphdb-query` + 上游 |
| 改 storage 一个文件 | 全包 | 仅 `graphdb-storage` |
| 改 config | 全包 | 仅 `graphdb-config` |
| CI 缓存 | 无法利用 | 下层包缓存命中 |
