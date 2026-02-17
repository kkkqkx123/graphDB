# GraphDB DQL 功能改进方案

## 概述

本文档详细描述了 GraphDB 查询语言（DQL）的功能改进方案，包括 YIELD 语句增强、SUBGRAPH 方向分离、路径功能补充等。

---

## 1. YIELD 语句增强

### 1.1 目标

支持 `YIELD ... WHERE ...` 语法，允许在 YIELD 后对结果进行过滤。

### 1.2 AST 修改

```rust
// src/query/parser/ast/stmt.rs
pub struct YieldStmt {
    pub span: Span,
    pub items: Vec<YieldItem>,
    pub where_clause: Option<Expression>,   // 新增：YIELD 后的 WHERE 过滤
    pub distinct: bool,
}

pub struct YieldClause {
    pub span: Span,
    pub items: Vec<YieldItem>,
    pub where_clause: Option<Expression>,   // 新增
    pub limit: Option<LimitClause>,
    pub skip: Option<SkipClause>,
    pub sample: Option<SampleClause>,
}
```

### 1.3 Parser 修改

```rust
// src/query/parser/parser/clause_parser.rs
pub fn parse_yield_clause(&mut self, ctx: &mut ParseContext) -> Result<YieldClause, ParseError> {
    let span = ctx.current_span();
    ctx.expect_token(TokenKind::Yield)?;
    
    let mut items = Vec::new();
    if ctx.match_token(TokenKind::Star) {
        // YIELD *
    } else {
        loop {
            let expr = self.parse_expression(ctx)?;
            let alias = if ctx.match_token(TokenKind::As) {
                Some(ctx.expect_identifier()?)
            } else {
                None
            };
            items.push(YieldItem { expression: expr, alias });
            if !ctx.match_token(TokenKind::Comma) {
                break;
            }
        }
    }
    
    // 新增：解析 WHERE 子句
    let where_clause = if ctx.match_token(TokenKind::Where) {
        Some(self.parse_expression(ctx)?)
    } else {
        None
    };
    
    // 解析 LIMIT
    let limit = if ctx.match_token(TokenKind::Limit) {
        let count = ctx.expect_integer_literal()? as usize;
        Some(LimitClause { span: ctx.current_span(), count })
    } else {
        None
    };
    
    // 解析 SKIP
    let skip = if ctx.match_token(TokenKind::Skip) {
        let count = ctx.expect_integer_literal()? as usize;
        Some(SkipClause { span: ctx.current_span(), count })
    } else {
        None
    };
    
    Ok(YieldClause { span, items, where_clause, limit, skip, sample: None })
}
```

### 1.4 Planner 修改

```rust
// src/query/planner/statements/clauses/yield_planner.rs
pub struct YieldClausePlanner;

impl YieldClausePlanner {
    pub fn plan_yield_clause(
        &self,
        input: SubPlan,
        yield_clause: &YieldClause,
        space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        let mut current = input;
        
        // 1. 构建投影节点
        current = self.plan_project(current, &yield_clause.items, space_id)?;
        
        // 2. 如有 WHERE，添加 Filter 节点
        if let Some(ref condition) = yield_clause.where_clause {
            current = self.plan_filter(current, condition, space_id)?;
        }
        
        // 3. 如有 LIMIT/SKIP，添加 Limit 节点
        if yield_clause.limit.is_some() || yield_clause.skip.is_some() {
            current = self.plan_pagination(current, yield_clause.skip, yield_clause.limit)?;
        }
        
        Ok(current)
    }
}
```

### 1.5 使用示例

```sql
-- YIELD 后过滤
GO 1 STEP FROM "player100" OVER follow YIELD dst(edge) AS id, rank(edge) AS rank
WHERE rank > 0

-- 等效于
GO 1 STEP FROM "player100" OVER follow 
YIELD dst(edge) AS id, rank(edge) AS rank 
| YIELD id, rank WHERE rank > 0
```

---

## 2. YIELD JOIN 集成

### 2.1 目标

支持 `YIELD ... | JOIN ON ...` 语法，允许在 YIELD 后进行 JOIN 操作。

### 2.2 AST 修改

