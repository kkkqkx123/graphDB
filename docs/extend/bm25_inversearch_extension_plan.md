# BM25 与 Inversearch 功能扩展方案

## 概述

**当前状态**：BM25 和 Inversearch 已经以嵌入式库方式集成到项目中，具备基础的索引和搜索能力。

**本文档重点**：分析需要扩展的查询功能、数据类型和语句设计，以支持完整的全文搜索能力。

---

## 一、现有功能分析

### 1.1 已实现功能

#### BM25 引擎（`crates/bm25/`）
- ✅ 基础索引操作（添加/删除文档）
- ✅ BM25 评分搜索
- ✅ 批量操作支持
- ✅ 字段权重配置
- ✅ 高亮功能
- ✅ gRPC 服务接口（可选特性）

#### Inversearch 引擎（`crates/inversearch/`）
- ✅ 倒排索引操作
- ✅ 多种分词模式（严格/正向/反向/双向）
- ✅ 上下文搜索
- ✅ 查询建议
- ✅ 结果高亮
- ✅ 多种存储后端（内存/文件/Redis/WAL）
- ✅ gRPC 服务接口（可选特性）

#### 搜索适配层（`src/search/`）
- ✅ `SearchEngine` trait 抽象
- ✅ `Bm25SearchEngine` 适配器
- ✅ `InversearchEngine` 适配器
- ✅ `FulltextIndexManager` 索引管理器
- ✅ 索引元数据管理
- ✅ `FulltextCoordinator` 协调器

### 1.2 缺失功能

#### 查询层
- ❌ 全文搜索 SQL 语法支持
- ❌ 全文搜索执行器
- ❌ 全文搜索表达式函数（`score()`, `highlight()`）
- ❌ 全文搜索执行计划生成

#### 索引管理
- ❌ 索引创建/删除的 SQL 支持
- ❌ 索引优化和维护工具
- ❌ 索引统计信息查询

#### 数据同步
- ❌ 自动索引同步机制
- ❌ 批量索引重建工具

---

## 二、数据类型设计

### 2.1 索引配置类型

#### 位置：`src/core/types/index.rs`（扩展现有）

```rust
/// 全文索引引擎类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FulltextEngineType {
    Bm25,
    Inversearch,
}

/// BM25 索引配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BM25IndexConfig {
    /// BM25 参数 k1 - 控制词频饱和度 (默认 1.2)
    pub k1: f32,
    /// BM25 参数 b - 控制长度归一化 (默认 0.75)
    pub b: f32,
    /// 字段权重配置
    pub field_weights: HashMap<String, f32>,
    /// 分词器配置
    pub analyzer: String,
    /// 是否存储原文
    pub store_original: bool,
}

impl Default for BM25IndexConfig {
    fn default() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            field_weights: HashMap::new(),
            analyzer: "standard".to_string(),
            store_original: true,
        }
    }
}

/// Inversearch 索引配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InversearchIndexConfig {
    /// 分词模式
    pub tokenize_mode: TokenizeMode,
    /// 分辨率 (默认 9)
    pub resolution: usize,
    /// 深度 (默认 3)
    pub depth: usize,
    /// 是否双向索引
    pub bidirectional: bool,
    /// 是否快速更新
    pub fast_update: bool,
    /// 字符集类型
    pub charset: CharsetType,
}

impl Default for InversearchIndexConfig {
    fn default() -> Self {
        Self {
            tokenize_mode: TokenizeMode::Bidirectional,
            resolution: 9,
            depth: 3,
            bidirectional: true,
            fast_update: true,
            charset: CharsetType::CJK,
        }
    }
}

/// 分词模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenizeMode {
    Strict,
    Forward,
    Reverse,
    Bidirectional,
    Full,
}

/// 字符集类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CharsetType {
    CJK,
    Latin,
    Exact,
    Normalized,
}

/// 全文索引字段配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextIndexField {
    pub field_name: String,
    pub field_type: PropertyType,
    pub analyzer: Option<String>,
    pub boost: f32,
    pub stored: bool,
    pub indexed: bool,
}

/// 全文索引配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextIndexOptions {
    pub engine_type: FulltextEngineType,
    pub bm25_config: Option<BM25IndexConfig>,
    pub inversearch_config: Option<InversearchIndexConfig>,
    pub fields: Vec<FulltextIndexField>,
    pub if_not_exists: bool,
}
```

