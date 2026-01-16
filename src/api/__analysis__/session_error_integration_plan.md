# 会话错误整合方案

## 整合目标
将 `src/api/session/errors.rs` 中的错误类型整合到 `src/core/error.rs` 的统一错误处理体系中，保持与项目现有架构的一致性。

## 当前状态

### 会话模块错误类型
```rust
// src/api/session/errors.rs
pub enum SessionError {
    SessionNotFound(i64),
    PermissionDenied,
    SessionExpired,
    MaxConnectionsExceeded,
    QueryNotFound(u32),
    KillSessionFailed(String),
    ManagerError(String),
}

pub enum QueryError {
    QueryNotFound(u32),
    KillQueryFailed(String),
    ExecutionError(String),
}

pub enum PermissionError {
    InsufficientPermission,
    RoleNotFound(String),
    GrantRoleFailed(String),
    RevokeRoleFailed(String),
    UserNotFound(String),
}
```

### 核心错误类型
```rust
// src/core/error.rs
pub enum DBError {
    Storage(#[from] StorageError),
    Query(#[from] QueryError),
    Expression(#[from] ExpressionErrorType),
    Plan(#[from] PlanNodeVisitError),
    Lock(#[from] LockError),
    Manager(#[from] ManagerError),
    Validation(String),
    Io(#[from] std::io::Error),
    TypeDeduction(String),
    Serialization(String),
    Index(String),
    Transaction(String),
    Internal(String),
}
```

## 整合方案

### 方案一：直接整合到DBError（推荐）

```rust
// 在 src/core/error.rs 中添加
#[derive(Error, Debug)]
pub enum DBError {
    // ... 现有错误类型 ...
    
    #[error("会话错误: {0}")]
    Session(#[from] SessionError),
    
    #[error("权限错误: {0}")]
    Permission(#[from] PermissionError),
    
    // ... 其他错误类型 ...
}

/// 会话相关错误
#[derive(Error, Debug, Clone)]
pub enum SessionError {
    #[error("会话不存在: {0}")]
    SessionNotFound(i64),
    
    #[error("权限不足，无法执行此操作")]
    PermissionDenied,
    
    #[error("会话已过期")]
    SessionExpired,
    
    #[error("超过最大连接数限制")]
    MaxConnectionsExceeded,
    
    #[error("查询不存在: {0}")]
    QueryNotFound(u32),
    
    #[error("无法终止会话: {0}")]
    KillSessionFailed(String),
    
    #[error("会话管理器错误: {0}")]
    ManagerError(String),
}

/// 权限相关错误
#[derive(Error, Debug, Clone)]
pub enum PermissionError {
    #[error("权限不足")]
    InsufficientPermission,
    
    #[error("角色不存在: {0}")]
    RoleNotFound(String),
    
    #[error("无法授予角色: {0}")]
    GrantRoleFailed(String),
    
    #[error("无法撤销角色: {0}")]
    RevokeRoleFailed(String),
    
    #[error("用户不存在: {0}")]
    UserNotFound(String),
}
```

### 方案二：保持独立但提供转换

```rust
// 保持 src/api/session/errors.rs 独立
// 但在 src/core/error.rs 中提供转换

impl From<SessionError> for DBError {
    fn from(error: SessionError) -> Self {
        DBError::Session(error)
    }
}

impl From<PermissionError> for DBError {
    fn from(error: PermissionError) -> Self {
        DBError::Permission(error)
    }
}
```

## 实施步骤

### 第一步：移动错误定义
1. 将 `SessionError` 和 `PermissionError` 定义移动到 `src/core/error.rs`
2. 删除 `src/api/session/errors.rs` 文件
3. 更新 `src/api/session/mod.rs` 中的导出

### 第二步：更新类型别名
```rust
// 在 src/core/error.rs 中添加类型别名
pub type SessionResult<T> = Result<T, SessionError>;
pub type PermissionResult<T> = Result<T, PermissionError>;
```

### 第三步：更新会话管理代码
```rust
// src/api/session/session_manager.rs
use crate::core::error::{SessionError, SessionResult};

impl GraphSessionManager {
    pub fn kill_session(&self, session_id: i64, current_user: &str, is_god: bool) -> SessionResult<()> {
        // 现有实现...
    }
}
```

### 第四步：更新GraphService
```rust
// src/api/service/graph_service.rs
use crate::core::error::{DBError, SessionError};

impl<S: StorageEngine + Clone + 'static> GraphService<S> {
    pub fn kill_session(&self, session_id: i64, current_user: &str) -> Result<(), DBError> {
        // 获取当前会话以检查权限
        let current_session = self.session_manager.find_session(session_id)
            .ok_or_else(|| SessionError::SessionNotFound(session_id))?;
        
        let is_god = current_session.is_god();
        
        self.session_manager.kill_session(session_id, current_user, is_god)?;
        self.stats_manager.dec_value(MetricType::NumActiveSessions);
        
        Ok(())
    }
}
```

## 优势分析

### 整合方案优势
1. **统一性**：所有错误都通过 `DBError` 统一处理
2. **自动转换**：使用 `#[from]` 属性自动实现 `From` trait
3. **一致性**：与项目中其他模块的错误处理保持一致
4. **简化接口**：可以使用统一的 `DBResult<T>` 类型

### 保持独立的优势
1. **模块化**：会话错误与核心错误分离
2. **灵活性**：可以独立修改会话错误而不影响核心
3. **清晰性**：错误类型更加具体和明确

## 推荐方案

**推荐采用方案一（直接整合）**，原因：

1. **符合项目设计原则**：项目已经建立了统一错误处理架构
2. **会话是核心功能**：会话管理是数据库的核心功能，其错误应该纳入统一体系
3. **便于使用**：其他模块可以方便地处理会话错误
4. **维护简单**：错误处理逻辑集中在一处

## 注意事项

1. **向后兼容性**：确保现有代码能够平滑迁移
2. **错误信息**：保持中文错误信息，符合项目规范
3. **测试覆盖**：更新相关测试用例
4. **文档更新**：同步更新相关文档

## 实施影响

### 正面影响
- 统一的错误处理体验
- 更好的错误传播机制
- 与项目架构保持一致

### 潜在影响
- 需要修改现有代码
- 可能需要更新依赖关系
- 测试用例需要相应调整

## 结论

将会话错误整合到核心错误系统中是最佳选择，这符合GraphDB项目的整体设计原则，能够提供更好的错误处理体验和代码维护性。