# 表达式系统完全重构设计

## 一、重构目标

### 1.1 核心目标

**消除 planner 层对 Expression 的直接依赖**，实现：
- ✅ Planner 层只能通过 `ContextualExpression` 操作表达式
- ✅ `Expression` 作为内部实现细节，不对外暴露
- ✅ 所有表达式操作通过 `ExpressionContext` 统一管理
- ✅ 类型安全、信息完整、性能优化

### 1.2 当前问题

```rust
// ❌ 问题 1: planner 层直接使用 Expression
use crate::core::Expression;
let expr = Expression::Binary { ... };

// ❌ 问题 2: 三步提取导致信息丢失
let filter_condition = filter_node.condition();
let filter_expr = filter_condition.expression()?.inner().clone();  // 丢失上下文
let ctx = filter_condition.context().clone();

// ❌ 问题 3: 工具函数操作 Expression
let (picked, remained) = split_filter(&filter_expr, picker);  // 丢失类型、常量等信息

// ❌ 问题 4: 重复注册和创建
let expr_meta = ExpressionMeta::new(new_expr);
let id = ctx.register_expression(expr_meta);
let ctx_expr = ContextualExpression::new(id, ctx);
```

## 二、新架构设计

### 2.1 架构分层

```
┌─────────────────────────────────────────────────────────────┐
│                    Planner Layer                          │
│  (只能使用 ContextualExpression，不接触 Expression)        │
├─────────────────────────────────────────────────────────────┤
│  - FilterNode.condition: ContextualExpression              │
│  - ProjectNode.columns: Vec<ContextualExpression>         │
│  - 所有重写规则只操作 ContextualExpression                │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              Expression Interface Layer                    │
│  (ContextualExpression + ExpressionContext)               │
├─────────────────────────────────────────────────────────────┤
│  - ContextualExpression: 唯一对外接口                    │
│  - ExpressionContext: 表达式管理中心                      │
│  - ExpressionBuilder: 表达式构建器                        │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              Expression Implementation                     │
│  (内部实现细节，不对外暴露)                              │
├─────────────────────────────────────────────────────────────┤
│  - Expression: AST 定义                                   │
│  - ExpressionMeta: 元数据包装                            │
│  - ExpressionId: 唯一标识符                              │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 模块职责

| 模块 | 职责 | 暴露给 planner |
|------|------|----------------|
| `Expression` | AST 定义 | ❌ 不暴露 |
| `ExpressionMeta` | 元数据包装 | ❌ 不暴露 |
| `ExpressionContext` | 表达式管理中心 | ✅ 暴露 |
| `ContextualExpression` | 唯一对外接口 | ✅ 暴露 |
| `ExpressionBuilder` | 表达式构建器 | ✅ 暴露 |

## 三、核心类型设计

### 3.1 ExpressionContext（表达式管理中心）

```rust
//! 表达式上下文
//!
//! 跨阶段共享的表达式信息存储，支持并发访问。
//! 所有表达式操作必须通过 ExpressionContext 进行。

use std::sync::Arc;
use dashmap::DashMap;

use super::{Expression, ExpressionMeta, ExpressionId};
use crate::core::types::DataType;
use crate::core::Value;

/// 表达式优化状态标记
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptimizationFlags {
    pub typed: bool,
    pub constant_folded: bool,
    pub cse_eliminated: bool,
}

/// 表达式上下文
///
/// 跨阶段共享的表达式信息存储，支持并发访问。
/// 存储表达式的完整信息，包括：
/// - 表达式注册表：存储所有表达式的完整信息
/// - 类型信息缓存：表达式ID -> 推导出的类型
/// - 常量折叠结果：表达式ID -> 计算出的常量值
/// - 优化标记：表达式ID -> 优化状态
#[derive(Debug, Clone)]
pub struct ExpressionContext {
    expressions: Arc<DashMap<ExpressionId, Arc<ExpressionMeta>>>,
    type_cache: Arc<DashMap<ExpressionId, DataType>>,
    constant_cache: Arc<DashMap<ExpressionId, Value>>,
    optimization_flags: Arc<DashMap<ExpressionId, OptimizationFlags>>,
}

