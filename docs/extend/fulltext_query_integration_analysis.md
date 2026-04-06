# 全文索引功能查询工作流集成分析报告

**文档版本**: 1.0  
**创建日期**: 2026-04-04  
**状态**: 分析完成，待实施

---

## 一、执行摘要

本文档全面分析了 GraphDB 全文索引功能在查询工作流中的集成现状，识别出架构设计完整但执行层缺失的关键问题，并提出了分阶段的集成实施方案。

### 核心发现

- ✅ **架构设计完整**：四层架构（搜索引擎层、协调器层、查询引擎层、SQL 语法层）清晰合理
- ✅ **基础设施完备**：SearchEngine Trait、FulltextIndexManager、FulltextCoordinator 均已实现
- ✅ **语法支持完整**：AST 定义、Parser、Validator 都已定义，支持丰富的全文搜索语法
- ❌ **执行器实现缺失**：所有全文搜索执行器都是占位符，返回空结果
- ❌ **集成链路断裂**：查询引擎无法实际调用 Coordinator 的搜索功能
- ❌ **数据同步非自动**：需要上层显式调用，未与存储层自动集成

### 关键风险

当前全文索引功能**无法在实际查询中使用**，所有执行器的 `execute()` 方法都返回 `ExecutionResult::Empty`，导致从 SQL 语法到搜索引擎的完整链路在执行器层中断。

---

## 二、架构概览

### 2.1 四层架构设计

```
┌─────────────────────────────────────────────────────────┐
│                    查询语言层 (SQL/nGQL)                 │
│  CREATE FULLTEXT INDEX, SEARCH, MATCH ... WHERE MATCH   │
│  LOOKUP FULLTEXT, FULLTEXT_MATCH()                      │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                    查询引擎层                            │
│  Parser → Validator → Planner → Executor               │
│  (AST 定义完整，执行器为占位符)                           │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                  协调器层 (Coordinator)                  │
│  FulltextCoordinator - 索引管理和数据同步协调           │
│  (实现完整，但需要显式调用)                              │
└─────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────┐
│                  搜索引擎层 (SearchEngine)               │
│  FulltextIndexManager + BM25/Inversearch 引擎适配器     │
│  (实现完整，支持多引擎插拔)                              │
└─────────────────────────────────────────────────────────┘
```

### 2.2 核心组件清单

#### 搜索引擎层（已完成 ✅）

| 文件路径 | 组件 | 状态 |
|---------|------|------|
| `src/search/engine.rs` | SearchEngine Trait | ✅ 完成 |
| `src/search/manager.rs` | FulltextIndexManager | ✅ 完成 |
| `src/search/factory.rs` | SearchEngineFactory | ✅ 完成 |
| `src/search/metadata.rs` | IndexMetadata | ✅ 完成 |
| `src/search/config.rs` | FulltextConfig | ✅ 完成 |
| `src/search/result.rs` | SearchResult | ✅ 完成 |
| `src/search/error.rs` | SearchError | ✅ 完成 |
| `crates/bm25/src/` | BM25 引擎实现 | ✅ 完成 |
| `crates/inversearch/src/` | Inversearch 引擎实现 | ✅ 完成 |

#### 协调器层（已完成 ✅）

| 文件路径 | 组件 | 状态 |
|---------|------|------|
| `src/coordinator/fulltext.rs` | FulltextCoordinator | ✅ 完成 |
| `src/coordinator/types.rs` | Coordinator 类型定义 | ✅ 完成 |

#### 查询引擎层（部分完成 ⚠️）

| 文件路径 | 组件 | 状态 | 说明 |
|---------|------|------|------|
| `src/query/parser/ast/fulltext.rs` | AST 定义 | ✅ 完成 | 完整的语法树节点 |
| `src/query/parser/parsing/fulltext_parser.rs` | Parser | ✅ 完成 | 支持所有语法 |
| `src/query/validator/fulltext_validator.rs` | Validator | ✅ 完成 | 语义验证 |
| `src/query/planning/plan/core/nodes/management/fulltext_nodes.rs` | Planner 节点 | ✅ 完成 | 计划节点定义 |
| `src/query/executor/factory/executor_factory.rs` | Executor Factory | ✅ 完成 | 执行器创建逻辑 |
| `src/query/executor/data_access/fulltext_search.rs` | **执行器** | ❌ **占位符** | **execute() 返回空结果** |
| `src/query/executor/data_access/match_fulltext.rs` | **执行器** | ❌ **占位符** | **execute() 返回空结果** |
| `src/query/executor/expression/functions/fulltext.rs` | 表达式函数 | ✅ 完成 | score/highlight 等 |

