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

    /// 递归查找数据源节点
    fn find_data_source(&self, node: &PlanNodeEnum) -> Option<PlanNodeEnum> {
        if Self::is_data_source(node) {
            return Some(node.clone());
        }

        if Self::is_intermediate_node(node) {
            if let Some(input) = node.dependencies().first() {
                return self.find_data_source(input);
            }
        }

        None
    }

    /// 通过中间节点链下推投影
    ///
    /// 这个方法处理以下场景：
    /// ```
    /// Project(col1, col2)
    ///       |
    ///    Filter(condition)
    ///       |
    ///   ScanVertices
    /// ```
    ///
    /// 转换为：
    /// ```
    /// Filter(condition)
    ///       |
    ///   ScanVertices(col1, col2)
    /// ```
    fn push_down_through_intermediate_nodes(
        &self,
        project_node: &crate::query::planner::plan::core::nodes::ProjectNode,
        intermediate_node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        let columns = project_node.columns();

        // 递归查找数据源节点
        let data_source = match self.find_data_source(intermediate_node) {
            Some(source) => source,
            None => return Ok(None),
        };

        // 创建带有投影列的新数据源节点
        let new_data_source = match self.create_data_source_with_projection(&data_source, columns) {
            Some(node) => node,
            None => return Ok(None),
        };

        // 重建中间节点链
        let mut current_node = new_data_source;

        // 从数据源向上重建节点链
        let node_chain = self.collect_intermediate_nodes(intermediate_node, &data_source)?;

        // 从下往上重建节点链
        for node in node_chain.into_iter().rev() {
            current_node = self.rebuild_node_with_new_input(node, current_node)?;
        }

        // 创建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(current_node);

        Ok(Some(result))
    }

    /// 收集从中间节点到数据源之间的所有中间节点
    fn collect_intermediate_nodes(
        &self,
        start: &PlanNodeEnum,
        end: &PlanNodeEnum,
    ) -> RewriteResult<Vec<PlanNodeEnum>> {
        let mut nodes = Vec::new();
        let mut current = start.clone();

        // 使用 ID 来判断是否到达数据源节点
        let end_id = end.id();

        while Self::is_intermediate_node(&current) {
            if current.id() == end_id {
                break;
            }

            nodes.push(current.clone());

            let deps = current.dependencies();
            if let Some(input) = deps.first() {
                current = (**input).clone();
            } else {
                break;
            }
        }

        Ok(nodes)
    }

    /// 使用新的输入节点重建节点
    fn rebuild_node_with_new_input(
        &self,
        mut node: PlanNodeEnum,
        new_input: PlanNodeEnum,
    ) -> RewriteResult<PlanNodeEnum> {
        match &mut node {
            PlanNodeEnum::Filter(n) => {
                n.set_input(new_input);
                Ok(node)
            }
            PlanNodeEnum::Dedup(n) => {
                n.set_input(new_input);
                Ok(node)
            }
            PlanNodeEnum::Limit(n) => {
                n.set_input(new_input);
                Ok(node)
            }
            _ => Ok(node),
        }
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
                // 找到了数据源，通过中间节点链下推投影
                return self.push_down_through_intermediate_nodes(project_node, input);
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
    use crate::core::types::ExpressionContext;
    use crate::query::planner::plan::core::nodes::{
        DedupNode, FilterNode, GetVerticesNode, LimitNode, ProjectNode, ScanVerticesNode, StartNode,
    };
    use std::sync::Arc;

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
        let ctx = Arc::new(ExpressionContext::new());
        let filter = FilterNode::from_expression(
            PlanNodeEnum::ScanVertices(scan.clone()),
            crate::core::Expression::literal(true),
            ctx,
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
        let transform = result.expect("Failed to apply rewrite rule");
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
        let transform = result.expect("Failed to apply rewrite rule");
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
        let ctx = Arc::new(ExpressionContext::new());

        // 直接数据源
        let scan = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        assert!(rule.contains_data_source(&scan));

        // 中间节点 -> 数据源
        let filter = FilterNode::from_expression(
            scan.clone(),
            crate::core::Expression::literal(true),
            ctx.clone(),
        )
        .expect("创建 FilterNode 失败");
        let filter_enum = PlanNodeEnum::Filter(filter);
        assert!(rule.contains_data_source(&filter_enum));

        // 非数据源
        let start = PlanNodeEnum::Start(StartNode::new());
        assert!(!rule.contains_data_source(&start));
    }

    #[test]
    fn test_find_data_source() {
        let rule = PushProjectDownRule::new();
        let ctx = Arc::new(ExpressionContext::new());

        // 直接数据源
        let scan = PlanNodeEnum::ScanVertices(ScanVerticesNode::new(1));
        assert!(rule.find_data_source(&scan).is_some());

        // 中间节点 -> 数据源
        let filter = FilterNode::from_expression(
            scan.clone(),
            crate::core::Expression::literal(true),
            ctx.clone(),
        )
        .expect("创建 FilterNode 失败");
        let filter_enum = PlanNodeEnum::Filter(filter);
        let found = rule.find_data_source(&filter_enum);
        assert!(found.is_some());
        assert!(matches!(found.unwrap(), PlanNodeEnum::ScanVertices(_)));

        // 非数据源
        let start = PlanNodeEnum::Start(StartNode::new());
        assert!(rule.find_data_source(&start).is_none());
    }

    #[test]
    fn test_push_down_through_filter() {
        let rule = PushProjectDownRule::new();
        let mut rewrite_ctx = RewriteContext::new();

        // 创建 ScanVertices 节点
        let scan_node = ScanVerticesNode::new(1);
        let scan = PlanNodeEnum::ScanVertices(scan_node);

        // 创建 Filter 节点
        use std::sync::Arc;
        let ctx = Arc::new(crate::core::types::ExpressionContext::new());
        let filter = FilterNode::from_expression(
            scan.clone(),
            crate::core::Expression::literal(true),
            ctx,
        )
        .expect("创建 FilterNode 失败");
        let filter_enum = PlanNodeEnum::Filter(filter);

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
        let project =
            ProjectNode::new(filter_enum.clone(), columns).expect("创建 ProjectNode 失败");
        let project_enum = PlanNodeEnum::Project(project);

        // 应用规则
        let result = rule.apply(&mut rewrite_ctx, &project_enum).expect("应用规则失败");

        assert!(result.is_some());
        let transform = result.expect("Failed to apply rewrite rule");
        assert!(transform.erase_curr);
        assert_eq!(transform.new_nodes.len(), 1);

        // 验证新节点是 Filter，其输入是带有投影列的 ScanVertices
        match &transform.new_nodes[0] {
            PlanNodeEnum::Filter(filter_node) => {
                match filter_node.input() {
                    PlanNodeEnum::ScanVertices(scan_node) => {
                        assert_eq!(scan_node.col_names(), &["id", "name"]);
                    }
                    _ => panic!("期望 Filter 的输入是 ScanVertices"),
                }
            }
            _ => panic!("期望 Filter 节点"),
        }
    }

    #[test]
    fn test_push_down_through_dedup() {
        let rule = PushProjectDownRule::new();
        let mut ctx = RewriteContext::new();

        // 创建 ScanVertices 节点
        let scan_node = ScanVerticesNode::new(1);
        let scan = PlanNodeEnum::ScanVertices(scan_node);

        // 创建 Dedup 节点
        let dedup = DedupNode::new(scan).expect("创建 DedupNode 失败");
        let dedup_enum = PlanNodeEnum::Dedup(dedup);

        // 创建 Project 节点
        let columns = vec![YieldColumn {
            expression: Expression::Variable("id".to_string()),
            alias: "id".to_string(),
            is_matched: false,
        }];
        let project =
            ProjectNode::new(dedup_enum.clone(), columns).expect("创建 ProjectNode 失败");
        let project_enum = PlanNodeEnum::Project(project);

        // 应用规则
        let result = rule.apply(&mut ctx, &project_enum).expect("应用规则失败");

        assert!(result.is_some());
        let transform = result.expect("Failed to apply rewrite rule");
        assert!(transform.erase_curr);

        // 验证新节点是 Dedup，其输入是带有投影列的 ScanVertices
        match &transform.new_nodes[0] {
            PlanNodeEnum::Dedup(dedup_node) => {
                match dedup_node.input() {
                    PlanNodeEnum::ScanVertices(scan_node) => {
                        assert_eq!(scan_node.col_names(), &["id"]);
                    }
                    _ => panic!("期望 Dedup 的输入是 ScanVertices"),
                }
            }
            _ => panic!("期望 Dedup 节点"),
        }
    }

    #[test]
    fn test_push_down_through_limit() {
        let rule = PushProjectDownRule::new();
        let mut ctx = RewriteContext::new();

        // 创建 ScanVertices 节点
        let scan_node = ScanVerticesNode::new(1);
        let scan = PlanNodeEnum::ScanVertices(scan_node);

        // 创建 Limit 节点
        let limit = LimitNode::new(scan, 0, 10).expect("创建 LimitNode 失败");
        let limit_enum = PlanNodeEnum::Limit(limit);

        // 创建 Project 节点
        let columns = vec![YieldColumn {
            expression: Expression::Variable("id".to_string()),
            alias: "id".to_string(),
            is_matched: false,
        }];
        let project =
            ProjectNode::new(limit_enum.clone(), columns).expect("创建 ProjectNode 失败");
        let project_enum = PlanNodeEnum::Project(project);

        // 应用规则
        let result = rule.apply(&mut ctx, &project_enum).expect("应用规则失败");

        assert!(result.is_some());
        let transform = result.expect("Failed to apply rewrite rule");
        assert!(transform.erase_curr);

        // 验证新节点是 Limit，其输入是带有投影列的 ScanVertices
        match &transform.new_nodes[0] {
            PlanNodeEnum::Limit(limit_node) => {
                match limit_node.input() {
                    PlanNodeEnum::ScanVertices(scan_node) => {
                        assert_eq!(scan_node.col_names(), &["id"]);
                    }
                    _ => panic!("期望 Limit 的输入是 ScanVertices"),
                }
            }
            _ => panic!("期望 Limit 节点"),
        }
    }

    #[test]
    fn test_push_down_through_multiple_intermediate_nodes() {
        let rule = PushProjectDownRule::new();
        let mut ctx = RewriteContext::new();
        let expr_ctx = Arc::new(ExpressionContext::new());

        // 创建 ScanVertices 节点
        let scan_node = ScanVerticesNode::new(1);
        let scan = PlanNodeEnum::ScanVertices(scan_node);

        // 创建 Filter 节点
        let filter = FilterNode::from_expression(
            scan.clone(),
            crate::core::Expression::literal(true),
            expr_ctx.clone(),
        )
        .expect("创建 FilterNode 失败");
        let filter_enum = PlanNodeEnum::Filter(filter);

        // 创建 Limit 节点
        let limit = LimitNode::new(filter_enum, 0, 10).expect("创建 LimitNode 失败");
        let limit_enum = PlanNodeEnum::Limit(limit);

        // 创建 Project 节点
        let columns = vec![YieldColumn {
            expression: Expression::Variable("id".to_string()),
            alias: "id".to_string(),
            is_matched: false,
        }];
        let project =
            ProjectNode::new(limit_enum.clone(), columns).expect("创建 ProjectNode 失败");
        let project_enum = PlanNodeEnum::Project(project);

        // 应用规则
        let result = rule.apply(&mut ctx, &project_enum).expect("应用规则失败");

        assert!(result.is_some());
        let transform = result.expect("Failed to apply rewrite rule");
        assert!(transform.erase_curr);

        // 验证新节点是 Limit，其输入是 Filter，Filter 的输入是带有投影列的 ScanVertices
        match &transform.new_nodes[0] {
            PlanNodeEnum::Limit(limit_node) => {
                match limit_node.input() {
                    PlanNodeEnum::Filter(filter_node) => {
                        match filter_node.input() {
                            PlanNodeEnum::ScanVertices(scan_node) => {
                                assert_eq!(scan_node.col_names(), &["id"]);
                            }
                            _ => panic!("期望 Filter 的输入是 ScanVertices"),
                        }
                    }
                    _ => panic!("期望 Limit 的输入是 Filter"),
                }
            }
            _ => panic!("期望 Limit 节点"),
        }
    }
}