impl ExpressionContext {
    pub fn new() -> Self;

    /// 注册表达式到上下文中
    pub fn register_expression(&self, expr: ExpressionMeta) -> ExpressionId;

    /// 获取表达式
    pub fn get_expression(&self, id: &ExpressionId) -> Option<Arc<ExpressionMeta>>;

    /// 设置表达式类型
    pub fn set_type(&self, id: &ExpressionId, data_type: DataType);

    /// 获取表达式类型
    pub fn get_type(&self, id: &ExpressionId) -> Option<DataType>;

    /// 设置常量值
    pub fn set_constant(&self, id: &ExpressionId, value: Value);

    /// 获取常量值
    pub fn get_constant(&self, id: &ExpressionId) -> Option<Value>;

    /// 设置优化标记
    pub fn set_optimization_flag(&self, id: &ExpressionId, flags: OptimizationFlags);

    /// 获取优化标记
    pub fn get_optimization_flags(&self, id: &ExpressionId) -> Option<OptimizationFlags>;

    /// 检查表达式是否为常量
    pub fn is_constant(&self, id: &ExpressionId) -> bool;

    /// 检查表达式是否已经过类型推导
    pub fn is_typed(&self, id: &ExpressionId) -> bool;

    /// 检查表达式是否已经过常量折叠
    pub fn is_constant_folded(&self, id: &ExpressionId) -> bool;

    /// 检查表达式是否已经过公共子表达式消除
    pub fn is_cse_eliminated(&self, id: &ExpressionId) -> bool;

    /// 获取注册的表达式数量
    pub fn expression_count(&self) -> usize;

    /// 清空所有缓存（保留表达式注册表）
    pub fn clear_caches(&self);

    /// 清空所有数据
    pub fn clear_all(&self);
}
```

### 3.2 ContextualExpression（唯一对外接口）

```rust
//! 上下文表达式
//!
//! 本模块定义 ContextualExpression，作为轻量级的表达式引用，
//! 持有 ExpressionId 和 Context 引用。
//!
//! 这是 planner 层唯一能使用的表达式类型。

use std::sync::Arc;
use std::collections::HashMap;
use std::ops::Add;

use super::{Expression, ExpressionMeta, ExpressionId};
use super::context::ExpressionContext;
use crate::core::types::DataType;
use crate::core::types::operators::{BinaryOperator, UnaryOperator, AggregateFunction};
use crate::core::Value;

/// 增强的表达式元数据，包含查询上下文引用
///
/// 轻量级的表达式引用，持有 ExpressionId 和 Context 引用。
/// 通过 ExpressionContext 可以访问表达式的完整信息、类型、常量值等。
///
/// # 设计原则
///
/// - **唯一接口**：planner 层只能使用 ContextualExpression
/// - **信息完整**：保留所有上下文信息（类型、常量、优化状态）
/// - **操作安全**：所有操作通过 ExpressionContext 统一管理
#[derive(Debug, Clone)]
pub struct ContextualExpression {
    id: ExpressionId,
    context: Arc<ExpressionContext>,
}

impl ContextualExpression {
    pub fn new(id: ExpressionId, context: Arc<ExpressionContext>) -> Self;

    // ========== 信息访问 ==========

    pub fn id(&self) -> &ExpressionId;
    pub fn expression(&self) -> Option<Arc<ExpressionMeta>>;
    pub fn data_type(&self) -> Option<DataType>;
    pub fn constant_value(&self) -> Option<Value>;
    pub fn is_constant(&self) -> bool;
    pub fn is_typed(&self) -> bool;
    pub fn is_constant_folded(&self) -> bool;
    pub fn is_cse_eliminated(&self) -> bool;
    pub fn context(&self) -> &Arc<ExpressionContext>;

    // ========== 类型检查 ==========

    pub fn is_literal(&self) -> bool;
    pub fn is_variable(&self) -> bool;
    pub fn is_aggregate(&self) -> bool;
    pub fn is_binary(&self) -> bool;
    pub fn is_unary(&self) -> bool;
    pub fn is_function(&self) -> bool;

