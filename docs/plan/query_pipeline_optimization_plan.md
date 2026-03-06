# 查询管道优化计划

## 概述

本文档详细描述了查询处理管道（Query Pipeline）的优化方案，包括四个阶段的修改计划。

## 当前架构分析

### 查询处理流程

```
查询字符串
    │
    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ 1. 解析阶段 (Parser)                                                    │
│    输入: &str (查询字符串)                                               │
│    输出: ParserResult { ast: Arc<Ast> }                                 │
│    Ast = { stmt: Stmt, expr_context: Arc<ExpressionAnalysisContext> }   │
└────────────────────────────────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ 2. 验证阶段 (Validator)                                                 │
│    输入: Arc<Ast>                                                       │
│    输出: ValidationInfo + ValidatedStatement                            │
└────────────────────────────────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ 3. 规划阶段 (Planner)                                                   │
│    输入: ValidatedStatement + Arc<QueryContext>                         │
│    输出: ExecutionPlan                                                  │
└────────────────────────────────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ 4. 优化阶段 (Optimizer)                                                 │
│    输入: ExecutionPlan                                                  │
│    输出: ExecutionPlan (优化后的)                                        │
└────────────────────────────────────────────────────────────────────────┘
    │
    ▼
┌────────────────────────────────────────────────────────────────────────┐
│ 5. 执行阶段 (Executor)                                                  │
│    输入: ExecutionPlan + Arc<QueryContext>                              │
│    输出: ExecutionResult                                                │
└────────────────────────────────────────────────────────────────────────┘
```

### 当前存在的问题

#### 问题1: QueryContext 重复创建

在 `query_pipeline_manager.rs` 中，验证阶段创建了临时 QueryContext：

```rust
fn validate_query_without_context(...) -> DBResult<ValidationInfo> {
    // 创建临时的 QueryContext 用于验证
    let temp_rctx = Arc::new(QueryRequestContext::new(String::new()));
    let temp_qctx = Arc::new(QueryContext::new(temp_rctx));
    // ...
}
```

**影响：**
- 临时上下文在执行阶段被丢弃
- `QueryResourceContext` 中的 ID 生成器、对象池等资源需要重建
- 符号表信息丢失

#### 问题2: ValidationInfo 字段冗余

| 字段 | 使用情况 | 建议 |
|------|---------|------|
| `alias_map` | 高 | 保留 |
| `path_analysis` | 中 | 保留 |
| `optimization_hints` | 高 | 保留 |
| `index_hints` | 高 | 保留 |
| `semantic_info` | 中 | 保留 |
| `variable_definitions` | 未使用 | **移除** |
| `validated_clauses` | 未使用 | **移除** |

#### 问题3: 表达式缓存无限制

当前 `ExpressionAnalysisContext` 使用 `DashMap` 存储缓存：
- 无过期机制
- 无大小限制
- 查询间不共享

## 优化阶段

### 第一阶段：统一 QueryContext 生命周期

**目标：** 消除临时 QueryContext 的创建，实现单一实例贯穿整个查询生命周期

**修改内容：**

1. 修改 `QueryPipelineManager::execute_query_with_profile`
   - 将 QueryContext 创建提前到验证阶段之前
   - 验证阶段复用已创建的上下文

2. 修改 `validate_query_without_context` 函数
   - 更名为 `validate_query_with_context`
   - 接受 `Arc<QueryContext>` 参数

**预期收益：** 减少 10-20% 内存分配

**实现复杂度：** 低

---

### 第二阶段：实现全局表达式缓存

**目标：** 实现跨查询共享的表达式分析结果缓存

**新增文件：**
- `src/query/validator/context/expression_cache.rs` - 全局表达式缓存实现

**修改内容：**

1. 创建 `GlobalExpressionCache` 结构
   - 使用 LRU 缓存策略
   - 支持类型缓存、常量缓存、分析缓存
   - 可配置大小限制和 TTL

2. 修改 `ExpressionAnalysisContext`
   - 集成全局缓存引用
   - 保持向后兼容的 API

3. 修改 `QueryPipelineManager`
   - 初始化全局表达式缓存
   - 传递给解析器和验证器

**预期收益：** 提升 20-40% 重复查询性能

**实现复杂度：** 中

---

### 第三阶段：清理 ValidationInfo 字段

**目标：** 移除未使用的字段，减少内存占用

**修改内容：**

1. 修改 `ValidationInfo` 结构
   - 移除 `variable_definitions` 字段
   - 移除 `validated_clauses` 字段

2. 更新所有使用 `ValidationInfo` 的代码
   - 检查并修复编译错误

**预期收益：** 减少 5-10% 内存占用

**实现复杂度：** 低

---

### 第四阶段：实现查询计划缓存

**目标：** 缓存相同查询模板的执行计划

**新增文件：**
- `src/query/planner/plan_cache.rs` - 查询计划缓存实现

**修改内容：**

1. 创建 `QueryPlanCache` 结构
   - 基于查询模板哈希的缓存键
   - 支持参数化查询
   - 版本感知（schema 变更时失效）

2. 修改 `QueryPipelineManager`
   - 集成计划缓存
   - 缓存命中时跳过解析和规划阶段

**预期收益：** 提升 50-80% prepared query 性能

**实现复杂度：** 高

## 实施顺序

1. **第一阶段**：统一 QueryContext 生命周期（低风险，明显收益）
2. **第二阶段**：实现全局表达式缓存（中等风险，高收益）
3. **第三阶段**：清理 ValidationInfo 字段（低风险，维护性提升）
4. **第四阶段**：实现查询计划缓存（高复杂度，最高收益）

## 性能预期汇总

| 优化项 | 预期收益 | 实现复杂度 |
|--------|---------|-----------|
| QueryContext 生命周期统一 | 减少 10-20% 内存分配 | 低 |
| ValidationInfo 字段清理 | 减少 5-10% 内存占用 | 低 |
| 全局表达式缓存 | 提升 20-40% 重复查询性能 | 中 |
| 查询计划缓存 | 提升 50-80% prepared query 性能 | 高 |

## 相关文件

- `src/query/query_pipeline_manager.rs` - 查询管道管理器
- `src/query/query_context.rs` - 查询上下文
- `src/query/validator/structs/validation_info.rs` - 验证信息
- `src/query/validator/context/expression_context.rs` - 表达式上下文
- `src/query/optimizer/decision/cache.rs` - 决策缓存（参考实现）
