# NebulaGraph Pattern 系统分析报告

**文档版本**: 1.0  
**创建日期**: 2026 年 3 月 3 日  
**分析对象**: nebula-graph 3.8.0 Pattern 系统  
**分析目的**: 评估当前 graphDB 项目的通配符匹配和条件匹配功能完整性

---

## 1. 执行摘要

本报告分析了 nebula-graph 中的 Pattern 匹配系统，并与当前 graphDB 项目进行对比分析。分析发现：

- **核心功能完整**: 当前项目已实现 6 种查找策略和 PatternApply 执行器
- **通配符支持不足**: 缺少 `anyLabel` 通配符标签扫描和全量边类型扫描
- **优化空间较大**: 索引选择、OR 条件处理等需要增强

**建议优先级**:
1. 🔴 **高优先级**: 补充 `anyLabel` 通配符支持
2. 🔴 **高优先级**: 补充 RollUpApply 路径收集功能
3. 🟡 **中优先级**: 索引选择优化
4. 🟡 **中优先级**: OR 条件索引嵌入

---

## 2. NebulaGraph Pattern 系统架构

### 2.1 核心组件

```
src/graph/
├── executor/query/
│   └── PatternApplyExecutor.{h,cpp}       # 模式应用执行器
├── planner/plan/
│   └── Query.h                             # PatternApply 计划节点定义
├── planner/match/
│   ├── SegmentsConnector.{h,cpp}           # 计划连接工具
│   ├── MatchClausePlanner.{h,cpp}          # MATCH 子句计划器
│   ├── MatchPathPlanner.{h,cpp}            # 路径计划器
│   ├── WhereClausePlanner.{h,cpp}          # WHERE 子句计划器
│   ├── MatchSolver.{h,cpp}                 # 匹配求解器
│   └── StartVidFinder.{h,cpp}              # 起始点查找器
└── context/ast/
    └── CypherAstContext.h                  # Path/NodeInfo/EdgeInfo 等上下文结构
```

### 2.2 核心数据结构

#### Path 结构体
```cpp
struct Path final {
  bool anonymous{true};
  std::string alias;
  std::vector<NodeInfo> nodeInfos;
  std::vector<EdgeInfo> edgeInfos;
  PathBuildExpression* pathBuild{nullptr};

  // Pattern 表达式相关
  bool rollUpApply{false};                 // 是否收集路径到列表
  std::vector<std::string> compareVariables;
  std::string collectVariable;

  // Pattern 谓词标志
  bool isPred{false};      // 是否为模式谓词 (EXISTS/NOT EXISTS)
  bool isAntiPred{false};  // 是否为反模式谓词
  bool genPath{true};      // 是否生成路径结构

  enum PathType { kDefault, kAllShortest, kSingleShortest };
  PathType pathType{PathType::kDefault};
};
```

#### NodeInfo / EdgeInfo 结构体
```cpp
struct NodeInfo {
  bool anonymous{false};
  std::vector<TagID> tids;
  std::vector<std::string> labels;
  std::vector<MapExpression*> labelProps;  // 模式内属性
  std::string alias;
  const MapExpression* props{nullptr};
  Expression* filter{nullptr};              // WHERE 子句过滤
};

struct EdgeInfo {
  bool anonymous{false};
  std::unique_ptr<MatchStepRange> range{nullptr};
  std::vector<EdgeType> edgeTypes;
  MatchEdge::Direction direction{MatchEdge::Direction::OUT_EDGE};
  std::vector<std::string> types;
  std::string alias;
  std::string innerAlias;
  const MapExpression* props{nullptr};
  Expression* filter{nullptr};
};
```

#### ScanInfo 结构体
```cpp
struct ScanInfo {
  Expression* filter{nullptr};
  std::vector<int32_t> schemaIds;
  std::vector<std::string> schemaNames;
  std::vector<IndexID> indexIds;
  MatchEdge::Direction direction{MatchEdge::Direction::OUT_EDGE};
  bool anyLabel{false};  // 通配符标志：空标签表示所有标签
};
```

---

## 3. 通配符匹配分析

### 3.1 标签通配符 (`anyLabel`)

**使用场景**: `MATCH (n) RETURN n` (无标签扫描)

