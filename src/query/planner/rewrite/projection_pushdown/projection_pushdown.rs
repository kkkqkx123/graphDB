//! 投影下推优化规则
//!
//! 这些规则负责将投影操作下推到计划树的底层，以减少数据传输量。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//! Project(col1, col2)
//!         |
//!       ScanVertices
//! ```
//!
//! After:
//! ```text
//! ScanVertices(col1, col2)
//! ```
//!
//! # 适用条件
//!
//! - Project 节点的子节点是数据访问节点（ScanVertices、ScanEdges、GetVertices、GetEdges、GetNeighbors）
//! - Project 节点有列定义

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};

/// 投影下推规则
///
/// 将投影操作下推到数据访问节点，减少不必要的数据传输
#[derive(Debug)]
pub struct ProjectionPushDownRule;

impl ProjectionPushDownRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查是否可以下推投影
    fn can_push_down_project(node: &crate::query::planner::plan::core::nodes::ProjectNode) -> bool {
        !node.columns().is_empty()
    }

    /// 检查节点是否支持投影下推
    fn is_push_down_target(node: &PlanNodeEnum) -> bool {
        matches!(
            node,
            PlanNodeEnum::ScanVertices(_)
                | PlanNodeEnum::ScanEdges(_)
                | PlanNodeEnum::GetVertices(_)
                | PlanNodeEnum::GetEdges(_)
                | PlanNodeEnum::GetNeighbors(_)
                | PlanNodeEnum::EdgeIndexScan(_)
        )
    }

    /// 创建带有投影列的数据访问节点
    fn create_node_with_projection(
        &self,
        target: &PlanNodeEnum,
        project_columns: &[crate::core::YieldColumn],
    ) -> Option<PlanNodeEnum> {
        let col_names: Vec<String> = project_columns
            .iter()
            .map(|col| col.alias.clone())
            .collect();

        match target {
            PlanNodeEnum::ScanVertices(node) => {
                let mut new_node = node.clone();
                new_node.set_col_names(col_names);
                Some(PlanNodeEnum::ScanVertices(new_node))
            }
            PlanNodeEnum::ScanEdges(node) => {
                let mut new_node = node.clone();
                new_node.set_col_names(col_names);
                Some(PlanNodeEnum::ScanEdges(new_node))
            }
            PlanNodeEnum::GetVertices(node) => {
                let mut new_node = node.clone();
                new_node.set_col_names(col_names);
                Some(PlanNodeEnum::GetVertices(new_node))
            }
            PlanNodeEnum::GetEdges(node) => {
                let mut new_node = node.clone();
                new_node.set_col_names(col_names);
                Some(PlanNodeEnum::GetEdges(new_node))
            }
            PlanNodeEnum::GetNeighbors(node) => {
                let mut new_node = node.clone();
                new_node.set_col_names(col_names);
                Some(PlanNodeEnum::GetNeighbors(new_node))
            }
            PlanNodeEnum::EdgeIndexScan(node) => {
                let mut new_node = node.clone();
                let return_cols: Vec<String> = project_columns
                    .iter()
                    .map(|col| col.alias.clone())
                    .collect();
                new_node.set_return_columns(return_cols);
                new_node.set_col_names(col_names);
                Some(PlanNodeEnum::EdgeIndexScan(new_node))
            }
            _ => None,
        }
    }
}

impl Default for ProjectionPushDownRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for ProjectionPushDownRule {
    fn name(&self) -> &'static str {
        "ProjectionPushDownRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Project")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Project 节点
        let project_node = match node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 检查是否可以下推
        if !Self::can_push_down_project(project_node) {
            return Ok(None);
        }

        // 获取输入节点
        let input = project_node.input();

        // 检查输入节点类型是否支持下推
        if !Self::is_push_down_target(input) {
            return Ok(None);
        }

        // 创建带有投影列的新数据访问节点
        let columns = project_node.columns();
        let new_node = match self.create_node_with_projection(input, columns) {
            Some(node) => node,
            None => return Ok(None),
        };

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(new_node);

        Ok(Some(result))
    }
}

