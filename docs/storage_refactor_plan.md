# Storage 层重构计划

## 目标

删除 `redb_storage.rs` 中的 Trait 实现，统一使用 `RedbReader`/`RedbWriter` 进行操作。

## 当前问题

1. **重复实现**：`redb_storage.rs` 和 `redb_operations.rs` 都实现了 `VertexReader`/`EdgeReader`/`VertexWriter`/`EdgeWriter`
2. **职责混乱**：`RedbStorage` 既管理元数据又实现数据操作
3. **维护困难**：相同逻辑分散在两个地方

## 分阶段修改方案

### 阶段1：创建辅助模块

创建 `src/storage/utils/` 目录，存放通用工具函数：

```
src/storage/
├── utils/
│   ├── mod.rs           # 导出工具模块
│   ├── serialization.rs # 序列化/反序列化工具
│   └── schema_builder.rs # Schema 构建工具
```

**内容**：
- `serialization.rs`：从 `redb_storage.rs` 提取 `serialize_value`, `deserialize_value`, `encode_vertex_key`, `encode_edge_key`
- `schema_builder.rs`：从 `redb_storage.rs` 提取 `build_vertex_schema`, `build_edge_schema`

### 阶段2：修改 RedbStorage 结构

**修改 `RedbStorage` 结构体**：

```rust
pub struct RedbStorage {
    db: Arc<Database>,                    // redb 数据库
    reader: RedbReader,                   // 读取操作
    writer: Arc<Mutex<RedbWriter>>,       // 写入操作（需要可变）
    // 保留元数据管理字段
    spaces: Arc<Mutex<HashMap<String, SpaceInfo>>>,
    tags: Arc<Mutex<HashMap<String, HashMap<String, TagInfo>>>>,
    edge_type_infos: Arc<Mutex<HashMap<String, HashMap<String, EdgeTypeSchema>>>>,
    tag_indexes: Arc<Mutex<HashMap<String, HashMap<String, Index>>>>,
    edge_indexes: Arc<Mutex<HashMap<String, HashMap<String, Index>>>>,
    users: Arc<Mutex<HashMap<String, UserInfo>>>,
    schema_manager: Arc<MemorySchemaManager>,
    extended_schema_manager: Arc<RedbExtendedSchemaManager>,
    db_path: PathBuf,
}
```

**删除字段**：
- `engine: Arc<Mutex<E>>` - 不再使用通用引擎
- `_id_generator` - 如果未使用

### 阶段3：删除 Trait 实现

从 `redb_storage.rs` 中删除以下实现：

1. `impl<E: Engine> VertexReader for RedbStorage<E>`
2. `impl<E: Engine> EdgeReader for RedbStorage<E>`
3. `impl<E: Engine> VertexWriter for RedbStorage<E>`
4. `impl<E: Engine> EdgeWriter for RedbStorage<E>`

**保留**：
- `impl<E: Engine> StorageClient for RedbStorage<E>` - 但改为委托给 `RedbReader`/`RedbWriter`

### 阶段4：更新 StorageClient 实现

将 `StorageClient` 的方法实现改为委托模式：

```rust
impl StorageClient for RedbStorage {
    fn get_vertex(&self, space: &str, id: &Value) -> Result<Option<Vertex>, StorageError> {
        self.reader.get_vertex(space, id)
    }
    
    fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
        let mut writer = self.writer.lock().map_err(|e| StorageError::DbError(e.to_string()))?;
        writer.insert_vertex(space, vertex)
    }
    // ... 其他方法类似
}
```

### 阶段5：更新索引和辅助方法

保留索引管理方法，但更新其实现以使用新的结构：
- `update_vertex_indexes`
- `update_edge_indexes`
- `delete_vertex_indexes`
- `delete_edge_indexes`
- `lookup_tag_index`
- `lookup_edge_index`

### 阶段6：更新模块导出

修改 `src/storage/mod.rs` 确保正确导出：

```rust
pub use operations::{RedbReader, RedbWriter, VertexReader, EdgeReader, VertexWriter, EdgeWriter};
```

## 迁移检查清单

- [ ] 创建 `utils/serialization.rs`
- [ ] 创建 `utils/schema_builder.rs`
- [ ] 修改 `RedbStorage` 结构体
- [ ] 删除 `VertexReader` 实现
- [ ] 删除 `EdgeReader` 实现
- [ ] 删除 `VertexWriter` 实现
- [ ] 删除 `EdgeWriter` 实现
- [ ] 更新 `StorageClient` 实现
- [ ] 更新索引方法
- [ ] 更新 `mod.rs` 导出
- [ ] 运行 `cargo test --lib`
- [ ] 运行集成测试

## 风险评估

**低风险**：
- 工具函数提取只是代码移动
- `RedbReader`/`RedbWriter` 已经存在且经过测试

**中风险**：
- `StorageClient` 实现变更需要确保所有方法正确委托
- 索引管理方法需要适配新的存储结构

**缓解措施**：
- 保持原有方法签名不变
- 逐步替换，确保每步编译通过
- 充分运行测试验证
