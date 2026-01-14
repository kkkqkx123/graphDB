# Optimizer 模块分析报告

## 概述

本文档详细分析了 GraphDB 的 `src/query/optimizer` 模块与 nebula-graph 的实现对比，识别了简化的地方并提出了改进方案。

## 一、核心架构层面的简化

### 1.1 OptGroup 和 OptGroupNode 的简化

#### GraphDB 的简化实现

**缺少的关键功能：**

1. **bodies_ 字段**：用于支持 Select 和 Loop 等控制流节点
2. **groupNodesReferenced_**：缺少引用跟踪机制
3. **validate() 方法**：缺少计划验证功能
4. **ObjectPool**：缺少内存管理机制

**当前实现：**
```rust
pub struct OptGroup {
    pub id: usize,
    pub nodes: Vec<OptGroupNode>,
    pub logical: bool,
    pub explored_rules: Vec<String>,
    pub root_group: bool,
}

pub struct OptGroupNode {
    pub id: usize,
    pub plan_node: PlanNodeEnum,
    pub dependencies: Vec<usize>,
    pub cost: f64,
    pub properties: PlanNodeProperties,
    pub explored_rules: Vec<String>,
    pub group_id: usize,
}
```

#### nebula-graph 的完整实现

```cpp
class OptGroup final {
  std::list<OptGroupNode *> groupNodes_;
  std::vector<const OptRule *> exploredRules_;
  std::string outputVar_;
  bool isRootGroup_{false};
  std::unordered_set<const OptGroupNode *> groupNodesReferenced_;

  Status validate(const OptRule *rule) const;
  Status explore(const OptRule *rule);
  Status exploreUntilMaxRound(const OptRule *rule);
  double getCost() const;
};

class OptGroupNode final {
  graph::PlanNode *node_{nullptr};
  const OptGroup *group_{nullptr};
  std::vector<OptGroup *> dependencies_;
  std::vector<OptGroup *> bodies_;  // 用于控制流节点
  std::vector<const OptRule *> exploredRules_;

  Status explore(const OptRule *rule);
  double getCost() const;
  void release();
};
```

**改进建议：**

```rust
pub struct OptGroup {
    pub id: usize,
    pub nodes: Vec<OptGroupNode>,
    pub logical: bool,
    pub explored_rules: Vec<String>,
    pub root_group: bool,
    pub output_var: Option<String>,
    pub bodies: Vec<OptGroup>,  // 用于控制流节点
    pub group_nodes_referenced: HashSet<usize>,  // 引用跟踪
}

impl OptGroup {
    pub fn validate(&self, rule: &dyn OptRule) -> Result<(), OptimizerError> {
        // 验证数据流
        for node in &self.nodes {
            self.validate_data_flow(node)?;
        }
        Ok(())
    }

    pub fn explore(&mut self, rule: &dyn OptRule) -> Result<(), OptimizerError> {
        // 实现探索逻辑
        Ok(())
    }

    pub fn explore_until_max_round(&mut self, rule: &dyn OptRule) -> Result<(), OptimizerError> {
        // 实现最大轮次探索
        Ok(())
    }
}
```

### 1.2 OptRule 的简化

#### GraphDB 的简化实现

```rust
pub trait OptRule: std::fmt::Debug {
    fn name(&self) -> &str;
    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError>;

    fn match_pattern(
        &self,
        ctx: &mut OptContext,
        group_node: &OptGroupNode,
    ) -> Result<Option<MatchedResult>, OptimizerError> {
        let pattern = self.pattern();
        self.match_pattern_with_result(ctx, group_node, &pattern)
    }

    fn pattern(&self) -> Pattern;
}
```

#### nebula-graph 的完整实现

```cpp
class OptRule {
 public:
  virtual const Pattern &pattern() const = 0;
  virtual bool match(OptContext *ctx, const MatchedResult &matched) const;
  virtual StatusOr<TransformResult> transform(OptContext *ctx,
                                            const MatchedResult &matched) const = 0;
  virtual std::string toString() const = 0;

 protected:
  bool checkDataflowDeps(OptContext *ctx,
                         const MatchedResult &matched,
                         const std::string &var,
                         bool isRoot) const;
};

struct TransformResult {
  bool eraseCurr{false};
  bool eraseAll{false};
  std::vector<OptGroupNode *> newGroupNodes;

  bool checkDataFlow(const std::vector<OptGroup *> &boundary);
  static bool checkDataFlow(const OptGroupNode *groupNode,
                           const std::vector<OptGroup *> &boundary);
};
```

**改进建议：**

```rust
pub trait OptRule: std::fmt::Debug {
    fn name(&self) -> &str;
    fn pattern(&self) -> Pattern;

    fn match(&self, ctx: &mut OptContext, matched: &MatchedResult) -> bool {
        true
    }

    fn transform(
        &self,
        ctx: &mut OptContext,
        matched: &MatchedResult,
    ) -> Result<TransformResult, OptimizerError>;

    fn check_dataflow_deps(
        &self,
        ctx: &OptContext,
        matched: &MatchedResult,
        var: &str,
        is_root: bool,
    ) -> bool {
        true
    }
}

pub struct TransformResult {
    pub erase_curr: bool,
    pub erase_all: bool,
    pub new_group_nodes: Vec<OptGroupNode>,
}

impl TransformResult {
    pub fn check_data_flow(&self, boundary: &[&OptGroup]) -> bool {
        // 实现数据流检查
        true
    }

    pub fn no_transform() -> Self {
        Self {
            erase_curr: false,
            erase_all: false,
            new_group_nodes: Vec::new(),
        }
    }
}
```

