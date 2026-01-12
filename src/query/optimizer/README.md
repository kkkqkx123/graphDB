# Query Optimizer

Query optimizer模块负责优化查询执行计划。该模块包含各种查询优化规则和实现，借鉴了NebulaGraph的优化器设计。

## 文件结构

### `mod.rs`
定义优化器模块的整体结构，导出所有子模块并重新导出所有规则结构：
- 导出 filter_rules 模块：过滤相关规则
- 导出 projection_rules 模块：投影相关规则
- 导出 general_rules 模块：通用规则
- 导出 index_rules 模块：索引相关规则
- 导出 index_scan_rules 模块：索引扫描相关规则
- 导出 join_rules 模块：连接相关规则
- 导出 limit_rules 模块：限制相关规则
- 导出 optimizer 模块：优化器主实现
- 重新导出所有规则结构体以提供统一访问入口

### `optimizer.rs`
优化器的核心实现文件，定义以下关键组件：
- `OptContext`: 优化过程中的上下文信息
- `OptimizationStats`: 优化统计信息
- `OptGroup` 和 `OptGroupNode`: 表示优化过程中的等价计划节点组
- `PlanNodeProperties`: 计划节点的属性描述
- `OptRule` trait: 所有优化规则需要实现的基础trait
- `Pattern`: 定义匹配计划节点的模式
- `RuleSet`: 规则集合管理
- `Optimizer`: 优化器主类，实现查询计划的优化算法
- `OptimizerError`: 优化过程中可能出现的错误类型

### `filter_rules.rs`
专门针对过滤操作的优化规则：
- `FilterPushDownRule`: 向计划树下层推送过滤条件以减少数据传输量
- `PushFilterDownTraverseRule`: 向遍历操作下层推送过滤条件
- `PushFilterDownExpandRule`: 向扩展操作下层推送过滤条件
- `CombineFilterRule`: 合并多个过滤操作
- `EliminateFilterRule`: 消除冗余的过滤操作
- `PredicatePushDownRule`: 将谓词条件尽可能推送到存储层

### `projection_rules.rs`
专门针对投影操作的优化规则：
- `ProjectionPushDownRule`: 向计划树下层推送投影操作以减少数据传输
- `CollapseProjectRule`: 折叠多个投影操作
- `RemoveNoopProjectRule`: 移除无操作的投影操作
- `PushProjectDownRule`: 向数据源推送投影操作

### `general_rules.rs`
通用优化规则，不特定于某一种操作类型：
- `DedupEliminationRule`: 消除重复操作
- `JoinOptimizationRule`: 优化连接操作
- `LimitOptimizationRule`: 优化LIMIT操作
- `IndexFullScanRule`: 优化索引全扫描为更高效的全表扫描（在特定场景）
- `TopNRule`: 优化Top-N查询操作
- `EliminateAppendVerticesRule`: 消除冗余的添加顶点操作
- `MergeGetVerticesAndProjectRule`: 合并获取顶点和投影操作
- `ScanWithFilterOptimizationRule`: 优化带过滤条件的扫描操作

### `index_rules.rs`
专门针对索引操作的优化规则：
- `OptimizeEdgeIndexScanByFilterRule`: 基于过滤条件优化边索引扫描
- `OptimizeTagIndexScanByFilterRule`: 基于过滤条件优化标签索引扫描
- `PushLimitDownRule`: 向下推送LIMIT操作

### `index_scan_rules.rs`
专门针对索引扫描操作的优化规则：
- `EdgeIndexFullScanRule`: 优化边索引全扫描操作
- `TagIndexFullScanRule`: 优化标签索引全扫描操作
- `IndexScanRule`: 通用的索引扫描优化
- `UnionAllEdgeIndexScanRule`: 优化边索引扫描的UNION ALL操作
- `UnionAllTagIndexScanRule`: 优化标签索引扫描的UNION ALL操作

### `join_rules.rs`
专门针对连接操作的优化规则：
- `PushFilterDownHashInnerJoinRule`: 向哈希内连接下层推送过滤条件
- `PushFilterDownHashLeftJoinRule`: 向哈希左连接下层推送过滤条件
- `PushFilterDownInnerJoinRule`: 向内连接下层推送过滤条件
- `MergeGetVerticesAndDedupRule`: 合并获取顶点和去重操作
- `MergeGetVerticesAndProjectRule`: 合并获取顶点和投影操作
- `MergeGetNbrsAndDedupRule`: 合并获取邻居和去重操作
- `MergeGetNbrsAndProjectRule`: 合并获取邻居和投影操作
- `RemoveAppendVerticesBelowJoinRule`: 移除连接下方的添加顶点操作

### `limit_rules.rs`
专门针对限制操作（LIMIT）的优化规则，主要实现将LIMIT操作推向底层执行以减少数据处理量：
- `PushLimitDownGetVerticesRule`: 向获取顶点操作推送LIMIT
- `PushLimitDownGetNeighborsRule`: 向获取邻居操作推送LIMIT
- `PushLimitDownGetEdgesRule`: 向获取边操作推送LIMIT
- `PushLimitDownScanVerticesRule`: 向扫描顶点操作推送LIMIT
- `PushLimitDownScanEdgesRule`: 向扫描边操作推送LIMIT
- `PushLimitDownIndexScanRule`: 向索引扫描操作推送LIMIT
- `PushLimitDownProjectRule`: 向投影操作推送LIMIT
- `PushLimitDownAllPathsRule`: 向全路径操作推送LIMIT
- `PushLimitDownExpandAllRule`: 向全展开操作推送LIMIT

## 工作流程

优化器通过以下步骤优化查询计划：

1. 将原始查询计划转换为优化图结构（OptGroup）
2. 应用逻辑优化规则（如谓词下推、投影下推等）
3. 应用物理优化规则（如索引扫描优化、JOIN优化等）
4. 将优化后的图结构转换回执行计划
5. 返回优化后的查询计划给执行引擎

## 设计原则

- 模式匹配：每个优化规则都定义了其适用的计划节点模式
- 迭代应用：规则会在计划上迭代应用直到不能再进行优化
- 规则独立：每个规则都是独立的单元，可以单独测试和维护
- 统一访问：通过mod.rs统一导出所有规则结构体，提供便捷访问
- 功能分组：规则按功能分组到专门模块中，便于维护和理解