# Nebula-Graph 设计分析与 Visitor 模式实施方案

## 📊 执行摘要

本文档总结了基于 Nebula-Graph 源码分析得出的设计建议，以及 GraphDB 引入 Visitor 模式的实施方案。

### 核心结论

| 功能 | 状态 | 优先级 | 工作量 | 收益 |
|------|------|--------|--------|------|
| ExpressionVisitor | ✅ 引入 | 高 | 2-3 天 | 高 |
| 规则匹配模式增强 | ❌ 不引入 | - | - | - |
| 成本计算模型 | ❌ 不引入 | - | - | - |

---

## 1. Nebula-Graph 设计分析

### 1.1 表达式系统设计

#### Nebula-Graph 的设计

**ExpressionContext** (`nebula-3.8.0/src/common/context/ExpressionContext.h`)
```cpp
class ExpressionContext {
public:
  virtual ~ExpressionContext() = default;

  // 运行时值访问接口
  virtual const Value& getVar(const std::string& var) const = 0;
  virtual void setInnerVar(const std::string& var, Value val) = 0;
  virtual const Value& getVarProp(const std::string& var, const std::string& prop) const = 0;
  virtual Value getEdgeProp(const std::string& edgeType, const std::string& prop) const = 0;
  // ... 更多访问方法
};
```

**QueryExpressionContext** (`nebula-3.8.0/src/graph/context/QueryExpressionContext.h`)
```cpp
class QueryExpressionContext final : public ExpressionContext {
private:
  ExecutionContext* ectx_{nullptr};
  Iterator* iter_{nullptr};
  std::unordered_map<std::string, Value> exprValueMap_;
};
```

#### 可借鉴的设计

✅ **职责分离**：
- ExpressionContext 只定义接口，不存储表达式
- QueryExpressionContext 持有执行上下文和迭代器
- 表达式本身不包含上下文引用

✅ **运行时值访问**：
- 提供统一的值访问接口（getVar, getVarProp, getEdgeProp 等）
- 支持不同类型的属性访问（变量、边、顶点、输入）

#### 对比当前 GraphDB 实现

**当前问题**：
- ExpressionContext 存储表达式注册表和缓存
- ContextualExpression 持有对 ExpressionContext 的引用

**改进建议**：
```rust
// 将 ExpressionContext 拆分为两个职责
pub struct ExpressionRegistry {
    // 只负责表达式注册和 ID 管理
    expressions: DashMap<ExpressionId, Arc<ExpressionMeta>>,
}

pub struct ExpressionContext {
    // 负责运行时值访问和缓存
    registry: Arc<ExpressionRegistry>,
    type_cache: DashMap<ExpressionId, DataType>,
    constant_cache: DashMap<ExpressionId, Value>,
}
```

### 1.2 访问者模式

#### Nebula-Graph 的设计

**ExprVisitor** (`nebula-3.8.0/src/common/expression/ExprVisitor.h`)
```cpp
class ExprVisitor {
public:
  virtual ~ExprVisitor() = default;

  // 为每种表达式类型定义访问方法
  virtual void visit(ConstantExpression *expr) = 0;
  virtual void visit(UnaryExpression *expr) = 0;
  virtual void visit(ArithmeticExpression *expr) = 0;
  virtual void visit(FunctionCallExpression *expr) = 0;
  virtual void visit(VariableExpression *expr) = 0;
  // ... 30+ 种表达式类型
};
```

#### 可借鉴的设计

✅ **类型安全的遍历**：
- 为每种表达式类型定义专门的访问方法
- 编译时确保所有类型都被处理

✅ **扩展性**：
- 添加新表达式类型时，只需在访问者中添加新方法
- 不需要修改现有代码

### 1.3 优化器架构

#### Nebula-Graph 的设计

**OptRule** (`nebula-3.8.0/src/graph/optimizer/OptRule.h`)
```cpp
class OptRule {
public:
  struct TransformResult {
    bool eraseCurr{false};
    bool eraseAll{false};
    std::vector<OptGroupNode *> newGroupNodes;
  };

  // 匹配模式
  virtual const Pattern &pattern() const = 0;

  // 检查是否匹配
  virtual bool match(OptContext *ctx, const MatchedResult &matched) const;

  // 执行转换
  virtual StatusOr<TransformResult> transform(OptContext *ctx,
                                         const MatchedResult &matched) const = 0;
};
```

**OptGroup** (`nebula-3.8.0/src/graph/optimizer/OptGroup.h`)
```cpp
class OptGroup final {
public:
  // 管理多个等价的计划节点
  void addGroupNode(OptGroupNode *groupNode);

  // 探索规则
  Status explore(const OptRule *rule);

  // 选择成本最低的计划
  const graph::PlanNode *getPlan() const;
};
```

