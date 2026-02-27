# 索引优化分析文档

## 概述

本文档分析 GraphDB 项目中索引机制的当前实现状态，并提出可进一步改进的方向。

## 当前已实现的索引

### 1. 索引架构

**整体架构**:
```
┌─────────────────────────────────────────────────────────────┐
│                    查询优化器                                │
│              (IndexSeekPlanner)                             │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                   索引选择器                                 │
│         (根据查询条件选择最优索引)                            │
└─────────────────────────────────────────────────────────────┘
                            │
            ┌───────────────┼───────────────┐
            ▼               ▼               ▼
┌──────────────────┐ ┌──────────────┐ ┌──────────────┐
│   标签索引        │ │   边类型索引  │ │   属性索引    │
│  (Tag Index)     │ │ (Edge Index) │ │ (Property)   │
└──────────────────┘ └──────────────┘ └──────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              索引数据管理器 (IndexDataManager)               │
│                     (RedbIndexDataManager)                  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    Redb 存储层                               │
└─────────────────────────────────────────────────────────────┘
```

### 2. 索引元数据管理

**实现位置**: `src/storage/metadata/index_metadata_manager.rs`

**功能说明**:
- 管理索引的元数据（名称、字段、类型等）
- 支持标签索引和边类型索引
- 支持索引的创建、删除、查询

**关键代码**:
```rust
/// 索引元数据管理器 trait
pub trait IndexMetadataManager {
    /// 创建标签索引
    fn create_tag_index(&self, space_id: u64, info: &Index) -> Result<bool, StorageError>;
    /// 删除标签索引
    fn drop_tag_index(&self, space_id: u64, index: &str) -> Result<bool, StorageError>;
    /// 获取标签索引
    fn get_tag_index(&self, space_id: u64, index: &str) -> Result<Option<Index>, StorageError>;
    /// 列出所有标签索引
    fn list_tag_indexes(&self, space_id: u64) -> Result<Vec<Index>, StorageError>;
    /// 重建标签索引
    fn rebuild_tag_index(&self, space_id: u64, index: &str) -> Result<bool, StorageError>;
    
    // 边索引类似...
}

/// 索引定义
pub struct Index {
    pub name: String,
    pub fields: Vec<String>,
    pub index_type: IndexType,
    pub status: IndexStatus,
}

pub enum IndexType {
    Tag,       // 标签索引
    Edge,      // 边索引
}

pub enum IndexStatus {
    Building,  // 构建中
    Active,    // 可用
    Error,     // 错误
}
```

### 3. 索引数据管理

**实现位置**: `src/storage/index/index_data_manager.rs`

**功能说明**:
- 管理索引数据的增删改查
- 支持正向索引和反向索引
- 基于 Redb 存储索引数据

**索引键设计**:
```rust
/// 顶点正向索引键格式
/// [space_id: u64] [type: u8=0x03] [index_name_len: u32] [index_name] 
/// [prop_value_len: u32] [prop_value] [vertex_id_len: u32] [vertex_id]
fn build_vertex_index_key(
    space_id: u64, 
    index_name: &str, 
    prop_value: &Value, 
    vertex_id: &Value
) -> Result<ByteKey, StorageError>;

/// 顶点反向索引键格式（用于删除）
/// [space_id: u64] [type: u8=0x01] [vertex_id_len: u32] [vertex_id] 
/// [index_name_len: u32] [index_name]
fn build_vertex_reverse_key(
    space_id: u64, 
    vertex_id: &Value
) -> Result<ByteKey, StorageError>;
```

**关键操作**:
```rust
pub trait IndexDataManager {
    /// 更新顶点索引
    fn update_vertex_indexes(
        &self, 
        space_id: u64, 
        vertex_id: &Value, 
        index_name: &str, 
        props: &[(String, Value)]
    ) -> Result<(), StorageError>;
    
    /// 查找标签索引
    fn lookup_tag_index(
        &self, 
        space_id: u64, 
        index: &Index, 
        value: &Value
    ) -> Result<Vec<Value>, StorageError>;
    
    /// 删除顶点所有索引
    fn delete_vertex_indexes(
        &self, 
        space_id: u64, 
        vertex_id: &Value
    ) -> Result<(), StorageError>;
}
```

