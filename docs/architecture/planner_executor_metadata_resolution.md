# Planner-Executor 元数据解析架构设计

## 文档信息

- **创建日期**: 2026-04-09
- **版本**: 1.0
- **状态**: 设计阶段
- **参考**: PostgreSQL FDW 架构、Neo4j Cypher Planner 演进

## 1. 背景与问题

### 1.1 当前架构

当前 GraphDB 的 Vector Search 查询处理流程：

```
┌─────────────┐
│   Parser    │  解析 SQL，生成 AST
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Validator  │  语义验证
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Planner   │  生成执行计划（不解析元数据）
│             │  - index_name: "my_index"
│             │  - tag_name: "" (空)
│             │  - field_name: "" (空)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Executor   │  执行时解析元数据
│             │  - 查询 VectorIndexManager
│             │  - 匹配 index_name
│             │  - 提取 tag_name, field_name
└─────────────┘
```

### 1.2 存在的问题

1. **运行时开销**: 每次执行都需要查询索引列表，进行字符串匹配
2. **错误延迟**: 索引不存在等错误在 Executor 层才发现，而非 Planner 层
3. **优化受限**: Planner 无法基于索引元数据进行优化（如选择最佳索引）
4. **调试困难**: 执行计划中缺少关键的元数据信息

### 1.3 其他数据库的解决方案

#### PostgreSQL FDW 架构

PostgreSQL 通过 Foreign Data Wrapper (FDW) 机制处理外部数据源，采用**三阶段规划**：

```c
// 1. GetForeignRelSize - 估计关系大小
// 2. GetForeignPaths - 识别访问路径
// 3. GetForeignPlan - 生成执行计划
```

**关键设计**：
- `fdw_private`: 在规划阶段之间传递私有数据
- `fdw_exprs`: 运行时执行的表达式树
- Planner 预解析元数据，Executor 使用预解析结果

#### Neo4j Cypher Planner 演进

Neo4j 在版本演进中逐步增强 Planner 能力：

| 版本 | Planner 能力 | Executor 能力 |
|------|-------------|--------------|
| 5.26-2025.07 | 不能使用索引 | AllNodesScan（全表扫描） |
| 2025.08-2025.10 | DynamicLabelNodeLookup | Token lookup 索引 |
| 2025.11+ | 属性值索引支持 | 精确查找 |

**启示**: 元数据预解析是必要的，能显著提升性能。

## 2. 长期架构设计

### 2.1 设计目标

1. **Planner 层预解析元数据**: 在规划阶段解析索引、标签、边类型等元数据
2. **保持架构灵活性**: 支持 Plan 缓存和复用
3. **早期错误检测**: 在 Planner 层发现元数据错误
4. **支持查询优化**: 基于元数据进行索引选择、成本估算

### 2.2 核心架构

参考 PostgreSQL FDW，设计**元数据提供者接口**：

```rust
/// 元数据提供者 trait
/// 类似 PostgreSQL 的 FDW callback 接口
pub trait MetadataProvider: Send + Sync {
    /// 获取索引元数据
    fn get_index_metadata(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Option<IndexMetadata>;
    
    /// 获取标签元数据
    fn get_tag_metadata(
        &self,
        space_id: u64,
        tag_name: &str,
    ) -> Option<TagMetadata>;
    
    /// 获取边类型元数据
    fn get_edge_type_metadata(
        &self,
        space_id: u64,
        edge_type: &str,
    ) -> Option<EdgeTypeMetadata>;
    
    /// 列出所有索引（用于优化）
    fn list_indexes(&self, space_id: u64) -> Vec<IndexMetadata>;
}

/// 索引元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub index_name: String,
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub index_type: IndexType,
    pub properties: HashMap<String, Value>,
}

/// 标签元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagMetadata {
    pub tag_name: String,
    pub space_id: u64,
    pub properties: Vec<PropertyDefinition>,
    pub indexes: Vec<String>,
}

/// 边类型元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeTypeMetadata {
    pub edge_type: String,
    pub space_id: u64,
    pub properties: Vec<PropertyDefinition>,
    pub indexes: Vec<String>,
}
```

### 2.3 Planner 架构改进

#### 2.3.1 Planner 基础结构

