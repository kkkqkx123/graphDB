# Statements 规划器重构与补全计划

## 一、现状概述

`src/query/planning/statements` 目录是查询规划引擎的核心模块，负责将验证后的 AST 语句转换为可执行的 `SubPlan`。当前目录按功能分为 6 个子模块：

```
statements/
├── clauses/          # 子句级规划器（ClausePlanner trait 实现）
├── ddl/              # 数据定义语言规划器
├── dml/              # 数据操作语言规划器
├── dql/              # 数据查询语言规划器
├── paths/            # MATCH 路径规划辅助模块
├── seeks/            # 搜索策略模块（MATCH 起始点查找）
├── match_statement_planner.rs   # MATCH 语句规划器（独立）
└── statement_planner.rs         # 核心 trait 定义
```

### 架构设计

采用三层 trait 体系：

```
Planner trait（基础接口：transform + match_planner）
  ├── StatementPlanner trait（语句级：statement_type + supported_clause_kinds）
  └── ClausePlanner trait（子句级：clause_kind + transform_clause）
```

调度机制：`PlannerEnum::from_stmt()` 通过 `Stmt` 枚举直接映射到对应规划器实例，消除动态分发。

### 覆盖率统计

| 维度            | 评估                                               |
| --------------- | -------------------------------------------------- |
| Stmt 变体覆盖率 | ~85%（46 个变体中约 39 个有规划器）                |
| 实现完整度      | ~70%（多个规划器存在未实现分支或降级处理）         |
| 架构一致性      | 中等（ClausePlanner 与 StatementPlanner 逻辑重复） |

---

## 二、问题清单

### 🔴 P0：严重问题（功能缺失/错误）

#### P0-1：CreatePlanner 未注册到 PlannerEnum

**现象**：`dml/create_planner.rs` 中的 `CreatePlanner` 处理 Cypher 风格的 `CREATE (n:Label)` 数据创建，但 `PlannerEnum::from_stmt()` 将 `Stmt::Create` 路由到了 `MaintainPlanner`。

**影响**：Cypher 风格的 `CREATE (n:Person {name: 'Alice'})` 语句无法正确生成执行计划。`MaintainPlanner` 中对 `CreateTarget::Node/Edge/Path` 的数据创建逻辑未实现，会降级为 PassThroughNode。

**根因**：`Stmt::Create` 同时承载 DDL（`CREATE TAG/EDGE/SPACE/INDEX`）和 DML（`CREATE (n:Label)`）两种语义，但 `from_stmt()` 只有一个匹配分支。

**修复方案**：

```rust
// planner.rs - PlannerEnum::from_stmt() 修改
Stmt::Create(create_stmt) => {
    match &create_stmt.target {
        // DML 数据创建 → CreatePlanner
        CreateTarget::Node { .. } | CreateTarget::Edge { .. } | CreateTarget::Path { .. } => {
            Some(PlannerEnum::CreateData(CreatePlanner::new()))
        }
        // DDL Schema 创建 → MaintainPlanner
        _ => Some(PlannerEnum::Maintain(MaintainPlanner::new()))
    }
}
```

需要在 `PlannerEnum` 中新增 `CreateData(CreatePlanner)` 变体，并在 `transform`/`match_planner`/`name` 等方法中添加对应分支。

#### P0-2：Assignment 语句无规划器

**现象**：`$var = GO FROM 1 OVER friend` 这种变量赋值语句在 `from_stmt()` 中返回 `None`。

**影响**：管道查询中的变量赋值功能完全不可用。

**修复方案**：新增 `AssignmentPlanner`，处理逻辑为：

1. 递归规划右侧语句得到 `SubPlan`
2. 将结果绑定到变量名
3. 在 `QueryContext` 中注册变量 schema

```rust
// 新增文件：dml/assignment_planner.rs
pub struct AssignmentPlanner;

impl Planner for AssignmentPlanner {
    fn transform(&mut self, validated: &ValidatedStatement, qctx: Arc<QueryContext>)
        -> Result<SubPlan, PlannerError>
    {
        let assignment_stmt = extract_assignment_stmt(validated.stmt())?;
        // 递归规划右侧语句
        let inner_plan = plan_subquery(&assignment_stmt.statement, qctx)?;
        // 注册变量到上下文
        qctx.register_variable(assignment_stmt.variable, inner_plan.schema());
        Ok(inner_plan)
    }
}
```

