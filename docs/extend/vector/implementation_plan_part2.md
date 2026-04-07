# 向量检索查询流程集成实施方案（续）

> 创建日期：2026-04-07  
> 前序文档：[implementation_plan.md](implementation_plan.md)  
> 本部分包含：Phase 5-8 详细实现

---

## Phase 5: PlanNode 扩展 (预计 3 小时)

### 5.1 创建向量检索计划节点

**文件**: `src/query/planning/plan/core/nodes/data_access/vector_search.rs` (新建)

```rust
//! Vector Search Plan Nodes

use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::types::span::Span;
use crate::define_plan_node;
use crate::query::planning::plan::core::node_id_generator::next_node_id;
use crate::query::planning::plan::core::nodes::base::plan_node_traits::PlanNode;
use crate::query::planning::plan::core::nodes::base::plan_node_category::PlanNodeCategory;
use serde::{Deserialize, Serialize};

use crate::query::ast::vector::{DistanceMetric, VectorQueryExpr};

/// 输出字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputField {
    pub name: String,
    pub alias: Option<String>,
    pub expr: ContextualExpression,
}

/// 向量搜索计划节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchNode {
    pub id: i64,
    pub span: Span,
    pub index_name: String,
    pub space_id: u64,
    pub tag_name: String,
    pub field_name: String,
    pub query: VectorQueryExpr,
    pub threshold: Option<f32>,
    pub filter: Option<ContextualExpression>,
    pub limit: usize,
    pub offset: usize,
    pub output_fields: Vec<OutputField>,
}

impl VectorSearchNode {
    pub fn new(
        index_name: String,
        space_id: u64,
        tag_name: String,
        field_name: String,
        query: VectorQueryExpr,
        threshold: Option<f32>,
        filter: Option<ContextualExpression>,
        limit: usize,
        offset: usize,
        output_fields: Vec<OutputField>,
    ) -> Self {
        Self {
            id: next_node_id(),
            span: query.span.clone(),
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
        }
    }
}

impl PlanNode for VectorSearchNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "VectorSearch"
    }

    fn category(&self) -> PlanNodeCategory {
        PlanNodeCategory::DataAccess
    }

    fn description(&self) -> &str {
        "向量相似度搜索"
    }

    fn output_var(&self) -> Option<&str> {
        None
    }

    fn col_names(&self) -> &[String] {
        // TODO: 返回输出列名
        &[]
    }

    fn set_output_var(&mut self, _var: String) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(
        self,
    ) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::VectorSearch(self)
    }
}

/// 创建向量索引计划节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorIndexNode {
    pub id: i64,
    pub span: Span,
    pub index_name: String,
    pub space_name: String,
    pub tag_name: String,
    pub field_name: String,
    pub config: crate::vector::VectorIndexConfig,
    pub if_not_exists: bool,
}

impl CreateVectorIndexNode {
    pub fn new(
        index_name: String,
        space_name: String,
        tag_name: String,
        field_name: String,
        config: crate::vector::VectorIndexConfig,
        if_not_exists: bool,
    ) -> Self {
        Self {
            id: next_node_id(),
            span: Span::default(),
            index_name,
            space_name,
            tag_name,
            field_name,
            config,
            if_not_exists,
        }
    }
}

impl PlanNode for CreateVectorIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "CreateVectorIndex"
    }

    fn category(&self) -> PlanNodeCategory {
        PlanNodeCategory::Management
    }

    fn description(&self) -> &str {
        "创建向量索引"
    }

    fn output_var(&self) -> Option<&str> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn set_output_var(&mut self, _var: String) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(
        self,
    ) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::CreateVectorIndex(self)
    }
}

/// 删除向量索引计划节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropVectorIndexNode {
    pub id: i64,
    pub span: Span,
    pub index_name: String,
    pub space_name: String,
    pub if_exists: bool,
}

impl DropVectorIndexNode {
    pub fn new(
        index_name: String,
        space_name: String,
        if_exists: bool,
    ) -> Self {
        Self {
            id: next_node_id(),
            span: Span::default(),
            index_name,
            space_name,
            if_exists,
        }
    }
}

impl PlanNode for DropVectorIndexNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "DropVectorIndex"
    }

    fn category(&self) -> PlanNodeCategory {
        PlanNodeCategory::Management
    }

    fn description(&self) -> &str {
        "删除向量索引"
    }

    fn output_var(&self) -> Option<&str> {
        None
    }

    fn col_names(&self) -> &[String] {
        &[]
    }

    fn set_output_var(&mut self, _var: String) {}

    fn set_col_names(&mut self, _names: Vec<String>) {}

    fn into_enum(
        self,
    ) -> crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum::DropVectorIndex(self)
    }
}
```