#### 可借鉴的设计

✅ **基于模式的匹配**：
- 使用 Pattern 描述要匹配的计划结构
- 支持嵌套模式匹配

✅ **等价计划管理**：
- OptGroup 管理多个等价的计划节点
- 自动选择成本最低的计划

✅ **迭代优化**：
- 规则可以多次应用
- 探索状态跟踪（exploredRules）

### 1.4 成本计算模型

**关键发现**：Nebula-Graph 的成本计算完全是空壳，没有实际实现。

**结论**：GraphDB 不需要引入成本计算模型，专注于启发式优化即可。

---

## 2. 当前 GraphDB Rewrite 层分析

### 2.1 现状分析

#### 规则匹配模式已经存在 ✅

当前 GraphDB 已经实现了完整的 Pattern 系统：

- **Pattern 类型** (`src/query/planner/rewrite/pattern.rs`)
  - 支持节点类型匹配
  - 支持嵌套依赖模式
  - 提供便捷构造方法

- **RewriteRule trait** (`src/query/planner/rewrite/rule.rs`)
  - 已有 30+ 个规则实现
  - 包含 `pattern()` 方法
  - 统一的规则应用接口

#### 表达式分析使用手动模式匹配 ⚠️

在多个文件中发现重复的 `match expr` 语句：

**expression_utils.rs** (`src/query/planner/rewrite/expression_utils.rs:42`)
```rust
fn check_col_name_expr(property_names: &[String], expr: &Expression) -> bool {
    match expr {
        Expression::Property { property, .. } => property_names.contains(property),
        Expression::Binary { left, right, .. } => {
            check_col_name_expr(property_names, left) || check_col_name_expr(property_names, right)
        }
        Expression::Unary { operand, .. } => check_col_name_expr(property_names, operand),
        Expression::Function { args, .. } => {
            args.iter().any(|arg| check_col_name_expr(property_names, arg))
        }
        // ... 更多分支
    }
}
```

**eliminate_filter.rs** (`src/query/planner/rewrite/elimination/eliminate_filter.rs:49`)
```rust
fn is_contradiction(&self, expression: &Expression) -> bool {
    match expression {
        Expression::Literal(Value::Bool(false)) => true,
        Expression::Literal(Value::Null(_)) => true,
        Expression::Binary { left, op, right } => {
            match (left.as_ref(), op, right.as_ref()) {
                (Expression::Literal(Value::Int(1)), BinaryOperator::Equal,
                 Expression::Literal(Value::Int(0))) => true,
                // ... 更多模式
            }
        }
        _ => false,
    }
}
```

### 2.2 规则匹配模式必要性评估

#### 当前 Pattern 系统功能对比

| 功能 | Nebula-Graph | GraphDB 当前 | 需求 |
|------|--------------|-------------|------|
| 节点类型匹配 | ✅ | ✅ | ✅ |
| 嵌套依赖模式 | ✅ | ✅ | ✅ |
| 通配符匹配 | ✅ | ❌ | ❌ |
| 条件匹配 | ✅ | ❌ | ❌ |
| 多类型匹配 | ✅ | ✅ | ✅ |

#### 实际使用情况分析

查看 30+ 个规则的实际使用：

1. **简单类型匹配**（90% 的规则）
   ```rust
   Pattern::new_with_name("Filter").with_dependency_name("Traverse")
   Pattern::new_with_name("Filter").with_dependency_name("Filter")
   ```

2. **多类型匹配**（10% 的规则）
   ```rust
   Pattern::multi(vec!["HashInnerJoin", "HashLeftJoin", "InnerJoin"])
   ```

3. **没有使用的功能**：
   - 通配符匹配
   - 条件匹配
   - 复杂嵌套模式

#### 评估结论 ❌

**不需要引入更复杂的规则匹配模式**

**理由**：
1. ✅ **当前 Pattern 系统已经足够**
   - 所有现有规则都能正常工作
   - 没有发现无法匹配的场景

2. ✅ **简单即优**
   - 当前的模式匹配清晰易懂
   - 维护成本低

3. ❌ **复杂模式匹配需求不存在**
   - 没有通配符匹配需求
   - 没有条件匹配需求
   - 没有复杂嵌套模式需求

4. ⚠️ **过度设计风险**
   - 引入复杂功能会增加维护成本
   - 可能导致规则难以理解

---

## 3. Visitor 模式实施方案

### 3.1 需要实现的功能

#### 核心访问者接口

