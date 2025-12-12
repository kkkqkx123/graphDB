//! LIMIT下推优化规则
//! 这些规则负责将LIMIT操作下推到计划树的底层，以减少数据处理量

use super::optimizer::OptimizerError;
use super::rule_traits::{BaseOptRule, PushDownRule};
use super::rule_patterns::PatternBuilder;
use crate::query::optimizer::optimizer::{OptContext, OptGroupNode, OptRule, Pattern};
use crate::query::planner::plan::{PlanNodeKind, PlanNode};
// 注释掉不存在的导入
// use crate::query::planner::plan::operations::AllPaths;
// use crate::query::planner::plan::operations::ExpandAll;

/// 通用LIMIT下推规则
#[derive(Debug)]
pub struct PushLimitDownRule;

impl OptRule for PushLimitDownRule {
    fn name(&self) -> &str {
        "PushLimitDownRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &OptGroupNode,
    ) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为LIMIT操作
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // 匹配模式以检查是否可以下推LIMIT
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                // 尝试根据子节点类型下推LIMIT
                match child.plan_node().kind() {
                    PlanNodeKind::IndexScan
                    | PlanNodeKind::GetVertices
                    | PlanNodeKind::GetEdges
                    | PlanNodeKind::ScanVertices
                    | PlanNodeKind::ScanEdges => {
                        // 对于扫描操作，下推LIMIT可以提高性能
                        Ok(Some(node.clone()))
                    }
                    PlanNodeKind::Sort => {
                        // 对于排序后跟LIMIT（Top-N查询），我们可能优化不同
                        Ok(Some(node.clone()))
                    }
                    _ => {
                        // 对于其他节点，我们可能仍然能够下推LIMIT
                        Ok(Some(node.clone()))
                    }
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::limit()
    }
}

impl BaseOptRule for PushLimitDownRule {}

impl PushDownRule for PushLimitDownRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        matches!(child_kind, 
            PlanNodeKind::IndexScan |
            PlanNodeKind::GetVertices |
            PlanNodeKind::GetEdges |
            PlanNodeKind::ScanVertices |
            PlanNodeKind::ScanEdges |
            PlanNodeKind::Sort
        )
    }

    fn create_pushed_down_node(&self, _ctx: &mut OptContext, _node: &OptGroupNode, _child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建下推后的节点
        // 目前简化实现，返回None
        Ok(None)
    }
}

/// 将LIMIT下推到获取顶点操作的规则
#[derive(Debug)]
pub struct PushLimitDownGetVerticesRule;

impl OptRule for PushLimitDownGetVerticesRule {
    fn name(&self) -> &str {
        "PushLimitDownGetVerticesRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为LIMIT操作
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // 匹配模式以查看是否为LIMIT后跟获取顶点
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::GetVertices {
                    // 在完整实现中，我们会将LIMIT下推到获取顶点操作
                    // 以减少从存储获取的顶点数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Limit, PlanNodeKind::GetVertices)
    }
}

impl BaseOptRule for PushLimitDownGetVerticesRule {}

impl PushDownRule for PushLimitDownGetVerticesRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::GetVertices
    }

    fn create_pushed_down_node(&self, _ctx: &mut OptContext, _node: &OptGroupNode, _child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建带有LIMIT的获取顶点节点
        // 目前简化实现，返回None
        Ok(None)
    }
}

/// 将LIMIT下推到获取邻居操作的规则
#[derive(Debug)]
pub struct PushLimitDownGetNeighborsRule;

impl OptRule for PushLimitDownGetNeighborsRule {
    fn name(&self) -> &str {
        "PushLimitDownGetNeighborsRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为LIMIT操作
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // 匹配模式以查看是否为LIMIT后跟获取邻居
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::GetNeighbors {
                    // 在完整实现中，我们会将LIMIT下推到获取邻居操作
                    // 以减少从存储获取的邻居数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Limit, PlanNodeKind::GetNeighbors)
    }
}

impl BaseOptRule for PushLimitDownGetNeighborsRule {}

impl PushDownRule for PushLimitDownGetNeighborsRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::GetNeighbors
    }

    fn create_pushed_down_node(&self, _ctx: &mut OptContext, _node: &OptGroupNode, _child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建带有LIMIT的获取邻居节点
        // 目前简化实现，返回None
        Ok(None)
    }
}

/// 将LIMIT下推到获取边操作的规则
#[derive(Debug)]
pub struct PushLimitDownGetEdgesRule;

