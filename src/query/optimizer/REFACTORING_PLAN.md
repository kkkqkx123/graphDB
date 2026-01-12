# 查询优化器重构计划

## 概述

本文档描述了对查询优化器模块的重构计划，旨在解决当前组织结构中的职责交叉、代码重复和维护困难等问题。

## 当前问题分析

### 主要问题

1. **职责边界不清晰**：
   - `index_rules.rs` 和 `index_scan_rules.rs` 职责重叠
   - 通用规则和特定规则之间的边界模糊

2. **代码重复严重**：
   - `MergeGetVerticesAndProjectRule` 在 `general_rules.rs` 和 `join_rules.rs` 中重复实现
   - 多个 `PushLimitDown*Rule` 有相似的实现模式
   - 过滤条件下推逻辑分散在多个文件中

3. **规则分散**：
   - 相关功能的规则分散在不同文件中
   - 增加了维护成本和出错风险

4. **扩展性差**：
   - 添加新规则时难以确定合适的归属
   - 涉及多种操作类型的规则难以分类

## 重构方案

### 1. 新的组织结构

```
src/query/optimizer/
├── mod.rs                    # 模块导出
├── optimizer.rs              # 优化器核心实现
├── README.md                 # 文档
├── rule_traits.rs            # 规则通用trait和工具（新增）
├── rule_patterns.rs          # 常用模式匹配（新增）
├── predicate_pushdown.rs     # 谓词下推优化（重组）
├── limit_pushdown.rs         # LIMIT下推优化（重组）
├── projection_pushdown.rs    # 投影下推优化（重组）
├── operation_merge.rs        # 操作合并优化（重组）
├── scan_optimization.rs      # 扫描优化（重组）
├── index_optimization.rs     # 索引优化（合并）
├── join_optimization.rs      # 连接优化（重组）
└── elimination_rules.rs      # 消除优化（重组）
```

### 2. 模块职责说明

#### 基础设施模块

**rule_traits.rs**
- 定义通用规则接口和辅助trait
- 提供规则实现的辅助函数和宏
- 减少代码重复，统一实现模式

**rule_patterns.rs**
- 定义常用的模式匹配逻辑
- 提供可复用的模式匹配组件
- 简化规则实现

#### 优化策略模块

**predicate_pushdown.rs** - 谓词下推优化
- 整合所有过滤条件下推规则
- 统一处理通用过滤条件下推和特定操作的过滤条件下推
- 包含规则：
  - `FilterPushDownRule`
  - `PushFilterDownTraverseRule`
  - `PushFilterDownExpandRule`
  - `PushFilterDownHashInnerJoinRule`
  - `PushFilterDownHashLeftJoinRule`
  - `PushFilterDownInnerJoinRule`
  - `PredicatePushDownRule`

**limit_pushdown.rs** - LIMIT下推优化
- 整合所有LIMIT下推相关规则
- 统一处理各种操作的LIMIT下推
- 包含规则：
  - `PushLimitDownRule`
  - `PushLimitDownGetVerticesRule`
  - `PushLimitDownGetNeighborsRule`
  - `PushLimitDownGetEdgesRule`
  - `PushLimitDownScanVerticesRule`
  - `PushLimitDownScanEdgesRule`
  - `PushLimitDownIndexScanRule`
  - `PushLimitDownProjectRule`
  - `PushLimitDownAllPathsRule`
  - `PushLimitDownExpandAllRule`

**projection_pushdown.rs** - 投影下推优化
- 整合所有投影下推相关规则
- 包含规则：
  - `ProjectionPushDownRule`
  - `PushProjectDownRule`

**operation_merge.rs** - 操作合并优化
- 整合所有操作合并规则
- 包含规则：
  - `CombineFilterRule`
  - `CollapseProjectRule`
  - `MergeGetVerticesAndProjectRule`
  - `MergeGetVerticesAndDedupRule`
  - `MergeGetNbrsAndDedupRule`
  - `MergeGetNbrsAndProjectRule`

**scan_optimization.rs** - 扫描优化
- 整合所有扫描相关优化
- 包含规则：
  - `ScanWithFilterOptimizationRule`
  - `IndexFullScanRule`

**index_optimization.rs** - 索引优化
- 合并原 `index_rules.rs` 和 `index_scan_rules.rs`
- 统一处理所有索引相关优化
- 包含规则：
  - `OptimizeEdgeIndexScanByFilterRule`
  - `OptimizeTagIndexScanByFilterRule`
  - `EdgeIndexFullScanRule`
  - `TagIndexFullScanRule`
  - `IndexScanRule`
  - `UnionAllEdgeIndexScanRule`
  - `UnionAllTagIndexScanRule`

