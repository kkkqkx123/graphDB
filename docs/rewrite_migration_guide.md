# Planner Rewrite 规则迁移指南

## 迁移背景

将 `src/query/planner/rewrite` 目录下的规则从依赖 `optimizer` 层迁移到独立的 `planner` 层实现。

## 已完成的工作

### 1. 基础类型文件（已完成）
- `src/query/planner/rewrite/context.rs` - 重写上下文
- `src/query/planner/rewrite/pattern.rs` - 模式匹配
- `src/query/planner/rewrite/rule.rs` - 规则 trait 定义
- `src/query/planner/rewrite/result.rs` - 结果类型

### 2. 核心实现文件（已完成）
- `src/query/planner/rewrite/mod.rs` - 模块导出
- `src/query/planner/rewrite/rewrite_rule.rs` - 启发式规则适配
- `src/query/planner/rewrite/plan_rewriter.rs` - 计划重写器
- `src/query/planner/rewrite/rule_enum.rs` - 规则枚举

### 3. Elimination 规则（已完成 - 7个）
- ✅ `eliminate_filter.rs` - 消除冗余过滤
- ✅ `remove_noop_project.rs` - 移除无操作投影
- ✅ `eliminate_append_vertices.rs` - 消除冗余添加顶点
- ✅ `remove_append_vertices_below_join.rs` - 移除连接下方的添加顶点
- ✅ `eliminate_row_collect.rs` - 消除冗余数据收集
- ✅ `eliminate_empty_set_operation.rs` - 空集操作优化
- ✅ `dedup_elimination.rs` - 消除重复操作

## 待完成的任务

### 1. Merge 规则（7个）
- [ ] `combine_filter.rs` - 合并过滤条件
- [ ] `collapse_project.rs` - 折叠投影
- [ ] `collapse_consecutive_project.rs` - 折叠连续投影
- [ ] `merge_get_vertices_and_project.rs` - 合并获取顶点和投影
- [ ] `merge_get_vertices_and_dedup.rs` - 合并获取顶点和去重
- [ ] `merge_get_nbrs_and_project.rs` - 合并获取邻居和投影
- [ ] `merge_get_nbrs_and_dedup.rs` - 合并获取邻居和去重

### 2. Predicate Pushdown 规则（12个）
- [ ] `push_filter_down_traverse.rs` - 下推过滤到遍历
- [ ] `push_filter_down_expand_all.rs` - 下推过滤到扩展
- [ ] `push_filter_down_join.rs` - 下推过滤到连接
- [ ] `push_filter_down_node.rs` - 下推过滤到节点
- [ ] `push_efilter_down.rs` - 下推边过滤
- [ ] `push_vfilter_down_scan_vertices.rs` - 下推顶点过滤到扫描
- [ ] `push_filter_down_inner_join.rs` - 下推过滤到内连接
- [ ] `push_filter_down_hash_inner_join.rs` - 下推过滤到哈希内连接
- [ ] `push_filter_down_hash_left_join.rs` - 下推过滤到哈希左连接
- [ ] `push_filter_down_cross_join.rs` - 下推过滤到交叉连接
- [ ] `push_filter_down_get_nbrs.rs` - 下推过滤到获取邻居
- [ ] `push_filter_down_all_paths.rs` - 下推过滤到全路径

### 3. Projection Pushdown 规则（2个）
- [ ] `projection_pushdown.rs` - 投影下推
- [ ] `push_project_down.rs` - 下推投影

### 4. Limit Pushdown 规则（6个）
- [ ] `push_limit_down_get_vertices.rs` - 下推限制到获取顶点
- [ ] `push_limit_down_get_edges.rs` - 下推限制到获取边
- [ ] `push_limit_down_scan_vertices.rs` - 下推限制到扫描顶点
- [ ] `push_limit_down_scan_edges.rs` - 下推限制到扫描边
- [ ] `push_limit_down_index_scan.rs` - 下推限制到索引扫描
- [ ] `push_topn_down_index_scan.rs` - 下推 TopN 到索引扫描

### 5. Aggregate 规则（1个）
- [ ] `push_filter_down_aggregate.rs` - 下推过滤到聚合

## 修改原则

### 1. 导入替换

**旧导入（删除）:**
```rust
use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, OptimizerError};
use crate::query::optimizer::rule_traits::{create_basic_pattern, BaseOptRule};
use crate::query::planner::plan::core::nodes::plan_node_visitor::PlanNodeVisitor;
use std::rc::Rc;
use std::cell::RefCell;
```

**新导入（使用）:**
```rust
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::{具体节点类型};
use crate::query::planner::plan::core::nodes::plan_node_traits::{SingleInputNode, BinaryInputNode};
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule, MergeRule, PushDownRule};
```

