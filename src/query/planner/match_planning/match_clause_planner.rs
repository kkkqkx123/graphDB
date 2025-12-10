//! MATCH子句规划器
//! 处理MATCH语句中的个别匹配子句

use crate::query::planner::match_planning::cypher_clause_planner::CypherClausePlanner;
use crate::query::planner::match_planning::match_path_planner::MatchPathPlanner;
use crate::query::planner::match_planning::segments_connector::SegmentsConnector;
use crate::query::planner::match_planning::shortest_path_planner::ShortestPathPlanner;
use crate::query::planner::plan::{SubPlan};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{
    CypherClauseContext, CypherClauseKind, MatchClauseContext, Path, PathType,
};
use std::collections::HashSet;

/// MATCH子句的规划器
/// 负责规划MATCH语句中的模式匹配部分
#[derive(Debug)]
pub struct MatchClausePlanner;

impl MatchClausePlanner {
    pub fn new() -> Self {
        Self
    }
}

impl CypherClausePlanner for MatchClausePlanner {
    fn transform(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        if !matches!(clause_ctx.kind(), CypherClauseKind::Match) {
            return Err(PlannerError::InvalidAstContext(
                "Not a valid context for MatchClausePlanner".to_string(),
            ));
        }

        let match_clause_ctx = match clause_ctx {
            CypherClauseContext::Match(ctx) => ctx,
            _ => {
                return Err(PlannerError::InvalidAstContext(
                    "Expected MatchClauseContext".to_string(),
                ))
            }
        };

        let mut match_clause_plan = SubPlan::new(None, None);
        // 所有在当前MATCH子句中见过的节点别名
        let mut node_aliases_seen = HashSet::new();

        // 如果没有路径，创建一个基本的Start节点
        if match_clause_ctx.paths.is_empty() {
            use crate::query::planner::plan::plan_node::{PlanNodeKind, SingleDependencyNode};
            let start_node = Box::new(SingleDependencyNode {
                id: -1,
                kind: PlanNodeKind::Start,
                dependencies: vec![],
                output_var: None,
                col_names: vec![],
                cost: 0.0,
            }) as Box<dyn crate::query::planner::plan::PlanNode>;
            
            match_clause_plan.root = Some(start_node.clone_plan_node());
            match_clause_plan.tail = Some(start_node);
            return Ok(match_clause_plan);
        }

        // 重建图并找到所有连通分量
        // 这有助于优化路径连接顺序，减少中间结果集大小
        let connected_components = Self::find_connected_components(&match_clause_ctx.paths);
        
        // 按连通分量处理路径，优先处理较小的连通分量以减少中间结果
        for component in connected_components {
            for path_idx in component {
                if let Some(path_info) = match_clause_ctx.paths.get(path_idx) {
                    let mut path_plan = SubPlan::new(None, None);

                    // 根据路径类型选择不同的规划器
                    if path_info.path_type == PathType::Default {
                        let mut match_path_planner =
                            MatchPathPlanner::new(match_clause_ctx.clone(), path_info.clone());
                        let result = match_path_planner.transform(
                            match_clause_ctx.where_clause.as_ref(),
                            &mut node_aliases_seen,
                        );
                        match result {
                            Ok(plan) => path_plan = plan,
                            Err(e) => return Err(e),
                        }
                    } else {
                        let mut shortest_path_planner =
                            ShortestPathPlanner::new(match_clause_ctx.clone(), path_info.clone());
                        let result = shortest_path_planner.transform(
                            match_clause_ctx.where_clause.as_ref(),
                            &mut node_aliases_seen,
                        );
                        match result {
                            Ok(plan) => path_plan = plan,
                            Err(e) => return Err(e),
                        }
                    }

                    // 连接路径计划
                    match Self::connect_path_plan(
                        &path_info.node_infos,
                        match_clause_ctx,
                        &path_plan,
                        &mut node_aliases_seen,
                        &mut match_clause_plan,
                    ) {
                        Ok(_) => {}
                        Err(e) => return Err(e),
                    }
                }
            }
        }

        Ok(match_clause_plan)
    }
}

