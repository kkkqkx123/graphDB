# 全文检索错误类型集成分析

## 1. 当前状态

### 1.1 错误类型层次结构

```
src/core/error/mod.rs          - DBError (统一入口)
src/core/error/fulltext.rs     - FulltextError, CoordinatorError (新增)
src/search/error.rs            - SearchError (搜索引擎层)
src/sync/manager.rs            - SyncError (同步模块)
```

### 1.2 各模块当前使用的错误类型

| 模块 | 文件位置 | 当前错误类型 | 问题 |
|------|----------|--------------|------|
| 搜索引擎 | `src/search/` | `SearchError` | ✅ 正确，底层模块 |
| 协调器 | `src/coordinator/fulltext.rs` | `SearchError` | ⚠️ 应使用 `CoordinatorError` |
| 同步模块 | `src/sync/manager.rs` | `SyncError` | ⚠️ `CoordinatorError(String)` 丢失上下文 |
| DDL执行器 | `src/query/executor/admin/` | `DBError` | ✅ 正确 |

### 1.3 已实现的转换关系

```
SearchError ──From──> FulltextError ──From──> DBError
SyncError ────From──> CoordinatorError ──From──> DBError
FulltextError ─From──> CoordinatorError
```

## 2. 问题分析

### 2.1 协调器层错误类型不匹配

**当前代码** (`src/coordinator/fulltext.rs`):
```rust
use crate::search::error::SearchError;

impl FulltextCoordinator {
    pub async fn create_index(...) -> Result<String, SearchError> {
        // ...
    }
}
```

**问题**:
- 协调器职责不限于搜索引擎操作，还包括索引生命周期管理、数据同步协调
- 直接使用 `SearchError` 无法表达协调器特有的错误场景（如空间不存在、标签不存在）

### 2.2 同步模块错误转换丢失上下文

**当前代码** (`src/sync/manager.rs`):
```rust
pub enum SyncError {
    #[error("Coordinator error: {0}")]
    CoordinatorError(String),  // 字符串形式，丢失原始错误类型
}
```

**问题**:
- 将协调器错误转为字符串，丢失错误类型信息
- 无法根据错误类型进行精确处理

### 2.3 错误传播链不完整

```
用户请求
    ↓
DDL执行器 (DBError)
    ↓
协调器 (SearchError) ← 类型不匹配
    ↓
搜索引擎 (SearchError)
```

## 3. 正确集成方案

### 3.1 分层错误设计原则

```
┌─────────────────────────────────────────────────────────────┐
│                    DBError (统一入口)                        │
├─────────────────────────────────────────────────────────────┤
│  StorageError │ QueryError │ FulltextError │ CoordinatorError │
├─────────────────────────────────────────────────────────────┤
│                    模块层错误类型                             │
│  SearchError (搜索) │ SyncError (同步) │ ...                  │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 各层应使用的错误类型

| 层级 | 模块 | 应使用错误类型 | 说明 |
|------|------|----------------|------|
| API层 | DDL执行器 | `DBError` | 统一入口，接收所有错误 |
| 协调层 | FulltextCoordinator | `CoordinatorResult<T>` | 协调器特有错误 + 包装下层错误 |
| 同步层 | SyncManager | `SyncResult<T>` | 同步特有错误 |
| 引擎层 | SearchEngine | `SearchResult<T>` | 搜索引擎特有错误 |

### 3.3 错误转换链

```
SearchError ──From──> FulltextError ──From──> CoordinatorError ──From──> DBError
     │                    │                      │
     └── 搜索引擎层 ────────┴── 全文检索层 ─────────┴── 协调层
```

## 4. 实施步骤

### 4.1 修改协调器错误类型

**文件**: `src/coordinator/fulltext.rs`

```rust
// 修改前
use crate::search::error::SearchError;

// 修改后
use crate::core::error::{CoordinatorError, CoordinatorResult, FulltextError};

impl FulltextCoordinator {
    pub async fn create_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        engine_type: Option<EngineType>,
    ) -> CoordinatorResult<String> {
        self.manager
            .create_index(space_id, tag_name, field_name, engine_type)
            .await
            .map_err(FulltextError::from)?;
        Ok(format!("{}_{}_{}", space_id, tag_name, field_name))
    }
}
```

### 4.2 修改同步模块错误类型

**文件**: `src/sync/manager.rs`

```rust
// 修改前
pub enum SyncError {
    #[error("Coordinator error: {0}")]
    CoordinatorError(String),
}

