# 验证器迁移指南

## 概述

本文档描述了将验证器从使用 `AstContext` 迁移到使用 `Arc<QueryContext>` 和 `&Stmt` 的详细步骤。

## 已完成的工作

以下验证器已经完成迁移：

1. ✅ `use_validator.rs` - USE 语句验证器
2. ✅ `match_validator.rs` - MATCH 语句验证器
3. ✅ `go_validator.rs` - GO 语句验证器
4. ✅ `lookup_validator.rs` - LOOKUP 语句验证器
5. ✅ `fetch_vertices_validator.rs` - FETCH VERTICES 语句验证器
6. ✅ `insert_vertices_validator.rs` - INSERT VERTICES 语句验证器

## 待迁移的验证器列表

还有 **27 个验证器**需要迁移：

### DML 验证器
- [ ] `insert_edges_validator.rs` - INSERT EDGES 语句验证器
- [ ] `update_validator.rs` - UPDATE 语句验证器
- [ ] `delete_validator.rs` - DELETE 语句验证器

### 查询验证器
- [ ] `fetch_edges_validator.rs` - FETCH EDGES 语句验证器
- [ ] `find_path_validator.rs` - FIND PATH 语句验证器
- [ ] `get_subgraph_validator.rs` - GET SUBGRAPH 语句验证器
- [ ] `group_by_validator.rs` - GROUP BY 子句验证器
- [ ] `limit_validator.rs` - LIMIT 子句验证器
- [ ] `order_by_validator.rs` - ORDER BY 子句验证器
- [ ] `pipe_validator.rs` - PIPE 操作符验证器
- [ ] `query_validator.rs` - 通用查询验证器
- [ ] `return_validator.rs` - RETURN 子句验证器
- [ ] `sequential_validator.rs` - 顺序执行验证器
- [ ] `set_operation_validator.rs` - 集合操作验证器（UNION/INTERSECT/MINUS）
- [ ] `unwind_validator.rs` - UNWIND 子句验证器
- [ ] `with_validator.rs` - WITH 子句验证器
- [ ] `yield_validator.rs` - YIELD 子句验证器

### DDL 验证器
- [ ] `create_validator.rs` - CREATE 语句验证器
- [ ] `drop_validator.rs` - DROP 语句验证器
- [ ] `alter_validator.rs` - ALTER 语句验证器

### 管理验证器
- [ ] `admin_validator.rs` - 管理命令验证器
- [ ] `acl_validator.rs` - 访问控制验证器
- [ ] `assignment_validator.rs` - 赋值验证器
- [ ] `explain_validator.rs` - EXPLAIN 语句验证器
- [ ] `merge_validator.rs` - MERGE 语句验证器
- [ ] `remove_validator.rs` - REMOVE 语句验证器
- [ ] `set_validator.rs` - SET 语句验证器
- [ ] `update_config_validator.rs` - 配置更新验证器

## 迁移步骤

对于每个验证器文件，按照以下步骤进行迁移：

### 步骤 1: 更新导入语句

**旧代码：**
```rust
use crate::query::context::ast::AstContext;
```

**新代码：**
```rust
use std::sync::Arc;
use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
```

### 步骤 2: 添加文档注释

在文件头部添加重构说明：
```rust
//!
//! # 重构变更
//! - 使用 Arc<QueryContext> 替代 &mut AstContext
//! - validate 方法接收 &Stmt 和 Arc<QueryContext>
```

### 步骤 3: 更新 validate 方法签名

**旧代码：**
```rust
impl StatementValidator for XxxValidator {
    fn validate(&mut self, ast: &mut AstContext) -> Result<ValidationResult, ValidationError> {
```

**新代码：**
```rust
/// 实现 StatementValidator trait
///
/// # 重构变更
/// - validate 方法接收 &Stmt 和 Arc<QueryContext> 替代 &mut AstContext
impl StatementValidator for XxxValidator {
    fn validate(
        &mut self,
        stmt: &Stmt,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
```

### 步骤 4: 更新空间检查逻辑

**旧代码：**
```rust
// 1. 检查是否需要空间
let query_context = ast.query_context();
if !self.is_global_statement() && query_context.is_none() {
    return Err(ValidationError::new(
        "未选择图空间，请先执行 USE <space>".to_string(),
        ValidationErrorType::SemanticError,
    ));
}
```

**新代码：**
```rust
// 1. 检查是否需要空间
if !self.is_global_statement() && qctx.space_info().is_none() {
    return Err(ValidationError::new(
        "未选择图空间，请先执行 USE <space>".to_string(),
        ValidationErrorType::SemanticError,
    ));
}
```

### 步骤 5: 更新语句获取逻辑