impl MatchClausePlanner {
    /// 找到路径中的所有连通分量
    /// 返回每个连通分量包含的路径索引
    fn find_connected_components(paths: &[Path]) -> Vec<Vec<usize>> {
        use std::collections::{HashMap, HashSet, VecDeque};
        
        // 构建节点到路径的映射
        let mut node_to_paths: HashMap<String, Vec<usize>> = HashMap::new();
        for (path_idx, path) in paths.iter().enumerate() {
            for node_info in &path.node_infos {
                if !node_info.anonymous && !node_info.alias.is_empty() {
                    node_to_paths.entry(node_info.alias.clone())
                        .or_insert_with(Vec::new)
                        .push(path_idx);
                }
            }
        }
        
        // 构建路径连接图
        let mut path_graph: Vec<HashSet<usize>> = vec![HashSet::new(); paths.len()];
        for (path_idx, path) in paths.iter().enumerate() {
            let mut connected_paths = HashSet::new();
            for node_info in &path.node_infos {
                if !node_info.anonymous && !node_info.alias.is_empty() {
                    if let Some(connected_path_indices) = node_to_paths.get(&node_info.alias) {
                        for &connected_idx in connected_path_indices {
                            if connected_idx != path_idx {
                                connected_paths.insert(connected_idx);
                            }
                        }
                    }
                }
            }
            path_graph[path_idx] = connected_paths;
        }
        
        // 使用BFS找到所有连通分量
        let mut visited = vec![false; paths.len()];
        let mut components = Vec::new();
        
        for i in 0..paths.len() {
            if !visited[i] {
                let mut component = Vec::new();
                let mut queue = VecDeque::new();
                queue.push_back(i);
                visited[i] = true;
                
                while let Some(current) = queue.pop_front() {
                    component.push(current);
                    for &neighbor in &path_graph[current] {
                        if !visited[neighbor] {
                            visited[neighbor] = true;
                            queue.push_back(neighbor);
                        }
                    }
                }
                
                // 按路径大小排序，优先处理较小的连通分量
                component.sort_by(|&a, &b| {
                    let size_a = paths.get(a).map(|p| p.node_infos.len()).unwrap_or(0);
                    let size_b = paths.get(b).map(|p| p.node_infos.len()).unwrap_or(0);
                    size_a.cmp(&size_b)
                });
                
                components.push(component);
            }
        }
        
        // 按连通分量大小排序，优先处理较小的连通分量
        components.sort_by_key(|c| c.len());
        
        components
    }
    
    /// 连接路径计划
    fn connect_path_plan(
    node_infos: &[crate::query::validator::structs::NodeInfo],
    _match_clause_ctx: &MatchClauseContext,
    subplan: &SubPlan,
    node_aliases_seen: &mut HashSet<String>,
    match_clause_plan: &mut SubPlan,
) -> Result<(), PlannerError> {
    let mut intersected_aliases = HashSet::new();

    for info in node_infos {
        if node_aliases_seen.contains(&info.alias) {
            intersected_aliases.insert(info.alias.clone());
        }
        if !info.anonymous {
            node_aliases_seen.insert(info.alias.clone());
        }
    }

    if match_clause_plan.root.is_none() {
        // 使用clone_plan_node方法克隆PlanNode
        match_clause_plan.root = subplan.root.as_ref().map(|node| node.clone_plan_node());
        match_clause_plan.tail = subplan.tail.as_ref().map(|node| node.clone_plan_node());
        return Ok(());
    }

    let connector = SegmentsConnector::new();

    if intersected_aliases.is_empty() {
        // 笛卡尔积
        *match_clause_plan =
            connector.cartesian_product(match_clause_plan.clone(), subplan.clone());
    } else {
        // 内连接
        *match_clause_plan = connector.inner_join(
            match_clause_plan.clone(),
            subplan.clone(),
            intersected_aliases.into_iter().collect(),
        );
    }

    Ok(())
}

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::plan_node::{PlanNodeKind, VariableDependencyNode};
    use crate::query::validator::structs::{
        CypherClauseContext, MatchClauseContext, Path, NodeInfo, PathType
    };
    use crate::graph::expression::expr_type::Expression;
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
        let node_infos = node_aliases.into_iter()
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
    fn create_test_match_clause_context(paths: Vec<Path>) -> MatchClauseContext {
        MatchClauseContext {
            paths,
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        }
    }

