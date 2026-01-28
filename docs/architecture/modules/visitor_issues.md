# Visitor 模块问题清单与修改方案

## 问题清单

| 序号 | 问题描述 | 严重程度 | 问题类型 | 状态 |
|------|----------|----------|----------|------|
| 9.1 | 与 Expression Evaluator 功能重叠 | 高 | 架构问题 | 已解决 |
| 9.2 | FoldConstantExprVisitor 实现不完整 | 中 | 功能缺失 | 已解决 |
| 9.3 | 访问者之间缺乏代码共享 | 中 | 代码重复 | 已解决 |
| 9.4 | 错误处理不统一 | 低 | 一致性问题 | 已解决 |
| 9.5 | 缺乏访问者基类 | 低 | 设计问题 | 已解决 |

---

## 详细问题分析

### 问题 9.1: 与 Expression Evaluator 功能重叠

**涉及文件**: 
- `src/query/visitor/`
- `src/query/expression/`

**当前实现**:
```rust
// src/query/visitor/evaluable.rs
pub fn can_evaluate_statically(expr: &Expression) -> bool {
    let mut visitor = EvaluableExprVisitor::new();
    visitor.visit_expression(expr);
    visitor.is_evaluable()
}

// src/query/expression/mod.rs
impl ExpressionEvaluator {
    pub fn can_evaluate(&self, expr: &Expression, _ctx: &mut dyn ExpressionContext) -> bool {
        match expr {
            Expression::Literal(_) => true,
            Expression::Variable(_) => false,
            Expression::Property(_) => false,
            Expression::Binary { left, right, .. } => {
                self.can_evaluate(left, _ctx) && self.can_evaluate(right, _ctx)
            }
            // ...
        }
    }
}
```

**问题**:
- `EvaluableExprVisitor` 和 `ExpressionEvaluator::can_evaluate` 实现相同逻辑
- 代码重复
- 维护困难
- 容易出现不一致

---

### 问题 9.2: FoldConstantExprVisitor 实现不完整

**涉及文件**: `src/query/visitor/constant_folder.rs`

**当前实现**:
```rust
impl FoldConstantExprVisitor {
    pub fn fold(&mut self, expr: &Expression) -> Expression {
        self.reset();
        self.visit_expression(expr);
        self.result.take().unwrap_or_else(|| expr.clone())
    }
    
    fn fold_binary_expr(&mut self, left: &Expression, op: &BinaryOperator, right: &Expression) {
        // 只处理部分情况
        if let (Expression::Literal(lit_l), Expression::Literal(lit_r)) = (left, right) {
            if let Some(result) = self.evaluate_binary(lit_l, op, lit_r) {
                self.set_result(Expression::Literal(result));
                return;
            }
        }
        // 其他情况：保留原表达式
        self.set_result(Expression::Binary {
            left: Box::new(left.clone()),
            op: op.clone(),
            right: Box::new(right.clone()),
        });
    }
    
    fn evaluate_binary(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Option<Value> {
        match op {
            BinaryOperator::Add => Some(left.add(right)),
            BinaryOperator::Subtract => Some(left.subtract(right)),
            // ... 部分运算符
            _ => None,
        }
    }
}
```

**问题**:
- 只处理基本算术运算
- 不处理函数调用
- 不处理类型转换
- 不处理条件表达式

---

### 问题 9.3: 访问者之间缺乏代码共享

**当前实现**: 每个访问者独立实现

**问题**:
- 重复的遍历逻辑
- 重复的错误处理
- 难以维护

---

## 修改方案

### 修改方案 9.1: 明确 Visitor 和 Evaluator 的职责边界

**预估工作量**: 2-3 人天

**修改目标**:
- 消除功能重叠
- 明确职责边界
- 代码复用

**修改步骤**:

**步骤 1**: 定义职责边界