**旧代码：**
```rust
// 2. 获取 XXX 语句
let stmt = ast.sentence()
    .ok_or_else(|| ValidationError::new(
        "No statement found in AST context".to_string(),
        ValidationErrorType::SemanticError,
    ))?;

let xxx_stmt = match stmt {
    Stmt::Xxx(xxx_stmt) => xxx_stmt,
    _ => {
        return Err(ValidationError::new(
            "Expected XXX statement".to_string(),
            ValidationErrorType::SemanticError,
        ));
    }
};
```

**新代码：**
```rust
// 2. 获取 XXX 语句
let xxx_stmt = match stmt {
    Stmt::Xxx(xxx_stmt) => xxx_stmt,
    _ => {
        return Err(ValidationError::new(
            "Expected XXX statement".to_string(),
            ValidationErrorType::SemanticError,
        ));
    }
};
```

### 步骤 6: 更新 space_id 获取逻辑

**旧代码：**
```rust
// 获取 space_id
let space_id = ast.space().space_id.map(|id| id as u64).unwrap_or(0);
```

**新代码：**
```rust
// 获取 space_id
let space_id = qctx.space_info()
    .map(|info| info.space_id)
    .unwrap_or(0);
```

### 步骤 7: 更新空间名称获取逻辑（如需要）

**旧代码：**
```rust
let space_name = ast.space().space_name.clone();
```

**新代码：**
```rust
let space_name = qctx.space_info()
    .map(|info| info.space_name.clone())
    .unwrap_or_default();
```

### 步骤 8: 更新测试代码

如果验证器包含测试代码，需要更新测试中的调用方式：

**旧代码：**
```rust
let mut ast = AstContext::default();
ast.set_sentence(Stmt::Xxx(xxx_stmt));
let result = validator.validate(&mut ast);
```

**新代码：**
```rust
let qctx = Arc::new(QueryContext::default());
let result = validator.validate(&Stmt::Xxx(xxx_stmt), qctx);
```

## 迁移检查清单

每个验证器迁移完成后，请检查：

- [ ] 导入语句已更新
- [ ] 文档注释已添加
- [ ] validate 方法签名已更新
- [ ] 空间检查逻辑已更新
- [ ] 语句获取逻辑已更新
- [ ] space_id 获取逻辑已更新
- [ ] 空间名称获取逻辑已更新（如需要）
- [ ] 测试代码已更新（如需要）
- [ ] 编译通过（`cargo check`）

## 迁移完成后需要删除的文件

所有验证器迁移完成后，以下文件和目录应该被删除：

### 1. 删除旧 QueryContext 实现
```
src/query/context/
├── mod.rs          # 删除整个目录
├── ast_context.rs  # 删除
├── symbol/         # 删除整个目录
│   ├── mod.rs
│   ├── symbol_table.rs
│   └── symbol_entry.rs
└── ...             # 其他文件
```

### 2. 删除旧验证器模块（如果存在）
```
src/query/validator/
├── validator_old.rs    # 删除（如果存在）
└── ...                 # 其他旧文件
```

### 3. 删除临时文件
```
src/query/
├── context_old.rs      # 删除（如果存在）
└── ...                 # 其他临时文件
```

## 验证步骤

所有验证器迁移完成后，执行以下验证步骤：

1. **编译检查：**
   ```bash
   cargo check --lib
   ```

2. **运行测试：**
   ```bash
   cargo test --lib
   ```

3. **检查未使用的导入：**
   ```bash
   cargo clippy --lib
   ```

4. **格式化代码：**
   ```bash
   cargo fmt
   ```

## 常见问题

### 问题 1: 如何处理需要修改 AST 的验证器？

**解决方案：** 如果验证器需要修改语句，应该返回一个新的 `Stmt` 而不是修改输入。修改 trait 定义：

```rust
fn validate(
    &mut self,
    stmt: &Stmt,
    qctx: Arc<QueryContext>,
) -> Result<(ValidationResult, Option<Stmt>), ValidationError>;
```

### 问题 2: 如何处理需要访问符号表的验证器？

**解决方案：** 通过 `QueryContext` 访问符号表：

```rust
let sym_table = qctx.sym_table();
```

### 问题 3: 如何处理需要访问 SchemaManager 的验证器？

**解决方案：** 通过 `QueryContext` 访问：

```rust
if let Some(schema_mgr) = qctx.schema_manager() {
    // 使用 schema_mgr
}
```

## 参考示例

完整的迁移示例请参考以下文件：

- `src/query/validator/use_validator.rs` - 简单验证器示例
- `src/query/validator/match_validator.rs` - 复杂验证器示例
- `src/query/validator/go_validator.rs` - 带测试的验证器示例

## 联系信息

如有问题，请参考：
- 重构计划文档：`docs/query_context_implementation_plan.md`
- 架构文档：`docs/query_context_architecture.md`