---

## 三、详细集成分析

### 3.1 搜索引擎层集成

#### 核心接口

```rust
// src/search/engine.rs
pub trait SearchEngine: Send + Sync {
    /// 索引文档
    async fn index(&self, doc_id: &str, content: &str) -> Result<(), SearchError>;
    
    /// 删除文档
    async fn delete(&self, doc_id: &str) -> Result<(), SearchError>;
    
    /// 搜索
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError>;
    
    /// 提交索引更改
    async fn commit(&self) -> Result<(), SearchError>;
    
    /// 关闭引擎
    async fn close(&self) -> Result<(), SearchError>;
}
```

#### 索引管理器

```rust
// src/search/manager.rs
pub struct FulltextIndexManager {
    engines: DashMap<IndexKey, Arc<dyn SearchEngine>>,
    metadata: DashMap<IndexKey, IndexMetadata>,
    base_path: PathBuf,
    default_engine: EngineType,
    config: FulltextConfig,
}

impl FulltextIndexManager {
    pub async fn create_index(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        engine_type: Option<EngineType>,
    ) -> Result<String, SearchError>
    
    pub fn get_engine(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
    ) -> Option<Arc<dyn SearchEngine>>
    
    pub async fn search(
        &self,
        space_id: u64,
        tag_name: &str,
        field_name: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, SearchError>
}
```

**集成特点**：
- ✅ 通过 Trait 抽象，支持多引擎插拔（BM25、Inversearch）
- ✅ 使用 `DashMap` 实现线程安全的索引管理
- ✅ 支持索引元数据持久化
- ✅ 提供统一的搜索接口

---

### 3.2 协调器层集成

#### FulltextCoordinator 核心方法

```rust
// src/coordinator/fulltext.rs
pub struct FulltextCoordinator {
    manager: Arc<FulltextIndexManager>,
}

impl FulltextCoordinator {
    // 索引管理
    pub async fn create_index(...) -> Result<String, SearchError>
    pub async fn drop_index(...) -> Result<(), SearchError>
    pub async fn rebuild_index(...) -> Result<(), SearchError>
    pub fn list_indexes() -> Vec<IndexMetadata>
    
    // 数据同步（需要显式调用）
    pub async fn on_vertex_inserted(...) -> Result<(), SearchError>
    pub async fn on_vertex_updated(...) -> Result<(), SearchError>
    pub async fn on_vertex_deleted(...) -> Result<(), SearchError>
    pub async fn on_vertex_change(...) -> Result<(), SearchError>
    
    // 搜索
    pub async fn search(...) -> Result<Vec<SearchResult>, SearchError>
    pub async fn commit_all() -> Result<(), SearchError>
}
```

**数据同步流程**：
```
存储层 (Redb) 事务提交
    ↓
查询引擎/业务层调用 Coordinator
    ↓
FulltextCoordinator::on_vertex_inserted()
    ↓
遍历顶点的所有 Tag 和属性
    ↓
检查字段是否有索引 → 调用 engine.index()
    ↓
异步完成（不阻塞主流程）
```

**集成特点**：
- ✅ 位于程序层，与存储层解耦
- ✅ 支持异步非阻塞同步
- ✅ 最终一致性模型
- ⚠️ **依赖上层显式调用**，非自动触发

---

### 3.3 查询引擎层集成

#### 3.3.1 AST 定义（完整）

支持的语句类型：
```rust
// src/query/parser/ast/fulltext.rs
pub enum Stmt {
    CreateFulltextIndex(CreateFulltextIndex),
    DropFulltextIndex(DropFulltextIndex),
    AlterFulltextIndex(AlterFulltextIndex),
    ShowFulltextIndex(ShowFulltextIndex),
    DescribeFulltextIndex(DescribeFulltextIndex),
    Search(SearchStatement),              // SEARCH 语句
    LookupFulltext(LookupFulltext),       // LOOKUP FULLTEXT
    MatchFulltext(MatchFulltext),         // MATCH with fulltext
}
```

查询表达式类型（支持 9 种查询）：
```rust
pub enum FulltextQueryExpr {
    Simple(String),                      // MATCH 'database'
    Field(String, String),               // title:'database'
    MultiField(Vec<(String, String)>),   // 多字段搜索
    Boolean {                            // 布尔查询
        must: Vec<FulltextQueryExpr>,
        should: Vec<FulltextQueryExpr>,
        must_not: Vec<FulltextQueryExpr>,
    },
    Phrase(String),                      // "database optimization"
    Prefix(String),                      // data*
    Fuzzy(String, Option<u8>),           // database~
    Range { ... },                       // [2020 TO 2023]
    Wildcard(String),                    // data*ase
}
```