### 2.2 查询类型

#### 位置：`src/core/types/query.rs`（新增）

```rust
/// 全文查询类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FulltextQuery {
    /// 简单文本查询
    Simple(String),
    /// 多字段查询
    MultiField(Vec<FieldQuery>),
    /// 布尔查询
    Boolean {
        must: Vec<FulltextQuery>,
        should: Vec<FulltextQuery>,
        must_not: Vec<FulltextQuery>,
        minimum_should_match: Option<usize>,
    },
    /// 短语查询
    Phrase {
        text: String,
        slop: u32,
    },
    /// 前缀查询
    Prefix {
        field: String,
        prefix: String,
    },
    /// 模糊查询
    Fuzzy {
        field: String,
        value: String,
        distance: u8,
        transpositions: bool,
    },
    /// 范围查询
    Range {
        field: String,
        lower: Option<String>,
        upper: Option<String>,
        include_lower: bool,
        include_upper: bool,
    },
    /// 通配符查询
    Wildcard {
        field: String,
        pattern: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldQuery {
    pub field: String,
    pub query: String,
    pub boost: f32,
}

/// 全文查询选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FulltextQueryOptions {
    /// 返回结果数量限制
    pub limit: usize,
    /// 偏移量
    pub offset: usize,
    /// 是否需要解释
    pub explain: bool,
    /// 高亮配置
    pub highlight: Option<HighlightOptions>,
    /// 排序配置
    pub sort: Vec<SortField>,
    /// 是否统计总数
    pub track_total_hits: bool,
    /// 最小分数阈值
    pub min_score: Option<f64>,
}

/// 高亮配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightOptions {
    /// 需要高亮的字段
    pub fields: Vec<String>,
    /// 高亮前缀
    pub pre_tag: String,
    /// 高亮后缀
    pub post_tag: String,
    /// 片段大小
    pub fragment_size: usize,
    /// 最大片段数
    pub num_fragments: usize,
    /// 编码器
    pub encoder: String,
    /// 边界检测器
    pub boundary_detector: String,
}

impl Default for HighlightOptions {
    fn default() -> Self {
        Self {
            fields: vec![],
            pre_tag: "<em>".to_string(),
            post_tag: "</em>".to_string(),
            fragment_size: 100,
            num_fragments: 3,
            encoder: "default".to_string(),
            boundary_detector: "sentence".to_string(),
        }
    }
}

/// 排序字段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortField {
    pub field: String,
    pub order: SortOrder,
    pub missing: SortMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortMissing {
    First,
    Last,
}
```

### 2.3 查询结果类型

#### 位置：`src/search/result.rs`（扩展现有）

```rust
/// 全文搜索结果
#[derive(Debug, Clone)]
pub struct FulltextSearchResult {
    /// 搜索结果列表
    pub results: Vec<SearchResultEntry>,
    /// 总命中数
    pub total_hits: usize,
    /// 最高分数
    pub max_score: f64,
    /// 耗时（毫秒）
    pub took_ms: u64,
    /// 是否超时
    pub timed_out: bool,
    /// 分片信息
    pub shards: Option<ShardsInfo>,
}

/// 搜索结果条目
#[derive(Debug, Clone)]
pub struct SearchResultEntry {
    /// 文档 ID
    pub doc_id: Value,
    /// 相关性分数
    pub score: f64,
    /// 高亮结果
    pub highlights: Option<HashMap<String, Vec<String>>>,
    /// 匹配的字段
    pub matched_fields: Vec<String>,
    /// 查询解释
    pub explanation: Option<QueryExplanation>,
    /// 排序值
    pub sort_values: Vec<Value>,
    /// 原始文档数据
    pub source: Option<HashMap<String, Value>>,
}

/// 查询解释
#[derive(Debug, Clone)]
pub struct QueryExplanation {
    pub value: f64,
    pub description: String,
    pub details: Vec<QueryExplanation>,
}

/// 分片信息
#[derive(Debug, Clone)]
pub struct ShardsInfo {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub failures: Vec<ShardFailure>,
}

#[derive(Debug, Clone)]
pub struct ShardFailure {
    pub shard: usize,
    pub index: String,
    pub reason: String,
}
```

