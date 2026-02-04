//! LIMIT下推优化规则
//! 这些规则负责将LIMIT操作下推到计划树的底层，以减少数据处理量

use super::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult};
use super::rule_patterns::PatternBuilder;
use super::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use std::rc::Rc;
use std::cell::RefCell;

/// 将LIMIT下推到获取顶点操作的规则
/// 
/// 该规则识别 Limit -> GetVertices 模式，
/// 并将LIMIT值集成到GetVertices操作中。
#[derive(Debug)]
pub struct PushLimitDownGetVerticesRule;

impl OptRule for PushLimitDownGetVerticesRule {
    fn name(&self) -> &str {
        "PushLimitDownGetVerticesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, super::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
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
        
        // 检查子节点是否为获取顶点操作
        if !child_ref.plan_node.is_get_vertices() {
            return Ok(None);
        }

        // 获取LIMIT值
        let limit_value = match node_ref.plan_node.as_limit() {
            Some(limit) => limit.count(),
            None => return Ok(None),
        };

        // 简化实现：创建新的GetVertices节点，集成LIMIT值
        if let Some(get_vertices) = child_ref.plan_node.as_get_vertices() {
            let mut new_get_vertices = get_vertices.clone();
            new_get_vertices.set_limit(limit_value);
            
            // 设置输出变量
            if let Some(output_var) = node_ref.plan_node.output_var() {
                new_get_vertices.set_output_var(output_var.clone());
            }

            // 创建新的组节点
            let mut new_node = child_ref.clone();
            new_node.plan_node = PlanNodeEnum::GetVertices(new_get_vertices);

            let mut result = TransformResult::new();
            result.add_new_group_node(Rc::new(RefCell::new(new_node)));
            return Ok(Some(result));
        }

        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Limit -> GetVertices 模式
        PatternBuilder::with_dependency("Limit", "GetVertices")
    }
}

impl BaseOptRule for PushLimitDownGetVerticesRule {}

/// 将LIMIT下推到获取边操作的规则
/// 
/// 该规则识别 Limit -> GetEdges 模式，
/// 并将LIMIT值集成到GetEdges操作中。
#[derive(Debug)]
pub struct PushLimitDownGetEdgesRule;

impl OptRule for PushLimitDownGetEdgesRule {
    fn name(&self) -> &str {
        "PushLimitDownGetEdgesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, super::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
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
        
        // 检查子节点是否为获取边操作
        if !child_ref.plan_node.is_get_edges() {
            return Ok(None);
        }

        // 获取LIMIT值
        let limit_value = match node_ref.plan_node.as_limit() {
            Some(limit) => limit.count(),
            None => return Ok(None),
        };

        // 简化实现：创建新的GetEdges节点，集成LIMIT值
        if let Some(get_edges) = child_ref.plan_node.as_get_edges() {
            let mut new_get_edges = get_edges.clone();
            new_get_edges.set_limit(limit_value);
            
            // 设置输出变量
            if let Some(output_var) = node_ref.plan_node.output_var() {
                new_get_edges.set_output_var(output_var.clone());
            }

            // 创建新的组节点
            let mut new_node = child_ref.clone();
            new_node.plan_node = PlanNodeEnum::GetEdges(new_get_edges);

            let mut result = TransformResult::new();
            result.add_new_group_node(Rc::new(RefCell::new(new_node)));
            return Ok(Some(result));
        }

        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Limit -> GetEdges 模式
        PatternBuilder::with_dependency("Limit", "GetEdges")
    }
}

impl BaseOptRule for PushLimitDownGetEdgesRule {}

/// 将LIMIT下推到扫描顶点操作的规则
/// 
/// 该规则识别 Limit -> ScanVertices 模式，
/// 并将LIMIT值集成到ScanVertices操作中。
#[derive(Debug)]
pub struct PushLimitDownScanVerticesRule;

impl OptRule for PushLimitDownScanVerticesRule {
    fn name(&self) -> &str {
        "PushLimitDownScanVerticesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, super::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
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
        
        // 检查子节点是否为扫描顶点操作
        if !child_ref.plan_node.is_scan_vertices() {
            return Ok(None);
        }

        // 获取LIMIT值
        let limit_value = match node_ref.plan_node.as_limit() {
            Some(limit) => limit.count(),
            None => return Ok(None),
        };

        // 简化实现：创建新的ScanVertices节点，集成LIMIT值
        if let Some(scan_vertices) = child_ref.plan_node.as_scan_vertices() {
            let mut new_scan_vertices = scan_vertices.clone();
            new_scan_vertices.set_limit(limit_value);
            
            // 设置输出变量
            if let Some(output_var) = node_ref.plan_node.output_var() {
                new_scan_vertices.set_output_var(output_var.clone());
            }

            // 创建新的组节点
            let mut new_node = child_ref.clone();
            new_node.plan_node = PlanNodeEnum::ScanVertices(new_scan_vertices);

            let mut result = TransformResult::new();
            result.add_new_group_node(Rc::new(RefCell::new(new_node)));
            return Ok(Some(result));
        }

        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Limit -> ScanVertices 模式
        PatternBuilder::with_dependency("Limit", "ScanVertices")
    }
}

