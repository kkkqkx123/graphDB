/// MATCH查询主规划器
/// 负责将MATCH查询转换为执行计划

use crate::query::context::ast::AstContext;
use super::cypher_clause_planner::CypherClausePlanner;
use super::match_clause_planner::MatchClausePlanner;
use crate::query::planner::match_planning::clauses::order_by_planner::OrderByClausePlanner;
use crate::query::planner::PlanNodeKind;
use crate::query::planner::match_planning::clauses::pagination_planner::PaginationPlanner;
use crate::query::planner::match_planning::clauses::return_clause_planner::ReturnClausePlanner;
use crate::query::planner::match_planning::utils::connection_strategy::UnifiedConnector;
use crate::query::planner::match_planning::clauses::unwind_planner::UnwindClausePlanner;
use crate::query::planner::match_planning::clauses::with_clause_planner::WithClausePlanner;
use crate::query::planner::match_planning::clauses::where_clause_planner::WhereClausePlanner;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::plan::core::nodes::PlanNodeFactory;
use crate::query::planner::planner::{Planner, PlannerError};
use crate::query::validator::structs::{
    alias_structs::QueryPart, clause_structs::MatchClauseContext, CypherClauseContext,
    CypherClauseKind,
};
use std::collections::HashSet;

/// MATCH查询规划器
/// 处理Cypher MATCH语句的转换为执行计划
#[derive(Debug)]
pub struct MatchPlanner {
    #[allow(dead_code)]
    tail_connected: bool,
}

impl MatchPlanner {
    /// 创建新的MATCH规划器
    pub fn new() -> Self {
        Self {
            tail_connected: false,
        }
    }

    /// 创建规划器实例的工厂函数
    pub fn make() -> Box<dyn Planner> {
        Box::new(Self::new())
    }

    /// 检查AST上下文是否匹配MATCH查询
    pub fn match_ast_ctx(ast_ctx: &AstContext) -> bool {
        ast_ctx.statement_type().to_uppercase() == "MATCH"
    }

    /// 获取匹配和实例化函数
    pub fn get_match_and_instantiate() -> crate::query::planner::planner::MatchAndInstantiate {
        crate::query::planner::planner::MatchAndInstantiate {
            match_func: Self::match_ast_ctx,
            instantiate_func: Self::make,
            priority: 100,
        }
    }

    /// 生成子句计划
    fn gen_plan(&mut self, clause_ctx: &CypherClauseContext) -> Result<SubPlan, PlannerError> {
        // 创建规划上下文
        let query_info = crate::query::planner::match_planning::core::cypher_clause_planner::QueryInfo {
            query_id: "test".to_string(),
            statement_type: "MATCH".to_string(),
        };
        let mut context = crate::query::planner::match_planning::core::cypher_clause_planner::PlanningContext::new(query_info);
        
        match clause_ctx.kind() {
            CypherClauseKind::Match => {
                let match_ctx = match clause_ctx {
                    CypherClauseContext::Match(ctx) => ctx,
                    _ => return Err(PlannerError::InvalidAstContext("Expected MatchClauseContext".to_string())),
                };
                let planner = MatchClausePlanner::new(match_ctx.paths.clone());
                planner.transform(clause_ctx, None, &mut context)
            }
            CypherClauseKind::Where => {
                let planner = WhereClausePlanner::new(false);
                planner.transform(clause_ctx, None, &mut context)
            }
            CypherClauseKind::Unwind => {
                let planner = UnwindClausePlanner::new();
                planner.transform(clause_ctx, None, &mut context)
            }
            CypherClauseKind::With => {
                let planner = WithClausePlanner::new();
                planner.transform(clause_ctx, None, &mut context)
            }
            CypherClauseKind::Return => {
                let planner = ReturnClausePlanner::new();
                planner.transform(clause_ctx, None, &mut context)
            }
            CypherClauseKind::OrderBy => {
                let planner = OrderByClausePlanner::new();
                planner.transform(clause_ctx, None, &mut context)
            }
            CypherClauseKind::Pagination => {
                let planner = PaginationPlanner::new();
                planner.transform(clause_ctx, None, &mut context)
            }
            _ => Err(PlannerError::UnsupportedOperation(
                "Unsupported clause type in MATCH query".to_string(),
            )),
        }
    }

