//! 统一 MATCH 语句规划器
//!
//! 实现 StatementPlanner 接口，处理完整的 MATCH 查询规划。
//! 整合了以下功能：
//! - 节点和边模式匹配（支持多路径）
//! - WHERE 条件过滤
//! - RETURN 投影
//! - ORDER BY 排序
//! - LIMIT/SKIP 分页
//! - 智能扫描策略选择（索引扫描、属性扫描、全表扫描）

use crate::core::Expression;
use crate::core::Value;
use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::parser::ast::pattern::{Pattern, PathElement};
use crate::query::planner::plan::ExecutionPlan;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::{LimitNode, ProjectNode, ScanVerticesNode, SortNode, SortItem};
use crate::query::planner::plan::core::nodes::ExpandAllNode;
use crate::core::types::graph_schema::OrderDirection;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::planner::statements::statement_planner::StatementPlanner;
use crate::query::planner::statements::paths::{PathPattern, PathPatternKind, EdgePattern};
use crate::query::planner::statements::seeks::NodePattern;
use crate::query::planner::statements::seeks::seek_strategy_base::{SeekStrategyContext, SeekStrategySelector, SeekStrategyType};
use crate::core::YieldColumn;
use crate::query::validator::structs::OrderByItem;
use crate::query::validator::structs::CypherClauseKind;
use std::sync::Arc;

/// 分页信息结构体
#[derive(Debug, Clone)]
pub struct PaginationInfo {
    pub skip: usize,
    pub limit: usize,
}

/// MATCH 语句规划器
///
/// 负责将 MATCH 查询转换为可执行的执行计划。
/// 实现 StatementPlanner 接口，提供统一的规划入口。
#[derive(Debug, Clone)]
pub struct MatchStatementPlanner {
    config: MatchPlannerConfig,
}

#[derive(Debug, Clone, Default)]
pub struct MatchPlannerConfig {
    pub default_limit: usize,
    pub max_limit: usize,
    pub enable_index_optimization: bool,
}

impl Default for MatchStatementPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl MatchStatementPlanner {
    pub fn new() -> Self {
        Self {
            config: MatchPlannerConfig::default(),
        }
    }

    pub fn with_config(config: MatchPlannerConfig) -> Self {
        Self { config }
    }

    /// 将 AST Pattern 转换为内部 PathPattern
    fn convert_pattern_to_path_pattern(&self, pattern: &Pattern) -> Option<PathPattern> {
        match pattern {
            Pattern::Path(path) => {
                let elements = &path.elements;
                if elements.len() < 3 {
                    return None;
                }

                // 解析起始节点
                let start = self.convert_path_element_to_node(&elements[0])?;
                
                // 解析边
                let edge = self.convert_path_element_to_edge(&elements[1])?;
                
                // 解析结束节点
                let end = self.convert_path_element_to_node(&elements[2])?;

                // 检查是否为可变长度路径
                if let PathElement::Edge(edge_elem) = &elements[1] {
                    if edge_elem.range.is_some() {
                        let range = edge_elem.range.as_ref()?;
                        return Some(PathPattern {
                            kind: PathPatternKind::VariableLength {
                                start,
                                edge,
                                end,
                                lower: range.min,
                                upper: range.max,
                            },
                        });
                    }
                }

                Some(PathPattern {
                    kind: PathPatternKind::Simple { start, edge, end },
                })
            }
            _ => None,
        }
    }

    /// 将 PathElement::Node 转换为 NodePattern
    fn convert_path_element_to_node(&self, element: &PathElement) -> Option<NodePattern> {
        match element {
            PathElement::Node(node) => {
                let labels = node.labels.clone();
                let properties = node.properties.as_ref()
                    .map(|props| self.extract_properties_from_expr(props))
                    .unwrap_or_default();
                
                Some(NodePattern {
                    vid: None, // TODO: 从属性中提取 VID
                    labels,
                    properties,
                })
            }
            _ => None,
        }
    }

