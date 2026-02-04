//! 谓词下推优化规则
//! 这些规则负责将过滤条件下推到计划树的底层，以减少数据处理量

use super::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result as OptResult};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::planner::plan::PlanNodeEnum;
use std::rc::Rc;
use std::cell::RefCell;

/// 将过滤条件下推到扫描操作的规则
/// 
/// 该规则识别 Filter -> ScanVertices/ScanEdges 模式，
/// 并将过滤条件集成到扫描操作中，减少后续处理的数据量。
#[derive(Debug)]
pub struct PushFilterDownScanVerticesRule;

impl OptRule for PushFilterDownScanVerticesRule {
    fn name(&self) -> &str {
        "PushFilterDownScanVerticesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有且仅有一个子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为扫描操作
        if !matches!(child_ref.plan_node.name(), "ScanVertices" | "ScanEdges" | "IndexScan") {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition(),
            None => return Ok(None),
        };

        // 简化实现：直接创建新的扫描节点，集成过滤条件
        // 在实际实现中，这里应该创建支持过滤条件的扫描节点变体
        let mut new_child_node = child_ref.clone();
        
        // 创建新的过滤节点，使用原始扫描节点作为子节点
        let mut new_filter_node = node_ref.clone();
        new_filter_node.dependencies = vec![child_id];

        // 创建转换结果
        let mut result = TransformResult::new();
        result.add_new_group_node(Rc::new(RefCell::new(new_filter_node)));
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> ScanVertices/ScanEdges/IndexScan 模式
        PatternBuilder::filter_with("ScanVertices")
    }
}

impl BaseOptRule for PushFilterDownScanVerticesRule {}

/// 将过滤条件下推到遍历操作的规则
/// 
/// 该规则识别 Filter -> Traverse 模式，
/// 并将过滤条件集成到遍历操作中。
#[derive(Debug)]
pub struct PushFilterDownTraverseRule;

impl OptRule for PushFilterDownTraverseRule {
    fn name(&self) -> &str {
        "PushFilterDownTraverseRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有且仅有一个子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为遍历操作
        if child_ref.plan_node.name() != "Traverse" {
            return Ok(None);
        }

        // 简化实现：保持原有结构，后续可以添加更复杂的条件分析
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> Traverse 模式
        PatternBuilder::with_dependency("Filter", "Traverse")
    }
}

impl BaseOptRule for PushFilterDownTraverseRule {}

/// 将过滤条件下推到扩展操作的规则
/// 
/// 该规则识别 Filter -> Expand 模式，
/// 并将过滤条件集成到扩展操作中。
#[derive(Debug)]
pub struct PushFilterDownExpandRule;

impl OptRule for PushFilterDownExpandRule {
    fn name(&self) -> &str {
        "PushFilterDownExpandRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有且仅有一个子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为扩展操作
        if child_ref.plan_node.name() != "Expand" {
            return Ok(None);
        }

        // 简化实现：保持原有结构
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> Expand 模式
        PatternBuilder::with_dependency("Filter", "Expand")
    }
}

impl BaseOptRule for PushFilterDownExpandRule {}

/// 将过滤条件下推到连接操作的规则
/// 
/// 该规则识别 Filter -> Join 模式，
/// 并将过滤条件下推到连接的一侧或两侧。
#[derive(Debug)]
pub struct PushFilterDownJoinRule;

impl OptRule for PushFilterDownJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否只有一个子节点（单输入连接）
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为连接操作
        if !matches!(child_ref.plan_node.name(), "HashInnerJoin" | "HashLeftJoin" | "HashRightJoin") {
            return Ok(None);
        }

        // 简化实现：保持原有结构，后续可以添加更复杂的连接条件下推
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> HashJoin 模式
        PatternBuilder::with_dependency("Filter", "HashInnerJoin")
    }
}

impl BaseOptRule for PushFilterDownJoinRule {}

/// 将过滤条件下推到Traverse/AppendVertices节点的规则
///
/// 该规则识别Traverse或AppendVertices节点中的顶点过滤条件，
/// 并将可以下推的条件提取出来推送到GetVertices操作。
#[derive(Debug)]
pub struct PushFilterDownNodeRule;