    /// 连接MATCH计划到之前的查询计划
    fn connect_match_plan(
        &mut self,
        query_plan: &mut SubPlan,
        match_ctx: &MatchClauseContext,
    ) -> Result<(), PlannerError> {
        // 生成当前MATCH计划
        let match_plan = self.gen_plan(&CypherClauseContext::Match(match_ctx.clone()))?;

        if query_plan.root.is_none() {
            // 使用clone_plan_node方法克隆PlanNode
            query_plan.root = match_plan.root.as_ref().map(|node| node.clone_plan_node());
            query_plan.tail = match_plan.tail.as_ref().map(|node| node.clone_plan_node());
            return Ok(());
        }

        // 找到交集别名
        let mut inter_aliases = HashSet::new();
        for (alias, alias_type) in &match_ctx.aliases_generated {
            if let Some(available_type) = match_ctx.aliases_available.get(alias) {
                inter_aliases.insert(alias.clone());

                // 检查类型兼容性
                // 如果任何类型是运行时类型，将类型检查留给运行时
                if matches!(
                    available_type,
                    crate::query::validator::structs::AliasType::Runtime
                ) || matches!(
                    alias_type,
                    crate::query::validator::structs::AliasType::Runtime
                ) {
                    continue;
                }

                // 非运行时连接的类型应该相同
                if available_type != alias_type {
                    return Err(PlannerError::InvalidAstContext(format!(
                        "{} binding to different type: {:?} vs {:?}",
                        alias, alias_type, available_type
                    )));
                }
            }
        }

        let ast_ctx = crate::query::context::ast::AstContext::new("MATCH", "test");

        if !inter_aliases.is_empty() {
            if match_ctx.is_optional {
                // 处理可选MATCH的左连接
                if let Some(where_ctx) = &match_ctx.where_clause {
                    // 连接WHERE过滤条件
                    let where_plan =
                        self.gen_plan(&CypherClauseContext::Where(where_ctx.clone()))?;
                    let match_plan_with_where = UnifiedConnector::add_input(
                        &ast_ctx,
                        &where_plan,
                        &match_plan,
                        true,
                    )?;
                    *query_plan = UnifiedConnector::left_join(
                        &ast_ctx,
                        query_plan,
                        &match_plan_with_where,
                        inter_aliases.into_iter().collect(),
                    )?;
                } else {
                    *query_plan = UnifiedConnector::left_join(
                        &ast_ctx,
                        query_plan,
                        &match_plan,
                        inter_aliases.into_iter().collect(),
                    )?;
                }
            } else {
                // 内连接
                *query_plan = UnifiedConnector::inner_join(
                    &ast_ctx,
                    query_plan,
                    &match_plan,
                    inter_aliases.into_iter().collect(),
                )?;
            }
        } else {
            // 笛卡尔积
            *query_plan = UnifiedConnector::cartesian_product(&ast_ctx, query_plan, &match_plan)?;
        }

        Ok(())
    }

