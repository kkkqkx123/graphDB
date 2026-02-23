//! 向数据源推送投影操作的规则
//!
//! 该规则将投影操作推向数据源，减少数据传输量。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//! Project(col1)
//!       |
//!   ScanVertices
//! ```
//!
//! After:
//! ```text
//! ScanVertices(col1)
//! ```
//!
//! # 适用条件
//!
//! - Project 节点的子节点是数据访问节点（ScanVertices、ScanEdges、GetVertices）
//! - Project 节点有列定义

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, PushDownRule};

/// 向数据源推送投影操作的规则
///
/// 该规则专注于将投影操作下推到最底层的数据访问节点，
/// 从而减少查询执行过程中的数据传输量
#[derive(Debug)]
pub struct PushProjectDownRule;

impl PushProjectDownRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查是否可以下推投影
    fn can_push_down_project(node: &crate::query::planner::plan::core::nodes::ProjectNode) -> bool {
        !node.columns().is_empty()
    }

    /// 检查节点是否是数据源节点（支持投影下推）
    fn is_data_source(node: &PlanNodeEnum) -> bool {
        matches!(
            node,
            PlanNodeEnum::ScanVertices(_)
                | PlanNodeEnum::ScanEdges(_)
                | PlanNodeEnum::GetVertices(_)
                | PlanNodeEnum::GetEdges(_)
                | PlanNodeEnum::GetNeighbors(_)
                | PlanNodeEnum::EdgeIndexScan(_)
                | PlanNodeEnum::IndexScan(_)
        )
    }

    /// 检查节点是否是中间节点（可以继续下推）
    fn is_intermediate_node(node: &PlanNodeEnum) -> bool {
        matches!(
            node,
            PlanNodeEnum::Filter(_) | PlanNodeEnum::Dedup(_) | PlanNodeEnum::Limit(_)
        )
    }

    /// 创建带有投影列的数据源节点
    fn create_data_source_with_projection(
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

    /// 检查节点是否包含数据源（通过中间节点）
    fn contains_data_source(&self, node: &PlanNodeEnum) -> bool {
        if Self::is_data_source(node) {
            return true;
        }

        if Self::is_intermediate_node(node) {
            // 对于中间节点，继续向下查找
            if let Some(input) = node.dependencies().first() {
                return self.contains_data_source(input);
            }
        }

        false
    }
}

impl Default for PushProjectDownRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for PushProjectDownRule {
    fn name(&self) -> &'static str {
        "PushProjectDownRule"
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

        // 如果直接子节点就是数据源，直接下推
        if Self::is_data_source(input) {
            let columns = project_node.columns();
            let new_node = match self.create_data_source_with_projection(input, columns) {
                Some(node) => node,
                None => return Ok(None),
            };

            let mut result = TransformResult::new();
            result.erase_curr = true;
            result.add_new_node(new_node);
            return Ok(Some(result));
        }

        // 如果子节点是中间节点，尝试继续向下查找数据源
        if Self::is_intermediate_node(input) {
            if self.contains_data_source(input) {
                // 找到了数据源，但当前实现简化处理
                // 实际应该重构整个子树，将投影下推到数据源
                // 这里仅作演示，返回 None 表示不进行转换
                return Ok(None);
            }
        }

        Ok(None)
    }
}

impl PushDownRule for PushProjectDownRule {
    fn can_push_down(&self, node: &PlanNodeEnum, target: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Project(project) => {
                if project.columns().is_empty() {
                    return false;
                }
                Self::is_data_source(target)
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
        FilterNode, GetVerticesNode, ProjectNode, ScanVerticesNode, StartNode,
    };

    #[test]
    fn test_rule_name() {
        let rule = PushProjectDownRule::new();
        assert_eq!(rule.name(), "PushProjectDownRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = PushProjectDownRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_is_data_source() {
        let scan = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        assert!(PushProjectDownRule::is_data_source(&scan));

        let start = PlanNodeEnum::Start(StartNode::new());
        assert!(!PushProjectDownRule::is_data_source(&start));
    }

    #[test]
    fn test_is_intermediate_node() {
        let scan = ScanVerticesNode::new(1);
        let filter = FilterNode::new(
            PlanNodeEnum::ScanVertices(scan.clone()),
            crate::core::Expression::literal(true),
        )
        .expect("创建 FilterNode 失败");
        assert!(PushProjectDownRule::is_intermediate_node(&PlanNodeEnum::Filter(filter)));

        assert!(!PushProjectDownRule::is_intermediate_node(&PlanNodeEnum::ScanVertices(scan)));
    }

    #[test]
    fn test_apply_with_direct_data_source() {
        let rule = PushProjectDownRule::new();
        let mut ctx = RewriteContext::new();

        // 创建 ScanVertices 节点
        let scan_node = ScanVerticesNode::new(1);
        let scan = PlanNodeEnum::ScanVertices(scan_node);

        // 创建 Project 节点
        let columns = vec![
            YieldColumn {
                expression: Expression::Variable("id".to_string()),
                alias: "id".to_string(),
                is_matched: false,
            },
            YieldColumn {
                expression: Expression::Variable("name".to_string()),
                alias: "name".to_string(),
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
                assert_eq!(node.col_names(), &["id", "name"]);
            }
            _ => panic!("期望 ScanVertices 节点"),
        }
    }

    #[test]
    fn test_apply_with_get_vertices() {
        let rule = PushProjectDownRule::new();
        let mut ctx = RewriteContext::new();

        // 创建 GetVertices 节点
        let get_vertices = GetVerticesNode::new(1, "vids");
        let get_vertices_enum = PlanNodeEnum::GetVertices(get_vertices);

        // 创建 Project 节点
        let columns = vec![YieldColumn {
            expression: Expression::Variable("vertex".to_string()),
            alias: "vertex".to_string(),
            is_matched: false,
        }];
        let project =
            ProjectNode::new(get_vertices_enum.clone(), columns).expect("创建 ProjectNode 失败");
        let project_enum = PlanNodeEnum::Project(project);

        // 应用规则
        let result = rule.apply(&mut ctx, &project_enum).expect("应用规则失败");

        assert!(result.is_some());
        let transform = result.unwrap();
        assert!(transform.erase_curr);

        // 验证新节点是 GetVertices
        match &transform.new_nodes[0] {
            PlanNodeEnum::GetVertices(node) => {
                assert_eq!(node.col_names(), &["vertex"]);
            }
            _ => panic!("期望 GetVertices 节点"),
        }
    }

    #[test]
    fn test_apply_with_non_data_source() {
        let rule = PushProjectDownRule::new();
        let mut ctx = RewriteContext::new();

        // 创建 Start 节点（不是数据源）
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
        let rule = PushProjectDownRule::new();

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

    #[test]
    fn test_contains_data_source() {
        let rule = PushProjectDownRule::new();

        // 直接数据源
        let scan = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        assert!(rule.contains_data_source(&scan));

        // 中间节点 -> 数据源
        let filter = FilterNode::new(
            scan.clone(),
            crate::core::Expression::literal(true),
        )
        .expect("创建 FilterNode 失败");
        let filter_enum = PlanNodeEnum::Filter(filter);
        assert!(rule.contains_data_source(&filter_enum));

        // 非数据源
        let start = PlanNodeEnum::Start(StartNode::new());
        assert!(!rule.contains_data_source(&start));
    }
}
