# NGQL规划器实现待办事项

## 概述

当前的NGQL规划器（GoPlanner、FetchVerticesPlanner、FetchEdgesPlanner、LookupPlanner、PathPlanner、SubgraphPlanner、MaintainPlanner）由于依赖的上下文结构不完整，暂时无法完全实现。

## 问题分析

### 1. 当前限制

graphDB中的`AstContext`结构过于简单，仅包含以下字段：
- `statement_type`: 语句类型
- `query_text`: 查询文本
- `contains_path`: 是否包含路径查询

而nebula-graph中的查询上下文结构（如`GoContext`、`FetchVerticesContext`等）包含了详细的查询参数，例如：
- 起始顶点信息
- 边类型和方向
- 步数限制
- 过滤条件
- 投影字段
- 索引信息
- 各种子句的详细信息

### 2. 需要的改进

为了实现完整的NGQL规划器功能，需要以下改进：

1. 扩展`AstContext`结构或创建专门的查询上下文结构
2. 实现查询解析器，将SQL文本解析为具体的上下文信息
3. 实现完整的执行计划节点结构
4. 实现规划器的内部逻辑，如边扩展、顶点获取等

## 待办事项

### 高优先级
1. 设计并实现扩展的查询上下文结构
2. 实现查询解析器，将查询文本转换为详细上下文
3. 扩展执行计划节点结构
4. 完成各NGQL规划器的实现

### 低优先级
1. 优化各规划器的执行计划生成逻辑
2. 添加查询优化规则
3. 实现更复杂的查询规划算法

## 依赖关系

各NGQL规划器的实现依赖于：
- 查询解析器的完整实现
- 详细的查询上下文结构
- 执行计划节点的完整实现
- 底层存储接口的实现

## 风险评估

目前规划器返回`PlannerError::UnsupportedOperation`错误，这是一种安全的处理方式，确保了系统不会处理不完整的查询规划。