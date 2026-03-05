# Query 模块重构 - 旧代码修改分析

## 概述

本文档分析了 Query 模块重构后需要修改的旧代码，以及修改的优先级和策略。

## 问题分析

### 1. QueryContext 命名冲突 ⚠️

**问题描述**:
- `src/query/query_context.rs` 中定义了 `query::QueryContext`
- `src/api/core/types.rs` 中定义了 `api::core::QueryContext`
- 两个结构体名称相同但结构不同

**影响范围**:
- API 层代码使用 `api::core::QueryContext`
- Query 层代码使用 `query::QueryContext`
- 可能导致命名冲突和混淆

**当前状态**:
- ✅ 编译通过，说明目前没有直接的命名冲突
- ⚠️ 但代码可读性和维护性存在问题

### 2. API 层的 QueryContext 使用

**文件列表**:
1. `src/api/core/types.rs` - 定义 API 层 QueryContext
2. `src/api/core/query_api.rs` - 使用 API 层 QueryContext
3. `src/api/embedded/statement.rs` - 使用 API 层 QueryContext
4. `src/api/embedded/transaction.rs` - 使用 API 层 QueryContext
5. `src/api/embedded/session.rs` - 使用 API 层 QueryContext

**当前实现**:
```rust
// src/api/core/types.rs
pub struct QueryContext {
    pub space_id: Option<u64>,
    pub auto_commit: bool,
    pub transaction_id: Option<u64>,
    pub parameters: Option<HashMap<String, Value>>,
}
```

**问题**:
- API 层的 QueryContext 是一个简单的数据结构
- Query 层的 QueryContext 是一个复杂的上下文管理器
- 两者职责不同，但名称相同容易混淆

### 3. Validator 和 Planner 中的临时 QueryContext 创建

**文件列表**:
1. `src/query/validator/statements/insert_edges_validator.rs` - Line 426
2. `src/query/validator/clauses/limit_validator.rs` - Line 326
3. `src/query/validator/statements/lookup_validator.rs` - Line 523
4. `src/query/validator/statements/insert_vertices_validator.rs` - Line 388
5. `src/query/validator/statements/go_validator.rs` - Line 536
6. `src/query/planner/statements/statement_planner.rs` - Line 135
7. `src/query/planner/statements/clauses/where_clause_planner.rs` - Line 145, 188
8. `src/query/planner/statements/clauses/return_clause_planner.rs` - Line 229, 265, 348, 409, 467
9. `src/query/planner/statements/clauses/order_by_planner.rs` - Line 207, 250, 298
10. `src/query/planner/statements/clauses/yield_planner.rs` - Line 421, 472
11. `src/query/planner/statements/clauses/pagination_planner.rs` - Line 172, 215, 256
12. `src/query/planner/connector.rs` - Line 134

**问题**:
- 这些地方创建了临时的 QueryContext
- 使用 `QueryContext::new(Arc::new(QueryRequestContext::new(...)))`
- 可能需要使用 Builder 模式来简化创建过程

### 4. QueryContext 字段访问

**文件列表**:
- `src/query/validator/statements/insert_edges_validator.rs` - Line 285, 351
- `src/query/validator/statements/fetch_edges_validator.rs` - Line 327, 349
- `src/query/validator/statements/update_validator.rs` - Line 708, 729
- `src/query/validator/statements/delete_validator.rs` - Line 496, 518
- `src/query/validator/statements/fetch_vertices_validator.rs` - Line 254, 276
- `src/query/validator/clauses/limit_validator.rs` - Line 237, 261
- `src/query/planner/statements/match_statement_planner.rs` - Line 89, 90
- `src/query/validator/statements/lookup_validator.rs` - Line 368, 379, 399
- `src/query/validator/statements/insert_vertices_validator.rs` - Line 238, 309
- `src/query/validator/statements/go_validator.rs` - Line 363, 416
- `src/query/validator/statements/match_validator.rs` - Line 680, 704
- `src/query/validator/statements/get_subgraph_validator.rs` - Line 192, 217
- `src/query/validator/statements/find_path_validator.rs` - Line 134, 159
- `src/query/planner/statements/lookup_planner.rs` - Line 54
- `src/query/validator/statements/create_validator.rs` - Line 756, 764

**问题**:
- 这些地方访问了 QueryContext 的字段
- 主要访问 `space_id()`, `space_name()`, `sym_table()` 等方法
- 需要确认这些方法在新架构中是否仍然可用

## 修改优先级

### 高优先级 🔴

#### 1. 解决 QueryContext 命名冲突

**建议方案**:
- 将 API 层的 QueryContext 重命名为 `ApiQueryContext` 或 `QueryRequest`
- 或者使用模块路径区分：`api::core::QueryContext` 和 `query::QueryContext`