```rust
// src/query/parser/ast/stmt.rs
pub struct YieldStmt {
    pub span: Span,
    pub items: Vec<YieldItem>,
    pub where_clause: Option<Expression>,
    pub join_clause: Option<JoinClause>,    // 新增：JOIN 子句
    pub distinct: bool,
}

pub struct JoinClause {
    pub span: Span,
    pub join_type: JoinType,
    pub right_stmt: Box<Stmt>,              // JOIN 右侧的语句
    pub left_keys: Vec<Expression>,         // 左表连接键
    pub right_keys: Vec<Expression>,        // 右表连接键
    pub where_clause: Option<Expression>,   // JOIN 后的过滤条件
}

pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}
```

### 2.3 Parser 修改

```rust
// src/query/parser/parser/clause_parser.rs
pub fn parse_join_clause(&mut self, ctx: &mut ParseContext) -> Result<JoinClause, ParseError> {
    let span = ctx.current_span();
    
    // 解析 JOIN 类型
    let join_type = if ctx.match_token(TokenKind::Inner) {
        JoinType::Inner
    } else if ctx.match_token(TokenKind::Left) {
        JoinType::Left
    } else if ctx.match_token(TokenKind::Right) {
        JoinType::Right
    } else if ctx.match_token(TokenKind::Full) {
        JoinType::Full
    } else {
        JoinType::Inner  // 默认内连接
    };
    
    ctx.expect_token(TokenKind::Join)?;
    
    // 解析右侧语句（通常是另一个查询）
    let right_stmt = Box::new(self.parse_statement(ctx)?);
    
    // 解析 ON 条件
    ctx.expect_token(TokenKind::On)?;
    let (left_keys, right_keys) = self.parse_join_keys(ctx)?;
    
    // 可选的 WHERE 子句
    let where_clause = if ctx.match_token(TokenKind::Where) {
        Some(self.parse_expression(ctx)?)
    } else {
        None
    };
    
    Ok(JoinClause { span, join_type, right_stmt, left_keys, right_keys, where_clause })
}
```

### 2.4 Planner 修改

```rust
// src/query/planner/statements/clauses/join_planner.rs
pub struct JoinClausePlanner;

impl JoinClausePlanner {
    pub fn plan_join_clause(
        &self,
        left_plan: SubPlan,
        join_clause: &JoinClause,
        space_id: i32,
    ) -> Result<SubPlan, PlannerError> {
        // 1. 规划右侧语句
        let right_plan = self.plan_right_stmt(&join_clause.right_stmt, space_id)?;
        
        // 2. 创建 Join 节点
        let join_node = match join_clause.join_type {
            JoinType::Inner => InnerJoinNode::new(
                left_plan.root()?.clone(),
                right_plan.root()?.clone(),
                join_clause.left_keys.clone(),
                join_clause.right_keys.clone(),
            )?,
            JoinType::Left => LeftJoinNode::new(
                left_plan.root()?.clone(),
                right_plan.root()?.clone(),
                join_clause.left_keys.clone(),
                join_clause.right_keys.clone(),
            )?,
            // ... 其他类型
        };
        
        let mut result = SubPlan::new(Some(join_node.into_enum()), None);
        
        // 3. 如有 WHERE，添加 Filter 节点
        if let Some(ref condition) = join_clause.where_clause {
            result = self.plan_filter(result, condition, space_id)?;
        }
        
        Ok(result)
    }
}
```

### 2.5 使用示例

```sql
-- 基本 JOIN
GO 1 STEP FROM "player100" OVER follow YIELD dst(edge) AS friend
| JOIN 
  GO 1 STEP FROM "player101" OVER follow YIELD dst(edge) AS friend
ON $-.friend == $-.friend

-- 左外连接
GO 1 STEP FROM "player100" OVER follow YIELD src(edge) AS player, dst(edge) AS friend
| LEFT JOIN 
  FETCH PROP ON player $-.friend YIELD player.name
ON $-.friend == $-.player
```

---

## 3. SUBGRAPH 方向分离

### 3.1 目标

支持 `GET SUBGRAPH IN ... OUT ... BOTH ...` 语法，分别指定不同方向的边类型。

