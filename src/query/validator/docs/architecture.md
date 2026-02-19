# 验证器架构文档（重构版）

## 概述

本文档描述 GraphDB 查询验证器的新架构设计，采用 trait + 枚举模式替代原有的组合式继承设计。

## 设计原则

1. **类型安全**：优先使用编译时类型检查，减少运行时错误
2. **零成本抽象**：使用泛型和枚举实现静态分发，避免 dyn 的开销
3. **单一职责**：每个验证器只负责一种语句类型的验证
4. **开闭原则**：新增语句类型时无需修改现有代码
5. **统一接口**：所有验证器实现相同的 trait 接口

## 架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                     Validator (enum)                             │
├─────────────────────────────────────────────────────────────────┤
│  Match(MatchValidator)                                          │
│  Go(GoValidator)                                                │
│  FetchVertices(FetchVerticesValidator)                          │
│  FetchEdges(FetchEdgesValidator)                                │
│  Lookup(LookupValidator)                                        │
│  FindPath(FindPathValidator)                                    │
│  GetSubgraph(GetSubgraphValidator)                              │
│  Insert(InsertValidator)                                        │
│  Update(UpdateValidator)                                        │
│  Delete(DeleteValidator)                                        │
│  Create(CreateValidator)                                        │
│  Drop(DropValidator)                                            │
│  Use(UseValidator)                                              │
│  Pipe(PipeValidator)                                            │
│  Yield(YieldValidator)                                          │
│  Unwind(UnwindValidator)                                        │
│  Set(SetValidator)                                              │
│  Sequential(SequentialValidator)                                │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ impl StatementValidator
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              StatementValidator (trait)                          │
├─────────────────────────────────────────────────────────────────┤
│  fn validate(&mut self, ctx: &mut ValidationContext)             │
│      -> Result<(), ValidationError>                              │
│  fn statement_type(&self) -> StatementType                       │
│  fn inputs(&self) -> &[ColumnDef]                                │
│  fn outputs(&self) -> &[ColumnDef]                               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              ValidationContext                                   │
├─────────────────────────────────────────────────────────────────┤
│  统一的验证上下文，包含：                                         │
│  - 当前图空间信息                                                │
│  - Schema 管理器                                                 │
│  - 变量表                                                        │
│  - 错误列表                                                      │
│  - 输入/输出列定义                                               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│              ValidationStrategy (trait)                          │
├─────────────────────────────────────────────────────────────────┤
│  泛型策略接口，避免 dyn 开销                                      │
│  fn validate<C: ValidationContext>(&self, ctx: &mut C)           │
│      -> Result<(), ValidationError>                              │
└─────────────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. StatementValidator Trait

所有语句验证器必须实现的统一接口：

```rust
pub trait StatementValidator {
    /// 执行验证
    fn validate(&mut self, ctx: &mut ValidationContext) -> Result<(), ValidationError>;
    
    /// 获取语句类型
    fn statement_type(&self) -> StatementType;
    
    /// 获取输入列定义
    fn inputs(&self) -> &[ColumnDef];
    
    /// 获取输出列定义
    fn outputs(&self) -> &[ColumnDef];
    
    /// 添加输入列
    fn add_input(&mut self, col: ColumnDef);
    
    /// 添加输出列
    fn add_output(&mut self, col: ColumnDef);
}
```

### 2. Validator 枚举

包装所有具体验证器类型，实现统一接口：

```rust
pub enum Validator {
    Match(MatchValidator),
    Go(GoValidator),
    // ... 其他变体
}

impl StatementValidator for Validator {
    fn validate(&mut self, ctx: &mut ValidationContext) -> Result<(), ValidationError> {
        match self {
            Validator::Match(v) => v.validate(ctx),
            Validator::Go(v) => v.validate(ctx),
            // ...
        }
    }
    // ...
}
```

### 3. ValidationContext

统一的验证上下文，替代原有的分散式上下文设计：