    pub fn as_variable(&self) -> Option<String>;
    pub fn as_literal(&self) -> Option<Value>;

    // ========== 表达式分析 ==========

    /// 获取变量列表
    pub fn get_variables(&self) -> Vec<String>;

    /// 获取属性引用列表
    pub fn get_property_refs(&self) -> Vec<String>;

    /// 检查是否包含指定属性
    pub fn contains_property(&self, property_names: &[String]) -> bool;

    /// 检查是否包含聚合函数
    pub fn contains_aggregate(&self) -> bool;

    /// 检查是否包含指定变量
    pub fn contains_variable(&self, var_name: &str) -> bool;

    // ========== 表达式转换 ==========

    /// 分割过滤条件
    ///
    /// 将复合过滤条件（如 AND 连接的条件）分割为两部分：
    /// - 符合选择器函数的部分
    /// - 剩余的部分
    ///
    /// # 参数
    /// - `picker`: 选择器函数，返回 true 表示该部分应该被选中
    ///
    /// # 返回
    /// (选中的部分, 剩余的部分)
    pub fn split_filter<F>(
        &self,
        picker: F,
    ) -> (Option<ContextualExpression>, Option<ContextualExpression>)
    where
        F: Fn(&Expression) -> bool;

    /// 重写表达式，将变量引用替换为实际表达式
    ///
    /// # 参数
    /// - `rewrite_map`: 重写映射表，键为变量名，值为要替换的表达式
    ///
    /// # 返回
    /// 重写后的 ContextualExpression
    pub fn rewrite(
        &self,
        rewrite_map: &HashMap<String, ContextualExpression>,
    ) -> ContextualExpression;

    /// 合并两个表达式（使用 AND）
    pub fn and(&self, other: &ContextualExpression) -> ContextualExpression;

    /// 合并两个表达式（使用 OR）
    pub fn or(&self, other: &ContextualExpression) -> ContextualExpression;

    /// 取反表达式
    pub fn not(&self) -> ContextualExpression;

    // ========== 表达式构造 ==========

    /// 创建字面量表达式
    pub fn literal(ctx: Arc<ExpressionContext>, value: Value) -> Self;

    /// 创建变量表达式
    pub fn variable(ctx: Arc<ExpressionContext>, name: String) -> Self;

    /// 创建属性访问表达式
    pub fn property(
        ctx: Arc<ExpressionContext>,
        object: ContextualExpression,
        property: String,
    ) -> Self;

    /// 创建二元运算表达式
    pub fn binary(
        ctx: Arc<ExpressionContext>,
        left: ContextualExpression,
        op: BinaryOperator,
        right: ContextualExpression,
    ) -> Self;

    /// 创建一元运算表达式
    pub fn unary(
        ctx: Arc<ExpressionContext>,
        op: UnaryOperator,
        operand: ContextualExpression,
    ) -> Self;

    /// 创建函数调用表达式
    pub fn function(
        ctx: Arc<ExpressionContext>,
        name: String,
        args: Vec<ContextualExpression>,
    ) -> Self;

    /// 创建聚合函数表达式
    pub fn aggregate(
        ctx: Arc<ExpressionContext>,
        func: AggregateFunction,
        arg: ContextualExpression,
        distinct: bool,
    ) -> Self;

    // ========== 字符串表示 ==========

    pub fn to_expression_string(&self) -> String;
}

// ========== 运算符重载 ==========

impl Add for &ContextualExpression {
    type Output = ContextualExpression;

    fn add(self, rhs: Self) -> Self::Output {
        let ctx = self.context().clone();
        ContextualExpression::binary(ctx, self.clone(), BinaryOperator::Add, rhs.clone())
    }
}
```

### 3.3 ExpressionBuilder（表达式构建器）

```rust
//! 表达式构建器
//!
//! 提供流畅的 API 用于构建复杂表达式。

use std::sync::Arc;

use super::contextual::ContextualExpression;
use super::context::ExpressionContext;
use crate::core::Value;
use crate::core::types::operators::{BinaryOperator, UnaryOperator, AggregateFunction};