```rust
/// 表达式访问者 trait
///
/// 用于遍历和分析表达式树，避免重复的模式匹配代码
pub trait ExpressionVisitor {
    /// 访问表达式
    fn visit(&mut self, expr: &Expression) {
        match expr {
            Expression::Literal(value) => self.visit_literal(value),
            Expression::Variable(name) => self.visit_variable(name),
            Expression::Property { object, property } => {
                self.visit_property(object, property);
            }
            Expression::Binary { left, op, right } => {
                self.visit_binary(*op, left, right);
            }
            Expression::Unary { op, operand } => {
                self.visit_unary(*op, operand);
            }
            Expression::Function { name, args } => {
                self.visit_function(name, args);
            }
            Expression::Aggregate { func, arg, distinct } => {
                self.visit_aggregate(*func, arg, *distinct);
            }
            Expression::Case { test_expr, conditions, default } => {
                self.visit_case(test_expr.as_deref(), conditions, default.as_deref());
            }
            Expression::List(items) => {
                self.visit_list(items);
            }
            Expression::Map(entries) => {
                self.visit_map(entries);
            }
            Expression::TypeCast { expression, target_type } => {
                self.visit_type_cast(expression, *target_type);
            }
            Expression::Subscript { collection, index } => {
                self.visit_subscript(collection, index);
            }
            Expression::Range { collection, start, end } => {
                self.visit_range(collection, start.as_deref(), end.as_deref());
            }
            Expression::Path(items) => {
                self.visit_path(items);
            }
            Expression::Label(label) => {
                self.visit_label(label);
            }
            Expression::ListComprehension { variable, source, filter, map } => {
                self.visit_list_comprehension(variable, source, filter.as_deref(), map.as_deref());
            }
        }
    }

    fn visit_literal(&mut self, value: &Value);
    fn visit_variable(&mut self, name: &str);
    fn visit_property(&mut self, object: &Expression, property: &str);
    fn visit_binary(&mut self, op: BinaryOperator, left: &Expression, right: &Expression);
    fn visit_unary(&mut self, op: UnaryOperator, operand: &Expression);
    fn visit_function(&mut self, name: &str, args: &[Expression]);
    fn visit_aggregate(&mut self, func: AggregateFunction, arg: &Expression, distinct: bool);
    fn visit_case(&mut self, test_expr: Option<&Expression>, conditions: &[(Expression, Expression)], default: Option<&Expression>);
    fn visit_list(&mut self, items: &[Expression]);
    fn visit_map(&mut self, entries: &[(String, Expression)]);
    fn visit_type_cast(&mut self, expression: &Expression, target_type: DataType);
    fn visit_subscript(&mut self, collection: &Expression, index: &Expression);
    fn visit_range(&mut self, collection: &Expression, start: Option<&Expression>, end: Option<&Expression>);
    fn visit_path(&mut self, items: &[Expression]);
    fn visit_label(&mut self, label: &str);
    fn visit_list_comprehension(&mut self, variable: &str, source: &Expression, filter: Option<&Expression>, map: Option<&Expression>);
}
```

#### 具体分析器实现

##### 1. PropertyCollector - 属性收集器

**功能**：收集表达式中所有使用的属性名

**使用场景**：
- `push_vfilter_down_scan_vertices.rs` - 收集顶点属性
- `push_efilter_down.rs` - 收集边属性
- `eliminate_append_vertices.rs` - 检查属性使用

**实现**：
```rust
#[derive(Debug, Default)]
pub struct PropertyCollector {
    pub properties: Vec<String>,
}

impl PropertyCollector {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ExpressionVisitor for PropertyCollector {
    fn visit_literal(&mut self, _value: &Value) {}
    fn visit_variable(&mut self, _name: &str) {}

    fn visit_property(&mut self, _object: &Expression, property: &str) {
        let prop_name = property.to_string();
        if !self.properties.contains(&prop_name) {
            self.properties.push(prop_name);
        }
    }

    fn visit_binary(&mut self, _op: BinaryOperator, left: &Expression, right: &Expression) {
        self.visit(left);
        self.visit(right);
    }

    fn visit_unary(&mut self, _op: UnaryOperator, operand: &Expression) {
        self.visit(operand);
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) {
        for arg in args {
            self.visit(arg);
        }
    }

    fn visit_aggregate(&mut self, _func: AggregateFunction, arg: &Expression, _distinct: bool) {
        self.visit(arg);
    }

    fn visit_case(&mut self, _test_expr: Option<&Expression>, conditions: &[(Expression, Expression)], default: Option<&Expression>) {
        for (when, then) in conditions {
            self.visit(when);
            self.visit(then);
        }
        if let Some(default_expr) = default {
            self.visit(default_expr);
        }
    }

    fn visit_list(&mut self, items: &[Expression]) {
        for item in items {
            self.visit(item);
        }
    }

    fn visit_map(&mut self, entries: &[(String, Expression)]) {
        for (_, value) in entries {
            self.visit(value);
        }
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: DataType) {
        self.visit(expression);
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) {
        self.visit(collection);
        self.visit(index);
    }

    fn visit_range(&mut self, collection: &Expression, start: Option<&Expression>, end: Option<&Expression>) {
        self.visit(collection);
        if let Some(start_expr) = start {
            self.visit(start_expr);
        }
        if let Some(end_expr) = end {
            self.visit(end_expr);
        }
    }

    fn visit_path(&mut self, items: &[Expression]) {
        for item in items {
            self.visit(item);
        }
    }

    fn visit_label(&mut self, _label: &str) {}

    fn visit_list_comprehension(&mut self, _variable: &str, source: &Expression, filter: Option<&Expression>, map: Option<&Expression>) {
        self.visit(source);
        if let Some(filter_expr) = filter {
            self.visit(filter_expr);
        }
        if let Some(map_expr) = map {
            self.visit(map_expr);
        }
    }
}
```

