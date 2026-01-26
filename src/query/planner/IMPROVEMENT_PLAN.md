# Planner 模块改进建议

## 一、Parser → Planner 集成问题

### 1.1 当前问题

Planner 未正确使用 Parser 生成的 AST：

```rust
// statements/match_planner.rs:56-61
fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
    let _stmt = ast_ctx.sentence().ok_or_else(|| {  // ← 只检查是否存在
        PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
    })?;
    // stmt 未被使用，实际执行的是硬编码逻辑
}
```

### 1.2 正确用法

Planner 应该从 `stmt` 中提取信息进行规划：

```rust
// 正确做法
fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
    let stmt = ast_ctx.sentence().ok_or_else(|| {
        PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
    })?;

    // 从 Stmt 中提取模式信息
    match &stmt {
        Statement::Match(match_stmt) => {
            // 使用 match_stmt 中的路径、WHERE 条件等进行规划
        }
        Statement::Go(go_stmt) => {
            // 使用 go_stmt 中的遍历信息进行规划
        }
        _ => Err(PlannerError::InvalidAstContext(
            "不支持的语句类型".to_string()
        ))
    }
}
```

### 1.3 架构结论

| 问题 | 结论 |
|------|------|
| Parser → Planner 转换链路多余？ | **不是**，分离设计是合理的 |
| 直接使用统一类型？ | **不可行**，语法 AST 与执行计划是不同抽象层次 |
| 当前问题 | 转换链路**未完成**，Planner 未使用 Parser 结果 |

---

## 二、需要修改的问题清单

### 2.1 高优先级问题

#### P1. Planner 未使用 Parser 结果

**位置**: `src/query/planner/statements/match_planner.rs`
**问题**: `transform()` 方法中 `_stmt` 未被使用
**修改**: 从 `stmt` 中提取 MATCH 语句的路径和条件信息

#### P2. MatchPlanner 功能不完整

**位置**: `src/query/planner/statements/match_planner.rs`
**问题**: 只创建了空的 `ScanVerticesNode`，未处理：
- 节点模式 `(n:Tag {prop: value})`
- 边模式 `-[e:Edge]->`
- WHERE 条件过滤
- 多路径匹配
**修改**: 实现完整的 MATCH 模式解析和规划

#### P3. MatchClausePlanner 路径处理未实现

**位置**: `src/query/planner/statements/core/match_clause_planner.rs:85`
**问题**: TODO 注释表明路径处理逻辑未实现
**修改**: 实现路径遍历和变量绑定逻辑

### 2.2 中优先级问题

#### P4. 移除硬编码数据

**位置**: `src/query/planner/statements/match_planner.rs` (旧版本)
**问题**: 存在硬编码的 `MatchClauseContext`
**修改**: 从 `stmt` 动态生成，而非使用测试数据

#### P5. 统一错误处理

**位置**: 多处 `connector.rs`, `join_node.rs`
**问题**: 使用 `unwrap()` 和 `panic!` 而非 `Result`
**修改**: 返回 `Result<SubPlan, PlannerError>`

```rust
// 错误做法
.join_node(...).unwrap()

// 正确做法
.join_node(...)
    .map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?
```

#### P6. 完成管理节点执行器

**位置**: `src/executor/factory.rs`
**问题**: `ManagementNodeEnum` 变体未实现执行器
**修改**: 为所有管理节点实现 `AdminExecutor`

### 2.3 低优先级问题

#### P7. 拆分庞大的 PlanNodeEnum

**位置**: `src/query/planner/plan/core/nodes/plan_node_enum.rs`
**问题**: 50+ 变体导致编译时间增加和维护困难
**修改**: 按功能拆分为多个枚举

```rust
// 拆分为：
pub enum QueryNodeEnum { /* 查谽操作 */ }
pub enum AdminNodeEnum { /* 管理操作 */ }
pub enum DataNodeEnum { /* 数据操作 */ }
```

#### P8. 统一表达式处理框架

**位置**: `Evaluator`, `Optimizer`, `Visitor` 模块
**问题**: 表达式处理逻辑分散
**修改**: 创建统一的 `ExpressionProcessor` trait

---

## 三、具体修改建议

### 3.1 MatchPlanner 完整实现

