//! 查找器模块
//! 提供参数查找和起始顶点ID查找功能
//! 合并了原来的ArgumentFinder和StartVidFinder功能

use crate::query::validator::structs::{
    path_structs::NodeInfo, CypherClauseContext, MatchClauseContext,
};
use std::collections::HashSet;

/// 查找器
/// 提供参数查找和起始顶点ID查找功能
#[derive(Debug)]
pub struct Finder;

impl Finder {
    /// 创建新的查找器实例
    pub fn new() -> Self {
        Self
    }

    // ==================== 参数查找功能 ====================

    /// 查找参数
    ///
    /// 根据子句上下文查找所有相关的参数
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

    // ==================== 起始顶点ID查找功能 ====================

    /// 查找起始顶点ID
    ///
    /// 根据子句上下文查找所有可能的起始顶点
    pub fn find_start_vids(&self, clause_ctx: &CypherClauseContext) -> Vec<String> {
        let mut start_vids = Vec::new();

        match clause_ctx {
            CypherClauseContext::Match(match_ctx) => {
                self.find_match_start_vids(match_ctx, &mut start_vids);
            }
            _ => {
                // 其他类型的子句不处理起始顶点查找
            }
        }

        start_vids
    }

    /// 查找MATCH子句中的起始顶点
    fn find_match_start_vids(&self, match_ctx: &MatchClauseContext, start_vids: &mut Vec<String>) {
        for path in &match_ctx.paths {
            if let Some(first_node) = path.node_infos.first() {
                // 检查第一个节点是否有特定的ID或属性可以用于起始查找
                if self.is_good_start_node(first_node, match_ctx) {
                    start_vids.push(first_node.alias.clone());
                }
            }
        }
    }

    /// 判断节点是否适合作为起始节点
    fn is_good_start_node(&self, node_info: &NodeInfo, match_ctx: &MatchClauseContext) -> bool {
        // 如果节点有标签，通常是好的起始点
        if !node_info.labels.is_empty() {
            return true;
        }

        // 如果节点有属性过滤条件，可能是好的起始点
        if node_info.filter.is_some() || node_info.props.is_some() {
            return true;
        }

        // 如果节点在WHERE条件中被引用，可能是好的起始点
        if let Some(_where_ctx) = &match_ctx.where_clause {
            // 这里应该检查WHERE条件是否包含对该节点的引用
            // 简化处理：如果别名在可用别名中，认为可能被引用
            if match_ctx.aliases_available.contains_key(&node_info.alias) {
                return true;
            }
        }

        // 如果节点不是匿名的，通常是好的起始点
        !node_info.anonymous
    }

    /// 查找最优起始顶点
    ///
    /// 从所有可能的起始顶点中选择最优的一个
    pub fn find_best_start_vid(&self, clause_ctx: &CypherClauseContext) -> Option<String> {
        let start_vids = self.find_start_vids(clause_ctx);

        if start_vids.is_empty() {
            return None;
        }

        // 返回第一个找到的起始顶点（可以扩展为更智能的选择算法）
        Some(start_vids[0].clone())
    }

    // ==================== 组合查找功能 ====================

    /// 查找参数和起始顶点ID
    ///
    /// 同时查找参数和起始顶点ID，返回一个包含两者的结构
    pub fn find_arguments_and_start_vids(&self, clause_ctx: &CypherClauseContext) -> FinderResult {
        let arguments = self.find_arguments(clause_ctx);
        let start_vids = self.find_start_vids(clause_ctx);
        let best_start_vid = self.find_best_start_vid(clause_ctx);

        FinderResult {
            arguments,
            start_vids,
            best_start_vid,
        }
    }

    /// 检查节点是否既是参数又是起始顶点
    ///
    /// 返回既是参数又是起始顶点的节点列表
    pub fn find_argument_start_vids(&self, clause_ctx: &CypherClauseContext) -> Vec<String> {
        let arguments = self.find_arguments(clause_ctx);
        let start_vids = self.find_start_vids(clause_ctx);

        start_vids
            .into_iter()
            .filter(|vid| arguments.contains(vid))
            .collect()
    }
}

/// 查找结果
///
/// 包含参数查找和起始顶点ID查找的结果
#[derive(Debug, Clone)]
pub struct FinderResult {
    /// 找到的参数集合
    pub arguments: HashSet<String>,
    /// 找到的起始顶点ID列表
    pub start_vids: Vec<String>,
    /// 最优起始顶点ID
    pub best_start_vid: Option<String>,
}

impl FinderResult {
    /// 创建新的查找结果
    pub fn new(
        arguments: HashSet<String>,
        start_vids: Vec<String>,
        best_start_vid: Option<String>,
    ) -> Self {
        Self {
            arguments,
            start_vids,
            best_start_vid,
        }
    }