impl BaseOptRule for PushLimitDownScanVerticesRule {}

/// 将LIMIT下推到扫描边操作的规则
/// 
/// 该规则识别 Limit -> ScanEdges 模式，
/// 并将LIMIT值集成到ScanEdges操作中。
#[derive(Debug)]
pub struct PushLimitDownScanEdgesRule;

impl OptRule for PushLimitDownScanEdgesRule {
    fn name(&self) -> &str {
        "PushLimitDownScanEdgesRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, super::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
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
        
        // 检查子节点是否为扫描边操作
        if !child_ref.plan_node.is_scan_edges() {
            return Ok(None);
        }

        // 获取LIMIT值
        let limit_value = match node_ref.plan_node.as_limit() {
            Some(limit) => limit.count(),
            None => return Ok(None),
        };

        // 简化实现：创建新的ScanEdges节点，集成LIMIT值
        if let Some(scan_edges) = child_ref.plan_node.as_scan_edges() {
            let mut new_scan_edges = scan_edges.clone();
            new_scan_edges.set_limit(limit_value);
            
            // 设置输出变量
            if let Some(output_var) = node_ref.plan_node.output_var() {
                new_scan_edges.set_output_var(output_var.clone());
            }

            // 创建新的组节点
            let mut new_node = child_ref.clone();
            new_node.plan_node = PlanNodeEnum::ScanEdges(new_scan_edges);

            let mut result = TransformResult::new();
            result.add_new_group_node(Rc::new(RefCell::new(new_node)));
            return Ok(Some(result));
        }

        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Limit -> ScanEdges 模式
        PatternBuilder::with_dependency("Limit", "ScanEdges")
    }
}

impl BaseOptRule for PushLimitDownScanEdgesRule {}

/// 将LIMIT下推到索引扫描操作的规则
/// 
/// 该规则识别 Limit -> IndexScan 模式，
/// 并将LIMIT值集成到IndexScan操作中。
#[derive(Debug)]
pub struct PushLimitDownIndexScanRule;

impl OptRule for PushLimitDownIndexScanRule {
    fn name(&self) -> &str {
        "PushLimitDownIndexScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, super::engine::OptimizerError> {
        let node_ref = group_node.borrow();
        
        // 检查是否为LIMIT操作
        if !node_ref.plan_node.is_limit() {
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
        
        // 检查子节点是否为索引扫描操作
        if !child_ref.plan_node.is_index_scan() {
            return Ok(None);
        }

        // 获取LIMIT值
        let limit_value = match node_ref.plan_node.as_limit() {
            Some(limit) => limit.count(),
            None => return Ok(None),
        };

        // 简化实现：创建新的IndexScan节点，集成LIMIT值
        if let Some(index_scan) = child_ref.plan_node.as_index_scan() {
            let mut new_index_scan = index_scan.clone();
            new_index_scan.set_limit(limit_value);
            
            // 设置输出变量
            if let Some(output_var) = node_ref.plan_node.output_var() {
                new_index_scan.set_output_var(output_var.clone());
            }

            // 创建新的组节点
            let mut new_node = child_ref.clone();
            new_node.plan_node = PlanNodeEnum::IndexScan(new_index_scan);

            let mut result = TransformResult::new();
            result.add_new_group_node(Rc::new(RefCell::new(new_node)));
            return Ok(Some(result));
        }

        Ok(None)
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Limit -> IndexScan 模式
        PatternBuilder::with_dependency("Limit", "IndexScan")
    }
}

impl BaseOptRule for PushLimitDownIndexScanRule {}

/// LIMIT下推规则集合
/// 
/// 提供所有LIMIT下推规则的便捷访问。
pub struct LimitPushDownRules;

impl LimitPushDownRules {
    /// 获取所有LIMIT下推规则
    pub fn all_rules() -> Vec<Box<dyn OptRule>> {
        vec![
            Box::new(PushLimitDownGetVerticesRule),
            Box::new(PushLimitDownGetEdgesRule),
            Box::new(PushLimitDownScanVerticesRule),
            Box::new(PushLimitDownScanEdgesRule),
            Box::new(PushLimitDownIndexScanRule),
        ]
    }
}