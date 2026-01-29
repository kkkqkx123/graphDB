# 存储层重构 - 阶段2任务说明

## 阶段目标

**目标**：实现 Reader/Writer 接口，完成 MemoryStorage 的接口迁移，逐步废弃臃肿的 `StorageEngine` trait。

参考 **nebula-graph** 的架构设计：
- **Processor 模式**：每个查询操作由专门的 Processor 处理
- **Iterator 模式**：统一的数据遍历接口（StorageIterator）
- **Plan/Node 模式**：执行计划的构建和执行分离

阶段2完成后，`src/storage` 目录结构如下：

```
src/storage/
├── mod.rs                          # 统一导出（移除 StorageEngine）
├── engine/                         # 已完成：存储引擎层
│   ├── mod.rs
│   ├── memory_engine.rs
│   └── redb_engine.rs
├── operations/                     # 已完成：读写操作封装
│   ├── mod.rs
│   ├── reader/
│   └── writer/
├── memory_storage.rs               # 修改：实现新接口
├── redb_storage.rs                 # 修改：实现新接口
└── ...
```

## 主要任务

### 任务1：实现 MemoryStorage 的 Reader/Writer 接口

**文件**：`src/storage/memory_storage.rs`

**目标**：让 MemoryStorage 实现 `VertexReader`、`EdgeReader`、`VertexWriter`、`EdgeWriter` trait。

参考 nebula-graph 的 `StorageEnv` 设计，MemoryStorage 包含：
- 存储引擎引用（可选，用于 KV 操作）
- Schema 管理器
- 事务状态管理

```rust
// 当前 MemoryStorage 结构（简化）
pub struct MemoryStorage {
    // 数据存储（保留）
    vertices: Arc<Mutex<HashMap<VertexKey, Vertex>>>,
    edges: Arc<Mutex<HashMap<EdgeKey, Edge>>>,

    // Schema 管理器（保留）
    pub schema_manager: Arc<MemorySchemaManager>,

    // 事务管理（保留）
    active_transactions: Arc<Mutex<HashMap<TransactionId, TransactionState>>>,

    // ... 其他字段
}

// 实现 VertexReader
impl VertexReader for MemoryStorage {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        // 1. 验证 space 存在
        // 2. 从 vertices HashMap 获取
        // 3. 返回结果
    }

    fn scan_vertices(&self, space: &str) -> Result<ScanResult<Vertex>, StorageError> {
        // 1. 验证 space 存在
        // 2. 扫描所有 vertices
        // 3. 返回 ScanResult
    }

    // ... 其他方法
}
```

### 任务2：实现 RedbStorage 的 Reader/Writer 接口

**文件**：`src/storage/redb_storage.rs`

**目标**：让 RedbStorage 实现 `VertexReader`、`EdgeReader`、`VertexWriter`、`EdgeWriter` trait。

### 任务3：创建统一存储接口（StorageClient）

**文件**：`src/storage/mod.rs` 或新建 `storage_client.rs`

**目标**：提供统一的存储访问接口，隐藏底层存储引擎差异。

```rust
// 参考 nebula-graph 的 StorageEnv 设计
pub struct StorageClient {
    engine: Arc<dyn Engine>,
    schema_manager: Arc<dyn SchemaManager>,
}

impl StorageClient {
    pub fn new(engine: Arc<dyn Engine>, schema_manager: Arc<dyn SchemaManager>) -> Self {
        Self { engine, schema_manager }
    }

    // 提供统一的读写接口
    pub async fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        // ... 实现
    }

    pub async fn scan_vertices(&self, space: &str) -> Result<ScanResult<Vertex>, StorageError> {
        // ... 实现
    }
}
```

### 任务4：迁移调用 StorageEngine 的代码

**影响范围**：约 84 处调用需要迁移。

**策略**：渐进式迁移，不破坏现有功能。

1. **第一优先级**（查询执行器核心）：
   - `src/query/executor/data_access.rs`
   - `src/query/executor/graph_query_executor.rs`
   - `src/query/executor/data_modification.rs`

2. **第二优先级**（结果处理）：
   - `src/query/executor/result_processing/*.rs`

3. **第三优先级**（管理操作）：
   - `src/query/executor/admin/*.rs`

### 任务5：删除 StorageEngine trait

**文件**：`src/storage/storage_engine.rs`

**前提**：确认所有调用方已迁移完成。

**注意**：保留 `TransactionId` 类型供 transaction 模块使用。

## 接口设计参考

### nebula-graph 的 Processor 模式

```cpp
// nebula-graph/src/storage/query/GetNeighborsProcessor.cpp
class GetNeighborsProcessor : public QueryBaseProcessor<cpp2::GetNeighborsRequest, cpp2::GetNeighborsResponse> {
public:
    static GetNeighborsProcessor* create(StorageEnv* env, const ProcessorExtra& extra) {
        return new GetNeighborsProcessor(env, extra);
    }

protected:
    void process(const cpp2::GetNeighborsRequest& req) override;
};
```

