# PlanNode 节点分类分析文档

## 概述

本文档描述 GraphDB 查询计划节点（PlanNode）的分类体系设计，基于功能特性和职责对节点进行分类，以提高代码可读性和可维护性。

## 分类体系

根据节点的职责和功能特性，将所有 PlanNode 分为以下七个类别：

### 1. 访问层（Access Layer）

**职责**：从存储层读取数据，是执行计划的起始点。

| 节点类型 | 说明 | 依赖 |
|---------|------|-----|
| StartNode | 起始节点，执行计划的入口 | 无 |
| ScanVerticesNode | 全表扫描顶点 | 无 |
| ScanEdgesNode | 全表扫描边 | 无 |
| GetVerticesNode | 按ID/属性获取顶点 | 索引 |
| GetEdgesNode | 按ID/属性获取边 | 索引 |
| GetNeighborsNode | 获取顶点的邻居节点 | 顶点 |
| IndexScanNode | 索引扫描节点 | 索引 |
| FulltextIndexScanNode | 全文索引扫描 | 索引 |

### 2. 操作层（Operation Layer）

**职责**：对数据进行转换、过滤、聚合等操作。

| 节点类型 | 说明 | 依赖 |
|---------|------|-----|
| FilterNode | 条件过滤 | 输入数据流 |
| ProjectNode | 投影/列选择 | 输入数据流 |
| AggregateNode | 聚合运算（GROUP BY） | 输入数据流 |
| SortNode | 排序 | 输入数据流 |
| LimitNode | 限制返回行数 | 输入数据流 |
| TopNNode | Top N 排序 | 输入数据流 |
| SampleNode | 采样 | 输入数据流 |
| DedupNode | 去重 | 输入数据流 |

### 3. 连接层（Join Layer）

**职责**：多数据流的连接操作。

| 节点类型 | 说明 | 依赖 |
|---------|------|-----|
| InnerJoinNode | 内连接 | 两个输入流 |
| LeftJoinNode | 左连接 | 两个输入流 |
| CrossJoinNode | 交叉连接 | 两个输入流 |
| HashInnerJoinNode | 哈希内连接 | 两个输入流 |
| HashLeftJoinNode | 哈希左连接 | 两个输入流 |
| CartesianProductNode | 笛卡尔积 | 两个输入流 |

### 4. 遍历层（Traversal Layer）

**职责**：图数据的遍历和扩展。

| 节点类型 | 说明 | 依赖 |
|---------|------|-----|
| ExpandNode | 扩展边 | 顶点 |
| ExpandAllNode | 全扩展 | 顶点 |
| TraverseNode | 遍历 | 顶点/边 |
| AppendVerticesNode | 追加顶点 | 顶点/遍历结果 |

### 5. 控制流层（Control Flow Layer）

**职责**：执行流程控制。

| 节点类型 | 说明 | 依赖 |
|---------|------|-----|
| ArgumentNode | 参数传递 | 依赖特定 |
| LoopNode | 循环执行 | 循环体 |
| PassThroughNode | 直通传递 | 输入流 |
| SelectNode | 条件选择 | 多分支 |

### 6. 数据处理层（Data Processing Layer）

**职责**：复杂数据操作和转换。

| 节点类型 | 说明 | 依赖 |
|---------|------|-----|
| DataCollectNode | 数据收集 | 多输入流 |
| UnionNode | 并集操作 | 多输入流 |
| UnwindNode | 展开数组 | 输入数据流 |
| AssignNode | 变量赋值 | 输入数据流 |
| PatternApplyNode | 模式应用 | 模式匹配 |
| RollUpApplyNode | 上卷应用 | 聚合模式 |

### 7. 算法层（Algorithm Layer）

**职责**：图算法执行。

| 节点类型 | 说明 | 依赖 |
|---------|------|-----|
| ShortestPathNode | 最短路径 | 起点/终点 |
| AllPathsNode | 所有路径 | 起点/终点 |
| MultiShortestPathNode | 多源最短路径 | 多起点 |
| BFSShortestNode | BFS最短路径 | 起点 |

### 8. 管理/DDL层（Management Layer）

**职责**：元数据管理和DDL操作。

