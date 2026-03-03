# Visitor模式后续重构方案

## 概述

Visitor模块已经成功实现并完成类型检查。本文档详细说明了后续的重构方案，包括需要重构的文件、重构步骤以及如何使用新的visitor类来替换现有的模式匹配代码。

## Visitor模块实现总结

### 已完成的模块

1. **visitor.rs** - 定义了`ExpressionVisitor` trait，包含所有表达式类型的访问方法
2. **visitor_collectors.rs** - 实现了三个收集器：
   - `PropertyCollector` - 收集表达式中所有使用的属性名
   - `VariableCollector` - 收集表达式中所有使用的变量名
   - `FunctionCollector` - 收集表达式中所有使用的函数名
3. **visitor_checkers.rs** - 实现了两个检查器：
   - `ConstantChecker` - 检查表达式是否为常量表达式
   - `PropertyContainsChecker` - 检查表达式是否包含指定的属性名

### 类型检查结果

- ✅ visitor模块所有文件通过类型检查
- ⚠️ 存在3个错误（与visitor模块无关，位于`return_clause_planner.rs`中的`generate_default_alias`函数未找到）

## 需要重构的文件

### 1. 表达式工具类

**文件**: `src/query/planner/rewrite/expression_utils.rs`

**当前问题**: 使用大量的模式匹配代码来分析表达式

**重构方案**:

#### 1.1 收集属性名

**当前代码模式**:
```rust
fn collect_properties(expr: &Expression) -> Vec<String> {
    match expr {
        Expression::Property { object, property } => {
            let mut props = vec![property.clone()];
            props.extend(collect_properties(object));
            props
        }
        Expression::Binary { left, right, .. } => {
            let mut props = collect_properties(left);
            props.extend(collect_properties(right));
            props
        }
        // ... 其他模式
    }
}
```

**重构后**:
```rust
use crate::core::types::expression::visitor::PropertyCollector;

fn collect_properties(expr: &Expression) -> Vec<String> {
    let mut collector = PropertyCollector::new();
    collector.visit(expr);
    collector.properties
}
```

#### 1.2 收集变量名

**当前代码模式**:
```rust
fn collect_variables(expr: &Expression) -> Vec<String> {
    match expr {
        Expression::Variable(name) => vec![name.clone()],
        Expression::Binary { left, right, .. } => {
            let mut vars = collect_variables(left);
            vars.extend(collect_variables(right));
            vars
        }
        // ... 其他模式
    }
}
```

**重构后**:
```rust
use crate::core::types::expression::visitor::VariableCollector;

fn collect_variables(expr: &Expression) -> Vec<String> {
    let mut collector = VariableCollector::new();
    collector.visit(expr);
    collector.variables
}
```

#### 1.3 检查常量表达式

**当前代码模式**:
```rust
fn is_constant(expr: &Expression) -> bool {
    match expr {
        Expression::Literal(_) => true,
        Expression::Variable(_) => false,
        Expression::Property { .. } => false,
        Expression::Binary { left, right, .. } => {
            is_constant(left) && is_constant(right)
        }
        // ... 其他模式
    }
}
```

**重构后**:
```rust
use crate::core::types::expression::visitor::ConstantChecker;

fn is_constant(expr: &Expression) -> bool {
    ConstantChecker::check(expr)
}
```

### 2. 过滤下推遍历

**文件**: `src/query/planner/rewrite/push_filter_down_traverse.rs`

**当前问题**: 使用模式匹配来分析表达式中的属性和变量

**重构方案**:

#### 2.1 检查表达式是否包含特定属性

**当前代码模式**:
```rust
fn contains_property(expr: &Expression, property: &str) -> bool {
    match expr {
        Expression::Property { object, property: p } => {
            p == property || contains_property(object, property)
        }
        Expression::Binary { left, right, .. } => {
            contains_property(left, property) || contains_property(right, property)
        }
        // ... 其他模式
    }
}
```

**重构后**:
```rust
use crate::core::types::expression::visitor::PropertyContainsChecker;

fn contains_property(expr: &Expression, property: &str) -> bool {
    PropertyContainsChecker::check(expr, &[property.to_string()])
}
```

### 3. 其他重写规则文件

以下文件可能包含类似的表达式分析代码，需要进行重构：

- `src/query/planner/rewrite/predicate_push_down.rs`
- `src/query/planner/rewrite/project_pruning.rs`
- `src/query/planner/rewrite/limit_push_down.rs`
- `src/query/planner/rewrite/combine_filter.rs`

## 重构步骤

### 阶段1: 准备工作

1. ✅ 完成visitor模块的实现
2. ✅ 通过类型检查
3. ⏳ 修复`return_clause_planner.rs`中的`generate_default_alias`错误

### 阶段2: 重构expression_utils.rs

1. 导入visitor模块:
   ```rust
   use crate::core::types::expression::visitor::{
       PropertyCollector, VariableCollector, ConstantChecker, PropertyContainsChecker
   };
   ```

2. 替换`collect_properties`函数
3. 替换`collect_variables`函数
4. 替换`is_constant`函数
5. 替换其他类似的函数

### 阶段3: 重构push_filter_down_traverse.rs

1. 导入visitor模块
2. 替换属性检查相关函数
3. 运行测试验证

### 阶段4: 重构其他重写规则文件