## 二、Optimizer 主流程的简化

### 2.1 缺少关键的后处理步骤

#### GraphDB 的简化实现

```rust
fn post_process(
    &self,
    ctx: &mut OptContext,
    root_group: &mut OptGroup,
) -> Result<(), OptimizerError> {
    // 只做了基本的数据流验证
    for node in &root_group.nodes {
        for &dep_id in &node.dependencies {
            if !root_group.nodes.iter().any(|n| n.id == dep_id) {
                return Err(OptimizerError::OptimizationFailed(format!(
                    "Invalid dependency: node {} depends on non-existent node {}",
                    node.id, dep_id
                )));
            }
        }

        let boundary = vec![&*root_group];
        if !ctx.validate_data_flow(node, &boundary) {
            return Err(OptimizerError::OptimizationFailed(format!(
                "Data flow validation failed for node {}",
                node.id
            )));
        }
    }

    Ok(())
}
```

#### nebula-graph 的完整实现

```cpp
Status Optimizer::postprocess(PlanNode *root, QueryContext *qctx, GraphSpaceID spaceID) {
  std::unordered_set<const PlanNode *> visitedPlanNode;

  // 1. 重写 Argument 输入变量
  NG_RETURN_IF_ERROR(rewriteArgumentInputVar(root, visitedPlanNode));

  // 2. 属性修剪
  if (FLAGS_enable_optimizer_property_pruner_rule) {
    graph::PropertyTracker propsUsed;
    graph::PrunePropertiesVisitor visitor(propsUsed, qctx, spaceID);
    root->accept(&visitor);
  }

  return Status::OK();
}
```

**改进建议：**

```rust
impl Optimizer {
    fn post_process(
        &self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        // 1. 重写 Argument 输入变量
        self.rewrite_argument_input_var(root_group)?;

        // 2. 属性修剪
        if ctx.query_context.enable_property_pruner {
            self.prune_properties(root_group)?;
        }

        // 3. 验证计划深度
        self.check_plan_depth(root_group)?;

        // 4. 验证数据流
        self.validate_data_flow(root_group)?;

        Ok(())
    }

    fn rewrite_argument_input_var(&self, root_group: &mut OptGroup) -> Result<(), OptimizerError> {
        // 实现 Argument 输入变量重写逻辑
        Ok(())
    }

    fn prune_properties(&self, root_group: &mut OptGroup) -> Result<(), OptimizerError> {
        // 使用 PropertyTracker 和 PrunePropertiesVisitor 进行属性修剪
        Ok(())
    }

    fn check_plan_depth(&self, root_group: &OptGroup) -> Result<(), OptimizerError> {
        const MAX_PLAN_DEPTH: usize = 512;
        let depth = self.calculate_plan_depth(root_group)?;
        if depth > MAX_PLAN_DEPTH {
            return Err(OptimizerError::PlanTooDeep(depth));
        }
        Ok(())
    }
}
```

### 2.2 缺少计划深度检查

**GraphDB：** 完全缺少计划深度检查机制

**nebula-graph：**
```cpp
DEFINE_uint64(max_plan_depth, 512, "The max depth of plan tree");

Status Optimizer::checkPlanDepth(const PlanNode *root) const {
  std::queue<const PlanNode *> queue;
  queue.push(root);
  size_t depth = 0;

  while (!queue.empty()) {
    size_t size = queue.size();
    for (size_t i = 0; i < size; ++i) {
      auto node = queue.front();
      queue.pop();
      for (size_t j = 0; j < node->numDeps(); ++j) {
        queue.push(node->dep(j));
      }
    }
    depth++;
    if (depth > FLAGS_max_plan_depth) {
      return Status::Error("Plan depth exceeds limit");
    }
  }

  return Status::OK();
}
```

## 三、具体优化规则的简化

### 3.1 FilterPushDownRule 的简化

#### GraphDB 的简化实现

```rust
impl OptRule for FilterPushDownRule {
    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if !node.plan_node.is_filter() {
            return Ok(None);
        }

        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child_node = &matched.dependencies[0];

                if let Some(filter_plan_node) = node.plan_node.as_filter() {
                    let filter_condition = filter_plan_node.condition();

                    match child_node.plan_node().name() {
                        "ScanVertices" => {
                            let split_result = can_push_down_to_scan(filter_condition);

                            if let Some(pushable_condition) = split_result.pushable_condition {
                                // 简化的实现，只做基本的条件合并
                                // 缺少表达式分割和重写逻辑
                            }
                        }
                        _ => Ok(None)
                    }
                }
            }
        }
        Ok(None)
    }
}
```

#### nebula-graph 的完整实现

