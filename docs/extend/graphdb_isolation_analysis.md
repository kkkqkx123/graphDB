# GraphDB 向量检索与全文检索隔离机制分析

## 1. 当前实现概述

### 1.1 Space 概念

GraphDB 使用 **Space** 作为核心隔离单元，类似于 NebulaGraph 的图空间概念：

```rust
// src/core/types/space.rs
pub struct SpaceInfo {
    pub space_id: u64,        // 全局唯一 Space ID
    pub space_name: String,   // Space 名称
    pub vid_type: DataType,
    pub tags: Vec<TagInfo>,
    pub edge_types: Vec<EdgeTypeInfo>,
    pub version: MetadataVersion,
    pub comment: Option<String>,
}
```

### 1.2 向量检索隔离机制

#### 索引位置标识
```rust
// src/sync/vector_sync.rs
pub struct VectorIndexLocation {
    pub space_id: u64,      // Space 级别隔离
    pub tag_name: String,   // Tag 级别隔离
    pub field_name: String, // 字段级别隔离
}

impl VectorIndexLocation {
    pub fn to_collection_name(&self) -> String {
        format!("space_{}_{}_{}", self.space_id, self.tag_name, self.field_name)
    }
}
```

#### 集合命名规则
- **格式**：`space_{space_id}_{tag_name}_{field_name}`
- **示例**：`space_1_product_description`
- **特点**：
  - 使用 `space_id` 而非 `space_name`，避免重命名问题
  - 三层隔离：Space → Tag → Field

#### VectorClient 实现
```rust
// src/sync/external_index/vector_client.rs
pub struct VectorClient {
    space_id: u64,
    tag_name: String,
    field_name: String,
    vector_manager: Arc<vector_client::VectorManager>,
    // ...
}

fn collection_name(&self) -> String {
    format!("{}_{}_{}", self.space_id, self.tag_name, self.field_name)
}
```

### 1.3 全文检索隔离机制

#### 索引键设计
```rust
// src/search/metadata.rs
pub struct IndexKey {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
}

impl IndexKey {
    pub fn to_index_id(&self) -> String {
        format!("{}_{}_{}", self.space_id, self.tag_name, self.field_name)
    }
}
```

#### 索引元数据
```rust
pub struct IndexMetadata {
    pub index_id: String,
    pub index_name: String,
    pub space_id: u64,        // Space 隔离标识
    pub tag_name: String,
    pub field_name: String,
    pub engine_type: EngineType,
    pub storage_path: String,
    // ...
}
```

#### FulltextIndexManager 实现
```rust
// src/search/manager.rs
pub struct FulltextIndexManager {
    engines: DashMap<IndexKey, Arc<dyn SearchEngine>>,
    metadata: DashMap<IndexKey, IndexMetadata>,
    base_path: PathBuf,
    // ...
}

pub fn get_space_indexes(&self, space_id: u64) -> Vec<IndexMetadata> {
    self.metadata
        .iter()
        .filter(|entry| entry.value().space_id == space_id)
        .map(|entry| entry.value().clone())
        .collect()
}
```

---

## 2. 与主流数据库对比分析

### 2.1 隔离层级对比

| 数据库 | 隔离层级 | GraphDB 对应实现 | 对比说明 |
|-------|---------|-----------------|---------|
| PostgreSQL | Schema | Space | 概念相似，但 Space 是 GraphDB 特有概念 |
| Neo4j | Database | Space | Neo4j 数据库更重量级，Space 更轻量 |
| GraphDB | Space → Tag → Field | - | 三层隔离，粒度更细 |

### 2.2 索引命名对比

| 数据库 | 命名格式 | 优势 | 劣势 |
|-------|---------|-----|------|
| PostgreSQL | `{schema}.{index_name}` | 符合 SQL 标准 | Schema 重命名影响索引 |
| Neo4j | `{database}.{index_name}` | 数据库级强隔离 | 数据库创建开销大 |
| GraphDB | `space_{id}_{tag}_{field}` | Space 重命名无影响 | 名称较长 |

### 2.3 存储隔离对比

| 数据库 | 存储隔离方式 | GraphDB 实现 |
|-------|-------------|-------------|
| PostgreSQL | Tablespace + 文件系统 | 通过 `base_path` + `index_id` 分离 |
| Neo4j | 数据库独立目录 | 单实例设计，共享存储目录 |
| GraphDB | `base_path/{index_id}/` | 每个索引独立子目录 |

---

## 3. 最佳实践符合度分析

### 3.1 符合最佳实践的设计

#### ✅ 使用 ID 而非名称作为隔离标识
```rust
// 优秀实践：使用 space_id 而非 space_name
format!("space_{}_{}_{}", self.space_id, self.tag_name, self.field_name)
```
**优点**：
- Space 重命名不影响索引
- 避免名称冲突问题
- 符合数据库设计原则

#### ✅ 三层隔离粒度
- **Space 级**：不同图空间完全隔离
- **Tag 级**：同一空间内不同标签隔离
- **Field 级**：同一标签内不同字段隔离

**优点**：
- 细粒度控制
- 避免命名冲突
- 支持灵活的索引策略

#### ✅ 统一索引键设计
```rust
pub struct IndexKey {
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
}
```
**优点**：
- 类型安全
- 可作为 HashMap 键
- 清晰的隔离边界

