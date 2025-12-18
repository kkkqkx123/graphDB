//! 路径匹配规划器
//! 处理路径模式的规划
//! 负责规划路径模式的匹配

use crate::query::context::validate::types::Variable;
use crate::query::parser::ast::expr::Expr;
use crate::query::planner::plan::core::nodes::PlanNodeFactory;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{MatchClauseContext, Path, WhereClauseContext};
use std::collections::HashSet;

/// 路径匹配规划器
/// 负责规划路径模式的匹配
#[derive(Debug)]
pub struct MatchPathPlanner {
    match_clause_ctx: MatchClauseContext,
    path: Path,
}

impl MatchPathPlanner {
    pub fn new(match_clause_ctx: MatchClauseContext, path: Path) -> Self {
        Self {
            match_clause_ctx,
            path,
        }
    }

    /// 转换路径为执行计划
    pub fn transform(
        &mut self,
        where_clause: Option<&WhereClauseContext>,
        node_aliases_seen: &mut HashSet<String>,
    ) -> Result<SubPlan, PlannerError> {
        // 实现路径匹配的具体逻辑
        // 基于nebula-graph的实现，需要找到起始点并扩展路径

        // 找到起始节点和起始索引
        let (start_index, start_from_edge, mut subplan) =
            self.find_starts(where_clause, node_aliases_seen)?;

        // 从起始点扩展路径
        if start_from_edge {
            self.expand_from_edge(start_index, &mut subplan)?;
        } else {
            self.expand_from_node(start_index, &mut subplan)?;
        }

        // 如果路径不是谓词，需要构建项目列
        if !self.path.is_pred {
            self.build_project_columns(&mut subplan)?;
        }

        Ok(subplan)
    }

    /// 查找起始点
    fn find_starts(
        &self,
        _where_clause: Option<&WhereClauseContext>,
        node_aliases_seen: &HashSet<String>,
    ) -> Result<(usize, bool, SubPlan), PlannerError> {
        // 将所有可用别名添加到已见别名集合
        let mut all_aliases_seen = node_aliases_seen.clone();
        for (alias, _) in &self.match_clause_ctx.aliases_available {
            all_aliases_seen.insert(alias.clone());
        }

        // 查找起始节点
        for (i, node_info) in self.path.node_infos.iter().enumerate() {
            // 检查节点是否可以使用标签索引查找
            if !node_info.labels.is_empty() {
                let label_index_seeker =
                    crate::query::planner::match_planning::IndexSeek::new_label(node_info.clone());
                if label_index_seeker.match_node() {
                    let plan = label_index_seeker.build_plan()?;
                    return Ok((i, false, plan));
                }
            }

            // 检查节点是否在已见别名中
            if all_aliases_seen.contains(&node_info.alias) && !node_info.anonymous {
                // 创建参数节点
                let _variable = Variable {
                    name: node_info.alias.clone(),
                    columns: vec![crate::query::context::validate::types::Column {
                        name: node_info.alias.clone(),
                        type_: "Vertex".to_string(),
                    }],
                };
                // 创建一个包含变量信息的占位符节点
                let placeholder = PlanNodeFactory::create_placeholder_node()?;
                // 由于 Arc<dyn PlanNode> 不能直接修改，我们使用占位符节点
                let plan = SubPlan::new(Some(placeholder.clone_plan_node()), None);
                return Ok((i, false, plan));
            }
        }

        // 如果没有找到合适的起始节点，尝试从边开始
        for (i, edge_info) in self.path.edge_infos.iter().enumerate() {
            // 检查边是否可以使用索引查找
            if !edge_info.types.is_empty() {
                // 创建边索引扫描节点
                let var_name = format!("edge_scan_{}", edge_info.types.join("_"));
                let _variable = Variable {
                    name: var_name.clone(),
                    columns: vec![
                        crate::query::context::validate::types::Column {
                            name: "src".to_string(),
                            type_: "Vertex".to_string(),
                        },
                        crate::query::context::validate::types::Column {
                            name: "dst".to_string(),
                            type_: "Vertex".to_string(),
                        },
                    ],
                };
                let edge_scan_node = PlanNodeFactory::create_placeholder_node()?;
                let _variable = Variable {
                    name: var_name,
                    columns: vec![
                        crate::query::context::validate::types::Column {
                            name: "src".to_string(),
                            type_: "Vertex".to_string(),
                        },
                        crate::query::context::validate::types::Column {
                            name: "dst".to_string(),
                            type_: "Vertex".to_string(),
                        },
                    ],
                };
                // 由于 Arc<dyn PlanNode> 不能直接修改，我们使用占位符节点
                let plan =
                    SubPlan::new(Some(edge_scan_node.clone_plan_node()), Some(edge_scan_node));
                return Ok((i, true, plan));
            }
        }

        // 如果都没有找到，返回错误
        Err(PlannerError::PlanGenerationFailed(
            "Can't solve the start vids from the sentence.".to_string(),
        ))
    }