```cpp
StatusOr<OptRule::TransformResult> PushFilterDownTraverseRule::transform(
    OptContext* octx, const MatchedResult& matched) const {
  auto* filterGNode = matched.node;
  auto* filter = static_cast<graph::Filter*>(filterGNode->node());
  auto* condition = filter->condition();

  auto* tvGNode = matched.dependencies[0].node;
  auto* tvNode = static_cast<graph::Traverse*>(tvGNode->node());
  auto& edgeAlias = tvNode->edgeAlias();

  auto qctx = octx->qctx();
  auto pool = qctx->objPool();

  // 使用表达式访问者模式分析表达式
  auto picker = [&edgeAlias](const Expression* expr) -> bool {
    bool shouldNotPick = false;
    auto finder = [&shouldNotPick, &edgeAlias](const Expression* e) -> bool {
      // 检查是否为单步边属性表达式
      if (graph::ExpressionUtils::isOneStepEdgeProp(edgeAlias, e)) return true;

      // 检查是否包含 InputProperty 或 VarProperty
      if (e->kind() == Expression::Kind::kInputProperty ||
          e->kind() == Expression::Kind::kVarProperty) {
        shouldNotPick = true;
        return false;
      }

      return false;
    };

    graph::FindVisitor visitor(finder, true, true);
    const_cast<Expression*>(expr)->accept(&visitor);
    if (shouldNotPick) return false;
    if (!visitor.results().empty()) {
      return true;
    }
    return false;
  };

  // 分割过滤条件
  Expression* filterPicked = nullptr;
  Expression* filterUnpicked = nullptr;
  graph::ExpressionUtils::splitFilter(condition, picker, &filterPicked, &filterUnpicked);

  if (!filterPicked) {
    return TransformResult::noTransform();
  }

  // 重写边属性过滤条件
  auto* newFilterPicked =
      graph::ExpressionUtils::rewriteEdgePropertyFilter(pool, edgeAlias, filterPicked->clone());
  auto* eFilter = tvNode->eFilter();
  Expression* newEFilter = eFilter
                               ? LogicalExpression::makeAnd(pool, newFilterPicked, eFilter->clone())
                               : newFilterPicked;

  // 创建新的 Traverse 节点
  auto* newTvNode = static_cast<graph::Traverse*>(tvNode->clone());
  newTvNode->setEdgeFilter(newEFilter);

  // 构建转换结果
  TransformResult result;
  result.eraseAll = true;
  if (filterUnpicked) {
    auto* newFilterNode = graph::Filter::make(qctx, newTvNode, filterUnpicked);
    auto newFilterGNode = OptGroupNode::create(octx, newFilterNode, filterGroup);
    auto newTvGroup = OptGroup::create(octx);
    auto newTvGNode = newTvGroup->makeGroupNode(newTvNode);
    newTvGNode->setDeps(tvGNode->dependencies());
    newFilterGNode->setDeps({newTvGroup});
    result.newGroupNodes.emplace_back(newFilterGNode);
  } else {
    auto newTvGNode = OptGroupNode::create(octx, newTvNode, filterGroup);
    newTvGNode->setDeps(tvGNode->dependencies());
    result.newGroupNodes.emplace_back(newTvGNode);
  }

  return result;
}
```

**改进建议：**

```rust
impl FilterPushDownRule {
    fn split_filter_condition(
        &self,
        condition: &Expression,
        edge_alias: &str,
    ) -> (Option<Expression>, Option<Expression>) {
        // 使用 ExpressionVisitor 分割过滤条件
        let picker = |expr: &Expression| -> bool {
            // 检查是否为单步边属性表达式
            if ExpressionUtils::is_one_step_edge_prop(edge_alias, expr) {
                return true;
            }

            // 检查是否包含 InputProperty 或 VarProperty
            if matches!(expr.kind(), ExpressionKind::InputProperty | ExpressionKind::VarProperty) {
                return false;
            }

            false
        };

        ExpressionUtils::split_filter(condition, picker)
    }

    fn transform_filter_down_traverse(
        &self,
        ctx: &mut OptContext,
        matched: &MatchedResult,
    ) -> Result<TransformResult, OptimizerError> {
        let filter_node = matched.node;
        let traverse_node = &matched.dependencies[0];

        if let Some(filter_plan_node) = filter_node.plan_node.as_filter() {
            if let Some(traverse_plan_node) = traverse_node.plan_node.as_traverse() {
                let edge_alias = traverse_plan_node.edge_alias();

                // 分割过滤条件
                let (filter_picked, filter_unpicked) =
                    self.split_filter_condition(filter_plan_node.condition(), edge_alias);

                if let Some(picked) = filter_picked {
                    // 重写边属性过滤条件
                    let new_filter_picked = ExpressionUtils::rewrite_edge_property_filter(
                        ctx.query_context.obj_pool(),
                        edge_alias,
                        picked,
                    );

                    // 合并现有的边过滤器
                    let new_e_filter = if let Some(e_filter) = traverse_plan_node.e_filter() {
                        Expression::and(new_filter_picked, e_filter.clone())
                    } else {
                        new_filter_picked
                    };

                    // 创建新的 Traverse 节点
                    let mut new_traverse_node = traverse_plan_node.clone();
                    new_traverse_node.set_edge_filter(new_e_filter);

                    // 构建转换结果
                    let mut result = TransformResult::no_transform();
                    result.erase_all = true;

                    if let Some(unpicked) = filter_unpicked {
                        // 创建新的 Filter 节点
                        let new_filter_node = FilterNode::new(new_traverse_node, unpicked);
                        let new_filter_opt_node = OptGroupNode::new(
                            filter_node.id,
                            PlanNodeEnum::Filter(new_filter_node),
                        );
                        new_filter_opt_node.dependencies = traverse_node.dependencies.clone();
                        result.new_group_nodes.push(new_filter_opt_node);
                    } else {
                        // 直接使用新的 Traverse 节点
                        let new_traverse_opt_node = OptGroupNode::new(
                            filter_node.id,
                            PlanNodeEnum::Traverse(new_traverse_node),
                        );
                        new_traverse_opt_node.dependencies = traverse_node.dependencies.clone();
                        result.new_group_nodes.push(new_traverse_opt_node);
                    }

                    return Ok(result);
                }
            }
        }

        Ok(TransformResult::no_transform())
    }
}
```

