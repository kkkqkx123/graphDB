# Expression 目录设计分析报告

## 一、当前模块结构

```
src/core/types/expression/
├── def.rs              # Expression 枚举定义
├── expression.rs        # ExpressionId, ExpressionMeta
├── context.rs          # ExpressionContext, OptimizationFlags
├── contextual.rs       # ContextualExpression
├── serializable.rs      # SerializableExpression
├── constructors.rs     # Expression 构造方法
├── query.rs            # Expression 查询方法
├── traverse.rs         # Expression 遍历方法
├── display.rs          # Expression 字符串表示
├── type_deduce.rs     # Expression 类型推导
├── utils.rs            # 工具函数（GroupSuite）
└── mod.rs             # 模块导出
```

## 二、设计缺陷分析

### 2.1 职责分散问题

**问题**：`Expression` 的方法被分散到 6 个文件中

| 文件 | 职责 | 方法数 |
|------|------|--------|
| `constructors.rs` | 构造方法 | 15+ |
| `query.rs` | 查询方法 | 10+ |
| `traverse.rs` | 遍历方法 | 20+ |
| `display.rs` | 显示方法 | 15+ |
| `type_deduce.rs` | 类型推导 | 10+ |

**影响**：
- ❌ 难以理解 `Expression` 的完整功能
- ❌ 维护困难：修改一个功能需要查找多个文件
- ❌ 代码导航困难：IDE 的"跳转到定义"功能效果不佳

**示例**：
```rust
// 要理解 Expression 的完整功能，需要阅读 6 个文件
impl Expression {
    // constructors.rs: 15+ 构造方法
    pub fn literal(value: impl Into<Value>) -> Self;
    pub fn variable(name: impl Into<String>) -> Self;
    // ...

    // query.rs: 10+ 查询方法
    pub fn is_constant(&self) -> bool;
    pub fn contains_aggregate(&self) -> bool;
    // ...

    // traverse.rs: 20+ 遍历方法
    pub fn children(&self) -> Vec<&Expression>;
    pub fn children_mut(&mut self) -> Vec<&mut Expression>;
    // ...

    // display.rs: 15+ 显示方法
    pub fn to_expression_string(&self) -> String;
    // ...

    // type_deduce.rs: 10+ 类型推导方法
    pub fn deduce_type(&self) -> DataType;
    // ...
}
```

### 2.2 表达式变体冗余

**问题**：`Expression` 枚举包含 20+ 个变体，存在功能重叠

#### 2.2.1 属性访问变体过多

```rust
// ❌ 问题：4 个属性访问变体，功能重叠

// 1. 通用属性访问
Property {
    object: Box<Expression>,
    property: String,
}

// 2. 标签属性动态访问
LabelTagProperty {
    tag: Box<Expression>,
    property: String,
}

// 3. 标签属性访问
TagProperty {
    tag_name: String,
    property: String,
}

// 4. 边属性访问
EdgeProperty {
    edge_name: String,
    property: String,
}
```

**问题分析**：
- `Property` 可以表示所有属性访问，包括标签属性和边属性
- `LabelTagProperty` 和 `TagProperty` 功能重复
- `EdgeProperty` 可以用 `Property` 表示（`EdgeType.property`）

**影响**：
- ❌ 增加模式匹配的复杂度
- ❌ 类型转换困难（需要在多个变体间转换）
- ❌ 代码冗余（需要在多个变体上实现相同逻辑）

#### 2.2.2 未使用的变体

```rust
// ❌ 问题：部分变体可能未被使用

// 这些变体在代码中出现，但使用频率低
LabelTagProperty { tag, property }
TagProperty { tag_name, property }
EdgeProperty { edge_name, property }
Predicate { func, args }
Reduce { accumulator, initial, variable, source, mapping }
PathBuild(Vec<Expression>)
Parameter(String)
```

**验证**：
```bash
# 在 planner/rewrite 中搜索这些变体的使用
grep -r "LabelTagProperty" src/query/planner/rewrite/
# 结果：0 匹配

grep -r "TagProperty" src/query/planner/rewrite/
# 结果：0 匹配

grep -r "EdgeProperty" src/query/planner/rewrite/
# 结果：0 匹配
```

