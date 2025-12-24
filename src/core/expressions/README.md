# 表达式求值上下文模块

## 模块概述

本模块提供表达式求值过程中的**运行时上下文管理**，包括变量绑定、函数注册、缓存管理和错误处理等功能。

## 与 src/expression 的区别

| 模块 | 职责 | 核心内容 |
|------|------|---------|
| `src/core/expressions` | **求值上下文**（Runtime Context） | 变量管理、函数注册、缓存、错误处理 |
| `src/expression` | **表达式类型和操作**（Type System & Operations） | Expression类型、访问者模式、聚合函数、存储层接口 |

**重要**：这两个模块虽然名称相似，但职责完全不同，**不应合并**。

## 模块结构

```
expressions/
├── mod.rs              # 模块入口，重新导出公共API
├── basic_context.rs    # 基础表达式上下文实现
├── default_context.rs # 默认表达式上下文实现
├── cache.rs            # 表达式缓存管理器
├── functions.rs        # 函数定义和实现
├── error.rs            # 错误定义和处理
└── evaluation.rs       # 求值选项和统计信息
```

## 核心组件

### 1. 表达式上下文（ExpressionContext）

定义在 `src/core/evaluator/traits.rs`，本模块提供具体实现：

```rust
pub trait ExpressionContext {
    fn get_variable(&self, name: &str) -> Option<Value>;
    fn set_variable(&mut self, name: String, value: Value);
    fn get_vertex(&self) -> Option<&Vertex>;
    fn get_edge(&self) -> Option<&Edge>;
    // ...
}
```

### 2. 基础上下文（BasicExpressionContext）

提供完整的上下文功能：
- 变量绑定和查找（支持嵌套作用域）
- 函数注册和调用
- 缓存管理
- 层次化上下文支持

### 3. 默认上下文（DefaultExpressionContext）

轻量级上下文实现，适用于简单场景：
- 基本变量管理
- 顶点/边/路径访问
- 无函数注册功能

### 4. 缓存管理器（ExpressionCacheManager）

提供三级缓存：
- 函数执行结果缓存
- 表达式解析结果缓存
- 变量查找缓存

### 5. 函数系统（functions.rs）

定义内置函数和自定义函数：
- 数学函数（abs, sqrt, pow, log, sin, cos, tan, round, ceil, floor）
- 字符串函数（length, upper, lower, trim, substring, concat, replace, contains）
- 聚合函数（count, sum, avg, min, max, collect, distinct）
- 类型转换函数（to_string, to_int, to_float, to_bool）
- 日期时间函数（now, date, time, year, month, day, hour, minute, second）

### 6. 错误处理（error.rs）

统一的错误类型：
- 类型错误
- 未定义变量/函数
- 参数数量错误
- 除零错误
- 溢出错误
- 索引越界
- 空值错误
- 语法错误
- 运行时错误

### 7. 求值选项和统计（evaluation.rs）

- `EvaluationOptions`: 求值配置（严格模式、类型转换、递归深度、超时、缓存）
- `EvaluationStatistics`: 求值统计（表达式数量、函数调用次数、变量访问次数、缓存命中率）

## 使用示例

### 基础用法

```rust
use crate::core::expressions::{DefaultExpressionContext, ExpressionContext};

let mut context = DefaultExpressionContext::new();
context.set_variable("x".to_string(), Value::Int(42));

let value = context.get_variable("x");
assert_eq!(value, Some(Value::Int(42)));
```

### 带缓存的上下文

```rust
use crate::core::expressions::BasicExpressionContext;
use crate::cache::CacheConfig;

let cache_config = CacheConfig::default();
let context = BasicExpressionContext::with_cache(cache_config);
```

### 函数注册

```rust
use crate::core::expressions::{BasicExpressionContext, BuiltinFunction, MathFunction};

let mut context = BasicExpressionContext::new();
context.register_builtin_function(BuiltinFunction::Math(MathFunction::Abs));
```

### 层次化上下文

```rust
use crate::core::expressions::{BasicExpressionContext, ExpressionContextType};

let parent = BasicExpressionContext::new();
let child = parent.create_child_context();
```

## 依赖关系

```
src/expression (高级API)
    ↓ 依赖
src/core/expressions (求值上下文)
    ↓ 依赖
src/core/evaluator (求值器trait)
```

## 设计原则

1. **关注点分离**：本模块只负责上下文管理，不涉及表达式类型定义
2. **无循环依赖**：expression模块可以作为独立模块使用
3. **性能优先**：使用枚举避免动态分发，提供缓存机制
4. **可扩展性**：支持自定义函数和嵌套上下文

## 注意事项

1. **不应与 src/expression 合并**：两个模块职责不同
2. **ExpressionContext trait 定义在 src/core/evaluator/traits.rs**：本模块提供实现
3. **缓存功能可选**：通过 CacheConfig 控制是否启用
4. **函数执行尚未完全实现**：当前返回错误，等待后续实现