### 3.2 EliminateFilterRule 的简化

#### GraphDB 的简化实现

```rust
impl EliminationRule for EliminateFilterRule {
    fn can_eliminate(&self, _ctx: &OptContext, node: &OptGroupNode) -> bool {
        if !node.plan_node.is_filter() {
            return false;
        }

        if let Some(filter_plan_node) = node.plan_node.as_filter() {
            let condition = filter_plan_node.condition();
            is_expression_tautology(condition)
        } else {
            false
        }
    }
}
```

**只检查简单的永真式：**
```rust
pub fn is_expression_tautology(expr: &Expression) -> bool {
    match expr {
        Expression::Literal(Value::Bool(true)) => true,
        Expression::Literal(Value::Bool(false)) => false,
        Expression::Binary { left, op, right } => {
            match (left.as_ref(), op, right.as_ref()) {
                (Expression::Literal(Value::Int(1)), BinaryOperator::Equal, Expression::Literal(Value::Int(1))) => true,
                (Expression::Literal(Value::Int(0)), BinaryOperator::Equal, Expression::Literal(Value::Int(0))) => true,
                (Expression::Variable(a), BinaryOperator::Equal, Expression::Variable(b)) if a == b => true,
                _ => false,
            }
        }
        _ => false,
    }
}
```

#### nebula-graph 的完整实现

```cpp
bool EliminateFilterRule::match(OptContext* octx, const MatchedResult& matched) const {
  if (!OptRule::match(octx, matched)) {
    return false;
  }

  const auto* filterNode = static_cast<const Filter*>(matched.node->node());
  const auto* expr = filterNode->condition();

  // 检查是否为常量表达式
  if (expr->kind() != Expression::Kind::kConstant) {
    return false;
  }

  const auto* constant = static_cast<const ConstantExpression*>(expr);

  // 检查是否为 false 或 null
  auto ret = (constant->value().isImplicitBool() && constant->value().getBool() == false) ||
             constant->value().isNull();
  return ret;
}

StatusOr<OptRule::TransformResult> EliminateFilterRule::transform(
    OptContext* octx, const MatchedResult& matched) const {
  auto filterGroupNode = matched.node;
  auto filter = static_cast<const Filter*>(filterGroupNode->node());

  // 创建 Start 节点
  auto newStart = StartNode::make(octx->qctx());
  auto newStartGroup = OptGroup::create(octx);
  newStartGroup->makeGroupNode(newStart);

  // 创建 Value 节点
  auto newValue = ValueNode::make(octx->qctx(), newStart, DataSet(filter->colNames()));
  newValue->setOutputVar(filter->outputVar());
  auto newValueGroupNode = OptGroupNode::create(octx, newValue, filterGroupNode->group());
  newValueGroupNode->dependsOn(newStartGroup);

  TransformResult result;
  result.eraseAll = true;
  result.newGroupNodes.emplace_back(newValueGroupNode);
  return result;
}
```

**改进建议：**

```rust
impl EliminateFilterRule {
    fn can_eliminate(&self, ctx: &OptContext, node: &OptGroupNode) -> bool {
        if !node.plan_node.is_filter() {
            return false;
        }

        if let Some(filter_plan_node) = node.plan_node.as_filter() {
            let condition = filter_plan_node.condition();

            // 检查是否为常量表达式
            if !matches!(condition.kind(), ExpressionKind::Constant) {
                return false;
            }

            // 检查是否为 false 或 null
            if let Expression::Literal(value) = condition {
                match value {
                    Value::Bool(false) => return true,
                    Value::Null => return true,
                    _ => return false,
                }
            }
        }

        false
    }

    fn transform(
        &self,
        ctx: &mut OptContext,
        matched: &MatchedResult,
    ) -> Result<TransformResult, OptimizerError> {
        let filter_group_node = matched.node;
        let filter = filter_group_node.plan_node.as_filter().expect("Should be filter node");

        // 创建 Start 节点
        let new_start = StartNode::new(ctx.query_context.clone());
        let new_start_group = OptGroup::new(ctx.next_group_id(), false);
        new_start_group.nodes.push(OptGroupNode::new(
            ctx.next_node_id(),
            PlanNodeEnum::Start(new_start),
        ));

        // 创建 Value 节点（空数据集）
        let data_set = DataSet::new_with_columns(filter.col_names().clone());
        let new_value = ValueNode::new(new_start_group.nodes[0].plan_node.clone(), data_set);
        new_value.set_output_var(filter.output_var().clone());

        let mut new_value_opt_node = OptGroupNode::new(
            filter_group_node.id,
            PlanNodeEnum::Value(new_value),
        );
        new_value_opt_node.dependencies = vec![new_start_group.nodes[0].id];

        let mut result = TransformResult::no_transform();
        result.erase_all = true;
        result.new_group_nodes.push(new_value_opt_node);

        Ok(result)
    }
}
```