### 3.2 AST 修改

```rust
// src/query/parser/ast/stmt.rs
pub struct SubgraphStmt {
    pub span: Span,
    pub steps: Steps,
    pub from: FromClause,
    pub in_edges: Option<Vec<String>>,      // 新增：入边类型
    pub out_edges: Option<Vec<String>>,     // 新增：出边类型
    pub both_edges: Option<Vec<String>>,    // 新增：双向边类型
    pub where_clause: Option<Expression>,
    pub yield_clause: Option<YieldClause>,
    pub with_prop: bool,                    // 新增：携带属性
}
```

### 3.3 Parser 修改

```rust
// src/query/parser/parser/traversal_parser.rs
pub fn parse_subgraph_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
    let start_span = ctx.current_span();
    ctx.expect_token(TokenKind::Get)?;
    
    // 解析 WITH PROP
    let with_prop = ctx.match_token(TokenKind::With) && ctx.match_token(TokenKind::Prop);
    
    ctx.expect_token(TokenKind::Subgraph)?;
    
    // 解析步数
    let steps = if ctx.match_token(TokenKind::Step) {
        self.parse_steps(ctx)?
    } else {
        Steps::Fixed(1)
    };
    
    ctx.expect_token(TokenKind::From)?;
    let from_span = ctx.current_span();
    let vertices = self.parse_expression_list(ctx)?;
    let from_clause = FromClause { span: from_span, vertices };
    
    // 解析 IN/OUT/BOTH 子句
    let mut in_edges = None;
    let mut out_edges = None;
    let mut both_edges = None;
    
    loop {
        if ctx.match_token(TokenKind::In) {
            ctx.expect_token(TokenKind::Over)?;
            in_edges = Some(self.parse_edge_types(ctx)?);
        } else if ctx.match_token(TokenKind::Out) {
            ctx.expect_token(TokenKind::Over)?;
            out_edges = Some(self.parse_edge_types(ctx)?);
        } else if ctx.match_token(TokenKind::Both) {
            ctx.expect_token(TokenKind::Over)?;
            both_edges = Some(self.parse_edge_types(ctx)?);
        } else {
            break;
        }
    }
    
    // 解析 WHERE 和 YIELD
    let where_clause = if ctx.match_token(TokenKind::Where) {
        Some(self.parse_expression(ctx)?)
    } else {
        None
    };
    
    let yield_clause = if ctx.match_token(TokenKind::Yield) {
        Some(ClauseParser::new().parse_yield_clause(ctx)?)
    } else {
        None
    };
    
    let end_span = ctx.current_span();
    let span = ctx.merge_span(start_span.start, end_span.end);
    
    Ok(Stmt::Subgraph(SubgraphStmt {
        span,
        steps,
        from: from_clause,
        in_edges,
        out_edges,
        both_edges,
        where_clause,
        yield_clause,
        with_prop,
    }))
}
```

### 3.4 Planner 修改

```rust
// src/query/planner/statements/subgraph_planner.rs
impl SubgraphPlanner {
    fn create_directional_expands(
        &self,
        subgraph_ctx: &SubgraphContext,
    ) -> Result<Vec<PlanNodeEnum>, PlannerError> {
        let mut expand_nodes = Vec::new();
        
        // 为每个方向创建扩展节点
        if let Some(ref edges) = subgraph_ctx.in_edges {
            let expand = ExpandAllNode::new(
                subgraph_ctx.space_id,
                edges.clone(),
                "in",
            );
            expand_nodes.push(PlanNodeEnum::ExpandAll(expand));
        }
        
        if let Some(ref edges) = subgraph_ctx.out_edges {
            let expand = ExpandAllNode::new(
                subgraph_ctx.space_id,
                edges.clone(),
                "out",
            );
            expand_nodes.push(PlanNodeEnum::ExpandAll(expand));
        }
        
        if let Some(ref edges) = subgraph_ctx.both_edges {
            let expand = ExpandAllNode::new(
                subgraph_ctx.space_id,
                edges.clone(),
                "both",
            );
            expand_nodes.push(PlanNodeEnum::ExpandAll(expand));
        }
        
        Ok(expand_nodes)
    }
    
    fn plan_with_directions(
        &mut self,
        ast_ctx: &AstContext,
    ) -> Result<SubPlan, PlannerError> {
        let subgraph_ctx = SubgraphContext::new(ast_ctx.clone());
        
        // 创建多个方向的扩展节点
        let expand_nodes = self.create_directional_expands(&subgraph_ctx)?;
        
        // 如果有多个方向，使用 Union 合并结果
        let current_node = if expand_nodes.len() == 1 {
            expand_nodes.into_iter().next().unwrap()
        } else {
            // 创建 Union 节点合并多个方向的结果
            UnionNode::new(expand_nodes)?.into_enum()
        };
        
        // ... 后续处理
        
        Ok(SubPlan::new(Some(current_node), None))
    }
}
```