---

## 三、查询语句设计

### 3.1 DDL 语句

#### 位置：`src/query/parser/ast/fulltext.rs`（扩展现有）

```rust
/// 创建全文索引语句
#[derive(Debug, Clone, PartialEq)]
pub struct CreateFulltextIndexStmt {
    pub span: Span,
    pub index_name: Identifier,
    pub tag_name: Identifier,
    pub fields: Vec<IndexFieldSpec>,
    pub engine_type: FulltextEngineType,
    pub if_not_exists: bool,
    pub options: FulltextIndexOptions,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexFieldSpec {
    pub field_name: Identifier,
    pub analyzer: Option<String>,
    pub boost: Option<f32>,
    pub stored: Option<bool>,
}

/// 删除全文索引语句
#[derive(Debug, Clone, PartialEq)]
pub struct DropFulltextIndexStmt {
    pub span: Span,
    pub index_name: Identifier,
    pub if_exists: bool,
}

/// 修改全文索引语句
#[derive(Debug, Clone, PartialEq)]
pub struct AlterFulltextIndexStmt {
    pub span: Span,
    pub index_name: Identifier,
    pub operation: AlterIndexOperation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlterIndexOperation {
    AddField(IndexFieldSpec),
    DropField(Identifier),
    UpdateOptions(FulltextIndexOptions),
    Rebuild,
    Optimize { max_segments: Option<usize> },
    Refresh,
}

/// 显示全文索引语句
#[derive(Debug, Clone, PartialEq)]
pub struct ShowFulltextIndexesStmt {
    pub span: Span,
    pub index_name: Option<Identifier>,
    pub tag_name: Option<Identifier>,
    pub show_status: bool,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
```

#### SQL 语法示例

```sql
-- 创建 BM25 索引
CREATE FULLTEXT INDEX idx_article_content 
ON article(title, content, tags)
ENGINE BM25
OPTIONS (
    k1 = 1.2,
    b = 0.75,
    analyzer = 'chinese',
    field_weights = {title: 2.0, content: 1.0}
);

-- 创建 Inversearch 索引
CREATE FULLTEXT INDEX idx_product_name
ON product(name, description)
ENGINE INVERSEARCH
OPTIONS (
    tokenize_mode = 'bidirectional',
    resolution = 9,
    charset = 'cjk'
);

-- 删除索引
DROP FULLTEXT INDEX idx_article_content;
DROP FULLTEXT INDEX IF EXISTS idx_product_name;

-- 修改索引
ALTER FULLTEXT INDEX idx_article_content ADD FIELD author;
ALTER FULLTEXT INDEX idx_article_content REBUILD;

-- 显示索引
SHOW FULLTEXT INDEXES;
SHOW FULLTEXT INDEXES ON article;
SHOW FULLTEXT INDEXES WITH STATUS;
```

### 3.2 DML 语句

#### 位置：`src/query/parser/ast/stmt.rs`（新增）

```rust
/// 全文搜索语句
#[derive(Debug, Clone, PartialEq)]
pub struct SearchStmt {
    pub span: Span,
    pub index_name: Identifier,
    pub query: FulltextQueryExpr,
    pub yield_clause: Option<YieldClause>,
    pub where_clause: Option<ContextualExpression>,
    pub order_clause: Option<OrderClause>,
    pub limit: Option<LimitClause>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FulltextQueryExpr {
    pub query_type: FulltextQueryType,
    pub query_text: String,
    pub fields: Vec<Identifier>,
    pub options: QueryOptionsMap,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FulltextQueryType {
    Match,
    PhraseMatch,
    PrefixMatch,
    FuzzyMatch,
    WildcardMatch,
    BooleanMatch,
}
```