impl PushDownRule for ProjectionPushDownRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Project(project) => {
                if project.columns().is_empty() {
                    return false;
                }
                Self::is_push_down_target(target)
            }
            _ => false,
        }
    }

    fn push_down(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        _target: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Expression, YieldColumn};
    use crate::query::planner::plan::core::nodes::{
        ProjectNode, ScanVerticesNode, StartNode,
    };

    #[test]
    fn test_rule_name() {
        let rule = ProjectionPushDownRule::new();
        assert_eq!(rule.name(), "ProjectionPushDownRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = ProjectionPushDownRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_can_push_down_target() {
        let scan = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        assert!(ProjectionPushDownRule::is_push_down_target(&scan));

        let start = PlanNodeEnum::Start(StartNode::new());
        assert!(!ProjectionPushDownRule::is_push_down_target(&start));
    }

    #[test]
    fn test_apply_with_scan_vertices() {
        let rule = ProjectionPushDownRule::new();
        let mut ctx = RewriteContext::new();

        // 创建 ScanVertices 节点
        let scan_node = ScanVerticesNode::new(1);
        let scan = PlanNodeEnum::ScanVertices(scan_node);

        // 创建 Project 节点
        let columns = vec![
            YieldColumn {
                expression: Expression::Variable("name".to_string()),
                alias: "name".to_string(),
                is_matched: false,
            },
            YieldColumn {
                expression: Expression::Variable("age".to_string()),
                alias: "age".to_string(),
                is_matched: false,
            },
        ];
        let project = ProjectNode::new(scan.clone(), columns).expect("创建 ProjectNode 失败");
        let project_enum = PlanNodeEnum::Project(project);

        // 应用规则
        let result = rule.apply(&mut ctx, &project_enum).expect("应用规则失败");

        assert!(result.is_some());
        let transform = result.unwrap();
        assert!(transform.erase_curr);
        assert_eq!(transform.new_nodes.len(), 1);

        // 验证新节点是 ScanVertices 且带有正确的列名
        match &transform.new_nodes[0] {
            PlanNodeEnum::ScanVertices(node) => {
                assert_eq!(node.col_names(), &["name", "age"]);
            }
            _ => panic!("期望 ScanVertices 节点"),
        }
    }

    #[test]
    fn test_apply_with_non_pushable_target() {
        let rule = ProjectionPushDownRule::new();
        let mut ctx = RewriteContext::new();

        // 创建 Start 节点（不支持下推）
        let start_node = StartNode::new();
        let start = PlanNodeEnum::Start(start_node);

        // 创建 Project 节点
        let columns = vec![YieldColumn {
            expression: Expression::Variable("test".to_string()),
            alias: "test".to_string(),
            is_matched: false,
        }];
        let project = ProjectNode::new(start.clone(), columns).expect("创建 ProjectNode 失败");
        let project_enum = PlanNodeEnum::Project(project);

        // 应用规则
        let result = rule.apply(&mut ctx, &project_enum).expect("应用规则失败");

        // 不支持下推，返回 None
        assert!(result.is_none());
    }

    #[test]
    fn test_push_down_rule_trait() {
        let rule = ProjectionPushDownRule::new();

        let scan = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        let columns = vec![YieldColumn {
            expression: Expression::Variable("test".to_string()),
            alias: "test".to_string(),
            is_matched: false,
        }];
        let project = ProjectNode::new(scan.clone(), columns).expect("创建 ProjectNode 失败");
        let project_enum = PlanNodeEnum::Project(project);

        assert!(rule.can_push_down(&project_enum, &scan));

        let start = PlanNodeEnum::Start(StartNode::new());
        assert!(!rule.can_push_down(&project_enum, &start));
    }
}