#### P0-3：Explain/Profile 语句无规划器

**现象**：`EXPLAIN` 和 `PROFILE` 是重要的调试语句，当前在 `from_stmt()` 中返回 `None`。

**修复方案**：新增 `ExplainPlanner` 和 `ProfilePlanner`，逻辑为：

1. 提取内部语句
2. 递归规划内部语句得到 `SubPlan`
3. 包装为 `ExplainNode`/`ProfileNode`（需要新增 PlanNode 类型）

---

### 🟡 P1：部分实现问题

#### P1-1：DeletePlanner 的 Tags/Index 目标未实现

**现象**：`DeleteTarget::Tags` 和 `DeleteTarget::Index` 返回 "not yet implemented" 错误。

**修复方案**：

- `DeleteTarget::Tags`：生成 `RemoveTagNode`（复用现有 RemoveNode 逻辑）
- `DeleteTarget::Index`：生成 `DropIndexNode`（委托给 MaintainPlanner 的 DROP INDEX 逻辑）

#### P1-2：UpdatePlanner 的 Tag 目标未实现

**现象**：`UpdateTarget::Tag` 返回 "UPDATE TAG not yet supported" 错误。

**修复方案**：实现 `UpdateTarget::Tag` 分支，生成 `ScanVerticesNode(tag=Tag) → FilterNode → UpdateNode` 链路。

#### P1-3：MergePlanner 仅支持节点模式

**现象**：`MERGE (a)-[:TYPE]->(b)` 边模式返回错误。

**修复方案**：

1. 新增 `pattern_to_edge_info()` 方法
2. 边 MERGE 逻辑：先查找匹配边 → 存在则 ON MATCH → 不存在则 ON CREATE（创建边及两端节点）
3. 生成 `SelectNode` 条件分支计划

#### P1-4：MaintainPlanner 使用字符串匹配分发

**现象**：使用 `stmt_type == "SHOW"` / `stmt_type.starts_with("CREATE")` 等字符串比较，不如枚举匹配安全。

**修复方案**：将 `MaintainPlanner::transform()` 改为直接匹配 `Stmt` 枚举变体：

```rust
// 修改前
let stmt_type = validated.stmt().kind().to_uppercase();
if stmt_type == "SHOW" { ... }
else if stmt_type.starts_with("CREATE") { ... }

// 修改后
match validated.stmt() {
    Stmt::Show(show_stmt) => { ... }
    Stmt::Create(create_stmt) => { ... }
    Stmt::Drop(drop_stmt) => { ... }
    Stmt::Alter(alter_stmt) => { ... }
    Stmt::Desc(desc_stmt) => { ... }
    Stmt::ClearSpace(clear_stmt) => { ... }
    Stmt::ShowCreate(show_create_stmt) => { ... }
    // ...
}
```

#### P1-5：多个管理语句降级为 PassThroughNode

**现象**：`DescribeUser`、`ShowUsers`、`ShowRoles`、`ShowSessions`、`ShowQueries`、`KillQuery`、`ShowConfigs`、`UpdateConfigs` 都降级为 PassThroughNode。

**修复方案**：为每个语句类型生成专用的 PlanNode：

| 语句            | 目标 PlanNode       |
| --------------- | ------------------- |
| `DescribeUser`  | `DescUserNode`      |
| `ShowUsers`     | `ShowUsersNode`     |
| `ShowRoles`     | `ShowRolesNode`     |
| `ShowSessions`  | `ShowSessionsNode`  |
| `ShowQueries`   | `ShowQueriesNode`   |
| `KillQuery`     | `KillQueryNode`     |
| `ShowConfigs`   | `ShowConfigsNode`   |
| `UpdateConfigs` | `UpdateConfigsNode` |

#### P1-6：ShowCreate 仅支持 Tag

**现象**：`ShowCreateTarget::Edge` 等目标降级为 PassThroughNode。

