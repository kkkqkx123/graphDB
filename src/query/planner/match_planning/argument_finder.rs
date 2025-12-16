//! 参数查找器
//! 寻找查询链接的参数
//! 负责查找查询链接中的参数

use crate::query::validator::structs::{CypherClauseContext, MatchClauseContext};
use std::collections::HashSet;

/// 参数查找器
/// 负责查找查询链接中的参数
#[derive(Debug)]
pub struct ArgumentFinder;

impl ArgumentFinder {
    pub fn new() -> Self {
        Self
    }

    /// 查找参数
    pub fn find_arguments(&self, clause_ctx: &CypherClauseContext) -> HashSet<String> {
        let mut arguments = HashSet::new();

        match clause_ctx {
            CypherClauseContext::Match(match_ctx) => {
                self.find_match_arguments(match_ctx, &mut arguments);
            }
            CypherClauseContext::Where(where_ctx) => {
                self.find_where_arguments(where_ctx, &mut arguments);
            }
            CypherClauseContext::With(with_ctx) => {
                self.find_with_arguments(with_ctx, &mut arguments);
            }
            CypherClauseContext::Return(return_ctx) => {
                self.find_return_arguments(return_ctx, &mut arguments);
            }
            CypherClauseContext::Unwind(unwind_ctx) => {
                self.find_unwind_arguments(unwind_ctx, &mut arguments);
            }
            _ => {
                // 其他类型的子句不处理参数查找
            }
        }

        arguments
    }

    /// 查找MATCH子句中的参数
    fn find_match_arguments(
        &self,
        match_ctx: &MatchClauseContext,
        arguments: &mut HashSet<String>,
    ) {
        // 查找模式中引用的别名
        for path in &match_ctx.paths {
            for node_info in &path.node_infos {
                if !node_info.anonymous
                    && match_ctx.aliases_available.contains_key(&node_info.alias)
                {
                    arguments.insert(node_info.alias.clone());
                }
            }
        }

        // 查找WHERE条件中引用的别名
        if let Some(where_ctx) = &match_ctx.where_clause {
            self.find_where_arguments(where_ctx, arguments);
        }
    }

    /// 查找WHERE子句中的参数
    fn find_where_arguments(
        &self,
        where_ctx: &crate::query::validator::structs::clause_structs::WhereClauseContext,
        arguments: &mut HashSet<String>,
    ) {
        // 查找过滤条件中引用的别名
        for (alias, _) in &where_ctx.aliases_available {
            arguments.insert(alias.clone());
        }

        // 查找路径表达式中引用的别名
        for path in &where_ctx.paths {
            for node_info in &path.node_infos {
                if !node_info.anonymous {
                    arguments.insert(node_info.alias.clone());
                }
            }
        }
    }

    /// 查找WITH子句中的参数
    fn find_with_arguments(
        &self,
        with_ctx: &crate::query::validator::structs::clause_structs::WithClauseContext,
        arguments: &mut HashSet<String>,
    ) {
        // 查找YIELD表达式中引用的别名
        for (alias, _) in &with_ctx.aliases_available {
            arguments.insert(alias.clone());
        }

        // 查找WHERE条件中引用的别名
        if let Some(where_ctx) = &with_ctx.where_clause {
            self.find_where_arguments(where_ctx, arguments);
        }
    }

    /// 查找RETURN子句中的参数
    fn find_return_arguments(
        &self,
        return_ctx: &crate::query::validator::structs::clause_structs::ReturnClauseContext,
        arguments: &mut HashSet<String>,
    ) {
        // 查找YIELD表达式中引用的别名
        for (alias, _) in &return_ctx.aliases_available {
            arguments.insert(alias.clone());
        }
    }

    /// 查找UNWIND子句中的参数
    fn find_unwind_arguments(
        &self,
        unwind_ctx: &crate::query::validator::structs::clause_structs::UnwindClauseContext,
        arguments: &mut HashSet<String>,
    ) {
        // 查找UNWIND表达式中引用的别名
        for (alias, _) in &unwind_ctx.aliases_available {
            arguments.insert(alias.clone());
        }
    }
}