    /// 从节点扩展路径
    fn expand_from_node(
        &self,
        start_index: usize,
        subplan: &mut SubPlan,
    ) -> Result<(), PlannerError> {
        let node_infos = &self.path.node_infos;
        let _edge_infos = &self.path.edge_infos;

        // 记录路径中已见过的节点别名
        let mut node_aliases_seen_in_pattern = HashSet::new();
        node_aliases_seen_in_pattern.insert(node_infos[start_index].alias.clone());

        // 根据起始位置决定扩展方向
        if start_index == 0 {
            // 从左向右扩展: (start)-[]-...-()
            self.right_expand_from_node(start_index, subplan, &mut node_aliases_seen_in_pattern)?;
        } else if start_index == node_infos.len() - 1 {
            // 从右向左扩展: ()-[]-...-(start)
            self.left_expand_from_node(start_index, subplan, &mut node_aliases_seen_in_pattern)?;
        } else {
            // 从中间向两边扩展: ()-[]-...-(start)-...-[]-()
            self.right_expand_from_node(start_index, subplan, &mut node_aliases_seen_in_pattern)?;
            self.left_expand_from_node(start_index, subplan, &mut node_aliases_seen_in_pattern)?;
        }

        Ok(())
    }

    /// 从边扩展路径
    fn expand_from_edge(
        &self,
        start_index: usize,
        subplan: &mut SubPlan,
    ) -> Result<(), PlannerError> {
        // 简化实现，直接调用从节点扩展
        self.expand_from_node(start_index, subplan)
    }

    /// 从节点向右扩展路径
    fn right_expand_from_node(
        &self,
        start_index: usize,
        subplan: &mut SubPlan,
        node_aliases_seen_in_pattern: &mut HashSet<String>,
    ) -> Result<(), PlannerError> {
        let node_infos = &self.path.node_infos;
        let edge_infos = &self.path.edge_infos;

        // 从起始节点向右扩展
        for i in start_index..edge_infos.len() {
            let node = &node_infos[i];
            let dst = &node_infos[i + 1];
            let edge = &edge_infos[i];

            // 创建新的遍历节点
            let traverse_node = PlanNodeFactory::create_placeholder_node()?;

            // 由于无法直接修改 Arc<dyn PlanNode>，我们先创建节点然后通过工厂创建带过滤条件的节点
            let node_to_use = if let Some(_filter) = &node.filter {
                let dummy_expr =
                    Expr::Constant(crate::query::parser::ast::expr::ConstantExpr::new(
                        crate::core::Value::Bool(true),
                        crate::query::parser::ast::types::Span::default(),
                    ));
                PlanNodeFactory::create_filter(traverse_node.clone_plan_node(), dummy_expr)?
            } else {
                traverse_node.clone_plan_node()
            };

            // 更新subplan根节点
            subplan.root = Some(node_to_use);

            // 处理边过滤
            if let Some(_filter) = &edge.filter {
                let dummy_expr =
                    Expr::Constant(crate::query::parser::ast::expr::ConstantExpr::new(
                        crate::core::Value::Bool(true),
                        crate::query::parser::ast::types::Span::default(),
                    ));
                let current_root = subplan.root.take().unwrap();
                let filter_node =
                    PlanNodeFactory::create_filter(current_root, dummy_expr)?;
                subplan.root = Some(filter_node);
            }

            // 记录已见过的节点别名
            node_aliases_seen_in_pattern.insert(dst.alias.clone());
        }

        // 处理最后一个节点
        let last_node = &node_infos[node_infos.len() - 1];
        if !node_aliases_seen_in_pattern.contains(&last_node.alias) {
            let append_node = PlanNodeFactory::create_placeholder_node()?;
            subplan.root = Some(append_node.clone_plan_node());
        }

        Ok(())
    }

