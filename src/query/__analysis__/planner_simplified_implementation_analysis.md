# Planner 目录简化实现分析报告

## 概述

本文档详细分析了 `src/query/planner` 目录中的简化实现，并与 Nebula-Graph 官方实现进行对比。通过识别当前实现中的简化之处，为后续改进提供参考和方向。

---

## 一、架构层面的简化

### 1.1 规划器注册机制

**当前实现简化点：**

在 [planner.rs](src/query/planner/planner.rs) 中，虽然设计了 `SentenceKind` 枚举和 `PlannerRegistry`，但 NGQL 规划器的注册被大量注释掉：

```rust
// 暂时注释掉，因为现有的规划器还没有实现新的接口
// self.register_planner(
//     SentenceKind::Go,
//     crate::query::planner::statements::GoPlanner::match_ast_ctx,
//     crate::query::planner::statements::GoPlanner::make,
//     100,
// );
```

**Nebula-Graph 实现：**

Nebula-Graph 在 `PlannersRegister` 中完整注册了所有规划器：
- `SequentialPlanner`：处理复合语句
- `PathPlanner`：处理路径查询
- `LookupPlanner`：处理索引查找
- `GoPlanner`：处理 GO 查询
- `MatchPlanner`：处理 MATCH 查询

每个规划器都有完整的 `make` 和 `transform` 方法实现。

**差异分析：**

| 特性 | 当前实现 | Nebula-Graph |
|------|----------|--------------|
| 规划器注册 | 部分注释掉 | 完整注册 |
| 优先级机制 | 已实现 | 已实现 |
| 动态选择 | 已设计 | 完整实现 |

---

### 1.2 子句规划器接口

**当前实现简化点：**

在 [cypher_clause_planner.rs](src/query/planner/statements/core/cypher_clause_planner.rs) 中，虽然定义了完整的接口，但存在以下简化：

1. **类型系统简化**：删除了 `VariableRequirement` 和 `VariableProvider`，使用简单的 `VariableInfo` 替代
2. **验证机制简化**：使用简化的 `DataFlowManager` 替代复杂的 `DataFlowValidator`

**Nebula-Graph 实现：**

Nebula-Graph 的 `CypherClausePlanner` 提供了更丰富的接口：
- `validateInput()`：验证输入
- `toPlan()`：生成执行计划
- `appendFilter()`：追加过滤条件
- 完整的连接策略支持

**差异分析：**

| 特性 | 当前实现 | Nebula-Graph |
|------|----------|--------------|
| 变量需求定义 | 简化（VariableInfo） | 完整（VariableRequirement） |
| 输入验证 | 简化（DataFlowManager） | 完整（validateInput） |
| 过滤追加 | 无 | appendFilter() |
| 连接策略 | UnifiedConnector | 多策略（AddInput, InnerJoin, LeftOuterJoin） |

---

## 二、计划节点层面的简化

### 2.1 计划节点类型

**当前实现：**

在 `plan/core/nodes/` 目录下，定义了基础的计划节点类型，但节点数量相对有限。

**Nebula-Graph 实现：**

Nebula-Graph 在 `src/planner/plan/` 目录下定义了七个大类，超过 100 种计划节点：

| 类别 | 说明 | 示例节点 |
|------|------|----------|
| Admin | 管理操作 | Show, Kill |
| Algo | 算法操作 | BFS, ShortestPath |
| Logic | 逻辑操作 | And, Or |
| Maintain | 维护操作 | CreateTag, AlterEdge |
| Mutate | 变更操作 | Insert, Update |
| Query | 查询操作 | GetNeighbors, Aggregate |
| Scan | 扫描操作 | IndexScan, Scan |

**关键节点对比：**

| 节点类型 | 当前实现 | Nebula-Graph |
|----------|----------|--------------|
| GetNeighbors | 基础实现 | 完整实现，支持属性投影、统计、过滤等 |
| Aggregate | 基础实现 | 完整实现，支持分组键和聚合项 |
| Loop | 无 | 完整实现，支持循环展开 |
| InnerJoin | 基础实现 | 完整实现，支持 Hash Join |
| IndexScan | 基础实现 | 完整实现，支持范围扫描和全文索引 |

---

### 2.2 节点创建工厂

**当前实现简化点：**

在 [factory.rs](src/query/planner/plan/core/nodes/factory.rs) 中，工厂模式实现相对简单。

**示例代码：**

```rust
pub struct PlanNodeFactory;

impl PlanNodeFactory {
    pub fn create_argument_node(id: i32, var_name: &str) -> PlanNodeEnum {
        PlanNodeEnum::Argument(ArgumentNode::new(id, var_name))
    }
}
```

