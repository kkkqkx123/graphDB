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

use crate::core::types::{ContextualExpression, ExpressionContext};
use crate::core::SymbolTable;
use crate::query::QueryContext;
use crate::query::parser::ast::Stmt;
use crate::query::parser::ast::pattern::{Pattern, PathElement, RepetitionType};
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::filter_node::FilterNode;
use crate::query::planner::plan::core::nodes::plan_node_traits::PlanNode;
use crate::query::planner::plan::core::nodes::{LimitNode, ProjectNode, ScanVerticesNode, SortNode, SortItem, LeftJoinNode, UnionNode, LoopNode, ArgumentNode};
use crate::query::planner::plan::core::nodes::ExpandAllNode;
use crate::query::planner::planner::{Planner, PlannerError, ValidatedStatement};
use crate::query::planner::statements::statement_planner::StatementPlanner;
use crate::query::validator::ValidationInfo;
use crate::core::YieldColumn;
use crate::query::parser::OrderByItem;
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
        let sym_table = qctx.sym_table();

        // 使用验证信息进行优化规划
        let validation_info = &validated.validation_info;

        // 检查优化提示
        for hint in &validation_info.optimization_hints {
            log::debug!("优化提示: {:?}", hint);
        }

        // 使用别名映射优化规划
        self.plan_match_pattern(&validated.stmt, space_id, sym_table, Some(validation_info))
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
        sym_table: &SymbolTable,
        _validation_info: Option<&ValidationInfo>,
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
                    self.plan_path_pattern(first_pattern, space_id, sym_table)?
                };

                // 处理额外的路径模式（使用交叉连接）
                for pattern in match_stmt.patterns.iter().skip(1) {
                    let path_plan = self.plan_path_pattern(pattern, space_id, sym_table)?;
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
        sym_table: &SymbolTable,
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
                        PathElement::Alternative(patterns) => {
                            // 处理替代路径：将多个路径选项合并为并集
                            let alt_plan = self.plan_alternative_patterns(
                                patterns,
                                space_id,
                                prev_node_alias.as_deref(),
                                sym_table,
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
                            // 处理可选路径：使用左连接保留左侧所有数据
                            let opt_plan = self.plan_optional_element(
                                elem,
                                space_id,
                                prev_node_alias.as_deref(),
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
                            // 处理重复路径：使用循环节点实现可变长度路径
                            let rep_plan = self.plan_repeated_element(
                                elem,
                                *rep_type,
                                space_id,
                                prev_node_alias.as_deref(),
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
            // 非路径模式委托给 plan_pattern 处理
            _ => self.plan_pattern(pattern, space_id, sym_table),
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
            let label_filter = Self::build_label_filter_expression(&node.variable, &node.labels);
            let filter_node = FilterNode::new(
                plan.root.as_ref().expect("plan的root应该存在").clone(),
                label_filter,
            ).map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
            plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
        }

        // 如果有属性过滤，添加过滤器
        if let Some(ref props) = node.properties {
            let filter_node = FilterNode::new(
                plan.root.as_ref().expect("plan的root应该存在").clone(),
                props.clone(),
            ).map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
            plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
        }

        // 如果有谓词过滤，添加过滤器
        if !node.predicates.is_empty() {
            for pred in &node.predicates {
                let filter_node = FilterNode::new(
                    plan.root.as_ref().expect("plan的root应该存在").clone(),
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
                plan.root.as_ref().expect("plan的root应该存在").clone(),
                props.clone(),
            ).map_err(|e| PlannerError::PlanGenerationFailed(e.to_string()))?;
            plan = SubPlan::new(Some(filter_node.into_enum()), plan.tail);
        }

        // 如果有谓词过滤，添加过滤器
        if !edge.predicates.is_empty() {
            for pred in &edge.predicates {
                let filter_node = FilterNode::new(
                    plan.root.as_ref().expect("plan的root应该存在").clone(),
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
    ///
    /// 当存在前一个节点别名时，使用哈希内连接基于节点 ID 进行连接。
    /// 这用于处理路径模式中的连续节点，如 MATCH (a)-[]->(b) 中 a 和 b 的连接。
    fn join_node_plans(
        &self,
        left: SubPlan,
        right: SubPlan,
        left_alias: &str,
        right_alias: &Option<String>,
    ) -> Result<SubPlan, PlannerError> {
        use crate::query::planner::plan::core::nodes::HashInnerJoinNode;

        let left_root = match left.root {
            Some(ref r) => r,
            None => return Ok(right),
        };

        let right_root = match right.root {
            Some(ref r) => r,
            None => return Ok(left),
        };

        let ctx = Arc::new(ExpressionContext::new());

        // 构建哈希键和探测键表达式
        // 左表使用已存在的别名作为哈希键
        let hash_key_expr = crate::core::Expression::variable(left_alias);
        let hash_key_meta = crate::core::types::expression::ExpressionMeta::new(hash_key_expr);
        let hash_key_id = ctx.register_expression(hash_key_meta);
        let hash_keys = vec![ContextualExpression::new(hash_key_id, ctx.clone())];

        // 右表使用新节点的变量名或默认名称作为探测键
        let probe_alias = right_alias.as_deref().unwrap_or("n");
        let probe_key_expr = crate::core::Expression::variable(probe_alias);
        let probe_key_meta = crate::core::types::expression::ExpressionMeta::new(probe_key_expr);
        let probe_key_id = ctx.register_expression(probe_key_meta);
        let probe_keys = vec![ContextualExpression::new(probe_key_id, ctx)];

        // 创建哈希内连接节点
        let join_node = HashInnerJoinNode::new(
            left_root.clone(),
            right_root.clone(),
            hash_keys,
            probe_keys,
        ).map_err(|e| PlannerError::JoinFailed(format!("哈希内连接失败: {}", e)))?;

        Ok(SubPlan {
            root: Some(join_node.into_enum()),
            tail: left.tail.or(right.tail),
        })
    }

    fn plan_node_pattern(&self, space_id: u64) -> Result<SubPlan, PlannerError> {
        let scan_node = ScanVerticesNode::new(space_id);
        Ok(SubPlan::from_root(scan_node.into_enum()))
    }

    fn plan_filter(
        &self,
        input_plan: SubPlan,
        condition: ContextualExpression,
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
                let column = self.contextual_expression_to_string(&item.expression);
                SortItem::new(column, item.direction)
            })
            .collect();

        let sort_node = SortNode::new(input_node.clone(), sort_items)?;
        Ok(SubPlan::new(Some(sort_node.into_enum()), input_plan.tail))
    }

    fn contextual_expression_to_string(&self, expr: &ContextualExpression) -> String {
        if let Some(expr_meta) = expr.expression() {
            expr_meta.inner().to_expression_string()
        } else {
            String::new()
        }
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
                                let ctx = Arc::new(ExpressionContext::new());
                                let expr_meta = crate::core::types::expression::ExpressionMeta::new(
                                    crate::core::Expression::Variable("*".to_string())
                                );
                                let id = ctx.register_expression(expr_meta);
                                let ctx_expr = ContextualExpression::new(id, ctx);
                                columns.push(YieldColumn {
                                    expression: ctx_expr,
                                    alias: "*".to_string(),
                                    is_matched: false,
                                });
                            }
                        }
                    }
                    if columns.is_empty() {
                        let ctx = Arc::new(ExpressionContext::new());
                        let expr_meta = crate::core::types::expression::ExpressionMeta::new(
                            crate::core::Expression::Variable("*".to_string())
                        );
                        let id = ctx.register_expression(expr_meta);
                        let ctx_expr = ContextualExpression::new(id, ctx);
                        columns.push(YieldColumn {
                            expression: ctx_expr,
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

    /// 规划替代路径模式
    ///
    /// 将多个路径选项转换为并集操作
    /// 例如: (a)-[:KNOWS|WORKS_WITH]->(b) 表示 KNOWS 或 WORKS_WITH 两种关系
    fn plan_alternative_patterns(
        &self,
        patterns: &[Pattern],
        space_id: u64,
        _prev_alias: Option<&str>,
        sym_table: &SymbolTable,
    ) -> Result<SubPlan, PlannerError> {
        if patterns.is_empty() {
            return Err(PlannerError::PlanGenerationFailed(
                "替代路径不能为空".to_string()
            ));
        }

        // 规划第一个路径选项
        let mut plan = self.plan_pattern(&patterns[0], space_id, sym_table)?;

        // 将剩余路径选项通过并集合并
        for pattern in patterns.iter().skip(1) {
            let pattern_plan = self.plan_pattern(pattern, space_id, sym_table)?;
            plan = self.union_plans(plan, pattern_plan)?;
        }

        Ok(plan)
    }

    /// 规划单个模式（节点、边或路径）
    fn plan_pattern(
        &self,
        pattern: &Pattern,
        space_id: u64,
        sym_table: &SymbolTable,
    ) -> Result<SubPlan, PlannerError> {
        match pattern {
            Pattern::Node(node) => self.plan_pattern_node(node, space_id),
            Pattern::Edge(edge) => self.plan_pattern_edge(edge, space_id),
            Pattern::Path(_) => self.plan_path_pattern(pattern, space_id, sym_table),
            Pattern::Variable(var) => self.plan_variable_pattern(var, space_id, sym_table),
        }
    }

    /// 规划变量模式
    ///
    /// 变量模式引用之前定义的变量，使用 ArgumentNode 作为数据源
    /// 参考 nebula-graph 的 VariableVertexIdSeek 实现
    ///
    /// # 设计说明
    ///
    /// 变量模式用于引用之前 MATCH 子句中定义的变量，例如：
    /// ```cypher
    /// MATCH (a), a RETURN a
    /// ```
    /// 在这个例子中，第二个 `a` 是变量模式，引用第一个 `(a)` 定义的节点。
    ///
    /// # 执行流程
    ///
    /// 1. 创建 ArgumentNode 来标记需要从执行上下文中获取变量
    /// 2. 在执行阶段，ArgumentExecutor 从 ExecutionContext 中获取变量值
    /// 3. 如果变量不存在，返回执行错误
    fn plan_variable_pattern(
        &self,
        var: &crate::query::parser::ast::pattern::VariablePattern,
        _space_id: u64,
        sym_table: &SymbolTable,
    ) -> Result<SubPlan, PlannerError> {
        // 使用 SymbolTable 验证变量是否存在
        if !sym_table.has_variable(&var.name) {
            return Err(PlannerError::PlanGenerationFailed(
                format!("变量 '{}' 未定义", var.name)
            ));
        }

        // 获取变量信息（用于类型检查）
        let _var_info = sym_table.get_variable_info(&var.name)
            .ok_or_else(|| PlannerError::PlanGenerationFailed(
                format!("无法获取变量 '{}' 的信息", var.name)
            ))?;

        // 创建 ArgumentNode 来引用变量
        // ArgumentNode 表示从外部变量输入数据，用于子查询或模式引用
        let argument_node = ArgumentNode::new(0, &var.name);

        // 创建 SubPlan，只包含 ArgumentNode
        let sub_plan = SubPlan::from_root(argument_node.into_enum());

        Ok(sub_plan)
    }

    /// 合并两个计划为并集
    fn union_plans(
        &self,
        left: SubPlan,
        right: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let left_root = match left.root {
            Some(ref r) => r,
            None => return Ok(right),
        };

        let _right_root = match right.root {
            Some(ref r) => r,
            None => return Ok(left),
        };

        // 创建并集节点，去重
        let union_node = UnionNode::new(
            left_root.clone(),
            true, // distinct = true，去重
        ).map_err(|e| PlannerError::PlanGenerationFailed(format!("并集操作失败: {}", e)))?;

        Ok(SubPlan {
            root: Some(union_node.into_enum()),
            tail: left.tail.or(right.tail),
        })
    }

    /// 规划可选路径元素
    ///
    /// 使用左连接实现可选匹配，保留左侧所有数据
    /// 例如: (a)-[:KNOWS]->(b)? 表示 KNOWS 关系是可选的
    fn plan_optional_element(
        &self,
        element: &PathElement,
        space_id: u64,
        _prev_alias: Option<&str>,
    ) -> Result<SubPlan, PlannerError> {
        // 规划可选元素
        let opt_plan = match element {
            PathElement::Node(node) => self.plan_pattern_node(node, space_id)?,
            PathElement::Edge(edge) => self.plan_pattern_edge(edge, space_id)?,
            _ => {
                return Err(PlannerError::PlanGenerationFailed(
                    "可选路径不支持嵌套的复杂模式".to_string()
                ));
            }
        };

        Ok(opt_plan)
    }

    /// 左连接两个计划
    fn left_join_plans(
        &self,
        left: SubPlan,
        right: SubPlan,
    ) -> Result<SubPlan, PlannerError> {
        let left_root = match left.root {
            Some(ref r) => r,
            None => return Ok(right),
        };

        let right_root = match right.root {
            Some(ref r) => r,
            None => return Ok(left),
        };

        // 创建左连接节点
        let join_node = LeftJoinNode::new(
            left_root.clone(),
            right_root.clone(),
            vec![], // hash_keys
            vec![], // probe_keys
        ).map_err(|e| PlannerError::JoinFailed(format!("左连接失败: {}", e)))?;

        Ok(SubPlan {
            root: Some(join_node.into_enum()),
            tail: left.tail.or(right.tail),
        })
    }

    /// 规划重复路径元素
    ///
    /// 使用循环节点实现可变长度路径
    /// 例如: (a)-[:KNOWS*1..3]->(b) 表示 1 到 3 跳 KNOWS 关系
    fn plan_repeated_element(
        &self,
        element: &PathElement,
        rep_type: RepetitionType,
        space_id: u64,
        _prev_alias: Option<&str>,
    ) -> Result<SubPlan, PlannerError> {
        // 规划重复元素的基本计划
        let base_plan = match element {
            PathElement::Node(node) => self.plan_pattern_node(node, space_id)?,
            PathElement::Edge(edge) => self.plan_pattern_edge(edge, space_id)?,
            _ => {
                return Err(PlannerError::PlanGenerationFailed(
                    "重复路径不支持嵌套的复杂模式".to_string()
                ));
            }
        };

        // 根据重复类型确定循环条件
        let condition_str = match rep_type {
            RepetitionType::ZeroOrMore => "loop_count >= 0".to_string(),
            RepetitionType::OneOrMore => "loop_count >= 1".to_string(),
            RepetitionType::ZeroOrOne => "loop_count <= 1".to_string(),
            RepetitionType::Exactly(n) => format!("loop_count == {}", n),
            RepetitionType::Range(min, max) => format!("loop_count >= {} && loop_count <= {}", min, max),
        };

        // 创建循环节点
        use std::sync::Arc;
        let ctx = Arc::new(crate::core::types::ExpressionContext::new());
        let mut loop_node = LoopNode::from_string(-1, condition_str, ctx);

        // 设置循环体
        if let Some(base_root) = base_plan.root {
            loop_node.set_body(base_root);
        }

        Ok(SubPlan {
            root: Some(loop_node.into_enum()),
            tail: base_plan.tail,
        })
    }

    /// 构建标签过滤表达式
    ///
    /// 将节点标签列表转换为表达式，用于过滤具有指定标签的节点
    /// 例如: 标签 ["Person", "Actor"] 转换为: labels(n) CONTAINS "Person" AND labels(n) CONTAINS "Actor"
    fn build_label_filter_expression(
        variable: &Option<String>,
        labels: &[String],
    ) -> ContextualExpression {
        let var_name = variable.as_deref().unwrap_or("n");
        let var_expr = crate::core::Expression::variable(var_name);

        let ctx = Arc::new(ExpressionContext::new());

        // 创建 labels() 函数调用表达式
        let labels_func = crate::core::Expression::function("labels", vec![var_expr]);

        let expr = if labels.len() == 1 {
            // 单个标签: labels(n) CONTAINS "label"
            let label_literal = crate::core::Expression::literal(labels[0].clone());
            crate::core::Expression::function("contains", vec![labels_func, label_literal])
        } else {
            // 多个标签: labels(n) CONTAINS "label1" AND labels(n) CONTAINS "label2" AND ...
            let first_label = crate::core::Expression::literal(labels[0].clone());
            let first_condition = crate::core::Expression::function("contains", vec![labels_func.clone(), first_label]);

            labels.iter().skip(1).fold(first_condition, |acc, label| {
                let label_literal = crate::core::Expression::literal(label.clone());
                let condition = crate::core::Expression::function("contains", vec![labels_func.clone(), label_literal]);
                crate::core::Expression::binary(acc, crate::core::types::operators::BinaryOperator::And, condition)
            })
        };

        let expr_meta = crate::core::types::expression::ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        ContextualExpression::new(id, ctx)
    }
}
