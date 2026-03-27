# Expression 内存估算设计方案

## 1. 概述

本文档分析如何为 `Expression` 类型设计内存估算方案，包括设计原则、实现策略和潜在问题。

## 2. Expression 类型结构分析

### 2.1 Expression 枚举定义

```rust
pub enum Expression {
    // 基础类型
    Literal(Value),
    Variable(String),
    Label(String),
    Parameter(String),
    
    // 属性访问
    Property { object: Box<Expression>, property: String },
    TagProperty { tag_name: String, property: String },
    EdgeProperty { edge_name: String, property: String },
    LabelTagProperty { tag: Box<Expression>, property: String },
    
    // 运算表达式
    Binary { left: Box<Expression>, op: BinaryOperator, right: Box<Expression> },
    Unary { op: UnaryOperator, operand: Box<Expression> },
    
    // 函数调用
    Function { name: String, args: Vec<Expression> },
    Aggregate { func: AggregateFunction, arg: Box<Expression>, distinct: bool },
    Predicate { func: String, args: Vec<Expression> },
    
    // 集合类型
    List(Vec<Expression>),
    Map(Vec<(String, Expression)>),
    Path(Vec<Expression>),
    PathBuild(Vec<Expression>),
    
    // 条件表达式
    Case { 
        test_expr: Option<Box<Expression>>, 
        conditions: Vec<(Expression, Expression)>, 
        default: Option<Box<Expression>> 
    },
    
    // 访问表达式
    TypeCast { expression: Box<Expression>, target_type: DataType },
    Subscript { collection: Box<Expression>, index: Box<Expression> },
    Range { collection: Box<Expression>, start: Option<Box<Expression>>, end: Option<Box<Expression>> },
    
    // 高级表达式
    ListComprehension { 
        variable: String, 
        source: Box<Expression>, 
        filter: Option<Box<Expression>>, 
        map: Option<Box<Expression>> 
    },
    Reduce { 
        accumulator: String, 
        initial: Box<Expression>, 
        variable: String, 
        source: Box<Expression>, 
        mapping: Box<Expression> 
    },
}
```

### 2.2 类型分类

| 类别 | 包含变体 | 特点 |
|------|----------|------|
| 叶子节点 | Literal, Variable, Label, Parameter | 无嵌套表达式 |
| 单目运算 | Unary, TypeCast, Aggregate | 包含一个子表达式 |
| 双目运算 | Binary, Subscript | 包含两个子表达式 |
| 多目运算 | Function, Predicate, List, Map | 包含多个子表达式 |
| 条件运算 | Case | 复杂的条件分支 |
| 属性访问 | Property, TagProperty, EdgeProperty | 包含对象和属性名 |

## 3. 设计原则

### 3.1 核心原则

1. **递归估算**：Expression 是树形结构，需要递归计算所有子表达式
2. **精确与性能平衡**：在估算精度和计算开销之间取得平衡
3. **避免循环引用**：虽然 Expression 理论上无循环，但需防御性编程
4. **统一接口**：与 PlanNode 使用相同的 `MemoryEstimatable` trait

### 3.2 内存计算要素

```
Expression 内存 = 枚举标签大小 + 变体数据大小

其中：
- 枚举标签大小：通常 1 byte（ discriminant）+ 对齐填充
- 变体数据大小：取决于具体变体
```

## 4. 实现方案

### 4.1 方案一：基础递归实现（推荐）

