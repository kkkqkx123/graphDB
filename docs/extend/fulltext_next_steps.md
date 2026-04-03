# BM25 与 Inversearch 集成后续任务

## 当前状态

已完成的核心功能模块：
- ✅ 数据类型定义（`src/core/types/index.rs`, `src/core/types/fulltext_query.rs`）
- ✅ 搜索结果类型（`src/search/result.rs`）
- ✅ AST 定义（`src/query/parser/ast/fulltext.rs`）
- ✅ 解析器实现（`src/query/parser/parsing/fulltext_parser.rs`）
- ✅ 执行器实现（`src/query/executor/data_access/fulltext_search.rs`）
- ✅ 表达式函数（`src/query/executor/expression/functions/fulltext.rs`）
- ✅ 执行计划生成器（`src/query/planning/planner/fulltext.rs`）
- ✅ 验证器（`src/query/validator/fulltext_validator.rs`）

## 后续集成任务

### 任务 1：注册全文搜索解析器

**位置**：`src/query/parser/parser.rs` 或 `src/query/parser/parsing/mod.rs`

**需要修改的文件**：
- `src/query/parser/parsing/mod.rs` - 已导出 FulltextParser ✅
- `src/query/parser/parser.rs` - 需要在语句解析中集成

**实现步骤**：

```rust
// 在 src/query/parser/parser.rs 的 parse_statement 方法中添加
use crate::query::parser::parsing::FulltextParser;

fn parse_statement(&mut self) -> ParserResult {
    // ... existing code ...
    
    // Try full-text search parser
    if let Ok(result) = FulltextParser::new(&mut self.ctx).parse() {
        return Ok(result);
    }
    
    // ... other parsers ...
}
```

**测试验证**：
```rust
#[test]
fn test_parse_fulltext_search() {
    let sql = r#"SEARCH INDEX idx_article MATCH 'database' YIELD doc_id, score()"#;
    let mut parser = Parser::new(sql);
    let result = parser.parse();
    assert!(result.is_ok());
}
```

---

### 任务 2：扩展 Operator 枚举

**位置**：`src/query/planning/plan/core/nodes/operation.rs`

**需要修改的文件**：
- `src/query/planning/plan/core/nodes/operation.rs`
- `src/query/planning/plan/core/nodes/mod.rs`

**实现步骤**：

```rust
// 在 src/query/planning/plan/core/nodes/operation.rs 中添加
use crate::core::types::FulltextEngineType;
use crate::query::parser::ast::{
    CreateFulltextIndex, DropFulltextIndex, AlterFulltextIndex,
    SearchStatement, LookupFulltext, MatchFulltext,
    IndexFieldDef, IndexOptions, YieldClause, WhereClause, OrderClause,
    FulltextMatchCondition,
};

#[derive(Debug, Clone)]
pub enum Operator {
    // ... existing operators ...
    
    // Full-text DDL operators
    CreateFulltextIndex {
        index_name: String,
        schema_name: String,
        fields: Vec<IndexFieldDef>,
        engine_type: FulltextEngineType,
        options: IndexOptions,
        if_not_exists: bool,
    },
    DropFulltextIndex {
        index_name: String,
        if_exists: bool,
    },
    AlterFulltextIndex {
        index_name: String,
        actions: Vec<AlterIndexAction>,
    },
    ShowFulltextIndex {
        pattern: Option<String>,
        from_schema: Option<String>,
    },
    DescribeFulltextIndex {
        index_name: String,
    },
    
    // Full-text DML operators
    FulltextSearch {
        index_name: String,
        query: FulltextQueryExpr,
        yield_clause: Option<YieldClause>,
        where_clause: Option<WhereClause>,
        order_clause: Option<OrderClause>,
        limit: Option<usize>,
        offset: Option<usize>,
    },
    FulltextLookup {
        schema_name: String,
        index_name: String,
        query: String,
        yield_clause: Option<YieldClause>,
        limit: Option<usize>,
    },
    MatchFulltext {
        pattern: String,
        fulltext_condition: FulltextMatchCondition,
        yield_clause: Option<YieldClause>,
    },
}
```

