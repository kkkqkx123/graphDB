//! 起始顶点ID查找器
//! 寻找查询的起始顶点ID
//! 负责在查询规划中找到起始顶点

use crate::query::validator::structs::{
    CypherClauseContext, MatchClauseContext,
    path_structs::NodeInfo,
};

/// 起始顶点ID查找器
/// 负责在查询规划中找到起始顶点
#[derive(Debug)]
pub struct StartVidFinder;

impl StartVidFinder {
    pub fn new() -> Self {
        Self
    }

    /// 查找起始顶点ID
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
        if let Some(where_ctx) = &match_ctx.where_clause {
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
    pub fn find_best_start_vid(&self, clause_ctx: &CypherClauseContext) -> Option<String> {
        let start_vids = self.find_start_vids(clause_ctx);
        
        if start_vids.is_empty() {
            return None;
        }

        // 返回第一个找到的起始顶点（可以扩展为更智能的选择算法）
        Some(start_vids[0].clone())
    }
}

impl Default for StartVidFinder {
    fn default() -> Self {
        Self::new()
    }
}