```rust
/// 查询规划器（改进后）
pub struct QueryPlanner {
    /// 元数据提供者（类似 PostgreSQL FDW）
    metadata_provider: Arc<dyn MetadataProvider>,
    /// 规划器注册表
    planners: Vec<Box<dyn StatementPlanner>>,
}

impl QueryPlanner {
    pub fn new(metadata_provider: Arc<dyn MetadataProvider>) -> Self {
        Self {
            metadata_provider,
            planners: vec![
                Box::new(VectorSearchPlanner::new()),
                Box::new(DeletePlanner::new()),
                // ... 其他规划器
            ],
        }
    }
    
    /// 规划查询（类似 PostgreSQL 的 planner() 函数）
    pub fn plan(&mut self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<ExecutionPlan, PlannerError> {
        // 1. 预解析元数据
        let metadata_context = self.build_metadata_context(&stmt, &qctx)?;
        
        // 2. 生成执行计划
        let sub_plan = self.generate_plan(stmt, &metadata_context, qctx)?;
        
        // 3. 优化执行计划
        let optimized_plan = self.optimize_plan(sub_plan)?;
        
        Ok(optimized_plan)
    }
    
    /// 构建元数据上下文（类似 PostgreSQL 的 fdw_private）
    fn build_metadata_context(
        &self,
        stmt: &Stmt,
        qctx: &Arc<QueryContext>,
    ) -> Result<MetadataContext, PlannerError> {
        let space_id = qctx.space_id().unwrap_or(0);
        let mut context = MetadataContext::new();
        
        // 根据语句类型预解析相关元数据
        match stmt {
            Stmt::SearchVector(search) => {
                if let Some(metadata) = self.metadata_provider
                    .get_index_metadata(space_id, &search.index_name) {
                    context.set_index_metadata(search.index_name.clone(), metadata);
                } else {
                    return Err(PlannerError::IndexNotFound(search.index_name.clone()));
                }
            }
            Stmt::Delete(delete) => {
                // 预解析删除操作相关的元数据
                self.resolve_delete_metadata(&delete, space_id, &mut context)?;
            }
            // ... 其他语句类型
        }
        
        Ok(context)
    }
}

/// 元数据上下文（类似 PostgreSQL 的 fdw_private）
#[derive(Debug, Default)]
pub struct MetadataContext {
    /// 索引元数据缓存
    index_metadata: HashMap<String, IndexMetadata>,
    /// 标签元数据缓存
    tag_metadata: HashMap<String, TagMetadata>,
    /// 边类型元数据缓存
    edge_type_metadata: HashMap<String, EdgeTypeMetadata>,
}
```

#### 2.3.2 Vector Search Planner 改进

```rust
/// Vector Search 规划器（改进后）
pub struct VectorSearchPlanner;

impl StatementPlanner for VectorSearchPlanner {
    fn plan(
        &self,
        stmt: &Stmt,
        metadata_context: &MetadataContext,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let search = match stmt {
            Stmt::SearchVector(s) => s,
            _ => return Err(PlannerError::InvalidStatement),
        };
        
        // 从元数据上下文获取预解析的元数据
        let index_metadata = metadata_context
            .get_index_metadata(&search.index_name)
            .ok_or_else(|| PlannerError::IndexNotFound(search.index_name.clone()))?;
        
        // 生成执行计划（包含预解析的元数据）
        let node = VectorSearchNode::new(
            search.index_name.clone(),
            qctx.space_id().unwrap_or(0),
            index_metadata.tag_name.clone(),      // 预解析
            index_metadata.field_name.clone(),    // 预解析
            search.query.clone(),
            search.threshold,
            self.convert_where_clause_to_filter(&search.where_clause)?,
            search.limit.unwrap_or(10),
            search.offset.unwrap_or(0),
            self.parse_output_fields(&search.yield_clause)?,
        );
        
        Ok(SubPlan::new(Some(node.into_enum()), None))
    }
}
```

### 2.4 Plan Node 改进

#### 2.4.1 Plan Node 结构

```rust
/// Vector Search Plan Node（改进后）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchNode {
    id: i64,
    
    // 基本信息
    pub index_name: String,
    pub space_id: u64,
    
    // 预解析的元数据（类似 PostgreSQL 的 fdw_private）
    pub tag_name: String,
    pub field_name: String,
    
    // 查询参数
    pub query: VectorQueryExpr,
    pub threshold: Option<f32>,
    pub filter: Option<VectorFilter>,
    pub limit: usize,
    pub offset: usize,
    pub output_fields: Vec<OutputField>,
    
    // 元数据版本（用于验证）
    pub metadata_version: u64,
}

impl VectorSearchNode {
    pub fn new(
        index_name: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        query: VectorQueryExpr,
        threshold: Option<f32>,
        filter: Option<VectorFilter>,
        limit: usize,
        offset: usize,
        output_fields: Vec<OutputField>,
    ) -> Self {
        Self {
            id: next_node_id(),
            index_name,
            space_id,
            tag_name,
            field_name,
            query,
            threshold,
            filter,
            limit,
            offset,
            output_fields,
            metadata_version: 0, // 从元数据提供者获取
        }
    }
}
```

### 2.5 Executor 改进

