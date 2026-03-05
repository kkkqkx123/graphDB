# 验证器迁移方案 - 从 Arc<Stmt> 到 Arc<Ast>

## 目标

将验证器体系从使用 `Arc<Stmt>` 迁移到 `Arc<Ast>`，实现统一的 AST 传递方式。

## 背景

当前状态：
- `ValidatedStatement` 已使用 `Arc<Ast>`
- `PlannerEnum` 已支持 `from_ast()`
- 但验证器 trait 和枚举仍使用 `Arc<Stmt>`

问题：
- 验证器无法直接访问表达式上下文
- 需要在多个地方分别传递 `Stmt` 和 `ExpressionAnalysisContext`
- 不统一的 AST 传递方式

## 迁移阶段

### 阶段 1: 更新 validator_trait.rs 使用 Arc<Ast>

**目标**: 修改 `StatementValidator` trait 接口

**修改文件**:
- `src/query/validator/validator_trait.rs`

**变更内容**:
```rust
// 修改前
pub trait StatementValidator {
    fn validate(
        &mut self,
        _stmt: Arc<Stmt>,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError>;

    // ... 其他方法
}

// 修改后
pub trait StatementValidator {
    fn validate(
        &mut self,
        _ast: Arc<Ast>,
        _qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError>;

    // ... 其他方法
}
```

**影响**:
- 所有实现 `StatementValidator` 的具体验证器需要更新 `validate` 方法签名
- 验证器内部通过 `ast.stmt` 访问语句
- 验证器可以通过 `ast.expr_context` 访问表达式上下文

---

### 阶段 2: 更新 validator_enum.rs 使用 Arc<Ast>

**目标**: 修改 `Validator` 枚举的 `validate` 和 `create_from_stmt` 方法

**修改文件**:
- `src/query/validator/validator_enum.rs`

**变更内容**:
```rust
// 修改前
impl Validator {
    pub fn validate(&mut self, stmt: Arc<Stmt>, qctx: Arc<QueryContext>) -> ValidationResult {
        match self {
            Validator::Match(v) => v.validate(stmt, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            // ...
        }
    }

    pub fn create_from_stmt(stmt: &Stmt) -> Option<Validator> {
        let stmt_type = Self::infer_statement_type(stmt);
        Some(Self::create(stmt_type))
    }
}

// 修改后
impl Validator {
    pub fn validate(&mut self, ast: Arc<Ast>, qctx: Arc<QueryContext>) -> ValidationResult {
        match self {
            Validator::Match(v) => v.validate(ast, qctx).unwrap_or_else(|e| ValidationResult::failure(vec![e])),
            // ...
        }
    }

    pub fn create_from_ast(ast: &Arc<Ast>) -> Option<Validator> {
        let stmt_type = Self::infer_statement_type(&ast.stmt);
        Some(Self::create(stmt_type))
    }
}
```

**影响**:
- `infer_statement_type` 方法保持不变，接收 `&Stmt`
- 调用方需要传递 `Arc<Ast>` 替代 `Arc<Stmt>`

---

### 阶段 3: 更新所有验证器实现

**目标**: 更新所有实现 `StatementValidator` 的具体验证器

**修改文件**:
- `src/query/validator/statements/*.rs`
- `src/query/validator/clauses/*.rs`
- `src/query/validator/ddl/*.rs`
- `src/query/validator/dml/*.rs`
- `src/query/validator/utility/*.rs`

**变更内容**:
```rust
// 修改前
impl StatementValidator for MatchValidator {
    fn validate(
        &mut self,
        stmt: Arc<Stmt>,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let match_stmt = match stmt.as_ref() {
            Stmt::Match(s) => s,
            _ => return Err(ValidationError::InvalidStatement("Expected Match statement".to_string())),
        };
        // ...
    }
}

// 修改后
impl StatementValidator for MatchValidator {
    fn validate(
        &mut self,
        ast: Arc<Ast>,
        qctx: Arc<QueryContext>,
    ) -> Result<ValidationResult, ValidationError> {
        let match_stmt = match ast.stmt.as_ref() {
            Stmt::Match(s) => s,
            _ => return Err(ValidationError::InvalidStatement("Expected Match statement".to_string())),
        };
        // 可以访问 ast.expr_context
        // ...
    }
}
```

**影响**:
- 所有验证器的 `validate` 方法签名需要修改
- 内部通过 `ast.stmt` 访问语句
- 可以通过 `ast.expr_context` 访问表达式上下文

---

### 阶段 4: 更新调用方代码

**目标**: 更新所有调用验证器的地方

**修改文件**:
- `src/query/query_pipeline_manager.rs`
- 其他使用 `Validator::create_from_stmt()` 的地方

**变更内容**:
```rust
// 修改前
let mut validator = Validator::create_from_stmt(&stmt)
    .ok_or_else(|| DBError::from(QueryError::InvalidQuery("不支持的语句类型".to_string())))?;

let validation_result = validator.validate(Arc::new(stmt), query_context);

// 修改后
let mut validator = Validator::create_from_ast(&ast)
    .ok_or_else(|| DBError::from(QueryError::InvalidQuery("不支持的语句类型".to_string())))?;

let validation_result = validator.validate(ast.clone(), query_context);
```

**影响**:
- `QueryPipelineManager` 中的验证调用需要适配
- 其他调用 `Validator::create_from_stmt()` 的地方需要改为 `create_from_ast()`

---

## 设计决策

### 为什么 `Stmt` 不需要改进？

1. **职责分离**: `Stmt` 专注于语句结构，`Ast` 负责包装和上下文
2. **简洁性**: `Stmt` 保持简单，不包含额外的元数据
3. **灵活性**: `Ast` 可以在不修改 `Stmt` 的情况下扩展功能

### 迁移的好处

1. **统一接口**: 所有阶段都使用 `Arc<Ast>` 传递 AST
2. **访问上下文**: 验证器可以直接访问表达式上下文
3. **减少传递**: 避免在多个地方分别传递 `Stmt` 和 `ExpressionAnalysisContext`
4. **类型安全**: 编译时确保 AST 和上下文的一致性

---

## 实施顺序

1. **阶段 1**: 修改 `validator_trait.rs` - 基础接口变更
2. **阶段 2**: 修改 `validator_enum.rs` - 枚举适配
3. **阶段 3**: 更新所有验证器实现 - 批量修改
4. **阶段 4**: 更新调用方代码 - 完成迁移

## 兼容性说明

- **不添加向后兼容代码**: 直接修改接口，不保留旧方法
- **编译错误作为进度参考**: 每个阶段后检查编译错误，了解剩余工作量
- **分阶段提交**: 每个阶段独立提交，便于回滚

## 预期收益

1. **代码一致性**: 统一使用 `Arc<Ast>` 传递 AST
2. **功能增强**: 验证器可以访问表达式上下文
3. **简化调用**: 减少参数传递的复杂度
4. **类型安全**: 编译时检查 AST 和上下文的一致性
