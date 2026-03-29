# Web 管理模块核心 API 集成方案

## 概述

本文档描述 Web 管理模块（web management）与 GraphDB 核心 API 的集成方案，解决当前 handlers 中 TODO 标记的缺失功能。

## 当前状态分析

### 已完成的集成

1. **基础架构**
   - WebState 已集成 core_state，可访问核心服务
   - 认证中间件已实现，支持 session 验证
   - 路由结构已优化，支持泛型状态管理

2. **已集成的核心 API**
   - `GraphService::execute()` - 用于执行查询语句
   - `SchemaApi::create_space/drop_space/use_space` - Space 基础操作
   - `SchemaApi::create_tag/drop_tag` - Tag 基础操作
   - `SchemaApi::create_edge_type/drop_edge_type` - EdgeType 基础操作
   - `SchemaApi::create_index/drop_index` - Index 基础操作

### 待完成的集成

当前有 20 个 TODO 需要处理：

| 模块 | TODO 数量 | 类型 |
|------|----------|------|
| schema_ext.rs | 18 | 列表查询、更新操作、统计信息 |
| data_browser.rs | 2 | 数据总数统计 |

## 核心 API 能力分析

### 1. SchemaApi 已提供的方法

位于 `src/api/core/schema_api.rs`：

```rust
// Space 管理
- create_space(&self, name: &str, config: SpaceConfig) -> CoreResult<()>
- drop_space(&self, name: &str) -> CoreResult<()>
- use_space(&self, name: &str) -> CoreResult<u64>

// Tag 管理
- create_tag(&self, space_id: u64, name: &str, properties: Vec<PropertyDef>) -> CoreResult<()>
- drop_tag(&self, space_id: u64, name: &str) -> CoreResult<()>

// EdgeType 管理
- create_edge_type(&self, space_id: u64, name: &str, properties: Vec<PropertyDef>) -> CoreResult<()>
- drop_edge_type(&self, space_id: u64, name: &str) -> CoreResult<()>

// Index 管理
- create_index(&self, space_id: u64, name: &str, target: IndexTarget) -> CoreResult<()>
- drop_index(&self, space_id: u64, name: &str) -> CoreResult<()>

// Schema 描述
- describe_schema(&self, space_id: u64) -> CoreResult<String>
```

### 2. StorageClient 已提供的方法

位于 `src/storage/storage_client.rs`：

```rust
// Space 查询
- get_space(&self, space: &str) -> Result<Option<SpaceInfo>>
- get_space_by_id(&self, space_id: u64) -> Result<Option<SpaceInfo>>
- list_spaces(&self) -> Result<Vec<SpaceInfo>>
- get_space_id(&self, space: &str) -> Result<u64>

// Tag 查询
- get_tag(&self, space: &str, tag: &str) -> Result<Option<TagInfo>>
- list_tags(&self, space: &str) -> Result<Vec<TagInfo>>

// EdgeType 查询
- get_edge_type(&self, space: &str, edge: &str) -> Result<Option<EdgeTypeInfo>>
- list_edge_types(&self, space: &str) -> Result<Vec<EdgeTypeInfo>>

// Index 查询
- get_tag_index(&self, space: &str, index: &str) -> Result<Option<Index>>
- list_tag_indexes(&self, space: &str) -> Result<Vec<Index>>
- get_edge_index(&self, space: &str, index: &str) -> Result<Option<Index>>
- list_edge_indexes(&self, space: &str) -> Result<Vec<Index>>
```

### 3. 需要扩展的核心 API

#### SchemaApi 需要添加

```rust
// Tag/EdgeType 更新
pub fn alter_tag(
    &self,
    space_id: u64,
    name: &str,
    operations: Vec<AlterTagOperation>
) -> CoreResult<()>

pub fn alter_edge_type(
    &self,
    space_id: u64,
    name: &str,
    operations: Vec<AlterEdgeOperation>
) -> CoreResult<()>

// Index 重建
pub fn rebuild_index(&self, space_id: u64, name: &str) -> CoreResult<()>
```

#### StorageClient 需要添加

```rust
// 数据统计
fn count_vertices(&self, space: &str, tag: Option<&str>) -> Result<u64, StorageError>;
fn count_edges(&self, space: &str, edge_type: Option<&str>) -> Result<u64, StorageError>;
```

## 集成方案

### 方案一：直接使用 StorageClient（推荐）

对于列表查询类操作，直接通过 `web_state.core_state.server.get_storage()` 获取 StorageClient 并调用对应方法。

**适用场景**：
- list_spaces
- list_tags / get_tag
- list_edge_types / get_edge_type
- list_tag_indexes / list_edge_indexes / get_tag_index / get_edge_index

**优势**：
- 无需修改核心 API
- 直接访问存储层，性能更好
- 代码简洁

**示例代码**：

