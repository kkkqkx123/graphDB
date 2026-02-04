//! 谓词下推优化规则
//! 这些规则负责将过滤条件下推到计划树的底层，以减少数据处理量

use super::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result as OptResult};
use super::rule_patterns::{CommonPatterns, PatternBuilder};
use super::rule_traits::BaseOptRule;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
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
        ]
    }
}