impl OptRule for PushFilterDownNodeRule {
    fn name(&self) -> &str {
        "PushFilterDownNodeRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 只处理Traverse或AppendVertices节点
        let node_name = node_ref.plan_node.name();
        if !matches!(node_name, "Traverse" | "AppendVertices") {
            return Ok(None);
        }

        // 获取vFilter
        let v_filter = match &node_ref.plan_node {
            PlanNodeEnum::Traverse(traverse) => traverse.v_filter().cloned(),
            PlanNodeEnum::AppendVertices(append) => append.v_filter().cloned(),
            _ => return Ok(None),
        };

        // 如果没有vFilter，则不进行转换
        let v_filter = match v_filter {
            Some(filter) => filter,
            None => return Ok(None),
        };

        // 使用表达式访问器分离可以下推和不能下推的表达式
        // 这里简化处理：假设所有表达式都可以下推
        // 在实际实现中，应该使用ExtractFilterExprVisitor来分离表达式
        
        // 创建新的节点
        let mut new_node = node_ref.clone();
        let new_group_node = Rc::new(RefCell::new(new_node));
        
        // 创建转换结果
        let mut result = TransformResult::new();
        result.add_new_group_node(new_group_node);
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::multi(vec!["Traverse", "AppendVertices"])
    }
}

impl BaseOptRule for PushFilterDownNodeRule {}

/// 将边过滤条件下推到Traverse节点的规则
///
/// 该规则识别Traverse节点中的边过滤条件，
/// 并将条件重写后推送到GetNeighbors操作。
#[derive(Debug)]
pub struct PushEFilterDownRule;

impl OptRule for PushEFilterDownRule {
    fn name(&self) -> &str {
        "PushEFilterDownRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 只处理Traverse节点
        let traverse = match &node_ref.plan_node {
            PlanNodeEnum::Traverse(traverse) => traverse,
            _ => return Ok(None),
        };

        // 获取eFilter
        let e_filter = match traverse.e_filter() {
            Some(filter) => filter.clone(),
            None => return Ok(None),
        };

        // 检查是否为零步遍历（零步遍历不能下推eFilter）
        // 这里简化处理：假设不是零步遍历
        // 在实际实现中，应该检查step_range.min() == 0

        // 重写边属性表达式
        // 这里简化处理：直接使用原始表达式
        // 在实际实现中，应该使用rewriteStarEdge函数重写表达式

        // 使用表达式访问器分离可以下推和不能下推的表达式
        // 这里简化处理：假设所有表达式都可以下推
        // 在实际实现中，应该使用ExtractFilterExprVisitor来分离表达式

        // 创建新的Traverse节点
        let mut new_traverse = traverse.clone();
        // 将eFilter设置为filter，将remainedExpr设置为eFilter
        // 这里简化处理：保持原有结构
        let mut new_node = node_ref.clone();
        let new_group_node = Rc::new(RefCell::new(new_node));
        
        // 创建转换结果
        let mut result = TransformResult::new();
        result.add_new_group_node(new_group_node);
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::single("Traverse")
    }
}

impl BaseOptRule for PushEFilterDownRule {}

/// 将顶点过滤条件下推到ScanVertices节点的规则
///
/// 该规则识别Filter -> AppendVertices -> ScanVertices模式，
/// 并将顶点过滤条件下推到ScanVertices操作。
#[derive(Debug)]
pub struct PushVFilterDownScanVerticesRule;

impl OptRule for PushVFilterDownScanVerticesRule {
    fn name(&self) -> &str {
        "PushVFilterDownScanVerticesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为AppendVertices节点
        let append_vertices = match &node_ref.plan_node {
            PlanNodeEnum::AppendVertices(append) => append,
            _ => return Ok(None),
        };

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为ScanVertices
        if child_ref.plan_node.name() != "ScanVertices" {
            return Ok(None);
        }

        // 获取vFilter
        let v_filter = match append_vertices.v_filter() {
            Some(filter) => filter.clone(),
            None => return Ok(None),
        };

        // 使用表达式访问器分离可以下推和不能下推的表达式
        // 这里简化处理：假设所有表达式都可以下推
        // 在实际实现中，应该使用ExtractFilterExprVisitor来分离表达式

        // 创建新的ScanVertices节点，集成过滤条件
        // 这里简化处理：保持原有结构
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::filter_with("AppendVertices")
    }
}

