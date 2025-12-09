# Context 模块分析：services/context.rs vs query/context/

## 概述

项目中存在两个不同功能的context模块，容易造成混淆。

## 模块对比

### 1. `src/services/context.rs` - 全局应用上下文

**职责**：应用级别的全局上下文
- **GraphContext**: 整个数据库应用的上下文（包含存储、配置、会话）
- **ExecutionContext**: 单个操作/查询的执行跟踪（超时、时间、顶点/边/路径状态）
- **Metrics**: 数据库操作统计（创建/读取/删除计数）

**用途**：
```rust
// 服务层使用
pub async fn execute_query(ctx: &GraphContext<S>) -> Result<()> {
    ctx.increment_queries_executed();
    // 执行查询...
    ctx.add_execution_time(elapsed_ms);
}
```

**特点**：
- 持有存储引擎、配置等资源
- 跟踪运行时指标和统计
- 处理会话变量
- 超时管理

---

### 2. `src/query/context/` - 查询处理上下文（新建）

**职责**：查询处理流程中的各种上下文

#### a. `execution_context.rs` - 查询执行上下文
- **职责**：管理查询执行期间的变量值和版本历史
- **特点**：
  - 多版本变量支持
  - 版本历史追踪
  - 针对查询处理流程

```rust
// 查询执行中使用
let ectx = ExecutionContext::new();
ectx.set_value("x", Value::Int(42))?;
let prev = ectx.get_versioned_value("x", -1)?;  // 前一版本
```

#### b. `expression_context.rs` - 表达式求值上下文（新建）
- **职责**：为表达式提供运行时上下文
- **特点**：
  - 变量访问（来自ExecutionContext）
  - 列访问（来自迭代器）
  - 属性访问（标签、边属性）
  - 表达式内部变量

```rust
// 表达式求值时使用
let qctx = QueryExpressionContext::new(ectx);
let val = qctx.get_column("name")?;  // 从迭代器获取
```

#### c. `query_context.rs` - 顶级查询上下文
- **职责**：整个查询请求的上下文
- **特点**：整合ExecutionContext、ValidateContext、SymbolTable等

#### d. `validate_context.rs` - 验证上下文
- **职责**：查询验证/语义分析阶段的上下文
- **特点**：管理空间、变量定义、Schema等

---

## 是否多余？

### 结论：**不多余，功能不重叠**

`src/services/context.rs` 和 `src/query/context/` 是完全不同的概念：

| 方面 | services/context.rs | query/context/ |
|------|-------------------|-----------------|
| **层级** | 应用层 | 查询处理层 |
| **生命周期** | 长期（会话级） | 短期（查询级） |
| **职责** | 全局资源、配置、统计 | 查询执行中的变量和上下文 |
| **使用方** | 服务接口、API处理 | 查询执行器、表达式求值 |
| **持有数据** | 存储引擎、配置、指标 | 变量值、版本历史、符号表 |
| **时间范围** | 数据库运行整个生命周期 | 单个查询执行过程 |

---

## 使用场景

### services/context.rs 使用场景

```rust
// API服务处理
async fn handle_query_request(
    graph_ctx: &GraphContext<S>,
    query: String
) -> Result<Response> {
    graph_ctx.increment_queries_executed();
    
    let start = Instant::now();
    // 解析、规划、执行...
    let elapsed = start.elapsed().as_millis() as u64;
    
    graph_ctx.add_execution_time(elapsed);
    Ok(response)
}
```

### query/context/ 使用场景

```rust
// 查询执行器
fn execute_filter(
    ectx: &ExecutionContext,      // 查询执行上下文
    iter: Box<dyn Iterator>,      // 迭代器
    condition: &Expr
) -> Result<()> {
    let qectx = QueryExpressionContext::new(Arc::new(ectx.clone()))
        .with_iterator(iter);
    
    let result = evaluate_expr(condition, &qectx)?;
    Ok(())
}
```

---

## 改进建议

### 1. 避免命名混淆

在 `query/context/execution_context.rs` 中的 `ExecutionContext` 与 `services/context.rs` 中的 `ExecutionContext` 名字相同但用途不同。

**建议**：
```rust
// 选项A：重命名query/context中的类
pub struct QueryExecutionContext { ... }

// 选项B：使用完整路径区分
use query::context::ExecutionContext as QueryExecutionContext;
use services::context::ExecutionContext as AppExecutionContext;
```

### 2. 文档区分

添加清晰的说明：
- `services/context.rs`: "应用全局上下文"
- `query/context/`: "查询处理上下文"

### 3. 模块组织

在lib.rs中明确导出，避免混淆：

```rust
// 应用级上下文
pub use services::context::{GraphContext, Metrics};

// 查询级上下文
pub use query::context::{
    ExecutionContext as QueryExecutionContext,
    QueryExpressionContext,
};
```

---

## 关键区别总结

| 特性 | services/context.rs | query/context/ |
|------|-------------------|-----------------|
| 存储引擎 | ✓ 持有 | ✗ 不持有 |
| 配置信息 | ✓ 持有 | ✗ 不持有 |
| 性能指标 | ✓ 追踪 | ✗ 不追踪 |
| 变量多版本 | ✗ 不支持 | ✓ 支持 |
| 表达式求值 | ✗ 不支持 | ✓ 支持 |
| 迭代器集成 | ✗ 不支持 | ✓ 支持 |

---

## 建议行动

1. **保留两个模块**：都是必要的
2. **重命名以避免混淆**：
   - `services::context::ExecutionContext` → `services::context::AppExecutionContext`
   或
   - `query::context::execution_context::ExecutionContext` → `query::context::execution_context::QueryExecutionContext`
3. **更新导出**：在lib.rs明确标注用途
4. **文档注释**：在两个模块顶部添加清晰的说明