```rust
async fn list_spaces<S: StorageClient + Clone + Send + Sync + 'static>(
    State(web_state): State<WebState<S>>,
) -> WebResult<Json<ApiResponse<serde_json::Value>>> {
    let result = task::spawn_blocking(move || {
        let storage = web_state.core_state.server.get_storage();
        let storage = storage.lock();
        
        let spaces = storage
            .list_spaces()
            .map_err(|e| WebError::Storage(e.to_string()))?;
        
        let space_list: Vec<SpaceSummary> = spaces
            .into_iter()
            .map(|s| SpaceSummary {
                id: s.space_id,
                name: s.space_name,
                vid_type: format!("{:?}", s.vid_type),
                partition_num: 100,
                replica_factor: 1,
                comment: s.comment,
                created_at: s.created_at,
            })
            .collect();

        Ok::<_, WebError>(serde_json::json!({
            "spaces": space_list,
        }))
    })
    .await
    .map_err(|e| WebError::Internal(format!("Task execution failed: {}", e)))?;

    Ok(Json(ApiResponse::success(result?)))
}
```

### 方案二：扩展 SchemaApi

对于更新类操作（alter_tag, alter_edge_type）和特殊操作（rebuild_index），需要在 SchemaApi 中实现。

**适用场景**：
- update_tag (需要 alter_tag)
- update_edge_type (需要 alter_edge_type)
- rebuild_index

**实现步骤**：
1. 在 `src/api/core/schema_api.rs` 添加新方法
2. 在 `src/storage/storage_client.rs` 添加对应的存储层方法
3. 在 web handlers 中调用新的 SchemaApi 方法

### 方案三：使用查询引擎

对于复杂的数据统计和查询，使用 `graph_service.execute()` 执行 nGQL 语句。

**适用场景**：
- 数据总数统计（当 StorageClient 未提供直接方法时）
- 复杂的图遍历查询

**示例代码**：

```rust
// 统计顶点数量
let query = format!("USE {}; MATCH (v:{}) RETURN count(v) as count", space_name, tag_name);
match graph_service.execute(session_id, &query) {
    Ok(ExecutionResult::Values(values)) => {
        if let Some(Value::Int(count)) = values.first() {
            *count as i64
        } else { 0 }
    }
    _ => 0,
}
```

## 实施计划

### 第一阶段：StorageClient 列表查询集成（高优先级）

可直接解决的 TODO（12个）：

1. `list_spaces` - 使用 `storage.list_spaces()`
2. `get_space_details` - 使用 `storage.get_space()`
3. `list_tags` - 使用 `storage.list_tags()`
4. `get_tag` - 使用 `storage.get_tag()`
5. `list_edge_types` - 使用 `storage.list_edge_types()`
6. `get_edge_type` - 使用 `storage.get_edge_type()`
7. `list_indexes` - 使用 `storage.list_tag_indexes()` + `storage.list_edge_indexes()`
8. `get_index` - 使用 `storage.get_tag_index()` + `storage.get_edge_index()`

### 第二阶段：核心 API 扩展（中优先级）

需要扩展 SchemaApi 和 StorageClient：

1. 实现 `SchemaApi::alter_tag` - 支持 Tag 属性修改
2. 实现 `SchemaApi::alter_edge_type` - 支持 EdgeType 属性修改
3. 实现 `SchemaApi::rebuild_index` - 支持索引重建
4. 实现 `StorageClient::count_vertices/count_edges` - 支持数据统计

### 第三阶段：查询引擎集成（低优先级）

对于无法直接通过 StorageClient 获取的统计信息，使用查询引擎：

1. Space 统计信息（tag_count, edge_type_count, index_count）
2. 数据分页的总数统计

## 代码修改清单

### Web Handlers 修改

1. **schema_ext.rs**
   - 修改 `list_spaces` - 使用 StorageClient
   - 修改 `get_space_details` - 使用 StorageClient
   - 修改 `get_space_statistics` - 使用查询引擎或扩展 API
   - 修改 `list_tags` - 使用 StorageClient
   - 修改 `get_tag` - 使用 StorageClient
   - 修改 `update_tag` - 使用扩展后的 SchemaApi
   - 修改 `list_edge_types` - 使用 StorageClient
   - 修改 `get_edge_type` - 使用 StorageClient
   - 修改 `update_edge_type` - 使用扩展后的 SchemaApi
   - 修改 `list_indexes` - 使用 StorageClient
   - 修改 `get_index` - 使用 StorageClient
   - 修改 `rebuild_index` - 使用扩展后的 SchemaApi

2. **data_browser.rs**
   - 修改 `list_vertices_by_tag` - 使用 StorageClient 获取总数
   - 修改 `list_edges_by_type` - 使用 StorageClient 获取总数

### 核心 API 扩展

1. **src/api/core/schema_api.rs**
   - 添加 `alter_tag` 方法
   - 添加 `alter_edge_type` 方法
   - 添加 `rebuild_index` 方法

2. **src/storage/storage_client.rs**
   - 添加 `count_vertices` 方法
   - 添加 `count_edges` 方法

3. **src/storage/redb_storage.rs**
   - 实现 `count_vertices` 方法
   - 实现 `count_edges` 方法

## 注意事项

1. **线程安全**：所有 StorageClient 调用都需要在 `task::spawn_blocking` 中执行，避免阻塞异步运行时

2. **错误处理**：统一使用 `WebError` 进行错误转换，保持 API 响应格式一致

3. **性能考虑**：
   - 列表查询使用 StorageClient 直接访问存储层
   - 避免在循环中重复获取 storage lock
   - 大数据量查询考虑添加分页限制

4. **兼容性**：
   - 保持现有 API 接口不变
   - 新增方法使用默认实现或返回未实现错误