**nebula-graph 实现**:
```cpp
// ScanSeek::matchNode
bool ScanSeek::matchNode(NodeContext *nodeCtx) {
  auto &node = *nodeCtx->info;
  if (node.tids.empty()) {
    // 空标签意味着所有标签
    const auto *qctx = nodeCtx->qctx;
    auto allLabels = qctx->schemaMng()->getAllTags(nodeCtx->spaceId);
    for (const auto &label : allLabels.value()) {
      nodeCtx->scanInfo.schemaIds.emplace_back(label.first);
      nodeCtx->scanInfo.schemaNames.emplace_back(label.second);
    }
    nodeCtx->scanInfo.anyLabel = true;  // 设置通配符标志
  }
  return true;
}
```

**执行逻辑** (ScanSeek::transformNode):
```cpp
// Filter vertices lack labels
Expression *prev = nullptr;
for (const auto &tag : nodeCtx->scanInfo.schemaNames) {
  auto *tagPropExpr = TagPropertyExpression::make(pool, tag, kTag);
  auto *notEmpty = UnaryExpression::makeIsNotEmpty(pool, tagPropExpr);
  if (prev != nullptr) {
    if (anyLabel) {
      // 通配符：任意标签存在即可 (OR 逻辑)
      auto *orExpr = LogicalExpression::makeOr(pool, prev, notEmpty);
      prev = orExpr;
    } else {
      // 多标签：所有标签都必须存在 (AND 逻辑)
      auto *andExpr = LogicalExpression::makeAnd(pool, prev, notEmpty);
      prev = andExpr;
    }
  } else {
    prev = notEmpty;
  }
}
```

### 3.2 边类型通配符

**使用场景**: `MATCH (a)-[]->(b) RETURN a,b` (任意边类型)

**nebula-graph 处理**:
- `EdgeInfo::edgeTypes` 为空时，获取所有边类型
- 在 `Traverse` 节点中设置空边类型列表表示任意边

---

## 4. 条件匹配分析

### 4.1 条件匹配的四种策略

| 策略 | 匹配条件 | 执行方式 |
|------|----------|----------|
| `VertexIdSeek` | `id(n) = value` | 直接通过顶点 ID 定位 |
| `LabelIndexSeek` | 仅有 Label (`:Tag`) | 标签索引扫描 |
| `PropIndexSeek` | `n:Tag{prop: value}` 或 `WHERE n.prop = value` | 属性索引扫描 |
| `ScanSeek` | 无条件或任意条件 | 全量扫描 + 过滤 |

### 4.2 PropIndexSeek - 属性索引匹配

**匹配逻辑**:
```cpp
bool PropIndexSeek::matchNode(NodeContext* nodeCtx) {
  // 仅支持单标签
  if (node.labels.size() != 1) {
    return false;
  }

  // 从 WHERE 子句构建过滤
  Expression* filterInWhere = nullptr;
  if (nodeCtx->bindWhereClause != nullptr && nodeCtx->bindWhereClause->filter != nullptr) {
    filterInWhere = MatchSolver::makeIndexFilter(...);
  }
  
  // 从模式内属性构建过滤
  Expression* filterInPattern = nullptr;
  if (!node.labelProps.empty()) {
    filterInPattern = MatchSolver::makeIndexFilter(...);
  }

  // 合并两种过滤
  if (!filterInPattern && !filterInWhere) {
    return false;  // 没有可索引的条件
  }
  // ...
}
```

**索引过滤器构建** (MatchSolver::makeIndexFilter):
```cpp
Expression* MatchSolver::makeIndexFilter(const std::string& label,
                                         const std::string& alias,
                                         Expression* filter,
                                         QueryContext* qctx,
                                         bool isEdgeProperties) {
  // 支持的关系操作符
  static const std::unordered_set<Expression::Kind> kinds = {
      Expression::Kind::kRelEQ,   // =
      Expression::Kind::kRelLT,   // <
      Expression::Kind::kRelLE,   // <=
      Expression::Kind::kRelGT,   // >
      Expression::Kind::kRelGE,   // >=
  };

  // 提取 AND 条件中的所有关系表达式
  // 为每个条件创建 TagPropertyExpression/EdgePropertyExpression
  // 返回合并后的过滤表达式
}
```

