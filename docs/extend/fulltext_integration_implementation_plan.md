# 全文索引功能查询工作流集成实施方案

**文档版本**: 1.0  
**创建日期**: 2026-04-04  
**状态**: 待实施  
**前置依赖**: Phase 0（环境准备）已完成

---

## 概述

本文档提供全文索引功能集成到查询工作流的详细分阶段实施方案。基于对现有代码的深入分析，我们将集成工作分为 4 个阶段（Phase 1-4），每个阶段都有明确的目标、交付物和验收标准。

### 实施路线图

```
Phase 1 (3-4 天)     Phase 2 (4-5 天)     Phase 3 (3-4 天)     Phase 4 (2-3 天)
    │                    │                    │                    │
    ▼                    ▼                    ▼                    ▼
┌────────────┐      ┌────────────┐      ┌────────────┐      ┌────────────┐
│ 执行器核心 │      │ 查询处理   │      │ 存储层集成 │      │ 测试优化   │
│ 实现       │  →   │ 增强       │  →   │ 与自动化   │  →   │ 与文档     │
└────────────┘      └────────────┘      └────────────┘      └────────────┘
    │                    │                    │                    │
    │                    │                    │                    │
    ▼                    ▼                    ▼                    ▼
SEARCH 可执行       所有语法可用      数据自动同步        生产就绪
```

---

## Phase 1: 执行器核心实现 ⭐⭐⭐⭐⭐

**目标**：实现三个全文搜索执行器的核心执行逻辑，使 SEARCH、LOOKUP FULLTEXT、MATCH FULLTEXT 语句能够执行并返回基本结果。

**工期**：3-4 天  
**优先级**：P0（必须完成）  
**前置依赖**：无

---

### Phase 1.1: FulltextSearchExecutor 实现

**目标**：实现 SEARCH 语句的执行器

**工作内容**：

1. **索引名称解析**（2 小时）
   - 解析索引名称格式：`space_id_tag_name_field_name`
   - 提取 space_id, tag_name, field_name

2. **查询表达式转换**（4 小时）
   - 实现 `FulltextQueryExpr` → `FulltextQuery` 的转换
   - 支持所有 9 种查询类型：
     - Simple, Field, MultiField
     - Boolean (must/should/must_not)
     - Phrase, Prefix, Fuzzy
     - Range, Wildcard

3. **调用 Coordinator 搜索**（2 小时）
   - 获取 Coordinator 引用
   - 调用 `coordinator.search()` 方法
   - 处理搜索结果

4. **结果转换**（4 小时）
   - 根据 doc_ids 获取完整顶点数据
   - 构建 ExecutionResult::Rows
   - 处理基本的 YIELD 子句（Field, Score）

5. **错误处理**（2 小时）
   - 索引不存在错误
   - 查询转换错误
   - 搜索执行错误

**关键代码**：

```rust
// src/query/executor/data_access/fulltext_search.rs
impl<S: StorageClient> Executor<S> for FulltextSearchExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // 1. 解析索引名称
        let (space_id, tag_name, field_name) = self.parse_index_name()?;
        
        // 2. 转换查询表达式
        let query = self.convert_query(&self.statement.query)?;
        
        // 3. 获取 Coordinator 并执行搜索
        let coordinator = self.get_coordinator()?;
        let search_results = futures::executor::block_on(
            coordinator.search(space_id, &tag_name, &field_name, &query, limit)
        )?;
        
        // 4. 构建结果
        let mut rows = Vec::new();
        for result in search_results {
            if let Some(vertex) = block_on(self.storage.get_vertex_by_id(&result.doc_id))? {
                let mut row = HashMap::new();
                
                // 处理 YIELD 子句
                for yield_item in &self.statement.yield_clause.items {
                    match &yield_item.expr {
                        YieldExpression::Field(name) => {
                            if let Some(value) = vertex.get_property(name) {
                                row.insert(name.clone(), value.clone());
                            }
                        }
                        YieldExpression::Score(_) => {
                            row.insert("score".to_string(), Value::Float(result.score));
                        }
                        _ => {} // 其他表达式后续实现
                    }
                }
                rows.push(row);
            }
        }
        
        Ok(ExecutionResult::Rows(rows))
    }
}
```

