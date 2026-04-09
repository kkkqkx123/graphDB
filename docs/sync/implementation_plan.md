# 全文索引自动数据同步实施方案

**文档版本**: 1.0  
**创建日期**: 2026-04-09  
**状态**: 实施中

---

## 执行摘要

本文档描述 GraphDB 全文索引自动数据同步的具体实施方案，基于对 PostgreSQL、Elasticsearch、SQLite FTS5 等成熟数据库系统的研究。

### 核心设计

**分层事件驱动架构**：

- **EventHub**：事件发布订阅系统（解耦层）
- **Storage Wrapper**：存储层事件包装器（Trigger 层）
- **Sync Handler**：全文索引同步处理器（业务层）

### 实施阶段

| 阶段    | 内容             | 周期 | 状态      |
| ------- | ---------------- | ---- | --------- |
| Phase 1 | 事件系统基础     | 2 周 | ⏳ 待开始 |
| Phase 2 | 同步 Handler     | 1 周 | ⏳ 待开始 |
| Phase 3 | 异步批量（可选） | 2 周 | ⏳ 待开始 |

---

## 一、架构设计

### 1.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                      Application Layer                       │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              EventEmittingStorage (包装器)                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  insert_vertex() {                                   │   │
│  │    result = inner.insert_vertex();                   │   │
│  │    event_hub.publish(VertexInserted);                │   │
│  │  }                                                   │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    MemoryEventHub (事件总线)                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │VertexHandler │  │ EdgeHandler  │  │IndexHandler  │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              FulltextSyncHandler (同步处理器)                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  handle_event(event) {                               │   │
│  │    match event {                                     │   │
│  │      VertexInserted => coordinator.index();          │   │
│  │      VertexUpdated => coordinator.update();          │   │
│  │      VertexDeleted => coordinator.delete();          │   │
│  │    }                                                 │   │
│  │  }                                                   │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              FulltextCoordinator (索引协调器)                │
└─────────────────────────────────────────────────────────────┘
```

### 1.2 核心组件

#### EventHub（事件总线）

**职责**：

- 发布事件到所有订阅的 Handler
- 管理订阅关系
- 支持同步/异步模式

**接口**：

```rust
pub trait EventHub: Send + Sync {
    fn publish(&self, event: StorageEvent) -> Result<(), EventError>;
    fn subscribe<F>(&self, event_type: EventType, handler: F) -> Result<SubscriptionId, EventError>;
    fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<(), EventError>;
}
```

#### StorageEvent（存储事件）

**类型**：

```rust
pub enum StorageEvent {
    VertexInserted { space_id: u64, vertex: Vertex, timestamp: u64 },
    VertexUpdated { space_id: u64, old_vertex: Vertex, new_vertex: Vertex, changed_fields: Vec<String>, timestamp: u64 },
    VertexDeleted { space_id: u64, vertex_id: Value, tag_name: String, timestamp: u64 },
}
```

#### FulltextSyncHandler（同步处理器）

**职责**：

- 监听 Vertex 事件
- 调用 FulltextCoordinator 更新索引
- 错误处理和重试

---

## 二、实施任务清单

### Phase 1: 事件系统基础

- [ ] **Task 1.1**: 创建 `src/event/mod.rs` - 定义模块结构
- [ ] **Task 1.2**: 创建 `src/event/types.rs` - 定义事件类型
- [ ] **Task 1.3**: 创建 `src/event/hub.rs` - 实现事件总线
- [ ] **Task 1.4**: 创建 `src/event/error.rs` - 定义错误类型
- [ ] **Task 1.5**: 创建 `src/storage/event_storage.rs` - 包装存储层
- [ ] **Task 1.6**: 修改 `src/storage/mod.rs` - 导出新模块
- [ ] **Task 1.7**: 编写单元测试
- [ ] **Task 1.8**: 编写集成测试

### Phase 2: 同步 Handler

- [ ] **Task 2.1**: 创建 `src/coordinator/fulltext_sync.rs` - 同步处理器
- [ ] **Task 2.2**: 修改 `src/coordinator/mod.rs` - 导出模块
- [ ] **Task 2.3**: 修改应用初始化代码 - 集成事件系统
- [ ] **Task 2.4**: 编写功能测试
- [ ] **Task 2.5**: 性能基准测试

### Phase 3: 异步批量（可选）

- [ ] **Task 3.1**: 实现异步队列
- [ ] **Task 3.2**: 实现批量处理器
- [ ] **Task 3.3**: 实现重试机制
- [ ] **Task 3.4**: 实现死信队列
- [ ] **Task 3.5**: 监控指标

---

## 三、代码结构

```
src/
├── event/                      # 新增：事件系统
│   ├── mod.rs                  # 模块导出
│   ├── types.rs                # 事件类型定义
│   ├── hub.rs                  # 事件总线实现
│   └── error.rs                # 错误类型定义
│
├── storage/
│   ├── mod.rs                  # 修改：导出 EventEmittingStorage
│   └── event_storage.rs        # 新增：事件包装器
│
├── coordinator/
│   ├── mod.rs                  # 修改：导出 FulltextSyncHandler
│   ├── fulltext.rs             # 现有：FulltextCoordinator
│   └── fulltext_sync.rs        # 新增：同步处理器
│
└── main.rs                     # 修改：初始化事件系统
```

---

## 四、配置

```toml
# config.toml