### 5.2 更新 PlanNodeEnum

**文件**: [`src/query/planning/plan/core/nodes/base/plan_node_enum.rs`](file:///d:/项目/database/graphDB/src/query/planning/plan/core/nodes/base/plan_node_enum.rs)

**修改 1**: 导入新节点类型 (第 20-30 行附近)

```rust
use crate::query::planning::plan::core::nodes::management::fulltext_nodes::{
    AlterFulltextIndexNode, CreateFulltextIndexNode, DescribeFulltextIndexNode,
    DropFulltextIndexNode, FulltextLookupNode, FulltextSearchNode, MatchFulltextNode,
    ShowFulltextIndexNode,
};

// 新增导入
use crate::query::planning::plan::core::nodes::data_access::vector_search::{
    VectorSearchNode, CreateVectorIndexNode, DropVectorIndexNode,
};
```

**修改 2**: 添加枚举变体 (第 200-210 行附近)

```rust
pub enum PlanNodeEnum {
    // ... 现有变体 ...
    
    // Full-text Search Nodes
    FulltextSearch(FulltextSearchNode),
    FulltextLookup(FulltextLookupNode),
    MatchFulltext(MatchFulltextNode),
    
    // Vector Search Nodes (新增)
    VectorSearch(VectorSearchNode),
    CreateVectorIndex(CreateVectorIndexNode),
    DropVectorIndex(DropVectorIndexNode),
}
```

**修改 3**: 更新宏定义 (第 438-445 行附近)

```rust
crate::define_enum_as_methods! {
    PlanNodeEnum,
    // ... 现有方法 ...
    
    // Full-text Search Nodes
    (FulltextSearch, as_fulltext_search, FulltextSearchNode),
    (FulltextLookup, as_fulltext_lookup, FulltextLookupNode),
    (MatchFulltext, as_match_fulltext, MatchFulltextNode),
    
    // Vector Search Nodes (新增)
    (VectorSearch, as_vector_search, VectorSearchNode),
    (CreateVectorIndex, as_create_vector_index, CreateVectorIndexNode),
    (DropVectorIndex, as_drop_vector_index, DropVectorIndexNode),
}
```

**修改 4**: 更新类型名称宏 (第 559-620 行附近)

```rust
crate::define_enum_type_name! {
    PlanNodeEnum,
    // ... 现有类型 ...
    
    // Full-text Search
    (FulltextSearch, "FulltextSearch"),
    (FulltextLookup, "FulltextLookup"),
    (MatchFulltext, "MatchFulltext"),
    
    // Vector Search (新增)
    (VectorSearch, "VectorSearch"),
    (CreateVectorIndex, "CreateVectorIndex"),
    (DropVectorIndex, "DropVectorIndex"),
}
```

**修改 5**: 更新 `is_query_node()` 方法 (第 1040-1056 行附近)

```rust
pub fn is_query_node(&self) -> bool {
    matches!(
        self,
        PlanNodeEnum::GetVertices(_)
            | PlanNodeEnum::GetEdges(_)
            | PlanNodeEnum::GetNeighbors(_)
            | PlanNodeEnum::Expand(_)
            | PlanNodeEnum::ExpandAll(_)
            | PlanNodeEnum::Traverse(_)
            | PlanNodeEnum::AppendVertices(_)
            | PlanNodeEnum::ScanVertices(_)
            | PlanNodeEnum::ScanEdges(_)
            | PlanNodeEnum::FulltextSearch(_)
            | PlanNodeEnum::FulltextLookup(_)
            | PlanNodeEnum::MatchFulltext(_)
            | PlanNodeEnum::VectorSearch(_)  // ← 新增
    )
}
```

**修改 6**: 更新 `is_admin_node()` 方法 (第 1020-1040 行附近)

```rust
pub fn is_admin_node(&self) -> bool {
    matches!(
        self,
        PlanNodeEnum::CreateSpace(_)
            | PlanNodeEnum::DropSpace(_)
            | PlanNodeEnum::CreateTag(_)
            | PlanNodeEnum::DropTag(_)
            | PlanNodeEnum::CreateEdge(_)
            | PlanNodeEnum::DropEdge(_)
            | PlanNodeEnum::CreateTagIndex(_)
            | PlanNodeEnum::DropTagIndex(_)
            | PlanNodeEnum::CreateEdgeIndex(_)
            | PlanNodeEnum::DropEdgeIndex(_)
            | PlanNodeEnum::CreateFulltextIndex(_)
            | PlanNodeEnum::DropFulltextIndex(_)
            | PlanNodeEnum::AlterFulltextIndex(_)
            | PlanNodeEnum::ShowFulltextIndex(_)
            | PlanNodeEnum::DescribeFulltextIndex(_)
            | PlanNodeEnum::CreateVectorIndex(_)  // ← 新增
            | PlanNodeEnum::DropVectorIndex(_)    // ← 新增
    )
}
```

**修改 7**: 更新内存估算方法 (第 1100-1600 行之间)

```rust
pub fn estimate_memory(&self) -> usize {
    let base_size = std::mem::size_of::<PlanNodeEnum>();
    
    match self {
        // ... 现有分支 ...
        
        // Vector Search Nodes (新增)
        PlanNodeEnum::VectorSearch(node) => base_size + estimate_node_memory(node),
        PlanNodeEnum::CreateVectorIndex(node) => base_size + estimate_node_memory(node),
        PlanNodeEnum::DropVectorIndex(node) => base_size + estimate_node_memory(node),
        
        // ... 其他分支 ...
    }
}
```

**修改 8**: 更新访问者模式 (在 `plan_node_visitor.rs` 中)

**文件**: `src/query/planning/plan/core/nodes/base/plan_node_visitor.rs`

```rust
pub trait PlanNodeVisitor<T> {
    // ... 现有方法 ...
    
    // Vector Search Nodes (新增)
    fn visit_vector_search(&mut self, node: &VectorSearchNode) -> T;
    fn visit_create_vector_index(&mut self, node: &CreateVectorIndexNode) -> T;
    fn visit_drop_vector_index(&mut self, node: &DropVectorIndexNode) -> T;
}

// 在 visit 方法中添加分支
impl PlanNodeVisitor<()> for SomeVisitor {
    fn visit(&self, node: &PlanNodeEnum) {
        match node {
            // ... 现有分支 ...
            
            // Vector Search Nodes (新增)
            PlanNodeEnum::VectorSearch(n) => self.visit_vector_search(n),
            PlanNodeEnum::CreateVectorIndex(n) => self.visit_create_vector_index(n),
            PlanNodeEnum::DropVectorIndex(n) => self.visit_drop_vector_index(n),
        }
    }
}
```

---

## Phase 6: Executor 扩展 (预计 8 小时)

### 6.1 创建向量检索执行器

**文件**: `src/query/executor/data_access/vector_search.rs` (新建)

```rust
//! Vector Search Executor

use std::sync::Arc;

use crate::core::error::QueryError;
use crate::core::{DataSet, Value};
use crate::query::ast::vector::{VectorQueryExpr, VectorQueryType};
use crate::query::executor::base::{
    BaseExecutor, DBResult, ExecutionResult, Executor, ExecutorStats, HasStorage,
};
use crate::query::executor::ExecutionContext;
use crate::query::planning::plan::core::nodes::data_access::vector_search::VectorSearchNode;
use crate::vector::VectorCoordinator;
use async_trait::async_trait;
use parking_lot::Mutex;

/// 向量搜索执行器
pub struct VectorSearchExecutor<S: crate::storage::StorageClient> {
    base: BaseExecutor<S>,
    node: VectorSearchNode,
    coordinator: Arc<VectorCoordinator>,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: crate::storage::StorageClient> VectorSearchExecutor<S> {
    pub fn new(
        base_config: crate::query::executor::base::ExecutorConfig<S>,
        node: VectorSearchNode,
        coordinator: Arc<VectorCoordinator>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                base_config.id,
                "VectorSearchExecutor".to_string(),
                base_config.storage,
                base_config.expr_context,
            ),
            node,
            coordinator,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// 解析查询向量
    async fn resolve_query_vector(
        &self,
        ctx: &ExecutionContext,
        query: &VectorQueryExpr,
    ) -> Result<Vec<f32>, QueryError> {
        match &query.query_type {
            VectorQueryType::Vector => {
                // 解析向量字面量 [0.1, 0.2, ...]
                self.parse_vector_literal(&query.query_text)
                    .ok_or_else(|| QueryError::ExecutionError("无效的向量格式".to_string()))
            }
            VectorQueryType::Text => {
                // 使用嵌入服务将文本转换为向量
                // TODO: 需要嵌入服务
                Err(QueryError::ExecutionError("嵌入服务尚未实现".to_string()))
            }
            VectorQueryType::Parameter => {
                // 从参数中获取向量
                // TODO: 从执行上下文获取参数
                Err(QueryError::ExecutionError("参数向量尚未实现".to_string()))
            }
        }
    }
    
    /// 解析向量字面量
    fn parse_vector_literal(&self, text: &str) -> Option<Vec<f32>> {
        // 解析 "[0.1, 0.2, 0.3]" 格式
        let text = text.trim().trim_start_matches('[').trim_end_matches(']');
        text.split(',')
            .map(|s| s.trim().parse::<f32>().ok())
            .collect()
    }
    
    /// 构建向量过滤器
    fn build_vector_filter(
        &self,
        ctx: &ExecutionContext,
        filter: &crate::core::types::expr::contextual::ContextualExpression,
    ) -> Result<crate::vector::VectorFilter, QueryError> {
        // TODO: 将表达式转换为 VectorFilter
        // 这需要将查询引擎的表达式转换为向量引擎的过滤器格式
        todo!()
    }
}

#[async_trait]
impl<S: crate::storage::StorageClient> Executor for VectorSearchExecutor<S> {
    async fn execute(&self, ctx: &mut ExecutionContext) -> Result<ExecutionResult, QueryError> {
        // 1. 获取查询向量
        let query_vector = self.resolve_query_vector(ctx, &self.node.query).await?;
        
        // 2. 构建过滤器
        let filter = self.node.filter.as_ref()
            .map(|f| self.build_vector_filter(ctx, f))
            .transpose()?;
        
        // 3. 构建搜索查询
        let search_query = crate::vector::SearchQuery {
            vector: query_vector,
            limit: self.node.limit as u64,
            offset: Some(self.node.offset as u64),
            filter,
            score_threshold: self.node.threshold,
            with_payload: true,
            with_vectors: false,
        };
        
        // 4. 执行向量搜索
        let search_results = self.coordinator
            .search(
                self.node.space_id,
                &self.node.tag_name,
                &self.node.field_name,
                search_query,
            )
            .await
            .map_err(|e| QueryError::ExecutionError(format!("向量搜索失败：{}", e)))?;
        
        // 5. 构建返回结果
        let mut dataset = DataSet::new();
        
        // 设置列名
        let mut col_names = Vec::new();
        for field in &self.node.output_fields {
            col_names.push(field.alias.clone().unwrap_or_else(|| field.name.clone()));
        }
        dataset.set_col_names(col_names);
        
        // 填充数据
        for result in search_results.results {
            let mut row = Vec::new();
            for field in &self.node.output_fields {
                let value = match field.name.as_str() {
                    "id" | "vertex_id" => {
                        // 从 payload 中获取顶点 ID
                        Value::String(result.id.clone())
                    }
                    "score" => {
                        Value::Double(result.score as f64)
                    }
                    _ => {
                        // 从 payload 中获取其他字段
                        result.payload.get(&field.name)
                            .cloned()
                            .unwrap_or(Value::Null)
                    }
                };
                row.push(value);
            }
            dataset.add_row(row);
        }
        
        Ok(ExecutionResult::from_dataset(dataset))
    }

    fn get_stats(&self) -> ExecutorStats {
        ExecutorStats {
            executor_type: "VectorSearchExecutor".to_string(),
            rows_processed: 0,
            bytes_processed: 0,
            execution_time_ms: 0,
        }
    }
}

impl<S: crate::storage::StorageClient> HasStorage<S> for VectorSearchExecutor<S> {
    fn storage(&self) -> &Arc<S> {
        self.base.storage()
    }
}
```

### 6.2 创建向量索引管理执行器

**文件**: `src/query/executor/admin/vector_index.rs` (新建)

```rust
//! Vector Index Management Executors

use std::sync::Arc;

use crate::core::error::QueryError;
use crate::query::executor::base::{
    BaseExecutor, DBResult, ExecutionResult, Executor, ExecutorStats, HasStorage,
};
use crate::query::executor::ExecutionContext;
use crate::query::planning::plan::core::nodes::management::vector_nodes::CreateVectorIndexNode;
use crate::vector::VectorCoordinator;
use async_trait::async_trait;

/// 创建向量索引执行器
pub struct CreateVectorIndexExecutor<S: crate::storage::StorageClient> {
    base: BaseExecutor<S>,
    node: CreateVectorIndexNode,
    coordinator: Arc<VectorCoordinator>,
}

impl<S: crate::storage::StorageClient> CreateVectorIndexExecutor<S> {
    pub fn new(
        base_config: crate::query::executor::base::ExecutorConfig<S>,
        node: CreateVectorIndexNode,
        coordinator: Arc<VectorCoordinator>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                base_config.id,
                "CreateVectorIndexExecutor".to_string(),
                base_config.storage,
                base_config.expr_context,
            ),
            node,
            coordinator,
        }
    }
}

#[async_trait]
impl<S: crate::storage::StorageClient> Executor for CreateVectorIndexExecutor<S> {
    async fn execute(&self, _ctx: &mut ExecutionContext) -> Result<ExecutionResult, QueryError> {
        // 1. 检查索引是否已存在
        if self.coordinator.index_exists(
            0, // TODO: 获取 space_id
            &self.node.tag_name,
            &self.node.field_name,
        ).await {
            if !self.node.if_not_exists {
                return Err(QueryError::ExecutionError(
                    format!("索引 '{}' 已存在", self.node.index_name)
                ));
            }
            return Ok(ExecutionResult::empty());
        }
        
        // 2. 创建向量索引
        self.coordinator
            .create_index(
                0, // TODO: 获取 space_id
                &self.node.tag_name,
                &self.node.field_name,
                self.node.config.clone(),
            )
            .await
            .map_err(|e| QueryError::ExecutionError(format!("创建向量索引失败：{}", e)))?;
        
        // 3. 返回成功
        Ok(ExecutionResult::empty())
    }

    fn get_stats(&self) -> ExecutorStats {
        ExecutorStats {
            executor_type: "CreateVectorIndexExecutor".to_string(),
            rows_processed: 0,
            bytes_processed: 0,
            execution_time_ms: 0,
        }
    }
}

/// 删除向量索引执行器
pub struct DropVectorIndexExecutor<S: crate::storage::StorageClient> {
    base: BaseExecutor<S>,
    node: crate::query::planning::plan::core::nodes::management::vector_nodes::DropVectorIndexNode,
    coordinator: Arc<VectorCoordinator>,
}

impl<S: crate::storage::StorageClient> DropVectorIndexExecutor<S> {
    pub fn new(
        base_config: crate::query::executor::base::ExecutorConfig<S>,
        node: crate::query::planning::plan::core::nodes::management::vector_nodes::DropVectorIndexNode,
        coordinator: Arc<VectorCoordinator>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                base_config.id,
                "DropVectorIndexExecutor".to_string(),
                base_config.storage,
                base_config.expr_context,
            ),
            node,
            coordinator,
        }
    }
}

#[async_trait]
impl<S: crate::storage::StorageClient> Executor for DropVectorIndexExecutor<S> {
    async fn execute(&self, _ctx: &mut ExecutionContext) -> Result<ExecutionResult, QueryError> {
        // 1. 检查索引是否存在
        let exists = self.coordinator.index_exists(
            0, // TODO: 获取 space_id
            &self.node.index_name,
            "", // field_name 需要从索引名解析
        ).await;
        
        if !exists && !self.node.if_exists {
            return Err(QueryError::ExecutionError(
                format!("索引 '{}' 不存在", self.node.index_name)
            ));
        }
        
        // 2. 删除向量索引
        if exists {
            self.coordinator
                .drop_index(
                    0, // TODO: 获取 space_id
                    &self.node.index_name,
                )
                .await
                .map_err(|e| QueryError::ExecutionError(format!("删除向量索引失败：{}", e)))?;
        }
        
        Ok(ExecutionResult::empty())
    }

    fn get_stats(&self) -> ExecutorStats {
        ExecutorStats {
            executor_type: "DropVectorIndexExecutor".to_string(),
            rows_processed: 0,
            bytes_processed: 0,
            execution_time_ms: 0,
        }
    }
}
```

### 6.3 更新 ExecutorEnum

**文件**: [`src/query/executor/executor_enum.rs`](file:///d:/项目/database/graphDB/src/query/executor/executor_enum.rs)

**修改 1**: 添加导入 (第 30-40 行附近)

```rust
use super::data_access::{
    FulltextScanExecutor, FulltextSearchExecutor, GetEdgesExecutor, GetNeighborsExecutor,
    GetPropExecutor, GetVerticesExecutor, IndexScanExecutor, MatchFulltextExecutor,
    ScanEdgesExecutor, ScanVerticesExecutor,
};

// 新增导入
use super::data_access::vector_search::VectorSearchExecutor;
use super::admin::vector_index::{CreateVectorIndexExecutor, DropVectorIndexExecutor};
```

**修改 2**: 添加枚举变体 (第 60-120 行之间)

```rust
pub enum ExecutorEnum<S: StorageClient + Send + 'static> {
    // ... 现有变体 ...
    
    // Full-text Search
    FulltextSearch(FulltextSearchExecutor<S>),
    FulltextScan(FulltextScanExecutor<S>),
    MatchFulltext(MatchFulltextExecutor<S>),
    
    // Vector Search (新增)
    VectorSearch(VectorSearchExecutor<S>),
    CreateVectorIndex(CreateVectorIndexExecutor<S>),
    DropVectorIndex(DropVectorIndexExecutor<S>),
}
```

**修改 3**: 更新 `mod.rs` 导出

**文件**: [`src/query/executor/data_access/mod.rs`](file:///d:/项目/database/graphDB/src/query/executor/data_access/mod.rs)

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
pub mod vector_search;  // ← 新增

pub use edge::{GetEdgesExecutor, ScanEdgesExecutor};
pub use fulltext_search::{FulltextScanConfig, FulltextScanExecutor, FulltextSearchExecutor};
pub use index::LookupIndexExecutor;
pub use match_fulltext::MatchFulltextExecutor;
pub use neighbor::GetNeighborsExecutor;
pub use path::AllPathsExecutor;
pub use property::GetPropExecutor;
pub use search::IndexScanExecutor;
pub use vertex::{GetVerticesExecutor, GetVerticesParams, ScanVerticesExecutor};
pub use vector_search::VectorSearchExecutor;  // ← 新增
```

---

## Phase 7: 执行器工厂扩展 (预计 3 小时)

### 7.1 更新执行器创建逻辑

**文件**: [`src/query/executor/factory/executors/plan_executor.rs`](file:///d:/项目/database/graphDB/src/query/executor/factory/executors/plan_executor.rs)

**修改 1**: 添加导入 (第 1-30 行附近)

```rust
use super::super::base::{ExecutionContext, ExecutionResult, Executor, InputExecutor};
use super::super::data_access::vector_search::VectorSearchExecutor;
use super::super::admin::vector_index::{CreateVectorIndexExecutor, DropVectorIndexExecutor};
use crate::vector::VectorCoordinator;
```

**修改 2**: 更新 stateful 执行器判断 (第 200-280 行附近)

```rust
let is_stateful_executor = matches!(
    executor_type,
    "CreateSpace"
        | "DropSpace"
        // ... 其他 stateful 执行器 ...
        | "FulltextSearch"
        | "FulltextLookup"
        | "VectorSearch"  // ← 新增
        | "CreateVectorIndex"  // ← 新增
        | "DropVectorIndex"  // ← 新增
);
```

**修改 3**: 添加执行器创建分支

```rust
fn build_executor_chain(
    &self,
    node: &PlanNodeEnum,
    storage: Arc<S>,
    ctx: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    match node {
        // ... 现有分支 ...
        
        // Vector Search Nodes (新增)
        PlanNodeEnum::VectorSearch(n) => {
            let coordinator = ctx.vector_coordinator()
                .ok_or_else(|| QueryError::ExecutionError("VectorCoordinator 不可用".to_string()))?;
            
            let base_config = crate::query::executor::base::ExecutorConfig {
                id: n.id,
                storage: storage.clone(),
                expr_context: ctx.expression_context().clone(),
            };
            
            Ok(ExecutorEnum::VectorSearch(
                VectorSearchExecutor::new(base_config, n.clone(), coordinator.clone())
            ))
        }
        
        PlanNodeEnum::CreateVectorIndex(n) => {
            let coordinator = ctx.vector_coordinator()
                .ok_or_else(|| QueryError::ExecutionError("VectorCoordinator 不可用".to_string()))?;
            
            let base_config = crate::query::executor::base::ExecutorConfig {
                id: n.id,
                storage: storage.clone(),
                expr_context: ctx.expression_context().clone(),
            };
            
            Ok(ExecutorEnum::CreateVectorIndex(
                CreateVectorIndexExecutor::new(base_config, n.clone(), coordinator.clone())
            ))
        }
        
        PlanNodeEnum::DropVectorIndex(n) => {
            let coordinator = ctx.vector_coordinator()
                .ok_or_else(|| QueryError::ExecutionError("VectorCoordinator 不可用".to_string()))?;
            
            let base_config = crate::query::executor::base::ExecutorConfig {
                id: n.id,
                storage: storage.clone(),
                expr_context: ctx.expression_context().clone(),
            };
            
            Ok(ExecutorEnum::DropVectorIndex(
                DropVectorIndexExecutor::new(base_config, n.clone(), coordinator.clone())
            ))
        }
        
        // ... 其他分支 ...
        
        _ => Err(QueryError::ExecutionError(format!(
            "暂不支持的执行器类型：{}",
            node.name()
        ))),
    }
}
```

---

## Phase 8: 执行上下文扩展 (预计 2 小时)

### 8.1 更新 ExecutionContext

**文件**: `src/query/executor/base/execution_context.rs` (可能需要新建)

```rust
//! Execution Context

use std::sync::Arc;

use crate::query::validator::context::ExpressionAnalysisContext;
use crate::vector::VectorCoordinator;

/// 执行上下文
pub struct ExecutionContext {
    /// 表达式分析上下文
    expr_context: Arc<ExpressionAnalysisContext>,
    
    /// 向量协调器
    vector_coordinator: Option<Arc<VectorCoordinator>>,
}

impl ExecutionContext {
    pub fn new(expr_context: Arc<ExpressionAnalysisContext>) -> Self {
        Self {
            expr_context,
            vector_coordinator: None,
        }
    }
    
    pub fn with_vector_coordinator(
        expr_context: Arc<ExpressionAnalysisContext>,
        coordinator: Arc<VectorCoordinator>,
    ) -> Self {
        Self {
            expr_context,
            vector_coordinator: Some(coordinator),
        }
    }
    
    pub fn expression_context(&self) -> &Arc<ExpressionAnalysisContext> {
        &self.expr_context
    }
    
    pub fn vector_coordinator(&self) -> Option<&Arc<VectorCoordinator>> {
        self.vector_coordinator.as_ref()
    }
    
    pub fn set_vector_coordinator(&mut self, coordinator: Arc<VectorCoordinator>) {
        self.vector_coordinator = Some(coordinator);
    }
}
```

### 8.2 更新 QueryPipelineManager

**文件**: `src/query/query_pipeline_manager.rs`

**修改**: 在执行计划前设置 VectorCoordinator

```rust
fn execute_plan(
    &mut self,
    query_context: Arc<QueryContext>,
    plan: crate::query::planning::plan::ExecutionPlan,
) -> DBResult<ExecutionResult> {
    use crate::query::executor::factory::executors::plan_executor::PlanExecutor;
    let mut plan_executor =
        PlanExecutor::with_object_pool(self.executor_factory.clone(), self.object_pool.clone());
    
    // 设置 VectorCoordinator
    if let Some(coordinator) = &self.vector_coordinator {
        plan_executor.set_vector_coordinator(coordinator.clone());
    }
    
    plan_executor
        .execute_plan(query_context, plan)
        .map_err(|e| DBError::from(QueryError::pipeline_execution_error(e)))
}
```

---

## 九、后续工作

完成上述 8 个 Phase 后，还需要：

1. **集成测试** (4 小时) - 编写端到端测试
2. **性能优化** (8 小时) - 缓存、批量处理等
3. **文档完善** (4 小时) - 用户指南、API 文档
4. **Bug 修复** (时间不定) - 根据测试反馈修复问题

---

*文档生成时间：2026-04-07*  
*版本：v1.0*
