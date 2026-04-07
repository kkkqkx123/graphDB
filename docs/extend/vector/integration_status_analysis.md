# 向量检索查询流程集成状态分析

> 分析日期：2026-04-07  
> 分析范围：查询引擎各层级（Parser → Validator → Planner → PlanNode → Executor）  
> 结论：**向量检索功能尚未集成到查询流程中**

---

## 执行摘要

当前项目的向量检索功能**仅完成了底层基础设施**，但**查询引擎集成完全缺失**。这意味着：

- ✅ 底层向量索引管理能力已就绪
- ✅ 数据同步机制已实现
- ✅ Qdrant 后端集成已完成
- ❌ **用户无法通过查询语言使用向量检索**
- ❌ **查询引擎各层级均未实现向量检索支持**

---

## 一、已完成部分

### 1.1 基础设施层 ✅

#### VectorIndexManager
**位置**: [`src/vector/manager.rs`](file:///d:/项目/database/graphDB/src/vector/manager.rs)

```rust
pub struct VectorIndexManager {
    config: VectorConfig,
    engine: Arc<dyn VectorEngine>,
    metadata: DashMap<IndexKey, VectorIndexMetadata>,
}
```

**功能**:
- ✅ 创建/删除向量索引
- ✅ 索引元数据管理
- ✅ 向量插入/更新/删除
- ✅ 向量搜索执行

#### VectorCoordinator
**位置**: [`src/vector/coordinator.rs`](file:///d:/项目/database/graphDB/src/vector/coordinator.rs)

```rust
pub struct VectorCoordinator {
    manager: Arc<VectorIndexManager>,
}
```

**功能**:
- ✅ 协调图数据与向量索引的变更
- ✅ 支持顶点插入、更新、删除时的向量同步
- ✅ 批量处理和异步同步

#### VectorConfig
**位置**: [`src/vector/config.rs`](file:///d:/项目/database/graphDB/src/vector/config.rs)

```rust
pub struct VectorIndexConfig {
    pub vector_size: usize,
    pub distance: VectorDistance,
    pub hnsw_m: Option<usize>,
    pub hnsw_ef_construct: Option<usize>,
}
```

### 1.2 数据同步机制 ✅

**复用全文检索的 Sync 模块**:
- ✅ `SyncManager` - 异步同步管理
- ✅ `TaskBuffer` - 批量任务缓冲
- ✅ `RecoveryManager` - 失败任务恢复
- ✅ 支持 `VectorSyncTask`

### 1.3 后端集成 ✅

**Qdrant 适配器**:
- ✅ `QdrantEngine` 实现 `VectorEngine` trait
- ✅ 支持向量搜索、插入、删除
- ✅ 支持过滤器

---

## 二、缺失部分（查询引擎集成）

### 2.1 AST 层缺失 ❌

**文件**: `src/query/parser/ast/stmt.rs`

**问题**: `Stmt` 枚举中没有向量检索相关的语句类型

**当前状态**:
```rust
pub enum Stmt {
    // ... 其他语句 ...
    
    // ✅ 全文检索已支持
    CreateFulltextIndex(CreateFulltextIndex),
    DropFulltextIndex(DropFulltextIndex),
    Search(SearchStatement),
    LookupFulltext(LookupFulltext),
    MatchFulltext(MatchFulltext),
    
    // ❌ 缺失：向量检索语句
    // 需要添加:
    // CreateVectorIndex(CreateVectorIndex),
    // DropVectorIndex(DropVectorIndex),
    // SearchVector(SearchVectorStatement),
    // LookupVector(LookupVector),
    // MatchVector(MatchVector),
}
```

**影响**: 无法解析 `SEARCH VECTOR`、`CREATE VECTOR INDEX` 等语法

---

### 2.2 解析器层缺失 ❌

**文件**: `src/query/parser/parser.rs`

**问题**: 没有向量检索语句的解析逻辑

**需要实现**:
```rust
impl Parser {
    // ❌ 缺失的方法
    // pub fn parse_search_vector(&mut self) -> Result<SearchVectorStatement, ParseError>
    // pub fn parse_create_vector_index(&mut self) -> Result<CreateVectorIndexStatement, ParseError>
    // pub fn parse_drop_vector_index(&mut self) -> Result<DropVectorIndexStatement, ParseError>
}
```

**对比全文检索**:
```rust
impl Parser {
    pub fn parse_search(&mut self) -> Result<SearchStatement, ParseError> {
        // ✅ 已实现
    }
}
```

---

### 2.3 验证器层缺失 ❌

**文件**: `src/query/validator/mod.rs`

**问题**: 没有向量检索验证器

**需要实现**:
```rust
// ❌ 缺失的文件
// src/query/validator/vector_validator.rs

pub struct VectorValidator;

impl VectorValidator {
    pub fn validate_create_vector_index(
        ctx: &mut ValidationContext,
        stmt: &CreateVectorIndexStatement,
    ) -> Result<(), ValidationError> {
        // 验证索引名称、Tag、字段、维度等
    }
    
    pub fn validate_search_vector(
        ctx: &mut ValidationContext,
        stmt: &SearchVectorStatement,
    ) -> Result<(), ValidationError> {
        // 验证查询向量、索引、过滤器等
    }
}
```

---

### 2.4 Planner 层缺失 ❌

**文件**: [`src/query/planning/planner.rs`](file:///d:/项目/database/graphDB/src/query/planning/planner.rs)

**问题**: `PlannerEnum` 中没有向量检索规划器

**当前状态** (第 260-270 行):
```rust
pub enum PlannerEnum {
    Match(MatchStatementPlanner),
    Go(GoPlanner),
    Lookup(LookupPlanner),
    // ... 其他规划器 ...
    FulltextSearch(FulltextSearchPlanner), // ✅ 全文检索规划器
    // ❌ 缺失：VectorSearch(VectorSearchPlanner)
}
```

**需要实现**:
```rust
// ❌ 缺失的文件
// src/query/planning/planner/vector_planner.rs

pub struct VectorSearchPlanner;

impl Planner for VectorSearchPlanner {
    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        // 将 AST 转换为 VectorSearchNode
    }
    
    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(
            stmt,
            Stmt::CreateVectorIndex(_)
                | Stmt::DropVectorIndex(_)
                | Stmt::SearchVector(_)
                | Stmt::LookupVector(_)
        )
    }
}
```

---

### 2.5 PlanNode 层缺失 ❌

**文件**: [`src/query/planning/plan/core/nodes/base/plan_node_enum.rs`](file:///d:/项目/database/graphDB/src/query/planning/plan/core/nodes/base/plan_node_enum.rs)

**问题**: `PlanNodeEnum` 中没有向量检索节点

**当前状态** (第 200-210 行):
```rust
pub enum PlanNodeEnum {
    // ... 其他节点 ...
    
    // ✅ 全文检索节点
    FulltextSearch(FulltextSearchNode),
    FulltextLookup(FulltextLookupNode),
    MatchFulltext(MatchFulltextNode),
    
    // ❌ 缺失：向量检索节点
    // VectorSearch(VectorSearchNode),
    // VectorScan(VectorScanNode),
    // CreateVectorIndex(CreateVectorIndexNode),
    // DropVectorIndex(DropVectorIndexNode),
}
```

**需要实现**:
```rust
// ❌ 缺失的文件
// src/query/planning/plan/core/nodes/data_access/vector_search.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchNode {
    pub id: NodeId,
    pub span: Span,
    pub index_name: String,
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub query: VectorQueryExpr,
    pub threshold: Option<f32>,
    pub filter: Option<Expression>,
    pub limit: usize,
    pub offset: usize,
    pub output_fields: Vec<OutputField>,
}
```

---

### 2.6 Executor 层缺失 ❌

**文件**: [`src/query/executor/executor_enum.rs`](file:///d:/项目/database/graphDB/src/query/executor/executor_enum.rs)

**问题**: `ExecutorEnum` 中没有向量检索执行器

**当前状态** (第 30-40 行):
```rust
use super::data_access::{
    FulltextScanExecutor, FulltextSearchExecutor, GetEdgesExecutor, GetNeighborsExecutor,
    GetPropExecutor, GetVerticesExecutor, IndexScanExecutor, MatchFulltextExecutor,
    ScanEdgesExecutor, ScanVerticesExecutor,
};

pub enum ExecutorEnum<S: StorageClient + Send + 'static> {
    // ... 其他执行器 ...
    
    // ✅ 全文检索执行器
    FulltextSearch(FulltextSearchExecutor<S>),
    FulltextScan(FulltextScanExecutor<S>),
    MatchFulltext(MatchFulltextExecutor<S>),
    
    // ❌ 缺失：向量检索执行器
    // VectorSearch(VectorSearchExecutor<S>),
    // VectorScan(VectorScanExecutor<S>),
}
```

**需要实现**:
```rust
// ❌ 缺失的文件
// src/query/executor/data_access/vector_search.rs

pub struct VectorSearchExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    node: VectorSearchNode,
    coordinator: Arc<VectorCoordinator>,
}

#[async_trait]
impl<S: StorageClient> Executor for VectorSearchExecutor<S> {
    async fn execute(&self, ctx: &mut ExecutionContext) -> Result<ExecutionResult, ExecutorError> {
        // 1. 获取查询向量
        // 2. 构建过滤器
        // 3. 执行向量搜索
        // 4. 返回结果
    }
}
```

**文件**: [`src/query/executor/data_access/mod.rs`](file:///d:/项目/database/graphDB/src/query/executor/data_access/mod.rs)

**当前状态**:
```rust
pub mod edge;
pub mod fulltext_search;
pub mod index;
pub mod match_fulltext;
pub mod neighbor;
pub mod path;
pub mod property;
pub mod search;
pub mod vertex;

// ❌ 缺失：
// pub mod vector_search;
```

---

### 2.7 执行器工厂缺失 ❌

**文件**: [`src/query/executor/factory/executors/plan_executor.rs`](file:///d:/项目/database/graphDB/src/query/executor/factory/executors/plan_executor.rs)

**问题**: 执行器创建逻辑中没有向量检索执行器

**当前状态** (第 200-280 行):
```rust
let is_stateful_executor = matches!(
    executor_type,
    "CreateSpace"
        | "DropSpace"
        // ... 其他执行器 ...
        | "FulltextSearch"
        | "FulltextLookup"
        // ❌ 缺失："VectorSearch"
);
```

**需要实现**:
```rust
impl<S: StorageClient> PlanExecutor<S> {
    fn build_executor_chain(
        &self,
        node: &PlanNodeEnum,
        storage: Arc<S>,
        ctx: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        match node {
            // ... 其他执行器 ...
            
            // ❌ 缺失：
            // PlanNodeEnum::VectorSearch(n) => {
            //     Ok(ExecutorEnum::VectorSearch(VectorSearchExecutor::new(n, coordinator, storage)?))
            // }
            // PlanNodeEnum::CreateVectorIndex(n) => {
            //     Ok(ExecutorEnum::CreateVectorIndex(CreateVectorIndexExecutor::new(n, coordinator)?))
            // }
            
            _ => Err(QueryError::ExecutionError(format!(
                "暂不支持的执行器类型：{}",
                node.name()
            ))),
        }
    }
}
```

---

### 2.8 执行上下文缺失 ❌

**文件**: `src/query/executor/base/execution_context.rs`

**问题**: `ExecutionContext` 中没有提供 `vector_coordinator()` 方法

**需要实现**:
```rust
pub struct ExecutionContext {
    // ... 其他字段 ...
    vector_coordinator: Option<Arc<VectorCoordinator>>,
}

impl ExecutionContext {
    // ❌ 缺失的方法
    // pub fn vector_coordinator(&self) -> Option<&Arc<VectorCoordinator>> {
    //     self.vector_coordinator.as_ref()
    // }
    
    // pub fn set_vector_coordinator(&mut self, coordinator: Arc<VectorCoordinator>) {
    //     self.vector_coordinator = Some(coordinator);
    // }
}
```

---

## 三、对比全文检索

### 3.1 完整性对比

| 层级 | 全文检索 | 向量检索 | 状态 |
|------|---------|---------|------|
| AST 语句定义 | ✅ 已实现 | ❌ 缺失 | 🔴 |
| 解析器支持 | ✅ 已实现 | ❌ 缺失 | 🔴 |
| 验证器 | ✅ 已实现 | ❌ 缺失 | 🔴 |
| Planner | ✅ 已实现 | ❌ 缺失 | 🔴 |
| PlanNode | ✅ 已实现 | ❌ 缺失 | 🔴 |
| Executor | ✅ 已实现 | ❌ 缺失 | 🔴 |
| 执行器工厂 | ✅ 已实现 | ❌ 缺失 | 🔴 |
| 执行上下文 | ✅ 已实现 | ❌ 缺失 | 🔴 |
| **底层基础设施** | ✅ 已实现 | ✅ 已实现 | 🟢 |
| **数据同步机制** | ✅ 已实现 | ✅ 已实现 | 🟢 |

### 3.2 代码量对比

| 模块 | 全文检索文件数 | 向量检索文件数 |
|------|--------------|--------------|
| AST | 1 (fulltext.rs) | 0 |
| Validator | 1 | 0 |
| Planner | 1 (fulltext_planner.rs) | 0 |
| PlanNode | 1 (fulltext_nodes.rs) | 0 |
| Executor | 3 (fulltext_search.rs, fulltext_scan.rs, match_fulltext.rs) | 0 |
| **总计** | **6** | **0** |

---

## 四、根本原因分析

### 4.1 设计策略

项目采用了**分阶段实施**的策略：

**Phase 1-3** (已完成):
- ✅ Phase 1: VectorEngine Trait 和 Qdrant 适配器
- ✅ Phase 2: VectorIndexManager (索引管理器)
- ✅ Phase 3: VectorCoordinator (协调器) 和同步扩展

**Phase 4-5** (未完成):
- ❌ Phase 4: 查询引擎集成 (Parser → Validator → Planner → PlanNode → Executor)
- ❌ Phase 5: 嵌入服务集成 (可选)

### 4.2 文档与实现的时间差

从文档日期分析：
- 设计文档：2026-04-06
- 当前代码：2026-04-07

设计文档完成时间早于当前代码时间，说明**设计已完成但实施尚未开始**。

---

## 五、影响评估

### 5.1 功能影响

**当前无法使用的功能**:
1. ❌ `SEARCH VECTOR` - 向量相似度搜索
2. ❌ `CREATE VECTOR INDEX` - 创建向量索引
3. ❌ `DROP VECTOR INDEX` - 删除向量索引
4. ❌ `LOOKUP VECTOR` - 向量查找
5. ❌ `MATCH WHERE ... SIMILAR TO` - 图查询中的向量过滤

**当前可以使用的功能**:
1. ✅ 通过 API 直接调用 `VectorIndexManager`
2. ✅ 数据变更时的自动向量同步

### 5.2 使用场景限制

**无法支持的应用**:
- 语义搜索应用
- 推荐系统
- RAG (检索增强生成) 应用
- 多模态搜索
- 基于向量相似度的图查询

---

## 六、技术债务

### 6.1 已识别的技术债务

1. **查询引擎集成缺失** - 8 个层级的实现缺失
2. **测试缺失** - 只有底层集成测试，缺少查询层测试
3. **文档更新** - 设计文档需要更新为实施指南

### 6.2 预估工作量

基于设计文档中的时间估算：

| 阶段 | 任务 | 预估时间 |
|------|------|---------|
| Phase 4-1 | AST 扩展 | 4 小时 |
| Phase 4-2 | 验证器 | 3 小时 |
| Phase 4-3 | Planner | 4 小时 |
| Phase 4-4 | PlanNode | 3 小时 |
| Phase 4-5 | Executor | 8 小时 |
| Phase 4-6 | 执行器工厂 | 3 小时 |
| Phase 4-7 | 执行上下文 | 2 小时 |
| Phase 4-8 | 集成测试 | 4 小时 |
| **总计** | | **~31 小时** (约 4 个工作日) |

---

## 七、建议

### 7.1 短期建议

1. **立即实施 Phase 4** - 完成查询引擎集成
2. **优先实现核心功能** - `SEARCH VECTOR` 和 `CREATE VECTOR INDEX`
3. **编写集成测试** - 确保端到端功能正常

### 7.2 中期建议

1. **实现 Phase 5** - 嵌入服务集成（文本到向量）
2. **优化性能** - 向量搜索缓存、批量搜索
3. **完善文档** - 用户指南、API 文档

### 7.3 长期建议

1. **支持更多向量引擎** - Milvus、Weaviate 适配器
2. **高级搜索功能** - 混合搜索（向量 + 全文）、多向量搜索
3. **查询优化** - 基于代价的向量查询优化

---

## 八、参考文档

- [向量检索集成分析](vector_search_integration_analysis.md) - 完整架构设计
- [查询集成方案](query_integration.md) - 查询引擎集成详细设计
- [Qdrant 适配器实现](qdrant_adapter_implementation.md) - 后端实现细节
- [数据同步机制](data_sync_mechanism.md) - 同步机制设计

---

*文档生成时间：2026-04-07*  
*版本：v1.0*
