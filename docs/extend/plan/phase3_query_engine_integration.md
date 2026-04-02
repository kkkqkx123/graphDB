# Phase 3: 查询引擎集成方案

## 阶段目标

将全文检索功能集成到查询引擎中，支持在 nGQL 查询中使用全文搜索语法。

**预计工期**: 5-7 天  
**前置依赖**: Phase 2 (FulltextCoordinator 协调器)

---

## 新增文件清单

### 1. SQL 语法扩展

| 文件路径 | 说明 |
|---------|------|
| `src/query/parser/fulltext.rs` | 全文检索语法解析 |
| `src/query/ast/fulltext.rs` | 全文检索 AST 节点 |

### 2. 查询执行

| 文件路径 | 说明 |
|---------|------|
| `src/query/executor/fulltext.rs` | 全文搜索执行器 |
| `src/query/planner/fulltext.rs` | 全文检索查询计划 |

### 3. 语句处理

| 文件路径 | 说明 |
|---------|------|
| `src/query/statements/create_fulltext_index.rs` | CREATE FULLTEXT INDEX 语句 |
| `src/query/statements/drop_fulltext_index.rs` | DROP FULLTEXT INDEX 语句 |
| `src/query/statements/show_fulltext_indexes.rs` | SHOW FULLTEXT INDEXES 语句 |

---

## 详细实现方案

### 1. SQL 语法扩展

#### 1.1 创建全文索引

```sql
-- 基本语法
CREATE FULLTEXT INDEX <index_name> ON <tag_name>(<field_name>);

-- 指定引擎
CREATE FULLTEXT INDEX idx_content ON Post(content) USING bm25;
CREATE FULLTEXT INDEX idx_content ON Post(content) USING inversearch;

-- 指定分词器（仅 Inversearch）
CREATE FULLTEXT INDEX idx_content ON Post(content) 
USING inversearch 
WITH TOKENIZER = 'cjk';

-- 多字段索引
CREATE FULLTEXT INDEX idx_post ON Post(title, content) USING bm25;
```

#### 1.2 全文搜索语法

```sql
-- 基本搜索（MATCH 操作符）
MATCH (p:Post)
WHERE p.content MATCH "图数据库"
RETURN p;

-- 带评分排序
MATCH (p:Post)
WHERE p.content MATCH "图数据库"
RETURN p, score(p) as relevance
ORDER BY relevance DESC
LIMIT 10;

-- 多字段搜索
MATCH (p:Post)
WHERE p.title MATCH "Rust" OR p.content MATCH "图数据库"
RETURN p;

-- 使用索引直接搜索
LOOKUP ON idx_content WHERE QUERY("搜索文本")
RETURN *;

-- 搜索并高亮
MATCH (p:Post)
WHERE p.content MATCH "图数据库"
RETURN p.id, highlight(p.content) as highlighted;
```

#### 1.3 索引管理语法

```sql
-- 查看全文索引列表
SHOW FULLTEXT INDEXES;

-- 查看索引状态
SHOW FULLTEXT INDEX STATUS idx_content;

-- 重建全文索引
REBUILD FULLTEXT INDEX idx_content;

-- 删除全文索引
DROP FULLTEXT INDEX idx_content;
```

### 2. AST 节点定义

**文件**: `src/query/ast/fulltext.rs`