**交付物**：
- ✅ `FulltextSearchExecutor::execute()` 完整实现
- ✅ 查询表达式转换函数
- ✅ 单元测试（`tests/fulltext_search_executor_test.rs`）

**验收标准**：
```sql
-- 能够执行并返回结果
SEARCH INDEX idx_article_content 
MATCH 'database'
YIELD doc_id, score();

-- 返回正确的结果集
-- 结果包含 doc_id 和 score 字段
```

---

### Phase 1.2: FulltextScanExecutor 实现

**目标**：实现 LOOKUP FULLTEXT 语句的执行器

**工作内容**：

1. **索引查找**（2 小时）
   - 根据 index_name 查找索引元数据
   - 验证索引存在性

2. **简单查询执行**（3 小时）
   - 调用 Coordinator 搜索
   - 处理简单文本查询

3. **结果构建**（3 小时）
   - 获取顶点数据
   - 处理 YIELD 子句
   - 应用 LIMIT

**交付物**：
- ✅ `FulltextScanExecutor::execute()` 完整实现
- ✅ 单元测试

**验收标准**：
```sql
LOOKUP ON article INDEX idx_content 
WHERE 'database'
YIELD doc_id, score()
LIMIT 20;
```

---

### Phase 1.3: MatchFulltextExecutor 实现

**目标**：实现 MATCH with fulltext 语句的执行器

**工作内容**：

1. **模式解析**（2 小时）
   - 解析 MATCH 模式字符串
   - 提取顶点和边信息

2. **全文条件处理**（3 小时）
   - 解析 FULLTEXT_MATCH 条件
   - 获取索引信息

3. **搜索和图遍历结合**（4 小时）
   - 先执行全文搜索获取 doc_ids
   - 根据 doc_ids 获取顶点
   - 执行图遍历（如果有边模式）

**交付物**：
- ✅ `MatchFulltextExecutor::execute()` 完整实现
- ✅ 单元测试

**验收标准**：
```sql
MATCH (a:article)
WHERE FULLTEXT_MATCH(a.content, 'database')
YIELD a, score() AS s;
```

---

### Phase 1.4: 执行上下文集成

**目标**：实现全文搜索函数的上下文传递

**工作内容**：

1. **创建执行上下文**（2 小时）
   - 为每个搜索结果创建 `FulltextExecutionContext`
   - 设置 score, highlights 等字段

2. **表达式求值集成**（3 小时）
   - 修改表达式求值器，支持 fulltext context
   - 实现 score(), highlight() 函数的实际调用

3. **高亮内容生成**（3 小时）
   - 调用搜索引擎的高亮功能
   - 存储到 execution context

**关键代码**：

```rust
// 在执行器中
for result in &search_results {
    let mut ft_context = FulltextExecutionContext {
        score: result.score,
        highlights: HashMap::new(),
        matched_fields: vec![],
        snippets: HashMap::new(),
    };
    
    // 生成高亮
    if let Some(highlights) = self.generate_highlights(&result)? {
        ft_context.highlights = highlights;
    }
    
    // 求值表达式
    for yield_item in &self.statement.yield_clause.items {
        let value = yield_item.expr.evaluate_with_context(&ft_context)?;
        row.insert(yield_item.alias.clone(), value);
    }
}
```

**交付物**：
- ✅ `FulltextExecutionContext` 创建和传递逻辑
- ✅ score() 函数实际工作
- ✅ highlight() 函数实际工作

**验收标准**：
```sql
SEARCH INDEX idx_article_content 
MATCH 'database'
YIELD doc_id, score(), highlight(content);

-- score() 返回正确的评分
-- highlight(content) 返回带高亮标签的文本
```

---

### Phase 1 交付清单

