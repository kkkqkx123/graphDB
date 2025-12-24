# GraphDB Error统一方案

## 一、现状分析

### 1.1 核心层Error定义（`src/core/error.rs`）

核心层已经定义了统一的错误处理系统，包含以下错误类型：

| 错误类型 | 说明 |
|---------|------|
| `DBError` | 统一的数据库错误类型，包含所有子系统的错误 |
| `StorageError` | 存储错误 |
| `QueryError` | 查询错误 |
| `ExpressionError` | 表达式错误 |
| `ExpressionErrorType` | 表达式错误类型枚举 |
| `PlanNodeVisitError` | 计划节点访问错误 |
| `LockError` | 锁操作错误 |

### 1.2 各模块独立定义的Error

#### Query模块 - 高度冗余

| 模块 | 错误类型 | 使用频率 | 重复情况 |
|------|---------|---------|---------|
| `query/planner/planner.rs` | `PlannerError` | 295次 | 与`DBError::Query`重复 |
| `query/optimizer/optimizer.rs` | `OptimizerError` | 110次 | 与`DBError::Query`重复 |
| `query/executor/cypher/mod.rs` | `CypherExecutorError` | 33次 | 已有`From`转换到`DBError` |
| `query/visitor/deduce_type_visitor.rs` | `TypeDeductionError` | 42次 | 已有`From`转换到`DBError` |

#### Parser模块 - 存在重复

| 模块 | 错误类型 | 重复情况 |
|------|---------|---------|
| `query/parser/core/error.rs` | `ParseError` | 与`query/parser/cypher/parser.rs`的`ParseError`重复 |
| `query/parser/cypher/parser.rs` | `ParseError` | 与`query/parser/ast/types.rs`的`ParseError`重复 |
| `query/parser/lexer/mod.rs` | `LexError` | 独立定义 |

#### Plan Node模块 - 完全重复

| 模块 | 错误类型 | 重复情况 |
|------|---------|---------|
| `query/planner/plan/core/nodes/plan_node_traits.rs` | `PlanNodeVisitError` | 与`core/error.rs`完全重复 |
| `query/planner/plan/core/nodes/management_node_traits.rs` | `ManagementNodeVisitError` | 与`PlanNodeVisitError`结构相同 |

#### 其他模块 - 功能独立

| 模块 | 错误类型 | 使用频率 | 是否需要统一 |
|------|---------|---------|-------------|
| `storage/storage_error.rs` | `StorageError` | - | 已有`From`转换到`DBError` |
| `graph/index.rs` | `IndexError` | 17次 | 建议统一 |
| `graph/transaction.rs` | `TransactionError` | 15次 | 建议统一 |
| `common/fs.rs` | `FsError` | 30次 | 建议统一 |
| `query/context/validate/schema.rs` | `SchemaValidationError` | 24次 | 建议统一 |

## 二、统一方案

### 2.1 高优先级（立即执行）

#### 1. 删除重复的 `PlanNodeVisitError`

**文件**: `src/query/planner/plan/core/nodes/plan_node_traits.rs`

**操作**:
- 删除 `PlanNodeVisitError` 枚举定义
- 修改所有引用为 `crate::core::error::PlanNodeVisitError`

#### 2. 删除重复的 `ManagementNodeVisitError`

**文件**: `src/query/planner/plan/core/nodes/management_node_traits.rs`

**操作**:
- 删除 `ManagementNodeVisitError` 枚举定义
- 修改所有引用为 `crate::core::error::PlanNodeVisitError`

#### 3. 统一 Parser 模块的 `ParseError`

**保留**: `src/query/parser/core/error.rs` 中的 `ParseError`

**删除**:
- `src/query/parser/cypher/parser.rs` 中的 `ParseError`
- `src/query/parser/ast/types.rs` 中的 `ParseError`

**操作**:
- 修改所有引用为 `crate::query::parser::core::error::ParseError`

### 2.2 中优先级（后续执行）

#### 4. 扩展 `DBError` 添加新的错误变体

**文件**: `src/core/error.rs`

**新增变体**:
```rust
pub enum DBError {
    // 现有错误
    Storage(#[from] StorageError),
    Query(#[from] QueryError),
    Expression(#[from] ExpressionError),
    Plan(#[from] PlanNodeVisitError),
    Lock(#[from] LockError),

    // 新增错误
    Index(IndexError),
    Transaction(TransactionError),
    FileSystem(FsError),
    Schema(SchemaValidationError),

    // 通用错误
    Validation(String),
    TypeDeduction(String),
    Io(#[from] std::io::Error),
    Serialization(String),
    Internal(String),
}
```

#### 5. 为新错误类型实现 `From` 转换

```rust
impl From<crate::graph::IndexError> for DBError {
    fn from(err: crate::graph::IndexError) -> Self {
        DBError::Index(err)
    }
}

impl From<crate::graph::TransactionError> for DBError {
    fn from(err: crate::graph::TransactionError) -> Self {
        DBError::Transaction(err)
    }
}

impl From<crate::common::fs::FsError> for DBError {
    fn from(err: crate::common::fs::FsError) -> Self {
        DBError::FileSystem(err)
    }
}

impl From<crate::query::context::validate::schema::SchemaValidationError> for DBError {
    fn from(err: crate::query::context::validate::schema::SchemaValidationError) -> Self {
        DBError::Schema(err)
    }
}
```

### 2.3 低优先级（可选）

#### 6. 保留向后兼容的错误类型

以下错误类型已有 `From` 转换到 `DBError`，保留用于向后兼容：

- `query/validator/validation_interface.rs` 中的 `ValidationError`
- `storage/storage_error.rs` 中的 `StorageError`

## 三、执行计划

### 第一阶段：删除重复定义

1. ✅ 创建方案文档
2. 删除 `plan_node_traits.rs` 中的 `PlanNodeVisitError`
3. 删除 `management_node_traits.rs` 中的 `ManagementNodeVisitError`
4. 统一 Parser 模块的 `ParseError`
5. 运行 `cargo check` 验证修改

### 第二阶段：扩展核心错误

1. 扩展 `DBError` 添加新的错误变体
2. 为新错误类型实现 `From` 转换
3. 运行 `cargo check` 验证修改
4. 运行 `cargo test` 确保测试通过

## 四、预期效果

- 减少代码重复，提高可维护性
- 统一错误处理，便于错误追踪和调试
- 保持向后兼容，不影响现有代码
- 为未来的错误扩展提供清晰的架构

## 五、注意事项

1. 修改后必须运行 `cargo check` 确保编译通过
2. 运行 `cargo test` 确保所有测试通过
3. 修改错误类型时注意保持错误信息的完整性
4. 保留必要的 `From` 实现以确保向后兼容