**结论**：这些变体在 planner 层未被使用，可能是历史遗留代码。

### 2.3 缺少文档说明

**问题**：部分变体缺少清晰的文档说明

```rust
// ❌ 问题：缺少文档说明

LabelTagProperty {
    tag: Box<Expression>,
    property: String,
}

TagProperty {
    tag_name: String,
    property: String,
}

EdgeProperty {
    edge_name: String,
    property: String,
}

Predicate {
    func: String,
    args: Vec<Expression>,
}

Reduce {
    accumulator: String,
    initial: Box<Expression>,
    variable: String,
    source: Box<Expression>,
    mapping: Box<Expression>,
}

PathBuild(Vec<Expression>)

Parameter(String)
```

**影响**：
- ❌ 开发者不知道何时使用这些变体
- ❌ 难以理解变体的语义
- ❌ 容易误用

### 2.4 工具函数位置不当

**问题**：`utils.rs` 包含 `GroupSuite` 和 `extract_group_suite`，与表达式模块职责不符

```rust
// ❌ 问题：utils.rs 包含分组相关逻辑

pub struct GroupSuite {
    pub group_keys: Vec<Expression>,
    pub group_items: Vec<Expression>,
    pub aggregates: Vec<Expression>,
}

pub fn extract_group_suite(expression: &Expression) -> Result<GroupSuite, String>;
```

**问题分析**：
- `GroupSuite` 是 GROUP BY 优化的辅助类型
- `extract_group_suite` 是优化器的工具函数
- 这些功能与表达式类型本身无关

**影响**：
- ❌ 表达式模块职责不清晰
- ❌ 依赖关系混乱（表达式模块依赖优化器逻辑）
- ❌ 难以重用（其他模块难以使用这些工具函数）

**建议位置**：
- `src/query/planner/optimizer/group_by_utils.rs`
- 或 `src/query/planner/rewrite/group_by_utils.rs`

### 2.5 缺少表达式验证

**问题**：没有表达式验证机制

**当前状态**：
```rust
// ❌ 问题：可以创建无效的表达式

// 示例 1：嵌套类型转换
let expr = Expression::cast(
    Expression::cast(
        Expression::literal(42),
        DataType::String,
    ),
    DataType::Int,
);

// 示例 2：无效的聚合函数
let expr = Expression::aggregate(
    AggregateFunction::Count,
    Expression::aggregate(
        AggregateFunction::Sum("x".to_string()),
        Expression::variable("x"),
        false,
    ),
    false,
);

// 示例 3：无效的属性访问
let expr = Expression::property(
    Expression::literal(42),  // 字面量不能有属性
    "name".to_string(),
);
```

**影响**：
- ❌ 运行时错误（而非编译时错误）
- ❌ 难以调试（错误发生在执行阶段）
- ❌ 类型不安全

**建议**：
```rust
impl Expression {
    /// 验证表达式是否有效
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Expression::TypeCast { expression, target_type } => {
                // 检查类型转换是否有效
                let source_type = expression.deduce_type();
                if !is_valid_cast(&source_type, target_type) {
                    return Err(ValidationError::InvalidCast {
                        source: source_type,
                        target: target_type.clone(),
                    });
                }
            }
            Expression::Property { object, .. } => {
                // 检查对象是否可以有属性
                if object.is_literal() {
                    return Err(ValidationError::LiteralHasProperty);
                }
            }
            Expression::Aggregate { arg, .. } => {
                // 检查聚合函数嵌套
                if arg.contains_aggregate() {
                    return Err(ValidationError::NestedAggregate);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

### 2.6 类型推导与上下文脱节

**问题**：`Expression::deduce_type()` 独立于 `ExpressionContext`

**当前状态**：
```rust
// ❌ 问题：类型推导结果无法缓存