| 文件 | 状态 | 说明 |
|------|------|------|
| `src/query/executor/data_access/fulltext_search.rs` | 待实现 | FulltextSearchExecutor |
| `src/query/executor/data_access/fulltext_scan.rs` | 待实现 | FulltextScanExecutor |
| `src/query/executor/data_access/match_fulltext.rs` | 待实现 | MatchFulltextExecutor |
| `src/query/executor/expression/functions/fulltext.rs` | 待增强 | 添加上下文传递 |
| `tests/fulltext_executor_test.rs` | 待创建 | 执行器单元测试 |

**验收测试**：
```rust
#[tokio::test]
async fn test_fulltext_search_executor() {
    // 1. 创建索引
    // 2. 插入测试数据
    // 3. 执行 SEARCH 语句
    // 4. 验证结果
    assert_eq!(results.len(), expected_count);
    assert!(results[0].contains_key("score"));
}
```

---

## Phase 2: 查询处理增强 ⭐⭐⭐⭐

**目标**：实现完整的查询处理功能，包括 WHERE 过滤、ORDER BY 排序、LIMIT/OFFSET 分页，以及所有表达式函数的完整支持。

**工期**：4-5 天  
**优先级**：P1（应该完成）  
**前置依赖**：Phase 1 完成

---

### Phase 2.1: WHERE 子句过滤

**目标**：实现搜索结果的 WHERE 条件过滤

**工作内容**：

1. **条件解析**（3 小时）
   - 解析 WHERE 子句的 AST
   - 提取过滤条件

2. **过滤器执行器**（4 小时）
   - 实现过滤逻辑
   - 支持比较操作符（=, !=, <, >, <=, >=）
   - 支持逻辑操作符（AND, OR, NOT）

3. **分数过滤**（2 小时）
   - 支持 `WHERE score > 0.5`
   - 支持 `WHERE score >= threshold`

**关键代码**：

```rust
// 在 FulltextSearchExecutor 中
if let Some(where_clause) = &self.statement.where_clause {
    rows = rows.into_iter()
        .filter(|row| self.evaluate_condition(&where_clause.condition, row))
        .collect();
}

fn evaluate_condition(&self, condition: &WhereCondition, row: &HashMap<String, Value>) -> bool {
    match condition {
        WhereCondition::Comparison(field, op, value) => {
            if let Some(row_value) = row.get(field) {
                self.compare_values(row_value, op, value)
            } else {
                false
            }
        }
        WhereCondition::And(left, right) => {
            self.evaluate_condition(left, row) && self.evaluate_condition(right, row)
        }
        WhereCondition::Or(left, right) => {
            self.evaluate_condition(left, row) || self.evaluate_condition(right, row)
        }
        WhereCondition::Not(cond) => {
            !self.evaluate_condition(cond, row)
        }
        _ => true,
    }
}
```

**交付物**：
- ✅ WHERE 子句过滤逻辑
- ✅ 支持所有比较和逻辑操作符
- ✅ 单元测试

**验收标准**：
```sql
SEARCH INDEX idx_article_content 
MATCH 'database'
YIELD doc_id, score()
WHERE score > 0.5 AND doc_id != '123';
```

---

### Phase 2.2: ORDER BY 排序

**目标**：实现搜索结果的排序功能

**工作内容**：

1. **排序解析**（2 小时）
   - 解析 ORDER BY 子句
   - 提取排序字段和方向

2. **排序执行器**（4 小时）
   - 实现多字段排序
   - 支持 ASC/DESC
   - 支持按 score 排序

**关键代码**：

```rust
if let Some(order_clause) = &self.statement.order_clause {
    rows.sort_by(|a, b| {
        for order_item in &order_clause.items {
            let cmp = self.compare_row_values(a, b, &order_item.expr);
            let cmp = if order_item.order == OrderDirection::Desc {
                cmp.reverse()
            } else {
                cmp
            };
            if cmp != Ordering::Equal {
                return cmp;
            }
        }
        Ordering::Equal
    });
}
```

**交付物**：
- ✅ ORDER BY 排序逻辑
- ✅ 支持多字段排序
- ✅ 单元测试