impl BaseOptRule for PushVFilterDownScanVerticesRule {}

/// 将过滤条件下推到InnerJoin节点的规则
///
/// 该规则识别Filter -> InnerJoin模式，
/// 并将过滤条件下推到InnerJoin的一侧或两侧。
#[derive(Debug)]
pub struct PushFilterDownInnerJoinRule;

impl OptRule for PushFilterDownInnerJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownInnerJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为InnerJoin
        if child_ref.plan_node.name() != "InnerJoin" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition(),
            None => return Ok(None),
        };

        // 根据过滤条件涉及的变量将条件下推到连接的一侧
        // 这里简化处理：假设所有条件都可以下推到左侧
        // 在实际实现中，应该根据变量来源决定下推到哪一侧

        // 创建新的左侧过滤节点
        // 这里简化处理：保持原有结构
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> InnerJoin 模式
        PatternBuilder::with_dependency("Filter", "InnerJoin")
    }
}

impl BaseOptRule for PushFilterDownInnerJoinRule {}

/// 将过滤条件下推到HashInnerJoin节点的规则
///
/// 该规则识别Filter -> HashInnerJoin模式，
/// 并将过滤条件下推到HashInnerJoin的一侧或两侧。
#[derive(Debug)]
pub struct PushFilterDownHashInnerJoinRule;

impl OptRule for PushFilterDownHashInnerJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownHashInnerJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为HashInnerJoin
        if child_ref.plan_node.name() != "HashInnerJoin" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition(),
            None => return Ok(None),
        };

        // 根据过滤条件涉及的变量将条件下推到连接的一侧
        // 这里简化处理：假设所有条件都可以下推到左侧
        // 在实际实现中，应该根据变量来源决定下推到哪一侧

        // 创建新的左侧过滤节点
        // 这里简化处理：保持原有结构
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> HashInnerJoin 模式
        PatternBuilder::with_dependency("Filter", "HashInnerJoin")
    }
}

impl BaseOptRule for PushFilterDownHashInnerJoinRule {}

/// 将过滤条件下推到HashLeftJoin节点的规则
///
/// 该规则识别Filter -> HashLeftJoin模式，
/// 并将过滤条件下推到HashLeftJoin的一侧或两侧。
#[derive(Debug)]
pub struct PushFilterDownHashLeftJoinRule;

impl OptRule for PushFilterDownHashLeftJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownHashLeftJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为HashLeftJoin
        if child_ref.plan_node.name() != "HashLeftJoin" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition(),
            None => return Ok(None),
        };

        // 根据过滤条件涉及的变量将条件下推到连接的一侧
        // 这里简化处理：假设所有条件都可以下推到左侧
        // 在实际实现中，应该根据变量来源决定下推到哪一侧

        // 创建新的左侧过滤节点
        // 这里简化处理：保持原有结构
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> HashLeftJoin 模式
        PatternBuilder::with_dependency("Filter", "HashLeftJoin")
    }
}

impl BaseOptRule for PushFilterDownHashLeftJoinRule {}

/// 将过滤条件下推到CrossJoin节点的规则
///
/// 该规则识别Filter -> CrossJoin模式，
/// 并将过滤条件下推到CrossJoin的一侧或两侧。
#[derive(Debug)]
pub struct PushFilterDownCrossJoinRule;

impl OptRule for PushFilterDownCrossJoinRule {
    fn name(&self) -> &str {
        "PushFilterDownCrossJoinRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为CrossJoin
        if child_ref.plan_node.name() != "CrossJoin" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition(),
            None => return Ok(None),
        };

        // 根据过滤条件涉及的变量将条件下推到交叉连接的一侧
        // 这里简化处理：假设所有条件都可以下推到左侧
        // 在实际实现中，应该根据变量来源决定下推到哪一侧

        // 创建新的左侧过滤节点
        // 这里简化处理：保持原有结构
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> CrossJoin 模式
        PatternBuilder::with_dependency("Filter", "CrossJoin")
    }
}

impl BaseOptRule for PushFilterDownCrossJoinRule {}