**修复方案**：新增 `ShowCreateEdgeNode`，处理 `SHOW CREATE EDGE` 语句。

---

### 🟢 P2：架构/设计优化

#### P2-1：ClausePlanner 与 StatementPlanner 逻辑重复

**现象**：

- `ReturnClausePlanner`（clauses/）和 `ReturnPlanner`（dql/）都实现了 RETURN 规划
- `WithClausePlanner`（clauses/）和 `WithPlanner`（dql/）都实现了 WITH 规划
- `UnwindClausePlanner`（clauses/）和 `UnwindPlanner`（dql/）都实现了 UNWIND 规划
- `YieldClausePlanner`（clauses/）和 `YieldPlanner`（dql/）都实现了 YIELD 规划
- `MatchStatementPlanner` 内部自己实现了 WHERE/RETURN/ORDER BY/LIMIT，未复用 ClausePlanner

**修复方案**：分两步走：

**步骤 1**：让 `MatchStatementPlanner` 复用 ClausePlanner

```rust
// match_statement_planner.rs 修改
impl MatchStatementPlanner {
    fn plan_match_pattern(&self, ...) -> Result<SubPlan, PlannerError> {
        // ... 生成 pattern scan 计划 ...

        // 复用 ClausePlanner 处理 WHERE
        if let Some(condition) = self.extract_where_condition(stmt)? {
            let where_planner = WhereClausePlanner::new();
            plan = where_planner.transform_clause(qctx.clone(), stmt, plan)?;
        }

        // 复用 ClausePlanner 处理 RETURN
        if let Some(columns) = self.extract_return_columns(stmt, qctx)? {
            let return_planner = ReturnClausePlanner::from_stmt(stmt);
            plan = return_planner.transform_clause(qctx.clone(), stmt, plan)?;
        }

        // 复用 ClausePlanner 处理 ORDER BY
        // 复用 ClausePlanner 处理 PAGINATION
    }
}
```

**步骤 2**：统一 DQL 独立语句规划器与 ClausePlanner

将 `ReturnPlanner`、`WithPlanner`、`YieldPlanner`、`UnwindPlanner` 的核心逻辑提取为共享函数，DQL 规划器和 ClausePlanner 都调用同一套逻辑：

```
clauses/return_clause_planner.rs  ←--调用--→  dql/return_planner.rs
        ↓                                      ↓
  共享逻辑：build_return_plan(input, columns, distinct, order_by, skip, limit)
```

#### P2-2：MaintainPlanner 职责过重

**现象**：一个规划器处理 CREATE/DROP/ALTER/SHOW/DESC/CLEAR_SPACE/SHOW_CREATE 等十余种语句，代码超过 460 行。

**修复方案**：拆分为多个专职规划器：

```
ddl/
├── maintain_planner.rs     → 拆分为：
├── schema_planner.rs       # CREATE TAG/EDGE/SPACE/INDEX, DROP, ALTER
├── show_planner.rs         # SHOW TAGS/EDGES/STATS/USERS/ROLES/...
├── desc_planner.rs         # DESC/DESCRIBE TAG/EDGE/SPACE
├── config_planner.rs       # SHOW CONFIGS, UPDATE CONFIGS
├── session_planner.rs      # SHOW SESSIONS, SHOW QUERIES, KILL QUERY
├── use_planner.rs          # USE（保持不变）
└── user_management_planner.rs  # 用户管理（保持不变）
```

`PlannerEnum` 相应新增变体，`from_stmt()` 按语句类型分发到对应规划器。

#### P2-3：dql/ 和 clauses/ 中的同名规划器职责不清

**现象**：`dql/unwind_planner.rs`（语句级 `UnwindPlanner`）和 `clauses/unwind_planner.rs`（子句级 `UnwindClausePlanner`）功能重叠。

**修复方案**：

- `clauses/` 下的规划器仅实现 `ClausePlanner` trait，接收 `input_plan` 参数
- `dql/` 下的规划器实现 `Planner` trait，负责创建初始 `ArgumentNode` 并调用 `clauses/` 中的逻辑
- 明确命名约定：`clauses/` 用 `XxxClausePlanner`，`dql/` 用 `XxxPlanner`

---

## 三、实施计划