**验收标准**：
```sql
SEARCH INDEX idx_article_content 
MATCH 'database'
YIELD doc_id, score()
ORDER BY score DESC, doc_id ASC;
```

---

### Phase 2.3: LIMIT/OFFSET 分页

**目标**：实现结果分页功能

**工作内容**：

1. **分页解析**（1 小时）
   - 解析 LIMIT 和 OFFSET 子句

2. **分页执行**（2 小时）
   - 应用 OFFSET 跳过记录
   - 应用 LIMIT 限制数量

**关键代码**：

```rust
if let Some(offset) = self.statement.offset {
    rows = rows.into_iter().skip(offset).collect();
}

if let Some(limit) = self.statement.limit {
    rows = rows.into_iter().take(limit).collect();
}
```

**交付物**：
- ✅ LIMIT/OFFSET 逻辑
- ✅ 单元测试

**验收标准**：
```sql
SEARCH INDEX idx_article_content 
MATCH 'database'
YIELD doc_id, score()
ORDER BY score DESC
LIMIT 10 OFFSET 20;
```

---

### Phase 2.4: 表达式函数完整支持

**目标**：实现所有全文搜索表达式函数的完整功能

**工作内容**：

1. **highlight() 函数增强**（4 小时）
   - 支持自定义高亮标签
   - 支持片段大小控制
   - 支持多片段

2. **matched_fields() 函数**（2 小时）
   - 返回匹配的字段列表
   - 用于多字段搜索

3. **snippet() 函数**（3 小时）
   - 生成文本片段
   - 支持最大长度控制
   - 支持省略号

**关键代码**：

```rust
// highlight() 实现
fn execute_highlight(
    &self,
    args: &[Value],
    context: &FulltextExecutionContext,
) -> Result<Value, ExpressionError> {
    let field_name = args[0].as_string()?;
    
    if let Some(highlight) = context.highlights.get(&field_name) {
        Ok(Value::String(highlight.clone()))
    } else {
        Ok(Value::Null)
    }
}

// snippet() 实现
fn execute_snippet(
    &self,
    args: &[Value],
    context: &FulltextExecutionContext,
) -> Result<Value, ExpressionError> {
    let field_name = args[0].as_string()?;
    let max_len = if args.len() > 1 { args[1].as_int()? as usize } else { 200 };
    
    // 生成片段逻辑
    let snippet = self.generate_snippet(&field_name, max_len)?;
    Ok(Value::String(snippet))
}
```

**交付物**：
- ✅ highlight() 完整功能
- ✅ matched_fields() 功能
- ✅ snippet() 功能
- ✅ 单元测试

**验收标准**：
```sql
SEARCH INDEX idx_article_content 
MATCH 'database'
YIELD 
    doc_id,
    score(),
    highlight(content, '<em>', '</em>', 100) as hl,
    snippet(content, 200) as snippet,
    matched_fields();
```

---

### Phase 2 交付清单

| 文件 | 状态 | 说明 |
|------|------|------|
| `src/query/executor/data_access/fulltext_search.rs` | 待增强 | 添加 WHERE/ORDER BY/LIMIT |
| `src/query/executor/data_access/filter.rs` | 待创建 | 过滤器执行器 |
| `src/query/executor/data_access/sort.rs` | 待创建 | 排序执行器 |
| `src/query/executor/expression/functions/fulltext.rs` | 待增强 | 完整函数实现 |
| `tests/fulltext_query_test.rs` | 待创建 | 查询功能测试 |

**验收测试**：
```rust
#[tokio::test]
async fn test_fulltext_complete_query() {
    // 完整查询测试
    let sql = r#"
        SEARCH INDEX idx_article_content 
        MATCH 'database optimization'
        YIELD doc_id, score(), highlight(content)
        WHERE score > 0.5
        ORDER BY score DESC
        LIMIT 10
    "#;
    
    let results = execute_query(sql).await;
    assert!(results.len() <= 10);
    assert!(results.iter().all(|r| r.get("score").unwrap().as_float() > 0.5));
}
```

