//! 最短路径规划器 - 对照 nebula-graph 实现
//! 处理最短路径查询的规划
//! 负责规划最短路径算法的执行

use crate::query::planner::plan::factory::PlanNodeFactory;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::PlannerError;
use crate::query::validator::structs::{MatchClauseContext, Path, PathType, WhereClauseContext};
use crate::query::validator::{Column, Variable};
use std::collections::HashSet;

/// 最短路径规划器
/// 负责规划最短路径算法的执行
#[derive(Debug)]
pub struct ShortestPathPlanner {
    match_clause_ctx: MatchClauseContext,
    path: Path,
}

impl ShortestPathPlanner {
    pub fn new(match_clause_ctx: MatchClauseContext, path: Path) -> Self {
        Self {
            match_clause_ctx,
            path,
        }
    }

    /// 转换最短路径为执行计划 - 对照 nebula-graph 实现
    pub fn transform(
        &mut self,
        where_clause: Option<&WhereClauseContext>,
        node_aliases_seen: &mut HashSet<String>,
    ) -> Result<SubPlan, PlannerError> {
        // 1. 验证路径配置
        self.validate_path_config()?;

        // 2. 查找起始和结束节点
        let (start_plan, end_plan) = self.find_start_end_nodes(where_clause, node_aliases_seen)?;

        // 3. 创建连接节点
        let start_root = start_plan.root.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("Start plan should have a root node".to_string())
        })?;
        let end_root = end_plan.root.ok_or_else(|| {
            PlannerError::PlanGenerationFailed("End plan should have a root node".to_string())
        })?;

        let join_node = PlanNodeFactory::create_inner_join(
            start_root.clone(),
            end_root.clone(),
            vec![], // hash keys
            vec![], // probe keys
        )?;

        // 4. 创建最短路径节点
        let shortest_path_node = self.create_shortest_path_node(join_node)?;

        // 5. 构建最终计划
        let mut subplan = SubPlan::new(Some(shortest_path_node), start_plan.tail);

        // 6. 构建项目列
        self.build_project_columns(&mut subplan)?;

        Ok(subplan)
    }

    /// 验证路径配置 - 对照 nebula-graph 实现
    fn validate_path_config(&self) -> Result<(), PlannerError> {
        let node_infos = &self.path.node_infos;

        if node_infos.len() < 2 {
            return Err(PlannerError::InvalidOperation(
                "Shortest path requires at least 2 nodes".to_string(),
            ));
        }

        // 检查起始和结束节点不能相同
        if node_infos[0].alias == node_infos[1].alias {
            return Err(PlannerError::InvalidOperation(
                "The shortest path algorithm does not work when the start and end nodes are the same".to_string(),
            ));
        }

        // 检查边信息
        if self.path.edge_infos.is_empty() {
            return Err(PlannerError::InvalidOperation(
                "Shortest path requires at least 1 edge".to_string(),
            ));
        }

        Ok(())
    }

    /// 查找起始和结束节点 - 对照 nebula-graph 实现
    fn find_start_end_nodes(
        &self,
        where_clause: Option<&WhereClauseContext>,
        node_aliases_seen: &mut HashSet<String>,
    ) -> Result<(SubPlan, SubPlan), PlannerError> {
        let node_infos = &self.path.node_infos;

        // 查找起始节点
        let start_plan = self.find_node_plan(&node_infos[0], where_clause, node_aliases_seen)?;

        // 查找结束节点
        let end_plan = self.find_node_plan(&node_infos[1], where_clause, node_aliases_seen)?;

        Ok((start_plan, end_plan))
    }

    /// 查找节点计划 - 对照 nebula-graph 实现
    fn find_node_plan(
        &self,
        node_info: &crate::query::validator::structs::path_structs::NodeInfo,
        _where_clause: Option<&WhereClauseContext>,
        node_aliases_seen: &HashSet<String>,
    ) -> Result<SubPlan, PlannerError> {
        // 检查是否是已存在的别名（ArgumentFinder）
        if node_aliases_seen.contains(&node_info.alias) && !node_info.anonymous {
            let _variable = Variable {
                name: node_info.alias.clone(),
                columns: vec![Column {
                    name: node_info.alias.clone(),
                    type_: "Vertex".to_string(),
                }],
            };
            let argument_node = PlanNodeFactory::create_argument(0, &node_info.alias)?;
            return Ok(SubPlan::new(
                Some(argument_node.clone()),
                Some(argument_node),
            ));
        }

        // 检查标签索引（LabelIndexSeek）
        if !node_info.labels.is_empty() && !node_info.tids.is_empty() {
            if let Some(plan) = self.create_label_index_scan(node_info)? {
                return Ok(plan);
            }
        }

        // 检查属性索引（PropIndexSeek）
        if let Some(props) = &node_info.props {
            if let Some(plan) = self.create_prop_index_scan(node_info, props)? {
                return Ok(plan);
            }
        }

        Err(PlannerError::PlanGenerationFailed(
            "Can't find start/end node for shortest path".to_string(),
        ))
    }

    /// 创建最短路径节点 - 对照 nebula-graph 实现
    fn create_shortest_path_node(
        &self,
        shortest_path_node: PlanNodeEnum,
    ) -> Result<PlanNodeEnum, PlannerError> {
        let _edge_info = &self.path.edge_infos[0];

        // 根据路径类型创建不同的最短路径节点
        match self.path.path_type {
            PathType::Shortest => {
                // 简化实现，直接返回输入节点
                Ok(shortest_path_node)
            }
            PathType::AllShortest => {
                // 简化实现，直接返回输入节点
                Ok(shortest_path_node)
            }
            PathType::SingleSourceShortest => {
                // 简化实现，直接返回输入节点
                Ok(shortest_path_node)
            }
            _ => Err(PlannerError::UnsupportedOperation(
                "Unsupported path type for shortest path".to_string(),
            )),
        }
    }

    /// 构建项目列 - 对照 nebula-graph 实现
    fn build_project_columns(&self, _subplan: &mut SubPlan) -> Result<(), PlannerError> {
        // 创建项目列
        let _yield_columns = Vec::new();

        // 添加路径列
        if !self.path.anonymous {
            // 创建路径 YieldColumn
            // 这里需要根据实际的 YieldColumn 结构来创建
        }

        // 添加节点列
        for node_info in &self.path.node_infos {
            if !node_info.anonymous {
                // 创建节点 YieldColumn
            }
        }

        // 添加边列
        for edge_info in &self.path.edge_infos {
            if !edge_info.anonymous {
                // 创建边 YieldColumn
            }
        }

        // 创建项目节点
        if !_yield_columns.is_empty() {
            let root = _subplan.root.take().ok_or_else(|| {
                PlannerError::PlanGenerationFailed("Subplan should have a root node".to_string())
            })?;
            let project_node = PlanNodeFactory::create_project(root, _yield_columns)?;
            _subplan.root = Some(project_node);
        }

        Ok(())
    }

    /// 创建标签索引扫描
    fn create_label_index_scan(
        &self,
        node_info: &crate::query::validator::structs::path_structs::NodeInfo,
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
}