### 3.3 JoinOptimizationRule 的简化

#### GraphDB 的简化实现

```rust
impl JoinOptimizationRule {
    fn should_optimize_join(&self, left_node: &OptGroupNode, right_node: &OptGroupNode) -> bool {
        // 简单的启发式：如果任一侧是索引扫描或者获取特定顶点/边的操作
        matches!(
            left_node.plan_node.type_name(),
            "IndexScan" | "ScanVertices" | "ScanEdges"
        ) || matches!(
            right_node.plan_node.type_name(),
            "IndexScan" | "ScanVertices" | "ScanEdges"
        )
    }
}
```

**缺少的功能：**
- 基于成本的连接算法选择
- 数据分布和选择性估计
- 多种连接算法的转换

#### nebula-graph 的完整实现

nebula-graph 的连接优化包括：
1. 基于成本的连接算法选择
2. 考虑数据分布和选择性
3. 支持多种连接算法的转换（HashJoin、NestedLoopJoin、SortMergeJoin）

**改进建议：**

```rust
impl JoinOptimizationRule {
    fn estimate_join_cost(&self, left: &OptGroupNode, right: &OptGroupNode) -> f64 {
        let left_cost = left.cost;
        let right_cost = right.cost;
        let left_rows = left.properties.estimated_rows.unwrap_or(1000);
        let right_rows = right.properties.estimated_rows.unwrap_or(1000);

        // 估计连接成本
        // HashJoin: O(left_rows + right_rows)
        // NestedLoopJoin: O(left_rows * right_rows)
        // SortMergeJoin: O(left_rows * log(left_rows) + right_rows * log(right_rows))

        let hash_join_cost = left_cost + right_cost + (left_rows + right_rows) as f64 * 0.1;
        let nested_loop_cost = left_cost + right_cost + (left_rows * right_rows) as f64 * 0.01;
        let sort_merge_cost = left_cost + right_cost +
            (left_rows as f64 * (left_rows as f64).log2() +
             right_rows as f64 * (right_rows as f64).log2()) * 0.1;

        hash_join_cost.min(nested_loop_cost).min(sort_merge_cost)
    }

    fn select_best_join_algorithm(
        &self,
        left: &OptGroupNode,
        right: &OptGroupNode,
    ) -> JoinAlgorithm {
        let left_rows = left.properties.estimated_rows.unwrap_or(1000);
        let right_rows = right.properties.estimated_rows.unwrap_or(1000);

        // 如果一侧很小，使用 HashJoin
        if left_rows < 100 || right_rows < 100 {
            return JoinAlgorithm::HashJoin;
        }

        // 如果两侧都很大，考虑 SortMergeJoin
        if left_rows > 10000 && right_rows > 10000 {
            return JoinAlgorithm::SortMergeJoin;
        }

        // 默认使用 HashJoin
        JoinAlgorithm::HashJoin
    }

    fn transform(
        &self,
        ctx: &mut OptContext,
        matched: &MatchedResult,
    ) -> Result<TransformResult, OptimizerError> {
        let join_node = matched.node;
        let left_node = &matched.dependencies[0];
        let right_node = &matched.dependencies[1];

        // 选择最佳连接算法
        let best_algorithm = self.select_best_join_algorithm(left_node, right_node);

        // 估计成本
        let estimated_cost = self.estimate_join_cost(left_node, right_node);

        // 转换为最佳算法
        match best_algorithm {
            JoinAlgorithm::HashJoin => self.transform_to_hash_join(ctx, matched),
            JoinAlgorithm::NestedLoopJoin => self.transform_to_nested_loop_join(ctx, matched),
            JoinAlgorithm::SortMergeJoin => self.transform_to_sort_merge_join(ctx, matched),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum JoinAlgorithm {
    HashJoin,
    NestedLoopJoin,
    SortMergeJoin,
}
```

### 3.4 IndexOptimizationRule 的简化

#### GraphDB 的简化实现

```rust
impl OptRule for OptimizeEdgeIndexScanByFilterRule {
    fn apply(
        &self,
        _ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        if !node.plan_node.is_index_scan() {
            return Ok(None);
        }

        // 缺少基于选择性的索引扫描优化
        // 缺少索引选择性的估计
        // 缺少索引扫描和全扫描的成本比较

        Ok(Some(node.clone()))
    }
}
```

**缺少的功能：**
- 基于选择性的索引扫描优化
- 索引选择性的估计
- 索引扫描和全扫描的成本比较
- 支持全文索引优化

#### nebula-graph 的完整实现

