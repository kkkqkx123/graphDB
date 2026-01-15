# 存储模块架构分析

## 目录结构

```
src/storage/
├── mod.rs                    # 模块入口，导出公共接口
├── storage_engine.rs         # 存储引擎trait定义
├── native_storage.rs         # 基于sled的原生存储实现
├── test_mock.rs              # 测试用Mock存储引擎
└── iterator/                 # 迭代器模块
    ├── mod.rs                # 迭代器trait定义
    ├── default_iter.rs       # 默认迭代器（单值）
    ├── sequential_iter.rs    # 顺序迭代器（DataSet行级）
    ├── get_neighbors_iter.rs # 邻居查询迭代器（图遍历）
    └── prop_iter.rs          # 属性查询迭代器
```

## 核心设计

### 1. 存储引擎抽象层

#### StorageEngine Trait

定义了图数据库存储引擎的核心接口，支持顶点和边的CRUD操作、扫描操作以及事务管理。

**核心方法：**

**顶点操作：**
- `insert_node(vertex: Vertex) -> Result<Value, StorageError>` - 插入顶点，返回生成的ID
- `get_node(id: &Value) -> Result<Option<Vertex>, StorageError>` - 根据ID获取顶点
- `update_node(vertex: Vertex) -> Result<(), StorageError>` - 更新顶点
- `delete_node(id: &Value) -> Result<(), StorageError>` - 删除顶点（级联删除关联边）
- `scan_all_vertices() -> Result<Vec<Vertex>, StorageError>` - 全表扫描所有顶点
- `scan_vertices_by_tag(tag: &str) -> Result<Vec<Vertex>, StorageError>` - 按标签扫描顶点

**边操作：**
- `insert_edge(edge: Edge) -> Result<(), StorageError>` - 插入边
- `get_edge(src, dst, edge_type) -> Result<Option<Edge>, StorageError>` - 获取指定边
- `get_node_edges(node_id, direction) -> Result<Vec<Edge>, StorageError>` - 获取顶点的边（支持方向）
- `delete_edge(src, dst, edge_type) -> Result<(), StorageError>` - 删除边
- `scan_edges_by_type(edge_type) -> Result<Vec<Edge>, StorageError>` - 按类型扫描边
- `scan_all_edges() -> Result<Vec<Edge>, StorageError>` - 全表扫描所有边

**事务操作：**
- `begin_transaction() -> Result<TransactionId, StorageError>` - 开始事务
- `commit_transaction(tx_id) -> Result<(), StorageError>` - 提交事务
- `rollback_transaction(tx_id) -> Result<(), StorageError>` - 回滚事务

### 2. NativeStorage实现