**影响文件**:
- `src/api/core/types.rs`
- `src/api/core/query_api.rs`
- `src/api/embedded/statement.rs`
- `src/api/embedded/transaction.rs`
- `src/api/embedded/session.rs`

**工作量**: 中等

#### 2. 优化临时 QueryContext 创建

**建议方案**:
- 在 Validator 和 Planner 中使用 Builder 模式创建 QueryContext
- 或者创建辅助函数来简化创建过程

**影响文件**:
- `src/query/validator/statements/*.rs`
- `src/query/planner/statements/clauses/*.rs`

**工作量**: 中等

### 中优先级 🟡

#### 3. 统一 QueryContext 字段访问

**建议方案**:
- 确认所有字段访问方法在新架构中仍然可用
- 更新文档说明字段访问的最佳实践

**影响文件**:
- `src/query/validator/statements/*.rs`
- `src/query/planner/statements/*.rs`

**工作量**: 低

#### 4. API 层适配

**建议方案**:
- 优化 `QueryApi::execute` 方法，更好地处理 QueryContext 转换
- 考虑是否需要将 API 层的 QueryContext 映射到 Query 层的 QueryContext

**影响文件**:
- `src/api/core/query_api.rs`

**工作量**: 低

### 低优先级 🟢

#### 5. 代码清理

**建议方案**:
- 移除未使用的导入
- 清理注释
- 统一代码风格

**影响文件**:
- 多个文件

**工作量**: 低

## 详细修改方案

### 方案 1: 重命名 API 层 QueryContext

#### 步骤 1: 重命名结构体

**文件**: `src/api/core/types.rs`

```rust
// 修改前
pub struct QueryContext {
    pub space_id: Option<u64>,
    pub auto_commit: bool,
    pub transaction_id: Option<u64>,
    pub parameters: Option<HashMap<String, Value>>,
}

// 修改后
pub struct QueryRequest {
    pub space_id: Option<u64>,
    pub auto_commit: bool,
    pub transaction_id: Option<u64>,
    pub parameters: Option<HashMap<String, Value>>,
}
```

#### 步骤 2: 更新所有引用

**文件**: `src/api/core/query_api.rs`

```rust
// 修改前
pub fn execute(&mut self, query: &str, ctx: QueryContext) -> CoreResult<QueryResult>

// 修改后
pub fn execute(&mut self, query: &str, ctx: QueryRequest) -> CoreResult<QueryResult>
```

**文件**: `src/api/embedded/statement.rs`

```rust
// 修改前
use crate::api::core::{CoreError, CoreResult, QueryApi, QueryContext};
let ctx = QueryContext { ... };

// 修改后
use crate::api::core::{CoreError, CoreResult, QueryApi, QueryRequest};
let ctx = QueryRequest { ... };
```

**文件**: `src/api/embedded/transaction.rs`

```rust
// 修改前
use crate::api::core::{CoreError, CoreResult, QueryContext, TransactionHandle};
let ctx = QueryContext { ... };

// 修改后
use crate::api::core::{CoreError, CoreResult, QueryRequest, TransactionHandle};
let ctx = QueryRequest { ... };
```

**文件**: `src/api/embedded/session.rs`

```rust
// 修改前
use crate::api::core::{CoreError, CoreResult, QueryApi, QueryContext, SchemaApi, TransactionApi};
let ctx = QueryContext { ... };

// 修改后
use crate::api::core::{CoreError, CoreResult, QueryApi, QueryRequest, SchemaApi, TransactionApi};
let ctx = QueryRequest { ... };
```

### 方案 2: 优化临时 QueryContext 创建

#### 步骤 1: 创建辅助函数

**文件**: `src/query/query_context.rs`

```rust
impl QueryContext {
    /// 创建用于验证的临时上下文
    pub fn new_for_validation(query_text: String) -> Self {
        let rctx = Arc::new(QueryRequestContext::new(query_text));
        Self::new(rctx)
    }

    /// 创建用于规划的临时上下文
    pub fn new_for_planning(query_text: String) -> Self {
        let rctx = Arc::new(QueryRequestContext::new(query_text));
        Self::new(rctx)
    }
}
```

#### 步骤 2: 更新使用位置

**文件**: `src/query/validator/statements/insert_edges_validator.rs`

```rust
// 修改前
let mut qctx = QueryContext::new(rctx);

// 修改后
let mut qctx = QueryContext::new_for_validation(query_text.clone());
```

**文件**: `src/query/planner/statements/clauses/return_clause_planner.rs`

```rust
// 修改前
let qctx = Arc::new(crate::query::QueryContext::new(
    Arc::new(QueryRequestContext::new(query_text.clone())),
));

// 修改后
let qctx = Arc::new(crate::query::QueryContext::new_for_planning(
    query_text.clone(),
));
```