impl Expression {
    pub fn deduce_type(&self) -> DataType {
        // 每次都重新计算，无法利用缓存
        match self {
            Expression::Literal(value) => Self::deduce_value_type(value),
            Expression::Binary { op, left, right } => {
                Self::deduce_binary_type(op, left, right)
            }
            // ...
        }
    }
}
```

**影响**：
- ❌ 重复计算（每次调用都重新推导）
- ❌ 无法利用 `ExpressionContext` 的缓存
- ❌ 性能开销（复杂表达式需要递归遍历）

**建议**：
```rust
impl Expression {
    /// 推导表达式类型（使用上下文缓存）
    pub fn deduce_type_with_context(
        &self,
        ctx: &ExpressionContext,
        id: &ExpressionId,
    ) -> DataType {
        // 检查缓存
        if let Some(cached_type) = ctx.get_type(id) {
            return cached_type;
        }

        // 计算类型
        let data_type = self.deduce_type();

        // 缓存结果
        ctx.set_type(id, data_type.clone());

        data_type
    }
}
```

### 2.7 SerializableExpression 冗余

**问题**：`SerializableExpression` 与 `ContextualExpression` 功能重复

**当前状态**：
```rust
// ❌ 问题：两个类型功能重复

// ContextualExpression
pub struct ContextualExpression {
    id: ExpressionId,
    context: Arc<ExpressionContext>,
}

// SerializableExpression
pub struct SerializableExpression {
    pub id: ExpressionId,
    pub expression: Expression,
    pub data_type: Option<DataType>,
    pub constant_value: Option<Value>,
}
```

**功能对比**：

| 功能 | ContextualExpression | SerializableExpression |
|------|---------------------|----------------------|
| 存储表达式 ID | ✅ | ✅ |
| 存储表达式 | ✅（通过 context） | ✅（直接存储） |
| 存储类型 | ✅（通过 context） | ✅（直接存储） |
| 存储常量值 | ✅（通过 context） | ✅（直接存储） |
| 序列化支持 | ❌ | ✅ |

**问题分析**：
- `SerializableExpression` 的所有信息都可以从 `ContextualExpression` 获取
- `SerializableExpression` 只是为了序列化而存在
- 两者功能重复，增加维护成本

**建议**：
```rust
// 方案 1：扩展 ContextualExpression 支持序列化
impl ContextualExpression {
    pub fn to_serializable(&self) -> SerializableExpression {
        SerializableExpression {
            id: self.id().clone(),
            expression: self.expression()
                .map(|meta| meta.inner().clone())
                .unwrap_or_else(|| Expression::Literal(Value::Null)),
            data_type: self.data_type(),
            constant_value: self.constant_value(),
        }
    }
}

// 方案 2：直接使用 serde 序列化 ContextualExpression
#[derive(Serialize, Deserialize)]
pub struct ContextualExpression {
    id: ExpressionId,
    #[serde(skip)]
    context: Arc<ExpressionContext>,
    #[serde(default)]
    cached_expression: Option<Expression>,
    #[serde(default)]
    cached_type: Option<DataType>,
    #[serde(default)]
    cached_constant: Option<Value>,
}
```

### 2.8 构造函数命名不一致

**问题**：部分构造函数命名不够直观

**当前状态**：
```rust
// ❌ 问题：参数顺序不直观

pub fn case(
    test_expr: Option<Expression>,  // 测试表达式
    conditions: Vec<(Expression, Expression)>,  // 条件列表
    default: Option<Expression>,  // 默认值
) -> Self

// 使用示例
let expr = Expression::case(
    Some(test_expr),  // 需要包装在 Some 中
    vec![
        (when_expr, then_expr),  // 元组顺序不直观
    ],
    Some(default_expr),  // 需要包装在 Some 中
);
```

**建议**：
```rust
// ✅ 改进：使用 Builder 模式

pub struct CaseBuilder {
    test_expr: Option<Expression>,
    cases: Vec<(Expression, Expression)>,
    default: Option<Expression>,
}

impl CaseBuilder {
    pub fn new() -> Self {
        Self {
            test_expr: None,
            cases: Vec::new(),
            default: None,
        }
    }

    pub fn test(mut self, expr: Expression) -> Self {
        self.test_expr = Some(expr);
        self
    }

    pub fn when(mut self, when: Expression, then: Expression) -> Self {
        self.cases.push((when, then));
        self
    }

    pub fn default(mut self, expr: Expression) -> Self {
        self.default = Some(expr);
        self
    }

    pub fn build(self) -> Expression {
        Expression::case(self.test_expr, self.cases, self.default)
    }
}