```rust
use crate::query::ast::{Expression, Identifier, Statement};
use crate::search::engine::EngineType;

/// CREATE FULLTEXT INDEX 语句
#[derive(Debug, Clone, PartialEq)]
pub struct CreateFulltextIndexStmt {
    /// 索引名称
    pub index_name: Identifier,
    /// Tag 名称
    pub tag_name: Identifier,
    /// 字段列表
    pub fields: Vec<Identifier>,
    /// 引擎类型（None 使用默认）
    pub engine_type: Option<EngineType>,
    /// 引擎配置选项
    pub engine_options: Vec<(String, String)>,
}

/// DROP FULLTEXT INDEX 语句
#[derive(Debug, Clone, PartialEq)]
pub struct DropFulltextIndexStmt {
    /// 索引名称
    pub index_name: Identifier,
    /// 是否存在检查
    pub if_exists: bool,
}

/// SHOW FULLTEXT INDEXES 语句
#[derive(Debug, Clone, PartialEq)]
pub struct ShowFulltextIndexesStmt {
    /// 是否显示详细状态
    pub show_status: bool,
    /// 指定索引名称（可选）
    pub index_name: Option<Identifier>,
}

/// REBUILD FULLTEXT INDEX 语句
#[derive(Debug, Clone, PartialEq)]
pub struct RebuildFulltextIndexStmt {
    /// 索引名称
    pub index_name: Identifier,
}

/// 全文搜索表达式
#[derive(Debug, Clone, PartialEq)]
pub struct FulltextMatchExpr {
    /// 字段引用（如 p.content）
    pub field_ref: FieldReference,
    /// 搜索查询字符串
    pub query: String,
}

/// 字段引用
#[derive(Debug, Clone, PartialEq)]
pub struct FieldReference {
    /// 变量名（如 p）
    pub variable: Identifier,
    /// 字段名（如 content）
    pub field: Identifier,
}

/// 全文搜索函数
#[derive(Debug, Clone, PartialEq)]
pub enum FulltextFunction {
    /// score(vertex) - 获取相关性评分
    Score(Expression),
    /// highlight(field) - 获取高亮文本
    Highlight(FieldReference),
}
```

### 3. 语法解析实现

**文件**: `src/query/parser/fulltext.rs`

```rust
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{delimited, pair, preceded, tuple},
};

use crate::query::ast::fulltext::*;
use crate::query::ast::Identifier;
use crate::query::parser::utils::*;
use crate::search::engine::EngineType;

/// 解析 CREATE FULLTEXT INDEX
pub fn parse_create_fulltext_index(input: &str) -> IResult<&str, CreateFulltextIndexStmt> {
    map(
        tuple((
            tag_no_case("CREATE"),
            ws(tag_no_case("FULLTEXT")),
            ws(tag_no_case("INDEX")),
            ws(parse_identifier),
            ws(tag_no_case("ON")),
            ws(parse_identifier),
            delimited(
                ws(tag("(")),
                separated_list1(ws(tag(",")), ws(parse_identifier)),
                ws(tag(")"))
            ),
            opt(parse_using_clause),
            opt(parse_with_options),
        )),
        |(_, _, _, index_name, _, tag_name, fields, engine_type, engine_options)| {
            CreateFulltextIndexStmt {
                index_name,
                tag_name,
                fields,
                engine_type,
                engine_options: engine_options.unwrap_or_default(),
            }
        }
    )(input)
}

/// 解析 USING 子句
fn parse_using_clause(input: &str) -> IResult<&str, EngineType> {
    preceded(
        ws(tag_no_case("USING")),
        alt((
            map(tag_no_case("bm25"), |_| EngineType::Bm25),
            map(tag_no_case("inversearch"), |_| EngineType::Inversearch),
        ))
    )(input)
}

/// 解析 WITH 选项
fn parse_with_options(input: &str) -> IResult<&str, Vec<(String, String)>> {
    preceded(
        ws(tag_no_case("WITH")),
        separated_list1(
            ws(tag(",")),
            map(
                tuple((
                    ws(parse_identifier),
                    ws(tag("=")),
                    ws(parse_string_literal),
                )),
                |(key, _, value)| (key.name, value)
            )
        )
    )(input)
}

/// 解析 MATCH 表达式
pub fn parse_fulltext_match(input: &str) -> IResult<&str, FulltextMatchExpr> {
    map(
        tuple((
            parse_field_reference,
            ws(tag_no_case("MATCH")),
            ws(parse_string_literal),
        )),
        |(field_ref, _, query)| FulltextMatchExpr { field_ref, query }
    )(input)
}

/// 解析字段引用（如 p.content）
fn parse_field_reference(input: &str) -> IResult<&str, FieldReference> {
    map(
        tuple((
            parse_identifier,
            ws(tag(".")),
            parse_identifier,
        )),
        |(variable, _, field)| FieldReference { variable, field }
    )(input)
}

/// 解析 DROP FULLTEXT INDEX
pub fn parse_drop_fulltext_index(input: &str) -> IResult<&str, DropFulltextIndexStmt> {
    map(
        tuple((
            tag_no_case("DROP"),
            ws(tag_no_case("FULLTEXT")),
            ws(tag_no_case("INDEX")),
            opt(ws(tag_no_case("IF EXISTS"))),
            ws(parse_identifier),
        )),
        |(_, _, _, if_exists, index_name)| DropFulltextIndexStmt {
            index_name,
            if_exists: if_exists.is_some(),
        }
    )(input)
}

/// 解析 SHOW FULLTEXT INDEXES
pub fn parse_show_fulltext_indexes(input: &str) -> IResult<&str, ShowFulltextIndexesStmt> {
    map(
        tuple((
            tag_no_case("SHOW"),
            ws(tag_no_case("FULLTEXT")),
            ws(tag_no_case("INDEXES")),
            opt(preceded(ws(tag_no_case("STATUS")), ws(parse_identifier))),
        )),
        |(_, _, _, index_name)| ShowFulltextIndexesStmt {
            show_status: index_name.is_some(),
            index_name,
        }
    )(input)
}

/// 解析 REBUILD FULLTEXT INDEX
pub fn parse_rebuild_fulltext_index(input: &str) -> IResult<&str, RebuildFulltextIndexStmt> {
    map(
        tuple((
            tag_no_case("REBUILD"),
            ws(tag_no_case("FULLTEXT")),
            ws(tag_no_case("INDEX")),
            ws(parse_identifier),
        )),
        |(_, _, _, index_name)| RebuildFulltextIndexStmt { index_name }
    )(input)
}
```

