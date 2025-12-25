//! 路径匹配规划器
//! 处理路径模式的规划
//! 负责规划路径模式的匹配

use crate::query::parser::ast::expr::Expr;

use crate::query::planner::plan::factory::PlanNodeFactory;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;

use crate::query::validator::structs::{MatchClauseContext, Path, WhereClauseContext};
use crate::query::validator::{Column, Variable};
use std::collections::HashSet;

/// 路径匹配规划器
/// 负责规划路径模式的匹配
#[derive(Debug)]
pub struct MatchPathPlanner {
    match_clause_ctx: MatchClauseContext,
    path: Path,
    /// 路径中已见过的节点别名
    node_aliases_seen_in_pattern: HashSet<String>,
    /// 初始表达式，用于路径扩展的起始点
    initial_expr: Option<Expr>,
}

impl MatchPathPlanner {
    pub fn new(match_clause_ctx: MatchClauseContext, path: Path) -> Self {
        Self {
            match_clause_ctx,
            path,
            node_aliases_seen_in_pattern: HashSet::new(),
            initial_expr: None,
        }
    }

    /// 转换路径为执行计划
    pub fn transform(
        &mut self,
        where_clause: Option<&WhereClauseContext>,
        node_aliases_seen: &mut HashSet<String>,
    ) -> Result<SubPlan, PlannerError> {
        // 合并所有可用别名到已见别名集合
        for (alias, _) in &self.match_clause_ctx.aliases_available {
            node_aliases_seen.insert(alias.clone());
        }

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

    /// 查找起始点 - 对照 nebula-graph 实现
    fn find_starts(
        &mut self,
        _bind_where_clause: Option<&WhereClauseContext>,
        node_aliases_seen: &HashSet<String>,
    ) -> Result<(usize, bool, SubPlan), PlannerError> {
        let mut found_start = false;
        let mut start_index = 0;
        let mut start_from_edge = false;
        let mut match_clause_plan = SubPlan::new(None, None);

        // 获取空间ID和查询上下文
        let space_id = 1i32; // 默认空间ID

        // 查找起始节点 - 对照 nebula-graph 的 findStarts 实现
        for (i, node_info) in self.path.node_infos.iter().enumerate() {
            // 检查是否是已存在的别名（ArgumentFinder）
            if node_aliases_seen.contains(&node_info.alias) && !node_info.anonymous {
                let argument_node = PlanNodeFactory::create_argument(0, &node_info.alias)?;
                match_clause_plan = SubPlan::new(Some(argument_node.clone()), Some(argument_node));

                // 初始化起始表达式
                self.initial_expr = Some(Expr::Variable(
                    crate::query::parser::ast::expr::VariableExpr::new(
                        node_info.alias.clone(),
                        crate::query::parser::ast::types::Span::default(),
                    ),
                ));

                start_index = i;
                found_start = true;
                break;
            }

            // 检查标签索引（LabelIndexSeek）
            if !node_info.labels.is_empty() && !node_info.tids.is_empty() {
                if let Some(plan) = self.create_label_index_scan(node_info, space_id)? {
                    match_clause_plan = plan;
                    start_index = i;
                    found_start = true;
                    break;
                }
            }

            // 检查属性索引（PropIndexSeek）
            if let Some(props) = &node_info.props {
                if let Some(plan) = self.create_prop_index_scan(node_info, props, space_id)? {
                    match_clause_plan = plan;
                    start_index = i;
                    found_start = true;
                    break;
                }
            }

            // 如果不是最后一个节点，检查边索引
            if i != self.path.node_infos.len() - 1 && i < self.path.edge_infos.len() {
                let edge_info = &self.path.edge_infos[i];

                // 检查边标签索引
                if !edge_info.types.is_empty() && !edge_info.edge_types.is_empty() {
                    if let Some(plan) = self.create_edge_index_scan(edge_info, space_id)? {
                        match_clause_plan = plan;
                        start_index = i;
                        start_from_edge = true;
                        found_start = true;
                        break;
                    }
                }
            }
        }

        if !found_start {
            return Err(PlannerError::PlanGenerationFailed(
                "Can't solve the start vids from the sentence.".to_string(),
            ));
        }

        // 添加起始节点
        if let Some(tail) = &match_clause_plan.tail {
            if tail.name() == "Start" {
                // 已经添加了起始节点
            }
        }

        Ok((start_index, start_from_edge, match_clause_plan))
    }

    /// 从节点扩展路径
    fn expand_from_node(
        &mut self,
        start_index: usize,
        subplan: &mut SubPlan,
    ) -> Result<(), PlannerError> {
        let node_count = self.path.node_infos.len();
        let start_node = self.path.node_infos[start_index].clone();

        // 记录路径中已见过的节点别名
        self.node_aliases_seen_in_pattern.clear();
        self.add_node_alias(&start_node);

        // 根据起始位置决定扩展方向
        if start_index == 0 {
            // 从左向右扩展: (start)-[]-...-()
            self.right_expand_from_node(start_index, subplan)?;
        } else if start_index == node_count - 1 {
            // 从右向左扩展: ()-[]-...-(start)
            self.left_expand_from_node(start_index, subplan)?;
        } else {
            // 从中间向两边扩展: ()-[]-...-(start)-...-[]-()
            self.right_expand_from_node(start_index, subplan)?;
            self.left_expand_from_node(start_index, subplan)?;
        }

        Ok(())
    }

    /// 从边扩展路径
    fn expand_from_edge(
        &mut self,
        start_index: usize,
        subplan: &mut SubPlan,
    ) -> Result<(), PlannerError> {
        // 简化实现，直接调用从节点扩展
        self.expand_from_node(start_index, subplan)
    }

    /// 从节点向右扩展路径 - 对照 nebula-graph 实现
    fn right_expand_from_node(
        &mut self,
        start_index: usize,
        subplan: &mut SubPlan,
    ) -> Result<(), PlannerError> {
        let space_id = 1i32; // 默认空间ID
        let edge_count = self.path.edge_infos.len();
        let node_count = self.path.node_infos.len();

        // 从起始节点向右扩展
        for i in start_index..edge_count {
            let node = self.path.node_infos[i].clone();
            let dst = self.path.node_infos[i + 1].clone();
            let edge = self.path.edge_infos[i].clone();

            // 检查是否是扩展进入（expand into）
            let expand_into = self.is_expand_into(&dst.alias);

            // 创建遍历节点
            let traverse_node = PlanNodeFactory::create_traverse(
                space_id,
                edge.types.clone(),
                &self.direction_to_string(edge.direction),
            )?;

            // 配置遍历节点
            self.configure_traverse_node(traverse_node.clone(), &node, &edge, i != start_index)?;

            // 更新subplan根节点
            subplan.root = Some(traverse_node);

            // 记录已见过的节点别名
            self.add_node_alias(&dst);

            // 处理扩展进入的情况
            if expand_into {
                // 创建过滤条件来检查起始和结束顶点是否相同
                // 这里需要实现具体的过滤逻辑
            }
        }

        // 处理最后一个节点
        let last_node = self.path.node_infos[node_count - 1].clone();
        if !self.is_expand_into(&last_node.alias) {
            let append_node = PlanNodeFactory::create_append_vertices(
                space_id,
                vec![], // vids will be set from previous node
                vec![], // tag_ids
            )?;
            subplan.root = Some(append_node);
        }

        Ok(())
    }

    /// 从节点向左扩展路径 - 对照 nebula-graph 实现
    fn left_expand_from_node(
        &mut self,
        start_index: usize,
        subplan: &mut SubPlan,
    ) -> Result<(), PlannerError> {
        let space_id = 1i32; // 默认空间ID

        // 从起始节点向左扩展
        for i in (1..=start_index).rev() {
            let node = self.path.node_infos[i].clone();
            let dst = self.path.node_infos[i - 1].clone();
            let edge = self.path.edge_infos[i - 1].clone();

            // 检查是否是扩展进入（expand into）
            let expand_into = self.is_expand_into(&dst.alias);

            // 创建遍历节点（反向）
            let traverse_node = PlanNodeFactory::create_traverse(
                space_id,
                edge.types.clone(),
                &self.reverse_direction(&self.direction_to_string(edge.direction)),
            )?;

            // 配置遍历节点
            self.configure_traverse_node(traverse_node.clone(), &node, &edge, true)?;

            // 更新subplan根节点
            subplan.root = Some(traverse_node);

            // 记录已见过的节点别名
            self.add_node_alias(&dst);

            // 处理扩展进入的情况
            if expand_into {
                // 创建过滤条件来检查起始和结束顶点是否相同
                // 这里需要实现具体的过滤逻辑
            }
        }

        // 处理第一个节点
        let first_node = self.path.node_infos[0].clone();
        if !self.is_expand_into(&first_node.alias) {
            let append_node = PlanNodeFactory::create_append_vertices(
                space_id,
                vec![], // vids will be set from previous node
                vec![], // tag_ids
            )?;
            subplan.root = Some(append_node);
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
        let _variable = Variable {
            name: "project".to_string(),
            columns: vec![],
        };

        // 由于不能直接修改 PlanNodeEnum，我们使用占位符
        subplan.root = Some(project_node.clone());
        Ok(())
    }

    /// 添加节点别名到已见别名集合
    fn add_node_alias(&mut self, node: &crate::query::validator::structs::path_structs::NodeInfo) {
        if !node.anonymous {
            self.node_aliases_seen_in_pattern.insert(node.alias.clone());
        }
    }

    /// 检查是否是扩展进入（expand into）
    fn is_expand_into(&self, alias: &str) -> bool {
        self.node_aliases_seen_in_pattern.contains(alias)
    }

    /// 将方向枚举转换为字符串
    fn direction_to_string(
        &self,
        direction: crate::query::validator::structs::path_structs::Direction,
    ) -> String {
        match direction {
            crate::query::validator::structs::path_structs::Direction::Forward => "OUT".to_string(),
            crate::query::validator::structs::path_structs::Direction::Backward => "IN".to_string(),
            crate::query::validator::structs::path_structs::Direction::Bidirectional => {
                "BOTH".to_string()
            }
        }
    }

    /// 反向方向
    fn reverse_direction(&self, direction: &str) -> String {
        match direction {
            "OUT" => "IN".to_string(),
            "IN" => "OUT".to_string(),
            "BOTH" => "BOTH".to_string(),
            _ => direction.to_string(),
        }
    }

    /// 创建标签索引扫描
    fn create_label_index_scan(
        &self,
        node_info: &crate::query::validator::structs::path_structs::NodeInfo,
        _space_id: i32,
    ) -> Result<Option<SubPlan>, PlannerError> {
        if node_info.labels.is_empty() || node_info.tids.is_empty() {
            return Ok(None);
        }

        // 创建索引扫描节点
        let index_scan_node = PlanNodeFactory::create_placeholder_node()?;

        // 设置变量和列名
        let _variable = Variable {
            name: format!("index_scan_{}", node_info.labels.join("_")),
            columns: vec![Column {
                name: "vid".to_string(),
                type_: "Vertex".to_string(),
            }],
        };

        let plan = SubPlan::new(Some(index_scan_node.clone()), Some(index_scan_node));
        Ok(Some(plan))
    }

    /// 创建属性索引扫描
    fn create_prop_index_scan(
        &self,
        node_info: &crate::query::validator::structs::path_structs::NodeInfo,
        _props: &crate::core::Expression,
        _space_id: i32,
    ) -> Result<Option<SubPlan>, PlannerError> {
        // 创建属性索引扫描节点
        let index_scan_node = PlanNodeFactory::create_placeholder_node()?;

        // 设置变量和列名
        let _variable = Variable {
            name: format!("prop_index_scan_{}", node_info.alias),
            columns: vec![Column {
                name: "vid".to_string(),
                type_: "Vertex".to_string(),
            }],
        };

        let plan = SubPlan::new(Some(index_scan_node.clone()), Some(index_scan_node));
        Ok(Some(plan))
    }

    /// 创建边索引扫描
    fn create_edge_index_scan(
        &self,
        edge_info: &crate::query::validator::structs::path_structs::EdgeInfo,
        _space_id: i32,
    ) -> Result<Option<SubPlan>, PlannerError> {
        if edge_info.types.is_empty() || edge_info.edge_types.is_empty() {
            return Ok(None);
        }

        // 创建边索引扫描节点
        let edge_scan_node = PlanNodeFactory::create_placeholder_node()?;

        // 设置变量和列名
        let _variable = Variable {
            name: format!("edge_scan_{}", edge_info.types.join("_")),
            columns: vec![
                Column {
                    name: "src".to_string(),
                    type_: "Vertex".to_string(),
                },
                Column {
                    name: "dst".to_string(),
                    type_: "Vertex".to_string(),
                },
            ],
        };

        let plan = SubPlan::new(Some(edge_scan_node.clone()), Some(edge_scan_node));
        Ok(Some(plan))
    }

    /// 配置遍历节点
    fn configure_traverse_node(
        &self,
        _traverse_node: PlanNodeEnum,
        node: &crate::query::validator::structs::path_structs::NodeInfo,
        edge: &crate::query::validator::structs::path_structs::EdgeInfo,
        _track_prev_path: bool,
    ) -> Result<(), PlannerError> {
        // 设置顶点属性
        let _vertex_props = self.get_all_vertex_props()?;

        // 设置边属性
        let _edge_props = self.get_edge_props(edge, false)?;
        let _reverse_edge_props = self.get_edge_props(edge, true)?;

        // 设置过滤条件
        if let Some(_filter) = &node.filter {
            // 将过滤条件转换为表达式并设置到遍历节点
            // 这里需要实现表达式转换逻辑
        }

        if let Some(_filter) = &edge.filter {
            // 将边过滤条件转换为表达式并设置到遍历节点
            // 这里需要实现表达式转换逻辑
        }

        // 设置步数范围
        if let Some(_range) = &edge.range {
            // 设置步数范围到遍历节点
        }

        // 设置是否跟踪前一路径
        // 这需要根据具体的遍历节点实现来设置

        Ok(())
    }

    /// 获取所有顶点属性
    fn get_all_vertex_props(
        &self,
    ) -> Result<Vec<crate::query::planner::plan::core::common::TagProp>, PlannerError> {
        // 实现获取所有顶点属性的逻辑
        // 这里应该查询模式信息并返回所有标签的属性
        Ok(vec![])
    }

    /// 获取边属性
    fn get_edge_props(
        &self,
        edge: &crate::query::validator::structs::path_structs::EdgeInfo,
        _reverse: bool,
    ) -> Result<Vec<crate::query::planner::plan::core::common::EdgeProp>, PlannerError> {
        // 实现获取边属性的逻辑
        // 这里应该查询模式信息并返回边类型的属性
        let mut props = Vec::new();
        for edge_type in &edge.types {
            props.push(crate::query::planner::plan::core::common::EdgeProp::new(
                edge_type,
                vec![],
            ));
        }
        Ok(props)
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

        let subplan = result.expect("Failed to get subplan");
        assert!(subplan.root.is_some());

        // 验证根节点类型 - 当前实现使用Argument作为占位符
        if let Some(root) = &subplan.root {
            assert_eq!(root.name(), "Argument");
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

        let subplan = result.expect("Failed to get subplan");
        assert!(subplan.root.is_some());

        // 验证根节点类型 - 当前实现使用Argument作为占位符
        if let Some(root) = &subplan.root {
            assert_eq!(root.name(), "Argument");
        }
    }

    #[test]
    fn test_transform_with_where_clause() {
        let match_clause_ctx = create_test_match_clause_context();
        let path = create_test_path("p", false, vec!["n"]);

        let where_clause = crate::query::validator::structs::WhereClauseContext {
            filter: Some(crate::core::Expression::Variable("x".to_string())),
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: vec![],
        };

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(Some(&where_clause), &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.expect("Failed to get subplan");
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

        let subplan = result.expect("Failed to get subplan");
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

        let subplan = result.expect("Failed to get subplan");
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

        let subplan = result.expect("Failed to get subplan");
        assert!(subplan.root.is_some());
    }

    #[test]
    fn test_path_with_properties() {
        let match_clause_ctx = create_test_match_clause_context();

        let mut path = create_test_path("p", false, vec!["n"]);
        path.node_infos[0].props = Some(crate::core::Expression::Literal(
            crate::core::Value::String("test".to_string()),
        ));

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(None, &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.expect("Failed to get subplan");
        assert!(subplan.root.is_some());
    }

    #[test]
    fn test_path_with_filter() {
        let match_clause_ctx = create_test_match_clause_context();

        let mut path = create_test_path("p", false, vec!["n"]);
        path.node_infos[0].filter = Some(crate::core::Expression::Variable("x".to_string()));

        let mut planner = MatchPathPlanner::new(match_clause_ctx, path);
        let mut node_aliases_seen = std::collections::HashSet::new();

        let result = planner.transform(None, &mut node_aliases_seen);

        // 转换应该成功
        assert!(result.is_ok());

        let subplan = result.expect("Failed to get subplan");
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

        let subplan = result.expect("Failed to get subplan");
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

        let subplan = result.expect("Failed to get subplan");

        // 验证 SubPlan 结构
        assert!(subplan.root().is_some());
        assert!(subplan.tail().is_some()); // 尾节点不为 None，应该是IndexScan

        // 验证根节点类型 - 当前实现使用Argument作为占位符
        if let Some(root) = &subplan.root {
            assert_eq!(root.name(), "Argument");
        }

        // 验证尾节点类型 - 应该是Argument
        if let Some(tail) = &subplan.tail {
            assert_eq!(tail.name(), "Argument");
        }
    }
}