/// 表达式构建器
///
/// 提供流畅的 API 用于构建复杂表达式。
///
/// # 示例
///
/// ```rust
/// use crate::core::types::expression::ExpressionBuilder;
///
/// let ctx = Arc::new(ExpressionContext::new());
/// let expr = ExpressionBuilder::new(&ctx)
///     .variable("a")
///     .add(ExpressionBuilder::new(&ctx).variable("b"))
///     .build();
/// ```
pub struct ExpressionBuilder<'a> {
    ctx: &'a Arc<ExpressionContext>,
    current: Option<ContextualExpression>,
}

impl<'a> ExpressionBuilder<'a> {
    pub fn new(ctx: &'a Arc<ExpressionContext>) -> Self {
        Self {
            ctx,
            current: None,
        }
    }

    // ========== 字面量 ==========

    pub fn literal(self, value: Value) -> Self {
        Self {
            current: Some(ContextualExpression::literal(self.ctx.clone(), value)),
            ..self
        }
    }

    pub fn int(self, value: i64) -> Self {
        self.literal(Value::Int(value))
    }

    pub fn float(self, value: f64) -> Self {
        self.literal(Value::Float(value))
    }

    pub fn string(self, value: String) -> Self {
        self.literal(Value::String(value))
    }

    pub fn bool(self, value: bool) -> Self {
        self.literal(Value::Bool(value))
    }

    // ========== 变量 ==========

    pub fn variable(self, name: String) -> Self {
        Self {
            current: Some(ContextualExpression::variable(self.ctx.clone(), name)),
            ..self
        }
    }

    // ========== 属性访问 ==========

    pub fn property(self, object: ContextualExpression, property: String) -> Self {
        Self {
            current: Some(ContextualExpression::property(
                self.ctx.clone(),
                object,
                property,
            )),
            ..self
        }
    }

    // ========== 二元运算 ==========

    pub fn add(self, rhs: Self) -> Self {
        self.binary_op(BinaryOperator::Add, rhs)
    }

    pub fn subtract(self, rhs: Self) -> Self {
        self.binary_op(BinaryOperator::Subtract, rhs)
    }

    pub fn multiply(self, rhs: Self) -> Self {
        self.binary_op(BinaryOperator::Multiply, rhs)
    }

    pub fn divide(self, rhs: Self) -> Self {
        self.binary_op(BinaryOperator::Divide, rhs)
    }

    pub fn equal(self, rhs: Self) -> Self {
        self.binary_op(BinaryOperator::Equal, rhs)
    }

    pub fn not_equal(self, rhs: Self) -> Self {
        self.binary_op(BinaryOperator::NotEqual, rhs)
    }

    pub fn greater_than(self, rhs: Self) -> Self {
        self.binary_op(BinaryOperator::GreaterThan, rhs)
    }

    pub fn less_than(self, rhs: Self) -> Self {
        self.binary_op(BinaryOperator::LessThan, rhs)
    }

    pub fn and(self, rhs: Self) -> Self {
        self.binary_op(BinaryOperator::And, rhs)
    }

    pub fn or(self, rhs: Self) -> Self {
        self.binary_op(BinaryOperator::Or, rhs)
    }

    fn binary_op(self, op: BinaryOperator, rhs: Self) -> Self {
        let left = self.current.expect("Left operand not set");
        let right = rhs.current.expect("Right operand not set");
        Self {
            current: Some(ContextualExpression::binary(
                self.ctx.clone(),
                left,
                op,
                right,
            )),
            ..self
        }
    }

    // ========== 一元运算 ==========

    pub fn not(self) -> Self {
        let operand = self.current.expect("Operand not set");
        Self {
            current: Some(ContextualExpression::unary(
                self.ctx.clone(),
                UnaryOperator::Not,
                operand,
            )),
            ..self
        }
    }

    pub fn negate(self) -> Self {
        let operand = self.current.expect("Operand not set");
        Self {
            current: Some(ContextualExpression::unary(
                self.ctx.clone(),
                UnaryOperator::Negate,
                operand,
            )),
            ..self
        }
    }

    // ========== 函数调用 ==========

