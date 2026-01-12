# Cypher 查询规划器架构设计

## 概述

本文档说明了 Cypher 查询规划器的正确架构设计，特别是关于起始节点创建和数据流的设计原则。

## 核心设计原则

### 1. 数据流原则

**正确的数据流方向：**
```
上游子句 -> 当前子句 -> 下游子句
```

**错误的做法：**
- 在非起始子句中创建起始节点
- 假设任何子句都可以作为查询的起点
- 忽略子句之间的依赖关系

### 2. 起始节点创建原则

**应该创建起始节点的子句：**
- `MATCH` 子句：查询的真正起点，从图中读取数据
- 查询级别的起始点：如独立的 `WITH` 子句开始新查询

**不应该创建起始节点的子句：**
- `RETURN` 子句：必须接收上游数据
- `UNWIND` 子句：必须接收上游数据
- `WHERE` 子句：过滤上游数据
- `ORDER BY` 子句：排序上游数据
- `LIMIT/OFFSET` 子句：分页上游数据
- `DISTINCT` 子句：去重上游数据

### 3. 子句规划器职责

每个子句规划器的职责：

1. **处理自己的逻辑**：专注于子句特定的功能
2. **验证输入**：确保接收到的上下文有效
3. **创建相应的计划节点**：但不创建起始节点（除非是真正的起始子句）
4. **设置节点属性**：配置执行所需的信息
5. **返回子计划**：供上层规划器连接

## 正确的实现模式

### 模式 1：处理型子句（如 RETURN、UNWIND）

```rust
impl CypherClausePlanner for SomeClausePlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        // 1. 验证上下文
        validate_context(clause_ctx)?;
        
        // 2. 提取子句特定上下文
        let ctx = extract_context(clause_ctx)?;
        
        // 3. 验证子句特定条件
        validate_clause_specific(&ctx)?;
        
        // 4. 创建节点（不创建起始节点）
        let node = create_node_without_input(&ctx)?;
        
        // 5. 返回子计划（由上层连接）
        Ok(SubPlan::new(Some(node.clone()), Some(node)))
    }
}
```

### 模式 2：起始型子句（如 MATCH）

```rust
impl CypherClausePlanner for MatchClausePlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        // 1. 验证上下文
        validate_context(clause_ctx)?;
        
        // 2. 提取子句特定上下文
        let ctx = extract_context(clause_ctx)?;
        
        // 3. 创建起始节点
        let start_node = create_start_node()?;
        
        // 4. 创建 MATCH 节点，使用起始节点作为输入
        let match_node = create_match_node(&ctx, start_node)?;
        
        // 5. 返回完整的计划
        Ok(SubPlan::new(Some(match_node.clone()), Some(match_node)))
    }
}
```

## 数据流连接

### 连接函数的设计

```rust
/// 将子计划连接到输入计划
pub fn connect_clause_to_input(
    input_plan: SubPlan,
    clause_plan: SubPlan,
) -> Result<SubPlan, PlannerError> {
    // 1. 验证输入计划
    if input_plan.root.is_none() {
        return Err(PlannerError::PlanGenerationFailed(
            "子句必须有有效的输入计划".to_string(),
        ));
    }
    
    // 2. 使用连接器连接
    let connector = SegmentsConnector::new();
    let connected_plan = connector.add_input(clause_plan, input_plan, true);
    
    Ok(connected_plan)
}
```

### 查询级别的连接

查询规划器负责：
1. 识别起始子句
2. 创建起始节点
3. 按顺序连接各个子句
4. 确保数据流的正确性

## 常见错误和解决方案

### 错误 1：在 RETURN 中创建起始节点

**问题：**
```rust
// 错误的做法
let start_node = create_start_node()?;
let return_node = SingleInputNode::new(PlanNodeKind::Project, start_node);
```

**解决方案：**
```rust
// 正确的做法
if plan.root.is_none() {
    return Err(PlannerError::PlanGenerationFailed(
        "RETURN 子句必须有输入数据源".to_string(),
    ));
}
let return_node = SingleInputNode::new(PlanNodeKind::Project, plan.root.unwrap());
```

### 错误 2：忽略输入验证

**问题：**
```rust
// 错误的做法
let node = create_node(ctx);
Ok(SubPlan::new(Some(node), Some(node)))
```

**解决方案：**
```rust
// 正确的做法
if input_plan.root.is_none() {
    return Err(PlannerError::PlanGenerationFailed(
        "子句必须有有效的输入计划".to_string(),
    ));
}
let node = create_node(ctx);
Ok(SubPlan::new(Some(node), Some(node)))
```

### 错误 3：硬编码起始节点

**问题：**
```rust
// 错误的做法
let placeholder = SingleDependencyNode {
    id: -1,
    kind: PlanNodeKind::Start,
    // ...
};
```

**解决方案：**
```rust
// 正确的做法
// 让上层规划器处理连接，子规划器只创建自己的节点
let node = create_clause_node(ctx);
// 返回不带输入的节点，由上层连接
```

## 最佳实践

### 1. 明确职责分离

- **子句规划器**：专注于子句逻辑
- **查询规划器**：负责整体连接和数据流
- **连接器**：提供标准的连接机制

### 2. 错误处理

- 验证输入的有效性
- 提供清晰的错误信息
- 使用中文错误信息提高可读性

### 3. 文档和注释

- 明确说明每个函数的职责
- 记录数据流的方向
- 说明设计决策的原因

### 4. 测试策略

- 测试正常的数据流
- 测试错误情况（如缺少输入）
- 测试边界条件

## 总结

正确的 Cypher 查询规划器架构应该：

1. **明确数据流方向**：从上游到下游
2. **正确创建起始节点**：只在真正的起始点创建
3. **职责分离**：每个组件专注于自己的职责
4. **完善的错误处理**：验证输入并提供清晰的错误信息
5. **标准化的连接机制**：使用统一的连接器

这种设计确保了查询规划的正确性、可维护性和可扩展性。