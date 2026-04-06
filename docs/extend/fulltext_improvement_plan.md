# 全文检索功能完善方案

**文档版本**: 1.0  
**创建日期**: 2026-04-06  
**状态**: 实施中

---

## 一、背景与目标

### 1.1 当前状态分析

经过对 GraphDB 全文检索功能的全面分析，当前实现状态如下：

| 模块 | 完成度 | 说明 |
|------|--------|------|
| 搜索引擎层 | 100% | 双引擎支持，功能完整 |
| 协调器层 | 100% | 数据同步、索引管理完整 |
| 查询语法 | 95% | 语法定义完整，解析器工作正常 |
| 执行器层 | 70% | 基本功能可用，缺少高级特性 |
| 表达式函数 | 90% | 核心函数实现，集成待完善 |
| 测试覆盖 | 85% | Coordinator 测试完整，执行器测试不足 |

**总体完成度：约 85%**

### 1.2 核心问题

1. **执行器缺少 WHERE 过滤**：无法在搜索结果上应用额外条件
2. **执行器缺少 ORDER BY 排序**：无法自定义排序规则
3. **执行器测试覆盖不足**：缺少执行器单元测试
4. **表达式函数集成不完整**：函数实现完整但执行器未充分利用

### 1.3 目标

完善全文检索执行器功能，实现完整的 SQL 查询支持：

- ✅ WHERE 条件过滤
- ✅ ORDER BY 自定义排序
- ✅ 完整的 YIELD 子句处理
- ✅ 执行器单元测试

---

## 二、修改方案

### 2.1 FulltextSearchExecutor 增强

#### 2.1.1 WHERE 过滤实现

**修改文件**: `src/query/executor/data_access/fulltext_search.rs`

**实现思路**：
1. 在获取搜索结果后，应用 WHERE 条件进行过滤
2. 支持对 score、字段值等进行条件判断
3. 使用表达式求值器处理复杂条件

**关键代码**：

```rust
impl<S: StorageClient> Executor<S> for FulltextSearchExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // ... 现有搜索逻辑 ...
        
        // 应用 WHERE 过滤
        if let Some(where_clause) = &self.statement.where_clause {
            rows = rows.into_iter()
                .filter(|row| self.evaluate_where_condition(row, where_clause))
                .collect();
        }
        
        // ... 后续处理 ...
    }
    
    fn evaluate_where_condition(&self, row: &HashMap<String, Value>, condition: &WhereCondition) -> bool {
        match condition {
            WhereCondition::Simple(expr) => {
                self.evaluate_expression(row, expr)
            }
            WhereCondition::And(left, right) => {
                self.evaluate_where_condition(row, left) && 
                self.evaluate_where_condition(row, right)
            }
            WhereCondition::Or(left, right) => {
                self.evaluate_where_condition(row, left) || 
                self.evaluate_where_condition(row, right)
            }
            WhereCondition::Not(inner) => {
                !self.evaluate_where_condition(row, inner)
            }
        }
    }
}
```

#### 2.1.2 ORDER BY 排序实现

**实现思路**：
1. 解析 ORDER BY 子句
2. 支持多字段排序
3. 支持 ASC/DESC 方向
4. 默认按 score DESC 排序

**关键代码**：

```rust
impl<S: StorageClient> Executor<S> for FulltextSearchExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        // ... 搜索和过滤逻辑 ...
        
        // 应用 ORDER BY 排序
        if let Some(order_clause) = &self.statement.order_clause {
            rows.sort_by(|a, b| {
                for item in &order_clause.items {
                    let cmp = self.compare_rows(a, b, item);
                    if cmp != Ordering::Equal {
                        return if item.direction == FulltextOrderDirection::Desc {
                            cmp.reverse()
                        } else {
                            cmp
                        };
                    }
                }
                Ordering::Equal
            });
        } else {
            // 默认按 score DESC 排序
            rows.sort_by(|a, b| {
                let score_a = a.get("score").and_then(|v| v.as_float()).unwrap_or(0.0);
                let score_b = b.get("score").and_then(|v| v.as_float()).unwrap_or(0.0);
                score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
            });
        }
        
        // ... 分页逻辑 ...
    }
    
    fn compare_rows(&self, a: &HashMap<String, Value>, b: &HashMap<String, Value>, item: &OrderItem) -> Ordering {
        let val_a = a.get(&item.field);
        let val_b = b.get(&item.field);
        
        match (val_a, val_b) {
            (Some(va), Some(vb)) => self.compare_values(va, vb),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (None, None) => Ordering::Equal,
        }
    }
}
```

### 2.2 FulltextScanExecutor 增强

**修改文件**: `src/query/executor/data_access/fulltext_search.rs`

**实现内容**：
- 添加 WHERE 过滤支持
- 改进 YIELD 子句处理
- 优化结果构建逻辑