```rust
use crate::core::types::expr::Expression;
use crate::core::value::Value;
use crate::query::planning::plan::core::nodes::base::memory_estimation::{
    estimate_string_memory, estimate_option_box_memory
};

impl MemoryEstimatable for Expression {
    fn estimate_memory(&self) -> usize {
        let base_size = std::mem::size_of::<Expression>();
        
        match self {
            // 叶子节点：基础大小 + 数据大小
            Expression::Literal(value) => base_size + estimate_value_memory(value),
            Expression::Variable(name) => base_size + estimate_string_memory(name),
            Expression::Label(name) => base_size + estimate_string_memory(name),
            Expression::Parameter(name) => base_size + estimate_string_memory(name),
            
            // 单目运算：基础大小 + 子表达式
            Expression::Unary { operand, .. } => {
                base_size + operand.estimate_memory()
            }
            Expression::TypeCast { expression, .. } => {
                base_size + expression.estimate_memory()
            }
            Expression::Aggregate { arg, .. } => {
                base_size + arg.estimate_memory()
            }
            
            // 双目运算：基础大小 + 两个子表达式
            Expression::Binary { left, right, .. } => {
                base_size + left.estimate_memory() + right.estimate_memory()
            }
            Expression::Subscript { collection, index } => {
                base_size + collection.estimate_memory() + index.estimate_memory()
            }
            
            // 属性访问
            Expression::Property { object, property } => {
                base_size + object.estimate_memory() + estimate_string_memory(property)
            }
            Expression::TagProperty { tag_name, property } => {
                base_size + estimate_string_memory(tag_name) + estimate_string_memory(property)
            }
            Expression::EdgeProperty { edge_name, property } => {
                base_size + estimate_string_memory(edge_name) + estimate_string_memory(property)
            }
            Expression::LabelTagProperty { tag, property } => {
                base_size + tag.estimate_memory() + estimate_string_memory(property)
            }
            
            // 集合类型
            Expression::List(items) => {
                base_size + items.iter().map(|e| e.estimate_memory()).sum::<usize>()
            }
            Expression::Map(entries) => {
                base_size + entries.iter()
                    .map(|(k, v)| estimate_string_memory(k) + v.estimate_memory())
                    .sum::<usize>()
            }
            Expression::Path(items) => {
                base_size + items.iter().map(|e| e.estimate_memory()).sum::<usize>()
            }
            Expression::PathBuild(items) => {
                base_size + items.iter().map(|e| e.estimate_memory()).sum::<usize>()
            }
            
            // 函数调用
            Expression::Function { name, args } => {
                base_size + estimate_string_memory(name)
                    + args.iter().map(|e| e.estimate_memory()).sum::<usize>()
            }
            Expression::Predicate { func, args } => {
                base_size + estimate_string_memory(func)
                    + args.iter().map(|e| e.estimate_memory()).sum::<usize>()
            }
            
            // 条件表达式
            Expression::Case { test_expr, conditions, default } => {
                let test_size = test_expr.as_ref().map(|e| e.estimate_memory()).unwrap_or(0);
                let conditions_size = conditions.iter()
                    .map(|(cond, result)| cond.estimate_memory() + result.estimate_memory())
                    .sum::<usize>();
                let default_size = default.as_ref().map(|e| e.estimate_memory()).unwrap_or(0);
                base_size + test_size + conditions_size + default_size
            }
            
            // 范围表达式
            Expression::Range { collection, start, end } => {
                let start_size = start.as_ref().map(|e| e.estimate_memory()).unwrap_or(0);
                let end_size = end.as_ref().map(|e| e.estimate_memory()).unwrap_or(0);
                base_size + collection.estimate_memory() + start_size + end_size
            }
            
            // 列表推导
            Expression::ListComprehension { variable, source, filter, map } => {
                let filter_size = filter.as_ref().map(|e| e.estimate_memory()).unwrap_or(0);
                let map_size = map.as_ref().map(|e| e.estimate_memory()).unwrap_or(0);
                base_size + estimate_string_memory(variable)
                    + source.estimate_memory() + filter_size + map_size
            }
            
            // Reduce 表达式
            Expression::Reduce { accumulator, initial, variable, source, mapping } => {
                base_size + estimate_string_memory(accumulator)
                    + estimate_string_memory(variable)
                    + initial.estimate_memory()
                    + source.estimate_memory()
                    + mapping.estimate_memory()
            }
        }
    }
}
```

### 4.2 方案二：带缓存的估算（复杂场景）