#### 3.3.2 支持的 SQL 语法

```sql
-- 1. 创建全文索引
CREATE FULLTEXT INDEX idx_article_content 
ON article(title, content)
ENGINE BM25
OPTIONS (k1 = 1.2, b = 0.75);

-- 2. SEARCH 语句
SEARCH INDEX idx_article_content 
MATCH 'database optimization'
YIELD doc_id, score() AS s, highlight(content)
WHERE score > 0.5
ORDER BY s DESC
LIMIT 10;

-- 3. LOOKUP FULLTEXT
LOOKUP ON article INDEX idx_content 
WHERE 'database'
YIELD doc_id, score()
LIMIT 20;

-- 4. MATCH with fulltext
MATCH (a:article)
WHERE FULLTEXT_MATCH(a.content, 'database')
YIELD a, score() AS s;

-- 5. 带评分排序的 MATCH
MATCH (p:Post)
WHERE p.content MATCH "图数据库"
RETURN p, score(p) as relevance
ORDER BY relevance DESC
LIMIT 10;
```

#### 3.3.3 计划节点（完整）

```rust
// src/query/planning/plan/core/nodes/management/fulltext_nodes.rs
pub struct FulltextSearchNode {     // SEARCH 语句
    id: i64,
    pub index_name: String,
    pub query: FulltextQueryExpr,
    pub yield_clause: Option<FulltextYieldClause>,
    pub where_clause: Option<WhereClause>,
    pub order_clause: Option<OrderClause>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub struct FulltextLookupNode {     // LOOKUP FULLTEXT
    id: i64,
    pub schema_name: String,
    pub index_name: String,
    pub query: String,
    pub yield_clause: Option<FulltextYieldClause>,
    pub limit: Option<usize>,
}

pub struct MatchFulltextNode {      // MATCH with fulltext
    pub pattern: String,
    pub fulltext_condition: FulltextMatchCondition,
    pub yield_clause: Option<FulltextYieldClause>,
}
```

#### 3.3.4 执行器（❌ 占位符 - 关键问题）

**当前实现**：
```rust
// src/query/executor/data_access/fulltext_search.rs
pub struct FulltextSearchExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    statement: SearchStatement,
    engine: Arc<dyn SearchEngine>,
    context: ExecutionContext,
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient> Executor<S> for FulltextSearchExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // ❌ 当前返回空结果
        Ok(ExecutionResult::Empty)
    }
}

// 同样的问题存在于 FulltextScanExecutor 和 MatchFulltextExecutor
```

**执行器创建逻辑**（完整）：
```rust
// src/query/executor/factory/executor_factory.rs
fn build_fulltext_search(
    &mut self,
    node: &FulltextSearchNode,
    storage: Arc<Mutex<S>>,
    context: &ExecutionContext,
) -> Result<ExecutorEnum<S>, QueryError> {
    let statement = SearchStatement { ... };
    let search_engine = context.search_engine()
        .ok_or_else(|| QueryError::ExecutionError("Search engine not available"))?
        .clone();
    
    let executor = FulltextSearchExecutor::new(
        node.id(),
        statement,
        search_engine,
        context.clone(),
        storage,
        context.expression_context().clone(),
    );
    Ok(ExecutorEnum::FulltextSearch(executor))
}
```

#### 3.3.5 表达式函数（完整但未集成）

```rust
// src/query/executor/expression/functions/fulltext.rs
pub enum FulltextFunction {
    Score,          // score() - 获取相关性评分
    Highlight,      // highlight(field) - 高亮显示
    MatchedFields,  // matched_fields() - 获取匹配字段列表
    Snippet,        // snippet(field, max_len) - 获取文本片段
}

pub struct FulltextExecutionContext {
    pub score: f64,
    pub highlights: HashMap<String, String>,
    pub matched_fields: Vec<String>,
    pub snippets: HashMap<String, String>,
}
```

**问题**：函数实现完整，但**缺少与执行器的集成**，无法在实际查询中使用。

---

### 3.4 验证器（完整）

