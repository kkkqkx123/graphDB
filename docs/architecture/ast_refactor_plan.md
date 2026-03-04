# AST 传递重构方案

## 目标

解决当前 AST 传递流程中存在的所有权混乱、不必要的克隆、职责不清等问题，提高代码的可维护性和性能。

## 当前问题总结

1. **所有权传递混乱**: `stmt` 先被 clone 再被 move，导致不必要的拷贝
2. **ValidatedStatement 设计问题**: 拥有 `Stmt` 所有权但只需要读取
3. **表达式上下文与 AST 分离**: 容易导致不一致
4. **QueryContext 职责过重**: 混合了不同阶段的数据
5. **Planner 重复解析 AST**: `SentenceKind` 对枚举做字符串匹配
6. **缺少不可变性保证**: 没有明确的不可变 AST 约定

## 重构阶段

### 阶段 1: 修改 ParserResult 使用 Arc<Stmt>

**目标**: 让 Parser 返回的 AST 可以被安全共享

**修改文件**:
- `src/query/parser/parser/parser.rs`

**变更内容**:
```rust
// 修改前
pub struct ParserResult {
    pub stmt: Stmt,
    pub expr_context: Arc<ExpressionAnalysisContext>,
}

// 修改后
pub struct ParserResult {
    pub stmt: Arc<Stmt>,
    pub expr_context: Arc<ExpressionAnalysisContext>,
}
```

**影响**:
- `Parser::parse()` 返回的 `stmt` 变为 `Arc<Stmt>`
- 所有使用 `ParserResult.stmt` 的地方需要适配

---

### 阶段 2: 修改 ValidatedStatement 使用 Arc<Stmt>

**目标**: 验证后的语句共享 AST 所有权，避免克隆

**修改文件**:
- `src/query/validator/structs/validation_info.rs`
- `src/query/validator/validator_enum.rs`
- `src/query/validator/validator_trait.rs`
- `src/query/planner/planner.rs`
- `src/query/planner/statements/*.rs`
- `src/query/query_pipeline_manager.rs`

**变更内容**:
```rust
// 修改前
pub struct ValidatedStatement {
    pub stmt: crate::query::parser::ast::Stmt,
    pub validation_info: ValidationInfo,
}

// 修改后
pub struct ValidatedStatement {
    pub stmt: Arc<crate::query::parser::ast::Stmt>,
    pub validation_info: ValidationInfo,
}
```

**验证器 trait 修改**:
```rust
// 修改前
fn validate(&mut self, stmt: Stmt, qctx: Arc<QueryContext>) -> ValidationResult;

// 修改后
fn validate(&mut self, stmt: Arc<Stmt>, qctx: Arc<QueryContext>) -> ValidationResult;
```

**影响**:
- 所有验证器的 `validate` 方法签名需要修改
- 所有规划器的 `transform` 方法需要适配 `Arc<Stmt>`
- `QueryPipelineManager` 中的调用需要适配

---

### 阶段 3: 消除 SentenceKind 字符串匹配

**目标**: 直接使用 `Stmt` 枚举进行模式匹配，消除字符串转换

**修改文件**:
- `src/query/planner/planner.rs`
- `src/query/planner/planner.rs` (PlannerEnum)

**变更内容**:
```rust
// 修改前
impl SentenceKind {
    pub fn from_stmt(stmt: &Stmt) -> Result<Self, PlannerError> {
        match stmt.kind().to_uppercase().as_str() {
            "MATCH" => Ok(SentenceKind::Match),
            "GO" => Ok(SentenceKind::Go),
            // ...
        }
    }
}

// 修改后 - 直接使用模式匹配
impl PlannerEnum {
    pub fn from_stmt(stmt: &Arc<Stmt>) -> Option<Self> {
        match stmt.as_ref() {
            Stmt::Match(_) => Some(PlannerEnum::Match(MatchStatementPlanner::new())),
            Stmt::Go(_) => Some(PlannerEnum::Go(GoPlanner::new())),
            Stmt::Lookup(_) => Some(PlannerEnum::Lookup(LookupPlanner::new())),
            // ...
        }
    }
}
```

**影响**:
- 删除 `SentenceKind` 类型（或标记为废弃）
- 所有使用 `SentenceKind::from_stmt()` 的地方改为使用 `PlannerEnum::from_stmt()`

