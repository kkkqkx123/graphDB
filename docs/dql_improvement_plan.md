# GraphDB DQL 功能完善计划

## 文档信息
- 创建日期: 2026-02-16
- 目标版本: GraphDB v0.2.0
- 优先级: 高

## 1. 概述

本文档详细描述了 GraphDB 与 NebulaGraph 在 DQL 功能上的差距，并提供了具体的修改方案。通过本次完善，GraphDB 将在保持简洁架构的同时，提供更强大的查询能力。

## 2. 功能差距分析

### 2.1 MATCH 语句 - 复杂起始点查找策略

**当前状态**: GraphDB 已实现基础的 VertexSeek、IndexSeek、ScanSeek，但缺少以下策略：
- PropIndexSeek: 基于属性条件的索引查找
- VariablePropIndexSeek: 基于变量属性的查找
- 从边开始查找 (startFromEdge)

**影响**: 复杂 MATCH 查询的性能优化受限

### 2.2 LOOKUP 语句 - 评分功能

**当前状态**: 基础索引查找已实现，但缺少：
- 评分 (Score) 计算
- HashInnerJoin 结果合并

**影响**: 无法支持基于相关性的排序

### 2.3 SUBGRAPH 语句 - 零步扩展

**当前状态**: 基础子图查询已实现，但缺少：
- 零步扩展 (zeroStep) 特殊处理
- DataCollect 节点结果整合

**影响**: `GET SUBGRAPH 0 STEPS` 语法不支持

### 2.4 优化器 - 规则补充

**当前状态**: 约 30 条规则，需要补充：
- 更多 Filter 下推规则
- 合并规则 (Merge*)

## 3. 修改方案

### 3.1 任务1: 实现 PropIndexSeek 策略

**目标**: 支持基于属性条件的索引查找

**修改文件**:
1. `src/query/planner/statements/seeks/prop_index_seek.rs` (新建)
2. `src/query/planner/statements/seeks/mod.rs`
3. `src/query/planner/statements/seeks/seek_strategy.rs`
4. `src/query/planner/statements/seeks/seek_strategy_base.rs`

**实现要点**:
```rust
// prop_index_seek.rs 核心逻辑
pub struct PropIndexSeek;

impl SeekStrategy for PropIndexSeek {
    fn supports(&self, context: &SeekStrategyContext) -> bool {
        // 检查是否有属性过滤条件且存在对应索引
        context.has_property_predicates() && 
        context.has_index_for_properties()
    }
    
    fn execute(&self, storage: &dyn StorageClient, context: &SeekStrategyContext) -> Result<SeekResult, StorageError> {
        // 使用属性索引进行查找
        // 支持 =, <, >, <=, >=, IN 等操作
    }
}
```

**工作量**: 2-3 天

### 3.2 任务2: 实现 VariablePropIndexSeek 策略

**目标**: 支持基于变量属性的查找

**修改文件**:
1. `src/query/planner/statements/seeks/variable_prop_index_seek.rs` (新建)
2. `src/query/planner/statements/seeks/mod.rs`
3. `src/query/planner/statements/seeks/seek_strategy.rs`

**实现要点**:
```rust
// 处理形如: MATCH (v:Person) WHERE v.name = $varName
// 变量值在运行时确定
```

**工作量**: 1-2 天

### 3.3 任务3: 实现从边开始查找

**目标**: 支持 MATCH 从边模式开始查找

**修改文件**:
1. `src/query/planner/statements/seeks/edge_seek.rs` (新建)
2. `src/query/planner/statements/seeks/mod.rs`
3. `src/query/planner/statements/match_clause_planner.rs`

**实现要点**:
```rust
// 处理形如: MATCH ()-[e:KNOWS]->() WHERE e.since > 2020
// 从边索引开始查找
```

**工作量**: 2-3 天

### 3.4 任务4: 完善 LOOKUP 评分功能

**目标**: 支持评分计算和结果合并

**修改文件**:
1. `src/query/context/ast/query_types/lookup.rs`
2. `src/query/planner/statements/lookup_planner.rs`
3. `src/query/planner/plan/core/nodes/index_nodes.rs` (如有需要)

**实现要点**:
```rust
// LookupContext 添加
pub has_score: bool,
pub score_column: String,

// 在 LOOKUP 计划中支持
// LOOKUP ON person WHERE person.name == "Alice" YIELD person.name, score
```

**工作量**: 1-2 天

### 3.5 任务5: 实现 SUBGRAPH 零步扩展

**目标**: 支持 `GET SUBGRAPH 0 STEPS`

**修改文件**:
1. `src/query/planner/statements/subgraph_planner.rs`
2. `src/query/planner/plan/core/nodes/data_processing_node.rs` (添加 DataCollectNode)

**实现要点**:
```rust
// SubgraphPlanner 中添加 zero_step 处理
fn zero_step_plan(&self, start_vid_plan: SubPlan, input: &str) -> Result<SubPlan, PlannerError> {
    // 只返回起始顶点的属性，不扩展
    // 使用 GetVerticesNode 获取顶点属性
    // 使用 DataCollectNode 整合结果
}
```

**工作量**: 1-2 天

### 3.6 任务6: 补充优化规则

**目标**: 增加优化规则数量

**修改文件**:
1. `src/query/optimizer/rules/predicate_pushdown/` (补充规则)
2. `src/query/optimizer/rules/merge/` (补充规则)

**优先级规则**:
1. `PushFilterDownGetVerticesRule`
2. `PushFilterDownGetEdgesRule`
3. `MergeGetVerticesAndDedupRule`
4. `MergeGetNeighborsAndProjectRule`

**工作量**: 3-4 天

## 4. 实施计划

### 阶段1: 核心功能 (第1-2周)
- [ ] 任务1: PropIndexSeek 实现
- [ ] 任务2: VariablePropIndexSeek 实现
- [ ] 任务3: EdgeSeek 实现

### 阶段2: 功能完善 (第3周)
- [ ] 任务4: LOOKUP 评分功能
- [ ] 任务5: SUBGRAPH 零步扩展

### 阶段3: 优化增强 (第4周)
- [ ] 任务6: 补充优化规则

## 5. 测试计划

每个任务完成后需要：
1. 单元测试覆盖新功能
2. 集成测试验证与现有功能的兼容性
3. 性能测试对比优化前后的差异

## 6. 风险评估

| 风险 | 可能性 | 影响 | 缓解措施 |
|-----|-------|------|---------|
| 属性索引实现复杂 | 中 | 高 | 分阶段实现，先支持等值查询 |
| 与现有代码冲突 | 低 | 中 | 充分测试，保持向后兼容 |
| 性能优化不及预期 | 中 | 中 | 添加性能测试基准 |

## 7. 附录

### 7.1 参考代码

NebulaGraph 参考实现:
- `nebula-3.8.0/src/graph/planner/match/PropIndexSeek.cpp`
- `nebula-3.8.0/src/graph/planner/match/VariablePropIndexSeek.cpp`
- `nebula-3.8.0/src/graph/planner/ngql/SubgraphPlanner.cpp`

### 7.2 相关文档

- `docs/release/查询语法分析.md`
- `src/query/planner/__analysis__/module_relationships.md`