| 节点类型 | 说明 | 依赖 |
|---------|------|-----|
| CreateSpaceNode | 创建图空间 | 无 |
| DropSpaceNode | 删除图空间 | 无 |
| DescSpaceNode | 描述图空间 | 无 |
| ShowSpacesNode | 显示所有图空间 | 无 |
| CreateTagNode | 创建标签 | 图空间 |
| AlterTagNode | 修改标签 | 标签 |
| DescTagNode | 描述标签 | 标签 |
| DropTagNode | 删除标签 | 标签 |
| ShowTagsNode | 显示所有标签 | 图空间 |
| CreateEdgeNode | 创建边类型 | 图空间 |
| AlterEdgeNode | 修改边类型 | 边类型 |
| DescEdgeNode | 描述边类型 | 边类型 |
| DropEdgeNode | 删除边类型 | 边类型 |
| ShowEdgesNode | 显示所有边类型 | 图空间 |
| CreateTagIndexNode | 创建标签索引 | 标签 |
| DropTagIndexNode | 删除标签索引 | 索引 |
| DescTagIndexNode | 描述标签索引 | 索引 |
| ShowTagIndexesNode | 显示所有标签索引 | 图空间 |
| CreateEdgeIndexNode | 创建边索引 | 边类型 |
| DropEdgeIndexNode | 删除边索引 | 索引 |
| DescEdgeIndexNode | 描述边索引 | 索引 |
| ShowEdgeIndexesNode | 显示所有边索引 | 图空间 |
| RebuildTagIndexNode | 重建标签索引 | 索引 |
| RebuildEdgeIndexNode | 重建边索引 | 索引 |

## 分类使用示例

### 节点分类识别

```rust
use crate::query::planner::plan::core::nodes::PlanNodeCategory;

impl PlanNodeEnum {
    /// 获取节点所属分类
    pub fn category(&self) -> PlanNodeCategory {
        match self {
            // 访问层
            PlanNodeEnum::Start(_) => PlanNodeCategory::Access,
            PlanNodeEnum::ScanVertices(_) => PlanNodeCategory::Access,
            PlanNodeEnum::ScanEdges(_) => PlanNodeCategory::Access,
            PlanNodeEnum::GetVertices(_) => PlanNodeCategory::Access,
            PlanNodeEnum::GetEdges(_) => PlanNodeCategory::Access,
            PlanNodeEnum::GetNeighbors(_) => PlanNodeCategory::Access,
            PlanNodeEnum::IndexScan(_) => PlanNodeCategory::Access,
            PlanNodeEnum::FulltextIndexScan(_) => PlanNodeCategory::Access,

            // 操作层
            PlanNodeEnum::Filter(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::Project(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::Aggregate(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::Sort(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::Limit(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::TopN(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::Sample(_) => PlanNodeCategory::Operation,
            PlanNodeEnum::Dedup(_) => PlanNodeCategory::Operation,

            // ... 其他分类
        }
    }
}
```

### 优化器使用场景

1. **下推过滤**：操作层节点优先于访问层节点
2. **连接重排**：连接层节点根据代价模型重排
3. **索引使用**：访问层节点优先使用索引
4. **并行执行**：数据处理层节点可并行

## 命名规范

### 统一命名规则

| 分类 | 前缀 | 示例 |
|-----|------|-----|
| 访问层 | Scan/Get | ScanVertices, GetNeighbors |
| 操作层 | 动词+名词 | Filter, Project, Aggregate |
| 连接层 | Join/Product | InnerJoin, CartesianProduct |
| 遍历层 | Expand/Traverse | Expand, Traverse |
| 控制流 | 描述性名称 | Argument, PassThrough |
| 数据处理 | 描述性名称 | Union, Unwind |
| 算法层 | 算法名称 | ShortestPath, BFSShortest |
| 管理/DDL | 操作+对象 | CreateSpace, DropTag |

### 不一致命名示例（已处理）

| 当前名称 | 状态 | 说明 |
|---------|------|-----|
| CrossJoinNode / CartesianProductNode | 已标准化 | 两者均为有效别名，映射到相同的 `CrossJoinNode` 结构体。后续新代码统一使用 `CrossJoin` 变体 |

### 命名规范指南

#### 1. 节点命名规则

| 分类 | 推荐后缀 | 示例 |
|-----|---------|-----|
| 访问层 | Scan/Get | `ScanVerticesNode`, `GetNeighborsNode` |
| 操作层 | 动词+名词 | `FilterNode`, `ProjectNode`, `AggregateNode` |
| 连接层 | Join/Product | `InnerJoinNode`, `CrossJoinNode` |
| 遍历层 | Expand/Traverse | `ExpandNode`, `TraverseNode` |
| 控制流 | 描述性名称 | `ArgumentNode`, `PassThroughNode` |
| 数据处理 | 描述性名称 | `UnionNode`, `UnwindNode` |
| 算法层 | 算法名称+Node | `ShortestPathNode`, `BFSShortestNode` |
| 管理/DDL | 操作+对象+Node | `CreateSpaceNode`, `DropTagNode` |

#### 2. 命名一致性原则

- **避免同义多名**：如果两个名称表示相同概念，选择一个作为标准
- **向后兼容**：如果已有代码使用某名称，考虑保留别名
- **语义清晰**：名称应能表达节点的功能
- **遵循惯例**：参考 SQL 标准语法（如 `CROSS JOIN` 而非 `CartesianProduct`）

#### 3. 现有别名处理

```
CrossJoinNode 和 CartesianProduct(CrossJoinNode) 是同一节点类型的两个变体名称。
- 推荐使用：CrossJoin（符合 SQL 标准）
- 兼容保留：CartesianProduct（已有代码依赖）
```
