//! 路径匹配规划器
//! 处理路径模式的规划
//! 负责规划路径模式的匹配

use crate::query::planner::plan::core::{PlanNode, PlanNodeMutable};
use crate::query::planner::plan::{SubPlan, PlanNodeKind, SingleInputNode};
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{
    MatchClauseContext, Path, WhereClauseContext,
};
use crate::query::context::validate::types::Variable;
use std::collections::HashSet;
use std::sync::Arc;

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
        let (start_index, start_from_edge, mut subplan) = self.find_starts(where_clause, node_aliases_seen)?;
        
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
                let label_index_seeker = crate::query::planner::match_planning::label_index_seek::LabelIndexSeek::new(node_info.clone());
                if label_index_seeker.match_node() {
                    let plan = label_index_seeker.build_plan()?;
                    return Ok((i, false, plan));
                }
            }
            
            // 检查节点是否在已见别名中
            if all_aliases_seen.contains(&node_info.alias) && !node_info.anonymous {
                // 创建参数节点
                let variable = Variable {
                    name: node_info.alias.clone(),
                    columns: vec![crate::query::context::validate::types::Column {
                        name: node_info.alias.clone(),
                        type_: "Vertex".to_string(),
                    }],
                };
                let arg_node = Arc::new(SingleInputNode::new(
                    PlanNodeKind::Argument,
                    create_empty_node()?,
                ));
                // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
                // 我们需要创建一个新的节点来设置属性
                let mut new_arg_node = (*arg_node).clone();
                new_arg_node.set_output_var(variable);
                new_arg_node.set_col_names(vec![node_info.alias.clone()]);
                let arg_node = Arc::new(new_arg_node);
                let plan = SubPlan::new(Some(arg_node.clone()), None);
                return Ok((i, false, plan));
            }
        }
        
        // 如果没有找到合适的起始节点，尝试从边开始
        for (i, edge_info) in self.path.edge_infos.iter().enumerate() {
            // 检查边是否可以使用索引查找
            if !edge_info.types.is_empty() {
                // 创建边索引扫描节点
                let var_name = format!("edge_scan_{}", edge_info.types.join("_"));
                let variable = Variable {
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
                let edge_scan_node = Arc::new(SingleInputNode::new(
                    PlanNodeKind::IndexScan,
                    create_empty_node()?,
                ));
                // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
                // 我们需要创建一个新的节点来设置属性
                let mut new_edge_scan_node = (*edge_scan_node).clone();
                new_edge_scan_node.set_output_var(variable);
                new_edge_scan_node.set_col_names(vec!["src".to_string(), "dst".to_string()]);
                let edge_scan_node = Arc::new(new_edge_scan_node);
                let plan = SubPlan::new(Some(edge_scan_node.clone()), Some(edge_scan_node));
                return Ok((i, true, plan));
            }
        }
        
        // 如果都没有找到，返回错误
        Err(PlannerError::PlanGenerationFailed(
            "Can't solve the start vids from the sentence.".to_string(),
        ))
    }
    
    /// 从节点扩展路径
    fn expand_from_node(&self, start_index: usize, subplan: &mut SubPlan) -> Result<(), PlannerError> {
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
    fn expand_from_edge(&self, start_index: usize, subplan: &mut SubPlan) -> Result<(), PlannerError> {
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
            
            // 创建遍历节点
            let traverse_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::Traverse,
                subplan.root.take().unwrap_or_else(|| create_empty_node().unwrap()),
            ));
            
            // 设置遍历参数
            let var_name = format!("traverse_{}_{}", node.alias, dst.alias);
            let variable = Variable {
                name: var_name,
                columns: vec![
                    crate::query::context::validate::types::Column {
                        name: dst.alias.clone(),
                        type_: "Vertex".to_string(),
                    },
                    crate::query::context::validate::types::Column {
                        name: "edge".to_string(),
                        type_: "Edge".to_string(),
                    },
                ],
            };
            
            // 设置列名
            let mut col_names = if let Some(root) = &subplan.root {
                root.col_names().clone()
            } else {
                vec![]
            };
            col_names.push(dst.alias.clone());
            col_names.push(edge.alias.clone());
            
            // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
            // 我们需要创建一个新的节点来设置属性
            let mut new_traverse_node = (*traverse_node).clone();
            new_traverse_node.set_output_var(variable);
            new_traverse_node.set_col_names(col_names);
            let traverse_node = Arc::new(new_traverse_node);
            
            // 处理节点过滤
            if let Some(_filter) = &node.filter {
                let var_name = format!("node_filter_{}", node.alias);
                let variable = Variable {
                    name: var_name,
                    columns: vec![crate::query::context::validate::types::Column {
                        name: node.alias.clone(),
                        type_: "Vertex".to_string(),
                    }],
                };
                let filter_node = Arc::new(SingleInputNode::new(
                    PlanNodeKind::Filter,
                    traverse_node,
                ));
                // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
                // 我们需要创建一个新的节点来设置属性
                let mut new_filter_node = (*filter_node).clone();
                new_filter_node.set_output_var(variable);
                let filter_node = Arc::new(new_filter_node);
                subplan.root = Some(filter_node);
            } else {
                subplan.root = Some(traverse_node);
            }
            
            // 处理边过滤
            if let Some(_filter) = &edge.filter {
                let var_name = format!("edge_filter_{}", edge.alias);
                let variable = Variable {
                    name: var_name,
                    columns: vec![crate::query::context::validate::types::Column {
                        name: edge.alias.clone(),
                        type_: "Edge".to_string(),
                    }],
                };
                let filter_node = Arc::new(SingleInputNode::new(
                    PlanNodeKind::Filter,
                    subplan.root.take().unwrap(),
                ));
                // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
                // 我们需要创建一个新的节点来设置属性
                let mut new_filter_node = (*filter_node).clone();
                new_filter_node.set_output_var(variable);
                let filter_node = Arc::new(new_filter_node);
                subplan.root = Some(filter_node);
            }
            
            // 记录已见过的节点别名
            node_aliases_seen_in_pattern.insert(dst.alias.clone());
        }
        
        // 处理最后一个节点
        let last_node = &node_infos[node_infos.len() - 1];
        if !node_aliases_seen_in_pattern.contains(&last_node.alias) {
            let var_name = format!("append_{}", last_node.alias);
            let variable = Variable {
                name: var_name,
                columns: vec![crate::query::context::validate::types::Column {
                    name: last_node.alias.clone(),
                    type_: "Vertex".to_string(),
                }],
            };
            let append_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::AppendVertices,
                subplan.root.take().unwrap(),
            ));
            // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
            // 我们需要创建一个新的节点来设置属性
            let mut new_append_node = (*append_node).clone();
            new_append_node.set_output_var(variable);
            let append_node = Arc::new(new_append_node);
            subplan.root = Some(append_node);
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
            
            // 创建遍历节点
            let traverse_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::Traverse,
                subplan.root.take().unwrap_or_else(|| create_empty_node().unwrap()),
            ));
            
            // 设置遍历参数
            let var_name = format!("traverse_{}_{}", node.alias, dst.alias);
            let variable = Variable {
                name: var_name,
                columns: vec![
                    crate::query::context::validate::types::Column {
                        name: dst.alias.clone(),
                        type_: "Vertex".to_string(),
                    },
                    crate::query::context::validate::types::Column {
                        name: "edge".to_string(),
                        type_: "Edge".to_string(),
                    },
                ],
            };
            
            // 设置列名
            let mut col_names = if let Some(root) = &subplan.root {
                root.col_names().clone()
            } else {
                vec![]
            };
            col_names.push(dst.alias.clone());
            col_names.push(edge.alias.clone());
            
            // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
            // 我们需要创建一个新的节点来设置属性
            let mut new_traverse_node = (*traverse_node).clone();
            new_traverse_node.set_output_var(variable);
            new_traverse_node.set_col_names(col_names);
            let traverse_node = Arc::new(new_traverse_node);
            
            // 处理节点过滤
            if let Some(_filter) = &node.filter {
                let var_name = format!("node_filter_{}", node.alias);
                let variable = Variable {
                    name: var_name,
                    columns: vec![crate::query::context::validate::types::Column {
                        name: node.alias.clone(),
                        type_: "Vertex".to_string(),
                    }],
                };
                let filter_node = Arc::new(SingleInputNode::new(
                    PlanNodeKind::Filter,
                    traverse_node,
                ));
                // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
                // 我们需要创建一个新的节点来设置属性
                let mut new_filter_node = (*filter_node).clone();
                new_filter_node.set_output_var(variable);
                let filter_node = Arc::new(new_filter_node);
                subplan.root = Some(filter_node);
            } else {
                subplan.root = Some(traverse_node);
            }
            
            // 处理边过滤
            if let Some(_filter) = &edge.filter {
                let var_name = format!("edge_filter_{}", edge.alias);
                let variable = Variable {
                    name: var_name,
                    columns: vec![crate::query::context::validate::types::Column {
                        name: edge.alias.clone(),
                        type_: "Edge".to_string(),
                    }],
                };
                let filter_node = Arc::new(SingleInputNode::new(
                    PlanNodeKind::Filter,
                    subplan.root.take().unwrap(),
                ));
                // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
                // 我们需要创建一个新的节点来设置属性
                let mut new_filter_node = (*filter_node).clone();
                new_filter_node.set_output_var(variable);
                let filter_node = Arc::new(new_filter_node);
                subplan.root = Some(filter_node);
            }
            
            // 记录已见过的节点别名
            node_aliases_seen_in_pattern.insert(dst.alias.clone());
        }
        
        // 处理第一个节点
        let first_node = &node_infos[0];
        if !node_aliases_seen_in_pattern.contains(&first_node.alias) {
            let var_name = format!("append_{}", first_node.alias);
            let variable = Variable {
                name: var_name,
                columns: vec![crate::query::context::validate::types::Column {
                    name: first_node.alias.clone(),
                    type_: "Vertex".to_string(),
                }],
            };
            let append_node = Arc::new(SingleInputNode::new(
                PlanNodeKind::AppendVertices,
                subplan.root.take().unwrap(),
            ));
            // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
            // 我们需要创建一个新的节点来设置属性
            let mut new_append_node = (*append_node).clone();
            new_append_node.set_output_var(variable);
            let append_node = Arc::new(new_append_node);
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
        let project_node = Arc::new(SingleInputNode::new(
            PlanNodeKind::Project,
            subplan.root.take().unwrap(),
        ));
        
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
        
        // 使用Arc::get_mut是不安全的，因为Arc可能有多个引用
        // 我们需要创建一个新的节点来设置属性
        let mut new_project_node = (*project_node).clone();
        new_project_node.set_col_names(col_names);
        new_project_node.set_output_var(variable);
        let project_node = Arc::new(new_project_node);
        
        subplan.root = Some(project_node);
        Ok(())
    }
}

