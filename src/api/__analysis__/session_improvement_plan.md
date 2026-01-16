# 会话管理改进方案

## 概述
本文档基于对nebula-graph原实现与新架构实现的对比分析，提出会话管理功能的短期、中期和长期改进方案。

## 当前实现状态

### ✅ 已实现功能
- **基础会话管理**: 创建、查找、删除会话
- **客户端会话**: 用户信息、空间管理、角色权限
- **服务层接口**: 认证、查询执行、权限检查
- **统计管理**: 会话和查询指标收集

### ❌ 缺失功能
- **分布式会话管理**（原meta server功能）
- **会话持久化**（完全内存实现）
- **高级会话操作**（SHOW/KILL SESSION/QUERY）
- **完整权限系统集成**
- **查询计划管理**

## 改进方案

### 🎯 短期改进（1-2周）

#### 1. 会话列表查询功能
**目标**: 实现SHOW SESSIONS功能
**参考nebula实现**:
```cpp
// nebula-3.8.0/src/graph/executor/admin/SessionExecutor.cpp
folly::Future<Status> ShowSessionsExecutor::listSessions() {
  return qctx()->getMetaClient()->listSessions().via(runner()).thenValue(
      [this](StatusOr<meta::cpp2::ListSessionsResp> resp) {
        // 处理响应并格式化输出
      });
}
```

**实现方案**:
```rust
// 在GraphSessionManager中添加
pub fn list_sessions(&self) -> Vec<SessionInfo> {
    let sessions = self.sessions.lock().unwrap();
    sessions.values().map(|session| {
        SessionInfo {
            session_id: session.id(),
            user_name: session.user(),
            space_name: session.space_name(),
            create_time: session.create_time(),
            graph_addr: session.graph_addr(),
            active_queries: session.active_queries_count(),
        }
    }).collect()
}
```

#### 2. 会话终止功能
**目标**: 实现KILL SESSION功能
**参考nebula实现**:
```cpp
// nebula-3.8.0/src/clients/meta/MetaClient.cpp
folly::Future<StatusOr<cpp2::RemoveSessionResp>> MetaClient::removeSessions(
    const std::vector<SessionID>& sessionIds)
```

**实现方案**:
```rust
// 在GraphSessionManager中添加
pub fn kill_session(&self, session_id: i64, current_user: &str) -> Result<(), SessionError> {
    // 1. 检查会话是否存在
    // 2. 检查当前用户权限（只有God或Admin可以kill其他用户会话）
    // 3. 终止会话中的所有查询
    // 4. 从缓存中移除会话
}
```

#### 3. 查询终止功能
**目标**: 实现KILL QUERY功能
**实现方案**:
```rust
// 在ClientSession中添加
pub fn kill_query(&self, query_id: u32) -> Result<(), QueryError> {
    // 1. 检查查询是否存在
    // 2. 标记查询为killed状态
    // 3. 从活动查询列表中移除
}
```

#### 4. 改进会话ID生成策略
**当前问题**: 使用简单时间戳
**改进方案**:
```rust
use std::sync::atomic::{AtomicU64, Ordering};

static SESSION_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

fn generate_session_id() -> i64 {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let counter = SESSION_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    // 组合时间戳和计数器，确保唯一性
    ((timestamp & 0xFFFFFFFFFFFF0000) | (counter & 0xFFFF)) as i64
}
```

#### 5. 完善错误处理机制
**定义专门的错误类型**:
```rust
#[derive(Debug, thiserror::Error)]
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
}
```

### 🎯 中期改进（2-4周）

#### 1. 完整权限系统集成
**参考nebula实现**:
```cpp
// nebula-3.8.0/src/graph/executor/admin/GrantRoleExecutor.cpp
auto *session = qctx_->rctx()->session();
PermissionManager::canWriteRole(session, grNode->role(), spaceId, *grNode->username())
```

**实现方案**:
```rust
// 扩展PermissionManager
impl PermissionManager {
    pub fn can_write_role(&self, session: &ClientSession, target_role: RoleType, 
                         space_id: i64, username: &str) -> Result<bool, PermissionError> {
        // 1. 检查当前会话用户角色
        // 2. 验证是否有权限授予目标角色
        // 3. 检查空间权限
    }
    
    pub fn grant_role(&self, username: &str, role: RoleType, space_id: i64) -> Result<(), PermissionError> {
        // 实现角色授予逻辑
    }
    
    pub fn revoke_role(&self, username: &str, space_id: i64) -> Result<(), PermissionError> {
        // 实现角色撤销逻辑
    }
}
```

#### 2. 查询计划管理
**实现功能**:
- 执行计划ID分配和管理
- 查询状态跟踪（运行中、已完成、已终止）
- 查询执行统计信息
- 查询超时管理

```rust
pub struct QueryPlan {
    pub plan_id: u32,
    pub session_id: i64,
    pub statement: String,
    pub start_time: Instant,
    pub status: QueryStatus,
    pub duration: Option<Duration>,
}

pub enum QueryStatus {
    Running,
    Completed,
    Failed(String),
    Killed,
    Timeout,
}
```

#### 3. 会话元数据管理
**实现功能**:
- 会话创建时间、最后活动时间记录
- 会话统计信息（总查询数、活跃查询数等）
- 会话配置参数管理
- 会话状态变更历史

### 🎯 长期规划（1-2个月）

#### 1. 分布式支持（可选）
如果需要支持多节点部署：
- 设计分布式会话存储方案
- 实现会话状态同步机制
- 处理网络分区和故障恢复

#### 2. 会话持久化（可选）
- 评估会话持久化需求
- 设计存储方案（RocksDB/文件系统）
- 实现会话恢复机制

## 实施计划

### 第一周
- [ ] 实现会话列表查询功能
- [ ] 改进会话ID生成策略
- [ ] 定义错误处理机制

### 第二周
- [ ] 实现会话终止功能
- [ ] 实现查询终止功能
- [ ] 添加操作日志记录

### 第三-四周
- [ ] 集成完整权限系统
- [ ] 实现查询计划管理
- [ ] 完善会话元数据管理

## 测试计划

### 单元测试
- 会话创建、查找、删除功能测试
- 权限检查逻辑测试
- 错误处理测试

### 集成测试
- 会话管理与其他组件集成测试
- 并发访问测试
- 性能测试

### 回归测试
- 确保现有功能不受影响
- 验证新功能正确性

## 风险评估

### 技术风险
- **并发访问**: 需要确保线程安全
- **内存管理**: 避免内存泄漏
- **性能影响**: 新功能不应显著影响性能

### 缓解措施
- 使用适当的同步机制（RwLock、Mutex）
- 实现资源清理机制
- 进行性能基准测试

## 结论

通过分阶段实施这些改进，可以逐步完善会话管理功能，使其更接近nebula-graph的完整实现，同时保持新架构的简洁性和高性能特性。短期改进可以立即提升用户体验，中期改进将提供完整的权限管理功能，长期规划则为可能的分布式部署做好准备。