### 4.3 LabelIndexSeek - 标签索引匹配

```cpp
bool LabelIndexSeek::matchNode(NodeContext* nodeCtx) {
  // 仅支持单标签索引
  if (node.tids.size() != 1) {
    return false;
  }

  nodeCtx->scanInfo.schemaIds = node.tids;
  nodeCtx->scanInfo.schemaNames = node.labels;

  // 选择最优索引（字段数最少的）
  auto indexResult = pickTagIndex(nodeCtx);
  if (!indexResult.ok()) {
    return false;
  }

  nodeCtx->scanInfo.indexIds = std::move(indexResult).value();
  return true;
}
```

**索引选择优化**:
```cpp
static std::shared_ptr<meta::cpp2::IndexItem> selectIndex(
    const std::shared_ptr<meta::cpp2::IndexItem> candidate,
    const std::shared_ptr<meta::cpp2::IndexItem> income) {
  // 字段数少的索引优先 (更精确)
  if (candidate->get_fields().size() > income->get_fields().size()) {
    return income;
  }
  return candidate;
}
```

---

## 5. PatternApply 执行器

### 5.1 核心功能

`PatternApply` 实现模式谓词语义，用于处理 `WHERE EXISTS((n)-[:likes]->())` 这类查询。

**执行流程**:
```
┌─────────────┐     ┌─────────────┐
│  Left Input │     │ Right Input │
│  (待匹配行)  │     │  (模式结果)  │
└──────┬──────┘     └──────┬──────┘
       │                   │
       └────────┬──────────┘
                │
          ┌─────▼─────┐
          │ Pattern   │
          │ Apply     │
          │ Executor  │
          └─────┬─────┘
                │
         如果 isAntiPred = false:
           保留右侧有匹配的行
         如果 isAntiPred = true:
           保留右侧无匹配的行
```

### 5.2 关键方法

```cpp
// 零键匹配 (无连接条件)
DataSet applyZeroKey(Iterator* appliedIter, const bool allValid);

// 单键匹配
DataSet applySingleKey(Expression* appliedKey,
                       Iterator* appliedIter,
                       const std::unordered_set<Value>& validKey);

// 多键匹配
DataSet applyMultiKey(std::vector<Expression*> appliedKeys,
                      Iterator* appliedIter,
                      const std::unordered_set<List>& validKeys);
```

### 5.3 反谓词逻辑

```cpp
// XOR 操作实现反谓词 (NOT EXISTS)
bool applyFlag = (validKeys.find(val) != validKeys.end()) ^ isAntiPred_;
```

---

## 6. 冗余分析

### 6.1 数据结构冗余

#### 🔴 问题 1: `filter` 字段的多重存储

```cpp
struct NodeInfo {
  std::vector<MapExpression*> labelProps;  // 模式内属性
  const MapExpression* props{nullptr};      // 重复！与 labelProps 功能重叠
  Expression* filter{nullptr};              // WHERE 子句过滤
};
```

**建议**: 合并 `labelProps` 和 `props` 为单一字段。

#### 🔴 问题 2: `ScanInfo` 与 `NodeInfo/EdgeInfo` 的信息重复

```cpp
struct ScanInfo {
  Expression* filter{nullptr};           // 与 NodeInfo::filter 重复
  std::vector<int32_t> schemaIds;        // 与 NodeInfo::tids 重复
  std::vector<std::string> schemaNames;  // 与 NodeInfo::labels 重复
  bool anyLabel{false};                  // 可从 labels.empty() 推断
};
```

**建议**: 使用引用或指针传递 `NodeInfo/EdgeInfo`，避免复制。

### 6.2 执行逻辑冗余

#### 🟡 问题 3: `getAllVertexProp` 未缓存

```cpp
// 在 MatchPathPlanner 中多次调用
auto vertexProps = SchemaUtil::getAllVertexProp(qctx, spaceId, true);
traverse->setVertexProps(std::move(vertexProps).value());

// ... 同一函数中再次调用
auto vertexProps = SchemaUtil::getAllVertexProp(qctx, spaceId, true);
appendV->setVertexProps(std::move(vertexProps).value());
```