// 使用示例
let expr = CaseBuilder::new()
    .test(test_expr)
    .when(when_expr1, then_expr1)
    .when(when_expr2, then_expr2)
    .default(default_expr)
    .build();
```

### 2.9 缺少表达式优化接口

**问题**：没有统一的表达式优化接口

**当前状态**：
```rust
// ❌ 问题：优化逻辑分散在各个模块中

// 在 planner/rewrite/expression_utils.rs 中
pub fn rewrite_expression(
    expr: &Expression,
    rewrite_map: &HashMap<String, Expression>,
) -> Expression;

// 在 planner/rewrite 中
pub fn split_filter<F>(
    condition: &Expression,
    picker: F,
) -> (Option<Expression>, Option<Expression>);

// 在 core/types/expression/type_deduce.rs 中
impl Expression {
    pub fn deduce_type(&self) -> DataType;
}
```

**影响**：
- ❌ 优化逻辑分散，难以维护
- ❌ 无法统一管理优化策略
- ❌ 难以添加新的优化规则

**建议**：
```rust
// ✅ 改进：统一的表达式优化接口

pub trait ExpressionOptimizer {
    /// 优化表达式
    fn optimize(&self, expr: &Expression) -> Expression;

    /// 检查是否可以优化
    fn can_optimize(&self, expr: &Expression) -> bool;
}

/// 常量折叠优化器
pub struct ConstantFoldingOptimizer;

impl ExpressionOptimizer for ConstantFoldingOptimizer {
    fn optimize(&self, expr: &Expression) -> Expression {
        match expr {
            Expression::Binary { left, op, right } => {
                if left.is_constant() && right.is_constant() {
                    // 计算常量
                    if let Some(result) = self.fold_binary(op, left, right) {
                        return Expression::literal(result);
                    }
                }
                expr.clone()
            }
            _ => expr.clone(),
        }
    }

    fn can_optimize(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Binary { left, right, .. } => {
                left.is_constant() && right.is_constant()
            }
            _ => false,
        }
    }
}

/// 表达式优化器
pub struct ExpressionOptimizerChain {
    optimizers: Vec<Box<dyn ExpressionOptimizer>>,
}

impl ExpressionOptimizerChain {
    pub fn new() -> Self {
        Self {
            optimizers: Vec::new(),
        }
    }

    pub fn add_optimizer(mut self, optimizer: Box<dyn ExpressionOptimizer>) -> Self {
        self.optimizers.push(optimizer);
        self
    }

    pub fn optimize(&self, expr: &Expression) -> Expression {
        let mut current = expr.clone();
        for optimizer in &self.optimizers {
            if optimizer.can_optimize(&current) {
                current = optimizer.optimize(&current);
            }
        }
        current
    }
}
```

## 三、设计改进建议

### 3.1 简化表达式变体

**建议**：合并冗余的属性访问变体

```rust
// ✅ 改进：统一属性访问

pub enum Expression {
    // ... 其他变体

    /// 统一的属性访问
    ///
    /// 支持以下场景：
    /// - 普通属性：`v.name`
    /// - 标签属性：`Tag.name`
    /// - 边属性：`EdgeType.name`
    /// - 动态标签属性：`(tagName).name`
    Property {
        object: Box<Expression>,
        property: String,
        property_type: PropertyType,  // 新增：属性类型
    },
}

/// 属性类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PropertyType {
    /// 普通属性
    Normal,
    /// 标签属性
    Label(String),
    /// 边属性
    Edge(String),
}
```

### 3.2 合并模块文件

**建议**：将 `Expression` 的方法合并到一个文件

```rust
// ✅ 改进：将所有方法合并到 expression_methods.rs

// src/core/types/expression/
├── def.rs              # Expression 枚举定义
├── expression.rs        # ExpressionId, ExpressionMeta
├── expression_methods.rs  # Expression 的所有方法
├── context.rs          # ExpressionContext
├── contextual.rs       # ContextualExpression
└── mod.rs             # 模块导出
```

**优势**：
- ✅ 易于理解 `Expression` 的完整功能
- ✅ 便于维护（修改一个功能只需在一个文件中）
- ✅ 改善代码导航（IDE 的"跳转到定义"效果更好）

### 3.3 移除未使用的变体

**建议**：移除或标记为 deprecated

```rust
// ✅ 改进：移除未使用的变体