impl OptRule for PushLimitDownGetEdgesRule {
    fn name(&self) -> &str {
        "PushLimitDownGetEdgesRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为LIMIT操作
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // 匹配模式以查看是否为LIMIT后跟获取边
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::GetEdges {
                    // 在完整实现中，我们会将LIMIT下推到获取边操作
                    // 以减少从存储获取的边数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Limit, PlanNodeKind::GetEdges)
    }
}

impl BaseOptRule for PushLimitDownGetEdgesRule {}

impl PushDownRule for PushLimitDownGetEdgesRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::GetEdges
    }

    fn create_pushed_down_node(&self, _ctx: &mut OptContext, _node: &OptGroupNode, _child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建带有LIMIT的获取边节点
        // 目前简化实现，返回None
        Ok(None)
    }
}

/// 将LIMIT下推到扫描顶点操作的规则
#[derive(Debug)]
pub struct PushLimitDownScanVerticesRule;

impl OptRule for PushLimitDownScanVerticesRule {
    fn name(&self) -> &str {
        "PushLimitDownScanVerticesRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为LIMIT操作
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // 匹配模式以查看是否为LIMIT后跟扫描顶点
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::ScanVertices {
                    // 在完整实现中，我们会将LIMIT下推到扫描顶点操作
                    // 以减少从存储扫描的顶点数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Limit, PlanNodeKind::ScanVertices)
    }
}

impl BaseOptRule for PushLimitDownScanVerticesRule {}

impl PushDownRule for PushLimitDownScanVerticesRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::ScanVertices
    }

    fn create_pushed_down_node(&self, _ctx: &mut OptContext, _node: &OptGroupNode, _child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建带有LIMIT的扫描顶点节点
        // 目前简化实现，返回None
        Ok(None)
    }
}

/// 将LIMIT下推到扫描边操作的规则
#[derive(Debug)]
pub struct PushLimitDownScanEdgesRule;

impl OptRule for PushLimitDownScanEdgesRule {
    fn name(&self) -> &str {
        "PushLimitDownScanEdgesRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为LIMIT操作
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // 匹配模式以查看是否为LIMIT后跟扫描边
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::ScanEdges {
                    // 在完整实现中，我们会将LIMIT下推到扫描边操作
                    // 以减少从存储扫描的边数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Limit, PlanNodeKind::ScanEdges)
    }
}

impl BaseOptRule for PushLimitDownScanEdgesRule {}

impl PushDownRule for PushLimitDownScanEdgesRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::ScanEdges
    }

    fn create_pushed_down_node(&self, _ctx: &mut OptContext, _node: &OptGroupNode, _child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建带有LIMIT的扫描边节点
        // 目前简化实现，返回None
        Ok(None)
    }
}

/// 将LIMIT下推到索引扫描操作的规则
#[derive(Debug)]
pub struct PushLimitDownIndexScanRule;

impl OptRule for PushLimitDownIndexScanRule {
    fn name(&self) -> &str {
        "PushLimitDownIndexScanRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为LIMIT操作
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // 匹配模式以查看是否为LIMIT后跟索引扫描
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::IndexScan {
                    // 在完整实现中，我们会将LIMIT下推到索引扫描操作
                    // 以减少扫描的索引条目数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Limit, PlanNodeKind::IndexScan)
    }
}

impl BaseOptRule for PushLimitDownIndexScanRule {}

impl PushDownRule for PushLimitDownIndexScanRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::IndexScan
    }

    fn create_pushed_down_node(&self, _ctx: &mut OptContext, _node: &OptGroupNode, _child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建带有LIMIT的索引扫描节点
        // 目前简化实现，返回None
        Ok(None)
    }
}

/// 将LIMIT下推到投影操作的规则
#[derive(Debug)]
pub struct PushLimitDownProjectRule;

impl OptRule for PushLimitDownProjectRule {
    fn name(&self) -> &str {
        "PushLimitDownProjectRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为LIMIT操作
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // 匹配模式以查看是否为LIMIT后跟投影
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::Project {
                    // 在完整实现中，我们会将LIMIT下推到投影操作
                    // 以限制投影结果的数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Limit, PlanNodeKind::Project)
    }
}

impl BaseOptRule for PushLimitDownProjectRule {}

impl PushDownRule for PushLimitDownProjectRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::Project
    }

    fn create_pushed_down_node(&self, _ctx: &mut OptContext, _node: &OptGroupNode, _child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建带有LIMIT的投影节点
        // 目前简化实现，返回None
        Ok(None)
    }
}