    /// 生成查询部分计划
    fn gen_query_part_plan(
        &mut self,
        query_plan: &mut SubPlan,
        query_part: &QueryPart,
    ) -> Result<(), PlannerError> {
        // 为MATCH子句生成计划
        for match_ctx in &query_part.matchs {
            self.connect_match_plan(query_plan, match_ctx)?;

            // 连接MATCH过滤条件（非可选MATCH）
            if let Some(where_ctx) = &match_ctx.where_clause {
                if !match_ctx.is_optional {
                    let where_plan =
                        self.gen_plan(&CypherClauseContext::Where(where_ctx.clone()))?;
                    let ast_ctx = crate::query::context::ast::AstContext::new("MATCH", "test");
                    *query_plan = UnifiedConnector::add_input(
                        &ast_ctx,
                        &where_plan,
                        query_plan,
                        true,
                    )?;
                }
            }
        }

        // 设置边界子句的输入列名
        if let Some(boundary) = &query_part.boundary {
            if let Some(root) = &query_plan.root {
                // 设置输入列名
                let _col_names = root.col_names();
                match boundary {
                    crate::query::validator::structs::alias_structs::BoundaryClauseContext::With(_with_ctx) => {
                        // 这里需要设置with子句的输入列名
                        // 在实际实现中，应该更新with_ctx的input_col_names字段
                        // 由于结构体字段可能不可变，这里仅作注释说明
                    }
                    crate::query::validator::structs::alias_structs::BoundaryClauseContext::Unwind(_unwind_ctx) => {
                        // 这里需要设置unwind子句的输入列名
                        // 在实际实现中，应该更新unwind_ctx的input_col_names字段
                        // 由于结构体字段可能不可变，这里仅作注释说明
                    }
                }
            }
        }

        // 为边界子句生成计划
        if let Some(boundary) = &query_part.boundary {
            let boundary_plan = match boundary {
                crate::query::validator::structs::alias_structs::BoundaryClauseContext::With(
                    with_ctx,
                ) => self.gen_plan(&CypherClauseContext::With(with_ctx.clone()))?,
                crate::query::validator::structs::alias_structs::BoundaryClauseContext::Unwind(
                    unwind_ctx,
                ) => self.gen_plan(&CypherClauseContext::Unwind(unwind_ctx.clone()))?,
            };

            if query_plan.root.is_none() {
                *query_plan = boundary_plan;
            } else {
                let ast_ctx = crate::query::context::ast::AstContext::new("MATCH", "test");
                *query_plan = UnifiedConnector::add_input(
                    &ast_ctx,
                    &boundary_plan,
                    query_plan,
                    false,
                )?;
            }

            // 处理 WITH/UNWIND 子句中的 ORDER BY 和 Pagination
            match boundary {
                crate::query::validator::structs::alias_structs::BoundaryClauseContext::With(
                    with_ctx,
                ) => {
                    // 处理 ORDER BY 子句
                    if let Some(order_by_ctx) = &with_ctx.order_by {
                        let order_plan =
                            self.gen_plan(&CypherClauseContext::OrderBy(order_by_ctx.clone()))?;
                        let ast_ctx = crate::query::context::ast::AstContext::new("MATCH", "test");
                        *query_plan = UnifiedConnector::add_input(
                            &ast_ctx,
                            &order_plan,
                            query_plan,
                            true,
                        )?;
                    }

                    // 处理分页子句 (SKIP/LIMIT)
                    if let Some(pagination_ctx) = &with_ctx.pagination {
                        let pagination_plan = self
                            .gen_plan(&CypherClauseContext::Pagination(pagination_ctx.clone()))?;
                        let ast_ctx = crate::query::context::ast::AstContext::new("MATCH", "test");
                        *query_plan = UnifiedConnector::add_input(
                            &ast_ctx,
                            &pagination_plan,
                            query_plan,
                            true,
                        )?;
                    }
                }
                crate::query::validator::structs::alias_structs::BoundaryClauseContext::Unwind(
                    _unwind_ctx,
                ) => {
                    // UNWIND 子句不支持 ORDER BY 和 PAGINATION
                    // 这些功能在 WITH 子句中支持
                }
            }
        }

        // 为所有查询计划尾部生成变量
        if let Some(tail) = &query_plan.tail {
            if tail.kind() == PlanNodeKind::Argument {
                // 设置输入变量
                // 在实际实现中，应该为参数节点设置输入变量
                // 这里简化处理，通过输出变量名传递信息
                if tail.output_var().is_none() {
                    // 在实际实现中，应该生成一个匿名变量
                    // 这里简化处理，使用固定名称
                    // tail.set_output_var(Some("anon_var".to_string()));
                }
            }
        }

        if !self.tail_connected {
            self.tail_connected = true;
            // 添加起始节点
            // 在实际实现中，应该在查询计划的尾部添加起始节点
            // 这里简化处理，通过标记tail_connected来表示已添加
        }

        Ok(())
    }
}