```rust
// src/query/visitor/traits.rs

/// Expression Visitor Trait
///
/// 用于静态分析和 AST 变换
/// - 不需要运行时上下文
/// - 返回分析结果
/// - 可以修改 AST
pub trait ExpressionVisitor {
    type Result;
    
    fn get_result(&self) -> Self::Result;
    fn reset(&mut self);
    fn is_success(&self) -> bool;
    
    // 访问方法
    fn visit_expression(&mut self, expr: &Expression);
    fn visit_literal(&mut self, literal: &Value);
    fn visit_variable(&mut self, name: &str);
    fn visit_binary_expr(&mut self, left: &Expression, op: &BinaryOperator, right: &Expression);
    fn visit_unary_expr(&mut self, op: &UnaryOperator, expr: &Expression);
    fn visit_property(&mut self, object: &Expression, property: &str);
    fn visit_function_call(&mut self, name: &str, args: &[Expression]);
    // ...
}

/// Expression Evaluator Trait
///
/// 用于运行时表达式求值
/// - 需要运行时上下文
/// - 返回求值结果
/// - 不修改 AST
#[async_trait]
pub trait ExpressionEvaluator {
    type Output;
    type Error;
    
    async fn evaluate(&self, expr: &Expression, ctx: &mut dyn ExpressionContext) 
        -> Result<Self::Output, Self::Error>;
    
    fn evaluate_literal(&self, literal: &Value) -> Self::Output;
    fn evaluate_variable(&self, name: &str, ctx: &mut dyn ExpressionContext) 
        -> Result<Self::Output, Self::Error>;
    fn evaluate_binary(&self, left: &Self::Output, op: &BinaryOperator, right: &Self::Output) 
        -> Result<Self::Output, Self::Error>;
    // ...
}
```

**步骤 2**: 统一可求值性检查

```rust
// src/query/visitor/evaluable.rs

use crate::query::visitor::traits::ExpressionVisitor;

/// 表达式可求值性检查器
pub struct EvaluableExprVisitor {
    is_evaluable: bool,
    unknown_variables: Vec<String>,
}

impl Default for EvaluableExprVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl EvaluableExprVisitor {
    pub fn new() -> Self {
        Self {
            is_evaluable: true,
            unknown_variables: Vec::new(),
        }
    }
    
    pub fn reset(&mut self) {
        self.is_evaluable = true;
        self.unknown_variables.clear();
    }
    
    pub fn get_result(&self) -> bool {
        self.is_evaluable
    }
    
    pub fn is_success(&self) -> bool {
        self.is_evaluable && self.unknown_variables.is_empty()
    }
    
    pub fn is_evaluable(&self) -> bool {
        self.is_evaluable && self.unknown_variables.is_empty()
    }
    
    pub fn unknown_variables(&self) -> &[String] {
        &self.unknown_variables
    }
}

impl ExpressionVisitor for EvaluableExprVisitor {
    type Result = bool;
    
    fn get_result(&self) -> Self::Result {
        self.is_evaluable()
    }
    
    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Literal(_) => {}
            Expression::Variable(name) => {
                // 假设变量在上下文中不可用
                self.is_evaluable = false;
                self.unknown_variables.push(name.clone());
            }
            Expression::Property { object, .. } => {
                self.visit_expression(object);
            }
            Expression::Binary { left, right, .. } => {
                self.visit_expression(left);
                self.visit_expression(right);
            }
            Expression::Unary { expr, .. } => {
                self.visit_expression(expr);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.visit_expression(arg);
                }
            }
            Expression::List { items } => {
                for item in items {
                    self.visit_expression(item);
                }
            }
            _ => {}
        }
    }
    
    fn visit_literal(&mut self, _literal: &Value) {}
    
    fn visit_variable(&mut self, name: &str) {
        self.is_evaluable = false;
        self.unknown_variables.push(name.to_string());
    }
    
    fn visit_binary_expr(&mut self, left: &Expression, _op: &BinaryOperator, right: &Expression) {
        self.visit_expression(left);
        self.visit_expression(right);
    }
    
    fn visit_unary_expr(&mut self, _op: &UnaryOperator, expr: &Expression) {
        self.visit_expression(expr);
    }
    
    fn visit_property(&mut self, object: &Expression, _property: &str) {
        self.visit_expression(object);
    }
    
    fn visit_function_call(&mut self, _name: &str, args: &[Expression]) {
        for arg in args {
            self.visit_expression(arg);
        }
    }
}

/// 检查表达式是否可以在运行时求值
pub fn can_evaluate_statically(expr: &Expression) -> bool {
    let mut visitor = EvaluableExprVisitor::new();
    visitor.visit_expression(expr);
    visitor.is_evaluable()
}

/// 检查表达式是否只引用已知变量
pub fn check_variable_references(
    expr: &Expression,
    known_variables: &HashSet<String>,
) -> Result<(), Vec<String>> {
    let mut visitor = VariableReferenceChecker::new(known_variables);
    visitor.visit_expression(expr);
    
    if visitor.is_success() {
        Ok(())
    } else {
        Err(visitor.unknown_variables().clone())
    }
}
```