```rust
// src/query/validator/fulltext_validator.rs
pub struct FulltextValidator;

impl FulltextValidator {
    fn validate_create(&self, stmt: &CreateFulltextIndex) -> Result<ValidationInfo, ValidationError>
    fn validate_search(&self, stmt: &SearchStatement) -> Result<ValidationInfo, ValidationError>
    fn validate_match(&self, stmt: &MatchFulltext) -> Result<ValidationInfo, ValidationError>
    // ... 其他验证方法
}
```

---

## 四、集成缺口分析

### 4.1 核心缺口

#### 缺口 1：执行器实现缺失（❌ 严重）

**问题描述**：所有全文搜索执行器的 `execute()` 方法都返回 `ExecutionResult::Empty`，导致查询无法执行。

**影响范围**：
- `FulltextSearchExecutor` - SEARCH 语句无法执行
- `FulltextScanExecutor` - LOOKUP FULLTEXT 无法执行
- `MatchFulltextExecutor` - MATCH with fulltext 无法执行

**缺失内容**：
1. 索引名称解析逻辑
2. 查询表达式转换（AST -> FulltextQuery）
3. 调用 Coordinator 执行搜索
4. 根据 doc_ids 获取完整顶点数据
5. 处理 YIELD 子句
6. 应用 WHERE 过滤
7. 应用 ORDER BY 排序
8. 应用 LIMIT/OFFSET 分页

**需要的实现**：
```rust
impl<S: StorageClient> Executor<S> for FulltextSearchExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 1. 解析索引名称，获取 space_id, tag_name, field_name
        let (space_id, tag_name, field_name) = self.parse_index_name()?;
        
        // 2. 转换查询表达式 (FulltextQueryExpr -> FulltextQuery)
        let query = self.convert_query(&self.statement.query)?;
        
        // 3. 通过 Coordinator 执行搜索
        let search_results = self.coordinator.search(
            space_id, &tag_name, &field_name, &query, limit
        ).await?;
        
        // 4. 根据 doc_ids 获取完整顶点数据
        let mut rows = Vec::new();
        for result in search_results {
            if let Some(vertex) = self.storage.get_vertex_by_id(&result.doc_id).await? {
                let mut row = HashMap::new();
                
                // 5. 处理 YIELD 子句
                for yield_item in &self.statement.yield_clause.items {
                    match yield_item.expr {
                        YieldExpression::Field(name) => {
                            row.insert(name, vertex.get_property(&name));
                        }
                        YieldExpression::Score(_) => {
                            row.insert("score".to_string(), Value::Float(result.score));
                        }
                        YieldExpression::Highlight(field, ..) => {
                            // 调用高亮函数
                        }
                        _ => {}
                    }
                }
                rows.push(row);
            }
        }
        
        // 6. 应用 WHERE 过滤
        if let Some(where_clause) = &self.statement.where_clause {
            rows = self.apply_filter(rows, where_clause)?;
        }
        
        // 7. 应用 ORDER BY
        if let Some(order_clause) = &self.statement.order_clause {
            rows = self.apply_sort(rows, order_clause)?;
        }
        
        // 8. 应用 LIMIT/OFFSET
        rows = self.apply_pagination(rows, self.statement.limit, self.statement.offset)?;
        
        Ok(ExecutionResult::Rows(rows))
    }
}
```

---

#### 缺口 2：执行上下文传递缺失（❌ 严重）

**问题描述**：全文搜索函数（score, highlight）需要 `FulltextExecutionContext`，但当前执行器没有创建和传递上下文。

**影响范围**：
- `score()` 函数无法获取文档评分
- `highlight()` 函数无法获取高亮内容
- `matched_fields()` 函数无法获取匹配字段列表
- `snippet()` 函数无法获取文本片段

**需要的实现**：
```rust
// 在执行器中创建上下文
let mut ft_context = FulltextExecutionContext {
    score: result.score,
    highlights: HashMap::new(),
    matched_fields: vec![],
    snippets: HashMap::new(),
};

// 在表达式求值时传递
let value = expression.evaluate_with_fulltext_context(&ft_context)?;
```

---

#### 缺口 3：查询表达式转换缺失（❌ 严重）

**问题描述**：需要将 AST 的 `FulltextQueryExpr` 转换为搜索引擎的 `FulltextQuery`。

**影响范围**：所有全文搜索查询类型