    pub fn function(self, name: String, args: Vec<Self>) -> Self {
        let args_exprs: Vec<ContextualExpression> = args
            .into_iter()
            .map(|b| b.current.expect("Argument not set"))
            .collect();
        Self {
            current: Some(ContextualExpression::function(
                self.ctx.clone(),
                name,
                args_exprs,
            )),
            ..self
        }
    }

    // ========== 聚合函数 ==========

    pub fn count(self, distinct: bool) -> Self {
        let arg = self.current.expect("Argument not set");
        Self {
            current: Some(ContextualExpression::aggregate(
                self.ctx.clone(),
                AggregateFunction::Count(None),
                arg,
                distinct,
            )),
            ..self
        }
    }

    pub fn sum(self, distinct: bool) -> Self {
        let arg = self.current.expect("Argument not set");
        Self {
            current: Some(ContextualExpression::aggregate(
                self.ctx.clone(),
                AggregateFunction::Sum(arg.as_variable().unwrap_or_default()),
                arg,
                distinct,
            )),
            ..self
        }
    }

    pub fn avg(self, distinct: bool) -> Self {
        let arg = self.current.expect("Argument not set");
        Self {
            current: Some(ContextualExpression::aggregate(
                self.ctx.clone(),
                AggregateFunction::Avg(arg.as_variable().unwrap_or_default()),
                arg,
                distinct,
            )),
            ..self
        }
    }

    pub fn max(self, distinct: bool) -> Self {
        let arg = self.current.expect("Argument not set");
        Self {
            current: Some(ContextualExpression::aggregate(
                self.ctx.clone(),
                AggregateFunction::Max,
                arg,
                distinct,
            )),
            ..self
        }
    }

    pub fn min(self, distinct: bool) -> Self {
        let arg = self.current.expect("Argument not set");
        Self {
            current: Some(ContextualExpression::aggregate(
                self.ctx.clone(),
                AggregateFunction::Min,
                arg,
                distinct,
            )),
            ..self
        }
    }

    // ========== 构建 ==========

    pub fn build(self) -> ContextualExpression {
        self.current.expect("Expression not built")
    }
}
```

## 四、Planner 层改造

### 4.1 FilterNode 改造

```rust
//! 过滤节点实现

use std::sync::Arc;

use crate::define_plan_node_with_deps;
use crate::core::types::ContextualExpression;
use crate::core::types::SerializableExpression;
use crate::core::types::ExpressionContext;
use super::plan_node_enum::PlanNodeEnum;

define_plan_node_with_deps! {
    pub struct FilterNode {
        condition: ContextualExpression,
        condition_serializable: Option<SerializableExpression>,
    }
    enum: Filter
    input: SingleInputNode
}

impl FilterNode {
    /// 创建新的过滤节点
    pub fn new(
        input: PlanNodeEnum,
        condition: ContextualExpression,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            condition,
            condition_serializable: None,
            output_var: None,
            col_names,
        })
    }

    /// 获取过滤条件
    pub fn condition(&self) -> &ContextualExpression {
        &self.condition
    }

    /// 设置过滤条件
    pub fn set_condition(&mut self, condition: ContextualExpression) {
        self.condition = condition;
        self.condition_serializable = None;
    }

    /// 合并两个过滤条件
    pub fn combine_conditions(&mut self, other: &ContextualExpression) {
        let combined = self.condition.and(other);
        self.set_condition(combined);
    }

    pub fn prepare_for_serialization(&mut self) {
        self.condition_serializable = Some(SerializableExpression::from_contextual(&self.condition));
    }

    pub fn after_deserialization(&mut self, ctx: Arc<ExpressionContext>) {
        if let Some(ref ser_expr) = self.condition_serializable {
            self.condition = ser_expr.clone().to_contextual(ctx);
        }
    }
}
```

### 4.2 ProjectNode 改造

```rust
//! 投影节点实现

use std::sync::Arc;

use crate::define_plan_node_with_deps;
use crate::core::types::ContextualExpression;
use crate::core::types::SerializableExpression;
use crate::core::types::ExpressionContext;
use super::plan_node_enum::PlanNodeEnum;