### 阶段 1：P0 修复（关键功能补全）

**目标**：修复影响核心功能的缺失和错误

| 任务                                     | 涉及文件                                       | 验收标准                                    |
| ---------------------------------------- | ---------------------------------------------- | ------------------------------------------- |
| P0-1：注册 CreatePlanner 到 PlannerEnum  | `planner.rs`, `dml/create_planner.rs`          | `CREATE (n:Person)` 生成 InsertVerticesNode |
| P0-2：新增 AssignmentPlanner             | 新增 `dml/assignment_planner.rs`, `planner.rs` | `$var = GO FROM 1 OVER friend` 可规划       |
| P0-3：新增 ExplainPlanner/ProfilePlanner | 新增 `dql/explain_planner.rs`, `planner.rs`    | `EXPLAIN MATCH ...` 可规划                  |

**验收测试**：

- `CREATE (n:Person {name: 'Alice'})` → InsertVerticesNode
- `CREATE (a)-[:KNOWS]->(b)` → InsertEdgesNode
- `$result = GO FROM 1 OVER friend` → 正确的 SubPlan
- `EXPLAIN MATCH (n) RETURN n` → ExplainNode 包装的 SubPlan

### 阶段 2：P1 修复（部分实现补全）

**目标**：补全所有未实现的分支

| 任务                                | 涉及文件                                 | 验收标准                                |
| ----------------------------------- | ---------------------------------------- | --------------------------------------- |
| P1-1：DeletePlanner 补全 Tags/Index | `dml/delete_planner.rs`                  | `DELETE TAG person FROM 1` 可规划       |
| P1-2：UpdatePlanner 补全 Tag        | `dml/update_planner.rs`                  | `UPDATE TAG person ON 1 SET ...` 可规划 |
| P1-3：MergePlanner 支持边模式       | `dml/merge_planner.rs`                   | `MERGE (a)-[:KNOWS]->(b)` 可规划        |
| P1-4：MaintainPlanner 改用枚举匹配  | `ddl/maintain_planner.rs`                | 无字符串匹配，所有分支覆盖              |
| P1-5：管理语句生成专用 PlanNode     | `ddl/maintain_planner.rs`, PlanNode 定义 | SHOW USERS → ShowUsersNode              |
| P1-6：ShowCreate 支持 Edge          | `ddl/maintain_planner.rs`                | `SHOW CREATE EDGE friend` 可规划        |

**验收测试**：

- 所有 `DeleteTarget`/`UpdateTarget` 变体均不返回 "not yet implemented"
- `MaintainPlanner` 中无 `stmt_type` 字符串比较
- 所有管理语句生成专用 PlanNode 而非 PassThroughNode

### 阶段 3：P2 优化（架构改进）

**目标**：消除重复、改善架构

| 任务                                           | 涉及文件                                                       | 验收标准                                       |
| ---------------------------------------------- | -------------------------------------------------------------- | ---------------------------------------------- |
| P2-1：MatchStatementPlanner 复用 ClausePlanner | `match_statement_planner.rs`                                   | WHERE/RETURN/ORDER BY/LIMIT 使用 ClausePlanner |
| P2-1b：统一 DQL 规划器与 ClausePlanner 逻辑    | `dql/return_planner.rs`, `clauses/return_clause_planner.rs` 等 | 共享核心逻辑函数，无重复代码                   |
| P2-2：拆分 MaintainPlanner                     | `ddl/` 目录重组                                                | MaintainPlanner 拆分为 5 个专职规划器          |
| P2-3：明确 dql/ 与 clauses/ 职责边界           | `dql/`, `clauses/`                                             | 命名约定一致，职责清晰                         |

**验收标准**：

- `MatchStatementPlanner` 的 WHERE/RETURN/ORDER BY/LIMIT 处理调用 ClausePlanner
- `MaintainPlanner` 代码行数 < 100 行（仅保留通用框架）
- clauses/ 和 dql/ 中无重复的规划逻辑

---

## 四、新增文件清单

