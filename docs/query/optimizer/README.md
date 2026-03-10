# 查询优化器Cost体系文档

## 目录

本文档目录包含GraphDB查询优化器Cost体系的完整分析和设计方案。

## 文档列表

### 1. [cost_system_analysis.md](./cost_system_analysis.md)
**Cost体系分析报告**

详细分析现有Cost体系的结构、实现机制及实际使用情况：
- 目录结构详解
- 核心组件分析（Config、Calculator、SelectivityEstimator等）
- 节点估算器架构
- 统计信息体系
- 当前局限性分析

### 2. [cost_system_extension_design.md](./cost_system_extension_design.md)
**Cost体系扩展设计方案**

提供各优化策略的详细设计方案：
- 优化策略价值与开销评估矩阵
- 第一阶段：基础增强（直方图、倾斜检测、双向BFS等）
- 第二阶段：运行时优化（统计反馈、CTE缓存）
- 第三阶段：高级优化（自适应连接、相关性统计）
- 第四阶段：未来优化（ML代价模型等）
- 关键设计决策

### 3. [implementation_roadmap.md](./implementation_roadmap.md)
**实施路线图**

详细的实施计划和时间安排：
- 分阶段任务分解
- 依赖关系分析
- 风险与缓解策略
- 成功指标

## 快速导航

### 如果你是第一次阅读
建议按以下顺序阅读：
1. [cost_system_analysis.md](./cost_system_analysis.md) - 了解现有体系
2. [cost_system_extension_design.md](./cost_system_extension_design.md) - 了解扩展方案
3. [implementation_roadmap.md](./implementation_roadmap.md) - 了解实施计划

### 如果你关注特定优化策略
- **直方图统计** → [cost_system_extension_design.md#21-直方图统计系统](./cost_system_extension_design.md#21-直方图统计系统)
- **数据倾斜检测** → [cost_system_extension_design.md#22-数据倾斜检测](./cost_system_extension_design.md#22-数据倾斜检测)
- **双向BFS遍历** → [cost_system_extension_design.md#23-双向bfs遍历优化](./cost_system_extension_design.md#23-双向bfs遍历优化)
- **CTE缓存** → [cost_system_extension_design.md#32-cte结果缓存](./cost_system_extension_design.md#32-cte结果缓存)

## 关键结论

### 现有体系优势
1. **模块化设计** - 各组件职责清晰，易于扩展
2. **统计信息支持** - Tag、EdgeType、Property三级统计
3. **多策略支持** - DP/贪心算法、多种连接算法、聚合策略
4. **图数据库特性** - 超级节点处理、多跳惩罚、遍历方向

### 优先实施建议
根据必要性评分，建议按以下优先级实施：

| 优先级 | 优化策略 | 必要性评分 | 预计收益 |
|-------|---------|-----------|---------|
| P0 | 直方图统计 | 95 | 选择性估计精度提升50% |
| P0 | 双向BFS遍历 | 90 | 最短路径性能提升10-100倍 |
| P0 | 数据倾斜检测 | 80 | 超级节点查询性能稳定 |
| P1 | 运行时统计反馈 | 85 | 自适应优化能力 |
| P1 | CTE结果缓存 | 75 | 重复查询性能提升 |

### 实施时间线
- **第1-2月**：基础增强（5个优化策略）
- **第2-3月**：运行时优化（2个优化策略）
- **第3-4月**：高级优化（2个优化策略）
- **第4-5月**：测试完善与文档

## 相关代码路径

```
src/query/optimizer/
├── cost/                       # 代价计算模块
│   ├── mod.rs
│   ├── config.rs              # 代价模型配置
│   ├── calculator.rs          # 代价计算器
│   ├── selectivity.rs         # 选择性估计器
│   ├── assigner.rs            # 代价赋值器
│   └── node_estimators/       # 节点估算器
├── stats/                      # 统计信息模块
│   ├── mod.rs
│   ├── manager.rs             # 统计信息管理器
│   ├── tag.rs                 # 标签统计
│   ├── edge.rs                # 边类型统计
│   └── property.rs            # 属性统计
├── strategy/                   # 优化策略模块
│   ├── index.rs               # 索引选择
│   ├── join_order.rs          # 连接顺序优化
│   ├── aggregate_strategy.rs  # 聚合策略选择
│   └── traversal_direction.rs # 遍历方向优化
└── engine.rs                   # 优化器引擎
```