##### 2. VariableCollector - 变量收集器

**功能**：收集表达式中所有使用的变量名

**使用场景**：
- 检查表达式中使用的变量
- 变量作用域分析

**实现**：
```rust
#[derive(Debug, Default)]
pub struct VariableCollector {
    pub variables: Vec<String>,
}

impl VariableCollector {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ExpressionVisitor for VariableCollector {
    fn visit_literal(&mut self, _value: &Value) {}

    fn visit_variable(&mut self, name: &str) {
        let var_name = name.to_string();
        if !self.variables.contains(&var_name) {
            self.variables.push(var_name);
        }
    }

    fn visit_property(&mut self, object: &Expression, _property: &str) {
        self.visit(object);
    }

    fn visit_binary(&mut self, _op: BinaryOperator, left: &Expression, right: &Expression) {
        self.visit(left);
        self.visit(right);
    }

    fn visit_unary(&mut self, _op: UnaryOperator, operand: &Expression) {
        self.visit(operand);
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) {
        for arg in args {
            self.visit(arg);
        }
    }

    fn visit_aggregate(&mut self, _func: AggregateFunction, arg: &Expression, _distinct: bool) {
        self.visit(arg);
    }

    fn visit_case(&mut self, _test_expr: Option<&Expression>, conditions: &[(Expression, Expression)], default: Option<&Expression>) {
        for (when, then) in conditions {
            self.visit(when);
            self.visit(then);
        }
        if let Some(default_expr) = default {
            self.visit(default_expr);
        }
    }

    fn visit_list(&mut self, items: &[Expression]) {
        for item in items {
            self.visit(item);
        }
    }

    fn visit_map(&mut self, entries: &[(String, Expression)]) {
        for (_, value) in entries {
            self.visit(value);
        }
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: DataType) {
        self.visit(expression);
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) {
        self.visit(collection);
        self.visit(index);
    }

    fn visit_range(&mut self, collection: &Expression, start: Option<&Expression>, end: Option<&Expression>) {
        self.visit(collection);
        if let Some(start_expr) = start {
            self.visit(start_expr);
        }
        if let Some(end_expr) = end {
            self.visit(end_expr);
        }
    }

    fn visit_path(&mut self, items: &[Expression]) {
        for item in items {
            self.visit(item);
        }
    }

    fn visit_label(&mut self, _label: &str) {}

    fn visit_list_comprehension(&mut self, _variable: &str, source: &Expression, filter: Option<&Expression>, map: Option<&Expression>) {
        self.visit(source);
        if let Some(filter_expr) = filter {
            self.visit(filter_expr);
        }
        if let Some(map_expr) = map {
            self.visit(map_expr);
        }
    }
}
```

##### 3. ConstantChecker - 常量检查器

**功能**：检查表达式是否为常量表达式（不包含变量或属性）

**使用场景**：
- 检查表达式是否可以在编译时求值
- 优化常量表达式