**Nebula-Graph 实现：**

Nebula-Graph 使用更灵活的静态工厂方法：

```cpp
static GetNeighbors* make(QueryContext* qctx,
    PlanNode* input,
    GraphSpaceID space,
    Expression* src,
    std::vector<EdgeType> edgeTypes,
    Direction edgeDirection,
    std::unique_ptr<std::vector<VertexProp>>&& vertexProps,
    std::unique_ptr<std::vector<EdgeProp>>&& edgeProps,
    std::unique_ptr<std::vector<StatProp>>&& statProps,
    std::unique_ptr<std::vector<Expr>>&& exprs,
    bool dedup = false,
    bool random = false,
    std::vector<storage::cpp2::OrderBy> orderBy = {},
    int64_t limit = -1,
    std::string filter = "")
```

**差异分析：**

| 特性 | 当前实现 | Nebula-Graph |
|------|----------|--------------|
| 参数传递 | 简单参数 | 智能指针 + 右值引用 |
| 可选参数 | 必需参数 | 可选参数模式 |
| 节点配置 | 基础配置 | 完整配置（过滤、排序、限制等） |

---

## 三、语句规划器层面的简化

### 3.1 MATCH 规划器

**当前实现：**

在 [match_planner.rs](src/query/planner/statements/match_planner.rs) 中，`MatchPlanner` 实现了基础的框架，但存在以下简化：

1. **子句解析简化**：`parse_clauses()` 方法返回空的 `MatchClauseContext`
2. **表达式处理简化**：WHERE 子句等关键表达式解析未完成
3. **路径处理简化**：复杂路径模式支持有限

**示例代码：**

```rust
fn parse_clauses(&self) -> Result<Vec<CypherClauseContext>, PlannerError> {
    // 这里应该解析查询文本并构建子句上下文
    // 暂时返回一个简单的 MATCH 子句
    let match_clause = crate::query::validator::structs::MatchClauseContext {
        paths: vec![],
        aliases_available: std::collections::HashMap::new(),
        // ... 其他字段为空
    };
    Ok(vec![CypherClauseContext::Match(match_clause)])
}
```

**Nebula-Graph 实现：**

Nebula-Graph 的 `MatchPlanner::transform` 实现了完整的处理流程：

```cpp
StatusOr<SubPlan> MatchPlanner::transform(AstContext* astCtx) {
    auto* matchCtx = static_cast<MatchAstContext*>(astCtx);
    std::vector<SubPlan> subplans;
    for (auto& clauseCtx : matchCtx->clauses) {
        switch (clauseCtx->kind) {
            case CypherClauseKind::kMatch: {
                auto subplan = std::make_unique<MatchClausePlanner>()->transform(clauseCtx.get());
                // ...
            }
            case CypherClauseKind::kUnwind: {
                auto subplan = std::make_unique<UnwindClausePlanner>()->transform(clauseCtx.get());
                // ...
            }
            // ... 其他子句处理
        }
    }
    auto finalPlan = connectSegments(astCtx, subplans, matchCtx->clauses);
    return std::move(finalPlan).value();
}
```

**差异分析：**

| 特性 | 当前实现 | Nebula-Graph |
|------|----------|--------------|
| 子句解析 | 空实现 | 完整解析 |
| 表达式树构建 | 部分 | 完整 |
| 多子句连接 | 基础 | SegmentsConnector 多策略 |
| 路径模式 | 基础 | 完整（包括可变长度路径） |

---

### 3.2 NGQL 规划器

**当前实现简化点：**

所有 NGQL 规划器（GO、LOOKUP、PATH、SUBGRAPH 等）都只有基础框架，大量功能未实现。

**GO 规划器 [go_planner.rs](src/query/planner/statements/go_planner.rs)：**

- 已实现基础的节点创建（ArgumentNode、ExpandAllNode）
- 部分实现连接逻辑（InnerJoinNode）
- 过滤逻辑未完整实现

**LOOKUP 规划器 [lookup_planner.rs](src/query/planner/statements/lookup_planner.rs)：**

- 已实现索引扫描节点创建
- 全文索引处理有基础实现
- 表达式过滤未完整实现

**PATH 规划器 [path_planner.rs](src/query/planner/statements/path_planner.rs)：**

- 仅有空壳实现
- 所有方法返回错误

**Nebula-Graph 实现：**

Nebula-Graph 的 NGQL 规划器都完整实现：

- **GoPlanner**：支持边类型过滤、方向控制、步数限制、条件过滤
- **LookupPlanner**：支持属性索引、全文索引、表达式过滤
- **PathPlanner**：支持最短路径、所有路径、K 最短路径

---

## 四、子句规划器层面的简化