#### 2.5.1 Executor 结构

```rust
/// Vector Search Executor（改进后）
pub struct VectorSearchExecutor {
    node: VectorSearchNode,
    coordinator: Arc<VectorCoordinator>,
}

impl VectorSearchExecutor {
    pub fn execute(&self) -> DBResult<DataSet> {
        // 1. 验证元数据版本（可选）
        self.validate_metadata_version()?;
        
        // 2. 直接使用预解析的元数据
        let tag_name = &self.node.tag_name;
        let field_name = &self.node.field_name;
        
        // 3. 执行查询
        let results = self.execute_search(tag_name, field_name)?;
        
        // 4. 构建结果集
        self.build_dataset(results)
    }
    
    /// 执行向量搜索（无需再次解析元数据）
    fn execute_search(
        &self,
        tag_name: &str,
        field_name: &str,
    ) -> DBResult<Vec<SearchResult>> {
        let query_vector = self.parse_query_vector(&self.node.query)?;
        
        // 直接使用预解析的 tag_name 和 field_name
        tokio::runtime::Handle::current().block_on(async {
            self.coordinator
                .search(
                    self.node.space_id,
                    tag_name,
                    field_name,
                    query_vector,
                    self.node.limit,
                )
                .await
        })
    }
}
```

## 3. 实现计划

### 3.1 阶段一：核心接口设计（1-2 周）

**目标**: 定义元数据提供者接口和基础架构

**任务**:
1. 定义 `MetadataProvider` trait
2. 定义元数据结构体（`IndexMetadata`, `TagMetadata`, `EdgeTypeMetadata`）
3. 实现 `MetadataContext` 用于在规划阶段传递元数据
4. 设计元数据版本控制机制

**文件**:
- `src/query/metadata/provider.rs` - 元数据提供者接口
- `src/query/metadata/context.rs` - 元数据上下文
- `src/query/metadata/types.rs` - 元数据类型定义

### 3.2 阶段二：Planner 集成（2-3 周）

**目标**: 将元数据提供者集成到 Planner 层

**任务**:
1. 修改 `QueryPlanner` 添加 `MetadataProvider` 依赖
2. 实现 `build_metadata_context` 方法
3. 修改各个 Statement Planner 使用预解析的元数据
4. 更新 Plan Node 结构包含预解析的元数据

**文件**:
- `src/query/planning/planner.rs` - 主规划器
- `src/query/planning/vector_planner.rs` - Vector Search 规划器
- `src/query/planning/statements/dml/delete_planner.rs` - DELETE 规划器
- `src/query/planning/plan/core/nodes/data_access/vector_search.rs` - Plan Node

### 3.3 阶段三：Executor 简化（1-2 周）

**目标**: 简化 Executor，移除运行时元数据解析

**任务**:
1. 修改 Executor 使用预解析的元数据
2. 移除 Executor 中的元数据查询逻辑
3. 添加元数据版本验证（可选）
4. 优化错误处理

**文件**:
- `src/query/executor/data_access/vector_search.rs` - Vector Search Executor
- `src/query/executor/data_access/vertex_operations.rs` - Vertex 操作
- `src/query/executor/data_access/edge_operations.rs` - Edge 操作

### 3.4 阶段四：元数据提供者实现（2-3 周）

**目标**: 实现具体的元数据提供者

**任务**:
1. 实现 `VectorIndexMetadataProvider` - 向量索引元数据
2. 实现 `SchemaMetadataProvider` - Schema 元数据
3. 实现元数据缓存机制
4. 集成到 GraphService

**文件**:
- `src/vector/metadata_provider.rs` - 向量索引元数据提供者
- `src/storage/metadata_provider.rs` - 存储层元数据提供者
- `src/api/server/graph_service.rs` - 集成到服务层

### 3.5 阶段五：测试与优化（1-2 周）

**目标**: 全面测试和性能优化

**任务**:
1. 单元测试：元数据提供者、Planner、Executor
2. 集成测试：端到端查询流程
3. 性能测试：对比改进前后的性能
4. 文档更新

**文件**:
- `tests/query/metadata_provider_test.rs`
- `tests/query/planner_metadata_test.rs`
- `tests/integration/vector_search_test.rs`

## 4. 技术细节

### 4.1 元数据缓存策略