**实现**：
```rust
#[derive(Debug, Default)]
pub struct ConstantChecker {
    pub is_constant: bool,
}

impl ConstantChecker {
    pub fn new() -> Self {
        Self { is_constant: true }
    }

    pub fn check(expr: &Expression) -> bool {
        let mut checker = Self::new();
        checker.visit(expr);
        checker.is_constant
    }
}

impl ExpressionVisitor for ConstantChecker {
    fn visit_literal(&mut self, _value: &Value) {}

    fn visit_variable(&mut self, _name: &str) {
        self.is_constant = false;
    }

    fn visit_property(&mut self, _object: &Expression, _property: &str) {
        self.is_constant = false;
    }

    fn visit_binary(&mut self, _op: BinaryOperator, left: &Expression, right: &Expression) {
        if self.is_constant {
            self.visit(left);
        }
        if self.is_constant {
            self.visit(right);
        }
    }

    fn visit_unary(&mut self, _op: UnaryOperator, operand: &Expression) {
        if self.is_constant {
            self.visit(operand);
        }
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) {
        if self.is_constant {
            for arg in args {
                self.visit(arg);
                if !self.is_constant {
                    break;
                }
            }
        }
    }

    fn visit_aggregate(&mut self, _func: AggregateFunction, arg: &Expression, _distinct: bool) {
        if self.is_constant {
            self.visit(arg);
        }
    }

    fn visit_case(&mut self, _test_expr: Option<&Expression>, conditions: &[(Expression, Expression)], default: Option<&Expression>) {
        if self.is_constant {
            if let Some(test) = _test_expr {
                self.visit(test);
                if !self.is_constant {
                    return;
                }
            }
            for (when, then) in conditions {
                self.visit(when);
                if !self.is_constant {
                    return;
                }
                self.visit(then);
                if !self.is_constant {
                    return;
                }
            }
            if let Some(default_expr) = default {
                self.visit(default_expr);
            }
        }
    }

    fn visit_list(&mut self, items: &[Expression]) {
        if self.is_constant {
            for item in items {
                self.visit(item);
                if !self.is_constant {
                    break;
                }
            }
        }
    }

    fn visit_map(&mut self, entries: &[(String, Expression)]) {
        if self.is_constant {
            for (_, value) in entries {
                self.visit(value);
                if !self.is_constant {
                    break;
                }
            }
        }
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: DataType) {
        if self.is_constant {
            self.visit(expression);
        }
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) {
        if self.is_constant {
            self.visit(collection);
            if self.is_constant {
                self.visit(index);
            }
        }
    }

    fn visit_range(&mut self, collection: &Expression, start: Option<&Expression>, end: Option<&Expression>) {
        if self.is_constant {
            self.visit(collection);
            if self.is_constant {
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                    if !self.is_constant {
                        return;
                    }
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr);
                }
            }
        }
    }

    fn visit_path(&mut self, items: &[Expression]) {
        if self.is_constant {
            for item in items {
                self.visit(item);
                if !self.is_constant {
                    break;
                }
            }
        }
    }

    fn visit_label(&mut self, _label: &str) {}

    fn visit_list_comprehension(&mut self, _variable: &str, source: &Expression, filter: Option<&Expression>, map: Option<&Expression>) {
        if self.is_constant {
            self.visit(source);
            if self.is_constant {
                if let Some(filter_expr) = filter {
                    self.visit(filter_expr);
                    if !self.is_constant {
                        return;
                    }
                }
                if let Some(map_expr) = map {
                    self.visit(map_expr);
                }
            }
        }
    }
}
```

##### 4. PropertyContainsChecker - 属性包含检查器

**功能**：检查表达式是否包含指定的属性名

**使用场景**：
- `expression_utils.rs::check_col_name` - 检查是否包含特定属性
- `push_filter_down_traverse.rs` - 检查是否包含边属性