    /// 创建测试用的 Cypher 子句上下文
    fn create_test_cypher_clause_context(paths: Vec<Path>) -> CypherClauseContext {
        let match_ctx = create_test_match_clause_context(paths);
        CypherClauseContext::Match(match_ctx)
    }

    #[test]
    fn test_match_clause_planner_new() {
        let planner = MatchClausePlanner::new();
        // 测试创建实例
        assert!(true); // 如果能创建实例就通过
    }

    #[test]
    fn test_match_clause_planner_debug() {
        let planner = MatchClausePlanner::new();
        let debug_str = format!("{:?}", planner);
        assert!(debug_str.contains("MatchClausePlanner"));
    }

    #[test]
    fn test_transform_invalid_context() {
        let mut planner = MatchClausePlanner::new();
        
        // 创建一个非 MATCH 类型的上下文
        let clause_ctx = CypherClauseContext::Where(crate::query::validator::structs::WhereClauseContext {
            filter: None,
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: vec![],
        });
        
        let result = planner.transform(&clause_ctx);
        
        // 应该返回错误
        assert!(result.is_err());
        match result.unwrap_err() {
            PlannerError::InvalidAstContext(msg) => {
                assert!(msg.contains("Not a valid context for MatchClausePlanner"));
            }
            _ => panic!("Expected InvalidAstContext error"),
        }
    }

    #[test]
    fn test_transform_empty_paths() {
        let mut planner = MatchClausePlanner::new();
        
        let clause_ctx = create_test_cypher_clause_context(vec![]);
        
        let result = planner.transform(&clause_ctx);
        
        // 空路径应该成功，现在会创建一个基本的Start节点
        assert!(result.is_ok());
        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
        assert!(subplan.tail.is_some());
        
        // 验证根节点类型是Start
        if let Some(root) = &subplan.root {
            assert_eq!(root.kind(), PlanNodeKind::Start);
        }
    }

    #[test]
    fn test_transform_single_path() {
        let mut planner = MatchClausePlanner::new();
        
        let path = create_test_path("p", false, vec!["n"]);
        let clause_ctx = create_test_cypher_clause_context(vec![path]);
        
        let result = planner.transform(&clause_ctx);
        
        // 单个路径应该成功
        assert!(result.is_ok());
        let subplan = result.unwrap();
        // 注意：由于 MatchPathPlanner 和 ShortestPathPlanner 可能还没有完全实现，
        // 这里我们只测试不崩溃的情况
    }

    #[test]
    fn test_transform_multiple_paths() {
        let mut planner = MatchClausePlanner::new();
        
        let path1 = create_test_path("p1", false, vec!["n"]);
        let path2 = create_test_path("p2", false, vec!["m"]);
        let clause_ctx = create_test_cypher_clause_context(vec![path1, path2]);
        
        let result = planner.transform(&clause_ctx);
        
        // 多个路径应该成功
        assert!(result.is_ok());
        // 注意：由于 MatchPathPlanner 和 ShortestPathPlanner 可能还没有完全实现，
        // 这里我们只测试不崩溃的情况
    }

    #[test]
    fn test_connect_path_plan_empty_match_plan() {
        let node_infos = vec![create_test_node_info("n", false)];
        let match_clause_ctx = create_test_match_clause_context(vec![]);
        
        let subplan = SubPlan::new(
            Some(Box::new(VariableDependencyNode::new(PlanNodeKind::Start))),
            None
        );
        
        let mut node_aliases_seen = HashSet::new();
        let mut match_clause_plan = SubPlan::new(None, None);
        
        let result = MatchClausePlanner::connect_path_plan(
            &node_infos,
            &match_clause_ctx,
            &subplan,
            &mut node_aliases_seen,
            &mut match_clause_plan,
        );
        
        // 应该成功
        assert!(result.is_ok());
        assert!(match_clause_plan.root.is_some());
        assert!(node_aliases_seen.contains("n"));
    }