[fulltext]
enabled = true
default_engine = "Bm25"

[fulltext.sync]
# 同步模式：sync | async
mode = "sync"
# 是否启用自动同步
enabled = true
# 失败重试次数
max_retries = 3
# 重试间隔（毫秒）
retry_interval_ms = 1000
```

---

## 五、验收标准

### 功能验收

- [ ] 插入顶点自动同步到全文索引
- [ ] 更新顶点自动更新全文索引
- [ ] 删除顶点自动删除全文索引
- [ ] 支持启用/禁用自动同步
- [ ] 错误处理和重试机制正常工作

### 性能验收

- [ ] 同步模式写入延迟增加 < 10%
- [ ] 异步模式写入延迟增加 < 5%
- [ ] 10000 次写入无事件丢失

### 测试验收

- [ ] 单元测试覆盖率 > 80%
- [ ] 集成测试全部通过
- [ ] 性能测试通过

---

## 六、实施进度

| 任务    | 预计开始   | 预计完成   | 实际完成 | 状态 |
| ------- | ---------- | ---------- | -------- | ---- |
| Phase 1 | 2026-04-09 | 2026-04-23 | -        | ⏳   |
| Phase 2 | 2026-04-23 | 2026-04-30 | -        | ⏳   |
| Phase 3 | 2026-04-30 | 2026-05-14 | -        | ⏳   |

---

## 七、当前状态与后续工作

### 已完成的工作

#### 1. 事件系统核心模块（Phase 1）

- ✅ **src/event/mod.rs** - 事件系统模块导出
- ✅ **src/event/types.rs** - 事件类型定义
  - `StorageEvent` 枚举（顶点/边的增删改事件）
  - `EventType` 枚举（顶点事件/边事件）
  - `SyncConfig` 同步配置
  - `SubscriptionId` 订阅 ID 类型
- ✅ **src/event/hub.rs** - 事件总线实现
  - `EventHub` trait（发布/订阅接口）
  - `MemoryEventHub` 内存实现（使用 DashMap）
  - 支持多个订阅者监听同一事件类型
- ✅ **src/event/error.rs** - 事件错误类型
  - `EventError` 枚举
  - `EventHandlerResult` 类型别名
- ✅ **src/storage/event_storage.rs** - 存储层事件包装器
  - `EventEmittingStorage<S>` 包装器
  - `publish_event` 方法
  - 核心存储方法的事件发布（insert_vertex, update_vertex, delete_vertex 等）
- ✅ **src/lib.rs** - 导出 event 模块
- ✅ **src/storage/mod.rs** - 导出 EventEmittingStorage

#### 2. 同步处理器（Phase 2）

- ✅ **src/coordinator/fulltext_sync.rs** - 全文索引同步处理器
  - `FulltextSyncHandler` 结构体
  - `handle_event` 方法（事件分发）
  - `on_vertex_inserted` 处理顶点插入
  - `on_vertex_updated` 处理顶点更新
  - `on_vertex_deleted` 处理顶点删除
- ✅ **src/coordinator/mod.rs** - 导出 FulltextSyncHandler

#### 3. 文档

- ✅ **docs/sync/implementation_plan.md** - 完整实施方案文档

### 未完成的工作

#### 1. EventEmittingStorage 完整实现（高优先级）

**问题**: `StorageClient` trait 包含超过 50 个方法，当前只实现了部分核心方法。

**需要实现的方法**:

```rust
// 数据操作方法（需要事件发布）
- alter_edge_type
- create_tag_index
- drop_tag_index
- get_tag_index
- list_tag_indexes
- rebuild_tag_index
- create_edge_index
- drop_edge_index
- get_edge_index
- list_edge_indexes
- rebuild_edge_index
- insert_vertex_data
- insert_edge_data
- delete_vertex_data
- delete_edge_data
- update_data