---

### 阶段 4: 简化 QueryContext

**目标**: 按阶段分离上下文，移除不必要的锁

**修改文件**:
- `src/query/query_context.rs`
- `src/query/query_request_context.rs`
- `src/query/query_pipeline_manager.rs`

**变更内容**:
```rust
// 修改前
pub struct QueryContext {
    request_context: Arc<QueryRequestContext>,
    expr_context: Arc<ExpressionAnalysisContext>,
    validation_info: RwLock<Option<ValidationInfo>>,
    space_info: RwLock<Option<SpaceInfo>>,
    // ...
}

// 修改后 - 按阶段分离
pub struct QueryContext {
    request_context: Arc<QueryRequestContext>,
    // 解析阶段数据
    pub expr_context: Arc<ExpressionAnalysisContext>,
    // 验证阶段数据
    pub validation_info: Option<ValidationInfo>,
    // 空间信息
    pub space_info: Option<SpaceInfo>,
}

impl QueryContext {
    // 使用 &mut self 替代内部可变性
    pub fn set_validation_info(&mut self, info: ValidationInfo) {
        self.validation_info = Some(info);
    }
}
```

**影响**:
- 需要修改 `QueryContext` 的所有使用方
- `QueryPipelineManager` 需要相应调整

---

### 阶段 5: 合并表达式上下文到 AST（可选）

**目标**: 将 `ExpressionAnalysisContext` 嵌入到 `Stmt` 中，避免分离存储

**修改文件**:
- `src/query/parser/ast/stmt.rs`
- `src/query/parser/parser/parser.rs`

**变更内容**:
```rust
// 新增包装类型
pub struct StmtWithContext {
    pub stmt: Stmt,
    pub expr_context: ExpressionAnalysisContext,
}

// 修改 ParserResult
pub struct ParserResult {
    pub ast: Arc<StmtWithContext>,
}

// 提供便捷方法
impl StmtWithContext {
    pub fn stmt(&self) -> &Stmt {
        &self.stmt
    }
    
    pub fn expr_context(&self) -> &ExpressionAnalysisContext {
        &self.expr_context
    }
}
```

**影响**:
- 这是一个较大的破坏性变更
- 所有访问表达式的地方需要调整
- 建议在前几个阶段稳定后再实施

---

## 实施状态

| 阶段 | 状态 | 说明 |
|------|------|------|
| 阶段 1 | ✅ 已完成 | ParserResult 使用 Arc<Stmt> |
| 阶段 2 | ✅ 已完成 | ValidatedStatement 使用 Arc<Stmt> |
| 阶段 3 | ✅ 已完成 | 消除 SentenceKind 字符串匹配 |
| 阶段 4 | ✅ 已完成 | 简化 QueryContext，移除不必要的锁 |
| 阶段 5 | ✅ 已完成 | 合并表达式上下文到 AST |

## 主要变更总结

### 新增类型
- `Ast` 结构体：包含 `Stmt` 和 `ExpressionAnalysisContext`
- `PlannerEnum::from_ast()` 方法：直接从 Ast 创建规划器

### 修改的类型
- `ParserResult`: 使用 `Arc<Ast>` 替代分开的 `stmt` 和 `expr_context`
- `ValidatedStatement`: 使用 `Arc<Ast>` 替代 `Arc<Stmt>`
- `QueryContext`: 移除 `expr_context` 字段，简化可选字段

### 删除的 API
- `QueryContext::with_expr_context()` - 表达式上下文现在在 Ast 中
- `QueryContext::expr_context()` - 通过 `ValidatedStatement` 访问
- `QueryContext::expr_context_clone()` - 通过 `ValidatedStatement` 访问

## 兼容性考虑

- 每个阶段独立提交，便于回滚
- 添加过渡性 API 减少破坏性变更
- 保持公共接口稳定

## 预期收益

1. **性能提升**: 消除不必要的 AST 克隆
2. **内存优化**: 共享 AST 所有权，减少内存占用
3. **代码清晰**: 明确的不可变性约定
4. **维护性**: 简化的上下文管理
5. **类型安全**: 消除字符串匹配，使用枚举模式匹配