**更新 mod.rs**：
```rust
// 在 src/query/planning/plan/core/nodes/mod.rs 中确保导出
pub use operation::Operator;
```

---

### 任务 3：注册 Planner

**位置**：`src/query/planning/planner.rs`

**需要修改的文件**：
- `src/query/planning/planner.rs`
- `src/query/planning/planner/mod.rs`

**实现步骤**：

```rust
// 在 src/query/planning/planner.rs 中添加导入
use crate::query::planning::planner::fulltext::FulltextSearchPlanner;

// 在 planner registry 初始化时注册
fn create_default_planners() -> Vec<Box<dyn Planner>> {
    let mut registry = vec![
        // ... existing planners ...
        Box::new(FulltextSearchPlanner::new()),
    ];
    registry
}
```

**或者使用注册函数**：
```rust
// 在 src/query/planning/planner/fulltext.rs 中已提供
pub fn register_fulltext_planner(registry: &mut PlannerRegistry) {
    registry.register(Box::new(FulltextSearchPlanner::new()));
}

// 在 src/query/planning/planner.rs 中调用
use crate::query::planning::planner::fulltext::register_fulltext_planner;

fn init_registry() -> PlannerRegistry {
    let mut registry = PlannerRegistry::new();
    // ... register other planners ...
    register_fulltext_planner(&mut registry);
    registry
}
```

---

### 任务 4：集成执行器工厂

**位置**：`src/query/executor/factory/executors/mod.rs`

**需要修改的文件**：
- `src/query/executor/factory/executors/mod.rs`
- `src/query/executor/factory/builders/data_access_builder.rs`

**实现步骤**：

```rust
// 在 src/query/executor/factory/executors/mod.rs 中添加
use crate::query::executor::data_access::FulltextSearchExecutor;
use crate::query::executor::data_access::FulltextScanExecutor;

pub fn create_executor(
    plan: &ExecutionPlan,
    context: ExecutionContext,
) -> Result<Box<dyn Executor>> {
    match plan.operator() {
        Operator::FulltextSearch { .. } => {
            let executor = FulltextSearchExecutor::new(/* params */);
            Ok(Box::new(executor))
        }
        Operator::FulltextLookup { .. } => {
            let executor = FulltextScanExecutor::new(/* params */);
            Ok(Box::new(executor))
        }
        // ... other operators ...
    }
}
```

**或者在 builder 中添加**：
```rust
// 在 src/query/executor/factory/builders/data_access_builder.rs 中添加
pub fn build_fulltext_search(
    plan: &FulltextSearchPlan,
    context: ExecutionContext,
) -> Result<Box<dyn Executor>> {
    let executor = FulltextSearchExecutor::new(
        plan.query.clone(),
        plan.index_name.clone(),
        context.search_engine(),
    );
    Ok(Box::new(executor))
}
```

---

### 任务 5：集成索引管理

**位置**：`src/search/mod.rs` 和 `src/index/mod.rs`

**需要修改的文件**：
- `src/search/mod.rs`
- `src/index/mod.rs`
- `src/api/service/fulltext_service.rs`（如果存在）

**实现步骤**：

```rust
// 在 src/search/mod.rs 中确保导出
pub use result::FulltextSearchResult;
pub use result::FulltextSearchEntry;

// 在索引管理器中添加全文索引支持
pub enum IndexType {
    // ... existing ...
    Fulltext(FulltextEngineType),
}

// 在索引创建逻辑中添加
match index_type {
    IndexType::Fulltext(engine) => {
        match engine {
            FulltextEngineType::Bm25 => {
                // 创建 BM25 索引
                bm25_manager.create_index(config).await?;
            }
            FulltextEngineType::Inversearch => {
                // 创建 Inversearch 索引
                inversearch_manager.create_index(config).await?;
            }
        }
    }
    // ... other index types ...
}
```

---

### 任务 6：更新查询上下文

**位置**：`src/query/context/mod.rs`

**需要修改的文件**：
- `src/query/context/mod.rs`
- `src/query/context/resource_context.rs`

**实现步骤**：