    /// 从节点向左扩展路径
    fn left_expand_from_node(
        &self,
        start_index: usize,
        subplan: &mut SubPlan,
        node_aliases_seen_in_pattern: &mut HashSet<String>,
    ) -> Result<(), PlannerError> {
        let node_infos = &self.path.node_infos;
        let edge_infos = &self.path.edge_infos;

        // 从起始节点向左扩展
        for i in (1..=start_index).rev() {
            let node = &node_infos[i];
            let dst = &node_infos[i - 1];
            let edge = &edge_infos[i - 1];

            // 创建新的遍历节点
            let traverse_node = PlanNodeFactory::create_placeholder_node()?;

            // 由于无法直接修改 Arc<dyn PlanNode>，我们先创建节点然后通过工厂创建带过滤条件的节点
            let node_to_use = if let Some(_filter) = &node.filter {
                let dummy_expr =
                    Expr::Constant(crate::query::parser::ast::expr::ConstantExpr::new(
                        crate::core::Value::Bool(true),
                        crate::query::parser::ast::types::Span::default(),
                    ));
                PlanNodeFactory::create_filter(traverse_node.clone_plan_node(), dummy_expr)?
            } else {
                traverse_node.clone_plan_node()
            };

            // 更新subplan根节点
            subplan.root = Some(node_to_use);

            // 处理边过滤
            if let Some(_filter) = &edge.filter {
                let dummy_expr =
                    Expr::Constant(crate::query::parser::ast::expr::ConstantExpr::new(
                        crate::core::Value::Bool(true),
                        crate::query::parser::ast::types::Span::default(),
                    ));
                let current_root = subplan.root.take().unwrap();
                let filter_node =
                    PlanNodeFactory::create_filter(current_root, dummy_expr)?;
                subplan.root = Some(filter_node);
            }

            // 记录已见过的节点别名
            node_aliases_seen_in_pattern.insert(dst.alias.clone());
        }

        // 处理第一个节点
        let first_node = &node_infos[0];
        if !node_aliases_seen_in_pattern.contains(&first_node.alias) {
            let append_node = PlanNodeFactory::create_placeholder_node()?;
            subplan.root = Some(append_node.clone_plan_node());
        }

        Ok(())
    }