---

## Phase 3: 存储层集成与自动化 ⭐⭐⭐

**目标**：将全文索引同步集成到存储层，实现数据变更的自动同步，并添加查询优化功能。

**工期**：3-4 天  
**优先级**：P2（可以后续完成）  
**前置依赖**：Phase 2 完成

---

### Phase 3.1: 存储层自动同步

**目标**：在存储层集成全文索引同步，实现数据变更自动更新索引

**工作内容**：

1. **RedbStorage 集成**（6 小时）
   - 在 `insert_vertex()` 中调用 Coordinator
   - 在 `update_vertex()` 中调用 Coordinator
   - 在 `delete_vertex()` 中调用 Coordinator

2. **事务集成**（4 小时）
   - 确保索引同步在事务提交后执行
   - 处理事务回滚情况

3. **批量操作优化**（4 小时）
   - 批量插入时的索引同步优化
   - 使用批量提交减少开销

**关键代码**：

```rust
// src/storage/redb_storage.rs
impl RedbStorage {
    pub async fn insert_vertex(&self, vertex: &Vertex) -> Result<(), StorageError> {
        let space_id = self.get_space_id()?;
        
        // 1. 写入存储
        let mut txn = self.db.begin_write()?;
        // ... 写入逻辑
        txn.commit()?;
        
        // 2. 同步到全文索引
        if let Some(coordinator) = &self.fulltext_coordinator {
            coordinator.on_vertex_inserted(space_id, vertex).await?;
            coordinator.commit_all().await?;
        }
        
        Ok(())
    }
    
    pub async fn update_vertex(&self, vertex: &Vertex, changed_fields: &[String]) -> Result<(), StorageError> {
        // 1. 更新存储
        // ...
        
        // 2. 同步索引
        if let Some(coordinator) = &self.fulltext_coordinator {
            coordinator.on_vertex_updated(space_id, vertex, changed_fields).await?;
            coordinator.commit_all().await?;
        }
        
        Ok(())
    }
}
```

**交付物**：
- ✅ RedbStorage 集成代码
- ✅ 事务集成
- ✅ 批量操作优化
- ✅ 集成测试

**验收标准**：
```rust
#[tokio::test]
async fn test_auto_sync_on_insert() {
    // 1. 创建索引
    // 2. 插入顶点
    storage.insert_vertex(&vertex).await?;
    
    // 3. 直接搜索，验证索引已自动更新
    let results = storage.fulltext_search("idx", "keyword").await?;
    assert_eq!(results.len(), 1);
}
```

---

### Phase 3.2: 查询优化

**目标**：实现全文搜索查询的优化功能

**工作内容**：

1. **索引选择优化**（4 小时）
   - 多索引时选择最优索引
   - 基于选择性的索引推荐

2. **谓词下推**（4 小时）
   - 将 WHERE 条件下推到搜索引擎
   - 减少结果传输量

3. **结果缓存**（4 小时）
   - 实现查询结果缓存
   - 缓存失效策略

**交付物**：
- ✅ 索引选择优化
- ✅ 谓词下推实现
- ✅ 结果缓存机制

**验收标准**：
- 查询性能提升 30%+
- 缓存命中率 > 50%

---

### Phase 3.3: 查询计划优化

**目标**：优化全文搜索的查询计划

**工作内容**：

1. **计划重写规则**（4 小时）
   - Filter 下推到 FulltextSearchNode
   - Limit 下推到 FulltextSearchNode

2. **并行搜索**（4 小时）
   - 多索引并行搜索
   - 结果合并优化

**交付物**：
- ✅ 计划重写规则
- ✅ 并行搜索支持

---

### Phase 3 交付清单

| 文件 | 状态 | 说明 |
|------|------|------|
| `src/storage/redb_storage.rs` | 待增强 | 添加 Coordinator 集成 |
| `src/storage/mod.rs` | 待增强 | 添加 fulltext_coordinator 字段 |
| `src/query/planning/rewrite/fulltext_optimizer.rs` | 待创建 | 查询优化器 |
| `src/query/cache/fulltext_cache.rs` | 待创建 | 结果缓存 |
| `tests/fulltext_auto_sync_test.rs` | 待创建 | 自动同步测试 |