```rust
// 在 QueryContext 中添加全文搜索相关字段
pub struct QueryContext {
    // ... existing fields ...
    
    /// Full-text search engine reference
    search_engine: Arc<dyn SearchEngine>,
    
    /// Full-text index manager
    fulltext_index_manager: Arc<FulltextIndexManager>,
}

impl QueryContext {
    pub fn with_search_engine(mut self, engine: Arc<dyn SearchEngine>) -> Self {
        self.search_engine = engine;
        self
    }
    
    pub fn search_engine(&self) -> &Arc<dyn SearchEngine> {
        &self.search_engine
    }
}
```

---

### 任务 7：添加 gRPC 服务接口（可选）

**位置**：`src/api/service/`

**如果需要 gRPC 接口**：

**需要创建的文件**：
- `proto/fulltext.proto`（如果需要外部服务）
- `src/api/service/fulltext_service.rs`

**实现步骤**：

```rust
// 在 src/api/service/fulltext_service.rs 中
use crate::core::types::FulltextEngineType;
use crate::search::SearchEngine;

pub struct FulltextService {
    search_engine: Arc<dyn SearchEngine>,
    index_manager: Arc<FulltextIndexManager>,
}

impl FulltextService {
    pub async fn create_index(
        &self,
        name: String,
        schema: String,
        fields: Vec<String>,
        engine: FulltextEngineType,
        options: IndexOptions,
    ) -> Result<()> {
        // 创建索引逻辑
    }
    
    pub async fn search(
        &self,
        index_name: String,
        query: FulltextQuery,
        options: QueryOptions,
    ) -> Result<FulltextSearchResult> {
        // 执行搜索逻辑
    }
}
```

---

### 任务 8：集成测试

**位置**：`tests/fulltext_integration_test.rs`

**需要创建的文件**：
- `tests/fulltext_integration_test.rs`

**测试用例**：

```rust
use graphdb::query::parser::Parser;
use graphdb::query::validator::Validator;
use graphdb::query::planning::planner::Planner;
use graphdb::query::executor::Executor;

#[tokio::test]
async fn test_fulltext_search_end_to_end() {
    // 1. 创建索引
    let create_sql = r#"
        CREATE FULLTEXT INDEX idx_article_content 
        ON article(title, content)
        ENGINE BM25
        OPTIONS (k1 = 1.2, b = 0.75)
    "#;
    
    // 2. 插入测试数据
    // ...
    
    // 3. 执行搜索
    let search_sql = r#"
        SEARCH INDEX idx_article_content 
        MATCH 'database optimization'
        YIELD doc_id, score() AS s, highlight(content)
        WHERE s > 0.5
        ORDER BY s DESC
        LIMIT 10
    "#;
    
    // 4. 验证结果
    // ...
}

#[test]
fn test_fulltext_parser() {
    let test_cases = vec![
        "CREATE FULLTEXT INDEX idx ON tag(field)",
        "SEARCH INDEX idx MATCH 'query' YIELD doc_id",
        "LOOKUP ON tag INDEX idx WHERE 'query' YIELD vid",
    ];
    
    for sql in test_cases {
        let parser = Parser::new(sql);
        let result = parser.parse();
        assert!(result.is_ok(), "Failed to parse: {}", sql);
    }
}

#[test]
fn test_fulltext_validator() {
    // 测试验证逻辑
}

#[tokio::test]
async fn test_fulltext_executor() {
    // 测试执行器逻辑
}
```

---

### 任务 9：文档更新

**需要更新的文档**：
- `docs/extend/fulltext_implementation_summary.md` - 已创建 ✅
- `docs/extend/bm25_inversearch_integration_plan.md` - 需要更新
- 创建 `docs/user_guide/fulltext_search.md`

**用户指南内容**：