### 4.1 RETURN 子句规划器

**当前实现简化点：**

在 [return_clause_planner.rs](src/query/planner/statements/clauses/return_clause_planner.rs) 中，`transform` 方法直接返回输入计划，未实现真正的投影逻辑：

```rust
impl CypherClausePlanner for ReturnClausePlanner {
    fn transform(
        &self,
        _clause_ctx: &CypherClauseContext,
        input_plan: Option<&SubPlan>,
        _context: &mut PlanningContext,
    ) -> Result<SubPlan, PlannerError> {
        self.validate_flow(input_plan)?;
        let input_plan = input_plan.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("RETURN 子句需要输入计划".to_string())
        })?;
        Ok(input_plan.clone())  // 直接返回输入，未实现投影
    }
}
```

**Nebula-Graph 实现：**

Nebula-Graph 的 `ReturnClausePlanner::transform` 实现了：
- 表达式投影
- DISTINCT 去重
- 列名设置
- 排序和分页处理

---

### 4.2 WHERE 子句规划器

**当前实现简化点：**

在 [where_clause_planner.rs](src/query/planner/statements/clauses/where_clause_planner.rs) 中，表达式解析和过滤条件处理未完整实现。

**Nebula-Graph 实现：**

Nebula-Graph 的 `WhereClausePlanner` 实现了：
- 表达式到过滤条件的完整转换
- 下推优化（表达式下推到存储层）
- 多条件组合（AND、OR）

---

### 4.3 其他子句规划器

| 子句 | 当前实现 | Nebula-Graph |
|------|----------|--------------|
| OrderBy | 基础框架 | 完整排序实现 |
| Limit | 基础框架 | 完整限制实现 |
| Skip | 基础框架 | 完整跳过实现 |
| With | 基础框架 | 完整管道实现 |
| Unwind | 基础框架 | 完整展开实现 |

---

## 五、索引查找策略的简化

### 5.1 查找策略实现

**当前实现：**

在 `statements/seeks/` 目录下定义了查找策略，但实现相对简化：

- [vertex_seek.rs](src/query/planner/statements/seeks/vertex_seek.rs)：基础实现
- [scan_seek.rs](src/query/planner/statements/seeks/scan_seek.rs)：基础实现
- [index_seek.rs](src/query/planner/statements/seeks/index_seek.rs)：基础实现

**Nebula-Graph 实现：**

Nebula-Graph 定义了完整的查找策略：

```cpp
startVidFinders.emplace_back(&VertexIdSeek::make);      // VID 查找
startVidFinders.emplace_back(&PropIndexSeek::make);     // 属性索引查找
startVidFinders.emplace_back(&LabelIndexSeek::make);    // 标签索引查找
```

每种策略都有完整的实现：
- **VertexIdSeek**：根据 VID 定位起始点
- **PropIndexSeek**：根据属性值使用索引
- **LabelIndexSeek**：根据标签扫描索引

---

### 5.2 策略优先级

**当前实现：**

查找策略优先级未完整实现。

**Nebula-Graph 实现：**

Nebula-Graph 明确策略优先级：
1. **VertexIdSeek**：最佳，能精确定位 VID
2. **PropIndexSeek**：次之，可转换为索引扫描
3. **LabelIndexSeek**：基本，会转换为全索引扫描

---

## 六、路径算法的简化

### 6.1 最短路径实现

**当前实现简化点：**

在 [shortest_path_planner.rs](src/query/planner/statements/paths/shortest_path_planner.rs) 中，仅有 TODO 注释，未实现真正的最短路径规划。

**Nebula-Graph 实现：**

Nebula-Graph 的 `PathPlanner` 实现了：
- **最短路径（Shortest Path）**：BFS 算法
- **所有路径（All Paths）**：DFS 算法
- **K 最短路径（K shortest paths）**：Yen's 算法

### 6.2 扩展算法

**当前实现：**

在 [path_algorithms.rs](src/query/planner/plan/algorithms/path_algorithms.rs) 中定义了基础框架，但实现不完整。

**Nebula-Graph 实现：**

使用 `Expand` 类处理多步扩展，生成 `Loop` 节点：

```cpp
Status Expand::doExpand(const NodeInfo& node, const EdgeInfo& edge, SubPlan* plan) {
    NG_RETURN_IF_ERROR(expandSteps(node, edge, plan));
    NG_RETURN_IF_ERROR(filterDatasetByPathLength(edge, plan->root, plan));
    return Status::OK();
}
```

---

## 七、数据流管理的简化

### 7.1 上下文传播

**当前实现简化点：**

