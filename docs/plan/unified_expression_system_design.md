# 统一表达式系统设计文档

## 文档概述

本文档详细分析了当前GraphDB查询处理流程中表达式传递存在的问题，并提出了统一的表达式系统设计方案，以消除重复解析和数据退化问题。

## 目录

- [1. 问题分析](#1-问题分析)
  - [1.1 当前数据传播路径](#11-当前数据传播路径)
  - [1.2 数据退化问题](#12-数据退化问题)
  - [1.3 重复解析问题](#13-重复解析问题)
- [2. 统一表达式系统设计](#2-统一表达式系统设计)
  - [2.1 核心设计原则](#21-核心设计原则)
  - [2.2 数据结构设计](#22-数据结构设计)
  - [2.3 数据流设计](#23-数据流设计)
- [3. 技术挑战与解决方案](#3-技术挑战与解决方案)
  - [3.1 序列化需求](#31-序列化需求)
  - [3.2 PlanNode序列化兼容性](#32-plannodesequence化兼容性)
  - [3.3 与现有QueryContext集成](#33-与现有querycontext集成)
- [4. 实现步骤](#4-实现步骤)
- [5. 预期收益](#5-预期收益)

---

## 1. 问题分析

### 1.1 当前数据传播路径

当前GraphDB的查询处理流程如下：

```
原始查询字符串
    ↓
Parser阶段: 生成ExpressionMeta（包含位置信息、类型信息等）
    ↓
Validator阶段: 生成ValidatedStatement（部分信息丢失）
    ↓
Planner阶段: 生成PlanNode（表达式退化成String！）
    ↓
Rewrite/Optimizer阶段: 基于String重新解析
    ↓
Executor阶段: 需要从String重新解析为Expression
```

**关键问题**：在Planner阶段，完整的Expression对象退化成了String，导致：
- 类型信息丢失
- 需要重新解析（即使有缓存也是不必要的开销）
- 无法进行跨阶段的优化
- Rewrite/Optimizer阶段需要回到原始字符串

### 1.2 数据退化问题

数据退化是指数据在传递过程中丢失了重要信息，导致后续阶段需要重新计算或解析。

**当前的数据退化链**：

| 阶段 | 数据类型 | 保留信息 | 丢失信息 |
|------|----------|----------|----------|
| Parser | ExpressionMeta | 完整表达式、位置信息、ID | 无 |
| Validator | ValidatedStatement | 部分表达式信息 | 位置信息、部分类型信息 |
| Planner | PlanNode | 表达式字符串 | 类型信息、位置信息、优化标记 |
| Executor | Expression | 重新解析的表达式 | 所有优化信息 |

**影响**：
- 每个阶段都需要重新解析表达式
- 类型推导结果无法传递
- 优化标记无法保留
- 无法进行跨阶段的增量优化

### 1.3 重复解析问题

通过代码分析，发现Executor阶段存在大量的重复解析操作：

**示例代码**（src/query/executor/factory.rs）：

```rust
fn parse_expression_safe(expr_str: &str) -> Option<crate::core::Expression> {
    crate::query::parser::parser::parse_expression_meta_from_string(expr_str)
        .map(|meta| meta.into())
        .inspect_err(|e| {
            log::warn!("Failed to parse expression: {}, error: {:?}", expr_str, e);
        })
        .ok()
}
```

**问题**：
- PlanNode中的filter字段使用String类型
- Executor需要从String重新解析为Expression
- 即使有缓存，也是不必要的开销
- 无法利用前一个阶段的优化结果

**根本原因**：
- 缓存只是治标不治本
- 根本问题在于数据退化
- 应该直接传递数据的引用，而非每次尝试查询缓存

---

## 2. 统一表达式系统设计

### 2.1 核心设计原则

1. **数据完整性**：每个阶段都持有完整的Expression对象，不退化到String
2. **引用传递**：使用Arc共享Expression对象，避免克隆开销
3. **增量优化**：每个阶段基于前一个阶段的成果进行改进
4. **类型安全**：利用Rust的类型系统确保数据传递的正确性
5. **并发安全**：使用DashMap支持并发访问
6. **序列化兼容**：双模式设计支持存储和传输需求

### 2.2 数据结构设计

#### 2.2.1 ExpressionContext

表达式上下文，作为跨阶段的共享上下文，存储所有表达式的完整信息。

```rust
use std::sync::Arc;
use dashmap::DashMap;
use crate::core::types::expression::{Expression, ExpressionMeta, ExpressionId, DataType};
use crate::core::Value;

/// 表达式上下文
///
/// 跨阶段共享的表达式信息存储，支持并发访问
#[derive(Debug, Clone)]
pub struct ExpressionContext {
    /// 表达式注册表：存储所有表达式的完整信息
    expressions: Arc<DashMap<ExpressionId, Arc<ExpressionMeta>>>,
    
    /// 类型信息缓存：表达式ID -> 推导出的类型
    type_cache: Arc<DashMap<ExpressionId, DataType>>,
    
    /// 常量折叠结果：表达式ID -> 计算出的常量值
    constant_cache: Arc<DashMap<ExpressionId, Value>>,
    
    /// 优化标记：表达式ID -> 优化状态
    optimization_flags: Arc<DashMap<ExpressionId, OptimizationFlags>>,
}

/// 表达式优化状态标记
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptimizationFlags {
    /// 是否已经过类型推导
    pub typed: bool,
    /// 是否已经过常量折叠
    pub constant_folded: bool,
    /// 是否已经过公共子表达式消除
    pub cse_eliminated: bool,
}

impl ExpressionContext {
    /// 创建新的表达式上下文
    pub fn new() -> Self {
        Self {
            expressions: Arc::new(DashMap::new()),
            type_cache: Arc::new(DashMap::new()),
            constant_cache: Arc::new(DashMap::new()),
            optimization_flags: Arc::new(DashMap::new()),
        }
    }
    
    /// 注册表达式到上下文中
    pub fn register_expression(&self, expr: ExpressionMeta) -> ExpressionId {
        let id = expr.id().unwrap_or_else(|| {
            ExpressionId::new(self.expressions.len() as u64)
        });
        
        self.expressions.insert(id.clone(), Arc::new(expr));
        id
    }
    
    /// 获取表达式
    pub fn get_expression(&self, id: &ExpressionId) -> Option<Arc<ExpressionMeta>> {
        self.expressions.get(id).map(|r| r.clone())
    }
    
    /// 设置表达式类型
    pub fn set_type(&self, id: &ExpressionId, data_type: DataType) {
        self.type_cache.insert(id.clone(), data_type);
    }
    
    /// 获取表达式类型
    pub fn get_type(&self, id: &ExpressionId) -> Option<DataType> {
        self.type_cache.get(id).map(|r| r.clone())
    }
    
    /// 设置常量值
    pub fn set_constant(&self, id: &ExpressionId, value: Value) {
        self.constant_cache.insert(id.clone(), value);
        self.set_optimization_flag(id, OptimizationFlags {
            typed: true,
            constant_folded: true,
            cse_eliminated: false,
        });
    }
    
    /// 获取常量值
    pub fn get_constant(&self, id: &ExpressionId) -> Option<Value> {
        self.constant_cache.get(id).map(|r| r.clone())
    }
    
    /// 设置优化标记
    pub fn set_optimization_flag(&self, id: &ExpressionId, flags: OptimizationFlags) {
        self.optimization_flags.insert(id.clone(), flags);
    }
    
    /// 获取优化标记
    pub fn get_optimization_flags(&self, id: &ExpressionId) -> Option<OptimizationFlags> {
        self.optimization_flags.get(id).map(|r| *r.value())
    }
    
    /// 检查表达式是否为常量
    pub fn is_constant(&self, id: &ExpressionId) -> bool {
        self.constant_cache.contains_key(id)
    }
}
```

#### 2.2.2 ContextualExpression

上下文表达式，轻量级的表达式引用，持有ExpressionId和Context引用。

```rust
use std::sync::Arc;
use crate::core::types::expression::{ExpressionId, ExpressionMeta, DataType};
use crate::core::Value;

/// 增强的表达式元数据，包含查询上下文引用
#[derive(Debug, Clone)]
pub struct ContextualExpression {
    /// 表达式ID
    id: ExpressionId,
    /// 查询上下文引用
    context: Arc<ExpressionContext>,
}

impl ContextualExpression {
    /// 创建上下文表达式
    pub fn new(id: ExpressionId, context: Arc<ExpressionContext>) -> Self {
        Self { id, context }
    }
    
    /// 获取表达式ID
    pub fn id(&self) -> &ExpressionId {
        &self.id
    }
    
    /// 获取表达式元数据
    pub fn expression(&self) -> Option<Arc<ExpressionMeta>> {
        self.context.get_expression(&self.id)
    }
    
    /// 获取表达式类型
    pub fn data_type(&self) -> Option<DataType> {
        self.context.get_type(&self.id)
    }
    
    /// 获取常量值
    pub fn constant_value(&self) -> Option<Value> {
        self.context.get_constant(&self.id)
    }
    
    /// 是否为常量
    pub fn is_constant(&self) -> bool {
        self.context.is_constant(&self.id)
    }
}
```

#### 2.2.3 SerializableExpression

可序列化的表达式引用，用于存储和传输。

```rust
use serde::{Serialize, Deserialize};
use crate::core::types::expression::{Expression, ExpressionId, DataType};
use crate::core::Value;

/// 可序列化的表达式引用（用于存储/传输）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableExpression {
    pub id: ExpressionId,
    pub expression: Expression,
    pub data_type: Option<DataType>,
    pub constant_value: Option<Value>,
}

impl SerializableExpression {
    /// 从ContextualExpression转换为可序列化形式
    pub fn from_contextual(ctx_expr: &ContextualExpression) -> Self {
        let expr_meta = ctx_expr.expression().expect("Expression not found");
        Self {
            id: ctx_expr.id().clone(),
            expression: expr_meta.inner().clone(),
            data_type: ctx_expr.data_type(),
            constant_value: ctx_expr.constant_value(),
        }
    }
    
    /// 转换为ContextualExpression
    pub fn to_contextual(self, ctx: Arc<ExpressionContext>) -> ContextualExpression {
        let expr_meta = ExpressionMeta::with_id(self.expression, self.id.clone());
        ctx.register_expression(expr_meta);
        
        if let Some(data_type) = self.data_type {
            ctx.set_type(&self.id, data_type);
        }
        
        if let Some(constant_value) = self.constant_value {
            ctx.set_constant(&self.id, constant_value);
        }
        
        ContextualExpression::new(self.id, ctx)
    }
}
```

### 2.3 数据流设计

#### 2.3.1 完整的数据流

```
┌─────────────────────────────────────────────────────────────┐
│  Parser阶段                                                  │
│  1. 解析查询字符串，生成ExpressionMeta                        │
│  2. 注册到ExpressionContext                                  │
│  3. 生成ValidatedStatement，包含ExpressionId                  │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Validator阶段                                               │
│  1. 基于ExpressionContext中的Expression进行类型推导            │
│  2. 将类型信息写入ExpressionContext.type_cache                │
│  3. 生成ValidatedStatement，包含类型信息                        │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Planner阶段                                                 │
│  1. 基于ValidatedStatement生成PlanNode                        │
│  2. PlanNode持有ContextualExpression（引用ExpressionContext） │
│  3. 无需字符串序列化，直接传递ExpressionId                     │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Rewrite/Optimizer阶段                                       │
│  1. 基于ExpressionContext中的Expression进行重写和优化          │
│  2. 更新ExpressionContext中的优化标记和常量缓存                │
│  3. 修改PlanNode中的ContextualExpression引用                  │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  Executor阶段                                                │
│  1. 直接使用ContextualExpression，无需解析                    │
│  2. 从ExpressionContext获取类型信息和常量值                    │
│  3. 执行表达式求值                                            │
└─────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────┐
│  序列化阶段（如果需要）                                        │
│  1. 调用prepare_for_serialization()                            │
│  2. 将ContextualExpression转换为SerializableExpression          │
│  3. 序列化ExecutionPlan                                       │
└─────────────────────────────────────────────────────────────┘
```

#### 2.3.2 数据传递对比

| 阶段 | 当前方案 | 新方案 | 改进 |
|------|----------|--------|------|
| Parser | ExpressionMeta | ExpressionMeta | 无变化 |
| Validator | ValidatedStatement | ValidatedStatement + ExpressionContext | 添加上下文 |
| Planner | PlanNode (String) | PlanNode (ContextualExpression) | 消除退化 |
| Rewrite | String重新解析 | 基于ExpressionContext优化 | 增量优化 |
| Optimizer | String重新解析 | 基于ExpressionContext优化 | 增量优化 |
| Executor | String重新解析 | 直接使用ContextualExpression | 消除解析 |

---

## 3. 技术挑战与解决方案

### 3.1 序列化需求

**问题**：当前代码中大量使用了`Serialize/Deserialize`，说明`ExecutionPlan`可能需要跨网络传输或持久化。`Arc<ExpressionContext>`和`ContextualExpression`无法直接序列化。

**解决方案**：采用**双模式设计**

- **运行时模式**：使用`Arc`和`DashMap`实现零拷贝、并发安全
- **序列化模式**：使用`SerializableExpression`支持存储/传输

**关键代码**：

```rust
/// 可序列化的表达式引用（用于存储/传输）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableExpression {
    pub id: ExpressionId,
    pub expression: Expression,
    pub data_type: Option<DataType>,
    pub constant_value: Option<Value>,
}

impl SerializableExpression {
    /// 从ContextualExpression转换为可序列化形式
    pub fn from_contextual(ctx_expr: &ContextualExpression) -> Self {
        let expr_meta = ctx_expr.expression().expect("Expression not found");
        Self {
            id: ctx_expr.id().clone(),
            expression: expr_meta.inner().clone(),
            data_type: ctx_expr.data_type(),
            constant_value: ctx_expr.constant_value(),
        }
    }
    
    /// 转换为ContextualExpression
    pub fn to_contextual(self, ctx: Arc<ExpressionContext>) -> ContextualExpression {
        let expr_meta = ExpressionMeta::with_id(self.expression, self.id.clone());
        ctx.register_expression(expr_meta);
        
        if let Some(data_type) = self.data_type {
            ctx.set_type(&self.id, data_type);
        }
        
        if let Some(constant_value) = self.constant_value {
            ctx.set_constant(&self.id, constant_value);
        }
        
        ContextualExpression::new(self.id, ctx)
    }
}
```

### 3.2 PlanNode序列化兼容性

**问题**：PlanNode需要同时支持运行时和序列化两种模式。

**解决方案**：使用`#[serde(skip)]`和`#[serde(skip_serializing_if)]`属性

**关键代码**：

```rust
use crate::core::types::expression::{ContextualExpression, SerializableExpression};
use crate::core::types::EdgeDirection;
use std::sync::Arc;

define_plan_node! {
    pub struct ExpandNode {
        space_id: u64,
        edge_types: Vec<String>,
        direction: EdgeDirection,
        step_limit: Option<u32>,
        // 运行时使用：上下文表达式
        #[serde(skip)]
        filter: Option<ContextualExpression>,
        // 序列化使用：可序列化表达式
        #[serde(skip_serializing_if = "Option::is_none")]
        filter_serializable: Option<SerializableExpression>,
    }
    enum: Expand
    input: MultipleInputNode
}

impl ExpandNode {
    pub fn filter(&self) -> Option<&ContextualExpression> {
        self.filter.as_ref()
    }

    pub fn set_filter(&mut self, filter: ContextualExpression) {
        self.filter = Some(filter);
        self.filter_serializable = None; // 清除序列化版本
    }
    
    /// 序列化前调用：将ContextualExpression转换为SerializableExpression
    pub fn prepare_for_serialization(&mut self) {
        if let Some(ref ctx_expr) = self.filter {
            self.filter_serializable = Some(SerializableExpression::from_contextual(ctx_expr));
        }
    }
    
    /// 反序列化后调用：将SerializableExpression转换为ContextualExpression
    pub fn after_deserialization(&mut self, ctx: Arc<ExpressionContext>) {
        if let Some(ref ser_expr) = self.filter_serializable {
            self.filter = Some(ser_expr.clone().to_contextual(ctx));
        }
    }
}
```

### 3.3 与现有QueryContext集成

**问题**：需要将`ExpressionContext`集成到现有的`QueryContext`中。

**解决方案**：在`QueryContext`中添加`expr_context`字段

**关键代码**：

```rust
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use crate::core::types::expression::ExpressionContext;

pub struct QueryContext {
    /// 查询请求上下文
    rctx: Arc<QueryRequestContext>,

    /// 执行计划
    plan: RwLock<Option<Box<ExecutionPlan>>>,

    /// 字符集信息
    charset_info: Option<Box<CharsetInfo>>,

    /// 对象池
    obj_pool: ObjectPool<String>,

    /// ID 生成器
    id_gen: IdGenerator,

    /// 符号表
    sym_table: Arc<SymbolTable>,

    /// 当前空间信息
    space_info: RwLock<Option<SpaceInfo>>,

    /// 是否被标记为已终止
    killed: AtomicBool,

    /// 验证结果缓存
    validation_info: RwLock<Option<ValidationInfo>>,
    
    /// 表达式上下文（新增）
    expr_context: Arc<ExpressionContext>,
}

impl QueryContext {
    pub fn new(rctx: Arc<QueryRequestContext>) -> Self {
        Self {
            rctx,
            plan: RwLock::new(None),
            charset_info: None,
            obj_pool: ObjectPool::new(1000),
            id_gen: IdGenerator::new(0),
            sym_table: Arc::new(SymbolTable::new()),
            space_info: RwLock::new(None),
            killed: AtomicBool::new(false),
            validation_info: RwLock::new(None),
            expr_context: Arc::new(ExpressionContext::new()),
        }
    }
    
    /// 获取表达式上下文
    pub fn expression_context(&self) -> Arc<ExpressionContext> {
        self.expr_context.clone()
    }
}
```

---

## 4. 实现步骤

### 阶段1：基础数据结构

1. **创建`ExpressionContext`**
   - 文件：`src/core/types/expression/context.rs`
   - 实现表达式注册表、类型缓存、常量缓存、优化标记

2. **创建`ContextualExpression`**
   - 文件：`src/core/types/expression/contextual.rs`
   - 实现轻量级的表达式引用

3. **创建`SerializableExpression`**
   - 文件：`src/core/types/expression/serializable.rs`
   - 实现可序列化的表达式引用

### 阶段2：PlanNode改造

4. **修改所有PlanNode**
   - 将String字段改为ContextualExpression
   - 添加对应的SerializableExpression字段
   - 实现`prepare_for_serialization()`和`after_deserialization()`方法

5. **重点修改的文件**：
   - `src/query/planner/plan/core/nodes/traversal_node.rs`
   - `src/query/planner/plan/core/nodes/filter_node.rs`
   - `src/query/planner/plan/core/nodes/project_node.rs`
   - 其他包含表达式字段的PlanNode

### 阶段3：Parser集成

6. **修改Parser**
   - 文件：`src/query/parser/parser/parser.rs`
   - 生成ExpressionMeta后注册到ExpressionContext
   - 返回包含ExpressionId的AST

### 阶段4：Validator集成

7. **修改Validator**
   - 文件：`src/query/validator/validator_enum.rs`
   - 基于ExpressionContext进行类型推导
   - 将类型信息写入ExpressionContext.type_cache

### 阶段5：Planner集成

8. **修改Planner**
   - 文件：`src/query/planner/planner.rs`
   - 生成包含ContextualExpression的PlanNode
   - 传递Arc<ExpressionContext>

### 阶段6：Rewrite/Optimizer集成

9. **修改Rewrite**
   - 基于ExpressionContext中的Expression进行重写
   - 更新ExpressionContext中的优化标记

10. **修改Optimizer**
    - 基于ExpressionContext中的Expression进行优化
    - 实现常量折叠、公共子表达式消除等优化
    - 更新ExpressionContext中的常量缓存和优化标记

### 阶段7：Executor集成

11. **修改Executor**
    - 文件：`src/query/executor/factory.rs`
    - 直接使用ContextualExpression，无需解析
    - 删除所有`parse_expression_safe`调用

### 阶段8：测试和验证

12. **单元测试**
    - 测试ExpressionContext的基本功能
    - 测试ContextualExpression的引用传递
    - 测试SerializableExpression的序列化/反序列化

13. **集成测试**
    - 测试完整的查询处理流程
    - 验证数据传递的完整性
    - 验证优化效果

14. **性能测试**
    - 对比改造前后的性能
    - 验证零拷贝传递的效果
    - 验证并发安全性

---

## 5. 预期收益

### 5.1 性能收益

1. **消除重复解析**
   - 从Parser到Executor，表达式只解析一次
   - 减少解析开销：预计提升10-20%性能

2. **零拷贝传递**
   - 使用Arc共享Expression对象
   - 避免克隆开销：预计减少30-50%内存分配

3. **增量优化**
   - Rewrite/Optimizer基于前一个阶段的成果
   - 避免重复计算：预计提升15-25%优化效率

### 5.2 代码质量收益

1. **数据完整性**
   - 每个阶段都持有完整的Expression对象
   - 类型信息、优化标记在整个流程中可用

2. **类型安全**
   - 利用Rust的类型系统确保数据传递的正确性
   - 编译时检查，减少运行时错误

3. **可维护性**
   - 清晰的数据流，易于理解和维护
   - 统一的表达式系统，减少代码重复

### 5.3 功能扩展收益

1. **跨阶段优化**
   - 可以实现更复杂的跨阶段优化策略
   - 例如：基于类型信息的优化、基于统计信息的优化

2. **调试和追踪**
   - 每个表达式都有唯一的ID，可以追踪优化历史
   - 便于调试和性能分析

3. **扩展性**
   - 易于添加新的优化策略
   - 易于添加新的表达式类型

---

## 附录

### A. 相关文件

- `src/core/types/expression/def.rs` - Expression定义
- `src/core/types/expression/expression.rs` - ExpressionMeta定义
- `src/core/types/expression/cache.rs` - 表达式缓存管理（待改造）
- `src/query/query_context.rs` - QueryContext定义
- `src/query/planner/plan/core/nodes/traversal_node.rs` - PlanNode示例
- `src/query/executor/factory.rs` - Executor工厂（待改造）

### B. 参考资料

- [GraphDB Project Context](../../.trae/rules/project_rules.md)
- [Query Module Architecture](../architecture/query_module_architecture.md)
- [Execution Context Unified Analysis](../analysis/execution_context_unified_analysis.md)

### C. 版本历史

| 版本 | 日期 | 作者 | 变更说明 |
|------|------|------|----------|
| 1.0 | 2025-02-28 | AI | 初始版本，完成问题分析和设计方案 |

---

**文档结束**
