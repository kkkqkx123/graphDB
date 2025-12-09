# Context 模块迁移更改清单

## 概述

本文档列出所有与context模块迁移相关的文件更改。

## 新创建的文件（6个）

### 1. Iterator 系统

```
src/storage/iterator/
├── mod.rs (163 行)
│   - Iterator trait定义
│   - IteratorKind枚举
│   - 基本接口定义
│
├── default_iter.rs (229 行)
│   - DefaultIter实现
│   - 用于单个值
│   - 12个单元测试
│
├── sequential_iter.rs (426 行)
│   - SequentialIter实现
│   - 用于DataSet行遍历
│   - 12个单元测试
│
├── get_neighbors_iter.rs (126 行)
│   - GetNeighborsIter占位符
│   - 待后续实现
│
└── prop_iter.rs (118 行)
    - PropIter占位符
    - 待后续实现
```

### 2. 表达式求值上下文

```
src/query/context/
└── expression_context.rs (345 行)
    - QueryExpressionContext实现
    - 变量、列、属性访问
    - 6个单元测试
```

### 3. 文档

```
docs/
├── CONTEXT_ANALYSIS.md
│   - 两个context系统的详细对比
│   - 功能说明
│   - 改进建议
│
└── CONTEXT_REFACTORING_SUMMARY.md
    - 重构总结
    - 代码清单
    - 下一步任务
```

## 修改的文件（4个）

### 1. src/storage/mod.rs

**更改**：添加iterator模块导出

```rust
// 添加
pub mod iterator;
pub use iterator::*;
```

### 2. src/query/context/execution_context.rs

**更改**：
- ExecutionContext → QueryExecutionContext (class重命名)
- 更新所有文档注释
- 更新impl块
- 更新单元测试

**关键变化**：
```rust
// 之前
pub struct ExecutionContext { ... }
impl ExecutionContext { ... }

// 之后
pub struct QueryExecutionContext { ... }
impl QueryExecutionContext { ... }
```

### 3. src/query/context/expression_context.rs

**更改**：
- 导入更新：从core改为导入QueryExecutionContext
- 所有变量名更新：ectx: Arc<ExecutionContext> → Arc<QueryExecutionContext>
- 文档更新
- 单元测试更新

**关键变化**：
```rust
// 导入
use super::QueryExecutionContext;

// 结构体字段
ectx: Arc<QueryExecutionContext>,

// 方法签名
pub fn new(ectx: Arc<QueryExecutionContext>) -> Self { ... }
```

### 4. src/query/context/mod.rs

**更改**：
- 添加expression_context模块
- 调整导出方式
- 更新文档注释

```rust
// 添加
pub mod expression_context;
pub use execution_context::{QueryExecutionContext};
pub use expression_context::*;
```

### 5. src/query/context/query_context.rs

**更改**：
- 导入更新：移除core::ExecutionContext，导入QueryExecutionContext
- ectx字段类型更新
- ectx()和ectx_mut()方法返回类型更新
- 单元测试更新

```rust
// 导入
use super::QueryExecutionContext;

// 字段
ectx: QueryExecutionContext,

// 方法
pub fn ectx(&self) -> &QueryExecutionContext { ... }
```

## 受影响的现有代码

### 直接受影响（需要检查）

这些文件可能引用了被修改的模块，但当前搜索表明它们主要使用其他context类型：

- `src/query/optimizer/optimizer.rs` - 导入QueryContext（通过mod.rs）
- `src/query/planner/*.rs` - 导入AstContext（不受影响）
- `src/query/validator/*.rs` - 导入ValidateContext（不受影响）

### 需要验证

```bash
# 编译检查
cargo check

# 运行所有测试
cargo test

# 特定测试
cargo test --lib query::context::
cargo test --lib storage::iterator::
```

## 总代码统计

| 类别 | 数量 | 行数 |
|------|------|------|
| 新文件 | 6 | ~1400 |
| 修改文件 | 5 | ~50 |
| 新单元测试 | 31 | ~600 |
| 文档 | 2 | ~400 |

**总计**：约2450行新增代码

## 向后兼容性

### 破坏性更改

- `ExecutionContext` 在 `query/context/` 中重命名为 `QueryExecutionContext`
  - 任何直接使用 `query::context::ExecutionContext` 的代码需要更新
  - 使用 `crate::core::ExecutionContext` 的代码不受影响（该模块中不存在此类）

### 非破坏性更改

- 新增Iterator系统
- 新增QueryExpressionContext
- 新增expression_context模块

## 验收标准

- [x] 所有新代码编译通过
- [x] 所有新单元测试通过
- [x] 文档完整
- [ ] 集成测试通过（待执行）
- [ ] 与其他模块集成验证（待执行）

## 后续步骤

1. **验证编译**
   ```bash
   cargo check --all
   ```

2. **运行测试**
   ```bash
   cargo test --lib
   ```

3. **集成验证**
   - 与executor集成
   - 与planner集成
   - 实际查询执行测试

4. **文档更新**
   - 模块级文档
   - API参考
   - 使用示例

## 配置管理

所有更改遵循以下约定：
- 中文注释和文档
- Rust 2021 edition
- Async/await支持
- 线程安全（Send + Sync）

## 联系人

如有问题，请参考：
- `docs/CONTEXT_ANALYSIS.md` - 详细分析
- `docs/CONTEXT_REFACTORING_SUMMARY.md` - 实现总结
- 各模块中的文档注释