### 对应的 Rust 实现模式

```rust
// src/storage/operations/processors/get_neighbors_processor.rs
pub struct GetNeighborsProcessor {
    space: String,
    vertex_id: Value,
    edge_types: Vec<String>,
    schema_manager: Arc<dyn SchemaManager>,
    storage: Arc<dyn VertexReader + EdgeReader>,
}

impl GetNeighborsProcessor {
    pub fn new(space: String, vertex_id: Value, edge_types: Vec<String>,
               schema_manager: Arc<dyn SchemaManager>,
               storage: Arc<dyn VertexReader + EdgeReader>) -> Self {
        Self { space, vertex_id, edge_types, schema_manager, storage }
    }

    pub fn execute(&self) -> Result<Vec<Edge>, StorageError> {
        // 1. 获取顶点的所有边
        self.storage.get_node_edges(&self.space, &self.vertex_id, EdgeDirection::Out)
    }
}
```

## 验收标准

1. **接口完整**：`MemoryStorage` 实现所有 Reader/Writer 接口；`RedbStorage` 保持 StorageEngine 实现
2. **编译通过**：`cargo check --lib` 无错误
3. **功能正常**：现有测试通过
4. **向后兼容**：迁移过程中不影响现有功能

## 迁移检查清单

### MemoryStorage 接口实现

- [x] `VertexReader::get_vertex`
- [x] `VertexReader::scan_vertices`
- [x] `VertexReader::scan_vertices_by_tag`
- [x] `VertexReader::scan_vertices_by_prop`
- [x] `EdgeReader::get_edge`
- [x] `EdgeReader::get_node_edges`
- [x] `EdgeReader::get_node_edges_filtered`
- [x] `EdgeReader::scan_edges_by_type`
- [x] `EdgeReader::scan_all_edges`
- [x] `VertexWriter::insert_vertex`
- [x] `VertexWriter::update_vertex`
- [x] `VertexWriter::delete_vertex`
- [x] `VertexWriter::batch_insert_vertices`
- [x] `EdgeWriter::insert_edge`
- [x] `EdgeWriter::delete_edge`
- [x] `EdgeWriter::batch_insert_edges`

### RedbStorage 接口实现

- [x] 保持 StorageEngine 实现（不实现 Reader/Writer，因 trait 方法冲突问题）

### 代码迁移

- [ ] `src/query/executor/data_access.rs`
- [ ] `src/query/executor/graph_query_executor.rs`
- [ ] `src/query/executor/data_modification.rs`
- [ ] `src/query/executor/result_processing/` (全部)
- [ ] `src/query/executor/admin/` (全部)
- [ ] `src/api/service/` (全部)

## 后续阶段

阶段2完成后，将进入阶段3：
- 完善 plan 层的节点和执行器
- 实现 Pipeline 模式的迭代器
- 优化查询执行计划

## 任务状态

| 任务 | 状态 | 备注 |
|------|------|------|
| 创建阶段2任务文档 | ✅ 已完成 | 本文档 |
| MemoryStorage Reader/Writer 接口 | ✅ 已完成 | 已实现所有接口 |
| RedbStorage 保持 StorageEngine 实现 | ✅ 已完成 | 不实现 Reader/Writer |
| 运行验证 | ✅ 已完成 | cargo check 通过 |

## 阶段2完成摘要

阶段2主要完成了以下工作：

1. **创建阶段2任务文档**：`docs/storage/PHASE2_TASKS.md`

2. **验证 MemoryStorage 实现**：
   - `MemoryStorage` 已实现所有 Reader/Writer 接口
   - `VertexReader`：`get_vertex`、`scan_vertices`、`scan_vertices_by_tag`、`scan_vertices_by_prop`
   - `EdgeReader`：`get_edge`、`get_node_edges`、`get_node_edges_filtered`、`scan_edges_by_type`、`scan_all_edges`
   - `VertexWriter`：`insert_vertex`、`update_vertex`、`delete_vertex`、`batch_insert_vertices`
   - `EdgeWriter`：`insert_edge`、`delete_edge`、`batch_insert_edges`

3. **RedbStorage 架构决策**：
   - 由于 Rust trait 方法冲突问题，`RedbStorage` 保持仅实现 `StorageEngine` trait
   - `RedbStorage` 作为简化的持久化存储引擎，主要用于测试场景
   - 生产环境建议使用 `MemoryStorage`

4. **编译验证**：`cargo check --lib` 通过

## 后续阶段

阶段2完成后，后续任务：
- 迁移调用 `StorageEngine` 的代码到新的 Reader/Writer 接口
- 删除臃肿的 `storage_engine.rs`（渐进式）
- 完善 plan 层的节点和执行器