### 4. 查询计划节点

**文件**: `src/query/planner/fulltext.rs`

```rust
use crate::query::planner::{PlanNode, PhysicalPlan};
use crate::core::Value;

/// 全文搜索计划节点
#[derive(Debug, Clone)]
pub struct FulltextSearchNode {
    /// 图空间ID
    pub space_id: u64,
    /// Tag 名称
    pub tag_name: String,
    /// 字段名称
    pub field_name: String,
    /// 搜索查询
    pub query: String,
    /// 结果数量限制
    pub limit: Option<usize>,
    /// 子节点
    pub child: Option<Box<PhysicalPlan>>,
}

/// 全文搜索连接节点
/// 将全文搜索结果与图数据连接
#[derive(Debug, Clone)]
pub struct FulltextJoinNode {
    /// 全文搜索结果列
    pub search_result_column: String,
    /// 顶点ID列
    pub vertex_id_column: String,
    /// 子节点
    pub child: Box<PhysicalPlan>,
}

/// 评分计算节点
#[derive(Debug, Clone)]
pub struct ScoreCalcNode {
    /// 评分列名
    pub score_column: String,
    /// 子节点
    pub child: Box<PhysicalPlan>,
}
```

### 5. 查询执行器

**文件**: `src/query/executor/fulltext.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;

use crate::coordinator::FulltextCoordinator;
use crate::query::executor::{Executor, ExecutionContext, ExecutionResult};
use crate::query::planner::fulltext::{FulltextSearchNode, FulltextJoinNode};
use crate::core::{Value, Vertex};
use crate::storage::StorageClient;

/// 全文搜索执行器
pub struct FulltextExecutor {
    coordinator: Arc<FulltextCoordinator>,
    storage: Arc<dyn StorageClient>,
}

impl FulltextExecutor {
    pub fn new(
        coordinator: Arc<FulltextCoordinator>,
        storage: Arc<dyn StorageClient>,
    ) -> Self {
        Self { coordinator, storage }
    }
    
    /// 执行全文搜索节点
    pub async fn execute_fulltext_search(
        &self,
        node: &FulltextSearchNode,
        ctx: &mut ExecutionContext,
    ) -> ExecutionResult {
        // 1. 执行全文搜索
        let search_results = self.coordinator
            .search(node.space_id, &node.tag_name, &node.field_name, &node.query, node.limit.unwrap_or(100))
            .await
            .map_err(|e| ExecutionError::FulltextError(e.to_string()))?;
        
        // 2. 提取文档ID列表
        let doc_ids: Vec<Value> = search_results.iter()
            .map(|r| r.doc_id.clone())
            .collect();
        
        // 3. 从存储层获取完整顶点数据
        let mut rows = Vec::new();
        for (search_result, doc_id) in search_results.iter().zip(&doc_ids) {
            if let Some(vertex) = self.storage.get_vertex_by_id(doc_id).await? {
                let mut row = HashMap::new();
                row.insert("vertex".to_string(), Value::Vertex(vertex));
                row.insert("score".to_string(), Value::from(search_result.score));
                rows.push(row);
            }
        }
        
        Ok(ExecutionResult::new(rows))
    }
    
    /// 执行全文搜索连接
    pub async fn execute_fulltext_join(
        &self,
        node: &FulltextJoinNode,
        input: ExecutionResult,
        ctx: &mut ExecutionContext,
    ) -> ExecutionResult {
        let mut rows = Vec::new();
        
        for mut row in input.rows {
            if let Some(Value::List(doc_ids)) = row.get(&node.search_result_column) {
                for doc_id in doc_ids {
                    if let Some(vertex) = self.storage.get_vertex_by_id(doc_id).await? {
                        let mut new_row = row.clone();
                        new_row.insert(node.vertex_id_column.clone(), doc_id.clone());
                        new_row.insert("vertex".to_string(), Value::Vertex(vertex));
                        rows.push(new_row);
                    }
                }
            }
        }
        
        Ok(ExecutionResult::new(rows))
    }
}
```