对于包含大量重复子表达式的场景，可以使用缓存优化：

```rust
use std::collections::HashMap;
use std::sync::Arc;

pub struct ExpressionMemoryEstimator {
    cache: HashMap<Arc<Expression>, usize>,
}

impl ExpressionMemoryEstimator {
    pub fn estimate(&mut self, expr: &Expression) -> usize {
        // 检查缓存（如果 Expression 使用 Arc）
        // ...
        
        let size = self.calculate(expr);
        // 存入缓存
        // ...
        size
    }
    
    fn calculate(&self, expr: &Expression) -> usize {
        // 递归计算，不缓存
        // ...
    }
}
```

**适用场景**：
- 查询优化器中的表达式复用
- 复杂查询包含大量相同子表达式

**缺点**：
- 增加复杂度
- 需要维护缓存生命周期

### 4.3 Value 类型内存估算

Expression::Literal 包含 Value 类型，需要单独实现：

```rust
impl MemoryEstimatable for Value {
    fn estimate_memory(&self) -> usize {
        let base_size = std::mem::size_of::<Value>();
        
        match self {
            // 固定大小类型
            Value::Empty | Value::Null(_) | Value::Bool(_) |
            Value::Int(_) | Value::Int8(_) | Value::Int16(_) | Value::Int32(_) | Value::Int64(_) |
            Value::UInt8(_) | Value::UInt16(_) | Value::UInt32(_) | Value::UInt64(_) |
            Value::Float(_) => base_size,
            
            // 变长类型
            Value::String(s) => base_size + s.capacity(),
            Value::FixedString { len, data } => base_size + *len,
            Value::Blob(b) => base_size + b.capacity(),
            
            // 复合类型（需要递归）
            Value::List(list) => {
                base_size + list.iter().map(|v| v.estimate_memory()).sum::<usize>()
            }
            Value::Map(map) => {
                base_size + map.iter()
                    .map(|(k, v)| k.capacity() + v.estimate_memory())
                    .sum::<usize>()
            }
            Value::Set(set) => {
                base_size + set.iter().map(|v| v.estimate_memory()).sum::<usize>()
            }
            
            // 图类型（简化估算）
            Value::Vertex(v) => base_size + std::mem::size_of_val(v.as_ref()),
            Value::Edge(e) => base_size + std::mem::size_of_val(e),
            Value::Path(p) => base_size + std::mem::size_of_val(p),
            
            // 其他类型
            Value::Decimal128(d) => base_size + std::mem::size_of_val(d),
            Value::Date(_) | Value::Time(_) | Value::DateTime(_) |
            Value::Duration(_) => base_size,
            Value::Geography(g) => base_size + std::mem::size_of_val(g),
            Value::DataSet(d) => base_size + std::mem::size_of_val(d),
        }
    }
}
```

## 5. 潜在问题与解决方案

### 5.1 问题一：递归深度

**问题**：
- 复杂表达式可能导致栈溢出
- 例如：嵌套的 Binary 表达式 `a + (b + (c + (d + ...)))`

**解决方案**：
```rust
impl Expression {
    /// 使用栈实现非递归内存估算
    pub fn estimate_memory_iterative(&self) -> usize {
        let mut total = 0usize;
        let mut stack = vec![self];
        
        while let Some(expr) = stack.pop() {
            total += std::mem::size_of::<Expression>();
            
            // 将子表达式压入栈
            match expr {
                Expression::Binary { left, right, .. } => {
                    stack.push(left);
                    stack.push(right);
                }
                Expression::Unary { operand, .. } => {
                    stack.push(operand);
                }
                // ... 其他变体
                _ => {}
            }
        }
        
        total
    }
}
```

### 5.2 问题二：循环引用

**问题**：
- 理论上 Expression 不应有循环引用
- 但防御性编程需要考虑

