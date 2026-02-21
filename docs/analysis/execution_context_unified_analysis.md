# ExecutionContext 统一重构方案

**日期**: 2026 年 2 月 21 日  
**状态**: ✅ 已执行

---

## 一、问题背景

项目中存在两个功能重复的上下文结构：

1. **`ExecutionContext`** - `src/query/executor/base/execution_context.rs`
2. **`QueryExecutionContext`** - `src/query/context/execution/query_execution.rs`

两者都提供变量管理功能（`set_value`/`get_value`），造成代码冗余和维护负担。

---

## 二、分析结论

### 2.1 功能对比

| 功能 | ExecutionContext | QueryExecutionContext |
|------|-----------------|---------------------|
| 变量管理 | ✅ `set_variable`/`get_variable` | ✅ `set_value`/`get_value` |
| 中间结果 | ✅ `set_result`/`get_result` | ❌ 无 |
| 依赖复杂度 | 低（仅依赖同模块） | 高（依赖 plan 模块） |

### 2.2 核心问题

**`QueryExecutionContext` 依赖了 `plan` 模块**：

```rust
// query_execution.rs
use crate::query::planner::plan::ExecutionPlan;  // ❌ 循环依赖风险
```

这导致：
- 无法将 `QueryExecutionContext` 放在更基础的模块
- 违反了分层架构原则
- 增加了编译依赖

### 2.3 使用范围

| 上下文 | 使用范围 | 实际使用 |
|--------|---------|---------|
| `ExecutionContext` | executor 内部 | ✅ 大量使用 |
| `QueryExecutionContext` | 跨模块 | ⚠️ 仅通过 `QueryContext.ectx()` 间接访问 |

**关键发现**：`QueryExecutionContext` 的实际使用非常有限，主要是：
1. `QueryContext` 内部持有
2. 通过 `qctx.ectx()` 访问变量
3. 测试代码

---

## 三、重构方案

### 3.1 决策

**删除 `QueryExecutionContext`，统一使用 `ExecutionContext`**

### 3.2 理由

1. **功能覆盖**: `ExecutionContext` 完全覆盖 `QueryExecutionContext` 的功能
2. **依赖简单**: `ExecutionContext` 无外部模块依赖
3. **代码简化**: 减少重复代码，降低维护成本
4. **架构清晰**: 单一上下文结构，避免混淆

### 3.3 修改范围

| 文件 | 修改内容 |
|------|---------|
| `query_execution.rs` | 删除 `QueryExecutionContext`，改用 `ExecutionContext` |
| `query_execution.rs` | 更新 `QueryContext` 字段类型 |
| `mod.rs` | 移除 `QueryExecutionContext` 导出 |
| `query_expression_context.rs` | 更新导入路径 |
| 测试代码 | 更新类型引用 |

---

## 四、执行步骤

### 步骤 1: 修改 `query_execution.rs`

```rust
// 删除原 QueryExecutionContext 定义
// 导入 ExecutionContext
use crate::query::executor::base::ExecutionContext;

// 修改 QueryContext 字段
pub struct QueryContext {
    ectx: ExecutionContext,  // 原：QueryExecutionContext
    // ...
}
```

### 步骤 2: 更新方法签名

```rust
// 原方法
pub fn ectx(&self) -> &QueryExecutionContext

// 新方法
pub fn ectx(&self) -> &ExecutionContext
```

### 步骤 3: 更新导入

```rust
// query_expression_context.rs
// 原：通过 qctx.ectx() 访问
// 新：保持不变（接口兼容）
```

### 步骤 4: 清理文档

- 删除 `__analysis__` 目录下的过时分析文档
- 更新 `README.md`

---

## 五、修改后结构

```
src/query/
├── context/
│   └── execution/
│       └── query_execution.rs
│           ├── QueryContext
│           └── ectx: ExecutionContext  ← 统一使用
│
└── executor/
    └── base/
        ├── execution_context.rs  ← 唯一上下文定义
        │   └── ExecutionContext
        └── mod.rs
            └── pub use ExecutionContext
```

---

## 六、依赖关系图（修改后）

```
src/query/
├── context/execution/query_execution.rs
│   └── QueryContext {
│       └── ectx: ExecutionContext  ← 来自 executor/base
│   }
│
└── executor/base/
    └── execution_context.rs
        └── ExecutionContext  ← 唯一上下文定义
```

**依赖方向**: `context` → `executor`（允许，符合查询执行流程）

---

## 七、验证清单

- [ ] 编译通过
- [ ] 测试通过
- [ ] 无循环依赖
- [ ] 文档更新

---

## 八、修改记录

| 日期 | 修改内容 | 状态 |
|------|---------|------|
| 2026-02-21 | 删除 `QueryExecutionContext` | ✅ |
| 2026-02-21 | 更新 `QueryContext` 使用 `ExecutionContext` | ✅ |
| 2026-02-21 | 更新导入和方法签名 | ✅ |
| 2026-02-21 | 清理分析文档 | ✅ |

---

## 附录：核心代码位置

- **唯一上下文**: `src/query/executor/base/execution_context.rs`
- **QueryContext**: `src/query/context/execution/query_execution.rs`
- **执行器基类**: `src/query/executor/base/executor_base.rs`