### 6. 语句执行实现

**文件**: `src/query/statements/create_fulltext_index.rs`

```rust
use crate::coordinator::FulltextCoordinator;
use crate::query::ast::fulltext::CreateFulltextIndexStmt;
use crate::query::statements::StatementExecutor;
use crate::query::ExecutionContext;
use std::sync::Arc;

pub struct CreateFulltextIndexExecutor {
    coordinator: Arc<FulltextCoordinator>,
}

impl CreateFulltextIndexExecutor {
    pub fn new(coordinator: Arc<FulltextCoordinator>) -> Self {
        Self { coordinator }
    }
    
    pub async fn execute(
        &self,
        stmt: &CreateFulltextIndexStmt,
        ctx: &ExecutionContext,
    ) -> Result<ExecutionResult, ExecutionError> {
        let space_id = ctx.current_space_id()
            .ok_or(ExecutionError::NoSpaceSelected)?;
        
        // 目前只支持单字段索引
        if stmt.fields.len() != 1 {
            return Err(ExecutionError::Unsupported(
                "Multi-field fulltext index not yet supported".to_string()
            ));
        }
        
        let field_name = &stmt.fields[0].name;
        let tag_name = &stmt.tag_name.name;
        
        let index_id = self.coordinator
            .create_index(space_id, tag_name, field_name, stmt.engine_type)
            .await
            .map_err(|e| ExecutionError::FulltextError(e.to_string()))?;
        
        Ok(ExecutionResult::success(format!(
            "Fulltext index '{}' created successfully",
            index_id
        )))
    }
}
```

**文件**: `src/query/statements/show_fulltext_indexes.rs`

```rust
use crate::coordinator::FulltextCoordinator;
use crate::query::ast::fulltext::ShowFulltextIndexesStmt;
use std::sync::Arc;

pub struct ShowFulltextIndexesExecutor {
    coordinator: Arc<FulltextCoordinator>,
}

impl ShowFulltextIndexesExecutor {
    pub fn new(coordinator: Arc<FulltextCoordinator>) -> Self {
        Self { coordinator }
    }
    
    pub async fn execute(
        &self,
        stmt: &ShowFulltextIndexesStmt,
    ) -> Result<ExecutionResult, ExecutionError> {
        let indexes = self.coordinator.list_indexes();
        
        let mut rows = Vec::new();
        for metadata in indexes {
            let mut row = HashMap::new();
            row.insert("Index Name".to_string(), Value::from(metadata.index_name));
            row.insert("Tag".to_string(), Value::from(metadata.tag_name));
            row.insert("Field".to_string(), Value::from(metadata.field_name));
            row.insert("Engine".to_string(), Value::from(metadata.engine_type.to_string()));
            row.insert("Status".to_string(), Value::from(metadata.status.to_string()));
            row.insert("Doc Count".to_string(), Value::from(metadata.doc_count as i64));
            rows.push(row);
        }
        
        Ok(ExecutionResult::new(rows))
    }
}
```

### 7. 查询重写器

**文件**: `src/query/rewriter/fulltext.rs`