nebula-graph 的索引优化包括：
1. 基于选择性的索引扫描优化
2. 索引选择性的估计
3. 索引扫描和全扫描的成本比较
4. 支持全文索引优化

**改进建议：**

```rust
impl OptimizeEdgeIndexScanByFilterRule {
    fn estimate_index_selectivity(
        &self,
        index_scan: &IndexScanNode,
        filter: &Expression,
    ) -> f64 {
        // 估计索引选择性
        // 选择性 = (满足条件的行数) / (总行数)

        match filter {
            Expression::Binary { left, op, right } => {
                match op {
                    BinaryOperator::Equal => {
                        // 等值条件的选择性通常较低
                        0.01
                    }
                    BinaryOperator::Less | BinaryOperator::LessEqual |
                    BinaryOperator::Greater | BinaryOperator::GreaterEqual => {
                        // 范围条件的选择性
                        0.25
                    }
                    _ => 0.5,
                }
            }
            _ => 0.5,
        }
    }

    fn compare_index_vs_full_scan(
        &self,
        index_scan: &IndexScanNode,
        filter: &Expression,
    ) -> ScanStrategy {
        let selectivity = self.estimate_index_selectivity(index_scan, filter);
        let total_rows = index_scan.estimated_total_rows();

        // 估计索引扫描成本
        let index_scan_cost = total_rows as f64 * selectivity * 0.1 + 100.0;

        // 估计全扫描成本
        let full_scan_cost = total_rows as f64 * 0.05;

        if index_scan_cost < full_scan_cost {
            ScanStrategy::IndexScan
        } else {
            ScanStrategy::FullScan
        }
    }

    fn transform(
        &self,
        ctx: &mut OptContext,
        matched: &MatchedResult,
    ) -> Result<TransformResult, OptimizerError> {
        let index_scan_node = matched.node;
        let filter_node = &matched.dependencies[0];

        if let Some(index_scan) = index_scan_node.plan_node.as_index_scan() {
            if let Some(filter) = filter_node.plan_node.as_filter() {
                // 比较索引扫描和全扫描
                let strategy = self.compare_index_vs_full_scan(index_scan, filter.condition());

                match strategy {
                    ScanStrategy::IndexScan => {
                        // 优化索引扫描
                        self.optimize_index_scan(ctx, index_scan, filter.condition())
                    }
                    ScanStrategy::FullScan => {
                        // 转换为全扫描
                        self.transform_to_full_scan(ctx, index_scan)
                    }
                }
            }
        } else {
            Ok(TransformResult::no_transform())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ScanStrategy {
    IndexScan,
    FullScan,
}
```

## 四、表达式处理的简化

### 4.1 缺少 ExpressionVisitor 模式

**GraphDB：** 完全缺少 ExpressionVisitor 模式

**nebula-graph 的完整实现：**

```cpp
// 表达式访问者基类
class ExpressionVisitor {
 public:
  virtual ~ExpressionVisitor() = default;
  virtual bool visit(const Expression* expr) = 0;
  virtual bool visit(const ConstantExpression* expr) {
    return visit(static_cast<const Expression*>(expr));
  }
  virtual bool visit(const UnaryExpression* expr) {
    return visit(static_cast<const Expression*>(expr));
  }
  virtual bool visit(const BinaryExpression* expr) {
    return visit(static_cast<const Expression*>(expr));
  }
  // ... 其他表达式类型的 visit 方法
};

// 查找访问者
class FindVisitor : public ExpressionVisitor {
 public:
  explicit FindVisitor(std::function<bool(const Expression*)> matcher,
                      bool matchFirst = true,
                      bool needCheck = false)
      : matcher_(std::move(matcher)), matchFirst_(matchFirst), needCheck_(needCheck) {}

  bool visit(const Expression* expr) override {
    if (matcher_(expr)) {
      results_.push_back(expr);
      if (matchFirst_) {
        return false;
      }
    }
    return true;
  }

  const std::vector<const Expression*>& results() const {
    return results_;
  }

 private:
  std::function<bool(const Expression*)> matcher_;
  bool matchFirst_;
  bool needCheck_;
  std::vector<const Expression*> results_;
};
```

**改进建议：**

```rust
// 表达式访问者 trait
pub trait ExpressionVisitor {
    fn visit(&mut self, expr: &Expression) -> bool;

    fn visit_constant(&mut self, expr: &ConstantExpression) -> bool {
        self.visit(&Expression::Constant(expr.clone()))
    }

    fn visit_variable(&mut self, expr: &VariableExpression) -> bool {
        self.visit(&Expression::Variable(expr.clone()))
    }

    fn visit_unary(&mut self, expr: &UnaryExpression) -> bool {
        self.visit(&Expression::Unary(expr.clone()))
    }

    fn visit_binary(&mut self, expr: &BinaryExpression) -> bool {
        self.visit(&Expression::Binary(expr.clone()))
    }

    fn visit_property(&mut self, expr: &PropertyExpression) -> bool {
        self.visit(&Expression::Property(expr.clone()))
    }
}

// 查找访问者
pub struct FindVisitor<F>
where
    F: Fn(&Expression) -> bool,
{
    matcher: F,
    match_first: bool,
    results: Vec<Expression>,
}

impl<F> FindVisitor<F>
where
    F: Fn(&Expression) -> bool,
{
    pub fn new(matcher: F, match_first: bool) -> Self {
        Self {
            matcher,
            match_first,
            results: Vec::new(),
        }
    }

    pub fn results(&self) -> &[Expression] {
        &self.results
    }
}

impl<F> ExpressionVisitor for FindVisitor<F>
where
    F: Fn(&Expression) -> bool,
{
    fn visit(&mut self, expr: &Expression) -> bool {
        if (self.matcher)(expr) {
            self.results.push(expr.clone());
            if self.match_first {
                return false;
            }
        }
        true
    }
}
```

