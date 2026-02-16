# MATCH 语句功能文档

本文档详细描述 GraphDB 项目中 MATCH 语句的所有功能实现。

## 目录

1. [概述](#概述)
2. [语法支持](#语法支持)
3. [核心功能](#核心功能)
4. [高级特性](#高级特性)
5. [实现架构](#实现架构)
6. [使用示例](#使用示例)

---

## 概述

MATCH 语句是 Cypher 查询语言的核心，用于在图数据库中匹配节点和边的模式。GraphDB 实现了完整的 MATCH 语句功能，包括基本模式匹配、可选匹配、最短路径查询等高级特性。

### 支持的 Cypher 标准

- OpenCypher 标准兼容
- 支持 Nebula Graph 扩展语法
- 支持自定义扩展

---

## 语法支持

### 1. 基本 MATCH 语法

```cypher
MATCH (n:Label)
RETURN n

MATCH (n:Label {property: value})
RETURN n

MATCH (a:Label1)-[:REL_TYPE]->(b:Label2)
RETURN a, b
```

### 2. OPTIONAL MATCH 语法

```cypher
OPTIONAL MATCH (n:Label)-[:REL_TYPE]->(m:OtherLabel)
RETURN n, m
```

**实现状态**: ✅ 已实现
- 解析器支持 `OPTIONAL MATCH` 关键字
- 规划器使用左连接（Left Join）实现
- 文件位置: `src/query/parser/parser/stmt_parser.rs`, `src/query/planner/statements/core/match_clause_planner.rs`

### 3. 最短路径查询

```cypher
MATCH p = shortestPath((a:Label)-[:REL_TYPE*..5]->(b:Label))
RETURN p

MATCH p = allShortestPaths((a:Label)-[:REL_TYPE*..5]->(b:Label))
RETURN p
```

**实现状态**: ✅ 已实现
- 支持 `shortestPath` 单条最短路径
- 支持 `allShortestPaths` 所有最短路径
- 使用双向 BFS 算法优化
- 文件位置: `src/query/planner/statements/core/match_clause_planner.rs`

### 4. 可变长度路径

```cypher
MATCH (a:Label)-[:REL_TYPE*2..5]->(b:Label)
RETURN a, b

MATCH (a:Label)-[:REL_TYPE*]->(b:Label)
RETURN a, b
```

**实现状态**: ✅ 已实现
- 支持指定最小/最大步数
- 支持无限长度路径（*）
- 文件位置: `src/query/validator/structs/path_structs.rs`

---

## 核心功能

### 1. 节点模式匹配

**功能描述**: 匹配具有特定标签和属性的节点

**实现细节**:
- 支持多标签匹配
- 支持属性过滤
- 支持匿名节点
- 文件位置: `src/query/validator/structs/path_structs.rs` (NodeInfo)

```rust
pub struct NodeInfo {
    pub alias: String,
    pub labels: Vec<String>,
    pub props: Option<Expression>,
    pub anonymous: bool,
    pub filter: Option<Expression>,
    pub tids: Vec<i32>,
    pub label_props: Vec<Option<Expression>>,
}
```

### 2. 边模式匹配

**功能描述**: 匹配具有特定类型和方向的边

**实现细节**:
- 支持多类型边
- 支持方向指定（->, <-, -）
- 支持属性过滤
- 支持可变长度
- 文件位置: `src/query/validator/structs/path_structs.rs` (EdgeInfo)

```rust
pub struct EdgeInfo {
    pub alias: String,
    pub inner_alias: String,
    pub types: Vec<String>,
    pub props: Option<Expression>,
    pub anonymous: bool,
    pub filter: Option<Expression>,
    pub direction: Direction,
    pub range: Option<MatchStepRange>,
    pub edge_types: Vec<i32>,
}
```

### 3. WHERE 子句过滤

**功能描述**: 对匹配结果进行条件过滤

**实现状态**: ✅ 已实现
- 支持复杂表达式
- 支持逻辑运算符（AND, OR, NOT）
- 支持比较运算符
- 文件位置: `src/query/validator/structs/clause_structs.rs`

### 4. 返回子句 (RETURN)

**功能描述**: 指定查询返回的结果

**实现状态**: ✅ 已实现
- 支持返回节点、边、路径
- 支持表达式计算
- 支持别名
- 支持 DISTINCT
- 支持聚合函数
- 文件位置: `src/query/parser/ast/stmt.rs`

### 5. 排序和分页

**功能描述**: 对查询结果进行排序和限制

**实现状态**: ✅ 已实现
- ORDER BY 排序
- SKIP 跳过指定行数
- LIMIT 限制返回行数
- 文件位置: `src/query/validator/structs/path_structs.rs` (PaginationContext)

---

## 高级特性

### 1. 智能起点选择 (StartVidFinder)

**功能描述**: 自动选择最优的查询起点，提高查询效率

**实现状态**: ✅ 已实现

**策略优先级**:
1. **VertexSeek** - 显式 VID 查找（最高优先级）
2. **IndexSeek** - 索引查找
3. **ScanSeek** - 全表扫描（最低优先级）

**选择逻辑**:
```rust
pub fn select_strategy<S: StorageClient + ?Sized>(
    &self,
    _storage: &S,
    context: &SeekStrategyContext,
) -> SeekStrategyType {
    if context.has_explicit_vid() {
        SeekStrategyType::VertexSeek
    } else if let Some(_) = context.get_index_for_labels(&context.node_pattern.labels) {
        if context.estimated_rows < self.scan_threshold {
            SeekStrategyType::IndexSeek
        } else {
            SeekStrategyType::ScanSeek
        }
    } else if context.estimated_rows < self.use_index_threshold {
        SeekStrategyType::VertexSeek
    } else {
        SeekStrategyType::ScanSeek
    }
}
```

**文件位置**:
- `src/query/planner/statements/seeks/seek_strategy_base.rs`
- `src/query/planner/statements/paths/match_path_planner.rs`

### 2. 双向遍历优化

**功能描述**: 使用双向 BFS 算法优化最短路径查询

**实现状态**: ✅ 已实现

**算法特点**:
- 从起点和终点同时开始搜索
- 显著减少搜索空间
- 提高最短路径查找效率

**文件位置**:
- `src/query/executor/data_processing/graph_traversal/algorithms/bidirectional_bfs.rs`

### 3. 模式表达式 (Pattern Apply)

**功能描述**: 将路径模式作为谓词使用，用于嵌套查询

**实现状态**: ✅ 已实现

**类型支持**:
- `is_pred` - 普通模式谓词
- `is_anti_pred` - 反向模式谓词（NOT EXISTS）

**实现细节**:
```rust
pub struct Path {
    // ... 其他字段
    pub is_pred: bool,                  // 是否为谓词
    pub is_anti_pred: bool,             // 是否为反向谓词
    // ...
}
```

**文件位置**:
- `src/query/validator/structs/path_structs.rs`
- `src/query/planner/statements/core/match_clause_planner.rs`
- `src/query/planner/plan/core/nodes/data_processing_node.rs` (PatternApplyNode)

### 4. 多 MATCH 子句优化 (QueryPart)

**功能描述**: 支持多个 MATCH 子句的连接和优化

**实现状态**: ✅ 已实现

**结构定义**:
```rust
pub struct QueryPart {
    pub matchs: Vec<MatchClauseContext>,
    pub boundary: Option<BoundaryClauseContext>,
    pub aliases_available: HashMap<String, AliasType>,
    pub aliases_generated: HashMap<String, AliasType>,
    pub paths: Vec<Path>,
}
```

**文件位置**:
- `src/query/validator/structs/alias_structs.rs`
- `src/query/validator/structs/clause_structs.rs`

### 5. 连接策略

**功能描述**: 支持多种连接方式处理多个路径

**实现状态**: ✅ 已实现

**连接类型**:
1. **Cross Join** - 交叉连接（无共享别名）
2. **Inner Join** - 内连接（有共享别名）
3. **Left Join** - 左连接（OPTIONAL MATCH）

**文件位置**:
- `src/query/planner/connector.rs`

---

## 实现架构

### 1. 解析层 (Parser)

**职责**: 将 Cypher 查询字符串解析为 AST

**关键文件**:
- `src/query/parser/parser/stmt_parser.rs` - 语句解析
- `src/query/parser/ast/stmt.rs` - AST 定义

**主要结构**:
```rust
pub struct MatchStmt {
    pub span: Span,
    pub patterns: Vec<Pattern>,
    pub where_clause: Option<Expression>,
    pub return_clause: Option<ReturnClause>,
    pub order_by: Option<OrderByClause>,
    pub limit: Option<usize>,
    pub skip: Option<usize>,
    pub optional: bool,  // 是否为 OPTIONAL MATCH
}
```

### 2. 验证层 (Validator)

**职责**: 验证查询的语义正确性

**关键文件**:
- `src/query/validator/match_validator.rs` - MATCH 验证器
- `src/query/validator/structs/` - 验证数据结构

**验证策略**:
- 别名验证
- 类型推断
- 表达式重写
- 聚合函数验证

### 3. 规划层 (Planner)

**职责**: 生成查询执行计划

**关键文件**:
- `src/query/planner/statements/core/match_clause_planner.rs` - MATCH 规划器
- `src/query/planner/connector.rs` - 连接策略

**规划流程**:
1. 解析路径模式
2. 选择起点策略
3. 生成扫描/扩展节点
4. 应用过滤条件
5. 连接多个路径
6. 处理 OPTIONAL MATCH（左连接）

### 4. 执行层 (Executor)

**职责**: 执行查询计划

**关键文件**:
- `src/query/executor/data_processing/graph_traversal/` - 图遍历执行
- `src/query/executor/data_processing/graph_traversal/algorithms/bidirectional_bfs.rs` - 双向 BFS

---

## 使用示例

### 示例 1: 基本节点查询

```cypher
MATCH (n:Person)
RETURN n
```

### 示例 2: 带属性的节点查询

```cypher
MATCH (n:Person {name: 'Alice', age: 30})
RETURN n
```

### 示例 3: 关系查询

```cypher
MATCH (a:Person)-[:KNOWS]->(b:Person)
RETURN a.name, b.name
```

### 示例 4: 带方向的边查询

```cypher
MATCH (a:Person)<-[:FOLLOWS]-(b:Person)
RETURN a.name, b.name
```

### 示例 5: 可变长度路径

```cypher
MATCH (a:Person)-[:KNOWS*2..4]->(b:Person)
RETURN a.name, b.name
```

### 示例 6: 最短路径

```cypher
MATCH p = shortestPath((a:Person {name: 'Alice'})-[:KNOWS*..5]->(b:Person {name: 'Bob'}))
RETURN p
```

### 示例 7: OPTIONAL MATCH

```cypher
MATCH (a:Person)
OPTIONAL MATCH (a)-[:WORKS_AT]->(c:Company)
RETURN a.name, c.name
```

### 示例 8: WHERE 子句

```cypher
MATCH (n:Person)
WHERE n.age > 25 AND n.name STARTS WITH 'A'
RETURN n.name, n.age
```

### 示例 9: 多路径连接

```cypher
MATCH (a:Person)-[:KNOWS]->(b:Person)
MATCH (b)-[:WORKS_AT]->(c:Company)
RETURN a.name, b.name, c.name
```

### 示例 10: 排序和分页

```cypher
MATCH (n:Person)
RETURN n.name, n.age
ORDER BY n.age DESC
SKIP 10
LIMIT 20
```

---

## 性能优化

### 1. 索引使用

- 自动检测可用索引
- 优先使用索引查找
- 回退到全表扫描

### 2. 查询计划优化

- 智能起点选择
- 双向遍历优化
- 连接顺序优化

### 3. 执行优化

- 双向 BFS 最短路径
- 惰性结果生成
- 内存使用优化

---

## 限制和注意事项

1. **递归深度**: 可变长度路径有最大深度限制（默认 5 步）
2. **内存使用**: 大规模图的查询可能消耗大量内存
3. **索引依赖**: 复杂查询性能依赖于适当的索引

---

## 相关文件

### 解析器
- `src/query/parser/parser/stmt_parser.rs`
- `src/query/parser/ast/stmt.rs`
- `src/query/parser/ast/pattern.rs`

### 验证器
- `src/query/validator/match_validator.rs`
- `src/query/validator/structs/path_structs.rs`
- `src/query/validator/structs/clause_structs.rs`
- `src/query/validator/structs/alias_structs.rs`

### 规划器
- `src/query/planner/statements/core/match_clause_planner.rs`
- `src/query/planner/statements/paths/match_path_planner.rs`
- `src/query/planner/statements/paths/shortest_path_planner.rs`
- `src/query/planner/statements/seeks/seek_strategy_base.rs`
- `src/query/planner/connector.rs`

### 执行器
- `src/query/executor/data_processing/graph_traversal/algorithms/bidirectional_bfs.rs`
- `src/query/executor/data_processing/graph_traversal/traverse.rs`

---

*文档生成时间: 2026-02-16*
*版本: 1.0*
