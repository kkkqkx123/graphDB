# Query 模块依赖流向分析报告

## 一、目录结构概览

```
src/query/
├── context/          # 查询上下文管理
├── executor/         # 执行器实现
├── optimizer/        # 查询优化器
├── parser/           # 查询解析器
├── planner/          # 查询规划器
└── __analysis__/     # 架构分析文档
```

## 二、模块依赖关系分析

### 2.1 依赖流向图

```
Parser → Context → Planner → Optimizer → Executor
   ↓         ↓          ↓           ↓           ↓
  AST    Validation  PlanNode   OptRule   Executor
```

### 2.2 各模块依赖详情

#### Parser 模块
**依赖**：
- `crate::core::Value`
- `crate::core::types::operators::*`

**被依赖**：
- Context 模块（41个文件）
- Planner 模块（45个文件）
- Executor 模块（部分）

#### Context 模块
**依赖**：
- `crate::core::Value`
- `crate::core::symbol::SymbolTable`
- `crate::expression::ExpressionContext`

**被依赖**：
- 所有查询模块（41个文件）

#### Planner 模块
**依赖**：
- `crate::query::context`（41个文件）
- `crate::query::parser`（部分）
- `crate::query::executor::base::EdgeDirection`（5个文件）

**被依赖**：
- Executor 模块（2个文件）
- Optimizer 模块（68个文件）

#### Optimizer 模块
**依赖**：
- `crate::query::planner`（68个文件）
- `crate::core::Expression`

**被依赖**：
- 无（或极少）

#### Executor 模块
**依赖**：
- `crate::query::planner`（2个文件）
- `crate::storage::StorageEngine`

**被依赖**：
- 无（或极少）

### 2.3 跨模块依赖统计

| 源模块 | 目标模块 | 依赖文件数 |
|--------|----------|-----------|
| Context | Parser | 41 |
| Planner | Context | 41 |
| Planner | Parser | 45 |
| Planner | Executor | 5 |
| Optimizer | Planner | 68 |
| Executor | Planner | 2 |

## 三、循环依赖检查结果

### 3.1 检查方法
通过分析所有 `use crate::query::*` 语句，检查是否存在模块间的循环引用。

### 3.2 检查结果

✅ **未发现循环依赖**

依赖流向符合分层架构原则：
```
Parser → Context → Planner → Optimizer → Executor
```

## 四、依赖关系问题

### 4.1 Planner 依赖 Executor（违反分层架构）

**问题位置**：
- `src/query/planner/ngql/subgraph_planner.rs:11`
- `src/query/planner/ngql/path_planner.rs:11`
- `src/query/planner/ngql/go_planner.rs:11`
- `src/query/planner/plan/core/nodes/factory.rs:18`
- `src/query/planner/plan/core/nodes/traversal_node.rs:8`

**依赖内容**：
```rust
use crate::query::executor::base::EdgeDirection;
```

**问题分析**：
- Planner 负责生成执行计划，不应依赖 Executor 的类型
- 这违反了分层架构原则，增加了模块耦合度
- EdgeDirection 应该定义在更底层的模块中

### 4.2 Context 成为中心化依赖点

**问题**：
- Context 被所有模块依赖（41个文件）
- Context 本身又依赖多个核心模块
- 形成了中心化的依赖结构

**影响**：
- 修改 Context 可能影响所有模块
- 增加了重构的风险和成本

### 4.3 Executor 依赖 Planner（合理）

**依赖位置**：
- `src/query/executor/factory.rs:8-9`

**依赖内容**：
```rust
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::{PlanNode, JoinNode, BinaryInputNode};
```

**分析**：
- 这是合理的依赖关系
- Executor 需要根据 PlanNodeEnum 创建对应的执行器
- 符合执行流程：Planner 生成计划 → Executor 执行计划

## 五、优化建议

### 5.1 修复 Planner 依赖 Executor 的问题

**方案**：将共享类型移到 Core 或 Common 模块

1. 将 `EdgeDirection` 移到 `src/core/types/` 或 `src/query/common/types.rs`
2. 更新所有引用该类型的文件
3. 确保 Planner 不再依赖 Executor

### 5.2 优化 Context 模块

**方案**：使用 Trait 抽象，减少直接依赖

1. 定义 Context Trait 接口
2. 各模块依赖 Trait 而非具体实现
3. 降低模块间的耦合度

### 5.3 保持合理的依赖关系

**保持**：
- Parser → Context → Planner → Optimizer → Executor
- Executor → Planner（工厂模式需要）

## 六、总结

### 6.1 优点
- ✅ 无循环依赖
- ✅ 依赖流向基本符合分层架构
- ✅ 模块职责相对清晰

### 6.2 问题
- ❌ Planner 依赖 Executor（违反分层）
- ❌ Context 成为中心化依赖点
- ❌ 部分类型定义位置不合理

### 6.3 优先级
**高优先级**：
1. 修复 Planner 依赖 Executor 的问题
2. 将共享类型移到合适的位置

**中优先级**：
3. 优化 Context 模块的依赖关系