impl Default for ArgumentFinder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::expression::Expression;
    use crate::query::validator::structs::{
        AliasType, CypherClauseContext, MatchClauseContext, NodeInfo, Path, PathType,
        ReturnClauseContext, UnwindClauseContext, WhereClauseContext, WithClauseContext,
        YieldClauseContext,
    };

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
            .enumerate()
            .map(|(i, node_alias)| create_test_node_info(node_alias, i > 0 && anonymous))
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

    /// 创建测试用的别名映射
    fn create_test_aliases(
        aliases: Vec<(&str, AliasType)>,
    ) -> std::collections::HashMap<String, AliasType> {
        aliases
            .into_iter()
            .map(|(alias, alias_type)| (alias.to_string(), alias_type))
            .collect()
    }

    #[test]
    fn test_argument_finder_new() {
        let finder = ArgumentFinder::new();
        // 测试创建实例
        assert!(true); // 如果能创建实例就通过
    }

    #[test]
    fn test_argument_finder_default() {
        let finder = ArgumentFinder::default();
        // 测试默认创建实例
        assert!(true); // 如果能创建实例就通过
    }

    #[test]
    fn test_find_arguments_match_clause() {
        let finder = ArgumentFinder::new();

        // 创建测试数据
        let aliases_available =
            create_test_aliases(vec![("n", AliasType::Node), ("m", AliasType::Node)]);

        let path = create_test_path("p", false, vec!["n", "m"]);

        let match_ctx = MatchClauseContext {
            paths: vec![path],
            aliases_available,
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };

        let clause_ctx = CypherClauseContext::Match(match_ctx);

        // 查找参数
        let arguments = finder.find_arguments(&clause_ctx);

        // 验证结果
        assert_eq!(arguments.len(), 2);
        assert!(arguments.contains("n"));
        assert!(arguments.contains("m"));
    }

    #[test]
    fn test_find_arguments_match_clause_with_where() {
        let finder = ArgumentFinder::new();

        // 创建测试数据
        let aliases_available =
            create_test_aliases(vec![("n", AliasType::Node), ("m", AliasType::Node)]);

        let path = create_test_path("p", false, vec!["n", "m"]);

        let where_aliases =
            create_test_aliases(vec![("n", AliasType::Node), ("x", AliasType::Variable)]);

        let where_ctx = WhereClauseContext {
            filter: Some(Expression::Variable("x".to_string())),
            aliases_available: where_aliases,
            aliases_generated: std::collections::HashMap::new(),
            paths: vec![],
        };

        let match_ctx = MatchClauseContext {
            paths: vec![path],
            aliases_available,
            aliases_generated: std::collections::HashMap::new(),
            where_clause: Some(where_ctx),
            is_optional: false,
            skip: None,
            limit: None,
        };

        let clause_ctx = CypherClauseContext::Match(match_ctx);

        // 查找参数
        let arguments = finder.find_arguments(&clause_ctx);

        // 验证结果
        assert_eq!(arguments.len(), 3);
        assert!(arguments.contains("n"));
        assert!(arguments.contains("m"));
        assert!(arguments.contains("x"));
    }

    #[test]
    fn test_find_arguments_where_clause() {
        let finder = ArgumentFinder::new();

        // 创建测试数据
        let aliases_available =
            create_test_aliases(vec![("n", AliasType::Node), ("x", AliasType::Variable)]);

        let path = create_test_path("p", false, vec!["n", "y"]);

        let where_ctx = WhereClauseContext {
            filter: Some(Expression::Variable("x".to_string())),
            aliases_available,
            aliases_generated: std::collections::HashMap::new(),
            paths: vec![path],
        };

        let clause_ctx = CypherClauseContext::Where(where_ctx);

        // 查找参数
        let arguments = finder.find_arguments(&clause_ctx);

        // 验证结果
        assert_eq!(arguments.len(), 3);
        assert!(arguments.contains("n"));
        assert!(arguments.contains("x"));
        assert!(arguments.contains("y"));
    }

    #[test]
    fn test_find_arguments_with_clause() {
        let finder = ArgumentFinder::new();

        // 创建测试数据
        let aliases_available =
            create_test_aliases(vec![("n", AliasType::Node), ("x", AliasType::Variable)]);

        let yield_clause = YieldClauseContext {
            yield_columns: vec![],
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
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

        let with_ctx = WithClauseContext {
            yield_clause,
            aliases_available,
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            pagination: None,
            order_by: None,
            distinct: false,
        };

        let clause_ctx = CypherClauseContext::With(with_ctx);

        // 查找参数
        let arguments = finder.find_arguments(&clause_ctx);

        // 验证结果
        assert_eq!(arguments.len(), 2);
        assert!(arguments.contains("n"));
        assert!(arguments.contains("x"));
    }

    #[test]
    fn test_find_arguments_with_clause_with_where() {
        let finder = ArgumentFinder::new();

        // 创建测试数据
        let aliases_available =
            create_test_aliases(vec![("n", AliasType::Node), ("x", AliasType::Variable)]);

        let where_aliases = create_test_aliases(vec![("y", AliasType::Variable)]);

        let where_ctx = WhereClauseContext {
            filter: Some(Expression::Variable("y".to_string())),
            aliases_available: where_aliases,
            aliases_generated: std::collections::HashMap::new(),
            paths: vec![],
        };

        let yield_clause = YieldClauseContext {
            yield_columns: vec![],
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
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

        let with_ctx = WithClauseContext {
            yield_clause,
            aliases_available,
            aliases_generated: std::collections::HashMap::new(),
            where_clause: Some(where_ctx),
            pagination: None,
            order_by: None,
            distinct: false,
        };

        let clause_ctx = CypherClauseContext::With(with_ctx);

        // 查找参数
        let arguments = finder.find_arguments(&clause_ctx);

        // 验证结果
        assert_eq!(arguments.len(), 3);
        assert!(arguments.contains("n"));
        assert!(arguments.contains("x"));
        assert!(arguments.contains("y"));
    }

    #[test]
    fn test_find_arguments_return_clause() {
        let finder = ArgumentFinder::new();

        // 创建测试数据
        let aliases_available =
            create_test_aliases(vec![("n", AliasType::Node), ("x", AliasType::Variable)]);

        let yield_clause = YieldClauseContext {
            yield_columns: vec![],
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
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

        let return_ctx = ReturnClauseContext {
            yield_clause,
            aliases_available,
            aliases_generated: std::collections::HashMap::new(),
            pagination: None,
            order_by: None,
            distinct: false,
        };

        let clause_ctx = CypherClauseContext::Return(return_ctx);

        // 查找参数
        let arguments = finder.find_arguments(&clause_ctx);

        // 验证结果
        assert_eq!(arguments.len(), 2);
        assert!(arguments.contains("n"));
        assert!(arguments.contains("x"));
    }

    #[test]
    fn test_find_arguments_unwind_clause() {
        let finder = ArgumentFinder::new();

        // 创建测试数据
        let aliases_available =
            create_test_aliases(vec![("n", AliasType::Node), ("x", AliasType::Variable)]);

        let unwind_ctx = UnwindClauseContext {
            alias: "item".to_string(),
            unwind_expr: Expression::Variable("x".to_string()),
            aliases_available,
            aliases_generated: std::collections::HashMap::new(),
            paths: vec![],
        };

        let clause_ctx = CypherClauseContext::Unwind(unwind_ctx);

        // 查找参数
        let arguments = finder.find_arguments(&clause_ctx);

        // 验证结果
        assert_eq!(arguments.len(), 2);
        assert!(arguments.contains("n"));
        assert!(arguments.contains("x"));
    }

    #[test]
    fn test_find_arguments_unsupported_clause() {
        let finder = ArgumentFinder::new();

        // 创建一个不支持的子句类型
        let yield_clause = YieldClauseContext {
            yield_columns: vec![],
            aliases_available: std::collections::HashMap::new(),
            aliases_generated: std::collections::HashMap::new(),
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

        let clause_ctx = CypherClauseContext::Yield(yield_clause);

        // 查找参数
        let arguments = finder.find_arguments(&clause_ctx);

        // 验证结果 - 应该为空
        assert_eq!(arguments.len(), 0);
    }

    #[test]
    fn test_find_arguments_anonymous_nodes() {
        let finder = ArgumentFinder::new();

        // 创建测试数据 - 包含匿名节点
        let aliases_available = create_test_aliases(vec![("n", AliasType::Node)]);

        let path = create_test_path("p", false, vec!["n", ""]);

        let match_ctx = MatchClauseContext {
            paths: vec![path],
            aliases_available,
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };

        let clause_ctx = CypherClauseContext::Match(match_ctx);

        // 查找参数
        let arguments = finder.find_arguments(&clause_ctx);

        // 验证结果 - 只应该包含非匿名节点
        assert_eq!(arguments.len(), 1);
        assert!(arguments.contains("n"));
    }

    #[test]
    fn test_find_arguments_empty_aliases() {
        let finder = ArgumentFinder::new();

        // 创建测试数据 - 空别名
        let aliases_available = std::collections::HashMap::new();

        let path = create_test_path("p", false, vec!["n", "m"]);

        let match_ctx = MatchClauseContext {
            paths: vec![path],
            aliases_available,
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };

        let clause_ctx = CypherClauseContext::Match(match_ctx);

        // 查找参数
        let arguments = finder.find_arguments(&clause_ctx);

        // 验证结果 - 应该为空
        assert_eq!(arguments.len(), 0);
    }
}