基于[sled](https://github.com/spacejam/sled)嵌入式数据库的原生存储实现，提供高性能的图数据存储。

**数据结构：**

```rust
pub struct NativeStorage {
    db: Db,                          // sled数据库实例
    nodes_tree: Tree,                // 顶点数据树
    edges_tree: Tree,                // 边数据树
    schema_tree: Tree,               // 模式信息树
    node_edge_index: Tree,          // 节点-边索引：node_id -> [edge_id]
    edge_type_index: Tree,           // 边类型索引：edge_type -> [edge_key]
    db_path: String,                 // 数据库路径
}
```

**索引策略：**

1. **节点-边索引（node_edge_index）**
   - 用于快速查询节点的所有关联边
   - 键：节点ID的字节序列
   - 值：边键列表的JSON序列化
   - 支持双向边查询（In/Out/Both）

2. **边类型索引（edge_type_index）**
   - 用于按边类型快速扫描
   - 键：边类型字符串
   - 值：边键列表的JSON序列化

**键设计：**

- **顶点键**：顶点ID的字节表示
- **边键**：`"{src:?}_{dst:?}_{edge_type}"` 格式的字符串，确保唯一性

**序列化：**

使用`serde_json`进行顶点和边的序列化/反序列化。

**ID生成：**

使用系统时间戳（纳秒级）生成唯一ID，生产环境应使用更健壮的方案。

### 3. 迭代器系统

#### Iterator Trait

零成本抽象的迭代器核心trait，使用泛型实现编译时多态，消除动态分发开销。

**迭代器类型：**

```rust
pub enum IteratorKind {
    Default,      // 默认常量迭代器
    Sequential,   // 顺序迭代器（DataSet行级）
    GetNeighbors, // 邻居迭代器（图遍历）
    Prop,         // 属性迭代器
}
```

**核心方法分类：**

1. **基本迭代**：`next()`, `valid()`, `reset()`
2. **删除操作**：`erase()`, `unstable_erase()`, `clear()`, `erase_range()`
3. **范围操作**：`select()`, `sample()`
4. **行访问**：`row()`, `move_row()`, `size()`, `is_empty()`
5. **列访问**：`get_column()`, `get_column_by_index()`, `get_column_index()`, `get_col_names()`
6. **图特定**：`get_tag_prop()`, `get_edge_prop()`, `get_vertex()`, `get_edge()`
7. **复制**：`copy()` - 返回具体类型，零成本抽象

#### DefaultIter

用于包装单个Value，使其可以通过迭代器接口访问。

**特点：**
- 始终有一个有效行，代表该值本身
- `next()`后变为无效
- 支持列访问（返回值本身）

#### SequentialIter

用于遍历DataSet中的每一行，支持行级操作。

**特点：**
- 支持行遍历、删除、修改
- 支持范围操作（select、erase_range、sample）
- 支持列访问（按名称或索引）

#### GetNeighborsIter

用于遍历邻居查询的复杂结果结构，支持四层嵌套遍历。

**四层遍历结构：**
1. **数据集列表层**：`ds_indices_`（可能多个分片返回多个数据集）
2. **数据行层**：`current_row_`（每个顶点一行）
3. **边列层**：`col_idx_`（每个边类型一列）
4. **边列表层**：`edge_idx_`（每个邻接边一条记录）

**索引结构：**

```rust
struct DataSetIndex {
    ds: Arc<Mutex<DataSet>>,
    col_indices: HashMap<String, usize>,              // 列名到索引的映射
    tag_edge_name_indices: HashMap<usize, String>,    // 列索引到标签/边名的映射
    tag_props_map: HashMap<String, PropIndex>,        // 标签属性映射
    edge_props_map: HashMap<String, PropIndex>,       // 边属性映射
    col_lower_bound: i64,                             // 边列下界
    col_upper_bound: i64,                             // 边列上界
}
```

**列名格式：**
- `_vid`：顶点ID
- `_stats`：统计信息
- `_tag:tag_name:prop1:prop2:...`：标签属性列
- `_edge:+edge_type:prop1:prop2:...`：出边属性列
- `_edge:-edge_type:prop1:prop2:...`：入边属性列

**遍历算法：**

使用嵌套循环遍历四层结构，支持：
- `next()`：移动到下一个边
- `erase()`：有序删除当前边
- `unstable_erase()`：快速删除（交换删除）
- `select()`：选择指定范围的边
- `sample()`：采样指定数量的边

#### PropIter

用于遍历属性查询结果，支持顶点和边的属性访问。

**特点：**
- 类似SequentialIter，但针对属性数据优化
- 支持属性索引：`{tag_name: {prop_name: col_index}}`
- 支持通配符属性查询（tag="*"）

### 4. 测试支持

#### MockStorage

提供统一的Mock存储引擎实现，用于单元测试。

**特点：**
- 实现所有StorageEngine trait方法
- 返回空结果或默认值
- 避免在各个测试模块中重复实现

## 数据流

### 顶点插入流程

```
1. 生成唯一ID（时间戳）
2. 创建带ID的顶点对象
3. 序列化为JSON
4. 插入nodes_tree
5. 刷新到磁盘
6. 返回生成的ID
```

### 边插入流程

```
1. 生成边键（src_dst_edge_type）
2. 序列化边对象为JSON
3. 插入edges_tree
4. 更新节点-边索引（src和dst）
5. 更新边类型索引
6. 刷新到磁盘
```

### 邻居查询流程

```
1. 查询节点-边索引获取所有边键
2. 遍历边键获取边对象
3. 根据方向过滤（In/Out/Both）
4. 返回符合条件的边列表
```

### 邻居查询迭代器遍历流程

```
1. 处理输入数据集列表
2. 为每个数据集构建索引
3. 移动到第一条有效边
4. 四层遍历：
   - 数据集层 -> 行层 -> 边列层 -> 边列表层
5. 支持删除、选择、采样等操作
```

## 性能特点

### 优点

1. **零成本抽象**：使用泛型trait实现编译时多态，无动态分发开销
2. **高效索引**：节点-边索引和边类型索引加速查询
3. **嵌入式存储**：sled提供ACID事务和持久化
4. **灵活迭代**：多种迭代器类型支持不同查询场景

### 待优化项

1. **ID生成**：当前使用时间戳，生产环境应使用更健壮的方案
2. **事务支持**：当前事务方法为TODO，需要实现真正的事务
3. **批量操作**：缺少批量插入/更新接口
4. **并发控制**：需要考虑读写锁和并发访问
5. **缓存策略**：可以添加查询缓存提升性能

## 错误处理

使用`StorageError`枚举统一处理存储相关错误：

```rust
pub enum StorageError {
    NodeNotFound(Value),
    EdgeNotFound(Value),
    DbError(String),
    SerializationError(String),
    // ...
}
```

## 测试策略

1. **单元测试**：每个迭代器类型都有完整的单元测试
2. **Mock存储**：使用MockStorage进行隔离测试
3. **集成测试**：在`data/tests`目录下进行存储集成测试
4. **测试数据**：使用临时数据库路径避免冲突

## 与原NebulaGraph对比

### 相似点

- 迭代器设计参考了NebulaGraph的Iterator.h/cpp
- 支持多种迭代器类型（Default、Sequential、GetNeighbors、Prop）
- 列名格式兼容（_tag、_edge等）

### 差异点

- 使用sled替代RocksDB（更轻量）
- 使用Rust替代C++（内存安全）
- 简化了分布式功能（单节点架构）
- 使用泛型trait替代虚函数（零成本抽象）

## 扩展建议

1. **添加更多索引**：如属性索引、复合索引
2. **实现批量操作**：批量插入、批量更新
3. **完善事务支持**：实现真正的ACID事务
4. **添加缓存层**：查询结果缓存、热点数据缓存
5. **支持更多查询**：最短路径、连通分量等图算法
6. **优化序列化**：考虑使用更高效的序列化格式（如MessagePack）
7. **添加监控**：性能指标、查询统计

## 总结

GraphDB的存储模块采用了清晰的分层架构：

1. **抽象层**：StorageEngine trait定义统一接口
2. **实现层**：NativeStorage基于sled提供高性能存储
3. **迭代器层**：多种迭代器类型支持不同查询场景
4. **测试层**：MockStorage和完整测试套件保证质量

该设计兼顾了性能、可维护性和扩展性，为图数据库的核心功能提供了坚实的基础。
