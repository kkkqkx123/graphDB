//! Unified MATCH Statement Planner
//!
//! Implement the StatementPlanner interface to handle the complete planning of MATCH queries.
//! It integrates the following functions:
//!   - Node and edge pattern matching (supports multiple paths)
//!   - WHERE condition filtering
//!   - RETURN Projection
//!   - ORDER BY: Sorting
//!   - LIMIT/SKIP – Pagination options
//!   - Selection of intelligent scanning strategies (index scanning, attribute scanning, full table scanning)

use crate::core::types::ContextualExpression;
use crate::core::YieldColumn;
use crate::query::parser::ast::pattern::{PathElement, Pattern, RepetitionType};
use crate::query::parser::ast::Stmt;
use crate::query::parser::OrderByItem;
use crate::query::planning::plan::core::nodes::base::plan_node_traits::PlanNode;
use crate::query::planning::plan::core::nodes::operation::filter_node::FilterNode;
use crate::query::planning::plan::core::nodes::ExpandAllNode;
use crate::query::planning::plan::core::nodes::{
    ArgumentNode, LeftJoinNode, LimitNode, LoopNode, ProjectNode, ScanVerticesNode, SortItem,
    SortNode, UnionNode,
};
use crate::query::planning::plan::SubPlan;
use crate::query::planning::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::planning::statements::statement_planner::StatementPlanner;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::structs::CypherClauseKind;
use crate::query::validator::ValidationInfo;
use crate::query::QueryContext;
use std::sync::Arc;

/// Pagination Information Structure
#[derive(Debug, Clone)]
pub struct PaginationInfo {
    pub skip: usize,
    pub limit: usize,
}

/// MATCH Statement Planner
///
/// Responsible for converting MATCH queries into executable execution plans.
/// Implement the StatementPlanner interface to provide a unified planning entry point.
#[derive(Debug, Clone)]
pub struct MatchStatementPlanner {
    config: MatchPlannerConfig,
    expr_context: Option<Arc<ExpressionAnalysisContext>>,
}

#[derive(Debug, Clone)]
pub struct MatchPlannerConfig {
    pub default_limit: usize,
    pub max_limit: usize,
    pub enable_index_optimization: bool,
}

impl Default for MatchPlannerConfig {
    fn default() -> Self {
        Self {
            default_limit: 10000,
            max_limit: 100000,
            enable_index_optimization: true,
        }
    }
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
            expr_context: None,
        }
    }

    pub fn with_config(config: MatchPlannerConfig) -> Self {
        Self {
            config,
            expr_context: None,
        }
    }
}

impl Planner for MatchStatementPlanner {
    fn match_planner(&self, stmt: &Stmt) -> bool {
        matches!(stmt, Stmt::Match(_))
    }