### 4.2 缺少表达式工具类

**GraphDB：** 缺少 ExpressionUtils 工具类

**nebula-graph 的完整实现：**

```cpp
class ExpressionUtils {
 public:
  // 检查是否为单步边属性表达式
  static bool isOneStepEdgeProp(const std::string& edgeAlias, const Expression* expr);

  // 分割过滤条件
  static void splitFilter(const Expression* filter,
                        std::function<bool(const Expression*)> picker,
                        Expression** picked,
                        Expression** unpicked);

  // 重写边属性过滤条件
  static Expression* rewriteEdgePropertyFilter(ObjectPool* pool,
                                             const std::string& edgeAlias,
                                             Expression* filter);

  // 检查表达式是否包含特定类型的表达式
  static bool hasAny(const Expression* expr,
                    const std::unordered_set<Expression::Kind>& kinds);

  // 收集表达式中的所有变量
  static std::vector<const Expression*> collectAll(const Expression* expr,
                                                const std::unordered_set<Expression::Kind>& kinds);
};
```

**改进建议：**

```rust
pub struct ExpressionUtils;

impl ExpressionUtils {
    /// 检查是否为单步边属性表达式
    pub fn is_one_step_edge_prop(edge_alias: &str, expr: &Expression) -> bool {
        match expr {
            Expression::Property { label, .. } => {
                label.starts_with(&format!("{}.", edge_alias))
            }
            _ => false,
        }
    }

    /// 分割过滤条件
    pub fn split_filter(
        filter: &Expression,
        picker: impl Fn(&Expression) -> bool,
    ) -> (Option<Expression>, Option<Expression>) {
        let mut picked_exprs = Vec::new();
        let mut unpicked_exprs = Vec::new();

        self.split_filter_recursive(filter, &picker, &mut picked_exprs, &mut unpicked_exprs);

        let picked = if picked_exprs.is_empty() {
            None
        } else {
            Some(Expression::and_all(picked_exprs))
        };

        let unpicked = if unpicked_exprs.is_empty() {
            None
        } else {
            Some(Expression::and_all(unpicked_exprs))
        };

        (picked, unpicked)
    }

    fn split_filter_recursive(
        &self,
        expr: &Expression,
        picker: &impl Fn(&Expression) -> bool,
        picked: &mut Vec<Expression>,
        unpicked: &mut Vec<Expression>,
    ) {
        match expr {
            Expression::Binary { left, op: BinaryOperator::And, right } => {
                self.split_filter_recursive(left, picker, picked, unpicked);
                self.split_filter_recursive(right, picker, picked, unpicked);
            }
            _ => {
                if picker(expr) {
                    picked.push(expr.clone());
                } else {
                    unpicked.push(expr.clone());
                }
            }
        }
    }

    /// 重写边属性过滤条件
    pub fn rewrite_edge_property_filter(
        pool: &mut ObjectPool,
        edge_alias: &str,
        filter: Expression,
    ) -> Expression {
        match filter {
            Expression::Property { label, prop_name } => {
                if label.starts_with(&format!("{}.", edge_alias)) {
                    // 重写为边属性表达式
                    Expression::EdgeProperty {
                        edge_alias: edge_alias.to_string(),
                        prop_name,
                    }
                } else {
                    filter
                }
            }
            Expression::Binary { left, op, right } => {
                Expression::Binary {
                    left: Box::new(self.rewrite_edge_property_filter(pool, edge_alias, *left)),
                    op,
                    right: Box::new(self.rewrite_edge_property_filter(pool, edge_alias, *right)),
                }
            }
            _ => filter,
        }
    }

    /// 检查表达式是否包含特定类型的表达式
    pub fn has_any(expr: &Expression, kinds: &[ExpressionKind]) -> bool {
        match expr {
            Expression::Binary { left, right, .. } => {
                kinds.contains(&expr.kind()) ||
                self.has_any(left, kinds) ||
                self.has_any(right, kinds)
            }
            Expression::Unary { operand, .. } => {
                kinds.contains(&expr.kind()) ||
                self.has_any(operand, kinds)
            }
            _ => kinds.contains(&expr.kind()),
        }
    }

    /// 收集表达式中的所有变量
    pub fn collect_all(expr: &Expression, kinds: &[ExpressionKind]) -> Vec<Expression> {
        let mut results = Vec::new();
        self.collect_all_recursive(expr, kinds, &mut results);
        results
    }

    fn collect_all_recursive(
        &self,
        expr: &Expression,
        kinds: &[ExpressionKind],
        results: &mut Vec<Expression>,
    ) {
        if kinds.contains(&expr.kind()) {
            results.push(expr.clone());
        }

        match expr {
            Expression::Binary { left, right, .. } => {
                self.collect_all_recursive(left, kinds, results);
                self.collect_all_recursive(right, kinds, results);
            }
            Expression::Unary { operand, .. } => {
                self.collect_all_recursive(operand, kinds, results);
            }
            _ => {}
        }
    }
}
```