define_plan_node_with_deps! {
    pub struct ProjectNode {
        columns: Vec<ProjectColumn>,
        columns_serializable: Option<Vec<SerializableExpression>>,
    }
    enum: Project
    input: SingleInputNode
}

/// 投影列定义
#[derive(Debug, Clone)]
pub struct ProjectColumn {
    pub expression: ContextualExpression,
    pub alias: String,
    pub is_matched: bool,
}

impl ProjectColumn {
    pub fn new(expression: ContextualExpression, alias: String) -> Self {
        Self {
            expression,
            alias,
            is_matched: false,
        }
    }

    pub fn with_matched(mut self, is_matched: bool) -> Self {
        self.is_matched = is_matched;
        self
    }

    pub fn name(&self) -> &str {
        &self.alias
    }
}

impl ProjectNode {
    pub fn new(
        input: PlanNodeEnum,
        columns: Vec<ProjectColumn>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names: Vec<String> = columns.iter().map(|col| col.alias.clone()).collect();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            columns,
            columns_serializable: None,
            output_var: None,
            col_names,
        })
    }

    pub fn columns(&self) -> &[ProjectColumn] {
        &self.columns
    }

    pub fn set_columns(&mut self, columns: Vec<ProjectColumn>) {
        self.columns = columns;
        self.col_names = self.columns.iter().map(|col| col.alias.clone()).collect();
    }

    pub fn prepare_for_serialization(&mut self) {
        self.columns_serializable = Some(
            self.columns
                .iter()
                .map(|col| SerializableExpression::from_contextual(&col.expression))
                .collect()
        );
    }

    pub fn after_deserialization(&mut self, ctx: Arc<ExpressionContext>) {
        if let Some(ref ser_columns) = self.columns_serializable {
            self.columns = ser_columns
                .iter()
                .map(|ser_expr| ProjectColumn {
                    expression: ser_expr.clone().to_contextual(ctx.clone()),
                    alias: ser_expr.expression.to_expression_string(),
                    is_matched: false,
                })
                .collect();
            self.col_names = self.columns.iter().map(|col| col.alias.clone()).collect();
        }
    }
}
```

### 4.3 重写规则改造示例

```rust
//! 合并多个过滤操作的规则

use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{MergeRule, RewriteRule};

/// 合并多个过滤操作的规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Filter(col2 > 200)
///       |
///   Filter(col1 > 100)
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   Filter(col1 > 100 AND col2 > 200)
///       |
///   ScanVertices
/// ```
#[derive(Debug)]
pub struct CombineFilterRule;

impl CombineFilterRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CombineFilterRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for CombineFilterRule {
    fn name(&self) -> &'static str {
        "CombineFilterRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("Filter")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        let top_filter = match node {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        let input = top_filter.input();
        let child_filter = match input {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        let top_condition = top_filter.condition();
        let child_condition = child_filter.condition();

        // 直接使用 ContextualExpression 的 and 方法
        let combined_condition = top_condition.and(child_condition);

        let child_input = child_filter.input().clone();

        let combined_filter_node = FilterNode::new(child_input, combined_condition)?;

        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(PlanNodeEnum::Filter(combined_filter_node));

        Ok(Some(result))
    }
}

impl MergeRule for CombineFilterRule {
    fn can_merge(&self, parent: &PlanNodeEnum, child: &PlanNodeEnum) -> bool {
        parent.is_filter() && child.is_filter()
    }

    fn create_merged_node(
        &self,
        _ctx: &mut RewriteContext,
        parent: &PlanNodeEnum,
        _child: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(_ctx, parent)
    }
}
```

```rust
//! 将过滤条件下推到哈希内连接操作的规则

use std::sync::Arc;

use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};
use crate::core::types::ExpressionContext;

/// 将过滤条件下推到哈希内连接操作的规则
#[derive(Debug)]
pub struct PushFilterDownHashInnerJoinRule;

impl PushFilterDownHashInnerJoinRule {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PushFilterDownHashInnerJoinRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushFilterDownHashInnerJoinRule {
    fn name(&self) -> &'static str {
        "PushFilterDownHashInnerJoinRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Filter").with_dependency_name("HashInnerJoin")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        let filter_node = match node {
            PlanNodeEnum::Filter(n) => n,
            _ => return Ok(None),
        };