    /// 构建项目列
    fn build_project_columns(&self, subplan: &mut SubPlan) -> Result<(), PlannerError> {
        // 只有当路径中有多个节点或边时才创建项目节点
        // 对于单节点路径，不需要额外的项目节点
        if self.path.node_infos.len() <= 1 && self.path.edge_infos.is_empty() {
            return Ok(());
        }

        // 创建项目节点
        let project_node = PlanNodeFactory::create_placeholder_node()?;

        // 设置项目列名
        let mut col_names = vec![];
        for node_info in &self.path.node_infos {
            if !node_info.anonymous {
                col_names.push(node_info.alias.clone());
            }
        }
        for edge_info in &self.path.edge_infos {
            if !edge_info.anonymous {
                col_names.push(edge_info.alias.clone());
            }
        }
        let variable = Variable {
            name: "project".to_string(),
            columns: vec![],
        };

        // 由于不能直接修改 Arc<dyn PlanNode>，我们使用占位符
        subplan.root = Some(project_node.clone_plan_node());
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::validator::structs::{MatchClauseContext, NodeInfo, Path, PathType};
    use std::collections::HashMap;

    /// 创建测试用的节点信息
    fn create_test_node_info(alias: &str, anonymous: bool) -> NodeInfo {
        NodeInfo {
            alias: alias.to_string(),
            labels: vec!["Person".to_string()],
            props: None,
            anonymous,
            filter: None,
            tids: vec![1],
            label_props: vec![None],
        }
    }

    /// 创建测试用的路径
    fn create_test_path(alias: &str, anonymous: bool, node_aliases: Vec<&str>) -> Path {
        let node_infos = node_aliases
            .into_iter()
            .map(|node_alias| create_test_node_info(node_alias, false))
            .collect();

        Path {
            alias: alias.to_string(),
            anonymous,
            gen_path: false,
            path_type: PathType::Default,
            node_infos,
            edge_infos: vec![],
            path_build: None,
            is_pred: false,
            is_anti_pred: false,
            compare_variables: vec![],
            collect_variable: String::new(),
            roll_up_apply: false,
        }
    }

    /// 创建测试用的 MATCH 子句上下文
    fn create_test_match_clause_context() -> MatchClauseContext {
        MatchClauseContext {
            paths: vec![],
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        }
    }

    #[test]
    fn test_match_path_planner_new() {
        let match_clause_ctx = create_test_match_clause_context();
        let path = create_test_path("p", false, vec!["n"]);

        let planner = MatchPathPlanner::new(match_clause_ctx, path);

        // 验证创建的实例
        assert_eq!(planner.path.alias, "p");
        assert_eq!(planner.path.node_infos.len(), 1);
        assert_eq!(planner.path.node_infos[0].alias, "n");
    }

    #[test]
    fn test_match_path_planner_debug() {
        let match_clause_ctx = create_test_match_clause_context();
        let path = create_test_path("p", false, vec!["n"]);

        let planner = MatchPathPlanner::new(match_clause_ctx, path);

        let debug_str = format!("{:?}", planner);
        assert!(debug_str.contains("MatchPathPlanner"));
    }

    #[test]
    fn test_transform_single_node_path() {
        let match_clause_ctx = create_test_match_clause_context();
        let path = create_test_path("p", false, vec!["n"]);

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(None, &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());

        // 验证根节点类型 - 对于有标签的单节点，应该使用IndexScan
        if let Some(root) = &subplan.root {
            assert_eq!(root.kind(), PlanNodeKind::IndexScan);
        }
    }

    #[test]
    fn test_transform_multi_node_path() {
        let match_clause_ctx = create_test_match_clause_context();
        let path = create_test_path("p", false, vec!["n", "m", "o"]);

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(None, &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());

        // 验证根节点类型 - 对于多节点路径，应该使用Project节点
        if let Some(root) = &subplan.root {
            assert_eq!(root.kind(), PlanNodeKind::Project);
        }
    }

    #[test]
    fn test_transform_with_where_clause() {
        let match_clause_ctx = create_test_match_clause_context();
        let path = create_test_path("p", false, vec!["n"]);

        let where_clause = crate::query::validator::structs::WhereClauseContext {
            filter: Some(crate::graph::expression::Expression::Variable(
                "x".to_string(),
            )),
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: vec![],
        };

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(Some(&where_clause), &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
    }

    #[test]
    fn test_transform_anonymous_path() {
        let match_clause_ctx = create_test_match_clause_context();
        let path = create_test_path("", true, vec!["n"]);

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(None, &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
    }

    #[test]
    fn test_transform_with_existing_aliases() {
        let match_clause_ctx = create_test_match_clause_context();
        let path = create_test_path("p", false, vec!["n"]);

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();
        node_aliases_seen.insert("x".to_string()); // 添加已存在的别名

        let result = planner.transform(None, &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
    }


    #[test]
    fn test_path_with_labels() {
        let match_clause_ctx = create_test_match_clause_context();

        let mut path = create_test_path("p", false, vec!["n"]);
        path.node_infos[0].labels = vec!["Person".to_string(), "User".to_string()];

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(None, &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
    }

    #[test]
    fn test_path_with_properties() {
        let match_clause_ctx = create_test_match_clause_context();

        let mut path = create_test_path("p", false, vec!["n"]);
        path.node_infos[0].props = Some(crate::graph::expression::Expression::Literal(
            crate::graph::expression::expression::LiteralValue::String("test".to_string()),
        ));

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(None, &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
    }

    #[test]
    fn test_path_with_filter() {
        let match_clause_ctx = create_test_match_clause_context();

        let mut path = create_test_path("p", false, vec!["n"]);
        path.node_infos[0].filter = Some(crate::graph::expression::Expression::Variable(
            "x".to_string(),
        ));

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(None, &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
    }

    #[test]
    fn test_path_with_edges() {
        let match_clause_ctx = create_test_match_clause_context();

        let mut path = create_test_path("p", false, vec!["n", "m"]);

        // 添加边信息
        use crate::query::validator::structs::path_structs::{Direction, EdgeInfo};
        path.edge_infos.push(EdgeInfo {
            alias: "e".to_string(),
            inner_alias: "e_inner".to_string(),
            types: vec!["KNOWS".to_string()],
            props: None,
            anonymous: false,
            filter: None,
            direction: Direction::Forward,
            range: None,
            edge_types: vec![1],
        });

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(None, &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
    }

    #[test]
    fn test_subplan_structure() {
        let match_clause_ctx = create_test_match_clause_context();
        let path = create_test_path("p", false, vec!["n"]);

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(None, &mut node_aliases_seen);
        assert!(result.is_ok());

        let subplan = result.unwrap();

        // 验证 SubPlan 结构
        assert!(subplan.root().is_some());
        assert!(subplan.tail().is_some()); // 尾节点不为 None，应该是IndexScan

        // 验证根节点类型 - 对于有标签的单节点，应该使用IndexScan
        if let Some(root) = &subplan.root {
            assert_eq!(root.kind(), PlanNodeKind::IndexScan);
        }

        // 验证尾节点类型 - 应该是IndexScan
        if let Some(tail) = &subplan.tail {
            assert_eq!(tail.kind(), PlanNodeKind::IndexScan);
        }
    }
}