// 用户管理方法（可选事件发布）
- change_password
- create_user
- alter_user
- drop_user
- grant_role
- revoke_role

// 索引查询方法（只读，不需要事件发布）
- lookup_index
- lookup_index_with_score

// 带 schema 的查询方法（可选事件发布）
- get_vertex_with_schema
- get_edge_with_schema
- scan_vertices_with_schema
- scan_edges_with_schema

// 持久化方法（可选事件发布）
- load_from_disk
- save_to_disk

// 统计和维护方法（可选事件发布）
- get_storage_stats
- find_dangling_edges
- repair_dangling_edges
- get_db_path
```

**建议方案**:

1. **方案 A - 使用宏生成**: 创建 `delegate_storage_methods!` 宏，自动生成所有透传代码
2. **方案 B - 分阶段实现**: 先实现核心方法，其他方法按需添加
3. **方案 C - 完整实现**: 一次性实现所有方法，保持代码完整性

#### 2. 事件订阅和注册（高优先级）

**需要完成**:

```rust
// 在 FulltextCoordinator 或应用启动时注册事件处理器
pub fn register_fulltext_sync(
    coordinator: Arc<FulltextCoordinator>,
    event_hub: Arc<dyn EventHub>,
) -> Result<(), EventError> {
    let handler = FulltextSyncHandler::new(coordinator);

    event_hub.subscribe(EventType::VertexEvent, move |event| {
        handler.handle_event(event)
    })?;

    Ok(())
}
```

#### 3. 异步批处理（Phase 3 - 中优先级）

**需要实现**:

- 异步事件队列（使用 tokio::sync::mpsc）
- 批量处理定时器
- 错误重试机制
- 背压处理

#### 4. 集成测试（中优先级）

**需要编写**:

```rust
#[test]
fn test_vertex_insert_sync() {
    // 1. 创建事件总线
    // 2. 创建 FulltextCoordinator
    // 3. 创建 EventEmittingStorage
    // 4. 注册 FulltextSyncHandler
    // 5. 插入顶点
    // 6. 验证全文索引中存在数据
}

#[test]
fn test_vertex_update_sync() {
    // 测试更新同步
}

#[test]
fn test_vertex_delete_sync() {
    // 测试删除同步
}
```

#### 5. 应用层集成（高优先级）

**需要修改**:

```rust
// src/main.rs 或应用初始化代码
fn initialize_storage() -> Result<EventEmittingStorage<DefaultStorage>> {
    let event_hub = Arc::new(MemoryEventHub::new());
    let storage = DefaultStorage::new(config)?;
    let mut event_storage = EventEmittingStorage::new(storage, event_hub.clone());

    // 注册全文索引同步处理器
    let coordinator = Arc::new(FulltextCoordinator::new(...));
    let sync_handler = FulltextSyncHandler::new(coordinator);

    event_hub.subscribe(EventType::VertexEvent, move |event| {
        sync_handler.handle_event(event)
    })?;

    event_storage.enable_events(true);

    Ok(event_storage)
}
```

### 已知问题

1. **项目原有编译错误**:
   - `src/query/planning/plan/core/nodes/data_access/vector_search.rs` 存在语法错误
   - `src/vector/coordinator.rs` 存在 struct 定义问题
   - `src/api/core/query_api.rs` 存在类型未定义错误

   这些错误与本次实现无关，需要单独修复。

2. **EventHub dyn 兼容性**:
   - `EventHub` trait 目前是 dyn 兼容的
   - 使用 `Arc<dyn EventHub>` 可以正常工作

3. **StorageClient 方法数量**:
   - trait 包含 50+ 个方法
   - 建议使用宏或代码生成工具减少重复代码

### 下一步行动建议

1. **立即**: 完成 `EventEmittingStorage` 的所有 `StorageClient` trait 方法实现
2. **短期**: 编写集成测试验证事件系统工作正常
3. **中期**: 实现异步批处理优化性能
4. **长期**: 根据实际使用情况调整和优化

---

**文档结束**