**需要的实现**：
```rust
fn convert_query(&self, expr: &FulltextQueryExpr) -> Result<FulltextQuery, QueryError> {
    match expr {
        FulltextQueryExpr::Simple(text) => Ok(FulltextQuery::Simple(text.clone())),
        FulltextQueryExpr::Field(field, text) => {
            Ok(FulltextQuery::Field(field.clone(), text.clone()))
        }
        FulltextQueryExpr::Boolean { must, should, must_not } => {
            Ok(FulltextQuery::Boolean {
                must: must.iter().map(|e| self.convert_query(e)).collect::<Result<_, _>>()?,
                should: should.iter().map(|e| self.convert_query(e)).collect::<Result<_, _>>()?,
                must_not: must_not.iter().map(|e| self.convert_query(e)).collect::<Result<_, _>>()?,
            })
        }
        FulltextQueryExpr::Phrase(text) => Ok(FulltextQuery::Phrase(text.clone())),
        FulltextQueryExpr::Prefix(text) => Ok(FulltextQuery::Prefix(text.clone())),
        FulltextQueryExpr::Fuzzy(text, distance) => {
            Ok(FulltextQuery::Fuzzy(text.clone(), *distance))
        }
        // ... 其他类型
    }
}
```

---

#### 缺口 4：与存储层集成缺失（⚠️ 中等）

**问题描述**：当前 `FulltextCoordinator` 的数据同步方法需要上层显式调用，没有自动集成到存储层的插入/更新/删除操作中。

**影响范围**：数据变更不会自动同步到全文索引

**需要的实现**：
```rust
// 在 RedbStorage 的插入/更新/删除操作中调用
pub async fn insert_vertex(&self, vertex: &Vertex) -> Result<(), StorageError> {
    // 1. 写入存储
    self.write_vertex(vertex).await?;
    
    // 2. 同步到全文索引（如果存在）
    if let Some(coordinator) = &self.fulltext_coordinator {
        coordinator.on_vertex_inserted(space_id, vertex).await?;
    }
    
    Ok(())
}
```

---

### 4.2 次要缺口

#### 缺口 5：结果转换和投影缺失

- YIELD 子句字段选择逻辑未实现
- score(), highlight() 等函数的结果处理未实现
- 结果列名映射未实现

#### 缺口 6：过滤和排序未实现

- WHERE 子句过滤逻辑未实现
- ORDER BY 排序逻辑未实现
- LIMIT/OFFSET 分页逻辑未实现

#### 缺口 7：错误处理不完善

- 索引不存在时的错误处理
- 查询语法错误的错误处理
- 搜索引擎故障的错误处理

---

## 五、集成状态总结

### 5.1 已完成组件（✅）

| 层级 | 组件 | 完成度 | 说明 |
|------|------|--------|------|
| **搜索引擎层** | SearchEngine Trait | 100% | 标准接口定义 |
| | FulltextIndexManager | 100% | 索引生命周期管理 |
| | BM25 引擎 | 100% | 基于 Tantivy |
| | Inversearch 引擎 | 100% | 独立 crate |
| | 元数据管理 | 100% | IndexMetadata |
| **协调器层** | FulltextCoordinator | 100% | 索引操作 API |
| | 数据同步方法 | 100% | on_vertex_* 方法族 |
| **查询引擎层** | AST 定义 | 100% | 完整的语法树节点 |
| | Parser | 100% | 支持所有语法 |
| | Validator | 100% | 语义验证 |
| | Planner 节点 | 100% | 计划节点定义 |
| | Executor Factory | 100% | 执行器创建逻辑 |
| | 表达式函数 | 100% | score/highlight 等 |

### 5.2 未完成组件（❌）

| 组件 | 完成度 | 问题 | 优先级 |
|------|--------|------|--------|
| **FulltextSearchExecutor** | 10% | execute() 返回空结果 | P0 |
| **FulltextScanExecutor** | 10% | execute() 返回空结果 | P0 |
| **MatchFulltextExecutor** | 10% | execute() 返回空结果 | P0 |
| **查询表达式转换** | 0% | 未实现 | P0 |
| **执行上下文传递** | 0% | 未实现 | P0 |
| **结果转换和投影** | 0% | 未实现 | P1 |
| **WHERE 过滤** | 0% | 未实现 | P1 |
| **ORDER BY 排序** | 0% | 未实现 | P1 |
| **LIMIT/OFFSET** | 0% | 未实现 | P1 |
| **存储层自动集成** | 0% | 未实现 | P2 |

---

## 六、风险评估

### 6.1 技术风险

| 风险 | 影响 | 可能性 | 缓解措施 |
|------|------|--------|----------|
| 执行器实现复杂度高 | 高 | 高 | 分阶段实施，先实现基本功能 |
| 查询性能不达标 | 中 | 中 | 添加结果缓存，优化搜索算法 |
| 并发索引操作冲突 | 中 | 低 | DashMap 提供线程安全 |
| 内存中索引过多 | 中 | 中 | 后续添加索引卸载机制 |