1. 逐个文件分析需要重构的函数
2. 使用visitor模式替换模式匹配代码
3. 运行测试验证

### 阶段5: 清理和优化

1. 删除不再使用的辅助函数
2. 优化代码结构
3. 添加文档注释
4. 运行完整测试套件

## 重构收益

### 代码质量提升

1. **减少重复代码**: 预计减少约75%的模式匹配代码
2. **提高可维护性**: 集中管理表达式遍历逻辑
3. **增强可扩展性**: 添加新的分析器只需实现`ExpressionVisitor` trait

### 性能优化

1. **减少递归调用**: visitor模式优化了遍历过程
2. **提前终止**: 检查器可以在找到目标后立即终止遍历
3. **内存效率**: 避免创建不必要的中间数据结构

### 开发效率

1. **简化新功能开发**: 添加新的表达式分析器更加简单
2. **降低错误率**: 减少手动编写模式匹配代码的错误
3. **提高代码可读性**: 使用语义化的收集器和检查器名称

## 注意事项

### 1. 保持向后兼容

在重构过程中，确保不改变公共API的接口，避免影响其他模块。

### 2. 测试覆盖

每次重构后，运行相关测试确保功能正确性：
```bash
cargo test --test <test_file>
```

### 3. 性能监控

重构后，监控查询性能，确保没有性能退化：
```bash
cargo bench
```

### 4. 代码审查

重构完成后，进行代码审查，确保代码质量和一致性。

## 示例：完整的重构示例

### 重构前

```rust
// src/query/planner/rewrite/expression_utils.rs

pub fn collect_properties(expr: &Expression) -> Vec<String> {
    match expr {
        Expression::Literal(_) => vec![],
        Expression::Variable(_) => vec![],
        Expression::Property { object, property } => {
            let mut props = vec![property.clone()];
            props.extend(collect_properties(object));
            props
        }
        Expression::Binary { left, right, .. } => {
            let mut props = collect_properties(left);
            props.extend(collect_properties(right));
            props
        }
        Expression::Unary { operand, .. } => collect_properties(operand),
        Expression::Function { args, .. } => {
            args.iter()
                .flat_map(|arg| collect_properties(arg))
                .collect()
        }
        Expression::Aggregate { arg, .. } => collect_properties(arg),
        Expression::Case { conditions, default, .. } => {
            let mut props = vec![];
            for (when, then) in conditions {
                props.extend(collect_properties(when));
                props.extend(collect_properties(then));
            }
            if let Some(default_expr) = default {
                props.extend(collect_properties(default_expr));
            }
            props
        }
        Expression::List(items) => {
            items.iter()
                .flat_map(|item| collect_properties(item))
                .collect()
        }
        Expression::Map(entries) => {
            entries.iter()
                .flat_map(|(_, value)| collect_properties(value))
                .collect()
        }
        Expression::TypeCast { expression, .. } => collect_properties(expression),
        Expression::Subscript { collection, index } => {
            let mut props = collect_properties(collection);
            props.extend(collect_properties(index));
            props
        }
        Expression::Range { collection, start, end } => {
            let mut props = collect_properties(collection);
            if let Some(start_expr) = start {
                props.extend(collect_properties(start_expr));
            }
            if let Some(end_expr) = end {
                props.extend(collect_properties(end_expr));
            }
            props
        }
        Expression::Path(items) => {
            items.iter()
                .flat_map(|item| collect_properties(item))
                .collect()
        }
        Expression::Label(_) => vec![],
        Expression::ListComprehension { source, filter, map, .. } => {
            let mut props = collect_properties(source);
            if let Some(filter_expr) = filter {
                props.extend(collect_properties(filter_expr));
            }
            if let Some(map_expr) = map {
                props.extend(collect_properties(map_expr));
            }
            props
        }
        Expression::LabelTagProperty { tag, .. } => collect_properties(tag),
        Expression::TagProperty { .. } => vec![],
        Expression::EdgeProperty { .. } => vec![],
        Expression::Predicate { args, .. } => {
            args.iter()
                .flat_map(|arg| collect_properties(arg))
                .collect()
        }
        Expression::Reduce { initial, source, mapping, .. } => {
            let mut props = collect_properties(initial);
            props.extend(collect_properties(source));
            props.extend(collect_properties(mapping));
            props
        }
        Expression::PathBuild(items) => {
            items.iter()
                .flat_map(|item| collect_properties(item))
                .collect()
        }
        Expression::Parameter(_) => vec![],
    }
}
```

### 重构后

```rust
// src/query/planner/rewrite/expression_utils.rs

use crate::core::types::expression::visitor::PropertyCollector;

pub fn collect_properties(expr: &Expression) -> Vec<String> {
    let mut collector = PropertyCollector::new();
    collector.visit(expr);
    collector.properties
}
```

**代码行数**: 从约60行减少到4行，减少约93%

## 总结

Visitor模块的实现为表达式分析提供了统一、高效的基础设施。通过按照本文档的重构方案，可以显著提高代码质量、减少重复代码、提升开发效率。重构过程应该分阶段进行，确保每个阶段都经过充分测试，避免引入新的错误。

## 下一步行动

1. 修复`return_clause_planner.rs`中的`generate_default_alias`错误
2. 开始重构`expression_utils.rs`文件
3. 逐步重构其他重写规则文件
4. 运行完整测试套件验证重构结果