#### 位置：`src/query/parser/ast/pattern.rs`（扩展）

```rust
/// MATCH 语句中的全文搜索条件
#[derive(Debug, Clone, PartialEq)]
pub struct FulltextMatchExpr {
    pub variable: Identifier,
    pub tag_name: Identifier,
    pub field_name: Identifier,
    pub query: String,
    pub options: Option<FulltextOptions>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FulltextOptions {
    pub boost: Option<f32>,
    pub highlight: Option<bool>,
    pub analyzer: Option<String>,
}
```

#### SQL 语法示例

```sql
-- 基本搜索语句
SEARCH INDEX idx_article_content 
MATCH '数据库优化'
YIELD doc_id, score, highlight(content);

-- 多字段搜索
SEARCH INDEX idx_article_content 
MATCH '数据库' IN title, content
YIELD doc_id, score;

-- 布尔搜索
SEARCH INDEX idx_article_content
BOOLEAN (MUST '数据库' AND SHOULD '优化' MUST_NOT '分布式')
YIELD doc_id, score;

-- 带过滤和排序的搜索
SEARCH INDEX idx_product_name
MATCH '智能手机'
WHERE price < 5000
YIELD doc_id, score, name, price
ORDER BY score DESC
LIMIT 10 OFFSET 0;

-- 在 MATCH 中使用全文搜索
MATCH (a:article)
WHERE FULLTEXT_MATCH(a.content, '数据库优化')
YIELD a, score() AS s
ORDER BY s DESC
LIMIT 10;

-- 带高亮的匹配
MATCH (p:product)
WHERE FULLTEXT_MATCH(p.description, '智能手机')
YIELD p, highlight(p.description) AS hl
ORDER BY score() DESC;

-- 在 GO 语句中使用全文搜索
GO FROM FULLTEXT_SEARCH(idx_article_content, '数据库') OVER like
YIELD $$.tag.name;
```

### 3.3 表达式函数

#### 位置：`src/query/parser/ast/expr.rs`（扩展）

```rust
/// 全文搜索函数
#[derive(Debug, Clone, PartialEq)]
pub enum FulltextFunction {
    /// 计算相关性分数
    Score(Option<Identifier>),
    
    /// 获取高亮片段
    Highlight {
        field: FieldReference,
        pre_tag: Option<String>,
        post_tag: Option<String>,
        fragment_size: Option<usize>,
    },
    
    /// 获取匹配的字段
    MatchedFields(Identifier),
    
    /// 获取查询解释
    Explain(Identifier),
}
```

#### SQL 语法示例

```sql
-- 分数函数
YIELD score() AS relevance;
YIELD score(a) AS article_score;

-- 高亮函数
YIELD highlight(content) AS hl_content;
YIELD highlight(content, '<em>', '</em>', 100) AS hl;

-- 匹配字段
YIELD matched_fields() AS fields;

-- 查询解释
YIELD explain() AS explanation;
```

---

## 四、执行器实现

### 4.1 全文搜索执行器

#### 位置：`src/query/executor/data_access/fulltext_search.rs`（新增）