**实现**：
```rust
#[derive(Debug)]
pub struct PropertyContainsChecker {
    pub property_names: Vec<String>,
    pub contains: bool,
}

impl PropertyContainsChecker {
    pub fn new(property_names: Vec<String>) -> Self {
        Self {
            property_names,
            contains: false,
        }
    }

    pub fn check(expr: &Expression, property_names: &[String]) -> bool {
        let mut checker = Self::new(property_names.to_vec());
        checker.visit(expr);
        checker.contains
    }
}

impl ExpressionVisitor for PropertyContainsChecker {
    fn visit_literal(&mut self, _value: &Value) {}
    fn visit_variable(&mut self, _name: &str) {}

    fn visit_property(&mut self, _object: &Expression, property: &str) {
        if self.property_names.contains(&property.to_string()) {
            self.contains = true;
        }
    }

    fn visit_binary(&mut self, _op: BinaryOperator, left: &Expression, right: &Expression) {
        if !self.contains {
            self.visit(left);
        }
        if !self.contains {
            self.visit(right);
        }
    }

    fn visit_unary(&mut self, _op: UnaryOperator, operand: &Expression) {
        if !self.contains {
            self.visit(operand);
        }
    }

    fn visit_function(&mut self, _name: &str, args: &[Expression]) {
        if !self.contains {
            for arg in args {
                self.visit(arg);
                if self.contains {
                    break;
                }
            }
        }
    }

    fn visit_aggregate(&mut self, _func: AggregateFunction, arg: &Expression, _distinct: bool) {
        if !self.contains {
            self.visit(arg);
        }
    }

    fn visit_case(&mut self, _test_expr: Option<&Expression>, conditions: &[(Expression, Expression)], default: Option<&Expression>) {
        if !self.contains {
            if let Some(test) = _test_expr {
                self.visit(test);
                if self.contains {
                    return;
                }
            }
            for (when, then) in conditions {
                self.visit(when);
                if self.contains {
                    return;
                }
                self.visit(then);
                if self.contains {
                    return;
                }
            }
            if let Some(default_expr) = default {
                self.visit(default_expr);
            }
        }
    }

    fn visit_list(&mut self, items: &[Expression]) {
        if !self.contains {
            for item in items {
                self.visit(item);
                if self.contains {
                    break;
                }
            }
        }
    }

    fn visit_map(&mut self, entries: &[(String, Expression)]) {
        if !self.contains {
            for (_, value) in entries {
                self.visit(value);
                if self.contains {
                    break;
                }
            }
        }
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: DataType) {
        if !self.contains {
            self.visit(expression);
        }
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) {
        if !self.contains {
            self.visit(collection);
            if !self.contains {
                self.visit(index);
            }
        }
    }

    fn visit_range(&mut self, collection: &Expression, start: Option<&Expression>, end: Option<&Expression>) {
        if !self.contains {
            self.visit(collection);
            if !self.contains {
                if let Some(start_expr) = start {
                    self.visit(start_expr);
                    if self.contains {
                        return;
                    }
                }
                if let Some(end_expr) = end {
                    self.visit(end_expr);
                }
            }
        }
    }

    fn visit_path(&mut self, items: &[Expression]) {
        if !self.contains {
            for item in items {
                self.visit(item);
                if self.contains {
                    break;
                }
            }
        }
    }

    fn visit_label(&mut self, _label: &str) {}

    fn visit_list_comprehension(&mut self, _variable: &str, source: &Expression, filter: Option<&Expression>, map: Option<&Expression>) {
        if !self.contains {
            self.visit(source);
            if !self.contains {
                if let Some(filter_expr) = filter {
                    self.visit(filter_expr);
                    if self.contains {
                        return;
                    }
                }
                if let Some(map_expr) = map {
                    self.visit(map_expr);
                }
            }
        }
    }
}
```

##### 5. FunctionCollector - 函数收集器（可选）

**功能**：收集表达式中所有使用的函数名

**使用场景**：
- 检查表达式中使用的函数
- 函数依赖分析

**实现**：
```rust
#[derive(Debug, Default)]
pub struct FunctionCollector {
    pub functions: Vec<String>,
}

impl FunctionCollector {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ExpressionVisitor for FunctionCollector {
    fn visit_literal(&mut self, _value: &Value) {}
    fn visit_variable(&mut self, _name: &str) {}
    fn visit_property(&mut self, object: &Expression, _property: &str) {
        self.visit(object);
    }

    fn visit_binary(&mut self, _op: BinaryOperator, left: &Expression, right: &Expression) {
        self.visit(left);
        self.visit(right);
    }

    fn visit_unary(&mut self, _op: UnaryOperator, operand: &Expression) {
        self.visit(operand);
    }

    fn visit_function(&mut self, name: &str, args: &[Expression]) {
        let func_name = name.to_string();
        if !self.functions.contains(&func_name) {
            self.functions.push(func_name);
        }
        for arg in args {
            self.visit(arg);
        }
    }

    fn visit_aggregate(&mut self, func: AggregateFunction, arg: &Expression, _distinct: bool) {
        let func_name = format!("{:?}", func);
        if !self.functions.contains(&func_name) {
            self.functions.push(func_name);
        }
        self.visit(arg);
    }

    fn visit_case(&mut self, _test_expr: Option<&Expression>, conditions: &[(Expression, Expression)], default: Option<&Expression>) {
        for (when, then) in conditions {
            self.visit(when);
            self.visit(then);
        }
        if let Some(default_expr) = default {
            self.visit(default_expr);
        }
    }

    fn visit_list(&mut self, items: &[Expression]) {
        for item in items {
            self.visit(item);
        }
    }

    fn visit_map(&mut self, entries: &[(String, Expression)]) {
        for (_, value) in entries {
            self.visit(value);
        }
    }

    fn visit_type_cast(&mut self, expression: &Expression, _target_type: DataType) {
        self.visit(expression);
    }

    fn visit_subscript(&mut self, collection: &Expression, index: &Expression) {
        self.visit(collection);
        self.visit(index);
    }

    fn visit_range(&mut self, collection: &Expression, start: Option<&Expression>, end: Option<&Expression>) {
        self.visit(collection);
        if let Some(start_expr) = start {
            self.visit(start_expr);
        }
        if let Some(end_expr) = end {
            self.visit(end_expr);
        }
    }

    fn visit_path(&mut self, items: &[Expression]) {
        for item in items {
            self.visit(item);
        }
    }

    fn visit_label(&mut self, _label: &str) {}

    fn visit_list_comprehension(&mut self, _variable: &str, source: &Expression, filter: Option<&Expression>, map: Option<&Expression>) {
        self.visit(source);
        if let Some(filter_expr) = filter {
            self.visit(filter_expr);
        }
        if let Some(map_expr) = map {
            self.visit(map_expr);
        }
    }
}
```