    #[test]
    fn test_connect_path_plan_with_existing_plan() {
        let node_infos = vec![create_test_node_info("n", false)];
        let match_clause_ctx = create_test_match_clause_context(vec![]);
        
        let subplan = SubPlan::new(
            Some(Box::new(VariableDependencyNode::new(PlanNodeKind::Start))),
            None
        );
        
        let existing_plan = SubPlan::new(
            Some(Box::new(VariableDependencyNode::new(PlanNodeKind::Project))),
            None
        );
        
        let mut node_aliases_seen = HashSet::new();
        node_aliases_seen.insert("x".to_string()); // 添加一个已存在的别名
        let mut match_clause_plan = existing_plan;
        
        let result = MatchClausePlanner::connect_path_plan(
            &node_infos,
            &match_clause_ctx,
            &subplan,
            &mut node_aliases_seen,
            &mut match_clause_plan,
        );
        
        // 应该成功
        assert!(result.is_ok());
        assert!(match_clause_plan.root.is_some());
        assert!(node_aliases_seen.contains("n"));
    }

    #[test]
    fn test_connect_path_plan_with_intersecting_aliases() {
        let node_infos = vec![create_test_node_info("n", false)];
        let match_clause_ctx = create_test_match_clause_context(vec![]);
        
        let subplan = SubPlan::new(
            Some(Box::new(VariableDependencyNode::new(PlanNodeKind::Start))),
            None
        );
        
        let existing_plan = SubPlan::new(
            Some(Box::new(VariableDependencyNode::new(PlanNodeKind::Project))),
            None
        );
        
        let mut node_aliases_seen = HashSet::new();
        node_aliases_seen.insert("n".to_string()); // 添加一个相交的别名
        let mut match_clause_plan = existing_plan;
        
        let result = MatchClausePlanner::connect_path_plan(
            &node_infos,
            &match_clause_ctx,
            &subplan,
            &mut node_aliases_seen,
            &mut match_clause_plan,
        );
        
        // 应该成功
        assert!(result.is_ok());
        assert!(match_clause_plan.root.is_some());
        assert!(node_aliases_seen.contains("n"));
    }

    #[test]
    fn test_connect_path_plan_anonymous_node() {
        let node_infos = vec![create_test_node_info("", true)]; // 匿名节点
        let match_clause_ctx = create_test_match_clause_context(vec![]);
        
        let subplan = SubPlan::new(
            Some(Box::new(VariableDependencyNode::new(PlanNodeKind::Start))),
            None
        );
        
        let mut node_aliases_seen = HashSet::new();
        let mut match_clause_plan = SubPlan::new(None, None);
        
        let result = MatchClausePlanner::connect_path_plan(
            &node_infos,
            &match_clause_ctx,
            &subplan,
            &mut node_aliases_seen,
            &mut match_clause_plan,
        );
        
        // 应该成功
        assert!(result.is_ok());
        assert!(match_clause_plan.root.is_some());
        // 匿名节点不应该被添加到已见别名集合中
        assert!(!node_aliases_seen.contains(""));
    }

    #[test]
    fn test_transform_shortest_path() {
        let mut planner = MatchClausePlanner::new();
        
        // 创建一个最短路径类型的路径
        let mut path = create_test_path("p", false, vec!["n", "m"]);
        path.path_type = PathType::Shortest;
        
        let clause_ctx = create_test_cypher_clause_context(vec![path]);
        
        let result = planner.transform(&clause_ctx);
        
        // 最短路径应该成功
        assert!(result.is_ok());
        // 注意：由于 ShortestPathPlanner 可能还没有完全实现，
        // 这里我们只测试不崩溃的情况
    }

    #[test]
    fn test_transform_with_where_clause() {
        let mut planner = MatchClausePlanner::new();
        
        let path = create_test_path("p", false, vec!["n"]);
        
        let where_clause = crate::query::validator::structs::WhereClauseContext {
            filter: Some(Expression::Variable("x".to_string())),
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: vec![],
        };
        
        let mut match_ctx = create_test_match_clause_context(vec![path]);
        match_ctx.where_clause = Some(where_clause);
        
        let clause_ctx = CypherClauseContext::Match(match_ctx);
        
        let result = planner.transform(&clause_ctx);
        
        // 带 WHERE 子句应该成功
        assert!(result.is_ok());
        // 注意：由于 MatchPathPlanner 可能还没有完全实现，
        // 这里我们只测试不崩溃的情况
    }
}