```rust
use std::sync::Arc;
use parking_lot::Mutex;

use crate::core::{DataSet, Value, NullType};
use crate::query::executor::base::{
    BaseExecutor, Executor, ExecutorConfig, ExecutionResult, DBResult
};
use crate::search::{FulltextIndexManager, FulltextQuery, FulltextQueryOptions};
use crate::search::result::{FulltextSearchResult, SearchResultEntry};

/// 全文搜索执行器
pub struct FulltextSearchExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    index_name: String,
    query: FulltextQuery,
    options: FulltextQueryOptions,
    search_manager: Arc<FulltextIndexManager>,
    return_columns: Vec<String>,
}

impl<S: StorageClient> FulltextSearchExecutor<S> {
    pub fn new(
        base_config: ExecutorConfig<S>,
        index_name: String,
        query: FulltextQuery,
        options: FulltextQueryOptions,
        return_columns: Vec<String>,
        search_manager: Arc<FulltextIndexManager>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                base_config.id,
                "FulltextSearchExecutor".to_string(),
                base_config.storage,
                base_config.expr_context,
            ),
            index_name,
            query,
            options,
            return_columns,
            search_manager,
        }
    }
    
    fn parse_index_name(&self) -> DBResult<(u64, String, String)> {
        // 解析索引名称格式："space_id_tag_name_field_name"
        let parts: Vec<&str> = self.index_name.split('_').collect();
        if parts.len() < 4 {
            return Err(DBError::InvalidIndexName(self.index_name.clone()));
        }
        
        let space_id = parts[1].parse::<u64>()
            .map_err(|_| DBError::InvalidSpaceId(parts[1].to_string()))?;
        let tag_name = parts[2].to_string();
        let field_name = parts[3..].join("_");
        
        Ok((space_id, tag_name, field_name))
    }
    
    async fn execute_search(&self) -> DBResult<FulltextSearchResult> {
        let (space_id, tag_name, field_name) = self.parse_index_name()?;
        
        // 执行搜索
        let results = self.search_manager
            .search(space_id, &tag_name, &field_name, &self.query, &self.options)
            .await?;
        
        Ok(results)
    }
    
    fn convert_to_dataset(&self, result: FulltextSearchResult) -> DataSet {
        let mut dataset = DataSet::new();
        dataset.col_names = self.return_columns.clone();
        
        for entry in result.results {
            let mut row = Vec::new();
            for col in &self.return_columns {
                let value = self.extract_value(&entry, col);
                row.push(value);
            }
            dataset.rows.push(row);
        }
        
        dataset
    }
    
    fn extract_value(&self, entry: &SearchResultEntry, column: &str) -> Value {
        match column {
            "doc_id" => entry.doc_id.clone(),
            "score" => Value::Float(entry.score),
            "highlights" => {
                if let Some(hl) = &entry.highlights {
                    let hl_str = hl.values()
                        .flatten()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("...");
                    Value::String(hl_str)
                } else {
                    Value::Null(NullType::Null)
                }
            },
            "matched_fields" => {
                Value::List(entry.matched_fields
                    .iter()
                    .map(|s| Value::String(s.clone()))
                    .collect()
                )
            },
            _ => Value::Null(NullType::Null),
        }
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for FulltextSearchExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let result = futures::executor::block_on(self.execute_search())?;
        let dataset = self.convert_to_dataset(result);
        Ok(ExecutionResult::new_dataset(dataset))
    }
    
    // 其他 Executor trait 方法...
}
```

### 4.2 全文扫描执行器（用于 LOOKUP）

#### 位置：`src/query/executor/data_access/fulltext_scan.rs`（新增）

```rust
use std::sync::Arc;

use crate::core::{DataSet, Value};
use crate::query::executor::base::{
    BaseExecutor, Executor, ExecutorConfig, ExecutionResult, DBResult,
    FulltextScanConfig,
};
use crate::search::FulltextIndexManager;

/// 全文扫描执行器（用于 LOOKUP 语句）
pub struct FulltextScanExecutor<S: StorageClient + Send + 'static> {
    base: BaseExecutor<S>,
    space_id: u64,
    tag_id: i32,
    index_id: i32,
    index_name: String,
    query: FulltextQuery,
    options: FulltextQueryOptions,
    filter: Option<ContextualExpression>,
    return_columns: Vec<String>,
    limit: Option<usize>,
    search_manager: Arc<FulltextIndexManager>,
}

impl<S: StorageClient> FulltextScanExecutor<S> {
    pub fn new(
        base_config: ExecutorConfig<S>,
        scan_config: FulltextScanConfig,
        search_manager: Arc<FulltextIndexManager>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(
                base_config.id,
                "FulltextScanExecutor".to_string(),
                base_config.storage,
                base_config.expr_context,
            ),
            space_id: scan_config.space_id,
            tag_id: scan_config.tag_id,
            index_id: scan_config.index_id,
            index_name: scan_config.index_name,
            query: scan_config.query,
            options: scan_config.options,
            filter: scan_config.filter,
            return_columns: scan_config.return_columns,
            limit: scan_config.limit,
            search_manager: search_manager,
        }
    }
}
```