在 [cypher_clause_planner.rs](src/query/planner/statements/core/cypher_clause_planner.rs) 中，`ContextPropagator` 实现了基础的上下文传播，但变量生命周期管理不完整。

**关键简化：**

```rust
pub fn propagate_to_clause(
    &self,
    context: &PlanningContext,
    clause_type: ClauseType,
) -> Option<PlanningContext> {
    // 简化实现：直接克隆上下文
    Some(context.clone())
}
```

**Nebula-Graph 实现：**

Nebula-Graph 使用更复杂的上下文管理：
- 完整的变量作用域追踪
- 别名解析和验证
- 类型推断和检查

---

### 7.2 连接策略

**当前实现简化点：**

使用简化的 `UnifiedConnector` 替代多种连接策略。

**Nebula-Graph 实现：**

Nebula-Graph 定义了多种连接策略：
- **AddInputStrategy**：添加输入依赖
- **InnerJoinStrategy**：内连接
- **LeftOuterJoinStrategy**：左外连接
- **CartesianProductStrategy**：笛卡尔积
- **UnionStrategy**：联合

---

## 八、改进建议

### 8.1 优先级 1：核心功能实现

#### 8.1.1 完善 NGQL 规划器

**建议修改：**

1. **GoPlanner**：
   - 实现完整的表达式过滤逻辑
   - 添加步数限制处理
   - 完善 JOIN 逻辑

2. **LookupPlanner**：
   - 完善属性索引选择逻辑
   - 实现表达式到索引条件的转换
   - 添加索引代价估算

3. **PathPlanner**：
   - 实现最短路径算法
   - 实现所有路径算法
   - 添加路径长度过滤

#### 8.1.2 完善子句规划器

**建议修改：**

1. **ReturnClausePlanner**：
   - 实现表达式投影逻辑
   - 添加 DISTINCT 处理
   - 实现列名设置

2. **WhereClausePlanner**：
   - 实现表达式解析
   - 添加过滤条件下推
   - 支持多条件组合

3. **其他子句**：
   - 实现 OrderBy 排序
   - 实现 Limit/Skip 分页
   - 实现 Unwind 展开

---

### 8.2 优先级 2：算法实现

#### 8.2.1 索引查找策略

**建议修改：**

1. 实现 VertexIdSeek 策略
2. 实现 PropIndexSeek 策略
3. 实现 LabelIndexSeek 策略
4. 添加策略优先级选择

#### 8.2.2 路径扩展算法

**建议修改：**

1. 实现多步扩展的 Loop 节点
2. 添加路径长度过滤
3. 实现可变长度路径

---

### 8.3 优先级 3：架构优化

#### 8.3.1 计划节点扩展

**建议修改：**

1. 添加完整的 GetNeighbors 节点
2. 添加 Loop 节点
3. 完善 Aggregate 节点
4. 扩展 Join 节点

#### 8.3.2 连接策略扩展

**建议修改：**

1. 实现 AddInputStrategy
2. 实现 InnerJoinStrategy
3. 实现 LeftOuterJoinStrategy
4. 添加策略自动选择

---

## 九、总结

### 9.1 当前实现状态

| 模块 | 完成度 | 说明 |
|------|--------|------|
| 规划器注册 | 30% | 部分注册，NGQL 规划器被注释 |
| MATCH 规划 | 40% | 框架完整，表达式解析缺失 |
| NGQL 规划 | 20% | 基础框架，功能未实现 |
| 子句规划 | 30% | 框架完整，业务逻辑缺失 |
| 索引查找 | 25% | 基础实现，选择策略缺失 |
| 路径算法 | 10% | 仅有框架，未实现 |

### 9.2 与 Nebula-Graph 的差距

| 方面 | 差距 | 原因 |
|------|------|------|
| 功能完整性 | 大 | 开发时间短，资源有限 |
| 优化能力 | 大 | 缺少代价模型和优化规则 |
| 测试覆盖 | 中 | 单元测试和集成测试不足 |
| 文档 | 小 | 有较好的代码注释 |

### 9.3 改进路线

1. **阶段 1**（1-2 个月）：完成 NGQL 核心规划器
2. **阶段 2**（2-3 个月）：完善子句规划器
3. **阶段 3**（1-2 个月）：实现索引查找策略
4. **阶段 4**（2-3 个月）：添加优化规则
5. **阶段 5**（持续）：完善测试和文档

---

## 参考资料

- [Nebula Graph 源码解析：Planner](https://www.nebula-graph.io/posts/nebula-graph-source-code-reading-03)
- [Nebula Graph 源码解析：Validator](https://www.nebula-graph.io/posts/nebula-graph-source-code-reading-02)
- [Nebula Graph GitHub 仓库](https://github.com/vesoft-inc/nebula)