### 4. 索引选择策略

**实现位置**: `src/query/planner/statements/seeks/`

**策略组件**:

| 组件 | 功能 | 说明 |
|-----|------|------|
| `IndexSeekPlanner` | 索引选择规划器 | 根据查询条件选择最优索引 |
| `PropIndexSeek` | 属性索引查找 | 单属性索引查找 |
| `VariablePropIndexSeek` | 变量属性索引 | 支持变量条件的索引查找 |
| `ScanSeek` | 扫描策略 | 无可用索引时使用全表扫描 |

**选择逻辑**:
```rust
pub struct IndexSeekPlanner;

impl IndexSeekPlanner {
    pub fn select_index(
        &self,
        space_id: u64,
        tag: &str,
        filters: &[FilterCondition],
        available_indexes: &[Index],
    ) -> Option<IndexSelection> {
        // 1. 筛选可用索引（匹配过滤条件的索引）
        let candidate_indexes: Vec<_> = available_indexes
            .iter()
            .filter(|idx| self.can_use_index(idx, filters))
            .collect();
        
        // 2. 选择最优索引（目前选择匹配字段最多的）
        candidate_indexes
            .into_iter()
            .max_by_key(|idx| idx.fields.len())
            .map(|idx| IndexSelection {
                index: idx.clone(),
                strategy: IndexScanStrategy::PointQuery,
            })
    }
}
```

## 可进一步改进的方向

### 1. 复合索引（Composite Index）

**当前状态**: 仅支持单字段索引

**需求场景**:
- 多条件联合查询（如 `WHERE age > 18 AND city = 'Beijing'`）
- 排序优化（索引天然有序）
- 覆盖索引（避免回表）

**建议实现**:

```rust
/// 复合索引定义
pub struct CompositeIndex {
    pub name: String,
    pub fields: Vec<IndexedField>,
    pub index_type: IndexType,
}

pub struct IndexedField {
    pub name: String,
    pub order: SortOrder,  // 升序/降序
}

pub enum SortOrder {
    Ascending,
    Descending,
}

/// 复合索引键构建
impl CompositeIndex {
    /// 构建复合索引键
    /// 格式: [space_id] [type] [index_name] [field1_value] [field2_value] ... [pk]
    pub fn build_key(
        &self,
        space_id: u64,
        field_values: &[Value],
        primary_key: &Value,
    ) -> Result<ByteKey, StorageError> {
        let mut key = Vec::new();
        key.extend_from_slice(&space_id.to_le_bytes());
        key.push(KEY_TYPE_COMPOSITE);
        key.extend_from_slice(&(self.name.len() as u32).to_le_bytes());
        key.extend_from_slice(self.name.as_bytes());
        
        // 按字段顺序编码值
        for (i, field) in self.fields.iter().enumerate() {
            let value = &field_values[i];
            let value_bytes = serialize_value(value)?;
            key.extend_from_slice(&(value_bytes.len() as u32).to_le_bytes());
            key.extend_from_slice(&value_bytes);
        }
        
        // 主键
        let pk_bytes = serialize_value(primary_key)?;
        key.extend_from_slice(&(pk_bytes.len() as u32).to_le_bytes());
        key.extend_from_slice(&pk_bytes);
        
        Ok(ByteKey(key))
    }
}
```

