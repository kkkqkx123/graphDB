# StorageClient 迁移进度总结

## 一、修改背景

根据 ARCHITECTURE_DESIGN.md 的设计，存储模块需要从原有的 `StorageEngine` trait 迁移到新的 `StorageClient` 统一接口。

### 主要变更

| 原有设计 | 新设计 |
|---------|-------|
| `StorageEngine` trait | `StorageClient` trait |
| 无 space 参数 | 所有方法添加 `space: &str` 参数 |
| `insert_node`, `get_node` | `insert_vertex`, `get_vertex` |
| 分散的 Reader/Writer 接口 | 统一的 StorageClient 接口 |

## 二、已完成修改

### 1. StorageClient 接口定义
- **文件**: `src/storage/storage_client.rs`
- **内容**: 定义了完整的 StorageClient trait，包含顶点、边、索引、Schema 管理等方法

### 2. MemoryStorage 实现
- **文件**: `src/storage/memory_storage.rs`
- **修改**: 为所有方法添加 space 参数，实现 StorageClient trait

### 3. RedbStorage 实现
- **文件**: `src/storage/redb_storage.rs`
- **修改**: 为所有方法添加 space 参数，实现 StorageClient trait

### 4. Mock 实现
- **文件**: `src/storage/test_mock.rs`
- **修改**: 更新 MockStorage 以实现 StorageClient trait

### 5. 调用方更新
已更新的文件：
- `src/query/executor/data_access.rs`
- `src/query/executor/data_modification.rs`
- `src/query/executor/admin/data/insert.rs`
- `src/query/executor/admin/data/update.rs`
- `src/query/executor/admin/tag/create_tag.rs`
- `src/query/executor/admin/edge/create_edge.rs`
- `src/query/executor/admin/index/tag_index.rs`
- `src/query/executor/admin/index/edge_index.rs`
- `src/query/executor/data_processing/graph_traversal/tests.rs`
- `src/query/executor/data_processing/graph_traversal/shorterst_path.rs`
- `src/query/executor/data_processing/graph_traversal/traversal_utils.rs`
- `src/query/executor/search_executors.rs`
- `src/query/planner/statements/seeks/index_seek.rs`
- `src/query/planner/statements/paths/shortest_path_planner.rs`
- `src/query/planner/statements/paths/match_path_planner.rs`
- `src/index/storage.rs`
- `src/query/context/managers/impl/index_manager_impl.rs`

### 6. 脚本工具
- `batch_replace.py`: 批量替换方法名 (insert_node -> insert_vertex)
- `fix_memory_storage.py`: 修复 MemoryStorage 方法签名
- `fix_redb_storage.py`: 修复 RedbStorage 方法签名
- `fix_storage_client_impl.py`: 修复 storage_client_impl.rs 调用

## 三、编译状态

### 当前错误统计
```
总错误数: 223
唯一错误模式: 104
涉及文件数: 16
```

### 主要错误来源

| 文件 | 错误数 | 主要问题 |
|------|--------|---------|
| `src/query/visitor/deduce_type_visitor.rs` | 61 | DBError 未定义、类型不匹配 |
| `src/query/context/managers/impl/storage_client_impl.rs` | 43 | StorageClient 方法调用方式 |
| `src/storage/test_mock.rs` | 41 | 方法签名不完整 |
| `src/storage/memory_storage.rs` | 27 | 方法签名缺少参数 |
| `src/storage/redb_storage.rs` | 19 | 方法签名问题 |

### 错误类型分析

1. **error[E0061]**: 方法参数数量不匹配 (20 个)
2. **error[E0599]**: 方法未找到 (47 个) - 主要是在 guard 上直接调用方法
3. **error[E0412]**: 类型未找到 (44 个) - DBError 未导入
4. **error[E0050]**: 方法参数数量与 trait 声明不匹配 (30 个)
5. **error[E0053]**: 方法实现与 trait 不兼容 (33 个)

## 四、后续修改任务

### 优先级 1：修复 storage_client_impl.rs (43 错误)

当前问题：在 `RwLockReadGuard` 上直接调用 `StorageClient` 方法。

解决方案：
```rust
// 错误写法
storage.scan_vertices("default")

// 正确写法
<MemoryStorage as StorageClient>::scan_vertices(&*storage, "default")
```

需要修复的方法：
- `scan_vertices`
- `scan_vertices_by_tag`
- `scan_vertices_by_prop`
- `scan_edges_by_type`
- `scan_all_edges`
- `get_vertex`
- `get_edge`
- `insert_vertex_data`
- `insert_edge_data`
- `update_data`

### 优先级 2：修复 test_mock.rs (41 错误)

需要添加的方法：
- `insert_vertex_data`
- `insert_edge_data`
- `delete_vertex_data`
- `delete_edge_data`
- `update_data`
- `change_password`
- `get_vertex_with_schema`
- `get_edge_with_schema`
- `scan_vertices_with_schema`
- `scan_edges_with_schema`

### 优先级 3：修复 deduce_type_visitor.rs (61 错误)

主要问题：
- `DBError` 未导入
- `StorageClient` trait bound 需要更新
- 部分方法调用缺少 space 参数

### 优先级 4：修复其他文件

- `src/storage/iterator/composite.rs` - FilterIter 迭代器问题
- `src/storage/transaction/snapshot.rs` - uses_snapshot 方法缺失
- `src/storage/transaction/lock.rs` - is_failure 方法缺失
- `src/expression/storage/row_reader.rs` - FieldType::Int 缺失

## 五、命名规范

### 方法名变更对照表

| 旧方法名 | 新方法名 |
|---------|---------|
| `insert_node` | `insert_vertex` |
| `get_node` | `get_vertex` |
| `update_node` | `update_vertex` |
| `delete_node` | `delete_vertex` |
| `batch_insert_nodes` | `batch_insert_vertices` |
| `scan_all_vertices` | `scan_vertices` |

### Space 参数规范

所有 StorageClient 方法的第一个参数必须是 `space: &str`，用于指定操作的图空间。

## 六、验证方法

运行以下命令检查编译状态：
```bash
cd d:\项目\database\graphDB
analyze_cargo --filter-warnings
```

## 七、注意事项

1. **不要滥用脚本**: 仅用于精确的字符串替换，复杂逻辑修改应手动进行
2. **保持代码风格**: 遵循 Rust 命名规范，使用 `_space` 前缀表示未使用的参数
3. **测试优先**: 在修改测试文件前，确保主代码编译通过
4. **逐步验证**: 每修改几个文件就运行一次 `cargo check`

## 八、参考文档

- `docs/storage/ARCHITECTURE_DESIGN.md` - 架构设计文档
- `docs/storage/PHASE1_TASKS.md` - 阶段1任务
- `docs/storage/PHASE2_TASKS.md` - 阶段2任务
- `src/storage/storage_client.rs` - StorageClient 接口定义