        let input = filter_node.input();
        let join = match input {
            PlanNodeEnum::HashInnerJoin(n) => n,
            _ => return Ok(None),
        };

        let filter_condition = filter_node.condition();

        // 直接使用 ContextualExpression 的 split_filter 方法
        let left_col_names = join.left_input().col_names().to_vec();
        let right_col_names = join.right_input().col_names().to_vec();

        let left_picker = |expr: &crate::core::Expression| -> bool {
            filter_condition.contains_property(&left_col_names)
        };

        let right_picker = |expr: &crate::core::Expression| -> bool {
            filter_condition.contains_property(&right_col_names)
        };

        let (left_picked, left_remained) = filter_condition.split_filter(left_picker);
        let (right_picked, right_remained) = filter_condition.split_filter(right_picker);

        if left_picked.is_none() && right_picked.is_none() {
            return Ok(None);
        }

        let mut new_join = join.clone();
        let mut new_left = join.left_input().clone();
        let mut new_right = join.right_input().clone();

        let ctx = filter_condition.context().clone();

        let left_pushed = left_picked.is_some();
        if let Some(left_filter) = left_picked {
            let left_filter_node = FilterNode::new(new_left, left_filter)?;
            new_left = PlanNodeEnum::Filter(left_filter_node);
        }

        let right_pushed = right_picked.is_some();
        if let Some(right_filter) = right_picked {
            let right_filter_node = FilterNode::new(new_right, right_filter)?;
            new_right = PlanNodeEnum::Filter(right_filter_node);
        }

        new_join.set_left_input(new_left);
        new_join.set_right_input(new_right);

        let mut result = TransformResult::new();

        let remaining_condition = if left_pushed && right_pushed {
            None
        } else if left_pushed {
            right_remained
        } else {
            left_remained
        };

        if let Some(remained) = remaining_condition {
            result.erase_curr = false;
            let mut new_filter = filter_node.clone();
            new_filter.set_condition(remained);
            result.add_new_node(PlanNodeEnum::Filter(new_filter));
        } else {
            result.erase_curr = true;
        }

        result.add_new_node(PlanNodeEnum::HashInnerJoin(new_join));

        Ok(Some(result))
    }
}