### 4.3 表达式求值器扩展

#### 位置：`src/query/executor/expression/functions/fulltext.rs`（新增）

```rust
use crate::query::executor::expression::functions::{Function, FunctionRegistry, FunctionSignature};
use crate::core::{Value, NullType, DataType};
use crate::query::executor::expression::evaluator::traits::ExpressionContext;

pub fn register_fulltext_functions(registry: &mut FunctionRegistry) {
    // 分数函数
    registry.register(
        "score",
        FulltextScoreFunction,
        FunctionSignature {
            name: "score",
            params: vec![],
            return_type: DataType::Float,
        },
    );
    
    // 高亮函数
    registry.register(
        "highlight",
        FulltextHighlightFunction,
        FunctionSignature {
            name: "highlight",
            params: vec![DataType::String],
            return_type: DataType::String,
        },
    );
    
    // 匹配字段函数
    registry.register(
        "matched_fields",
        FulltextMatchedFieldsFunction,
        FunctionSignature {
            name: "matched_fields",
            params: vec![],
            return_type: DataType::List(Box::new(DataType::String)),
        },
    );
}

/// 分数函数实现
pub struct FulltextScoreFunction;

impl Function for FulltextScoreFunction {
    fn evaluate(&self, ctx: &dyn ExpressionContext) -> DBResult<Value> {
        ctx.get_current_score()
            .map(Value::Float)
            .ok_or(DBError::Expression("No score available".into()))
    }
}

/// 高亮函数实现
pub struct FulltextHighlightFunction;

impl Function for FulltextHighlightFunction {
    fn evaluate(&self, ctx: &dyn ExpressionContext) -> DBResult<Value> {
        let field_name = ctx.get_param(0)?.as_string()?;
        
        if let Some(highlights) = ctx.get_current_highlights() {
            if let Some(field_hl) = highlights.get(&field_name) {
                return Ok(Value::String(field_hl.join("...")));
            }
        }
        
        Ok(Value::Null(NullType::Null))
    }
}

/// 匹配字段函数实现
pub struct FulltextMatchedFieldsFunction;

impl Function for FulltextMatchedFieldsFunction {
    fn evaluate(&self, ctx: &dyn ExpressionContext) -> DBResult<Value> {
        if let Some(fields) = ctx.get_current_matched_fields() {
            let list: Vec<Value> = fields
                .iter()
                .map(|s| Value::String(s.clone()))
                .collect();
            Ok(Value::List(list))
        } else {
            Ok(Value::Null(NullType::Null))
        }
    }
}
```

---

## 五、执行计划生成

### 5.1 全文搜索计划生成器

#### 位置：`src/query/planning/planner/fulltext.rs`（新增）

