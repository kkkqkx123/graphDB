//! 参数查找器
//! 寻找查询链接的参数
//! 负责查找查询链接中的参数

use crate::query::validator::structs::{
    CypherClauseContext, MatchClauseContext,
};
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
    fn find_match_arguments(&self, match_ctx: &MatchClauseContext, arguments: &mut HashSet<String>) {
        // 查找模式中引用的别名
        for path in &match_ctx.paths {
            for node_info in &path.node_infos {
                if !node_info.anonymous && match_ctx.aliases_available.contains_key(&node_info.alias) {
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
    fn find_where_arguments(&self, where_ctx: &crate::query::validator::structs::clause_structs::WhereClauseContext, arguments: &mut HashSet<String>) {
        // 查找过滤条件中引用的别名
        for (alias, _) in &where_ctx.aliases_available {
            arguments.insert(alias.clone());
        }

        // 查找路径表达式中引用的别名
        for path in &where_ctx.paths {
            for node_info in &path.node_infos {
                if !node_info.anonymous && where_ctx.aliases_available.contains_key(&node_info.alias) {
                    arguments.insert(node_info.alias.clone());
                }
            }
        }
    }

    /// 查找WITH子句中的参数
    fn find_with_arguments(&self, with_ctx: &crate::query::validator::structs::clause_structs::WithClauseContext, arguments: &mut HashSet<String>) {
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
    fn find_return_arguments(&self, return_ctx: &crate::query::validator::structs::clause_structs::ReturnClauseContext, arguments: &mut HashSet<String>) {
        // 查找YIELD表达式中引用的别名
        for (alias, _) in &return_ctx.aliases_available {
            arguments.insert(alias.clone());
        }
    }

    /// 查找UNWIND子句中的参数
    fn find_unwind_arguments(&self, unwind_ctx: &crate::query::validator::structs::clause_structs::UnwindClauseContext, arguments: &mut HashSet<String>) {
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