| 文件路径                    | 用途                                         | 阶段   |
| --------------------------- | -------------------------------------------- | ------ |
| `dml/assignment_planner.rs` | 变量赋值语句规划器                           | 阶段 1 |
| `dql/explain_planner.rs`    | EXPLAIN/PROFILE 语句规划器                   | 阶段 1 |
| `ddl/schema_planner.rs`     | Schema 操作规划器（从 MaintainPlanner 拆出） | 阶段 3 |
| `ddl/show_planner.rs`       | SHOW 语句规划器（从 MaintainPlanner 拆出）   | 阶段 3 |
| `ddl/desc_planner.rs`       | DESC 语句规划器（从 MaintainPlanner 拆出）   | 阶段 3 |
| `ddl/config_planner.rs`     | 配置管理规划器（从 MaintainPlanner 拆出）    | 阶段 3 |
| `ddl/session_planner.rs`    | 会话管理规划器（从 MaintainPlanner 拆出）    | 阶段 3 |

---

## 五、PlannerEnum 变更清单

### 阶段 1 新增变体

```rust
pub enum PlannerEnum {
    // ... 现有变体 ...
    CreateData(CreatePlanner),    // P0-1
    Assignment(AssignmentPlanner), // P0-2
    Explain(ExplainPlanner),       // P0-3
}
```

### 阶段 3 新增变体（MaintainPlanner 拆分后）

```rust
pub enum PlannerEnum {
    // ... 现有变体 ...
    Schema(SchemaPlanner),     // 替代 MaintainPlanner 的 Schema 部分
    Show(ShowPlanner),         // 替代 MaintainPlanner 的 SHOW 部分
    Desc(DescPlanner),         // 替代 MaintainPlanner 的 DESC 部分
    Config(ConfigPlanner),     // 替代 MaintainPlanner 的 CONFIG 部分
    Session(SessionPlanner),   // 替代 MaintainPlanner 的 SESSION 部分
    // Maintain 变体可保留用于过渡期，最终移除
}
```

---

## 六、from_stmt() 完整映射表（目标状态）