```rust
use crate::query::planning::plan::ExecutionPlan;
use crate::query::planning::planner::Planner;
use crate::query::parser::ast::{SearchStmt, FulltextQueryExpr, FulltextQueryType};
use crate::core::error::DBResult;

impl Planner {
    /// 为全文搜索语句生成执行计划
    pub fn plan_search_stmt(&self, stmt: &SearchStmt) -> DBResult<ExecutionPlan> {
        // 1. 验证索引存在
        let index_metadata = self.get_index_metadata(&stmt.index_name)?;
        
        // 2. 解析查询表达式
        let query = self.parse_fulltext_query(&stmt.query)?;
        
        // 3. 构建搜索配置
        let search_config = FulltextScanConfig {
            space_id: index_metadata.space_id,
            index_id: index_metadata.index_id,
            index_name: stmt.index_name.to_string(),
            query,
            options: self.build_query_options(&stmt.query.options)?,
            return_columns: self.extract_yield_columns(&stmt.yield_clause)?,
            limit: stmt.limit.as_ref().map(|l| l.count),
        };
        
        // 4. 创建全文搜索执行器
        let search_executor = FulltextSearchExecutor::new(
            ExecutorConfig {
                id: self.next_id(),
                storage: self.storage.clone(),
                expr_context: self.expr_context.clone(),
            },
            search_config.index_name,
            search_config.query,
            search_config.options,
            search_config.return_columns,
            self.search_manager.clone(),
        );
        
        // 5. 构建执行计划
        let mut plan = ExecutionPlan::new();
        plan.add_root(Box::new(search_executor));
        
        // 6. 添加过滤执行器（如果有 WHERE 子句）
        if let Some(where_clause) = &stmt.where_clause {
            let filter_executor = FilterExecutor::new(
                self.next_id(),
                self.storage.clone(),
                where_clause.clone(),
            );
            plan.add_child(plan.root_id().unwrap(), Box::new(filter_executor));
        }
        
        // 7. 添加排序执行器（如果有 ORDER BY 子句）
        if let Some(order_clause) = &stmt.order_clause {
            let sort_executor = SortExecutor::new(
                self.next_id(),
                self.storage.clone(),
                order_clause.clone(),
            );
            plan.add_child(plan.root_id().unwrap(), Box::new(sort_executor));
        }
        
        Ok(plan)
    }
    
    /// 解析全文查询表达式
    fn parse_fulltext_query(&self, expr: &FulltextQueryExpr) -> DBResult<FulltextQuery> {
        match expr.query_type {
            FulltextQueryType::Match => {
                if expr.fields.is_empty() {
                    Ok(FulltextQuery::Simple(expr.query_text.clone()))
                } else {
                    let field_queries = expr.fields
                        .iter()
                        .map(|f| FieldQuery {
                            field: f.to_string(),
                            query: expr.query_text.clone(),
                            boost: 1.0,
                        })
                        .collect();
                    Ok(FulltextQuery::MultiField(field_queries))
                }
            },
            FulltextQueryType::BooleanMatch => {
                // 解析布尔查询语法
                self.parse_boolean_query(&expr.query_text)
            },
            // 其他查询类型...
            _ => Ok(FulltextQuery::Simple(expr.query_text.clone())),
        }
    }
}
```

---

## 六、解析器扩展

### 6.1 全文搜索解析器

#### 位置：`src/query/parser/parsing/fulltext_parser.rs`（新增）

