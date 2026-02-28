# 表达式系统集成方案

## 概述

本文档描述 src/query 目录各层如何集成 expression 系统，确保数据正确传递，避免重复解析和字符串使用。

## 一、各层集成方案

### 1.1 Parser 层

**职责**：解析 SQL 字符串，生成 AST 和表达式上下文

**输入**：SQL 字符串

**输出**：
- AST (Stmt)：包含 ContextualExpression
- ExpressionContext：表达式上下文

**关键操作**：
1. ExprParser 解析表达式，生成 Expression
2. 将 Expression 包装为 ExpressionMeta
3. 注册到 ExpressionContext，获得 ExpressionId
4. 创建 ContextualExpression (ExpressionId + ExpressionContext)
5. 构建包含 ContextualExpression 的 AST

**设计要点**：
- Parser 是唯一创建 Expression 的地方
- 立即注册到 ExpressionContext，不延迟
- AST 层面使用 ContextualExpression

### 1.2 Validator 层

**职责**：语义检查、类型推导、常量折叠

**输入**：AST + ExpressionContext

**输出**：ValidatedStatement + ExpressionContext (优化后)

**关键操作**：
1. 使用 ContextualExpression 获取变量列表
2. ExpressionAnalyzer 进行类型推导
3. 将类型信息存储到 ExpressionContext
4. 进行常量折叠，存储结果到 ExpressionContext
5. 生成 ValidatedStatement

**设计要点**：
- 通过 ContextualExpression.expression() 获取 Expression
- 分析结果存储到 ExpressionContext
- 不创建新的 Expression，只更新元数据

### 1.3 Planner 层

**职责**：将 AST 转换为执行计划

**输入**：ValidatedStatement + ExpressionContext

**输出**：ExecutionPlan + ExpressionContext

**关键操作**：
1. QueryContext 持有 Arc<ExpressionContext>
2. 从 AST 提取 ContextualExpression
3. 创建 PlanNode，只接受 ContextualExpression
4. 构建计划树，所有节点使用 ContextualExpression

**设计要点**：
- 所有 PlanNode 只接受 ContextualExpression
- 删除所有 from_expression() 等转换方法
- Planner 层不接触 Expression

### 1.4 Rewrite 层

**职责**：应用启发式重写规则

**输入**：ExecutionPlan + ExpressionContext

**输出**：ExecutionPlan (重写后) + ExpressionContext (更新)

**关键操作**：
1. RewriteContext 持有 Arc<ExpressionContext>
2. 重写规则操作 ContextualExpression
3. 创建新表达式时注册到 ExpressionContext
4. 更新计划树中的 ContextualExpression

**设计要点**：
- Rewrite 作为独立阶段
- 共享同一个 ExpressionContext
- 创建新表达式必须注册到 context

### 1.5 Optimizer 层

**职责**：基于代价的优化、表达式分析

**输入**：ExecutionPlan + ExpressionContext

**输出**：ExecutionPlan (优化后) + ExpressionContext (更新)

**关键操作**：
1. ExpressionAnalyzer 分析 ContextualExpression
2. 类型推导，存储到 ExpressionContext
3. 常量折叠，存储到 ExpressionContext
4. 代价估算使用分析结果
5. 计划优化

**设计要点**：
- 利用 ExpressionContext 缓存
- 避免重复分析
- 分析结果存储到 ExpressionContext

### 1.6 Executor 层

**职责**：执行查询计划

**输入**：ExecutionPlan + ExpressionContext

**输出**：ExecutionResult

**关键操作**：
1. ExecutorFactory 从 PlanNode 创建 Executor
2. 从 ContextualExpression 提取 Expression
3. 使用 ExpressionEvaluator 执行表达式
4. 返回执行结果

**设计要点**：
- PlanNode 持有 ContextualExpression
- Executor 提取 Expression 用于执行
- 不修改 ExpressionContext

## 二、完整数据流

### 2.1 数据流图

```
Parser → Validator → Planner → Rewrite → Optimizer → Executor
   ↓         ↓          ↓         ↓          ↓          ↓
 AST      Validated    Plan     Plan       Plan       Result
Context   Statement    (raw)   (rewrite)  (opt)
```

### 2.2 详细流程

**阶段 1：Parser**
- 输入：SQL 字符串
- 创建 ExpressionContext
- 解析表达式 → Expression
- 注册 → ExpressionMeta → ExpressionContext
- 创建 ContextualExpression
- 输出：AST + ExpressionContext

**阶段 2：Validator**
- 输入：AST + ExpressionContext
- 使用 ContextualExpression 进行语义检查
- 类型推导 → ExpressionContext
- 常量折叠 → ExpressionContext
- 输出：ValidatedStatement + ExpressionContext

**阶段 3：Planner**
- 输入：ValidatedStatement + ExpressionContext
- QueryContext 持有 Arc<ExpressionContext>
- 提取 ContextualExpression
- 创建 PlanNode (只接受 ContextualExpression)
- 输出：ExecutionPlan + ExpressionContext

**阶段 4：Rewrite**
- 输入：ExecutionPlan + ExpressionContext
- RewriteContext 持有 Arc<ExpressionContext>
- 重写规则操作 ContextualExpression
- 创建新表达式 → 注册到 ExpressionContext
- 输出：ExecutionPlan (重写后) + ExpressionContext