**join_optimization.rs** - 连接优化
- 专注于连接算法和策略优化
- 包含规则：
  - `JoinOptimizationRule`

**elimination_rules.rs** - 消除优化
- 整合所有消除冗余操作的规则
- 包含规则：
  - `EliminateFilterRule`
  - `DedupEliminationRule`
  - `RemoveNoopProjectRule`
  - `EliminateAppendVerticesRule`
  - `RemoveAppendVerticesBelowJoinRule`
  - `TopNRule`

### 3. 规则迁移映射

| 原文件 | 新文件 | 规则 |
|--------|--------|------|
| filter_rules.rs | predicate_pushdown.rs | FilterPushDownRule, PushFilterDownTraverseRule, PushFilterDownExpandRule, PredicatePushDownRule |
| filter_rules.rs | operation_merge.rs | CombineFilterRule |
| filter_rules.rs | elimination_rules.rs | EliminateFilterRule |
| general_rules.rs | elimination_rules.rs | DedupEliminationRule, EliminateAppendVerticesRule |
| general_rules.rs | join_optimization.rs | JoinOptimizationRule |
| general_rules.rs | limit_pushdown.rs | LimitOptimizationRule |
| general_rules.rs | scan_optimization.rs | IndexFullScanRule, ScanWithFilterOptimizationRule |
| general_rules.rs | operation_merge.rs | MergeGetVerticesAndProjectRule |
| general_rules.rs | elimination_rules.rs | TopNRule |
| index_rules.rs + index_scan_rules.rs | index_optimization.rs | 所有索引相关规则 |
| join_rules.rs | predicate_pushdown.rs | PushFilterDown*Rule |
| join_rules.rs | operation_merge.rs | Merge*Rule |
| join_rules.rs | elimination_rules.rs | RemoveAppendVerticesBelowJoinRule |
| limit_rules.rs | limit_pushdown.rs | 所有LIMIT下推规则 |
| projection_rules.rs | projection_pushdown.rs | ProjectionPushDownRule, PushProjectDownRule |
| projection_rules.rs | operation_merge.rs | CollapseProjectRule |
| projection_rules.rs | elimination_rules.rs | RemoveNoopProjectRule |

## 实施计划

### 阶段1：基础设施准备

1. 创建 `rule_traits.rs`
   - 定义通用规则接口
   - 提供辅助函数和宏
   - 统一错误处理

2. 创建 `rule_patterns.rs`
   - 定义常用模式匹配逻辑
   - 提供可复用的模式匹配组件

### 阶段2：规则迁移

按以下顺序迁移规则文件：

1. `elimination_rules.rs` - 消除优化规则
2. `operation_merge.rs` - 操作合并规则
3. `predicate_pushdown.rs` - 谓词下推规则
4. `limit_pushdown.rs` - LIMIT下推规则
5. `projection_pushdown.rs` - 投影下推规则
6. `scan_optimization.rs` - 扫描优化规则
7. `index_optimization.rs` - 索引优化规则（合并两个文件）
8. `join_optimization.rs` - 连接优化规则

### 阶段3：更新和清理

1. 更新 `mod.rs`
   - 添加新模块导出
   - 保持向后兼容的导出

2. 更新 `optimizer.rs`
   - 更新规则引用
   - 确保所有规则正确注册

3. 清理旧文件
   - 删除已迁移的旧文件
   - 更新文档和注释

### 阶段4：测试和验证

1. 单元测试
   - 为每个新模块编写单元测试
   - 确保规则行为一致

2. 集成测试
   - 验证优化器整体功能
   - 确保性能没有退化

3. 文档更新
   - 更新 `README.md`
   - 添加新模块的文档

## 预期收益

1. **减少代码重复**：
   - 通过通用基础设施减少重复代码
   - 统一实现模式

2. **提高可维护性**：
   - 相关规则集中管理
   - 清晰的职责边界

3. **增强扩展性**：
   - 新规则更容易添加
   - 更好的模块化设计

4. **改善代码质量**：
   - 统一的命名和实现模式
   - 更好的测试覆盖率

## 风险和缓解措施

1. **兼容性风险**：
   - 通过保持导出兼容性缓解
   - 分阶段迁移，逐步验证

2. **性能风险**：
   - 通过基准测试验证性能
   - 确保重构不影响优化效果

3. **复杂性风险**：
   - 通过充分文档化缓解
   - 提供清晰的示例和指南

## 结论

通过这次重构，我们将解决当前组织结构中的主要问题，提高代码质量和可维护性，为未来的扩展奠定良好基础。重构将分阶段进行，确保每个阶段都可以独立验证和测试。