/// 将过滤条件下推到GetNeighbors节点的规则
///
/// 该规则识别Filter -> GetNeighbors模式，
/// 并将过滤条件下推到GetNeighbors操作。
#[derive(Debug)]
pub struct PushFilterDownGetNbrsRule;

impl OptRule for PushFilterDownGetNbrsRule {
    fn name(&self) -> &str {
        "PushFilterDownGetNbrsRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为GetNeighbors
        if child_ref.plan_node.name() != "GetNeighbors" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition(),
            None => return Ok(None),
        };

        // 将过滤条件下推到GetNeighbors操作
        // 这里简化处理：保持原有结构
        // 在实际实现中，应该将过滤条件集成到GetNeighbors的filter字段中
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> GetNeighbors 模式
        PatternBuilder::with_dependency("Filter", "GetNeighbors")
    }
}

impl BaseOptRule for PushFilterDownGetNbrsRule {}

/// 将过滤条件下推到ExpandAll节点的规则
///
/// 该规则识别Filter -> ExpandAll模式，
/// 并将过滤条件下推到ExpandAll操作。
#[derive(Debug)]
pub struct PushFilterDownExpandAllRule;

impl OptRule for PushFilterDownExpandAllRule {
    fn name(&self) -> &str {
        "PushFilterDownExpandAllRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为ExpandAll
        if child_ref.plan_node.name() != "ExpandAll" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition(),
            None => return Ok(None),
        };

        // 将过滤条件下推到ExpandAll操作
        // 这里简化处理：保持原有结构
        // 在实际实现中，应该将过滤条件集成到ExpandAll的filter字段中
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> ExpandAll 模式
        PatternBuilder::with_dependency("Filter", "ExpandAll")
    }
}

impl BaseOptRule for PushFilterDownExpandAllRule {}

/// 将过滤条件下推到AllPaths节点的规则
///
/// 该规则识别Filter -> AllPaths模式，
/// 并将过滤条件下推到AllPaths操作。
#[derive(Debug)]
pub struct PushFilterDownAllPathsRule;

impl OptRule for PushFilterDownAllPathsRule {
    fn name(&self) -> &str {
        "PushFilterDownAllPathsRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> OptResult<Option<TransformResult>> {
        let node_ref = group_node.borrow();
        
        // 检查是否为过滤节点
        if !node_ref.plan_node.is_filter() {
            return Ok(None);
        }

        // 检查是否有子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(node) => node,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();
        
        // 检查子节点是否为AllPaths
        if child_ref.plan_node.name() != "AllPaths" {
            return Ok(None);
        }

        // 获取过滤条件
        let filter_condition = match node_ref.plan_node.as_filter() {
            Some(filter) => filter.condition(),
            None => return Ok(None),
        };

        // 将过滤条件下推到AllPaths操作
        // 这里简化处理：保持原有结构
        // 在实际实现中，应该将过滤条件集成到AllPaths的filter字段中
        let mut result = TransformResult::new();
        result.add_new_group_node(group_node.clone());
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Filter -> AllPaths 模式
        PatternBuilder::with_dependency("Filter", "AllPaths")
    }
}

impl BaseOptRule for PushFilterDownAllPathsRule {}

/// 谓词下推规则集合
/// 
/// 提供所有谓词下推规则的便捷访问。
pub struct PredicatePushDownRules;

impl PredicatePushDownRules {
    /// 获取所有谓词下推规则
    pub fn all_rules() -> Vec<Box<dyn OptRule>> {
        vec![
            Box::new(PushFilterDownScanVerticesRule),
            Box::new(PushFilterDownTraverseRule),
            Box::new(PushFilterDownExpandRule),
            Box::new(PushFilterDownJoinRule),
            Box::new(PushFilterDownNodeRule),
            Box::new(PushEFilterDownRule),
            Box::new(PushVFilterDownScanVerticesRule),
            Box::new(PushFilterDownInnerJoinRule),
            Box::new(PushFilterDownHashInnerJoinRule),
            Box::new(PushFilterDownHashLeftJoinRule),
            Box::new(PushFilterDownCrossJoinRule),
            Box::new(PushFilterDownGetNbrsRule),
            Box::new(PushFilterDownExpandAllRule),
            Box::new(PushFilterDownAllPathsRule),
        ]
    }
}