**建议**: 在 `MatchPathPlanner` 中缓存结果。

#### 🟡 问题 4: `getEdgeProps` 的两个重载功能重叠

```cpp
// 重载 1: 基于 EdgeType 列表
static StatusOr<std::unique_ptr<std::vector<EdgeProp>>> getEdgeProps(
    QueryContext* qctx,
    const SpaceInfo& space,
    const std::vector<EdgeType>& edgeTypes,
    bool withProp);

// 重载 2: 基于 EdgeInfo
static std::unique_ptr<std::vector<storage::cpp2::EdgeProp>> getEdgeProps(
    const EdgeInfo& edge,
    bool reversely,
    QueryContext* qctx,
    GraphSpaceID spaceId);
```

**建议**: 合并为一个函数，使用可选参数处理方向。

---

## 7. 当前 graphDB 项目状态评估

### 7.1 已实现功能

| 功能模块 | 状态 | 文件位置 |
|---------|------|---------|
| **基础模式 AST** | ✅ | `query/parser/ast/pattern.rs` |
| **PatternApply 执行器** | ✅ | `query/executor/result_processing/transformations/pattern_apply.rs` |
| **匹配语句规划器** | ✅ | `query/planner/statements/match_statement_planner.rs` |
| **路径规划器** | ✅ | `query/planner/statements/paths/match_path_planner.rs` |
| **查找策略框架** | ✅ | `query/planner/statements/seeks/seek_strategy*.rs` |
| **6 种查找策略** | ✅ | VertexSeek, IndexSeek, PropIndexSeek, VariablePropIndexSeek, EdgeSeek, ScanSeek |
| **标签过滤** | ✅ | `query/executor/tag_filter.rs` |
| **模式重写** | ✅ | `query/planner/rewrite/pattern.rs` |

### 7.2 功能对比

| 功能 | nebula-graph | graphDB | 差距 |
|------|--------------|---------|------|
| **无标签扫描 (`anyLabel`)** | ✅ | ❌ | 高 |
| **全标签扫描** | ✅ | ⚠️ | 中 |
| **通配符边类型** | ✅ | ❌ | 高 |
| **ID 精确匹配** | ✅ | ✅ | 无 |
| **标签索引匹配** | ✅ | ✅ | 无 |
| **属性索引匹配** | ✅ | ✅ | 无 |
| **变量属性索引** | ✅ | ✅ | 无 |
| **多标签索引** | ⚠️ | ❌ | 低 |
| **OR 条件嵌入索引** | ✅ | ❌ | 中 |
| **索引选择优化** | ✅ | ⚠️ | 中 |
| **EXISTS 语义** | ✅ | ✅ | 无 |
| **NOT EXISTS 语义** | ✅ | ✅ | 无 |
| **路径收集 (RollUp)** | ✅ | ❌ | 高 |

---

## 8. 需要补充的功能

### 8.1 高优先级 (🔴)

#### 8.1.1 通配符标签匹配 (`anyLabel`)

**问题**: 当前无法处理 `MATCH (n) RETURN n` (无标签扫描)

**需要修改的文件**:
- `query/planner/statements/seeks/scan_seek.rs`
- `query/planner/statements/match_statement_planner.rs`
- `query/planner/statements/seeks/seek_strategy_base.rs`

**实现方案**:
```rust
// 在 ScanSeek 中添加通配符支持
pub struct ScanSeek {
    any_label: bool,  // 新增字段
}

impl SeekStrategy for ScanSeek {
    fn execute<S: StorageClient>(
        &self,
        storage: &S,
        context: &SeekStrategyContext,
    ) -> Result<SeekResult, StorageError> {
        if self.any_label {
            // 扫描所有标签的顶点
            self.scan_all_labels(storage, context)
        } else {
            // 当前逻辑
            self.scan_specific_labels(storage, context)
        }
    }
}
```

#### 8.1.2 通配符边类型匹配

**问题**: 当前无法处理 `MATCH (a)-[]->(b) RETURN a,b` (任意边类型)