    fn transform(
        &mut self,
        validated: &ValidatedStatement,
        qctx: Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let space_id = qctx.space_id().unwrap_or(1);
        let space_name = qctx.space_name().unwrap_or_else(|| "default".to_string());

        // Use the verification information to optimize the planning process.
        let validation_info = &validated.validation_info;

        // Set expr_context
        self.expr_context = Some(validated.ast.expr_context().clone());

        // Check the optimization suggestions.
        for hint in &validation_info.optimization_hints {
            log::debug!("优化提示: {:?}", hint);
        }

        // Optimize the planning using alias mapping.
        self.plan_match_pattern(validated, space_id, &space_name, validation_info, &qctx)
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
        validated: &ValidatedStatement,
        space_id: u64,
        space_name: &str,
        validation_info: &ValidationInfo,
        qctx: &Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let stmt = validated.stmt();
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                for hint in &validation_info.index_hints {
                    if hint.estimated_selectivity < 0.1 {
                        log::debug!("使用高选择性索引: {}", hint.index_name);
                    }
                }

                let referenced_tags = &validation_info.semantic_info.referenced_tags;
                if !referenced_tags.is_empty() {
                    log::debug!("引用的标签: {:?}", referenced_tags);
                }

                let mut plan = if match_stmt.patterns.is_empty() {
                    self.plan_node_pattern(space_id, space_name)?
                } else {
                    let first_pattern = &match_stmt.patterns[0];
                    self.plan_path_pattern(first_pattern, space_id, space_name, validation_info, qctx)?
                };

                for pattern in match_stmt.patterns.iter().skip(1) {
                    let path_plan =
                        self.plan_path_pattern(pattern, space_id, space_name, validation_info, qctx)?;
                    plan = self.cross_join_plans(plan, path_plan)?;
                }

                if let Some(condition) = self.extract_where_condition(stmt)? {
                    plan = self.plan_filter(plan, condition, space_id)?;
                }

                if let Some(columns) = self.extract_return_columns(stmt, qctx)? {
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
                "Expected MATCH statement".to_string(),
            )),
        }
    }

    /// Planning Path Mode
    fn plan_path_pattern(
        &self,
        pattern: &Pattern,
        space_id: u64,
        space_name: &str,
        validation_info: &ValidationInfo,
        qctx: &Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        match pattern {
            Pattern::Path(path) => {
                if path.elements.is_empty() {
                    return Err(PlannerError::PlanGenerationFailed("空路径模式".to_string()));
                }

                let mut plan = SubPlan::new(None, None);
                let mut prev_node_alias: Option<String> = None;

                for element in path.elements.iter() {
                    match element {
                        PathElement::Node(node) => {
                            let node_plan = self.plan_pattern_node(node, space_id, space_name)?;
                            eprintln!("[plan_path_pattern] Node plan root type: {:?}", node_plan.root.as_ref().map(|r| r.name()));

                            plan = if let Some(existing_root) = plan.root.take() {
                                eprintln!("[plan_path_pattern] existing_root type: {:?}, prev_node_alias: {:?}", existing_root.name(), prev_node_alias);
                                if let Some(ref alias) = prev_node_alias {
                                    eprintln!("[plan_path_pattern] Calling join_node_plans with left_alias: {}, right_alias: {:?}", alias, node.variable);
                                    self.join_node_plans(
                                        SubPlan::new(Some(existing_root), plan.tail),
                                        node_plan,
                                        alias,
                                        &node.variable,
                                        self.expr_context.as_ref().ok_or_else(|| {
                                            PlannerError::PlanGenerationFailed(
                                                "Expression context is unavailable".to_string(),
                                            )
                                        })?,
                                    )?
                                } else {
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
                            if prev_node_alias.is_none() {
                                return Err(PlannerError::PlanGenerationFailed(
                                    "边模式必须跟随节点模式".to_string(),
                                ));
                            }

                            let input_alias = prev_node_alias.as_deref().unwrap();

                            // Create edge expansion plan with proper input variable
                            let edge_plan = self.plan_pattern_edge_with_input(
                                edge,
                                space_id,
                                input_alias,
                            )?;

                            plan = if let Some(existing_root) = plan.root.take() {
                                // Use HashInnerJoin to connect the previous node with the edge expansion
                                // The previous node's alias should match with "src" column of ExpandAll
                                eprintln!("[plan_path_pattern] Edge: Using HashInnerJoin with left_alias: {}, right_alias: src", input_alias);
                                self.join_node_plans(
                                    SubPlan::new(Some(existing_root), plan.tail),
                                    edge_plan,
                                    input_alias,
                                    &Some("src".to_string()),
                                    self.expr_context.as_ref().ok_or_else(|| {
                                        PlannerError::PlanGenerationFailed(
                                            "Expression context is unavailable".to_string(),
                                        )
                                    })?,
                                )?
                            } else {
                                edge_plan
                            };

                            // After edge expansion, the next node should join with "dst" column
                            // which contains the destination vertex of the edge
                            prev_node_alias = Some("dst".to_string());
                        }
                        PathElement::Alternative(patterns) => {
                            let alt_plan = self.plan_alternative_patterns(
                                patterns,
                                space_id,
                                space_name,
                                prev_node_alias.as_deref(),
                                validation_info,
                                qctx,
                            )?;
                            plan = if let Some(existing_root) = plan.root.take() {
                                self.cross_join_plans(
                                    SubPlan::new(Some(existing_root), plan.tail),
                                    alt_plan,
                                )?
                            } else {
                                alt_plan
                            };
                        }
                        PathElement::Optional(elem) => {
                            let opt_plan = self.plan_optional_element(
                                elem,
                                space_id,
                                space_name,
                                prev_node_alias.as_deref(),
                                validation_info,
                                qctx,
                            )?;
                            plan = if let Some(existing_root) = plan.root.take() {
                                self.left_join_plans(
                                    SubPlan::new(Some(existing_root), plan.tail),
                                    opt_plan,
                                )?
                            } else {
                                opt_plan
                            };
                        }
                        PathElement::Repeated(elem, rep_type) => {
                            let rep_plan = self.plan_repeated_element(
                                elem,
                                *rep_type,
                                space_id,
                                space_name,
                                prev_node_alias.as_deref(),
                                validation_info,
                                self.expr_context.as_ref().ok_or_else(|| {
                                    PlannerError::PlanGenerationFailed(
                                        "Expression context is unavailable".to_string(),
                                    )
                                })?,
                            )?;
                            plan = if let Some(existing_root) = plan.root.take() {
                                self.cross_join_plans(
                                    SubPlan::new(Some(existing_root), plan.tail),
                                    rep_plan,
                                )?
                            } else {
                                rep_plan
                            };
                        }
                    }
                }

                Ok(plan)
            }
            _ => self.plan_pattern(pattern, space_id, space_name, validation_info, qctx),
        }
    }

    /// Planning Mode Node
    fn plan_pattern_node(
        &self,
        node: &crate::query::parser::ast::pattern::NodePattern,
        space_id: u64,
        space_name: &str,
    ) -> Result<SubPlan, PlannerError> {
        let mut scan_node = ScanVerticesNode::new(space_id, space_name);
        // Set the column name to the node variable name so that subsequent join operations can find the variable
        let var_name = node.variable.clone().unwrap_or_else(|| "n".to_string());
        scan_node.set_col_names(vec![var_name]);
        let mut plan = SubPlan::from_root(scan_node.into_enum());

        // If there is a label filtering option, please add the filter.
        if !node.labels.is_empty() {
            let expr_ctx = self
                .expr_context
                .as_ref()
                .expect("expr_context should be set");
            let label_filter =
                Self::build_label_filter_expression(&node.variable, &node.labels, expr_ctx);
            let root_node = plan.root.as_ref().expect("plan的root应该存在");
            eprintln!("[plan_pattern_node] Before FilterNode, root col_names: {:?}", root_node.col_names());
            let filter_node = FilterNode::new(
                root_node.clone(),
                label_filter,
            )
            .map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
            eprintln!("[plan_pattern_node] After FilterNode, filter col_names: {:?}", filter_node.col_names());
            plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
        }

        // If there is attribute filtering, add the filter.
        if let Some(ref props) = node.properties {
            let filter_node = FilterNode::new(
                plan.root.as_ref().expect("plan的root应该存在").clone(),
                props.clone(),
            )
            .map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
            plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
        }

        // If there is predicate filtering, add the filter.
        if !node.predicates.is_empty() {
            for pred in &node.predicates {
                let filter_node = FilterNode::new(
                    plan.root.as_ref().expect("plan的root应该存在").clone(),
                    pred.clone(),
                )
                .map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
                plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
            }
        }

        Ok(plan)
    }

    /// Planning mode sidebar
    fn plan_pattern_edge(
        &self,
        edge: &crate::query::parser::ast::pattern::EdgePattern,
        space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let direction = match edge.direction {
            crate::query::parser::ast::types::EdgeDirection::Out => "out",
            crate::query::parser::ast::types::EdgeDirection::In => "in",
            crate::query::parser::ast::types::EdgeDirection::Both => "both",
        };

        let edge_types = match &edge.edge_types {
            types if !types.is_empty() => types.clone(),
            _ => vec![],
        };

        let mut expand_node = ExpandAllNode::new(space_id, edge_types, direction);

        if edge.edge_types.is_empty() {
            expand_node.set_any_edge_type(true);
        }

        // Set the column name to the edge variable name so that subsequent join operations can find the variable
        let edge_var = edge.variable.clone().unwrap_or_else(|| "e".to_string());
        expand_node.set_col_names(vec![edge_var]);

        let mut plan = SubPlan::from_root(expand_node.into_enum());

        // If there is attribute filtering, add the filter.
        if let Some(ref props) = edge.properties {
            let filter_node = FilterNode::new(
                plan.root.as_ref().expect("plan的root应该存在").clone(),
                props.clone(),
            )
            .map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
            plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
        }

        // If there is predicate filtering, add the filter.
        if !edge.predicates.is_empty() {
            for pred in &edge.predicates {
                let filter_node = FilterNode::new(
                    plan.root.as_ref().expect("plan的root应该存在").clone(),
                    pred.clone(),
                )
                .map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
                plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
            }
        }

        Ok(plan)
    }

    /// Planning mode edge with input variable
    ///
    /// This method creates an edge expansion plan that takes the source node as input.
    /// The ExpandAll node will use the input variable to get the source vertices from ExecutionContext.
    fn plan_pattern_edge_with_input(
        &self,
        edge: &crate::query::parser::ast::pattern::EdgePattern,
        space_id: u64,
        input_var: &str,
    ) -> Result<SubPlan, PlannerError> {
        let direction = match edge.direction {
            crate::query::parser::ast::types::EdgeDirection::Out => "out",
            crate::query::parser::ast::types::EdgeDirection::In => "in",
            crate::query::parser::ast::types::EdgeDirection::Both => "both",
        };

        let edge_types = match &edge.edge_types {
            types if !types.is_empty() => types.clone(),
            _ => vec![],
        };

        let mut expand_node = ExpandAllNode::new(space_id, edge_types, direction);

        if edge.edge_types.is_empty() {
            expand_node.set_any_edge_type(true);
        }

        // Set the input variable so ExpandAll can get source vertices from ExecutionContext
        expand_node.set_input_var(input_var.to_string());

        // Set the column names to match ExpandAll's output format: ["src", "edge", "dst"]
        // This allows subsequent operations to access source vertex, edge, and destination vertex
        expand_node.set_col_names(vec!["src".to_string(), "edge".to_string(), "dst".to_string()]);

        let mut plan = SubPlan::from_root(expand_node.into_enum());

        // If there is attribute filtering, add the filter.
        if let Some(ref props) = edge.properties {
            let filter_node = FilterNode::new(
                plan.root.as_ref().expect("plan的root应该存在").clone(),
                props.clone(),
            )
            .map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
            plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
        }

        // If there is predicate filtering, add the filter.
        if !edge.predicates.is_empty() {
            for pred in &edge.predicates {
                let filter_node = FilterNode::new(
                    plan.root.as_ref().expect("plan的root应该存在").clone(),
                    pred.clone(),
                )
                .map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
                plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
            }
        }

        Ok(plan)
    }

    /// Interconnecting two plans
    fn cross_join_plans(&self, left: SubPlan, right: SubPlan) -> Result<SubPlan, PlannerError> {
        use crate::query::planning::plan::core::nodes::CrossJoinNode;

        let left_root = match left.root {
            Some(ref r) => r,
            None => return Ok(right),
        };

        let right_root = match right.root {
            Some(ref r) => r,
            None => return Ok(left),
        };

        // If right_root is ExpandAllNode, we need to set its input_var
        // But we don't know the CrossJoin node's ID yet, so we'll use a special marker
        // and update it later when we know the actual ID
        eprintln!("[cross_join_plans] right_root type: {:?}, is_expand_all: {}", right_root.name(), right_root.as_expand_all().is_some());
        let (right_root, needs_id_update) = if let Some(expand_all) = right_root.as_expand_all() {
            eprintln!("[cross_join_plans] ExpandAllNode found, input_var: {:?}", expand_all.get_input_var());
            if expand_all.get_input_var().is_none() {
                // Use a marker that indicates we need to update with the actual ID later
                let marker_var = "__CROSSJOIN_ID_MARKER__".to_string();
                eprintln!("[cross_join_plans] Setting input_var to marker");
                let mut new_expand = expand_all.clone();
                new_expand.set_input_var(marker_var);
                (new_expand.into_enum(), true)
            } else {
                (right_root.clone(), false)
            }
        } else {
            (right_root.clone(), false)
        };

        // Create the join node
        let mut join_node = CrossJoinNode::new(left_root.clone(), right_root.clone())
            .map_err(|e| PlannerError::JoinFailed(format!("Cross-connection failed: {}", e)))?;

        // If we used a marker, update it with the actual join node ID
        if needs_id_update {
            let join_id = join_node.id();
            let actual_var = format!("left_{}", join_id);
            eprintln!("[cross_join_plans] Updating marker to actual var: {}", actual_var);
            
            // Update the right child (ExpandAllNode) with the actual variable name
            if let Some(expand_all) = join_node.right_input().as_expand_all() {
                let mut new_expand = expand_all.clone();
                new_expand.set_input_var(actual_var);
                // Recreate the join node with the updated right child
                join_node = CrossJoinNode::new(left_root.clone(), new_expand.into_enum())
                    .map_err(|e| PlannerError::JoinFailed(format!("Cross-connection failed: {}", e)))?;
            }
        }

        // Set the output_var of the CrossJoinNode to match the left_var
        // This ensures that parent nodes (like HashInnerJoin) can find the result
        use crate::query::planning::plan::core::nodes::base::plan_node_traits::PlanNode;
        let output_var = if let Some(expand_all) = join_node.right_input().as_expand_all() {
            expand_all.get_input_var().map(|v| v.to_string())
        } else {
            None
        };
        
        if let Some(var) = output_var {
            eprintln!("[cross_join_plans] Setting CrossJoinNode output_var to: {}", var);
            join_node.set_output_var(var);
        }

        Ok(SubPlan {
            root: Some(join_node.into_enum()),
            tail: left.tail.or(right.tail),
        })
    }

    /// Plan to connect two nodes (based on aliases)
    ///
    /// When there is an alias for the previous node, a hash-based internal connection is used to establish the connection based on the node ID.
    /// 这用于处理路径模式中的连续节点，如 MATCH (a)-[]->(b) 中 a 和 b 的连接。
    fn join_node_plans(
        &self,
        mut left: SubPlan,
        mut right: SubPlan,
        left_alias: &str,
        right_alias: &Option<String>,
        expr_context: &Arc<ExpressionAnalysisContext>,
    ) -> Result<SubPlan, PlannerError> {
        use crate::query::planning::plan::core::nodes::HashInnerJoinNode;
        use crate::query::planning::plan::core::node_id_generator::next_node_id;

        // Take ownership of the root nodes
        let mut left_root = match left.root.take() {
            Some(r) => r,
            None => return Ok(right),
        };

        let mut right_root = match right.root.take() {
            Some(r) => r,
            None => return Ok(left),
        };

        let ctx = expr_context.clone();

        // Generate unique variable names for left and right inputs
        // These variable names will be used by extract_join_vars in join_builder.rs
        let join_id = next_node_id();
        let left_var = format!("left_{}", join_id);
        let right_var = format!("right_{}", join_id);
        
        eprintln!("[join_node_plans] Setting left_var: {}, right_var: {}", left_var, right_var);
        
        // Set output_var for left and right inputs so that extract_join_vars can find them
        // This ensures that when build_executor_chain stores the results, the variable names match
        left_root.set_output_var(left_var);
        right_root.set_output_var(right_var);

        // Constructing hash key and probe key expressions
        // The left table uses existing aliases as hash keys.
        let hash_key_expr = crate::core::Expression::variable(left_alias);
        let hash_key_meta = crate::core::types::expr::ExpressionMeta::new(hash_key_expr);
        let hash_key_id = ctx.register_expression(hash_key_meta);
        let hash_keys = vec![ContextualExpression::new(hash_key_id, ctx.clone())];

        // The right table uses the variable name of the new node or the default name as the detection key.
        let probe_alias = right_alias.as_deref().unwrap_or("n");
        let probe_key_expr = crate::core::Expression::variable(probe_alias);
        let probe_key_meta = crate::core::types::expr::ExpressionMeta::new(probe_key_expr);
        let probe_key_id = ctx.register_expression(probe_key_meta);
        let probe_keys = vec![ContextualExpression::new(probe_key_id, ctx)];

        // If right_root is a ScanVerticesNode and probe_alias is not the same as its column name,
        // we need to update the column name to match the probe_alias
        // This ensures that after join, the column name matches the variable name used in filter conditions
        if let Some(scan_node) = right_root.as_scan_vertices() {
            let right_col_names = scan_node.col_names();
            if right_col_names.len() == 1 && right_col_names[0] != probe_alias {
                eprintln!("[join_node_plans] Updating right column name from '{}' to '{}'", right_col_names[0], probe_alias);
                let mut new_scan = scan_node.clone();
                new_scan.set_col_names(vec![probe_alias.to_string()]);
                right_root = new_scan.into_enum();
            }
        }

        // Create a Hashne connection node with the roots that have output_var set
        let mut join_node =
            HashInnerJoinNode::new(left_root, right_root, hash_keys, probe_keys)
                .map_err(|e| {
                    PlannerError::JoinFailed(format!("Intra-hash connection failed: {}", e))
                })?;

        // Set output_var to help parent nodes find the result
        join_node.set_output_var(format!("join_result_{}", join_id));

        Ok(SubPlan {
            root: Some(join_node.into_enum()),
            tail: left.tail.or(right.tail),
        })
    }

    fn plan_node_pattern(&self, space_id: u64, space_name: &str) -> Result<SubPlan, PlannerError> {
        let scan_node = ScanVerticesNode::new(space_id, space_name);
        Ok(SubPlan::from_root(scan_node.into_enum()))
    }

    fn plan_filter(
        &self,
        input_plan: SubPlan,
        condition: ContextualExpression,
        _space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        let filter_node = FilterNode::new(input_node.clone(), condition)?;
        Ok(SubPlan::new(Some(filter_node.into_enum()), input_plan.tail))
    }

    fn plan_project(
        &self,
        input_plan: SubPlan,
        columns: Vec<YieldColumn>,
        _space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        let project_node = ProjectNode::new(input_node.clone(), columns)?;
        Ok(SubPlan::new(
            Some(project_node.into_enum()),
            input_plan.tail,
        ))
    }

    fn plan_sort(
        &self,
        input_plan: SubPlan,
        order_by: Vec<OrderByItem>,
        _space_id: u64,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        let sort_items: Vec<SortItem> = order_by
            .into_iter()
            .map(|item| {
                let column = self.contextual_expression_to_string(&item.expression);
                SortItem::new(column, item.direction)
            })
            .collect();

        let sort_node = SortNode::new(input_node.clone(), sort_items)?;
        Ok(SubPlan::new(Some(sort_node.into_enum()), input_plan.tail))
    }

    fn contextual_expression_to_string(&self, expr: &ContextualExpression) -> String {
        expr.to_expression_string()
    }

    fn plan_limit(
        &self,
        input_plan: SubPlan,
        pagination: PaginationInfo,
    ) -> Result<SubPlan, PlannerError> {
        let input_node = input_plan
            .root()
            .as_ref()
            .ok_or_else(|| PlannerError::PlanGenerationFailed("输入计划没有根节点".to_string()))?;

        let limit_node = LimitNode::new(
            input_node.clone(),
            pagination.skip as i64,
            pagination.limit as i64,
        )?;
        let limit_node_enum = limit_node.into_enum();
        Ok(SubPlan::new(Some(limit_node_enum), input_plan.tail))
    }

    fn extract_where_condition(
        &self,
        stmt: &crate::query::parser::ast::Stmt,
    ) -> Result<Option<ContextualExpression>, PlannerError> {
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
        _qctx: &Arc<QueryContext>,
    ) -> Result<Option<Vec<YieldColumn>>, PlannerError> {
        match stmt {
            crate::query::parser::ast::Stmt::Match(match_stmt) => {
                if let Some(return_clause) = &match_stmt.return_clause {
                    let mut columns = Vec::new();
                    for item in &return_clause.items {
                        match item {
                            crate::query::parser::ast::stmt::ReturnItem::Expression {
                                expression,
                                alias,
                            } => {
                                let col_alias = alias.clone().unwrap_or_else(|| {
                                    expression.to_expression_string()
                                });
                                columns.push(YieldColumn {
                                    expression: expression.clone(),
                                    alias: col_alias,
                                    is_matched: false,
                                });
                            }
                        }
                    }
                    if columns.is_empty() {
                        return Err(PlannerError::PlanGenerationFailed(
                            "RETURN 子句缺少返回项".to_string(),
                        ));
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
                    Ok(Some(order_by_clause.items.clone()))
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

    /// Planning Alternative Paths Pattern
    ///
    /// Convert multiple path options into a union operation.
    /// 例如: (a)-[:KNOWS|WORKS_WITH]->(b) 表示 KNOWS 或 WORKS_WITH 两种关系
    fn plan_alternative_patterns(
        &self,
        patterns: &[Pattern],
        space_id: u64,
        space_name: &str,
        _prev_alias: Option<&str>,
        validation_info: &ValidationInfo,
        qctx: &Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        if patterns.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "替代路径不能为空".to_string(),
            ));
        }

        let mut plan = self.plan_pattern(&patterns[0], space_id, space_name, validation_info, qctx)?;

        for pattern in patterns.iter().skip(1) {
            let pattern_plan = self.plan_pattern(pattern, space_id, space_name, validation_info, qctx)?;
            plan = self.union_plans(plan, pattern_plan)?;
        }

        Ok(plan)
    }

    /// Planning a single pattern (node, edge, or path)
    fn plan_pattern(
        &self,
        pattern: &Pattern,
        space_id: u64,
        space_name: &str,
        validation_info: &ValidationInfo,
        _qctx: &Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        match pattern {
            Pattern::Node(node) => self.plan_pattern_node(node, space_id, space_name),
            Pattern::Edge(edge) => self.plan_pattern_edge(edge, space_id),
            Pattern::Path(_) => self.plan_path_pattern(pattern, space_id, space_name, validation_info, _qctx),
            Pattern::Variable(var) => self.plan_variable_pattern(var, space_id, validation_info),
        }
    }

    /// Planning Variable Pattern
    ///
    /// The variable pattern references a previously defined variable, using an ArgumentNode as the data source.
    /// Refer to the implementation of VariableVertexIdSeek in nebula-graph.
    ///
    /// # Design Specifications
    ///
    /// The variable pattern is used to reference variables that were defined in a previous MATCH clause, for example:
    /// ```cypher
    /// MATCH (a), a RETURN a
    /// ```
    /// In this example, the second “a” represents a variable pattern that refers to the node defined by the first “(a)”.
    ///
    /// # Execution Process
    ///
    /// 1. Create an `ArgumentNode` to indicate that a variable needs to be retrieved from the execution context.
    /// 2. During the execution phase, the ArgumentExecutor retrieves the variable values from the ExecutionContext.
    /// 3. If the variable does not exist, return an execution error.
    fn plan_variable_pattern(
        &self,
        var: &crate::query::parser::ast::pattern::VariablePattern,
        _space_id: u64,
        validation_info: &ValidationInfo,
    ) -> Result<SubPlan, PlannerError> {
        // Use the alias_map of ValidationInfo to verify whether the variable exists.
        if !validation_info.alias_map.contains_key(&var.name) {
            return Err(PlannerError::PlanGenerationFailed(format!(
                "Variable '{}' undefined",
                var.name
            )));
        }

        // Create an ArgumentNode to reference the variable.
        // The `ArgumentNode` represents data input from external variables, which is used for subqueries or schema references.
        let argument_node = ArgumentNode::new(0, &var.name);

        // Create a SubPlan that contains only the ArgumentNode.
        let sub_plan = SubPlan::from_root(argument_node.into_enum());

        Ok(sub_plan)
    }

    /// Merge the two plans into a union.
    fn union_plans(&self, left: SubPlan, right: SubPlan) -> Result<SubPlan, PlannerError> {
        let left_root = match left.root {
            Some(ref r) => r,
            None => return Ok(right),
        };

        let _right_root = match right.root {
            Some(ref r) => r,
            None => return Ok(left),
        };

        // Create an union node to remove duplicates.
        let union_node = UnionNode::new(
            left_root.clone(),
            true, // `distinct = true` – to remove duplicates.
        )
        .map_err(|e| {
            PlannerError::PlanGenerationFailed(format!("Concatenation operation failed: {}", e))
        })?;

        Ok(SubPlan {
            root: Some(union_node.into_enum()),
            tail: left.tail.or(right.tail),
        })
    }

    /// Planning of optional path elements
    ///
    /// Use a left join to achieve an optional match, retaining all data from the left side.
    /// 例如: (a)-[:KNOWS]->(b)? 表示 KNOWS 关系是可选的
    fn plan_optional_element(
        &self,
        element: &PathElement,
        space_id: u64,
        space_name: &str,
        _prev_alias: Option<&str>,
        _validation_info: &ValidationInfo,
        _qctx: &Arc<QueryContext>,
    ) -> Result<SubPlan, PlannerError> {
        let opt_plan = match element {
            PathElement::Node(node) => self.plan_pattern_node(node, space_id, space_name)?,
            PathElement::Edge(edge) => self.plan_pattern_edge(edge, space_id)?,
            _ => {
                return Err(PlannerError::PlanGenerationFailed(
                    "可选路径不支持嵌套的复杂模式".to_string(),
                ));
            }
        };

        Ok(opt_plan)
    }

    /// The left join connects two plans.
    fn left_join_plans(&self, left: SubPlan, right: SubPlan) -> Result<SubPlan, PlannerError> {
        let left_root = match left.root {
            Some(ref r) => r,
            None => return Ok(right),
        };

        let right_root = match right.root {
            Some(ref r) => r,
            None => return Ok(left),
        };

        // Create a left join node.
        let join_node = LeftJoinNode::new(
            left_root.clone(),
            right_root.clone(),
            vec![], // hash_keys
            vec![], // probe_keys
        )
        .map_err(|e| PlannerError::JoinFailed(format!("Left connection failed: {}", e)))?;

        Ok(SubPlan {
            root: Some(join_node.into_enum()),
            tail: left.tail.or(right.tail),
        })
    }

    /// Planning for repeated path elements
    ///
    /// Implementing variable-length paths using loop nodes
    /// 例如: (a)-[:KNOWS*1..3]->(b) 表示 1 到 3 跳 KNOWS 关系
    fn plan_repeated_element(
        &self,
        element: &PathElement,
        rep_type: RepetitionType,
        space_id: u64,
        space_name: &str,
        _prev_alias: Option<&str>,
        _validation_info: &ValidationInfo,
        expr_context: &Arc<ExpressionAnalysisContext>,
    ) -> Result<SubPlan, PlannerError> {
        let base_plan = match element {
            PathElement::Node(node) => self.plan_pattern_node(node, space_id, space_name)?,
            PathElement::Edge(edge) => self.plan_pattern_edge(edge, space_id)?,
            _ => {
                return Err(PlannerError::PlanGenerationFailed(
                    "重复路径不支持嵌套的复杂模式".to_string(),
                ));
            }
        };

        // Determine the loop condition based on the type of repetition.
        let condition_str = match rep_type {
            RepetitionType::ZeroOrMore => "loop_count >= 0".to_string(),
            RepetitionType::OneOrMore => "loop_count >= 1".to_string(),
            RepetitionType::ZeroOrOne => "loop_count <= 1".to_string(),
            RepetitionType::Exactly(n) => format!("loop_count == {}", n),
            RepetitionType::Range(min, max) => {
                format!("loop_count >= {} && loop_count <= {}", min, max)
            }
        };

        // Create a loop condition expression
        let expr_meta = crate::core::types::expr::ExpressionMeta::new(
            crate::core::Expression::Variable(condition_str),
        );
        let id = expr_context.register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, expr_context.clone());

        // Create a loop node
        let mut loop_node = LoopNode::new(-1, ctx_expr);

        // Setting up the loop body
        if let Some(base_root) = base_plan.root {
            loop_node.set_body(base_root);
        }

        Ok(SubPlan {
            root: Some(loop_node.into_enum()),
            tail: base_plan.tail,
        })
    }

    /// Constructing tag filtering expressions
    ///
    /// Convert the list of node labels into an expression that can be used to filter nodes with the specified labels.
    /// 例如: 标签 ["Person", "Actor"] 转换为: labels(n) CONTAINS "Person" AND labels(n) CONTAINS "Actor"
    fn build_label_filter_expression(
        variable: &Option<String>,
        labels: &[String],
        expr_context: &Arc<ExpressionAnalysisContext>,
    ) -> ContextualExpression {
        let var_name = variable.as_deref().unwrap_or("n");
        let var_expr = crate::core::Expression::variable(var_name);

        let ctx = expr_context.clone();

        // 创建 labels() 函数调用表达式
        let labels_func = crate::core::Expression::function("labels", vec![var_expr]);

        let expr = if labels.len() == 1 {
            // 单个标签: labels(n) CONTAINS "label"
            let label_literal = crate::core::Expression::literal(labels[0].clone());
            crate::core::Expression::function("contains", vec![labels_func, label_literal])
        } else {
            // 多个标签: labels(n) CONTAINS "label1" AND labels(n) CONTAINS "label2" AND ...
            let first_label = crate::core::Expression::literal(labels[0].clone());
            let first_condition = crate::core::Expression::function(
                "contains",
                vec![labels_func.clone(), first_label],
            );

            labels.iter().skip(1).fold(first_condition, |acc, label| {
                let label_literal = crate::core::Expression::literal(label.clone());
                let condition = crate::core::Expression::function(
                    "contains",
                    vec![labels_func.clone(), label_literal],
                );
                crate::core::Expression::binary(
                    acc,
                    crate::core::types::operators::BinaryOperator::And,
                    condition,
                )
            })
        };

        let expr_meta = crate::core::types::expr::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        ContextualExpression::new(id, ctx)
    }
}