### 4.3 缺少属性修剪机制

**GraphDB：** 完全缺少属性修剪机制

**nebula-graph 的完整实现：**

```cpp
// 属性跟踪器
class PropertyTracker {
 public:
  void trackProperty(const std::string& var, const std::string& prop);
  bool isPropertyUsed(const std::string& var, const std::string& prop) const;

 private:
  std::unordered_map<std::string, std::unordered_set<std::string>> usedProperties_;
};

// 属性修剪访问者
class PrunePropertiesVisitor : public ExpressionVisitor {
 public:
  PrunePropertiesVisitor(PropertyTracker& tracker, QueryContext* qctx, GraphSpaceID spaceID)
      : tracker_(tracker), qctx_(qctx), spaceID_(spaceID) {}

  bool visit(const PropertyExpression* expr) override {
    auto* prop = static_cast<const PropertyExpression*>(expr);
    tracker_.trackProperty(prop->sym(), prop->prop());
    return true;
  }

  Status status() const { return status_; }

 private:
  PropertyTracker& tracker_;
  QueryContext* qctx_;
  GraphSpaceID spaceID_;
  Status status_;
};
```

**改进建议：**

```rust
// 属性跟踪器
pub struct PropertyTracker {
    used_properties: HashMap<String, HashSet<String>>,
}

impl PropertyTracker {
    pub fn new() -> Self {
        Self {
            used_properties: HashMap::new(),
        }
    }

    pub fn track_property(&mut self, var: &str, prop: &str) {
        self.used_properties
            .entry(var.to_string())
            .or_insert_with(HashSet::new)
            .insert(prop.to_string());
    }

    pub fn is_property_used(&self, var: &str, prop: &str) -> bool {
        if let Some(props) = self.used_properties.get(var) {
            props.contains(prop)
        } else {
            false
        }
    }

    pub fn get_used_properties(&self, var: &str) -> Option<&HashSet<String>> {
        self.used_properties.get(var)
    }
}

// 属性修剪访问者
pub struct PrunePropertiesVisitor {
    tracker: PropertyTracker,
    query_context: QueryContext,
    space_id: GraphSpaceID,
    status: Result<(), OptimizerError>,
}

impl PrunePropertiesVisitor {
    pub fn new(
        tracker: PropertyTracker,
        query_context: QueryContext,
        space_id: GraphSpaceID,
    ) -> Self {
        Self {
            tracker,
            query_context,
            space_id,
            status: Ok(()),
        }
    }

    pub fn status(&self) -> Result<(), OptimizerError> {
        self.status.clone()
    }
}

impl ExpressionVisitor for PrunePropertiesVisitor {
    fn visit(&mut self, expr: &Expression) -> bool {
        match expr {
            Expression::Property { label, prop_name } => {
                self.tracker.track_property(label, prop_name);
                true
            }
            _ => true,
        }
    }
}
```

## 五、改进计划

### 5.1 高优先级改进

1. **添加 ExpressionVisitor trait 和 ExpressionUtils**
   - 创建完整的表达式访问者模式
   - 实现表达式工具类
   - 支持表达式分割和重写

2. **完善 OptGroup 和 OptGroupNode**
   - 添加 bodies 字段支持控制流节点
   - 添加引用跟踪机制
   - 实现计划验证方法

3. **添加后处理步骤**
   - 实现 Argument 输入变量重写
   - 实现属性修剪机制
   - 添加计划深度检查

### 5.2 中优先级改进

4. **完善具体优化规则**
   - 改进 FilterPushDownRule 的表达式处理
   - 改进 EliminateFilterRule 的常量表达式分析
   - 改进 JoinOptimizationRule 的基于成本优化
   - 改进 IndexOptimizationRule 的选择性估计

5. **添加计划验证机制**
   - 实现数据流验证
   - 实现属性使用验证
   - 实现计划深度验证

### 5.3 低优先级改进

6. **添加成本模型**
   - 实现成本估计函数
   - 实现基于成本的优化决策
   - 支持多种执行算法的成本比较

7. **添加统计信息收集**
   - 实现统计信息收集机制
   - 支持基于统计信息的优化
   - 支持自适应优化

## 六、总结

通过对比分析，我们发现 GraphDB 的 optimizer 模块在以下方面存在显著简化：

1. **核心架构**：缺少控制流支持、引用跟踪、计划验证等关键功能
2. **优化流程**：缺少后处理步骤、计划深度检查等
3. **优化规则**：表达式处理过于简单，缺少基于成本的优化
4. **表达式处理**：缺少 ExpressionVisitor 模式和表达式工具类

这些简化导致优化器的功能完整性和优化效果远不如 nebula-graph。通过实施上述改进计划，可以显著提升优化器的功能完整性和优化效果。