```markdown
# 全文搜索用户指南

## 快速开始

### 1. 创建全文索引

```sql
CREATE FULLTEXT INDEX idx_article_content 
ON article(title, content)
ENGINE BM25
OPTIONS (k1 = 1.2, b = 0.75, analyzer = 'standard');
```

### 2. 执行搜索

```sql
SEARCH INDEX idx_article_content 
MATCH 'database optimization'
YIELD doc_id, score() AS s, highlight(content) AS h
WHERE s > 0.5
ORDER BY s DESC
LIMIT 10;
```

### 3. 图查询集成

```sql
MATCH (a:article)
WHERE FULLTEXT_MATCH(a.content, 'database')
YIELD a, score() AS s;
```

## 查询类型

- 简单查询：`MATCH 'keyword'`
- 多字段查询：`MATCH {title: 'keyword', content: 'database'}`
- 布尔查询：`MATCH {must: ['database'], should: ['optimization']}`
- 短语查询：`MATCH PHRASE 'database optimization'`
- 前缀查询：`MATCH PREFIX 'data'`
- 模糊查询：`MATCH FUZZY 'databse'~2`
- 范围查询：`MATCH RANGE field FROM 'a' TO 'z'`
- 通配符查询：`MATCH 'data*'`

## 表达式函数

- `score()` - 获取相关性分数
- `highlight(field, [pre='<b>'], [post='</b>'], [size=100])` - 高亮显示
- `matched_fields()` - 获取匹配字段列表
- `snippet(field, [max_len=200])` - 获取文本片段

## 索引配置

### BM25 参数

- `k1` - 词频饱和度（默认 1.2）
- `b` - 文档长度归一化（默认 0.75）
- `analyzer` - 分词器（standard/simple）

### Inversearch 参数

- `tokenize_mode` - 分词模式（strict/forward/backward/full）
- `charset` - 字符集（cjk/latin/exact）
- `resolution` - 分辨率（默认 1）
```

---

## 集成检查清单

### 解析器集成
- [ ] 在 parser.rs 中注册 FulltextParser
- [ ] 添加解析器单元测试
- [ ] 验证所有 SQL 语法

### 计划生成器集成
- [ ] 扩展 Operator 枚举
- [ ] 在 planner.rs 中注册 FulltextSearchPlanner
- [ ] 添加计划生成测试

### 执行器集成
- [ ] 在执行器工厂中添加全文搜索执行器
- [ ] 集成索引管理逻辑
- [ ] 添加执行器单元测试

### 验证器集成
- [ ] 在验证器工厂中注册 FulltextValidator
- [ ] 添加验证规则测试

### 上下文集成
- [ ] 更新 QueryContext 添加搜索引擎引用
- [ ] 更新 ExecutionContext 添加执行上下文

### 测试
- [ ] 创建集成测试文件
- [ ] 编写端到端测试
- [ ] 性能基准测试

### 文档
- [ ] 更新集成计划文档
- [ ] 创建用户指南
- [ ] 添加 API 文档

---

## 优先级排序

### 高优先级（必须完成）
1. ✅ 删除旧的 planner 目录
2. 扩展 Operator 枚举（任务 2）
3. 注册 Planner（任务 3）
4. 集成执行器工厂（任务 4）
5. 集成测试（任务 8）

### 中优先级（推荐完成）
6. 注册解析器（任务 1）
7. 集成索引管理（任务 5）
8. 更新查询上下文（任务 6）

### 低优先级（可选）
9. gRPC 服务接口（任务 7）- 仅在需要外部访问时
10. 文档更新（任务 9）- 可以逐步完善

---

## 预计工作量

- **核心集成**（任务 1-5）：2-3 天
- **完整集成**（任务 1-8）：4-5 天
- **文档和完善**（任务 9）：1 天

---

## 注意事项

1. **类型安全**：确保所有新类型都正确实现 Serialize/Deserialize
2. **错误处理**：使用 Result 类型，避免 unwrap()
3. **测试覆盖**：每个模块至少包含一个单元测试
4. **代码风格**：遵循 Rust 命名约定，使用英文注释
5. **性能考虑**：全文搜索可能涉及大量数据，注意内存管理
6. **并发安全**：使用 Arc 和 Mutex 确保线程安全

---

## 参考文档

- [实现总结](./fulltext_implementation_summary.md)
- [原始集成计划](./bm25_inversearch_integration_plan.md)
- [架构分析](../analysis/)