### 方案 3: 统一 QueryContext 字段访问

#### 步骤 1: 确认字段访问方法

**文件**: `src/query/query_context.rs`

确认以下方法仍然可用：
- `space_id()` ✅
- `space_name()` ✅
- `sym_table()` ✅
- `gen_id()` ✅
- `obj_pool()` ✅
- `is_killed()` ✅

#### 步骤 2: 更新文档

在 `docs/architecture/query_module_architecture.md` 中更新字段访问的文档说明。

### 方案 4: API 层适配

#### 步骤 1: 优化 QueryApi::execute

**文件**: `src/api/core/query_api.rs`

```rust
// 当前实现
pub fn execute(&mut self, query: &str, ctx: QueryContext) -> CoreResult<QueryResult> {
    let start_time = Instant::now();

    // 构建空间信息
    let space_info = ctx.space_id.map(|id| crate::core::types::SpaceInfo {
        space_id: id,
        space_name: String::new(),
        vid_type: crate::core::DataType::String,
        tags: Vec::new(),
        edge_types: Vec::new(),
        version: crate::core::types::MetadataVersion::default(),
        comment: None,
    });

    // 执行查询
    let execution_result = self
        .pipeline_manager
        .execute_query_with_space(query, space_info)
        .map_err(|e| CoreError::QueryExecutionFailed(e.to_string()))?;

    // 转换为结构化结果
    let mut result = Self::convert_to_query_result(execution_result)?;
    result.metadata.execution_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(result)
}

// 优化后的实现（如果需要）
pub fn execute(&mut self, query: &str, ctx: QueryRequest) -> CoreResult<QueryResult> {
    let start_time = Instant::now();

    // 构建 QueryRequestContext
    let rctx = Arc::new(QueryRequestContext::new(query.to_string()));

    // 构建空间信息
    let space_info = ctx.space_id.map(|id| crate::core::types::SpaceInfo {
        space_id: id,
        space_name: String::new(),
        vid_type: crate::core::DataType::String,
        tags: Vec::new(),
        edge_types: Vec::new(),
        version: crate::core::types::MetadataVersion::default(),
        comment: None,
    });

    // 使用新的 execute_query_with_request 方法
    let execution_result = self
        .pipeline_manager
        .execute_query_with_request(query, rctx, space_info)
        .map_err(|e| CoreError::QueryExecutionFailed(e.to_string()))?;

    // 转换为结构化结果
    let mut result = Self::convert_to_query_result(execution_result)?;
    result.metadata.execution_time_ms = start_time.elapsed().as_millis() as u64;

    Ok(result)
}
```

## 实施计划

### 阶段 1: 解决命名冲突（高优先级）

1. 重命名 API 层 QueryContext 为 QueryRequest
2. 更新所有引用
3. 运行测试验证

### 阶段 2: 优化临时 QueryContext 创建（中优先级）

1. 创建辅助函数
2. 更新使用位置
3. 运行测试验证

### 阶段 3: 统一字段访问（中优先级）

1. 确认字段访问方法
2. 更新文档
3. 代码审查

### 阶段 4: API 层适配（低优先级）

1. 优化 QueryApi::execute
2. 测试验证
3. 性能测试

### 阶段 5: 代码清理（低优先级）

1. 移除未使用的导入
2. 清理注释
3. 统一代码风格

## 风险评估

### 高风险

- **命名冲突修改**: 可能影响 API 的向后兼容性
- **临时 QueryContext 创建**: 可能影响现有功能

### 中风险

- **字段访问统一**: 可能需要调整现有代码
- **API 层适配**: 可能影响性能

### 低风险

- **代码清理**: 不影响功能

## 测试策略

### 单元测试

1. 测试 API 层 QueryRequest 的创建和使用
2. 测试 QueryContext 辅助函数
3. 测试字段访问方法

### 集成测试

1. 测试完整的查询流程
2. 测试 API 层到 Query 层的数据传递
3. 测试各种查询场景

### 性能测试

1. 对比修改前后的性能
2. 测试内存占用
3. 测试并发性能

## 总结

本次重构虽然已经完成了核心的 Query 模块优化，但仍有一些旧代码需要修改以提高代码质量和可维护性。建议按照优先级逐步实施这些修改，并在每个阶段进行充分的测试验证。

**关键点**:
1. 解决 QueryContext 命名冲突是最高优先级
2. 优化临时 QueryContext 创建可以提高代码可读性
3. 统一字段访问可以降低维护成本
4. API 层适配可以提高性能
5. 代码清理可以提高代码质量

**建议**:
- 优先实施高优先级的修改
- 每个阶段完成后进行充分测试
- 保持代码风格的一致性
- 及时更新文档

---

**文档版本**: 1.0
**最后更新**: 2026-03-05
**作者**: AI Assistant
**审核状态**: 待审核