// 修改后
use crate::core::error::CoordinatorError;

pub enum SyncError {
    #[error("Coordinator error: {0}")]
    CoordinatorError(#[from] CoordinatorError),  // 直接包装，保留类型
}
```

### 4.3 添加缺失的错误转换

**文件**: `src/core/error/fulltext.rs`

```rust
// 添加从 SearchError 到 CoordinatorError 的直接转换
impl From<SearchError> for CoordinatorError {
    fn from(err: SearchError) -> Self {
        CoordinatorError::Fulltext(FulltextError::from(err))
    }
}
```

## 5. 错误类型定义详解

### 5.1 FulltextError - 全文检索引擎层错误

```rust
pub enum FulltextError {
    // 索引操作错误
    IndexNotFound(String),
    IndexAlreadyExists(String),
    EngineNotFound { space_id: u64, tag_name: String, field_name: String },
    EngineUnavailable(String),
    IndexCorrupted(String),
    
    // 引擎特定错误
    Bm25Error(String),
    InversearchError(String),
    
    // 操作错误
    QueryParseError(String),
    InvalidDocId(String),
    ConfigError(String),
    Timeout,
    Locked(String),
    Cancelled,
    Internal(String),
}
```

### 5.2 CoordinatorError - 协调器层错误

```rust
pub enum CoordinatorError {
    // 包装下层错误
    Fulltext(#[from] FulltextError),
    Sync(String),
    
    // 索引生命周期错误
    IndexCreationFailed { tag_name: String, field_name: String, reason: String },
    IndexDropFailed { tag_name: String, field_name: String, reason: String },
    IndexRebuildFailed(String),
    
    // 数据变更错误
    VertexChangeFailed(String),
    
    // 元数据错误
    SpaceNotFound(u64),
    TagNotFound(String),
    FieldNotIndexed { tag_name: String, field_name: String },
    
    // 状态错误
    NotInitialized,
    ShuttingDown,
    InvalidOperation(String),
    Internal(String),
}
```

## 6. 错误处理最佳实践

### 6.1 错误传播

```rust
// 正确：保留错误链
fn some_operation() -> CoordinatorResult<()> {
    let engine = manager.get_engine(...)
        .ok_or_else(|| CoordinatorError::FieldNotIndexed {
            tag_name: tag.to_string(),
            field_name: field.to_string(),
        })?;
    
    engine.search(query).await?;  // SearchError 自动转换为 FulltextError 再转换为 CoordinatorError
    Ok(())
}

// 错误：丢失错误信息
fn bad_operation() -> Result<(), String> {
    engine.search(query).await.map_err(|e| e.to_string())?;  // 丢失类型信息
    Ok(())
}
```

### 6.2 错误匹配

```rust
match result {
    Err(CoordinatorError::Fulltext(FulltextError::IndexNotFound(name))) => {
        // 精确处理索引不存在
    }
    Err(CoordinatorError::IndexCreationFailed { tag_name, field_name, reason }) => {
        // 精确处理索引创建失败
    }
    Err(e) => {
        // 其他错误
        return Err(DBError::from(e));
    }
}
```

## 7. 迁移检查清单

- [ ] 协调器 `FulltextCoordinator` 改用 `CoordinatorResult`
- [ ] 同步模块 `SyncError::CoordinatorError` 改为包装类型
- [ ] 添加 `From<SearchError> for CoordinatorError` 转换
- [ ] 更新所有使用协调器的代码
- [ ] 运行测试确保错误传播正确

## 8. 总结

### 8.1 当前已完成

1. ✅ 创建 `FulltextError` 和 `CoordinatorError` 错误类型
2. ✅ 实现 `From<SearchError> for FulltextError` 转换
3. ✅ 实现 `From<SyncError> for CoordinatorError` 转换
4. ✅ 将错误类型集成到 `DBError`

### 8.2 待完成

1. ⏳ 协调器改用 `CoordinatorResult`
2. ⏳ 同步模块错误类型优化
3. ⏳ 添加更多上下文信息的错误变体

### 8.3 设计原则

1. **分层隔离**：每层使用自己的错误类型，不跨层使用
2. **保留上下文**：使用 `#[from]` 和结构化错误变体保留完整错误链
3. **统一入口**：所有错误最终汇聚到 `DBError`
4. **可恢复性**：错误类型应支持精确匹配，便于错误恢复