/// 创建空节点
fn create_empty_node() -> Result<Arc<dyn crate::query::planner::plan::PlanNode>, PlannerError> {
    use crate::query::planner::plan::SingleDependencyNode;
    
    // 创建一个空的计划节点作为占位符
    Ok(Arc::new(SingleDependencyNode {
        id: -1,
        kind: PlanNodeKind::Start,
        dependencies: vec![],
        output_var: None,
        col_names: vec![],
        cost: 0.0,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::validator::structs::{
        MatchClauseContext, Path, NodeInfo, PathType
    };
    use crate::query::context::validate::types::Variable;
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
            filter: Some(crate::graph::expression::expr_type::Expression::Variable("x".to_string())),
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
    fn test_create_empty_node() {
        let result = create_empty_node();
        
        // 创建空节点应该成功
        assert!(result.is_ok());
        
        let node = result.unwrap();
        assert_eq!(node.kind(), PlanNodeKind::Start);
        assert_eq!(node.id(), -1);
        assert_eq!(node.dependencies().len(), 0);
        assert_eq!(node.cost(), 0.0);
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
        path.node_infos[0].props = Some(crate::graph::expression::expr_type::Expression::Constant(
            crate::core::Value::String("test".to_string())
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
        path.node_infos[0].filter = Some(crate::graph::expression::expr_type::Expression::Variable("x".to_string()));
        
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
        use crate::query::validator::structs::path_structs::{EdgeInfo, Direction};
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