### 3.5 使用示例

```sql
-- 仅获取入边
GET SUBGRAPH 2 STEPS FROM "player100" 
IN OVER follow 
YIELD vertices AS nodes, edges AS relationships

-- 分别指定入边和出边
GET SUBGRAPH 3 STEPS FROM "player100"
IN OVER follow
OUT OVER serve
BOTH OVER like
YIELD vertices AS nodes, edges AS relationships

-- 携带属性
GET SUBGRAPH WITH PROP 2 STEPS FROM "player100"
OUT OVER follow
YIELD vertices AS nodes, edges AS relationships
```

---

## 4. 单条最短路径

### 4.1 目标

支持 `FIND SHORTEST PATH ...` 只返回一条最短路径。

### 4.2 AST 修改

```rust
// src/query/parser/ast/stmt.rs
pub struct FindPathStmt {
    pub span: Span,
    pub from: FromClause,
    pub to: Expression,
    pub over: Option<OverClause>,
    pub where_clause: Option<Expression>,
    pub path_type: PathType,        // 修改：从 bool 改为枚举
    pub max_steps: Option<usize>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub yield_clause: Option<YieldClause>,
    pub weight_expression: Option<String>,
    pub heuristic_expression: Option<String>,
    pub with_loop: bool,
    pub with_cycle: bool,
}

pub enum PathType {
    Default,        // 普通路径查询
    AllShortest,    // 所有最短路径
    SingleShortest, // 单条最短路径
}
```

### 4.3 Parser 修改

```rust
// src/query/parser/parser/traversal_parser.rs
pub fn parse_find_path_statement(&mut self, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
    let start_span = ctx.current_span();
    ctx.expect_token(TokenKind::Find)?;
    
    // 解析路径类型
    let path_type = if ctx.match_token(TokenKind::Shortest) {
        if ctx.match_token(TokenKind::Single) {
            PathType::SingleShortest
        } else {
            PathType::AllShortest
        }
    } else if ctx.match_token(TokenKind::All) {
        PathType::Default
    } else {
        PathType::Default
    };
    
    ctx.expect_token(TokenKind::Path)?;
    
    // ... 其余解析逻辑
    
    Ok(Stmt::FindPath(FindPathStmt {
        // ...
        path_type,
        // ...
    }))
}
```

### 4.4 Executor 修改

```rust
// src/query/executor/data_processing/graph_traversal/shortest_path.rs
impl<S: StorageClient> ShortestPathExecutor<S> {
    pub fn with_limits(mut self, single_shortest: bool, limit: usize) -> Self {
        self.single_shortest = single_shortest;
        self.limit = limit;
        self
    }
    
    fn execute_algorithm(&mut self) -> Result<Vec<Path>, QueryError> {
        match self.algorithm_type {
            // ...
            ShortestPathAlgorithmType::BFS => {
                let mut algorithm = BidirectionalBFS::new(storage)
                    .with_edge_direction(self.edge_direction);
                
                // 根据 path_type 决定查找策略
                let paths = algorithm.find_paths(
                    &self.start_vertex_ids,
                    &self.end_vertex_ids,
                    edge_types,
                    self.max_depth,
                    self.single_shortest,  // 传递给算法
                    self.limit,
                )?;
                // ...
            }
            // ...
        }
    }
}
```

### 4.5 使用示例