**解决方案**：
```rust
use std::collections::HashSet;

pub fn estimate_memory_safe(&self) -> usize {
    let mut visited = HashSet::new();
    self.estimate_memory_with_guard(&mut visited)
}

fn estimate_memory_with_guard(&self, visited: &mut HashSet<*const Expression>) -> usize {
    let ptr = self as *const Expression;
    if !visited.insert(ptr) {
        return 0; // 检测到循环，不再计算
    }
    
    // ... 正常计算
}
```

### 5.3 问题三：性能开销

**问题**：
- 每次缓存检查都进行完整递归估算，开销较大
- 对于简单表达式，估算成本可能超过收益

**解决方案**：
1. **阈值优化**：只对复杂表达式进行精确估算
```rust
impl MemoryEstimatable for Expression {
    fn estimate_memory(&self) -> usize {
        // 简单估算：只计算基础大小
        if self.is_simple() {
            return std::mem::size_of::<Expression>() * self.node_count();
        }
        
        // 复杂表达式：精确计算
        self.estimate_memory_precise()
    }
    
    fn is_simple(&self) -> bool {
        matches!(self, 
            Expression::Literal(_) | 
            Expression::Variable(_) | 
            Expression::Label(_) |
            Expression::Parameter(_)
        )
    }
}
```

2. **延迟估算**：只在需要时计算
```rust
pub struct CachedExpressionSize {
    expr: Expression,
    size: Cell<Option<usize>>,
}

impl CachedExpressionSize {
    pub fn estimate_memory(&self) -> usize {
        if let Some(size) = self.size.get() {
            return size;
        }
        let size = self.expr.estimate_memory();
        self.size.set(Some(size));
        size
    }
}
```

## 6. 实现建议

### 6.1 推荐实现步骤

1. **Phase 1：基础实现**
   - 实现 `MemoryEstimatable for Expression`
   - 实现 `MemoryEstimatable for Value`
   - 添加基础辅助函数

2. **Phase 2：优化**
   - 添加迭代版本（避免栈溢出）
   - 添加循环检测（防御性编程）
   - 性能优化（阈值判断）

3. **Phase 3：集成**
   - 在 PlanNode 中使用 Expression 估算
   - 更新缓存权重计算
   - 添加测试用例

### 6.2 代码组织建议

```
src/
├── core/
│   └── types/
│       └── expr/
│           └── memory_estimation.rs  # Expression 内存估算实现
├── query/
│   └── planning/
│       └── plan/
│           └── core/
│               └── nodes/
│                   └── base/
│                       └── memory_estimation.rs  # 已有，添加 Value 支持
```

### 6.3 测试策略

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_expression() {
        let expr = Expression::Variable("x".to_string());
        let size = expr.estimate_memory();
        assert!(size > 0);
    }
    
    #[test]
    fn test_nested_expression() {
        let expr = Expression::Binary {
            left: Box::new(Expression::Literal(Value::Int(1))),
            op: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Value::Int(2))),
        };
        let size = expr.estimate_memory();
        assert!(size > std::mem::size_of::<Expression>() * 3);
    }
    
    #[test]
    fn test_complex_expression() {
        let expr = Expression::Function {
            name: "sum".to_string(),
            args: vec![
                Expression::Variable("a".to_string()),
                Expression::Variable("b".to_string()),
                Expression::Variable("c".to_string()),
            ],
        };
        let size = expr.estimate_memory();
        // 验证包含所有参数的大小
        assert!(size > std::mem::size_of::<Expression>() * 4);
    }
}
```

## 7. 总结

Expression 内存估算的关键点：

1. **递归结构**：Expression 是树形结构，需要递归计算
2. **变体多样**：18 种变体，每种需要不同的计算逻辑
3. **Value 嵌套**：Literal 包含 Value，需要同时实现 Value 的估算
4. **性能考虑**：简单表达式直接估算，复杂表达式可优化
5. **安全性**：考虑栈溢出和循环引用的防御性处理

推荐采用**方案一（基础递归实现）**，它提供了良好的平衡：
- 实现简单，易于维护
- 精度足够满足缓存控制需求
- 性能开销在可接受范围内