```rust
use crate::query::ast::*;
use crate::query::ast::fulltext::*;

/// 全文检索查询重写器
/// 
/// 将包含全文搜索的查询重写为可执行的计划
pub struct FulltextQueryRewriter;

impl FulltextQueryRewriter {
    /// 重写 WHERE 子句中的 MATCH 表达式
    pub fn rewrite_where_clause(
        &self,
        where_clause: Option<Expression>,
    ) -> (Option<Expression>, Vec<FulltextMatchExpr>) {
        let mut fulltext_matches = Vec::new();
        
        let rewritten = where_clause.map(|expr| {
            self.extract_fulltext_matches(expr, &mut fulltext_matches)
        });
        
        (rewritten, fulltext_matches)
    }
    
    /// 提取全文搜索表达式
    fn extract_fulltext_matches(
        &self,
        expr: Expression,
        matches: &mut Vec<FulltextMatchExpr>,
    ) -> Expression {
        match expr {
            Expression::BinaryOp { left, op, right } => {
                // 检查是否是 MATCH 操作
                if let Expression::FulltextMatch(match_expr) = *right {
                    matches.push(match_expr);
                    // 将 MATCH 表达式替换为 IN 表达式
                    Expression::In {
                        expr: left,
                        list: vec![], // 将在执行时填充
                    }
                } else {
                    Expression::BinaryOp {
                        left: Box::new(self.extract_fulltext_matches(*left, matches)),
                        op,
                        right: Box::new(self.extract_fulltext_matches(*right, matches)),
                    }
                }
            }
            _ => expr,
        }
    }
}
```

---

## 数据流设计

### 完整查询执行流程

```
用户输入
    │ "MATCH (p:Post) WHERE p.content MATCH '图数据库' RETURN p"
    ▼
SQL Parser
    │ 解析为 AST
    ▼
Query Rewriter
    │ 识别全文搜索表达式
    │ 提取 MATCH 条件
    ▼
Query Planner
    │ 生成执行计划
    │ FulltextSearchNode -> GetVertexNode
    ▼
Query Executor
    │ 1. 执行 FulltextSearchNode
    │    - 调用 coordinator.search()
    │    - 获取 doc_ids
    │ 2. 执行 GetVertexNode
    │    - 根据 doc_ids 查询顶点
    ▼
结果返回
```

---

## 测试方案

### 集成测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fulltext_match_query() {
        let (query_engine, _temp) = setup_test_env().await;
        
        // 创建索引
        query_engine.execute("CREATE FULLTEXT INDEX idx_content ON Post(content) USING bm25").await.unwrap();
        
        // 插入数据
        query_engine.execute("INSERT VERTEX Post(content) VALUES 'Hello world'").await.unwrap();
        query_engine.execute("INSERT VERTEX Post(content) VALUES 'Hello Rust'").await.unwrap();
        
        // 等待索引（实际实现中可能需要同步等待）
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // 执行全文搜索
        let result = query_engine.execute(
            "MATCH (p:Post) WHERE p.content MATCH 'Hello' RETURN p"
        ).await.unwrap();
        
        assert_eq!(result.rows.len(), 2);
    }
    
    #[tokio::test]
    async fn test_fulltext_with_score() {
        let (query_engine, _temp) = setup_test_env().await;
        
        query_engine.execute("CREATE FULLTEXT INDEX idx_content ON Post(content) USING bm25").await.unwrap();
        query_engine.execute("INSERT VERTEX Post(content) VALUES 'Rust programming language'").await.unwrap();
        
        let result = query_engine.execute(
            "MATCH (p:Post) WHERE p.content MATCH 'Rust' RETURN p, score(p) as relevance"
        ).await.unwrap();
        
        assert_eq!(result.rows.len(), 1);
        assert!(result.rows[0].contains_key("relevance"));
    }
}
```

---

## 验收标准

- [ ] 支持 `CREATE FULLTEXT INDEX` 语句
- [ ] 支持 `DROP FULLTEXT INDEX` 语句
- [ ] 支持 `SHOW FULLTEXT INDEXES` 语句
- [ ] 支持 `MATCH` 操作符在 WHERE 子句中使用
- [ ] 支持 `score()` 函数获取相关性评分
- [ ] 支持 `highlight()` 函数（Inversearch 引擎）
- [ ] 全文搜索查询正确返回结果
- [ ] 所有集成测试通过

---

## 风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 语法解析冲突 | 中 | 确保 MATCH 关键字上下文敏感 |
| 查询计划优化 | 中 | 先实现基础版本，后续优化 |
| 多引擎语法差异 | 低 | 统一语法，引擎特定选项通过 WITH 子句传递 |

---

## 下一阶段依赖

本阶段完成后，用户可以：

- 使用 SQL 语句创建全文索引
- 在 MATCH 查询中使用全文搜索
- 查看和管理全文索引