### 2. 规则结构替换

**旧结构（删除）:**
```rust
crate::define_elimination_rule! {
    pub struct RuleName {
        target: TargetType,
        target_check: check_method,
        pattern: Pattern::new_with_name("NodeName")
    }
    visitor: VisitorName
}

#[derive(Clone)]
struct VisitorName<'a> {
    is_eliminated: bool,
    eliminated_node: Option<OptGroupNode>,
    ctx: &'a OptContext,
}

impl<'a> PlanNodeVisitor for VisitorName<'a> {
    type Result = Self;
    // ... 实现
}
```

**新结构（使用）:**
```rust
#[derive(Debug)]
pub struct RuleName;

impl RuleName {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RuleName {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for RuleName {
    fn name(&self) -> &'static str {
        "RuleName"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("NodeName")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 实现规则逻辑
    }
}

// 根据需要实现特定 trait
impl EliminationRule for RuleName {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        // 实现
    }

    fn eliminate(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(_ctx, node)
    }
}
```

### 3. 规则逻辑转换

**旧逻辑（基于访问者模式）:**
```rust
fn apply(
    &self,
    ctx: &mut OptContext,
    group_node: &Rc<RefCell<OptGroupNode>>,
) -> Result<Option<TransformResult>, OptimizerError> {
    let node_ref = group_node.borrow();
    let mut visitor = RuleVisitor {
        ctx,
        is_eliminated: false,
        eliminated_node: None,
    };

    let result = node_ref.plan_node.accept(&mut visitor);
    drop(node_ref);

    if result.is_eliminated {
        if let Some(new_node) = result.eliminated_node {
            let mut transform_result = TransformResult::new();
            transform_result.add_new_group_node(Rc::new(RefCell::new(new_node)));
            return Ok(Some(transform_result));
        }
    }
    Ok(None)
}
```

**新逻辑（直接处理）:**
```rust
fn apply(
    &self,
    _ctx: &mut RewriteContext,
    node: &PlanNodeEnum,
) -> RewriteResult<Option<TransformResult>> {
    // 检查节点类型
    let target_node = match node {
        PlanNodeEnum::Target(n) => n,
        _ => return Ok(None),
    };

    // 执行条件检查
    if !self.should_apply(target_node) {
        return Ok(None);
    }

    // 获取输入节点
    let input = target_node.input();

    // 创建转换结果
    let mut result = TransformResult::new();
    result.erase_curr = true;
    result.add_new_node(input.clone());

    Ok(Some(result))
}
```

### 4. 节点访问方式

**旧方式:**
```rust
let deps = node.dependencies();
let input = deps.first().unwrap();
let input_id = input.id() as usize;
if let Some(child_node) = self.ctx.find_group_node_by_plan_node_id(input_id) {
    // ...
}
```

**新方式:**
```rust
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

let input = node.input();  // 直接获取输入
// 或者使用 BinaryInputNode 获取左右输入
let left = node.left_input();
let right = node.right_input();
```

### 5. 结果返回方式

**旧方式:**
```rust
let mut result = TransformResult::new();
result.add_new_group_node(Rc::new(RefCell::new(new_node)));
Ok(Some(result))
```

**新方式:**
```rust
let mut result = TransformResult::new();
result.erase_curr = true;  // 标记删除当前节点
result.add_new_node(new_node);
Ok(Some(result))
```

## 规则类型对应关系

| 旧宏 | 新 Trait | 适用场景 |
|------|----------|----------|
`define_elimination_rule!` | `EliminationRule` | 消除冗余节点 |
`define_merge_rule!` | `MergeRule` | 合并连续节点 |
`define_pushdown_rule!` | `PushDownRule` | 下推操作到子节点 |
`define_limit_pushdown_rule!` | `PushDownRule` | 下推 Limit 操作 |

## 测试要求

每个规则文件应包含基本测试:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_name() {
        let rule = RuleName::new();
        assert_eq!(rule.name(), "RuleName");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = RuleName::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }
}
```

## 注意事项

1. **不要简化代码**: 保持原有逻辑完整性
2. **保持文档注释**: 保留所有文档注释和示例
3. **类型安全**: 使用模式匹配而不是类型转换
4. **错误处理**: 使用 `RewriteResult` 而不是 `Result<T, OptimizerError>`
5. **节点克隆**: 需要克隆节点时使用 `.clone()`
6. **输入获取**: 根据节点类型使用正确的 trait (SingleInputNode/BinaryInputNode)

## 验证步骤

1. 修改完成后运行 `analyze_cargo` 检查编译错误
2. 确保所有规则都实现了 `RewriteRule` trait
3. 根据需要实现特定的子 trait (`EliminationRule`, `MergeRule`, `PushDownRule`)
4. 运行测试验证规则行为正确