```rust
impl Planner for MatchPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        let stmt = ast_ctx.sentence().ok_or_else(|| {
            PlannerError::InvalidAstContext("AstContext 中缺少语句".to_string())
        })?;

        // 解析 Statement::Match 获取模式信息
        let match_stmt = match &stmt {
            Statement::Match(m) => m,
            _ => return Err(PlannerError::InvalidAstContext(
                "MatchPlanner 需要 Match 语句".to_string()
            )),
        };

        // 1. 创建起始节点
        let space_id = ast_ctx.space.space_id.unwrap_or(1) as i32;
        let start_node = ScanVerticesNode::new(space_id);
        let mut current_plan = SubPlan::from_root(start_node.into_enum());

        // 2. 处理每个路径模式
        for path in &match_stmt.patterns {
            // 2.1 处理节点模式
            for node in &path.nodes {
                let node_plan = self.plan_node_pattern(node, ast_ctx)?;
                current_plan = self.join_plans(current_plan, node_plan)?;
            }

            // 2.2 处理边模式
            for edge in &path.edges {
                let edge_plan = self.plan_edge_pattern(edge, ast_ctx)?;
                current_plan = self.join_plans(current_plan, edge_plan)?;
            }
        }

        // 3. 处理 WHERE 条件
        if let Some(where_cond) = &match_stmt.where_condition {
            let filter_node = self.plan_filter(where_cond)?;
            current_plan = self.add_node_to_plan(current_plan, filter_node)?;
        }

        Ok(current_plan)
    }
}
```

### 3.2 错误处理改进

```rust
// connector.rs 中的 inner_join 方法

// 修改前
.inner_join(...).unwrap()

// 修改后
.inner_join(...)
    .map_err(|e| PlannerError::JoinFailed(e.to_string()))?
```

### 3.3 路径处理实现

```rust
// match_clause_planner.rs

fn plan_path(
    &self,
    path: &Path,
    context: &mut PlanningContext,
) -> Result<SubPlan, PlannerError> {
    let mut plan = SubPlan::new(None, None);

    for (index, node) in path.nodes.iter().enumerate() {
        // 1. 创建节点扫描
        let scan_node = self.create_node_scan(node, context)?;

        // 2. 添加属性过滤
        if let Some(filter) = &node.filter {
            let filter_node = self.create_filter(filter)?;
            plan = self.compose_plan(plan, filter_node)?;
        }

        // 3. 绑定变量到上下文
        if !node.alias.is_empty() {
            context.add_variable(VariableInfo {
                name: node.alias.clone(),
                var_type: "Vertex".to_string(),
                source_clause: ClauseType::Match,
                is_output: false,
            });
        }
    }

    Ok(plan)
}
```

---

## 四、实施优先级

| 优先级 | 任务 | 预计工作量 | 影响范围 |
|--------|------|-----------|----------|
| P1 | 完成 Parser → Planner 集成 | 大 | 所有查询语句 |
| P2 | 实现 MatchPlanner 完整功能 | 大 | MATCH 查询 |
| P3 | 实现路径处理逻辑 | 中 | MATCH 多路径 |
| P4 | 移除硬编码数据 | 小 | 测试代码 |
| P5 | 统一错误处理 | 中 | 全局 |
| P6 | 完成管理节点执行器 | 大 | DDL/DML 语句 |
| P7 | 拆分 PlanNodeEnum | 大 | 编译时间 |
| P8 | 统一表达式框架 | 大 | 架构重构 |

---

## 五、验收标准

### 必须完成

- [ ] MatchPlanner 正确使用 `stmt` 进行规划
- [ ] MATCH 语句能解析节点、边、WHERE 条件
- [ ] 移除所有硬编码的测试数据
- [ ] 移除 `unwrap()` 和 `panic!`，使用 `Result`
- [ ] ScanEdgesExecutor 实现完成

### 建议完成

- [ ] 管理节点执行器实现
- [ ] PlanNodeEnum 按功能拆分
- [ ] 表达式处理框架统一

---

## 六、相关文档

- 与 Parser 的集成: [PARSER_IMPROVEMENT_PLAN.md](../parser/__analysis__/PARSER_IMPROVEMENT_PLAN.md)
- 架构分析: [modules_architecture_analysis.md](../__analysis__/modules_architecture_analysis.md)
- 计划节点分析: [plan_node_implementation_analysis.md](./plan/core/nodes/plan_node_implementation_analysis.md)