impl PushDownRule for PushFilterDownHashInnerJoinRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        matches!((node, target), (PlanNodeEnum::Filter(_), PlanNodeEnum::HashInnerJoin(_)))
    }

    fn push_down(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, node)
    }
}
```

## 五、迁移路径

### 5.1 阶段 1：扩展 ContextualExpression

**目标**：在 ContextualExpression 上添加所有表达式操作方法

**任务**：
1. 添加 `split_filter` 方法
2. 添加 `rewrite` 方法
3. 添加 `and`, `or`, `not` 方法
4. 添加表达式构造方法（`literal`, `variable`, `binary`, `unary` 等）
5. 添加表达式分析方法（`get_variables`, `contains_property` 等）

**影响范围**：
- `src/core/types/expression/contextual.rs`

### 5.2 阶段 2：改造 FilterNode 和 ProjectNode

**目标**：移除对 Expression 的直接依赖

**任务**：
1. FilterNode：保持 `condition: ContextualExpression`
2. ProjectNode：将 `YieldColumn` 改为 `ProjectColumn`，使用 `ContextualExpression`
3. 移除 `from_expression` 方法
4. 添加便捷方法（`combine_conditions` 等）

**影响范围**：
- `src/query/planner/plan/core/nodes/filter_node.rs`
- `src/query/planner/plan/core/nodes/project_node.rs`
- `src/core/types/mod.rs`（移除 `YieldColumn`，添加 `ProjectColumn`）

### 5.3 阶段 3：改造重写规则

**目标**：所有重写规则使用 ContextualExpression

**任务**：
1. 移除所有 `use crate::core::Expression`
2. 移除三步提取模式
3. 使用 ContextualExpression 的方法
4. 移除 `expression_utils.rs` 中的函数（或标记为 deprecated）

**影响范围**：
- `src/query/planner/rewrite/predicate_pushdown/*.rs`
- `src/query/planner/rewrite/merge/*.rs`
- `src/query/planner/rewrite/aggregate/*.rs`
- `src/query/planner/rewrite/expression_utils.rs`

### 5.4 阶段 4：改造 Planner

**目标**：Planner 使用 ExpressionBuilder

**任务**：
1. 移除所有 `use crate::core::Expression`
2. 使用 ExpressionBuilder 构建表达式
3. 使用 ContextualExpression 传递表达式

**影响范围**：
- `src/query/planner/statements/*.rs`
- `src/query/planner/statements/clauses/*.rs`

### 5.5 阶段 5：清理和优化

**目标**：移除不再需要的代码

**任务**：
1. 移除 `expression_utils.rs`
2. 将 `Expression` 标记为 `pub(crate)`（不对外暴露）
3. 将 `ExpressionMeta` 标记为 `pub(crate)`
4. 更新文档和注释

**影响范围**：
- `src/query/planner/rewrite/expression_utils.rs`
- `src/core/types/expression/mod.rs`
- `src/core/types/expression/expression.rs`

## 六、优势总结

### 6.1 类型安全

```rust
// ✅ 新架构：类型安全
let combined = top_condition.and(child_condition);

// ❌ 旧架构：需要手动处理
let combined = Expression::Binary {
    left: Box::new(child_expr),
    op: BinaryOperator::And,
    right: Box::new(top_expr),
};
```

### 6.2 信息完整

```rust
// ✅ 新架构：保留所有上下文信息
let (picked, remained) = condition.split_filter(picker);
// picked 和 remained 都包含类型、常量、优化状态

// ❌ 旧架构：信息丢失
let (picked, remained) = split_filter(&expr, picker);
// picked 和 remained 只是 Expression，丢失了所有上下文信息
```

### 6.3 代码简洁

```rust
// ✅ 新架构：一步完成
let (picked, remained) = condition.split_filter(picker);

// ❌ 旧架构：三步提取
let filter_condition = filter_node.condition();
let filter_expr = filter_condition.expression()?.inner().clone();
let ctx = filter_condition.context().clone();
let (picked, remained) = split_filter(&filter_expr, picker);
```

### 6.4 性能优化

```rust
// ✅ 新架构：利用常量折叠
if condition.is_constant() {
    return condition.clone();  // 直接使用常量值
}

// ❌ 旧架构：重复计算
let expr = condition.expression()?.inner().clone();
// 每次都要重新计算
```

## 七、风险评估

### 7.1 兼容性风险

**风险**：完全破坏向后兼容性

**缓解**：
- 这是一次彻底的重构，不考虑向后兼容
- 分阶段迁移，每个阶段都可以独立测试
- 保留 Expression 作为内部实现，便于调试

### 7.2 性能风险

**风险**：ContextualExpression 的方法调用可能引入开销

**缓解**：
- ContextualExpression 是轻量级结构（只包含 ID 和 Arc）
- 方法内联优化
- ExpressionContext 使用 DashMap，支持并发访问

### 7.3 学习成本

**风险**：开发者需要学习新的 API

**缓解**：
- 提供详细的文档和示例
- ExpressionBuilder 提供流畅的 API
- 运算符重载使代码更直观

## 八、总结

### 8.1 核心改进

1. **消除三步提取模式**：直接使用 ContextualExpression 的方法
2. **信息完整保留**：类型、常量、优化状态不丢失
3. **类型安全**：所有操作都通过类型系统保证
4. **代码简洁**：减少重复代码，提高可读性
5. **性能优化**：利用常量折叠和缓存

### 8.2 实施建议

1. **分阶段迁移**：按照 5 个阶段逐步实施
2. **充分测试**：每个阶段完成后进行完整测试
3. **文档更新**：及时更新文档和示例
4. **代码审查**：确保所有改动符合新架构

### 8.3 长期收益

1. **维护性提升**：代码更简洁，逻辑更清晰
2. **性能提升**：利用优化信息，减少重复计算
3. **扩展性提升**：易于添加新的表达式操作
4. **类型安全**：减少运行时错误