### 3.2 文件组织结构分析

#### 当前表达式目录结构

```
src/core/types/expression/
├── construction.rs      # 表达式构造方法
├── context.rs           # 表达式上下文
├── contextual.rs        # 上下文表达式
├── def.rs              # 表达式类型定义
├── display.rs          # 表达式显示
├── expression.rs       # 表达式元数据包装器
├── inspection.rs       # 表达式检查
├── mod.rs             # 模块导出
├── serializable.rs    # 序列化
├── traverse.rs        # 遍历
├── type_deduce.rs     # 类型推导
└── utils.rs          # 工具函数
```

#### 文件组织方案对比

##### 方案 A：单文件实现

**文件**：`src/core/types/expression/visitor.rs`

**优点**：
- 简单直接，易于理解
- 所有访问者代码集中在一处
- 便于查找和维护

**缺点**：
- 文件可能较长（约 600-800 行）
- 不同访问者混合在一起

**适用场景**：
- 访问者数量较少（< 10 个）
- 每个访问者实现较简单

##### 方案 B：拆分为多个文件

**目录结构**：
```
src/core/types/expression/visitor/
├── mod.rs              # 模块导出
├── trait.rs            # ExpressionVisitor trait 定义
├── collectors.rs       # 收集器（PropertyCollector, VariableCollector, FunctionCollector）
├── checkers.rs         # 检查器（ConstantChecker, PropertyContainsChecker）
└── transformers.rs     # 转换器（未来扩展）
```

**优点**：
- 职责分离清晰
- 文件大小适中
- 便于扩展

**缺点**：
- 目录结构增加一层
- 需要跨文件引用

**适用场景**：
- 访问者数量较多（> 10 个）
- 访问者类型多样

##### 方案 C：按功能拆分（推荐）

**目录结构**：
```
src/core/types/expression/
├── visitor.rs          # ExpressionVisitor trait 定义
├── visitor_collectors.rs  # 收集器实现
└── visitor_checkers.rs    # 检查器实现
```

**优点**：
- 保持扁平结构
- 文件大小适中
- 职责分离清晰
- 易于扩展

**缺点**：
- 需要管理多个文件

**适用场景**：
- 访问者数量适中（5-10 个）
- 需要清晰的职责分离

#### 推荐方案：方案 C

**理由**：

1. **保持扁平结构**
   - 与现有代码风格一致
   - 避免过度嵌套

2. **文件大小适中**
   - `visitor.rs`：约 150 行（trait 定义）
   - `visitor_collectors.rs`：约 300 行（3 个收集器）
   - `visitor_checkers.rs`：约 250 行（2 个检查器）

3. **职责分离清晰**
   - trait 定义与实现分离
   - 收集器与检查器分离

4. **易于扩展**
   - 新增收集器：添加到 `visitor_collectors.rs`
   - 新增检查器：添加到 `visitor_checkers.rs`
   - 新增转换器：创建 `visitor_transformers.rs`

### 3.3 实施计划

#### 阶段 1：创建 Visitor 基础设施（1-2 天）

**文件**：
- `src/core/types/expression/visitor.rs`
- `src/core/types/expression/visitor_collectors.rs`
- `src/core/types/expression/visitor_checkers.rs`

**内容**：
- ExpressionVisitor trait
- PropertyCollector
- VariableCollector
- ConstantChecker
- PropertyContainsChecker
- FunctionCollector（可选）

**修改**：`src/core/types/expression/mod.rs`
```rust
pub mod visitor;
pub mod visitor_collectors;
pub mod visitor_checkers;

pub use visitor::ExpressionVisitor;
pub use visitor_collectors::{
    PropertyCollector,
    VariableCollector,
    FunctionCollector,
};
pub use visitor_checkers::{
    ConstantChecker,
    PropertyContainsChecker,
};
```

#### 阶段 2：重构 expression_utils.rs（1 天）