---

### 修改方案 9.2: 完善常量折叠

**预估工作量**: 3-4 人天

**修改代码**:

```rust
// src/query/visitor/constant_folder.rs

use crate::query::visitor::traits::ExpressionVisitor;

/// 常量折叠访问者
pub struct FoldConstantExprVisitor {
    result: Option<Expression>,
    errors: Vec<String>,
}

impl Default for FoldConstantExprVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl FoldConstantExprVisitor {
    pub fn new() -> Self {
        Self {
            result: None,
            errors: Vec::new(),
        }
    }
    
    pub fn reset(&mut self) {
        self.result = None;
        self.errors.clear();
    }
    
    pub fn fold(&mut self, expr: &Expression) -> Expression {
        self.reset();
        self.visit_expression(expr);
        self.result.take().unwrap_or_else(|| expr.clone())
    }
    
    pub fn get_result(&self) -> Option<&Expression> {
        self.result.as_ref()
    }
    
    pub fn is_success(&self) -> bool {
        self.result.is_some()
    }
    
    pub fn errors(&self) -> &[String] {
        &self.errors
    }
}

impl ExpressionVisitor for FoldConstantExprVisitor {
    type Result = Expression;
    
    fn get_result(&self) -> Self::Result {
        self.result.clone().unwrap_or_else(|| Expression::null())
    }
    
    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Literal(value) => {
                self.set_result(Expression::Literal(value.clone()));
            }
            
            Expression::Variable(_) => {
                // 变量不能折叠，保留原表达式
                self.set_result(expr.clone());
            }
            
            Expression::Property { object, property } => {
                // 尝试折叠对象，但保留属性访问
                self.visit_expression(object);
                if let Some(obj) = self.result.take() {
                    self.set_result(Expression::Property {
                        object: Box::new(obj),
                        property: property.clone(),
                    });
                } else {
                    self.set_result(expr.clone());
                }
            }
            
            Expression::Binary { left, op, right } => {
                self.fold_binary_expr(left, op, right);
            }
            
            Expression::Unary { op, expr } => {
                self.fold_unary_expr(op, expr);
            }
            
            Expression::Function { name, args, distinct } => {
                self.fold_function_call(name, args, *distinct);
            }
            
            Expression::List { items } => {
                self.fold_list(items);
            }
            
            Expression::Case { test_expr, when_then_pairs, default } => {
                self.fold_case(test_expr, when_then_pairs, default);
            }
            
            _ => {
                self.set_result(expr.clone());
            }
        }
    }
    
    fn visit_literal(&mut self, literal: &Value) {
        self.set_result(Expression::Literal(literal.clone()));
    }
    
    fn visit_variable(&mut self, name: &str) {
        // 变量不能折叠
        self.set_result(Expression::Variable(name.to_string()));
    }
    
    fn visit_binary_expr(&mut self, left: &Expression, op: &BinaryOperator, right: &Expression) {
        self.fold_binary_expr(left, op, right);
    }
    
    fn visit_unary_expr(&mut self, op: &UnaryOperator, expr: &Expression) {
        self.fold_unary_expr(op, expr);
    }
    
    fn visit_property(&mut self, object: &Expression, property: &str) {
        self.visit_expression(object);
        if let Some(obj) = self.result.take() {
            self.set_result(Expression::Property {
                object: Box::new(obj),
                property: property.to_string(),
            });
        }
    }
    
    fn visit_function_call(&mut self, name: &str, args: &[Expression]) {
        self.fold_function_call(name, args, false);
    }
}

impl FoldConstantExprVisitor {
    fn set_result(&mut self, expr: Expression) {
        self.result = Some(expr);
    }
    
    fn fold_binary_expr(&mut self, left: &Expression, op: &BinaryOperator, right: &Expression) {
        // 递归折叠操作数
        self.visit_expression(left);
        let folded_left = self.result.take();
        
        self.visit_expression(right);
        let folded_right = self.result.take();
        
        // 如果两个操作数都是常量，进行计算
        if let (Some(Expression::Literal(lit_l)), Some(Expression::Literal(lit_r))) = 
            (folded_left, folded_right) {
            if let Some(result) = self.evaluate_binary(&lit_l, op, &lit_r) {
                self.set_result(Expression::Literal(result));
                return;
            }
        }
        
        // 构造新的二元表达式
        let left_expr = Box::new(folded_left.unwrap_or_else(|| left.clone()));
        let right_expr = Box::new(folded_right.unwrap_or_else(|| right.clone()));
        
        self.set_result(Expression::Binary {
            left: left_expr,
            op: op.clone(),
            right: right_expr,
        });
    }
    
    fn evaluate_binary(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Option<Value> {
        match op {
            // 算术运算
            BinaryOperator::Add => Some(left.add(right)),
            BinaryOperator::Subtract => Some(left.subtract(right)),
            BinaryOperator::Multiply => Some(left.multiply(right)),
            BinaryOperator::Divide => {
                if right.is_zero() {
                    None
                } else {
                    Some(left.divide(right))
                }
            }
            BinaryOperator::Modulo => Some(left.modulo(right)),
            BinaryOperator::Power => Some(left.power(right)),
            
            // 比较运算
            BinaryOperator::Equal => Some(left.equal(right)),
            BinaryOperator::NotEqual => Some(left.not_equal(right)),
            BinaryOperator::LessThan => Some(left.less_than(right)),
            BinaryOperator::GreaterThan => Some(left.greater_than(right)),
            BinaryOperator::LessEqual => Some(left.less_equal(right)),
            BinaryOperator::GreaterEqual => Some(left.greater_equal(right)),
            
            // 逻辑运算
            BinaryOperator::And => Some(left.and(right)),
            BinaryOperator::Or => Some(left.or(right)),
            
            // 字符串运算
            BinaryOperator::Contains => Some(left.contains(right)),
            BinaryOperator::StartsWith => Some(left.starts_with(right)),
            BinaryOperator::EndsWith => Some(left.ends_with(right)),
            
            _ => None,
        }
    }
    
    fn fold_unary_expr(&mut self, op: &UnaryOperator, expr: &Expression) {
        self.visit_expression(expr);
        let folded_expr = self.result.take();
        
        if let Some(Expression::Literal(lit)) = folded_expr {
            if let Some(result) = self.evaluate_unary(op, &lit) {
                self.set_result(Expression::Literal(result));
                return;
            }
        }
        
        self.set_result(Expression::Unary {
            op: op.clone(),
            expr: Box::new(folded_expr.unwrap_or_else(|| expr.clone())),
        });
    }
    
    fn evaluate_unary(&self, op: &UnaryOperator, value: &Value) -> Option<Value> {
        match op {
            UnaryOperator::Minus => value.negate(),
            UnaryOperator::Not => value.not(),
            UnaryOperator::IsNull => Some(Value::Bool(value.is_null())),
            UnaryOperator::IsNotNull => Some(Value::Bool(!value.is_null())),
            _ => None,
        }
    }
    
    fn fold_function_call(&mut self, name: &str, args: &[Expression], _distinct: bool) {
        // 递归折叠参数
        let mut folded_args = Vec::new();
        let mut all_literal = true;
        
        for arg in args {
            self.visit_expression(arg);
            if let Some(expr) = self.result.take() {
                if !matches!(expr, Expression::Literal(_)) {
                    all_literal = false;
                }
                folded_args.push(expr);
            } else {
                folded_args.push(arg.clone());
                all_literal = false;
            }
        }
        
        // 如果所有参数都是常量，尝试计算函数结果
        if all_literal {
            if let Some(result) = self.evaluate_function(name, &folded_args) {
                self.set_result(Expression::Literal(result));
                return;
            }
        }
        
        self.set_result(Expression::Function {
            name: name.to_string(),
            args: folded_args,
            distinct: _distinct,
        });
    }
    
    fn evaluate_function(&self, name: &str, args: &[Expression]) -> Option<Value> {
        let arg_values: Vec<Value> = args.iter()
            .filter_map(|a| {
                if let Expression::Literal(v) = a {
                    Some(v.clone())
                } else {
                    None
                }
            })
            .collect();
        
        match name.to_uppercase().            "ABS"as_str() {
 => {
                if let Some(Value::Int(n)) = arg_values.get(0) {
                    Some(Value::Int(n.abs()))
                } else {
                    None
                }
            }
            "LENGTH" | "SIZE" => {
                if let Some(Value::String(s)) = arg_values.get(0) {
                    Some(Value::Int(s.len() as i64))
                } else if let Some(Value::List(list)) = arg_values.get(0) {
                    Some(Value::Int(list.len() as i64))
                } else {
                    None
                }
            }
            "TO_STRING" | "STRING" => {
                if let Some(v) = arg_values.get(0) {
                    Some(Value::String(v.to_string()))
                } else {
                    None
                }
            }
            "COALESCE" => {
                for arg in &arg_values {
                    if !arg.is_null() {
                        return Some(arg.clone());
                    }
                }
                Some(Value::Null)
            }
            _ => None,
        }
    }
    
    fn fold_list(&mut self, items: &[Expression]) {
        let mut folded_items = Vec::new();
        let mut all_literal = true;
        
        for item in items {
            self.visit_expression(item);
            if let Some(expr) = self.result.take() {
                if !matches!(expr, Expression::Literal(_)) {
                    all_literal = false;
                }
                folded_items.push(expr);
            } else {
                folded_items.push(item.clone());
                all_literal = false;
            }
        }
        
        if all_literal {
            let values: Vec<Value> = folded_items
                .iter()
                .filter_map(|e| {
                    if let Expression::Literal(v) = e {
                        Some(v.clone())
                    } else {
                        None
                    }
                })
                .collect();
            self.set_result(Expression::Literal(Value::List(values)));
        } else {
            self.set_result(Expression::List { items: folded_items });
        }
    }
    
    fn fold_case(
        &mut self,
        test_expr: &Option<Box<Expression>>,
        when_then_pairs: &[(Expression, Expression)],
        default: &Option<Box<Expression>>,
    ) {
        // 尝试求值 CASE 表达式
        if test_expr.is_none() && !when_then_pairs.is_empty() {
            // 简单 CASE：WHEN expr THEN result
            // 尝试找到第一个匹配的条件
            for (when_expr, then_expr) in when_then_pairs {
                self.visit_expression(when_expr);
                let when_result = self.result.take();
                
                // 尝试将条件与常量比较
                if let Some(Expression::Literal(Value::Bool(true))) = when_result {
                    // 条件恒真，使用这个 THEN 分支
                    self.visit_expression(then_expr);
                    return;
                }
            }
        }
        
        // 无法折叠，构造新的 CASE 表达式
        let folded_test = test_expr.as_ref().map(|e| {
            self.visit_expression(e);
            Box::new(self.result.take().unwrap_or_else(|| *e.clone()))
        });
        
        let folded_pairs = when_then_pairs.iter()
            .map(|(w, t)| {
                self.visit_expression(w);
                let folded_w = self.result.take().unwrap_or_else(|| w.clone());
                self.visit_expression(t);
                let folded_t = self.result.take().unwrap_or_else(|| t.clone());
                (folded_w, folded_t)
            })
            .collect();
        
        let folded_default = default.as_ref().map(|d| {
            self.visit_expression(d);
            Box::new(self.result.take().unwrap_or_else(|| *d.clone()))
        });
        
        self.set_result(Expression::Case {
            test_expr: folded_test,
            when_then_pairs: folded_pairs,
            default: folded_default,
        });
    }
}
```