    /// 检查是否有任何结果
    pub fn has_results(&self) -> bool {
        !self.arguments.is_empty() || !self.start_vids.is_empty()
    }

    /// 获取既是参数又是起始顶点的节点
    pub fn get_argument_start_vids(&self) -> Vec<String> {
        self.start_vids
            .iter()
            .filter(|vid| self.arguments.contains(*vid))
            .cloned()
            .collect()
    }
}

impl Default for Finder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use crate::query::validator::structs::{
        AliasType, CypherClauseContext, MatchClauseContext, NodeInfo, Path, PathType,
    };

    /// 创建测试用的节点信息
    fn create_test_node_info(alias: &str, anonymous: bool) -> NodeInfo {
        NodeInfo {
            alias: alias.to_string(),
            labels: if anonymous {
                vec![]
            } else {
                vec!["Person".to_string()]
            },
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
    fn test_finder_new() {
        let finder = Finder::new();
        // 测试创建实例
        assert!(true); // 如果能创建实例就通过
    }

    #[test]
    fn test_finder_default() {
        let finder = Finder::default();
        // 测试默认创建实例
        assert!(true); // 如果能创建实例就通过
    }

    #[test]
    fn test_find_arguments_match_clause() {
        let finder = Finder::new();

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
    fn test_find_start_vids_match_clause() {
        let finder = Finder::new();

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

        // 查找起始顶点ID
        let start_vids = finder.find_start_vids(&clause_ctx);

        // 验证结果
        assert_eq!(start_vids.len(), 1);
        assert_eq!(start_vids[0], "n");
    }

    #[test]
    fn test_find_best_start_vid() {
        let finder = Finder::new();

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

        // 查找最优起始顶点ID
        let best_start_vid = finder.find_best_start_vid(&clause_ctx);

        // 验证结果
        assert!(best_start_vid.is_some());
        assert_eq!(best_start_vid.expect("best_start_vid should be Some"), "n");
    }

    #[test]
    fn test_find_arguments_and_start_vids() {
        let finder = Finder::new();

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

        // 查找参数和起始顶点ID
        let result = finder.find_arguments_and_start_vids(&clause_ctx);

        // 验证结果
        assert_eq!(result.arguments.len(), 2);
        assert!(result.arguments.contains("n"));
        assert!(result.arguments.contains("m"));
        assert_eq!(result.start_vids.len(), 1);
        assert_eq!(result.start_vids[0], "n");
        assert!(result.best_start_vid.is_some());
        assert_eq!(
            result
                .best_start_vid
                .expect("best_start_vid should be Some"),
            "n"
        );
    }

    #[test]
    fn test_find_argument_start_vids() {
        let finder = Finder::new();

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

        // 查找既是参数又是起始顶点的节点
        let argument_start_vids = finder.find_argument_start_vids(&clause_ctx);

        // 验证结果
        assert_eq!(argument_start_vids.len(), 1);
        assert_eq!(argument_start_vids[0], "n");
    }

    #[test]
    fn test_finder_result() {
        let mut arguments = HashSet::new();
        arguments.insert("n".to_string());
        arguments.insert("m".to_string());

        let start_vids = vec!["n".to_string(), "x".to_string()];
        let best_start_vid = Some("n".to_string());

        let result = FinderResult::new(
            arguments.clone(),
            start_vids.clone(),
            best_start_vid.clone(),
        );

        // 验证结果
        assert_eq!(result.arguments, arguments);
        assert_eq!(result.start_vids, start_vids);
        assert_eq!(result.best_start_vid, best_start_vid);
        assert!(result.has_results());

        let argument_start_vids = result.get_argument_start_vids();
        assert_eq!(argument_start_vids.len(), 1);
        assert_eq!(argument_start_vids[0], "n");
    }

    #[test]
    fn test_is_good_start_node() {
        let finder = Finder::new();

        // 创建测试数据
        let aliases_available =
            create_test_aliases(vec![("n", AliasType::Node), ("m", AliasType::Node)]);

        let match_ctx = MatchClauseContext {
            paths: vec![],
            aliases_available,
            aliases_generated: std::collections::HashMap::new(),
            where_clause: None,
            is_optional: false,
            skip: None,
            limit: None,
        };

        // 测试有标签的节点
        let node_with_labels = create_test_node_info("n", false);
        assert!(finder.is_good_start_node(&node_with_labels, &match_ctx));

        // 测试有过滤条件的节点
        let mut node_with_filter = create_test_node_info("n", false);
        node_with_filter.filter = Some(Expression::Variable("x".to_string()));
        assert!(finder.is_good_start_node(&node_with_filter, &match_ctx));

        // 测试匿名节点
        let anonymous_node = create_test_node_info("", true);
        assert!(!finder.is_good_start_node(&anonymous_node, &match_ctx));
    }
}