---

## Phase 4: 测试、优化与文档 ⭐⭐

**目标**：完善测试覆盖，进行性能优化，编写用户文档。

**工期**：2-3 天  
**优先级**：P2（可以后续完成）  
**前置依赖**：Phase 3 完成

---

### Phase 4.1: 测试完善

**目标**：完善测试覆盖，确保质量

**工作内容**：

1. **单元测试**（4 小时）
   - 执行器单元测试
   - 表达式函数单元测试
   - 转换器单元测试

2. **集成测试**（6 小时）
   - 完整查询流程测试
   - 多用户并发测试
   - 异常场景测试

3. **性能测试**（4 小时）
   - 基准测试
   - 负载测试
   - 压力测试

**交付物**：
- ✅ 单元测试覆盖率 > 80%
- ✅ 集成测试套件
- ✅ 性能测试报告

---

### Phase 4.2: 性能优化

**目标**：优化全文搜索性能

**工作内容**：

1. **性能分析**（4 小时）
   - 使用 perf/criterion 分析性能瓶颈
   - 识别热点代码

2. **优化实施**（6 小时）
   - 优化内存使用
   - 优化锁竞争
   - 优化 I/O 操作

3. **配置调优**（2 小时）
   - 优化默认配置
   - 提供性能调优指南

**交付物**：
- ✅ 性能分析报告
- ✅ 优化实施
- ✅ 配置调优指南

---

### Phase 4.3: 文档编写

**目标**：编写完整的用户文档和开发文档

**工作内容**：

1. **用户指南**（4 小时）
   - 快速开始
   - SQL 语法说明
   - 最佳实践

2. **API 文档**（3 小时）
   - Rust API 文档
   - 使用示例

3. **开发文档**（3 小时）
   - 架构说明
   - 实现细节
   - 扩展指南

**交付物**：
- ✅ `docs/user_guide/fulltext_search.md`
- ✅ `docs/api/fulltext_api.md`
- ✅ `docs/develop/fulltext_implementation.md`

---

### Phase 4 交付清单

| 文件 | 状态 | 说明 |
|------|------|------|
| `tests/fulltext_unit_test.rs` | 待创建 | 单元测试 |
| `tests/fulltext_integration_test.rs` | 待创建 | 集成测试 |
| `benches/fulltext_benchmark.rs` | 待增强 | 性能基准 |
| `docs/user_guide/fulltext_search.md` | 待创建 | 用户指南 |
| `docs/api/fulltext_api.md` | 待创建 | API 文档 |

---

## 总结

### 实施路线图总览

| 阶段 | 工期 | 优先级 | 主要交付物 |
|------|------|--------|------------|
| **Phase 1** | 3-4 天 | P0 | 执行器核心实现 |
| **Phase 2** | 4-5 天 | P1 | 查询处理增强 |
| **Phase 3** | 3-4 天 | P2 | 存储层集成与优化 |
| **Phase 4** | 2-3 天 | P2 | 测试、优化与文档 |
| **总计** | 12-16 天 | - | 完整可用的全文搜索功能 |

### 关键里程碑

1. **M1 (Phase 1 结束)**: SEARCH 语句可以执行并返回结果
2. **M2 (Phase 2 结束)**: 所有全文搜索语法都可以正常使用
3. **M3 (Phase 3 结束)**: 数据自动同步，性能优化完成
4. **M4 (Phase 4 结束)**: 生产就绪，文档完善

### 风险控制

| 风险 | 缓解措施 |
|------|----------|
| 执行器实现复杂 | 分阶段实施，先实现基本功能 |
| 性能不达标 | 早期性能测试，及时优化 |
| 测试覆盖不足 | 同步编写测试，确保覆盖率 |
| 工期延长 | 明确优先级，必要时削减范围 |

---

**文档结束**