---

### 修改方案 9.3: 创建访问者基类

**预估工作量**: 1 人天

**修改代码**:

```rust
// src/query/visitor/base.rs

use crate::query::ast::Expression;

/// 基础访问者实现
///
/// 提供通用的遍历逻辑
pub trait BaseVisitor {
    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Literal(l) => self.visit_literal(l),
            Expression::Variable(v) => self.visit_variable(v),
            Expression::Property(p) => self.visit_property(&p.object, &p.property),
            Expression::Binary(b) => {
                self.visit_expression(&b.left);
                self.visit_binary_expr(&b.left, &b.op, &b.right);
                self.visit_expression(&b.right);
            }
            Expression::Unary(u) => {
                self.visit_expression(&u.expr);
                self.visit_unary_expr(&u.op, &u.expr);
            }
            Expression::Function(f) => {
                for arg in &f.args {
                    self.visit_expression(arg);
                }
                self.visit_function_call(&f.name, &f.args);
            }
            Expression::List(l) => {
                for item in &l.items {
                    self.visit_expression(item);
                }
            }
            Expression::Case(c) => {
                if let Some(te) = &c.test_expr {
                    self.visit_expression(te);
                }
                for (w, t) in &c.when_then_pairs {
                    self.visit_expression(w);
                    self.visit_expression(t);
                }
                if let Some(d) = &c.default {
                    self.visit_expression(d);
                }
            }
            Expression::SubQuery(s) => {
                for stmt in &s.statements {
                    self.visit_statement(stmt);
                }
            }
            _ => {}
        }
    }
    
    fn visit_literal(&mut self, _literal: &Value) {}
    fn visit_variable(&mut self, _name: &str) {}
    fn visit_property(&mut self, _object: &Expression, _property: &str) {}
    fn visit_binary_expr(&mut self, _left: &Expression, _op: &BinaryOperator, _right: &Expression) {}
    fn visit_unary_expr(&mut self, _op: &UnaryOperator, _expr: &Expression) {}
    fn visit_function_call(&mut self, _name: &str, _args: &[Expression]) {}
    fn visit_statement(&mut self, _stmt: &Stmt) {}
}
```