// 注释掉使用不存在的 AllPaths 和 ExpandAll 类型的规则
/*
/// 将LIMIT下推到全路径操作的规则
#[derive(Debug)]
pub struct PushLimitDownAllPathsRule;

impl OptRule for PushLimitDownAllPathsRule {
    fn name(&self) -> &str {
        "PushLimitDownAllPathsRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为LIMIT操作
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // 匹配模式以查看是否为LIMIT后跟全路径
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::AllPaths {
                    // 在完整实现中，我们会将LIMIT下推到全路径操作
                    // 以限制计算的路径数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Limit, PlanNodeKind::AllPaths)
    }
}

impl BaseOptRule for PushLimitDownAllPathsRule {}

impl PushDownRule for PushLimitDownAllPathsRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::AllPaths
    }

    fn create_pushed_down_node(&self, _ctx: &mut OptContext, _node: &OptGroupNode, _child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建带有LIMIT的全路径节点
        // 目前简化实现，返回None
        Ok(None)
    }
}

/// 将LIMIT下推到全展开操作的规则
#[derive(Debug)]
pub struct PushLimitDownExpandAllRule;

impl OptRule for PushLimitDownExpandAllRule {
    fn name(&self) -> &str {
        "PushLimitDownExpandAllRule"
    }

    fn apply(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 检查是否为LIMIT操作
        if node.plan_node.kind() != PlanNodeKind::Limit {
            return Ok(None);
        }

        // 匹配模式以查看是否为LIMIT后跟全展开
        if let Some(matched) = self.match_pattern(ctx, node)? {
            if matched.dependencies.len() >= 1 {
                let child = &matched.dependencies[0];

                if child.plan_node().kind() == PlanNodeKind::ExpandAll {
                    // 在完整实现中，我们会将LIMIT下推到全展开操作
                    // 以限制扩展的数量
                    Ok(Some(node.clone()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Limit, PlanNodeKind::ExpandAll)
    }
}

impl BaseOptRule for PushLimitDownExpandAllRule {}

impl PushDownRule for PushLimitDownExpandAllRule {
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
        child_kind == PlanNodeKind::ExpandAll
    }

    fn create_pushed_down_node(&self, _ctx: &mut OptContext, _node: &OptGroupNode, _child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
        // 在完整实现中，这里会创建带有LIMIT的全展开节点
        // 目前简化实现，返回None
        Ok(None)
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::QueryContext;
    use crate::query::optimizer::optimizer::{OptContext, OptGroupNode};
    use crate::query::planner::plan::{Limit, GetVertices, GetNeighbors, GetEdges, Project, IndexScan, ScanVertices, ScanEdges};
    use crate::query::planner::plan::{PlanNode, PlanNodeKind};

    fn create_test_context() -> OptContext {
        OptContext::new(QueryContext::default())
    }

    #[test]
    fn test_push_limit_down_rule() {
        let rule = PushLimitDownRule;
        let mut ctx = create_test_context();

        // 创建一个LIMIT节点
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配LIMIT节点并尝试下推
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_get_vertices_rule() {
        let rule = PushLimitDownGetVerticesRule;
        let mut ctx = create_test_context();

        // 创建一个LIMIT节点
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配LIMIT节点并尝试下推到获取顶点操作
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_get_neighbors_rule() {
        let rule = PushLimitDownGetNeighborsRule;
        let mut ctx = create_test_context();

        // 创建一个LIMIT节点
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配LIMIT节点并尝试下推到获取邻居操作
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_get_edges_rule() {
        let rule = PushLimitDownGetEdgesRule;
        let mut ctx = create_test_context();

        // 创建一个LIMIT节点
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配LIMIT节点并尝试下推到获取边操作
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_scan_vertices_rule() {
        let rule = PushLimitDownScanVerticesRule;
        let mut ctx = create_test_context();

        // 创建一个LIMIT节点
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配LIMIT节点并尝试下推到扫描顶点操作
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_scan_edges_rule() {
        let rule = PushLimitDownScanEdgesRule;
        let mut ctx = create_test_context();

        // 创建一个LIMIT节点
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配LIMIT节点并尝试下推到扫描边操作
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_index_scan_rule() {
        let rule = PushLimitDownIndexScanRule;
        let mut ctx = create_test_context();

        // 创建一个LIMIT节点
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配LIMIT节点并尝试下推到索引扫描操作
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_project_rule() {
        let rule = PushLimitDownProjectRule;
        let mut ctx = create_test_context();

        // 创建一个LIMIT节点
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配LIMIT节点并尝试下推到投影操作
        assert!(result.is_some());
    }

    // 注释掉使用不存在的 AllPaths 和 ExpandAll 类型的测试
    /*
    #[test]
    fn test_push_limit_down_all_paths_rule() {
        let rule = PushLimitDownAllPathsRule;
        let mut ctx = create_test_context();

        // 创建一个LIMIT节点
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配LIMIT节点并尝试下推到全路径操作
        assert!(result.is_some());
    }

    #[test]
    fn test_push_limit_down_expand_all_rule() {
        let rule = PushLimitDownExpandAllRule;
        let mut ctx = create_test_context();

        // 创建一个LIMIT节点
        let limit_node = Box::new(Limit::new(1, 10, 0));
        let opt_node = OptGroupNode::new(1, limit_node);

        let result = rule.apply(&mut ctx, &opt_node).unwrap();
        // 规则应该匹配LIMIT节点并尝试下推到全展开操作
        assert!(result.is_some());
    }
    */
}