**最左前缀原则**:
```rust
/// 复合索引使用策略
pub enum CompositeIndexStrategy {
    /// 完全匹配所有字段
    FullMatch,
    /// 最左前缀匹配（部分字段）
    LeftmostPrefix { matched_fields: usize },
    /// 范围扫描（第一个字段是范围条件）
    RangeScan,
}

impl IndexSeekPlanner {
    /// 检查是否可以使用复合索引
    fn can_use_composite_index(
        &self,
        index: &CompositeIndex,
        filters: &[FilterCondition],
    ) -> Option<CompositeIndexStrategy> {
        let mut matched = 0;
        
        for (i, field) in index.fields.iter().enumerate() {
            if let Some(filter) = filters.iter().find(|f| f.field == field.name) {
                if i == matched {
                    matched += 1;
                } else {
                    break; // 不连续，无法使用
                }
            } else {
                break;
            }
        }
        
        if matched == index.fields.len() {
            Some(CompositeIndexStrategy::FullMatch)
        } else if matched > 0 {
            Some(CompositeIndexStrategy::LeftmostPrefix { matched_fields: matched })
        } else {
            None
        }
    }
}
```

**实现复杂度**: 中
**预期收益**: 高（多条件查询场景）

### 2. 覆盖索引（Covering Index）

**概念**: 索引包含查询所需的所有字段，避免回表查询

**建议实现**:

```rust
/// 覆盖索引定义
pub struct CoveringIndex {
    pub base: CompositeIndex,
    /// 包含字段（非索引字段，但存储在索引中）
    pub included_fields: Vec<String>,
}

/// 索引条目
pub struct IndexEntry {
    pub indexed_values: Vec<Value>,  // 索引字段值
    pub primary_key: Value,          // 主键
    pub included_values: Vec<Value>, // 包含字段值（覆盖索引使用）
}

impl CoveringIndex {
    /// 判断是否可以覆盖查询
    pub fn covers_query(&self, required_fields: &[String]) -> bool {
        // 检查所有需要的字段是否在索引中
        required_fields.iter().all(|field| {
            self.base.fields.iter().any(|f| &f.name == field) ||
            self.included_fields.iter().any(|f| f == field)
        })
    }
}

/// 查询优化器使用覆盖索引
impl IndexSeekPlanner {
    fn try_covering_index(
        &self,
        query: &Query,
        available_indexes: &[CoveringIndex],
    ) -> Option<IndexSelection> {
        let required_fields: Vec<_> = query.projection_fields();
        
        for index in available_indexes {
            if index.covers_query(&required_fields) {
                return Some(IndexSelection {
                    index: index.clone(),
                    strategy: IndexScanStrategy::CoveringIndex,
                    avoid_lookup: true, // 无需回表
                });
            }
        }
        None
    }
}
```

**收益**:
- 减少一次存储访问（无需根据主键查数据）
- 特别适合查询字段少的场景

**实现复杂度**: 中
**预期收益**: 中（特定查询场景）

### 3. 索引统计信息

**当前状态**: 无统计信息，优化器无法估算代价

**建议实现**:

```rust
/// 索引统计信息
pub struct IndexStatistics {
    pub index_name: String,
    pub space_id: u64,
    
    // 基本统计
    pub total_entries: u64,           // 总条目数
    pub unique_entries: u64,          // 唯一值数量
    pub avg_entries_per_key: f64,     // 每个键平均条目数
    
    // 选择性统计
    pub selectivity: f64,             // 选择性（0-1）
    pub null_fraction: f64,           // NULL 值比例
    
    // 分布统计
    pub value_distribution: Histogram, // 值分布直方图
    pub most_common_values: Vec<(Value, u64)>, // 最常见值
    
    // 物理统计
    pub index_size_bytes: usize,      // 索引大小
    pub index_height: u32,            // 索引树高度
    
    // 元数据
    pub last_analyzed: SystemTime,    // 上次分析时间
    pub sample_rows: u64,             // 采样行数
}

/// 直方图（用于估算范围查询选择性）
pub struct Histogram {
    pub buckets: Vec<Bucket>,
    pub bounds: Vec<Value>, // 桶边界
}

pub struct Bucket {
    pub count: u64,         // 桶内条目数
    pub distinct_values: u64, // 桶内不同值数量
}

/// 统计信息收集器
pub struct StatisticsCollector;

impl StatisticsCollector {
    /// 分析索引统计信息
    pub fn analyze_index(
        &self,
        storage: &dyn StorageClient,
        space_id: u64,
        index: &Index,
    ) -> Result<IndexStatistics, Error> {
        let mut stats = IndexStatistics::new(index.name.clone(), space_id);
        
        // 1. 收集基本统计
        let entries = self.collect_index_entries(storage, space_id, index)?;
        stats.total_entries = entries.len() as u64;
        
        // 2. 计算唯一值
        let unique_values: HashSet<_> = entries.iter().map(|e| &e.indexed_value).collect();
        stats.unique_entries = unique_values.len() as u64;
        
        // 3. 计算选择性
        stats.selectivity = if stats.total_entries > 0 {
            stats.unique_entries as f64 / stats.total_entries as f64
        } else {
            0.0
        };
        
        // 4. 构建直方图
        stats.value_distribution = self.build_histogram(&entries, 100)?;
        
        // 5. 收集最常见值
        stats.most_common_values = self.collect_most_common_values(&entries, 100)?;
        
        stats.last_analyzed = SystemTime::now();
        Ok(stats)
    }
    
    /// 估算查询选择性
    pub fn estimate_selectivity(
        &self,
        stats: &IndexStatistics,
        condition: &FilterCondition,
    ) -> f64 {
        match &condition.op {
            FilterOp::Eq => {
                // 等值查询选择性 ≈ 1 / 唯一值数量
                1.0 / stats.unique_entries as f64
            }
            FilterOp::Range { min, max } => {
                // 范围查询选择性使用直方图估算
                self.estimate_range_selectivity(&stats.value_distribution, min, max)
            }
            _ => 0.1, // 默认保守估计
        }
    }
}
```

**使用场景**:
- 优化器选择最优索引
- 估算查询返回行数
- 选择最优 Join 策略

**实现复杂度**: 中
**预期收益**: 高（查询优化基础）

### 4. 索引异步构建

**当前状态**: 同步构建，大数据量时阻塞

**建议实现**:

```rust
/// 异步索引构建器
pub struct AsyncIndexBuilder {
    db: Arc<Database>,
    task_queue: mpsc::Channel<IndexBuildTask>,
    workers: Vec<JoinHandle<()>>,
}

pub struct IndexBuildTask {
    pub space_id: u64,
    pub index: Index,
    pub priority: BuildPriority,
    pub callback: Option<Box<dyn FnOnce(BuildResult) + Send>>,
}

pub enum BuildPriority {
    High,    // 立即执行
    Normal,  // 排队执行
    Low,     // 后台执行
}

impl AsyncIndexBuilder {
    /// 提交索引构建任务
    pub async fn submit_task(&self, task: IndexBuildTask) -> Result<TaskId, Error> {
        let task_id = TaskId::new();
        
        // 设置索引状态为 Building
        self.set_index_status(task.space_id, &task.index.name, IndexStatus::Building)?;
        
        // 发送任务到队列
        self.task_queue.send(task).await?;
        
        Ok(task_id)
    }
    
    /// 工作线程处理任务
    async fn worker_loop(&self) {
        while let Ok(task) = self.task_queue.recv().await {
            let result = self.build_index(task.space_id, &task.index).await;
            
            // 更新索引状态
            let status = match &result {
                Ok(_) => IndexStatus::Active,
                Err(_) => IndexStatus::Error,
            };
            self.set_index_status(task.space_id, &task.index.name, status);
            
            // 执行回调
            if let Some(callback) = task.callback {
                callback(result);
            }
        }
    }
    
    /// 批量构建索引（分批提交，避免大事务）
    async fn build_index(
        &self,
        space_id: u64,
        index: &Index,
    ) -> Result<BuildResult, Error> {
        let batch_size = 10000;
        let mut total_processed = 0;
        
        // 扫描所有顶点
        let vertices = self.scan_all_vertices(space_id).await?;
        
        for batch in vertices.chunks(batch_size) {
            // 构建这批顶点的索引
            self.build_index_batch(space_id, index, batch).await?;
            total_processed += batch.len();
            
            // 定期提交，避免大事务
            if total_processed % (batch_size * 10) == 0 {
                self.commit_progress(space_id, index, total_processed).await?;
            }
        }
        
        Ok(BuildResult {
            total_entries: total_processed,
            duration: start.elapsed(),
        })
    }
}

/// 后台索引构建服务
pub struct IndexBuildService {
    builder: Arc<AsyncIndexBuilder>,
}

impl IndexBuildService {
    /// 启动后台服务
    pub async fn start(&self) {
        // 系统启动时恢复未完成的索引构建
        self.resume_pending_builds().await;
        
        // 启动工作线程
        self.builder.start_workers(4).await;
    }
    
    /// 恢复未完成的构建
    async fn resume_pending_builds(&self) {
        let pending_indexes = self.get_pending_indexes().await;
        for (space_id, index) in pending_indexes {
            self.builder.submit_task(IndexBuildTask {
                space_id,
                index,
                priority: BuildPriority::Normal,
                callback: None,
            }).await.ok();
        }
    }
}
```