```sql
-- 单条最短路径
FIND SHORTEST SINGLE PATH FROM "player100" TO "player101" OVER follow

-- 所有最短路径
FIND SHORTEST PATH FROM "player100" TO "player101" OVER follow

-- 带权重的单条最短路径
FIND SHORTEST SINGLE PATH FROM "player100" TO "player101" OVER follow
WEIGHT rank
```

---

## 5. 路径携带属性

### 5.1 目标

支持 `FIND PATH WITH PROP ...` 在返回路径时携带边和顶点的属性。

### 5.2 AST 修改

```rust
// src/query/parser/ast/stmt.rs
pub struct FindPathStmt {
    // ... 现有字段
    pub with_prop: bool,        // 新增：是否携带属性
}

pub struct SubgraphStmt {
    // ... 现有字段
    pub with_prop: bool,        // 已存在，保持一致
}
```

### 5.3 Core 类型修改

```rust
// src/core/types/path.rs
pub struct Path {
    pub steps: Vec<PathStep>,
    pub with_properties: bool,              // 新增
    pub vertex_properties: Vec<Property>,   // 新增
    pub edge_properties: Vec<Property>,     // 新增
}

pub struct PathStep {
    pub vertex: Value,
    pub edge: Option<Value>,
    pub vertex_props: Option<HashMap<String, Value>>,  // 新增
    pub edge_props: Option<HashMap<String, Value>>,    // 新增
}
```

### 5.4 Executor 修改

```rust
// src/query/executor/data_processing/graph_traversal/algorithms/bidirectional_bfs.rs
impl<S: StorageClient> BidirectionalBFS<S> {
    pub fn with_properties(mut self, with_prop: bool) -> Self {
        self.with_properties = with_prop;
        self
    }
    
    fn build_path_with_props(
        &self,
        vertex_ids: &[Value],
        edge_ids: &[Value],
    ) -> Result<Path, QueryError> {
        let mut steps = Vec::new();
        
        for (i, vertex_id) in vertex_ids.iter().enumerate() {
            let vertex_props = if self.with_properties {
                self.fetch_vertex_properties(vertex_id)?
            } else {
                None
            };
            
            let edge_props = if i < edge_ids.len() && self.with_properties {
                self.fetch_edge_properties(&edge_ids[i])?
            } else {
                None
            };
            
            steps.push(PathStep {
                vertex: vertex_id.clone(),
                edge: edge_ids.get(i).cloned(),
                vertex_props,
                edge_props,
            });
        }
        
        Ok(Path {
            steps,
            with_properties: self.with_properties,
            vertex_properties: Vec::new(),
            edge_properties: Vec::new(),
        })
    }
}
```

### 5.5 使用示例

```sql
-- 路径携带属性
FIND SHORTEST PATH WITH PROP FROM "player100" TO "player101" OVER follow
YIELD path

-- 子图携带属性
GET SUBGRAPH WITH PROP 2 STEPS FROM "player100" OVER follow
YIELD vertices, edges
```

---

## 6. 实施优先级

| 功能 | 优先级 | 工作量 | 依赖 |
|------|--------|--------|------|
| YIELD WHERE | P0 | 小 | 无 |
| 单条最短路径 | P0 | 小 | 已预留接口 |
| SUBGRAPH 方向分离 | P1 | 中 | Parser 修改 |
| YIELD JOIN | P1 | 中 | JOIN 已存在 |
| 路径携带属性 | P2 | 中 | 存储层支持 |

---

## 7. 测试计划

### 7.1 单元测试

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_yield_where() {
        // 测试 YIELD WHERE 解析和执行
    }
    
    #[test]
    fn test_subgraph_directions() {
        // 测试 IN/OUT/BOTH 方向分离
    }
    
    #[test]
    fn test_single_shortest_path() {
        // 测试单条最短路径
    }
}
```

### 7.2 集成测试

```rust
#[test]
fn test_yield_join_integration() {
    // 测试完整的 YIELD | JOIN 流程
}

#[test]
fn test_path_with_properties() {
    // 测试带属性的路径查询
}
```

---

## 8. 兼容性说明

- 所有修改保持向后兼容
- 新增语法为可选功能，不影响现有查询
- 默认行为保持不变