```rust
use crate::query::parser::ast::*;
use crate::query::parser::parsing::parse_context::ParseContext;
use crate::query::parser::core::TokenKind;

pub fn parse_create_fulltext_index(ctx: &mut ParseContext) -> DBResult<CreateFulltextIndexStmt> {
    let span = ctx.current_span();
    
    // 解析 INDEX 关键字
    ctx.expect(TokenKind::Keyword("INDEX".to_string()))?;
    
    // 解析索引名称
    let index_name = parse_identifier(ctx)?;
    
    // 解析 ON 关键字
    ctx.expect(TokenKind::Keyword("ON".to_string()))?;
    
    // 解析标签名称
    let tag_name = parse_identifier(ctx)?;
    
    // 解析字段列表
    ctx.expect(TokenKind::LeftParen)?;
    let mut fields = Vec::new();
    loop {
        let field = parse_index_field_spec(ctx)?;
        fields.push(field);
        
        if !ctx.match_token(TokenKind::Comma) {
            break;
        }
    }
    ctx.expect(TokenKind::RightParen)?;
    
    // 解析 ENGINE 子句
    ctx.expect(TokenKind::Keyword("ENGINE".to_string()))?;
    let engine_type = parse_engine_type(ctx)?;
    
    // 解析 OPTIONS 子句
    let options = if ctx.match_token(TokenKind::Keyword("OPTIONS".to_string())) {
        parse_fulltext_options(ctx)?
    } else {
        FulltextIndexOptions::default()
    };
    
    Ok(CreateFulltextIndexStmt {
        span,
        index_name,
        tag_name,
        fields,
        engine_type,
        if_not_exists: false,
        options,
    })
}

pub fn parse_search_stmt(ctx: &mut ParseContext) -> DBResult<SearchStmt> {
    let span = ctx.current_span();
    
    // 解析 INDEX 关键字
    ctx.expect(TokenKind::Keyword("INDEX".to_string()))?;
    
    // 解析索引名称
    let index_name = parse_identifier(ctx)?;
    
    // 解析查询类型和文本
    let query = parse_fulltext_query_expr(ctx)?;
    
    // 解析 YIELD 子句
    let yield_clause = if ctx.match_token(TokenKind::Keyword("YIELD".to_string())) {
        Some(parse_yield_clause(ctx)?)
    } else {
        None
    };
    
    // 解析 WHERE 子句
    let where_clause = if ctx.match_token(TokenKind::Keyword("WHERE".to_string())) {
        Some(parse_contextual_expression(ctx)?)
    } else {
        None
    };
    
    // 解析 ORDER BY 子句
    let order_clause = if ctx.match_token(TokenKind::Keyword("ORDER".to_string())) {
        Some(parse_order_clause(ctx)?)
    } else {
        None
    };
    
    // 解析 LIMIT 子句
    let limit = if ctx.match_token(TokenKind::Keyword("LIMIT".to_string())) {
        Some(parse_limit_clause(ctx)?)
    } else {
        None
    };
    
    Ok(SearchStmt {
        span,
        index_name,
        query,
        yield_clause,
        where_clause,
        order_clause,
        limit,
    })
}
```

---

## 七、实施步骤

### Phase 1: 数据类型定义（1-2 周）
1. 扩展 `src/core/types/index.rs` - 添加索引配置类型
2. 创建 `src/core/types/query.rs` - 添加查询类型
3. 扩展 `src/search/result.rs` - 添加搜索结果类型

### Phase 2: AST 和解析器（2-3 周）
1. 扩展 `src/query/parser/ast/fulltext.rs` - DDL 语句
2. 扩展 `src/query/parser/ast/stmt.rs` - DML 语句
3. 创建 `src/query/parser/parsing/fulltext_parser.rs` - 解析器实现

### Phase 3: 执行器实现（3-4 周）
1. 创建 `src/query/executor/data_access/fulltext_search.rs`
2. 创建 `src/query/executor/data_access/fulltext_scan.rs`
3. 创建 `src/query/executor/expression/functions/fulltext.rs`

### Phase 4: 执行计划生成（2-3 周）
1. 创建 `src/query/planning/planner/fulltext.rs`
2. 集成到现有的计划生成流程
3. 优化执行计划

### Phase 5: 验证和测试（2-3 周）
1. 创建验证器 `src/query/validator/fulltext_validator.rs`
2. 编写单元测试
3. 编写集成测试
4. 性能基准测试

---

## 八、技术要点

### 8.1 索引名称规范
```
格式：idx_{space_id}_{tag_name}_{field_name}
示例：idx_1_article_content
```

### 8.2 查询优化建议
1. **缓存查询结果** - 对高频查询使用缓存
2. **批量索引操作** - 合并多个索引操作
3. **异步索引更新** - 使用队列异步处理索引更新
4. **增量索引构建** - 支持增量重建索引

### 8.3 错误处理
- 索引不存在
- 查询语法错误
- 分词失败
- 存储空间不足

---

## 九、参考文档

- [BM25 模块分析](../../../crates/bm25/docs/MODULE_ANALYSIS.md)
- [Inversearch 功能分析](../../../crates/inversearch/docs/功能分析.md)
- [Phase 4 数据同步机制](./plan/phase4_data_sync_mechanism.md)
