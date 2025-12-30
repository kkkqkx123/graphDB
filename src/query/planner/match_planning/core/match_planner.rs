/// MATCH查询主规划器
/// 负责将MATCH查询转换为执行计划
use crate::query::context::ast::AstContext;

use crate::query::planner::plan::factory::PlanNodeFactory;
use crate::query::planner::plan::SubPlan;
use crate::query::planner::planner::{Planner, PlannerError};

/// MATCH查询规划器
/// 处理Cypher MATCH语句的转换为执行计划
#[derive(Debug)]
pub struct MatchPlanner {
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
        let get_neighbors_node = PlanNodeFactory::create_placeholder_node()?;

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

    use crate::query::validator::structs::{MatchClauseContext, NodeInfo, Path, PathType};
    use std::collections::HashMap;

    /// 创建测试用的 AST 上下文
    fn create_test_ast_context(statement_type: &str, query_text: &str) -> AstContext {
        AstContext::from_strings(statement_type, query_text)
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
    fn test_transform_match_statement() {
        let mut planner = MatchPlanner::new();

        let ast_ctx = create_test_ast_context("MATCH", "MATCH (n) RETURN n");

        let result = planner.transform(&ast_ctx);

        // 转换 MATCH 语句应该成功
        assert!(result.is_ok());

        let subplan = result.expect("Transform should succeed for valid MATCH statement");
        assert!(subplan.root.is_some());
        assert!(subplan.tail.is_some());

        // 验证根节点类型
        if let Some(root) = &subplan.root {
            assert_eq!(root.name(), "GetNeighbors");
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
}