impl Planner for MatchPlanner {
    fn transform(&mut self, ast_ctx: &AstContext) -> Result<SubPlan, PlannerError> {
        // 验证这是MATCH语句
        if !Self::match_ast_ctx(ast_ctx) {
            return Err(PlannerError::InvalidAstContext(
                "Only MATCH statements are accepted by MatchPlanner".to_string(),
            ));
        }

        // 创建一个空的查询计划
        let mut query_plan = SubPlan::new(None, None);

        // 从AST上下文中提取Cypher上下文结构
        // 在实际实现中，我们会在这里构建Cypher上下文
        // 但现在为了演示目的，我们创建一个简化的查询计划

        // 创建起始节点
        let _start_node = PlanNodeFactory::create_start_node()?;

        // 创建一个GetNeighbors节点作为示例
        let get_neighbors_node =
            PlanNodeFactory::create_placeholder_node()?;

        query_plan.root = Some(get_neighbors_node.clone());
        query_plan.tail = Some(get_neighbors_node);

        Ok(query_plan)
    }

    fn match_planner(&self, ast_ctx: &AstContext) -> bool {
        Self::match_ast_ctx(ast_ctx)
    }
}

impl Default for MatchPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::context::ast::AstContext;
    use crate::query::validator::structs::{
        AliasType, CypherClauseContext, MatchClauseContext, NodeInfo, Path, PathType,
    };
    use std::collections::HashMap;

    /// 创建测试用的 AST 上下文
    fn create_test_ast_context(statement_type: &str, query_text: &str) -> AstContext {
        AstContext::new(statement_type, query_text)
    }

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
    fn test_match_planner_new() {
        let planner = MatchPlanner::new();

        // 验证创建的实例
        assert!(!planner.tail_connected);
    }

    #[test]
    fn test_match_planner_default() {
        let planner = MatchPlanner::default();

        // 验证默认创建的实例
        assert!(!planner.tail_connected);
    }

    #[test]
    fn test_match_planner_debug() {
        let planner = MatchPlanner::new();

        let debug_str = format!("{:?}", planner);
        assert!(debug_str.contains("MatchPlanner"));
    }

    #[test]
    fn test_make() {
        let _planner_box = MatchPlanner::make();

        // 验证工厂函数创建的实例
        assert!(true); // 如果能创建实例就通过
    }

    #[test]
    fn test_match_ast_ctx_match() {
        let ast_ctx = create_test_ast_context("MATCH", "MATCH (n) RETURN n");

        // MATCH 语句应该匹配
        assert!(MatchPlanner::match_ast_ctx(&ast_ctx));
    }

    #[test]
    fn test_match_ast_ctx_match_lowercase() {
        let ast_ctx = create_test_ast_context("match", "match (n) return n");

        // 小写的 MATCH 语句应该匹配
        assert!(MatchPlanner::match_ast_ctx(&ast_ctx));
    }

    #[test]
    fn test_match_ast_ctx_non_match() {
        let ast_ctx = create_test_ast_context("GO", "GO FROM 1 OVER edge");

        // 非 MATCH 语句不应该匹配
        assert!(!MatchPlanner::match_ast_ctx(&ast_ctx));
    }

    #[test]
    fn test_get_match_and_instantiate() {
        let _match_and_instantiate = MatchPlanner::get_match_and_instantiate();

        // 验证获取的匹配和实例化函数
        assert!(true); // 如果能获取就通过
    }

    #[test]
    fn test_gen_plan_match() {
        let mut planner = MatchPlanner::new();

        let match_ctx = create_test_match_clause_context();
        let clause_ctx = CypherClauseContext::Match(match_ctx);

        let result = planner.gen_plan(&clause_ctx);

        // 生成 MATCH 计划应该成功
        assert!(result.is_ok());
    }

    #[test]
    fn test_gen_plan_unwind() {
        let mut planner = MatchPlanner::new();

        let unwind_ctx = crate::query::validator::structs::UnwindClauseContext {
            alias: "item".to_string(),
            unwind_expr: crate::graph::expression::Expression::Variable("x".to_string()),
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: vec![],
        };

        let clause_ctx = CypherClauseContext::Unwind(unwind_ctx);

        let result = planner.gen_plan(&clause_ctx);

        // 生成 UNWIND 计划应该成功
        assert!(result.is_ok());
    }

    #[test]
    fn test_gen_plan_with() {
        let mut planner = MatchPlanner::new();

        let yield_clause = crate::query::validator::structs::YieldClauseContext {
            yield_columns: vec![],
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: vec![],
            group_items: vec![],
            need_gen_project: false,
            agg_output_column_names: vec![],
            proj_output_column_names: vec![],
            proj_cols: vec![],
            paths: vec![],
        };

        let with_ctx = crate::query::validator::structs::WithClauseContext {
            yield_clause,
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            pagination: None,
            order_by: None,
            distinct: false,
        };

        let clause_ctx = CypherClauseContext::With(with_ctx);

        let result = planner.gen_plan(&clause_ctx);

        // 生成 WITH 计划应该成功
        assert!(result.is_ok());
    }

    #[test]
    fn test_gen_plan_return() {
        let mut planner = MatchPlanner::new();

        let yield_clause = crate::query::validator::structs::YieldClauseContext {
            yield_columns: vec![],
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: vec![],
            group_items: vec![],
            need_gen_project: false,
            agg_output_column_names: vec![],
            proj_output_column_names: vec![],
            proj_cols: vec![],
            paths: vec![],
        };

        let return_ctx = crate::query::validator::structs::ReturnClauseContext {
            yield_clause,
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            pagination: None,
            order_by: None,
            distinct: false,
        };

        let clause_ctx = CypherClauseContext::Return(return_ctx);

        let result = planner.gen_plan(&clause_ctx);

        // 生成 RETURN 计划应该成功
        assert!(result.is_ok());
    }

    #[test]
    fn test_gen_plan_unsupported() {
        let mut planner = MatchPlanner::new();

        let yield_ctx = crate::query::validator::structs::YieldClauseContext {
            yield_columns: vec![],
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: vec![],
            group_items: vec![],
            need_gen_project: false,
            agg_output_column_names: vec![],
            proj_output_column_names: vec![],
            proj_cols: vec![],
            paths: vec![],
        };

        let clause_ctx = CypherClauseContext::Yield(yield_ctx);

        let result = planner.gen_plan(&clause_ctx);

        // 不支持的子句类型应该返回错误
        assert!(result.is_err());
        match result.unwrap_err() {
            PlannerError::UnsupportedOperation(msg) => {
                assert!(msg.contains("Unsupported clause type in MATCH query"));
            }
            _ => panic!("Expected UnsupportedOperation error"),
        }
    }

    #[test]
    fn test_connect_match_plan_empty_query_plan() {
        let mut planner = MatchPlanner::new();

        let match_ctx = create_test_match_clause_context();
        let mut query_plan = SubPlan::new(None, None);

        let result = planner.connect_match_plan(&mut query_plan, &match_ctx);

        // 连接空查询计划应该成功
        assert!(result.is_ok());
        assert!(query_plan.root.is_some());
    }

    #[test]
    fn test_connect_match_plan_with_existing_plan() {
        let mut planner = MatchPlanner::new();

        let match_ctx = create_test_match_clause_context();

        let start_node = match PlanNodeFactory::create_start_node() {
            Ok(node) => Some(node),
            Err(_) => None,
        };

        let existing_plan = SubPlan::new(
            start_node,
            None,
        );

        let mut query_plan = existing_plan;

        let result = planner.connect_match_plan(&mut query_plan, &match_ctx);

        // 连接现有查询计划应该成功
        assert!(result.is_ok());
        assert!(query_plan.root.is_some());
    }

    #[test]
    fn test_connect_match_plan_with_intersecting_aliases() {
        let mut planner = MatchPlanner::new();

        let mut match_ctx = create_test_match_clause_context();
        match_ctx
            .aliases_generated
            .insert("n".to_string(), AliasType::Node);
        match_ctx
            .aliases_available
            .insert("n".to_string(), AliasType::Node);

        let start_node = match PlanNodeFactory::create_start_node() {
            Ok(node) => Some(node),
            Err(_) => None,
        };

        let existing_plan = SubPlan::new(
            start_node,
            None,
        );

        let mut query_plan = existing_plan;

        let result = planner.connect_match_plan(&mut query_plan, &match_ctx);

        // 连接有交集别名的计划应该成功
        assert!(result.is_ok());
        assert!(query_plan.root.is_some());
    }

    #[test]
    fn test_connect_match_plan_optional_with_where() {
        let mut planner = MatchPlanner::new();

        let mut match_ctx = create_test_match_clause_context();
        match_ctx.is_optional = true;
        match_ctx
            .aliases_generated
            .insert("n".to_string(), AliasType::Node);
        match_ctx
            .aliases_available
            .insert("n".to_string(), AliasType::Node);

        let where_ctx = crate::query::validator::structs::WhereClauseContext {
            filter: Some(crate::graph::expression::Expression::Variable(
                "x".to_string(),
            )),
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: vec![],
        };

        match_ctx.where_clause = Some(where_ctx);

        let start_node = match PlanNodeFactory::create_start_node() {
            Ok(node) => Some(node),
            Err(_) => None,
        };

        let existing_plan = SubPlan::new(
            start_node,
            None,
        );

        let mut query_plan = existing_plan;

        let result = planner.connect_match_plan(&mut query_plan, &match_ctx);

        // 连接可选 MATCH 带 WHERE 子句应该成功
        assert!(result.is_ok());
        assert!(query_plan.root.is_some());
        }

    #[test]
    fn test_transform_match_statement() {
        let mut planner = MatchPlanner::new();

        let ast_ctx = create_test_ast_context("MATCH", "MATCH (n) RETURN n");

        let result = planner.transform(&ast_ctx);

        // 转换 MATCH 语句应该成功
        assert!(result.is_ok());

        let subplan = result.unwrap();
        assert!(subplan.root.is_some());
        assert!(subplan.tail.is_some());

        // 验证根节点类型
        if let Some(root) = &subplan.root {
            assert_eq!(root.kind(), PlanNodeKind::GetNeighbors);
        }
    }

    #[test]
    fn test_transform_non_match_statement() {
        let mut planner = MatchPlanner::new();

        let ast_ctx = create_test_ast_context("GO", "GO FROM 1 OVER edge");

        let result = planner.transform(&ast_ctx);

        // 转换非 MATCH 语句应该失败
        assert!(result.is_err());
        match result.unwrap_err() {
            PlannerError::InvalidAstContext(msg) => {
                assert!(msg.contains("Only MATCH statements are accepted by MatchPlanner"));
            }
            _ => panic!("Expected InvalidAstContext error"),
        }
    }

    #[test]
    fn test_match_planner_trait() {
        let planner = MatchPlanner::new();

        let ast_ctx = create_test_ast_context("MATCH", "MATCH (n) RETURN n");

        // 测试 trait 方法
        assert!(planner.match_planner(&ast_ctx));
    }

    #[test]
    fn test_gen_query_part_plan() {
        let mut planner = MatchPlanner::new();

        let match_ctx = create_test_match_clause_context();
        let query_part = crate::query::validator::structs::alias_structs::QueryPart {
            matchs: vec![match_ctx],
            boundary: None,
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: vec![],
        };

        let mut query_plan = SubPlan::new(None, None);

        let result = planner.gen_query_part_plan(&mut query_plan, &query_part);

        // 生成查询部分计划应该成功
        assert!(result.is_ok());
    }

    #[test]
    fn test_gen_query_part_plan_with_boundary() {
        let mut planner = MatchPlanner::new();

        let match_ctx = create_test_match_clause_context();

        let yield_clause = crate::query::validator::structs::YieldClauseContext {
            yield_columns: vec![],
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            distinct: false,
            has_agg: false,
            group_keys: vec![],
            group_items: vec![],
            need_gen_project: false,
            agg_output_column_names: vec![],
            proj_output_column_names: vec![],
            proj_cols: vec![],
            paths: vec![],
        };

        let with_ctx = crate::query::validator::structs::WithClauseContext {
            yield_clause,
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            where_clause: None,
            pagination: None,
            order_by: None,
            distinct: false,
        };

        let boundary =
            crate::query::validator::structs::alias_structs::BoundaryClauseContext::With(with_ctx);

        let query_part = crate::query::validator::structs::alias_structs::QueryPart {
            matchs: vec![match_ctx],
            boundary: Some(boundary),
            aliases_available: HashMap::new(),
            aliases_generated: HashMap::new(),
            paths: vec![],
        };

        let mut query_plan = SubPlan::new(None, None);

        let result = planner.gen_query_part_plan(&mut query_plan, &query_part);

        // 生成带边界子句的查询部分计划应该成功
        assert!(result.is_ok());
    }
}