**阶段 5：Optimizer**
- 输入：ExecutionPlan + ExpressionContext
- ExpressionAnalyzer 分析 ContextualExpression
- 类型推导、常量折叠 → ExpressionContext
- 代价估算、计划优化
- 输出：ExecutionPlan (优化后) + ExpressionContext

**阶段 6：Executor**
- 输入：ExecutionPlan + ExpressionContext
- 从 ContextualExpression 提取 Expression
- 使用 ExpressionEvaluator 执行
- 输出：ExecutionResult

## 三、关键设计原则

### 3.1 单一数据源

- Parser 层是唯一创建 Expression 的地方
- 后续所有层只使用 ContextualExpression
- 所有表达式数据来自上一个阶段
- 禁止从字符串重新解析表达式

### 3.2 上下文共享

- QueryContext 持有 Arc<ExpressionContext>
- ExpressionContext 跨阶段共享
- 通过 Arc 实现并发安全
- 所有阶段访问同一个 ExpressionContext

### 3.3 类型安全

- Planner 层只能使用 ContextualExpression
- PlanNode 只接受 ContextualExpression
- Rewrite 规则只操作 ContextualExpression
- 编译时保证类型安全

### 3.4 信息完整

- ContextualExpression 保留所有上下文信息
- 通过 ExpressionContext 访问类型、常量、优化状态
- 避免信息丢失
- 支持跨阶段信息传递

## 四、数据传递保证

### 4.1 Parser → Validator

- Parser 返回：AST (with ContextualExpression) + ExpressionContext
- Validator 接收：AST + ExpressionContext
- Validator 使用 ContextualExpression 进行验证
- 不创建新的 Expression

### 4.2 Validator → Planner

- Validator 返回：ValidatedStatement (with ContextualExpression)
- Planner 接收：ValidatedStatement + QueryContext (包含 ExpressionContext)
- Planner 从 AST 提取 ContextualExpression
- 不进行转换，直接使用

### 4.3 Planner → Rewrite

- Planner 返回：ExecutionPlan (with ContextualExpression)
- Rewrite 接收：ExecutionPlan + ExpressionContext
- Rewrite 操作 ContextualExpression
- 创建新表达式时注册到 ExpressionContext

### 4.4 Rewrite → Optimizer

- Rewrite 返回：ExecutionPlan (with ContextualExpression, 更新后)
- Optimizer 接收：ExecutionPlan + ExpressionContext
- Optimizer 分析 ContextualExpression
- 分析结果存储到 ExpressionContext

### 4.5 Optimizer → Executor

- Optimizer 返回：ExecutionPlan (with ContextualExpression, 优化后)
- Executor 接收：ExecutionPlan
- Executor 从 ContextualExpression 提取 Expression
- 使用 ExpressionEvaluator 执行

## 五、避免重复解析的措施

### 5.1 禁止字符串解析

- 禁止从字符串解析表达式
- 禁止使用 parse_expression() 等函数
- 所有表达式必须来自上一个阶段

### 5.2 禁止重复创建 Expression

- 禁止从 ContextualExpression 创建新的 Expression
- 禁止直接操作 Expression
- 创建新表达式必须注册到 ExpressionContext

### 5.3 使用缓存

- ExpressionContext 提供类型缓存
- ExpressionContext 提供常量缓存
- 避免重复计算
- 利用缓存提高性能

### 5.4 统一接口

- 所有层只使用 ContextualExpression
- 不暴露 Expression 给外部
- 通过 ExpressionContext 访问完整信息
- 保证数据一致性

## 六、关键改造点

### 6.1 Parser 层

- ExprParser 返回 ContextualExpression
- AST 使用 ContextualExpression
- 立即注册到 ExpressionContext

### 6.2 QueryContext

- 添加 Arc<ExpressionContext> 字段
- 提供 expr_context() 访问方法
- 跨阶段共享 ExpressionContext

### 6.3 PlanNode

- 只接受 ContextualExpression
- 删除 from_expression() 等转换方法
- 删除所有 Expression 相关方法

### 6.4 RewriteContext

- 添加 Arc<ExpressionContext> 字段
- 提供 expr_context() 访问方法
- 重写规则操作 ContextualExpression

### 6.5 Executor

- 从 ContextualExpression 提取 Expression
- 使用 ExpressionEvaluator 执行
- 不修改 ExpressionContext

### 6.6 ExpressionAnalyzer

- 接受 ContextualExpression
- 分析结果存储到 ExpressionContext
- 利用缓存避免重复分析

## 七、总结

### 7.1 核心原则

1. Parser 层唯一创建 Expression
2. 后续层只使用 ContextualExpression
3. ExpressionContext 跨阶段共享
4. 数据完全来自上一阶段
5. 类型安全和信息完整

### 7.2 关键目标

- 消除 planner 层对 Expression 的直接依赖
- 避免重复解析和字符串使用
- 保证数据正确传递
- 实现类型安全和信息完整
- 优化性能，利用缓存

### 7.3 实施要点

- Parser 层：立即注册表达式
- Validator 层：分析并存储元数据
- Planner 层：只使用 ContextualExpression
- Rewrite 层：操作并注册新表达式
- Optimizer 层：分析并利用缓存
- Executor 层：提取并执行表达式

### 7.4 验证标准

- 无从字符串解析表达式的代码
- 无重复创建 Expression 的代码
- 所有 PlanNode 使用 ContextualExpression
- ExpressionContext 跨阶段共享
- 数据流清晰，无歧义