**实现方案**:
```rust
impl MatchStatementPlanner {
    fn plan_pattern_edge(
        &self,
        edge: &EdgePattern,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let edge_types = match &edge.types {
            Some(types) => types.clone(),
            None => self.get_all_edge_types(space_id)?,  // 获取所有边类型
        };
        // ...
    }
}
```

#### 8.1.3 路径收集 (RollUpApply)

**问题**: 当前无法处理路径变量返回

```cypher
MATCH p = (a)-[:KNOWS]->(b)
RETURN p  -- 当前无法处理
```

**需要补充**:
- 新增 `RollUpApply` 执行器
- 在 `PatternApply` 基础上添加路径收集逻辑
- 修改 `plan_path_pattern` 支持 `path.isPred` 判断

### 8.2 中优先级 (🟡)

#### 8.2.1 索引选择优化

**问题**: 当前索引选择过于简化

**实现方案**:
```rust
pub struct IndexInfo {
    pub name: String,
    pub properties: Vec<String>,
    pub selectivity: f32,  // 新增：选择性估计
}

impl SeekStrategySelector {
    pub fn select_best_index(&self, indexes: &[IndexInfo]) -> Option<&IndexInfo> {
        indexes.iter()
            .min_by(|a, b| {
                // 字段数少的优先
                // 选择性高的优先
                a.properties.len().cmp(&b.properties.len())
            })
    }
}
```

#### 8.2.2 OR 条件索引嵌入

**问题**: 无法利用索引处理 `WHERE n.age = 10 OR n.age = 20`

**nebula-graph 参考**:
```cpp
if (filter->kind() == Expression::Kind::kLogicalOr) {
    auto exprs = ExpressionUtils::collectAll(filter, {kLabelTagProperty});
    // 检查是否所有 OR 条件都是同一标签的同一属性
    // 如果是，嵌入到 IndexScan 中
}
```

#### 8.2.3 多标签索引支持

**问题**: `MATCH (n:Person:Actor)` 无法使用复合索引

### 8.3 低优先级 (🟢)

#### 8.3.1 模式匹配优化器

使用 `query/planner/rewrite/pattern.rs` 的 `Pattern` 匹配框架，实现：
- 连接顺序优化
- 过滤器下推
- 索引合并

---

## 9. 总结

### 9.1 完成度评估

| 类别 | 数量 | 完成度 |
|------|------|--------|
| **核心功能** | 6 项 | ✅ 100% |
| **通配符匹配** | 3 项 | ⚠️ 33% |
| **条件匹配优化** | 5 项 | ⚠️ 60% |
| **高级功能** | 3 项 | ❌ 0% |

### 9.2 整体评价

- **基础功能完整**: 已有 6 种查找策略和 PatternApply 执行器
- **通配符支持不足**: 缺少 `anyLabel` 和全量边类型扫描
- **优化空间较大**: 索引选择、OR 条件处理等需要增强

### 9.3 建议优先级

1. **立即补充**: `anyLabel` 通配符支持 (影响基础查询)
2. **近期补充**: RollUpApply 路径收集 (影响路径返回)
3. **中期优化**: 索引选择优化、OR 条件嵌入

---

## 附录 A: 关键文件清单

### nebula-graph
- `src/graph/executor/query/PatternApplyExecutor.h`
- `src/graph/executor/query/PatternApplyExecutor.cpp`
- `src/graph/planner/plan/Query.h` (PatternApply 定义)
- `src/graph/planner/match/SegmentsConnector.cpp`
- `src/graph/planner/match/MatchPathPlanner.cpp`
- `src/graph/planner/match/ScanSeek.cpp`
- `src/graph/planner/match/LabelIndexSeek.cpp`
- `src/graph/planner/match/PropIndexSeek.cpp`
- `src/graph/context/ast/CypherAstContext.h`

### graphDB
- `src/query/parser/ast/pattern.rs`
- `src/query/executor/result_processing/transformations/pattern_apply.rs`
- `src/query/planner/statements/match_statement_planner.rs`
- `src/query/planner/statements/paths/match_path_planner.rs`
- `src/query/planner/statements/seeks/seek_strategy_base.rs`
- `src/query/planner/statements/seeks/scan_seek.rs`
- `src/query/executor/tag_filter.rs`
- `src/query/planner/rewrite/pattern.rs`