### 6.2 项目风险

| 风险 | 影响 | 可能性 | 缓解措施 |
|------|------|--------|----------|
| 集成工期延长 | 高 | 高 | 明确优先级，分阶段交付 |
| 测试覆盖不足 | 中 | 高 | 同步编写单元测试和集成测试 |
| 文档不完善 | 中 | 中 | 同步更新用户指南和 API 文档 |

---

## 七、建议的集成优先级

### P0 - 核心功能（必须完成）

1. ✅ 实现 `FulltextSearchExecutor::execute()` 基本逻辑
2. ✅ 实现查询表达式转换（AST -> FulltextQuery）
3. ✅ 实现搜索结果到 ExecutionResult 的转换
4. ✅ 实现 score() 函数的上下文传递

**预期成果**：SEARCH 语句可以执行并返回基本结果

### P1 - 重要功能（应该完成）

5. ✅ 实现 `FulltextScanExecutor::execute()`
6. ✅ 实现 `MatchFulltextExecutor::execute()`
7. ✅ 实现 highlight(), matched_fields() 函数
8. ✅ 实现 WHERE 过滤
9. ✅ 实现 ORDER BY 排序
10. ✅ 实现 LIMIT/OFFSET 分页

**预期成果**：所有全文搜索语法都可以正常使用

### P2 - 增强功能（可以后续完成）

11. ⏸️ 实现自动数据同步（与存储层集成）
12. ⏸️ 实现查询优化（索引选择、谓词下推）
13. ⏸️ 实现结果缓存
14. ⏸️ 实现并发搜索优化
15. ⏸️ 实现查询计划优化

**预期成果**：性能优化和自动化提升

---

## 八、结论

当前全文索引功能的集成**框架完整但实现不完整**。核心问题在于**执行器层没有实现实际执行逻辑**，导致从查询语法到搜索引擎的完整链路在执行器这里中断。

**关键行动项**：
1. 立即实现三个执行器的 `execute()` 方法
2. 建立执行器与 Coordinator 的调用关系
3. 实现查询表达式转换和结果处理
4. 实现表达式函数的上下文传递

完成这些后，全文索引功能才能真正集成到查询工作流中，支持用户使用 SQL 语法进行搜索。

---

## 附录 A：相关文件清单

### A.1 核心实现文件

- `src/search/engine.rs` - SearchEngine Trait
- `src/search/manager.rs` - FulltextIndexManager
- `src/search/factory.rs` - SearchEngineFactory
- `src/coordinator/fulltext.rs` - FulltextCoordinator
- `src/query/parser/ast/fulltext.rs` - AST 定义
- `src/query/parser/parsing/fulltext_parser.rs` - Parser
- `src/query/validator/fulltext_validator.rs` - Validator
- `src/query/planning/plan/core/nodes/management/fulltext_nodes.rs` - Planner 节点
- `src/query/executor/data_access/fulltext_search.rs` - 执行器（占位符）
- `src/query/executor/data_access/match_fulltext.rs` - 执行器（占位符）
- `src/query/executor/factory/executor_factory.rs` - Executor Factory
- `src/query/executor/expression/functions/fulltext.rs` - 表达式函数

### A.2 引擎实现

- `crates/bm25/src/` - BM25 引擎（基于 Tantivy）
- `crates/inversearch/src/` - Inversearch 引擎

### A.3 测试文件

- `src/coordinator/fulltext_test.rs` - Coordinator 单元测试
- `tests/fulltext_integration_test.rs` - 集成测试
- `benches/fulltext_benchmark.rs` - 性能基准测试

---

## 附录 B：术语表

| 术语 | 说明 |
|------|------|
| SearchEngine | 搜索引擎 Trait，定义索引操作接口 |
| FulltextIndexManager | 全文索引管理器，管理所有索引的生命周期 |
| FulltextCoordinator | 全文索引协调器，协调数据变更和索引更新 |
| BM25 | 基于 Tantivy 的 BM25 全文搜索引擎 |
| Inversearch | 自研的倒排索引全文搜索引擎 |
| AST | 抽象语法树（Abstract Syntax Tree） |
| Executor | 执行器，负责执行查询计划节点 |
| FulltextQueryExpr | 全文查询表达式 AST 节点 |
| FulltextQuery | 全文查询类型（搜索引擎层） |

---

**文档结束**