### 2.3 MatchFulltextExecutor 增强

**修改文件**: `src/query/executor/data_access/match_fulltext.rs`

**实现内容**：
- 完善 WHERE 条件处理
- 添加 ORDER BY 支持
- 改进 YIELD 子句处理

### 2.4 表达式求值器集成

**修改文件**: `src/query/executor/data_access/fulltext_search.rs`

**实现思路**：
1. 复用现有的表达式求值器
2. 支持 score()、highlight() 等函数
3. 支持字段值比较

---

## 三、实施计划

### Phase 1: WHERE 过滤支持（预计 2-3 小时）

**任务清单**：
- [ ] 实现 WHERE 条件解析和求值
- [ ] 支持简单条件（字段比较）
- [ ] 支持复合条件（AND/OR/NOT）
- [ ] 支持 score 相关条件

**验收标准**：
```sql
SEARCH INDEX idx_article MATCH 'database' 
YIELD doc_id, score() 
WHERE score > 0.5
LIMIT 10;
```

### Phase 2: ORDER BY 排序支持（预计 2-3 小时）

**任务清单**：
- [ ] 实现 ORDER BY 子句解析
- [ ] 支持单字段排序
- [ ] 支持多字段排序
- [ ] 支持 ASC/DESC 方向

**验收标准**：
```sql
SEARCH INDEX idx_article MATCH 'database' 
YIELD doc_id, score() AS s
ORDER BY s DESC
LIMIT 10;
```

### Phase 3: 测试完善（预计 2-3 小时）

**任务清单**：
- [ ] 编写 FulltextSearchExecutor 单元测试
- [ ] 编写 FulltextScanExecutor 单元测试
- [ ] 编写 MatchFulltextExecutor 单元测试
- [ ] 编写端到端集成测试

**验收标准**：
- 所有测试通过
- 测试覆盖率 > 80%

---

## 四、技术细节

### 4.1 WHERE 条件求值

**支持的运算符**：
- 比较运算符：`=`, `!=`, `<`, `>`, `<=`, `>=`
- 逻辑运算符：`AND`, `OR`, `NOT`
- 特殊运算符：`LIKE`, `IN`, `IS NULL`, `IS NOT NULL`

**表达式类型**：
- 字段引用：`field_name`
- 常量：`123`, `'string'`, `true`
- 函数调用：`score()`, `highlight(field)`
- 算术表达式：`field + 1`

### 4.2 ORDER BY 排序规则

**排序优先级**：
1. 按 ORDER BY 子句指定的字段顺序
2. 默认按 score DESC 排序
3. NULL 值处理：NULL 排在最后

**数据类型排序**：
- 数值类型：按数值大小
- 字符串类型：按字典序
- 时间类型：按时间先后

### 4.3 性能考虑

**优化策略**：
1. 先应用 LIMIT 再排序（如果可能）
2. 使用快速排序算法
3. 缓存中间结果
4. 并行处理（未来优化）

---

## 五、风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 表达式求值复杂 | 中 | 中 | 复用现有表达式求值器 |
| 性能下降 | 中 | 低 | 添加性能测试，优化关键路径 |
| 兼容性问题 | 低 | 低 | 保持向后兼容，添加版本检查 |
| 测试覆盖不足 | 中 | 中 | 同步编写单元测试和集成测试 |

---

## 六、验收标准

### 6.1 功能验收

- [ ] WHERE 条件过滤正常工作
- [ ] ORDER BY 排序正常工作
- [ ] 所有 SQL 语法都能正确执行
- [ ] 错误处理完善

### 6.2 性能验收

- [ ] 查询响应时间 < 100ms（小数据集）
- [ ] 内存使用合理
- [ ] 无明显性能回归

### 6.3 测试验收

- [ ] 单元测试覆盖率 > 80%
- [ ] 集成测试全部通过
- [ ] 性能测试通过

---

## 七、后续优化方向

### 7.1 短期优化

1. **查询缓存**：缓存常用查询结果
2. **批量处理**：优化批量索引性能
3. **错误提示**：改进错误消息

### 7.2 中期优化

1. **查询优化器**：实现查询计划优化
2. **索引选择**：自动选择最优索引
3. **谓词下推**：将过滤条件下推到搜索引擎

### 7.3 长期优化

1. **分布式搜索**：支持分布式全文检索
2. **实时索引**：实现近实时索引更新
3. **机器学习**：智能相关性排序

---

## 八、参考资料

- [全文索引功能查询工作流集成分析报告](./fulltext_query_integration_analysis.md)
- [全文检索架构设计决策文档](./fulltext_architecture_decision.md)
- [全文检索嵌入式集成分析报告](./fulltext_embedding_analysis.md)
- [Phase 3: 查询引擎集成方案](./plan/phase3_query_engine_integration.md)

---

**文档结束**