    /// 将 PathElement::Edge 转换为 EdgePattern
    fn convert_path_element_to_edge(&self, element: &PathElement) -> Option<EdgePattern> {
        match element {
            PathElement::Edge(edge) => {
                let types = if edge.edge_types.is_empty() {
                    None
                } else {
                    Some(edge.edge_types.clone())
                };
                
                let direction = Some(match edge.direction {
                    crate::query::parser::ast::types::EdgeDirection::Out => crate::core::types::graph_schema::EdgeDirection::Out,
                    crate::query::parser::ast::types::EdgeDirection::In => crate::core::types::graph_schema::EdgeDirection::In,
                    crate::query::parser::ast::types::EdgeDirection::Both => crate::core::types::graph_schema::EdgeDirection::Both,
                });

                let properties = edge.properties.as_ref()
                    .map(|props| self.extract_properties_from_expr(props))
                    .unwrap_or_default();

                Some(EdgePattern {
                    types,
                    direction,
                    properties,
                })
            }
            _ => None,
        }
    }

    /// 从表达式中提取属性键值对
    fn extract_properties_from_expr(&self, expr: &Expression) -> Vec<(String, Value)> {
        let mut properties = Vec::new();
        
        if let Expression::Map(entries) = expr {
            for (key, value_expr) in entries {
                if let Some(value) = self.expr_to_value(value_expr) {
                    properties.push((key.clone(), value));
                }
            }
        }
        
        properties
    }

    /// 将表达式转换为 Value
    fn expr_to_value(&self, expr: &Expression) -> Option<Value> {
        match expr {
            Expression::Literal(val) => Some(val.clone()),
            _ => None,
        }
    }

    /// 选择最佳的节点扫描策略
    fn select_scan_strategy(&self, node_info: &crate::query::validator::structs::NodeInfo, space_id: u64) -> ScanStrategy {
        let node_pattern = NodePattern {
            vid: None,
            labels: node_info.labels.clone(),
            properties: node_info.props.as_ref()
                .map(|props| self.extract_properties_from_expr(props))
                .unwrap_or_default(),
        };

        let context = SeekStrategyContext::new(
            space_id,
            node_pattern,
            node_info.filter.clone().into_iter().collect(),
        );

        let selector = SeekStrategySelector::new();
        let strategy_type = selector.select_strategy(&DummyStorage, &context);

        match strategy_type {
            SeekStrategyType::VertexSeek => ScanStrategy::VertexSeek,
            SeekStrategyType::IndexSeek => ScanStrategy::IndexScan(node_info.tids.clone()),
            SeekStrategyType::PropIndexSeek => ScanStrategy::PropIndexScan,
            SeekStrategyType::VariablePropIndexSeek => ScanStrategy::VariablePropIndexScan,
            SeekStrategyType::EdgeSeek => ScanStrategy::EdgeScan,
            SeekStrategyType::ScanSeek => ScanStrategy::FullScan,
        }
    }
}

/// 扫描策略枚举
#[derive(Debug, Clone)]
enum ScanStrategy {
    VertexSeek,
    IndexScan(Vec<i32>),
    PropIndexScan,
    VariablePropIndexScan,
    EdgeScan,
    FullScan,
}

/// 虚拟存储实现，用于策略选择
#[derive(Debug)]
struct DummyStorage;