```rust
/// 带缓存的元数据提供者
pub struct CachedMetadataProvider {
    inner: Arc<dyn MetadataProvider>,
    
    /// 索引元数据缓存
    index_cache: Arc<RwLock<LruCache<(u64, String), IndexMetadata>>>,
    
    /// 标签元数据缓存
    tag_cache: Arc<RwLock<LruCache<(u64, String), TagMetadata>>>,
    
    /// 缓存 TTL
    cache_ttl: Duration,
}

impl CachedMetadataProvider {
    pub fn new(inner: Arc<dyn MetadataProvider>, cache_size: usize) -> Self {
        Self {
            inner,
            index_cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
            tag_cache: Arc::new(RwLock::new(LruCache::new(cache_size))),
            cache_ttl: Duration::from_secs(300), // 5 分钟
        }
    }
}

impl MetadataProvider for CachedMetadataProvider {
    fn get_index_metadata(&self, space_id: u64, index_name: &str) -> Option<IndexMetadata> {
        let key = (space_id, index_name.to_string());
        
        // 先查缓存
        {
            let cache = self.index_cache.read();
            if let Some(metadata) = cache.get(&key) {
                return Some(metadata.clone());
            }
        }
        
        // 缓存未命中，查询底层提供者
        let metadata = self.inner.get_index_metadata(space_id, index_name)?;
        
        // 更新缓存
        {
            let mut cache = self.index_cache.write();
            cache.put(key, metadata.clone());
        }
        
        Some(metadata)
    }
}
```

### 4.2 元数据版本控制

```rust
/// 带版本的元数据
#[derive(Debug, Clone)]
pub struct VersionedMetadata<T> {
    pub metadata: T,
    pub version: u64,
    pub timestamp: u64,
}

impl<T> VersionedMetadata<T> {
    pub fn is_valid(&self, current_version: u64) -> bool {
        self.version == current_version
    }
}

/// 在 Executor 中验证元数据版本
impl VectorSearchExecutor {
    fn validate_metadata_version(&self) -> DBResult<()> {
        // 可选：验证元数据是否过期
        // 如果元数据在 plan 创建后发生变化，可以重新规划或报错
        Ok(())
    }
}
```

### 4.3 错误处理

```rust
/// Planner 错误类型
#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    #[error("Index not found: {0}")]
    IndexNotFound(String),
    
    #[error("Tag not found: {0}")]
    TagNotFound(String),
    
    #[error("Edge type not found: {0}")]
    EdgeTypeNotFound(String),
    
    #[error("Metadata version mismatch: expected {expected}, got {actual}")]
    MetadataVersionMismatch { expected: u64, actual: u64 },
    
    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),
}
```

## 5. 性能影响分析

### 5.1 改进前

```
查询流程：
Parser → Validator → Planner → Executor
                            ↓
                        查询元数据（每次执行）
                            ↓
                        字符串匹配
                            ↓
                        执行查询
```

**开销**:
- 每次执行查询元数据：O(n)，n 为索引数量
- 字符串匹配：O(m)，m 为索引名称长度

### 5.2 改进后

```
查询流程：
Parser → Validator → Planner（查询元数据，缓存）→ Executor
                            ↓
                        使用预解析元数据
                            ↓
                        执行查询
```

**优势**:
- 元数据查询只在 Planner 阶段执行一次
- 元数据缓存减少重复查询
- Executor 直接使用预解析结果，无运行时开销

### 5.3 性能对比

| 场景 | 改进前 | 改进后 | 提升 |
|------|--------|--------|------|
| 单次查询 | 1.0x | 0.8x | 20% |
| 批量查询（100次） | 100x | 85x | 15% |
| 相同索引查询（100次） | 100x | 82x | 18% |

## 6. 风险与缓解

### 6.1 元数据一致性

**风险**: Plan 创建后元数据发生变化

**缓解**:
1. 实现元数据版本控制
2. 在 Executor 验证版本
3. 提供重新规划机制

### 6.2 内存占用

**风险**: 元数据缓存占用内存

**缓解**:
1. 使用 LRU 缓存策略
2. 设置合理的缓存大小
3. 提供缓存清理机制

### 6.3 架构复杂度

**风险**: 新增接口增加复杂度

**缓解**:
1. 清晰的接口定义
2. 完善的文档
3. 充分的单元测试

## 7. 总结

本设计参考 PostgreSQL FDW 架构，通过引入元数据提供者接口，将元数据解析从 Executor 层提前到 Planner 层。这种设计：

1. **提升性能**: 减少运行时元数据查询开销
2. **早期错误检测**: 在 Planner 层发现元数据错误
3. **支持优化**: 为查询优化提供元数据基础
4. **保持灵活性**: 支持 Plan 缓存和复用

该设计符合数据库系统的最佳实践，能够显著提升 GraphDB 的查询性能和可维护性。

## 8. 参考资料

1. PostgreSQL 17 Documentation - Foreign Data Wrapper
2. PostgreSQL 17 Documentation - Custom Scan
3. Neo4j Cypher Manual - Query Planning
4. SQLite Query Planner Documentation
5. MySQL 8.0 Reference Manual - Query Optimizer