---

## 修改优先级

| 序号 | 修改方案 | 优先级 | 预估工作量 | 依赖 |
|------|----------|--------|------------|------|
| 9.1 | 明确 Visitor 和 Evaluator 职责 | 高 | 2-3 人天 | 无 |
| 9.2 | 完善常量折叠 | 中 | 3-4 人天 | 9.1 |
| 9.3 | 创建访问者基类 | 低 | 1 人天 | 无 |

---

## 测试建议

### 测试用例 1: 常量折叠

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_fold_arithmetic() {
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Int(2))),
        };
        
        let mut folder = FoldConstantExprVisitor::new();
        let result = folder.fold(&expr);
        
        assert!(matches!(result, Expression::Literal(Value::Int(3))));
    }
    
    #[test]
    fn test_fold_with_variable() {
        let expr = Expression::Binary {
            left: Box::new(Expression::Variable("n".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Int(1))),
        };
        
        let mut folder = FoldConstantExprVisitor::new();
        let result = folder.fold(&expr);
        
        // 变量不能折叠，结果应该包含变量
        match &result {
            Expression::Binary { left, op: _, right } => {
                assert!(matches!(*left, Expression::Variable(_)));
            }
            _ => panic!("Expected Binary expression"),
        }
    }
    
    #[test]
    fn test_fold_function() {
        let expr = Expression::Function {
            name: "ABS".to_string(),
            args: vec![Expression::Literal(Value::Int(-5))],
            distinct: false,
        };
        
        let mut folder = FoldConstantExprVisitor::new();
        let result = folder.fold(&expr);
        
        assert!(matches!(result, Expression::Literal(Value::Int(5))));
    }
    
    #[test]
    fn test_evaluable_check() {
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Int(2))),
        };
        
        assert!(can_evaluate_statically(&expr));
        
        let expr_with_var = Expression::Binary {
            left: Box::new(Expression::Variable("n".to_string())),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Int(1))),
        };
        
        assert!(!can_evaluate_statically(&expr_with_var));
    }
}
```

---

## 风险与注意事项

### 风险 1: 常量折叠边界情况

- **风险**: 除零、溢出等边界情况
- **缓解措施**: 充分的边界测试
- **实现**: 在 evaluate_binary 中检查边界

### 风险 2: 函数求值安全性

- **风险**: 某些函数可能有副作用
- **缓解措施**: 只折叠无副作用函数
- **实现**: 白名单机制，只折叠已知安全函数
