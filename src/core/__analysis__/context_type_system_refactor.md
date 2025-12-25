# Context 类型系统重构方案

## 问题分析

### 当前问题
1. **类型职责不清**
   - `types::query::FieldValue` 定义在 types 模块，但主要在 context 模块使用
   - `QueryParameter` 和 `FieldValue::Scalar` 功能重叠
   - `ExecutionContext` 和 `QueryExecutionContext` 使用不同的值类型

2. **依赖关系混乱**
   - `query` 目录大量使用 `Expression`（59个文件）
   - `FieldValue` 主要在 `core/context` 中使用（2个文件）
   - 违反了模块职责边界

3. **类型不一致**
   - `ExecutionContext` 使用 `FieldValue`（支持 Vertex/Edge/Path）
   - `QueryExecutionContext` 使用 `Value`（不支持图结构）
   - 导致类型转换和兼容性问题

### 现有文件职责

**query.rs - QueryContext**
- 查询级别的元数据管理
- 使用 `QueryType` 和 `QueryParameter`
- 不存储变量值

**execution.rs - ExecutionContext**
- 执行级别的状态管理
- 使用 `FieldValue` 存储变量和记录字段
- 管理中间结果和执行统计

**query_execution.rs - QueryExecutionContext**
- 查询执行期间的版本控制和变量管理
- 使用 `Value` 和 `Result` 管理变量历史
- 支持多版本结果

## 重构方案

### 目标
统一 context 模块的类型系统，消除重复，支持图数据库特性。

### 方案概述
1. 将 `FieldValue` 从 `types::query` 迁移到 `context::value`
2. 统一使用 `FieldValue` 作为 context 模块的值类型
3. 移除 `QueryParameter`，使用 `FieldValue::Scalar`
4. 修改 `QueryExecutionContext` 使用 `FieldValue`

### 详细步骤

#### 步骤1：创建 context/value.rs
```rust
//! 上下文值类型定义
//!
//! 定义在上下文中使用的值类型，支持图数据库特有的数据结构

use crate::core::Value;
use crate::core::vertex_edge_path::{Edge, Path, Vertex};
use serde::{Deserialize, Serialize};

/// 上下文字段值
///
/// 支持图数据库特有的数据结构，用于上下文中的变量存储和值传递
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum FieldValue {
    Scalar(Value),
    List(Vec<FieldValue>),
    Map(Vec<(String, FieldValue)>),
    Vertex(Vertex),
    Edge(Edge),
    Path(Path),
}
```

#### 步骤2：更新 context/mod.rs
```rust
pub mod value;
pub use value::FieldValue;
```

#### 步骤3：修改 QueryExecutionContext
- 将 `HashMap<String, Vec<Result>>` 改为 `HashMap<String, Vec<FieldValue>>`
- 更新所有相关方法

#### 步骤4：移除 QueryParameter
- 在 `query.rs` 中移除 `QueryParameter` 枚举
- 将 `QueryContext::parameters` 改为 `HashMap<String, FieldValue>`

#### 步骤5：更新所有引用
- 更新 `execution.rs` 中的导入
- 更新 `basic_context.rs` 中的导入
- 更新 `expression_evaluator.rs` 中的导入

#### 步骤6：清理 types/query.rs
- 移除 `FieldValue` 定义
- 保留 `QueryType`（可能需要简化）

## 类型使用规范

### QueryContext
- **职责**：查询元数据管理
- **不存储**变量值
- **使用** `QueryType` 标记查询类型

### ExecutionContext
- **职责**：执行状态管理
- **使用** `FieldValue` 存储变量和记录
- **管理**中间结果和执行统计

### QueryExecutionContext
- **职责**：版本控制和变量管理
- **使用** `FieldValue` 管理变量历史
- **支持**多版本结果

## 预期效果

### 优点
1. **统一类型系统**：所有 context 模块使用统一的值类型
2. **消除重复**：移除 `QueryParameter`，统一使用 `FieldValue::Scalar`
3. **支持图结构**：`FieldValue` 支持 Vertex/Edge/Path
4. **职责清晰**：类型定义与使用场景匹配
5. **易于维护**：值类型集中管理

### 风险
1. **破坏性变更**：需要更新所有引用
2. **测试覆盖**：需要更新相关测试
3. **性能影响**：需要评估类型转换开销

## 执行计划

1. 创建 `context/value.rs`
2. 更新 `context/mod.rs`
3. 修改 `QueryExecutionContext`
4. 移除 `QueryParameter`
5. 更新所有引用
6. 运行编译检查
7. 运行测试验证

## 备注

- 保持向后兼容性，逐步迁移
- 优先保证编译通过
- 测试覆盖所有变更
- 文档同步更新