#### ✅ DashMap 并发管理
```rust
engines: DashMap<IndexKey, Arc<dyn SearchEngine>>,
metadata: DashMap<IndexKey, IndexMetadata>,
```
**优点**：
- 高性能并发访问
- 细粒度锁控制
- 适合多线程环境

### 3.2 可改进的方面

#### ⚠️ 物理存储隔离
**当前实现**：
```rust
base_path: PathBuf,  // 全局统一路径
```

**建议改进**：
```rust
// 支持 Space 级存储路径配置
pub struct SpaceConfig {
    pub storage_path: Option<PathBuf>,  // Space 自定义存储路径
    pub isolation_level: IsolationLevel, // 隔离级别
}

pub enum IsolationLevel {
    Shared,      // 共享存储（默认）
    Directory,   // 独立子目录
    Device,      // 独立存储设备
}
```

**理由**：
- 支持大 Space 独立磁盘部署
- 便于数据迁移和备份
- 符合 PostgreSQL Tablespace 理念

#### ⚠️ 索引命名一致性
**当前问题**：
- 向量索引：`space_{space_id}_{tag_name}_{field_name}`
- 全文索引：`{space_id}_{tag_name}_{field_name}`

**建议统一**：
```rust
// 统一使用 space_ 前缀
const VECTOR_INDEX_PREFIX: &str = "space_vec";
const FULLTEXT_INDEX_PREFIX: &str = "space_ft";

// 向量：space_vec_{space_id}_{tag}_{field}
// 全文：space_ft_{space_id}_{tag}_{field}
```

#### ⚠️ 缺少命名空间验证
**当前风险**：
- 未验证 Space 是否存在即创建索引
- 删除 Space 时可能残留索引数据

**建议改进**：
```rust
pub async fn create_index(&self, space_id: u64, ...) -> Result<...> {
    // 1. 验证 Space 存在
    if !self.space_manager.exists(space_id).await? {
        return Err(SearchError::SpaceNotFound(space_id));
    }
    
    // 2. 验证 Tag 和 Field 存在
    // 3. 创建索引
    // 4. 建立 Space-索引关联关系
}
```

#### ⚠️ 跨 Space 查询限制
**当前限制**：
- 不支持跨 Space 的向量/全文检索
- 没有类似 Neo4j Fabric 的联邦查询能力

**评估**：
- 对于单节点图数据库，这是合理的设计选择
- 如需支持，可考虑在查询层实现跨 Space 聚合

### 3.3 安全隔离评估

| 安全维度 | 当前实现 | 评估 | 建议 |
|---------|---------|-----|------|
| 数据隔离 | Space ID 前缀隔离 | ✅ 良好 | 保持 |
| 访问控制 | 无显式权限检查 | ⚠️ 需改进 | 增加 Space 级权限 |
| 资源隔离 | 共享线程池 | ⚠️ 可优化 | 考虑 Space 级资源限制 |
| 元数据隔离 | DashMap 隔离 | ✅ 良好 | 保持 |

---

## 4. 结论与建议

### 4.1 总体评估

GraphDB 的向量检索和全文检索隔离机制 **基本符合最佳实践**，主要体现在：

1. **合理的隔离粒度**：Space → Tag → Field 三层隔离
2. **稳定的标识设计**：使用 ID 而非名称，避免重命名问题
3. **清晰的边界定义**：IndexKey 结构明确隔离边界
4. **高效的并发管理**：DashMap 提供良好的并发性能

### 4.2 优化建议优先级

| 优先级 | 建议项 | 影响范围 | 实施难度 |
|-------|-------|---------|---------|
| P1 | 统一索引命名规范 | 兼容性 | 低 |
| P2 | 增加 Space 存在性验证 | 数据一致性 | 低 |
| P3 | 支持 Space 级存储路径配置 | 部署灵活性 | 中 |
| P4 | 增加 Space 级资源隔离 | 性能稳定性 | 中 |
| P5 | 增加显式权限检查 | 安全性 | 高 |

### 4.3 与 PostgreSQL/Neo4j 的差异说明

GraphDB 的设计差异是合理的：

1. **Space vs Schema/Database**：
   - Space 是 GraphDB 特有的图空间概念
   - 比 PostgreSQL Schema 更重量级（包含独立的标签/边类型定义）
   - 比 Neo4j Database 更轻量级（共享存储和计算资源）

2. **三层隔离的必要性**：
   - 图数据模型需要标签（Tag）级别的隔离
   - 字段级隔离支持灵活的索引策略
   - 这是图数据库相比关系数据库的特殊需求

3. **单实例设计**：
   - 符合项目定位（本地单节点部署）
   - 简化了隔离机制的实现
   - 避免了分布式系统的复杂性

---

## 5. 参考资料

- [src/core/types/space.rs](../../src/core/types/space.rs)
- [src/sync/vector_sync.rs](../../src/sync/vector_sync.rs)
- [src/sync/external_index/vector_client.rs](../../src/sync/external_index/vector_client.rs)
- [src/search/manager.rs](../../src/search/manager.rs)
- [src/search/metadata.rs](../../src/search/metadata.rs)
- [src/storage/extend/fulltext_storage.rs](../../src/storage/extend/fulltext_storage.rs)