impl crate::storage::StorageClient for DummyStorage {
    fn insert_vertex(&mut self, _space: &str, _vertex: crate::core::Vertex) -> Result<Value, crate::core::StorageError> {
        Ok(Value::Int(0))
    }
    fn get_vertex(&self, _space: &str, _id: &Value) -> Result<Option<crate::core::Vertex>, crate::core::StorageError> {
        Ok(None)
    }
    fn update_vertex(&mut self, _space: &str, _vertex: crate::core::Vertex) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn delete_vertex(&mut self, _space: &str, _id: &Value) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn scan_vertices(&self, _space: &str) -> Result<Vec<crate::core::Vertex>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn scan_vertices_by_tag(&self, _space: &str, _tag: &str) -> Result<Vec<crate::core::Vertex>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn scan_vertices_by_prop(&self, _space: &str, _tag: &str, _prop: &str, _value: &Value) -> Result<Vec<crate::core::Vertex>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn insert_edge(&mut self, _space: &str, _edge: crate::core::Edge) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn get_edge(&self, _space: &str, _src: &Value, _dst: &Value, _edge_type: &str) -> Result<Option<crate::core::Edge>, crate::core::StorageError> {
        Ok(None)
    }
    fn get_node_edges(&self, _space: &str, _node_id: &Value, _direction: crate::core::types::graph_schema::EdgeDirection) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn get_node_edges_filtered(&self, _space: &str, _node_id: &Value, _direction: crate::core::types::graph_schema::EdgeDirection, _filter: Option<Box<dyn Fn(&crate::core::Edge) -> bool + Send + Sync>>) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn delete_edge(&mut self, _space: &str, _src: &Value, _dst: &Value, _edge_type: &str) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn scan_edges_by_type(&self, _space: &str, _edge_type: &str) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn scan_all_edges(&self, _space: &str) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn batch_insert_vertices(&mut self, _space: &str, _vertices: Vec<crate::core::Vertex>) -> Result<Vec<Value>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn batch_insert_edges(&mut self, _space: &str, _edges: Vec<crate::core::Edge>) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn create_space(&mut self, _space: &crate::core::types::SpaceInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn drop_space(&mut self, _space: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_space(&self, _space: &str) -> Result<Option<crate::core::types::SpaceInfo>, crate::core::StorageError> {
        Ok(None)
    }
    fn get_space_by_id(&self, _space_id: u64) -> Result<Option<crate::core::types::SpaceInfo>, crate::core::StorageError> {
        Ok(None)
    }
    fn list_spaces(&self) -> Result<Vec<crate::core::types::SpaceInfo>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn create_tag(&mut self, _space: &str, _info: &crate::core::types::TagInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn alter_tag(&mut self, _space: &str, _tag_name: &str, _additions: Vec<crate::core::types::PropertyDef>, _deletions: Vec<String>) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_tag(&self, _space: &str, _tag_name: &str) -> Result<Option<crate::core::types::TagInfo>, crate::core::StorageError> {
        Ok(None)
    }
    fn drop_tag(&mut self, _space: &str, _tag_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn list_tags(&self, _space: &str) -> Result<Vec<crate::core::types::TagInfo>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn create_edge_type(&mut self, _space: &str, _info: &crate::core::types::EdgeTypeSchema) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn alter_edge_type(&mut self, _space: &str, _edge_type_name: &str, _additions: Vec<crate::core::types::PropertyDef>, _deletions: Vec<String>) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_edge_type(&self, _space: &str, _edge_type_name: &str) -> Result<Option<crate::core::types::EdgeTypeSchema>, crate::core::StorageError> {
        Ok(None)
    }
    fn drop_edge_type(&mut self, _space: &str, _edge_type_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn list_edge_types(&self, _space: &str) -> Result<Vec<crate::core::types::EdgeTypeSchema>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn create_tag_index(&mut self, _space: &str, _info: &crate::index::Index) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn drop_tag_index(&mut self, _space: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_tag_index(&self, _space: &str, _index_name: &str) -> Result<Option<crate::index::Index>, crate::core::StorageError> {
        Ok(None)
    }
    fn list_tag_indexes(&self, _space: &str) -> Result<Vec<crate::index::Index>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn rebuild_tag_index(&mut self, _space: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn create_edge_index(&mut self, _space: &str, _info: &crate::index::Index) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn drop_edge_index(&mut self, _space_name: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_edge_index(&self, _space_name: &str, _index_name: &str) -> Result<Option<crate::index::Index>, crate::core::StorageError> {
        Ok(None)
    }
    fn list_edge_indexes(&self, _space: &str) -> Result<Vec<crate::index::Index>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn rebuild_edge_index(&mut self, _space: &str, _index_name: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn change_password(&mut self, _info: &crate::core::types::PasswordInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn create_user(&mut self, _info: &crate::core::types::metadata::UserInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn alter_user(&mut self, _info: &crate::core::types::metadata::UserAlterInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn drop_user(&mut self, _username: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_space_id(&self, _space: &str) -> Result<u64, crate::core::StorageError> {
        Ok(1)
    }
    fn space_exists(&self, _space: &str) -> bool {
        false
    }
    fn clear_space(&mut self, _space: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn alter_space_comment(&mut self, _space_id: u64, _comment: String) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn grant_role(&mut self, _username: &str, _space_id: u64, _role: crate::api::service::permission_manager::RoleType) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn revoke_role(&mut self, _username: &str, _space_id: u64) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn insert_vertex_data(&mut self, _space: &str, _info: &crate::core::types::InsertVertexInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn insert_edge_data(&mut self, _space: &str, _info: &crate::core::types::InsertEdgeInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn delete_vertex_data(&mut self, _space: &str, _vertex_id: &str) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn delete_edge_data(&mut self, _space: &str, _src: &str, _dst: &str, _rank: i64) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn update_data(&mut self, _space: &str, _info: &crate::core::types::UpdateInfo) -> Result<bool, crate::core::StorageError> {
        Ok(true)
    }
    fn get_vertex_with_schema(&self, _space: &str, _tag_name: &str, _id: &Value) -> Result<Option<(crate::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(None)
    }
    fn get_edge_with_schema(&self, _space: &str, _edge_type_name: &str, _src: &Value, _dst: &Value) -> Result<Option<(crate::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(None)
    }
    fn scan_vertices_with_schema(&self, _space: &str, _tag_name: &str) -> Result<Vec<(crate::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn scan_edges_with_schema(&self, _space: &str, _edge_type_name: &str) -> Result<Vec<(crate::storage::Schema, Vec<u8>)>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn lookup_index(&self, _space: &str, _index: &str, _value: &Value) -> Result<Vec<Value>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn lookup_index_with_score(&self, _space: &str, _index: &str, _value: &Value) -> Result<Vec<(Value, f32)>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn load_from_disk(&mut self) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn save_to_disk(&self) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn get_storage_stats(&self) -> crate::storage::storage_client::StorageStats {
        crate::storage::storage_client::StorageStats {
            total_vertices: 0,
            total_edges: 0,
            total_spaces: 0,
            total_tags: 0,
            total_edge_types: 0,
        }
    }
    fn delete_vertex_with_edges(&mut self, _space: &str, _id: &Value) -> Result<(), crate::core::StorageError> {
        Ok(())
    }
    fn delete_tags(&mut self, _space: &str, _vertex_id: &Value, _tag_names: &[String]) -> Result<usize, crate::core::StorageError> {
        Ok(0)
    }
    fn find_dangling_edges(&self, _space: &str) -> Result<Vec<crate::core::Edge>, crate::core::StorageError> {
        Ok(Vec::new())
    }
    fn repair_dangling_edges(&mut self, _space: &str) -> Result<usize, crate::core::StorageError> {
        Ok(0)
    }
}

impl Planner for MatchStatementPlanner {
    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Match(_))
    }

    fn transform(&mut self, stmt: &Stmt, qctx: Arc<QueryContext>) -> Result<SubPlan, PlannerError> {
        let space_id = qctx.rctx().space_id().unwrap_or(1) as u64;
        self.plan_match_pattern(stmt, space_id)
    }

    fn transform_with_full_context(
        &mut self,
        qctx: Arc<QueryContext>,
        stmt: &Stmt,
    ) -> Result<ExecutionPlan, PlannerError> {
        let sub_plan = self.transform(stmt, qctx)?;
        Ok(ExecutionPlan::new(sub_plan.root().clone()))
    }
}

impl StatementPlanner for MatchStatementPlanner {
    fn statement_type(&self) -> &'static str {
        "MATCH"
    }

    fn supported_clause_kinds(&self) -> &[CypherClauseKind] {
        const SUPPORTED_CLAUSES: &[CypherClauseKind] = &[
            CypherClauseKind::Match,
            CypherClauseKind::Where,
            CypherClauseKind::Return,
            CypherClauseKind::OrderBy,
            CypherClauseKind::Pagination,
        ];
        SUPPORTED_CLAUSES
    }
}

impl MatchStatementPlanner {
    fn plan_match_pattern(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                // 处理路径模式
                let mut plan = if match_stmt.patterns.is_empty() {
                    // 没有路径模式时使用默认节点扫描
                    self.plan_node_pattern(space_id)?
                } else {
                    // 处理第一个路径模式
                    let first_pattern = &match_stmt.patterns[0];
                    self.plan_path_pattern(first_pattern, space_id)?
                };

                // 处理额外的路径模式（使用交叉连接）
                for pattern in match_stmt.patterns.iter().skip(1) {
                    let path_plan = self.plan_path_pattern(pattern, space_id)?;
                    plan = self.cross_join_plans(plan, path_plan)?;
                }

                if let Some(condition) = self.extract_where_condition(stmt)? {
                    plan = self.plan_filter(plan, condition, space_id)?;
                }

                if let Some(columns) = self.extract_return_columns(stmt)? {
                    plan = self.plan_project(plan, columns, space_id)?;
                }

                if let Some(order_by) = self.extract_order_by(stmt)? {
                    plan = self.plan_sort(plan, order_by, space_id)?;
                }

                if let Some(pagination) = self.extract_pagination(stmt)? {
                    plan = self.plan_limit(plan, pagination)?;
                }

                Ok(plan)
            }
            _ => Err(PlannerError::InvalidOperation(
                "Expected MATCH statement".to_string()
            ))
        }
    }

    /// 规划路径模式
    fn plan_path_pattern(
        &self,
        pattern: &Pattern,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        match pattern {
            Pattern::Path(path) => {
                if path.elements.is_empty() {
                    return Err(PlannerError::PlanGenerationFailed(
                        "空路径模式".to_string()
                    ));
                }

                let mut plan = SubPlan::new(None, None);
                let mut prev_node_alias: Option<String> = None;

                for (_idx, element) in path.elements.iter().enumerate() {
                    match element {
                        PathElement::Node(node) => {
                            // 规划节点扫描
                            let node_plan = self.plan_pattern_node(node, space_id)?;
                            
                            plan = if let Some(existing_root) = plan.root.take() {
                                if let Some(ref alias) = prev_node_alias {
                                    // 如果有前一个节点，使用连接
                                    self.join_node_plans(
                                        SubPlan::new(Some(existing_root), plan.tail),
                                        node_plan,
                                        alias,
                                        &node.variable,
                                    )?
                                } else {
                                    // 第一个节点，使用交叉连接
                                    self.cross_join_plans(
                                        SubPlan::new(Some(existing_root), plan.tail),
                                        node_plan,
                                    )?
                                }
                            } else {
                                node_plan
                            };

                            prev_node_alias = node.variable.clone();
                        }
                        PathElement::Edge(edge) => {
                            // 规划边扩展
                            if prev_node_alias.is_none() {
                                return Err(PlannerError::PlanGenerationFailed(
                                    "边模式必须跟随节点模式".to_string()
                                ));
                            }

                            let edge_plan = self.plan_pattern_edge(edge, space_id)?;
                            plan = if let Some(existing_root) = plan.root.take() {
                                self.cross_join_plans(
                                    SubPlan::new(Some(existing_root), plan.tail),
                                    edge_plan,
                                )?
                            } else {
                                edge_plan
                            };
                        }
                        PathElement::Alternative(_) => {
                            // TODO: 处理替代路径
                            return Err(PlannerError::PlanGenerationFailed(
                                "替代路径模式尚未实现".to_string()
                            ));
                        }
                        PathElement::Optional(_) => {
                            // TODO: 处理可选路径
                            return Err(PlannerError::PlanGenerationFailed(
                                "可选路径模式尚未实现".to_string()
                            ));
                        }
                        PathElement::Repeated(_, _) => {
                            // TODO: 处理重复路径
                            return Err(PlannerError::PlanGenerationFailed(
                                "重复路径模式尚未实现".to_string()
                            ));
                        }
                    }
                }

                Ok(plan)
            }
            Pattern::Node(node) => {
                self.plan_pattern_node(node, space_id)
            }
            Pattern::Edge(edge) => {
                self.plan_pattern_edge(edge, space_id)
            }
            Pattern::Variable(_) => {
                Err(PlannerError::PlanGenerationFailed(
                    "变量模式尚未实现".to_string()
                ))
            }
        }
    }

    /// 规划模式节点
    fn plan_pattern_node(
        &self,
        node: &crate::query::parser::ast::pattern::NodePattern,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        // 创建节点扫描
        let scan_node = ScanVerticesNode::new(space_id);
        let mut plan = SubPlan::from_root(scan_node.into_enum());

        // 如果有标签过滤，添加过滤器
        if !node.labels.is_empty() {
            // TODO: 添加标签过滤节点
        }

        // 如果有属性过滤，添加过滤器
        if let Some(ref props) = node.properties {
            let filter_node = FilterNode::new(
                plan.root.as_ref().unwrap().clone(),
                props.clone(),
            ).map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
            plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
        }

        // 如果有谓词过滤，添加过滤器
        if !node.predicates.is_empty() {
            for pred in &node.predicates {
                let filter_node = FilterNode::new(
                    plan.root.as_ref().unwrap().clone(),
                    pred.clone(),
                ).map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
                plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
            }
        }

        Ok(plan)
    }

    /// 规划模式边
    fn plan_pattern_edge(
        &self,
        edge: &crate::query::parser::ast::pattern::EdgePattern,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        // 确定边方向
        let direction = match edge.direction {
            crate::query::parser::ast::types::EdgeDirection::Out => "out",
            crate::query::parser::ast::types::EdgeDirection::In => "in",
            crate::query::parser::ast::types::EdgeDirection::Both => "both",
        };

        // 创建边扩展节点
        let expand_node = ExpandAllNode::new(
            space_id,
            edge.edge_types.clone(),
            direction,
        );

        let mut plan = SubPlan::from_root(expand_node.into_enum());

        // 如果有属性过滤，添加过滤器
        if let Some(ref props) = edge.properties {
            let filter_node = FilterNode::new(
                plan.root.as_ref().unwrap().clone(),
                props.clone(),
            ).map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
            plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
        }

        // 如果有谓词过滤，添加过滤器
        if !edge.predicates.is_empty() {
            for pred in &edge.predicates {
                let filter_node = FilterNode::new(
                    plan.root.as_ref().unwrap().clone(),
                    pred.clone(),
                ).map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
                plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
            }
        }

        Ok(plan)
    }

    /// 交叉连接两个计划
    fn cross_join_plans(
        &self,
        left: SubPlan,
        right: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        use crate::query::planner::plan::core::nodes::CrossJoinNode;

        let left_root = match left.root {
            Some(ref r) => r,
            None => return Ok(right),
        };

        let right_root = match right.root {
            Some(ref r) => r,
            None => return Ok(left),
        };

        let join_node = CrossJoinNode::new(
            left_root.clone(),
            right_root.clone(),
        ).map_err(|e| PlannerError::JoinFailed(format!("交叉连接失败: {}", e)))?;

        Ok(SubPlan {
            root: Some(join_node.into_enum()),
            tail: left.tail.or(right.tail),
        })
    }

    /// 连接两个节点计划（基于别名）
    fn join_node_plans(
        &self,
        left: SubPlan,
        right: SubPlan,
        _left_alias: &str,
        _right_alias: &Option<String>,
    ) -> Result<SubPlan, PlannerError> {
        // TODO: 实现基于别名的连接逻辑
        // 目前使用交叉连接作为默认实现
        self.cross_join_plans(left, right)
    }

    fn plan_node_pattern(&self, space_id: u64) -> Result<SubPlan, PlannerError> {
        let scan_node = ScanVerticesNode::new(space_id);
        Ok(SubPlan::from_root(scan_node.into_enum()))
    }

    fn plan_filter(
        &self,
        input_plan: SubPlan,
        condition: Expression,
        _space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let filter_node = FilterNode::new(input_node.clone(), condition)?;
        Ok(SubPlan::new(Some(filter_node.into_enum()), input_plan.tail))
    }

    fn plan_project(
        &self,
        input_plan: SubPlan,
        columns: Vec<YieldColumn>,
        _space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let project_node = ProjectNode::new(input_node.clone(), columns)?;
        Ok(SubPlan::new(Some(project_node.into_enum()), input_plan.tail))
    }

    fn plan_sort(
        &self,
        input_plan: SubPlan,
        order_by: Vec<OrderByItem>,
        _space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let sort_items: Vec<SortItem> = order_by
            .into_iter()
            .map(|item| {
                let column = self.expression_to_string(&item.expression);
                let direction = if item.desc { OrderDirection::Desc } else { OrderDirection::Asc };
                SortItem::new(column, direction)
            })
            .collect();

        let sort_node = SortNode::new(input_node.clone(), sort_items)?;
        Ok(SubPlan::new(Some(sort_node.into_enum()), input_plan.tail))
    }

    fn expression_to_string(&self, expr: &Expression) -> String {
        expr.to_expression_string()
    }

    fn plan_limit(
        &self,
        input_plan: SubPlan,
        pagination: PaginationInfo,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan.root().as_ref().ok_or_else(|| {
            PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string())
        })?;

        let limit_node = LimitNode::new(input_node.clone(), pagination.skip as i64, pagination.limit as i64)?;
        let limit_node_enum = limit_node.into_enum();
        Ok(SubPlan::new(Some(limit_node_enum), input_plan.tail))
    }

    fn extract_where_condition(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<Expression>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                Ok(match_stmt.where_clause.clone())
            }
            _ => Ok(None),
        }
    }

    fn extract_return_columns(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<Vec<YieldColumn>>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                if let Some(return_clause) = &match_stmt.return_clause {
                    let mut columns = Vec::new();
                    for item in &return_clause.items {
                        match item {
                            crate::query::parser::ast::stmt::ReturnItem::Expression { expression, alias } => {
                                columns.push(YieldColumn {
                                    expression: expression.clone(),
                                    alias: alias.clone().unwrap_or_default(),
                                    is_matched: false,
                                });
                            }
                            crate::query::parser::ast::stmt::ReturnItem::All => {
                                columns.push(YieldColumn {
                                    expression: crate::core::Expression::Variable("*".to_string()),
                                    alias: "*".to_string(),
                                    is_matched: false,
                                });
                            }
                        }
                    }
                    if columns.is_empty() {
                        columns.push(YieldColumn {
                            expression: crate::core::Expression::Variable("*".to_string()),
                            alias: "*".to_string(),
                            is_matched: false,
                        });
                    }
                    Ok(Some(columns))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn extract_order_by(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<Vec<OrderByItem>>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                if let Some(order_by_clause) = &match_stmt.order_by {
                    let items = order_by_clause.items.iter().map(|item| {
                        OrderByItem {
                            expression: item.expression.clone(),
                            desc: item.direction == crate::query::parser::ast::types::OrderDirection::Desc,
                        }
                    }).collect();
                    Ok(Some(items))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn extract_pagination(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<PaginationInfo>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                let skip = match_stmt.skip.unwrap_or(0);
                let limit = match_stmt.limit.unwrap_or(self.config.default_limit);
                Ok(Some(PaginationInfo { skip, limit }))
            }
            _ => Ok(None),
        }
    }
}