#[deprecated(since = "1.0.0", note = "使用 Property 代替")]
pub enum Expression {
    // ... 其他变体

    #[deprecated]
    LabelTagProperty {
        tag: Box<Expression>,
        property: String,
    },

    #[deprecated]
    TagProperty {
        tag_name: String,
        property: String,
    },

    #[deprecated]
    EdgeProperty {
        edge_name: String,
        property: String,
    },
}
```

### 3.4 移动工具函数

**建议**：将 `utils.rs` 移动到合适的位置

```rust
// ✅ 改进：移动到 optimizer 模块

// src/query/planner/optimizer/group_by_utils.rs
pub struct GroupSuite {
    pub group_keys: Vec<Expression>,
    pub group_items: Vec<Expression>,
    pub aggregates: Vec<Expression>,
}

pub fn extract_group_suite(expression: &Expression) -> Result<GroupSuite, String>;
```

### 3.5 添加表达式验证

**建议**：添加表达式验证机制

```rust
// ✅ 改进：添加验证

impl Expression {
    pub fn validate(&self) -> Result<(), ValidationError> {
        self.validate_recursive()
    }

    fn validate_recursive(&self) -> Result<(), ValidationError> {
        match self {
            Expression::TypeCast { expression, target_type } => {
                let source_type = expression.deduce_type();
                if !is_valid_cast(&source_type, target_type) {
                    return Err(ValidationError::InvalidCast {
                        source: source_type,
                        target: target_type.clone(),
                    });
                }
                expression.validate_recursive()
            }
            Expression::Property { object, .. } => {
                if object.is_literal() {
                    return Err(ValidationError::LiteralHasProperty);
                }
                object.validate_recursive()
            }
            Expression::Aggregate { arg, .. } => {
                if arg.contains_aggregate() {
                    return Err(ValidationError::NestedAggregate);
                }
                arg.validate_recursive()
            }
            _ => {
                for child in self.children() {
                    child.validate_recursive()?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    InvalidCast { source: DataType, target: DataType },
    LiteralHasProperty,
    NestedAggregate,
    InvalidAggregateFunction(String),
    InvalidFunctionCall(String),
}
```

### 3.6 统一类型推导接口

**建议**：类型推导使用 `ExpressionContext` 缓存

```rust
// ✅ 改进：统一类型推导

impl Expression {
    /// 推导表达式类型（不使用缓存）
    pub fn deduce_type(&self) -> DataType {
        // 原有实现
    }

    /// 推导表达式类型（使用上下文缓存）
    pub fn deduce_type_with_context(
        &self,
        ctx: &ExpressionContext,
        id: &ExpressionId,
    ) -> DataType {
        if let Some(cached_type) = ctx.get_type(id) {
            return cached_type;
        }

        let data_type = self.deduce_type();
        ctx.set_type(id, data_type.clone());

        data_type
    }
}
```

### 3.7 简化序列化

**建议**：移除 `SerializableExpression`，扩展 `ContextualExpression`

```rust
// ✅ 改进：ContextualExpression 支持序列化

#[derive(Serialize, Deserialize)]
pub struct ContextualExpression {
    id: ExpressionId,
    #[serde(skip)]
    context: Arc<ExpressionContext>,
    #[serde(default)]
    cached_expression: Option<Expression>,
    #[serde(default)]
    cached_type: Option<DataType>,
    #[serde(default)]
    cached_constant: Option<Value>,
}

impl ContextualExpression {
    /// 序列化前准备
    pub fn prepare_for_serialization(&mut self) {
        self.cached_expression = self.expression()
            .map(|meta| meta.inner().clone());
        self.cached_type = self.data_type();
        self.cached_constant = self.constant_value();
    }

    /// 反序列化后恢复
    pub fn after_deserialization(&mut self, ctx: Arc<ExpressionContext>) {
        self.context = ctx;

        if let Some(ref expr) = self.cached_expression {
            let meta = ExpressionMeta::new(expr.clone()).with_id(self.id.clone());
            self.context.register_expression(meta);
        }

        if let Some(ref data_type) = self.cached_type {
            self.context.set_type(&self.id, data_type.clone());
        }

        if let Some(ref constant) = self.cached_constant {
            self.context.set_constant(&self.id, constant.clone());
        }
    }
}
```

### 3.8 改进构造函数

**建议**：使用 Builder 模式

```rust
// ✅ 改进：Builder 模式

pub struct ExpressionBuilder {
    ctx: Arc<ExpressionContext>,
}

impl ExpressionBuilder {
    pub fn new(ctx: Arc<ExpressionContext>) -> Self {
        Self { ctx }
    }

    pub fn literal(self, value: Value) -> ContextualExpression {
        let expr = Expression::Literal(value);
        self.register(expr)
    }

    pub fn variable(self, name: String) -> ContextualExpression {
        let expr = Expression::Variable(name);
        self.register(expr)
    }

    pub fn property(
        self,
        object: ContextualExpression,
        property: String,
    ) -> ContextualExpression {
        let expr = Expression::Property {
            object: Box::new(object.to_expression()),
            property,
        };
        self.register(expr)
    }

    pub fn binary(
        self,
        left: ContextualExpression,
        op: BinaryOperator,
        right: ContextualExpression,
    ) -> ContextualExpression {
        let expr = Expression::Binary {
            left: Box::new(left.to_expression()),
            op,
            right: Box::new(right.to_expression()),
        };
        self.register(expr)
    }

    fn register(&self, expr: Expression) -> ContextualExpression {
        let meta = ExpressionMeta::new(expr);
        let id = self.ctx.register_expression(meta);
        ContextualExpression::new(id, self.ctx.clone())
    }
}
```

### 3.9 添加优化接口

**建议**：统一的表达式优化接口

```rust
// ✅ 改进：优化接口

pub trait ExpressionOptimizer {
    fn optimize(&self, expr: &Expression) -> Expression;
    fn can_optimize(&self, expr: &Expression) -> bool;
}

pub struct ExpressionOptimizerChain {
    optimizers: Vec<Box<dyn ExpressionOptimizer>>,
}

impl ExpressionOptimizerChain {
    pub fn new() -> Self {
        Self {
            optimizers: Vec::new(),
        }
    }

    pub fn add_optimizer(mut self, optimizer: Box<dyn ExpressionOptimizer>) -> Self {
        self.optimizers.push(optimizer);
        self
    }

    pub fn optimize(&self, expr: &Expression) -> Expression {
        let mut current = expr.clone();
        for optimizer in &self.optimizers {
            if optimizer.can_optimize(&current) {
                current = optimizer.optimize(&current);
            }
        }
        current
    }
}
```

## 四、总结

### 4.1 主要问题

| 问题 | 严重性 | 影响 |
|------|---------|------|
| 职责分散 | 🔴 高 | 难以理解和维护 |
| 变体冗余 | 🔴 高 | 增加复杂度和冗余代码 |
| 缺少文档 | 🟡 中 | 难以正确使用 |
| 工具函数位置不当 | 🟡 中 | 职责不清，依赖混乱 |
| 缺少验证 | 🟡 中 | 运行时错误 |
| 类型推导脱节 | 🟡 中 | 性能开销 |
| SerializableExpression 冗余 | 🟢 低 | 维护成本 |
| 构造函数命名不一致 | 🟢 低 | 易用性差 |
| 缺少优化接口 | 🟢 低 | 难以扩展 |

### 4.2 改进优先级

**高优先级**：
1. 合并 `Expression` 的方法到一个文件
2. 简化表达式变体（合并冗余的属性访问变体）
3. 移除未使用的变体

**中优先级**：
4. 移动工具函数到合适的位置
5. 添加表达式验证机制
6. 统一类型推导接口

**低优先级**：
7. 简化序列化（移除 `SerializableExpression`）
8. 改进构造函数（使用 Builder 模式）
9. 添加优化接口

### 4.3 重构建议

**阶段 1：清理**
- 移除未使用的变体
- 移动工具函数到合适的位置
- 添加文档说明

**阶段 2：简化**
- 合并 `Expression` 的方法到一个文件
- 简化表达式变体
- 统一属性访问变体

**阶段 3：增强**
- 添加表达式验证机制
- 统一类型推导接口
- 添加优化接口

**阶段 4：改进**
- 简化序列化
- 改进构造函数
- 添加 Builder 模式