```rust
pub fn from_stmt(stmt: &Arc<Stmt>) -> Option<Self> {
    match stmt.as_ref() {
        // DQL
        Stmt::Match(_)           => Some(PlannerEnum::Match(MatchStatementPlanner::new())),
        Stmt::Go(_)              => Some(PlannerEnum::Go(GoPlanner::new())),
        Stmt::Lookup(_)          => Some(PlannerEnum::Lookup(LookupPlanner::new())),
        Stmt::FindPath(_)        => Some(PlannerEnum::Path(PathPlanner::new())),
        Stmt::Subgraph(_)        => Some(PlannerEnum::Subgraph(SubgraphPlanner::new())),
        Stmt::Fetch(fetch_stmt)  => match &fetch_stmt.target {
            FetchTarget::Vertices { .. } => Some(PlannerEnum::FetchVertices(FetchVerticesPlanner::new())),
            FetchTarget::Edges { .. }    => Some(PlannerEnum::FetchEdges(FetchEdgesPlanner::new())),
        },
        Stmt::GroupBy(_)         => Some(PlannerEnum::GroupBy(GroupByPlanner::new())),
        Stmt::SetOperation(_)    => Some(PlannerEnum::SetOperation(SetOperationPlanner::new())),
        Stmt::Return(_)          => Some(PlannerEnum::Return(ReturnPlanner::new())),
        Stmt::With(_)            => Some(PlannerEnum::With(WithPlanner::new())),
        Stmt::Yield(_)           => Some(PlannerEnum::Yield(YieldPlanner::new())),
        Stmt::Unwind(_)          => Some(PlannerEnum::Unwind(UnwindPlanner::new())),
        Stmt::Pipe(_)            => Some(PlannerEnum::Pipe(PipePlanner::new())),
        Stmt::Explain(_)         => Some(PlannerEnum::Explain(ExplainPlanner::new())),     // 新增
        Stmt::Profile(_)         => Some(PlannerEnum::Explain(ExplainPlanner::new())),     // 新增

        // DML
        Stmt::Create(create_stmt) => match &create_stmt.target {                          // 修改
            CreateTarget::Node { .. } | CreateTarget::Edge { .. } | CreateTarget::Path { .. } =>
                Some(PlannerEnum::CreateData(CreatePlanner::new())),
            _ => Some(PlannerEnum::Schema(SchemaPlanner::new())),
        },
        Stmt::Insert(_)          => Some(PlannerEnum::Insert(InsertPlanner::new())),
        Stmt::Delete(_)          => Some(PlannerEnum::Delete(DeletePlanner::new())),
        Stmt::Update(_)          => Some(PlannerEnum::Update(UpdatePlanner::new())),
        Stmt::Set(_)             => Some(PlannerEnum::Set(SetPlanner::new())),
        Stmt::Remove(_)          => Some(PlannerEnum::Remove(RemovePlanner::new())),
        Stmt::Merge(_)           => Some(PlannerEnum::Merge(MergePlanner::new())),
        Stmt::Assignment(_)      => Some(PlannerEnum::Assignment(AssignmentPlanner::new())), // 新增

        // DDL
        Stmt::Use(_)             => Some(PlannerEnum::Use(UsePlanner::new())),
        Stmt::Drop(_)            => Some(PlannerEnum::Schema(SchemaPlanner::new())),
        Stmt::Alter(_)           => Some(PlannerEnum::Schema(SchemaPlanner::new())),
        Stmt::Desc(_)            => Some(PlannerEnum::Desc(DescPlanner::new())),
        Stmt::ClearSpace(_)      => Some(PlannerEnum::Schema(SchemaPlanner::new())),
        Stmt::Show(_)            => Some(PlannerEnum::Show(ShowPlanner::new())),
        Stmt::ShowCreate(_)      => Some(PlannerEnum::Show(ShowPlanner::new())),
        Stmt::ShowConfigs(_)     => Some(PlannerEnum::Config(ConfigPlanner::new())),
        Stmt::UpdateConfigs(_)   => Some(PlannerEnum::Config(ConfigPlanner::new())),
        Stmt::ShowSessions(_) | Stmt::ShowQueries(_) | Stmt::KillQuery(_) =>
            Some(PlannerEnum::Session(SessionPlanner::new())),

        // DCL
        Stmt::CreateUser(_) | Stmt::AlterUser(_) | Stmt::DropUser(_) |
        Stmt::ChangePassword(_) | Stmt::Grant(_) | Stmt::Revoke(_) |
        Stmt::DescribeUser(_) | Stmt::ShowUsers(_) | Stmt::ShowRoles(_) =>
            Some(PlannerEnum::UserManagement(UserManagementPlanner::new())),

        // 全文搜索
        Stmt::CreateFulltextIndex(_) | Stmt::DropFulltextIndex(_) | ... =>
            Some(PlannerEnum::FulltextSearch(FulltextSearchPlanner::new())),

        // 向量搜索
        Stmt::CreateVectorIndex(_) | ... =>
            Some(PlannerEnum::VectorSearch(VectorSearchPlanner::new())),

        // 兜底
        Stmt::Query(_) => None,  // 需要进一步评估 QueryStmt 的语义
    }
}
```

---

## 七、风险与注意事项

### 向后兼容

- `PlannerEnum` 新增变体不影响现有代码，但需要同步更新 `transform()`、`match_planner()`、`name()` 等方法
- `MaintainPlanner` 拆分需要逐步迁移，可先保留 `Maintain` 变体作为过渡

### 测试策略

- 每个阶段完成后运行 `cargo clippy --all-targets --all-features` 确保编译通过
- 每个新增/修改的规划器需要编写单元测试
- P0 修复后需要集成测试验证端到端流程

### 依赖关系

- P0-1（CreatePlanner 注册）必须先于 P1-3（MergePlanner 边模式），因为 MERGE 的 ON CREATE 分支依赖 CreatePlanner
- P2-1（ClausePlanner 复用）应在 P1 全部完成后进行，避免在未稳定的代码上重构
- P2-2（MaintainPlanner 拆分）应在 P1-4（枚举匹配）和 P1-5（专用 PlanNode）完成后进行

---

## 八、修订历史

| 日期       | 版本 | 变更                                   |
| ---------- | ---- | -------------------------------------- |
| 2026-05-02 | 1.0  | 初始版本，基于 statements 目录完整分析 |