```rust
pub struct ValidationContext {
    /// 当前图空间
    space: SpaceRef,
    /// Schema 管理器
    schema_manager: Option<Arc<dyn SchemaProvider>>,
    /// 变量表
    variables: HashMap<String, Variable>,
    /// 错误列表
    errors: Vec<ValidationError>,
    /// 输入列定义
    inputs: Vec<ColumnDef>,
    /// 输出列定义
    outputs: Vec<ColumnDef>,
    /// 是否需要选择图空间
    no_space_required: bool,
}
```

### 4. ValidationStrategy Trait

泛型策略接口，用于实现各种验证策略：

```rust
pub trait ValidationStrategy {
    fn validate<C: ValidationContext>(&self, ctx: &mut C) -> Result<(), ValidationError>;
    fn strategy_type(&self) -> ValidationStrategyType;
}
```

## 语句类型枚举

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatementType {
    Match,
    Go,
    FetchVertices,
    FetchEdges,
    Lookup,
    FindPath,
    GetSubgraph,
    Insert,
    Update,
    Delete,
    Create,
    Drop,
    Alter,
    Use,
    Pipe,
    Yield,
    Unwind,
    Set,
    Sequential,
}
```

## 列定义

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
}
```

## 使用示例

### 创建验证器

```rust
// 从 AST 创建验证器
let validator = Validator::from_ast(&stmt)?;

// 或者直接创建具体验证器
let match_validator = MatchValidator::new(match_stmt);
let validator = Validator::Match(match_validator);
```

### 执行验证

```rust
let mut ctx = ValidationContext::new(schema_manager);
let mut validator = Validator::from_ast(&stmt)?;

match validator.validate(&mut ctx) {
    Ok(()) => {
        // 验证成功
        let outputs = validator.outputs();
        // ...
    }
    Err(e) => {
        // 验证失败
        eprintln!("验证错误: {}", e);
    }
}
```

### 使用策略

```rust
// 创建策略集
let strategies = StrategySet::default();

// 在验证器中使用
pub struct MatchValidator {
    stmt: MatchStmt,
    strategies: StrategySet,
}

impl StatementValidator for MatchValidator {
    fn validate(&mut self, ctx: &mut ValidationContext) -> Result<(), ValidationError> {
        // 执行所有策略
        for strategy in &self.strategies {
            strategy.validate(ctx)?;
        }
        // ...
    }
}
```

## 迁移指南

### 从旧架构迁移

1. **具体验证器**：保持 `MatchValidator`, `GoValidator` 等结构体，但实现新的 `StatementValidator` trait
2. **上下文**：将分散的上下文合并到统一的 `ValidationContext`
3. **策略**：将 `Box<dyn ValidationStrategy>` 改为泛型实现
4. **基类**：删除 `base_validator.rs` 中的 `Validator` 结构体，使用新的枚举替代

### 文件结构

```
src/query/validator/
├── mod.rs                    # 模块导出
├── docs/
│   └── architecture.md       # 架构文档
├── core/
│   ├── mod.rs                # 核心模块导出
│   ├── validator.rs          # Validator 枚举和 StatementValidator trait
│   ├── context.rs            # ValidationContext
│   ├── types.rs              # 类型定义（ColumnDef, StatementType 等）
│   └── strategy.rs           # ValidationStrategy trait
├── strategies/               # 策略实现（保持现有）
└── impls/                    # 具体验证器实现（后续迁移）
```

## 性能考虑

1. **静态分发**：使用枚举和泛型实现静态分发，避免虚函数调用开销
2. **栈分配**：小型结构体在栈上分配，减少堆分配
3. **内联优化**：编译器可以内联枚举 match 的分支
4. **缓存友好**：数据布局更紧凑，提高缓存命中率

## 扩展性

新增语句类型的步骤：

1. 在 `StatementType` 枚举中添加新变体
2. 创建新的验证器结构体（如 `NewStmtValidator`）
3. 实现 `StatementValidator` trait
4. 在 `Validator` 枚举中添加对应变体
5. 实现 `From<NewStmt> for Validator`

无需修改任何现有验证器代码。