**实现复杂度**: 中
**预期收益**: 中（大数据量场景）

### 5. 全文索引（Full-text Index）

**需求场景**:
- 文本内容搜索
- 模糊匹配查询
- 多关键词搜索

**建议实现**:

```rust
/// 全文索引
pub struct FullTextIndex {
    pub name: String,
    pub field: String,
    pub analyzer: TextAnalyzer,  // 分词器
    pub index: Arc<RwLock<InvertedIndex>>, // 倒排索引
}

/// 倒排索引
pub struct InvertedIndex {
    /// 词项 -> 文档列表
    postings: DashMap<String, Vec<DocId>>,
    /// 文档频率
    doc_freq: DashMap<String, u64>,
    /// 词项位置信息（用于短语查询）
    positions: DashMap<DocId, DashMap<String, Vec<u32>>>,
}

/// 分词器
pub enum TextAnalyzer {
    Standard,   // 标准分词（按空格和标点）
    CJK,        // 中日韩分词
    NGram(u32), // N-gram 分词
}

impl FullTextIndex {
    /// 索引文档
    pub fn index_document(&self, doc_id: DocId, text: &str) {
        let tokens = self.analyzer.tokenize(text);
        
        for (pos, token) in tokens.iter().enumerate() {
            // 添加到倒排列表
            self.index.postings
                .entry(token.clone())
                .or_insert_with(Vec::new)
                .push(doc_id);
            
            // 记录位置
            self.index.positions
                .entry(doc_id)
                .or_insert_with(DashMap::new)
                .entry(token.clone())
                .or_insert_with(Vec::new)
                .push(pos as u32);
        }
    }
    
    /// 搜索
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        let query_tokens = self.analyzer.tokenize(query);
        
        // 获取每个词项的文档列表
        let mut doc_lists: Vec<Vec<DocId>> = Vec::new();
        for token in &query_tokens {
            if let Some(docs) = self.index.postings.get(token) {
                doc_lists.push(docs.clone());
            }
        }
        
        // 取交集（AND 查询）
        let result_docs = intersect_doc_lists(&doc_lists);
        
        // 计算相关性评分
        self.score_results(&result_docs, &query_tokens)
    }
}
```

**实现复杂度**: 高
**预期收益**: 中（文本搜索场景）

## 总结

### 已实现的优势

1. **完整的索引架构** - 元数据管理 + 数据管理分离
2. **自动索引选择** - 查询优化器自动选择可用索引
3. **双向索引键** - 支持高效查找和删除
4. **多空间隔离** - 通过 space_id 实现数据隔离

### 建议优先级

| 优先级 | 优化项 | 预期收益 | 实现复杂度 |
|-------|--------|---------|-----------|
| P0 | 索引统计信息 | 高 | 中 |
| P1 | 复合索引 | 高 | 中 |
| P2 | 覆盖索引 | 中 | 中 |
| P3 | 异步索引构建 | 中 | 中 |
| P4 | 全文索引 | 中 | 高 |

**建议**: 优先实现索引统计信息，这是代价模型的基础，也是其他优化的前提。