**替换前**：
```rust
fn check_col_name_expr(property_names: &[String], expr: &Expression) -> bool {
    match expr {
        Expression::Property { property, .. } => property_names.contains(property),
        Expression::Binary { left, right, .. } => {
            check_col_name_expr(property_names, left) || check_col_name_expr(property_names, right)
        }
        Expression::Unary { operand, .. } => check_col_name_expr(property_names, operand),
        Expression::Function { args, .. } => {
            args.iter().any(|arg| check_col_name_expr(property_names, arg))
        }
        Expression::Case { conditions, default, .. } => {
            let has_in_conditions = conditions.iter().any(|(when, then)| {
                check_col_name_expr(property_names, when) || check_col_name_expr(property_names, then)
            });
            let has_in_default = default
                .as_ref()
                .map(|e| check_col_name_expr(property_names, e))
                .unwrap_or(false);
            has_in_conditions || has_in_default
        }
        _ => false,
    }
}
```

**替换后**：
```rust
pub fn check_col_name(property_names: &[String], expr: &Expression) -> bool {
    PropertyContainsChecker::check(expr, property_names)
}
```

#### 阶段 3：重构其他规则文件（1-2 天）

**需要重构的文件**：
- `eliminate_filter.rs` - 使用 ConstantChecker（部分）
- `push_vfilter_down_scan_vertices.rs` - 使用 PropertyCollector
- `push_efilter_down.rs` - 使用 PropertyCollector
- `eliminate_append_vertices.rs` - 使用 PropertyCollector
- `remove_append_vertices_below_join.rs` - 使用 PropertyCollector

**示例重构**：

**替换前**（`push_vfilter_down_scan_vertices.rs:134`）：
```rust
match expr {
    Expression::Property { property, .. } => {
        property_names.push(property.clone());
    }
    Expression::Binary { left, right, .. } => {
        collect_property_names(property_names, left);
        collect_property_names(property_names, right);
    }
    // ... 更多分支
}
```

**替换后**：
```rust
let mut collector = PropertyCollector::new();
collector.visit(expr);
property_names.extend(collector.properties);
```

#### 阶段 4：测试验证（0.5 天）

**运行测试**：
```bash
cargo test --lib rewrite
analyze_cargo
```

### 3.4 预期收益

#### 代码质量提升

| 指标 | 改进前 | 改进后 | 提升 |
|------|--------|--------|------|
| 重复代码行数 | ~200 行 | ~50 行 | ↓ 75% |
| 表达式分析函数 | 9 个手动匹配 | 4 个 visitor | ↓ 56% |
| 代码可读性 | 中 | 高 | ↑ |
| 维护成本 | 高 | 低 | ↓ |

#### 具体改进

1. **消除重复代码**
   - `check_col_name_expr` → `PropertyContainsChecker::check`
   - `collect_property_names` → `PropertyCollector`
   - 手动遍历逻辑 → 统一的 visitor 模式

2. **提高可维护性**
   - 新增表达式类型时，只需修改 visitor trait
   - 所有分析逻辑自动支持新类型

3. **增强可扩展性**
   - 轻松添加新的分析器（如 AggregateChecker, FunctionCollector）
   - 访问者模式天然支持组合

---

## 4. 不引入的功能及理由

### 4.1 复杂的规则匹配模式

**当前 Pattern 系统已经足够**：
```rust
// 90% 的规则只需要这种简单匹配
Pattern::new_with_name("Filter").with_dependency_name("Traverse")

// 10% 的规则需要多类型匹配
Pattern::multi(vec!["HashInnerJoin", "HashLeftJoin", "InnerJoin"])
```

**不需要的功能**：
- 通配符匹配
- 条件匹配
- 复杂嵌套模式

**理由**：
- 没有实际需求
- 增加维护成本
- 简单即优

### 4.2 成本计算模型

**理由**：
- Nebula-Graph 的成本计算是空壳
- GraphDB 专注于启发式优化
- 当前规则已经足够高效

---

## 5. 总结

### 核心建议

1. ✅ **立即实施**：引入 ExpressionVisitor 模式
2. ❌ **不实施**：复杂的规则匹配模式
3. ❌ **不实施**：成本计算模型

### 文件组织方案

**推荐方案 C**：按功能拆分为 3 个文件
- `visitor.rs` - trait 定义
- `visitor_collectors.rs` - 收集器实现
- `visitor_checkers.rs` - 检查器实现

### 实施优先级

| 阶段 | 任务 | 优先级 | 工作量 |
|------|------|--------|--------|
| 1 | 创建 Visitor 基础设施 | 高 | 1-2 天 |
| 2 | 重构 expression_utils.rs | 高 | 1 天 |
| 3 | 重构其他规则文件 | 中 | 1-2 天 |
| 4 | 测试验证 | 高 | 0.5 天 |

### 预期收益

- **代码重复减少 75%**
- **维护成本显著降低**
- **可扩展性大幅提升**

这个方案在保持代码简洁的同时，显著提升了可维护性和可扩展性。
