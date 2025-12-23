//! 常用模式匹配逻辑
//! 提供可复用的模式匹配组件，简化规则实现

use super::optimizer::{MatchNode, Pattern};

/// 常用的模式匹配构建器
pub struct PatternBuilder;

impl PatternBuilder {
    /// 创建基本的单节点模式
    pub fn single(kind: PlanNodeKind) -> Pattern {
        Pattern::new(kind)
    }

    /// 创建多节点模式（匹配任一）
    pub fn multi(kinds: Vec<PlanNodeKind>) -> Pattern {
        Pattern::multi(kinds)
    }

    /// 创建带单个依赖的模式
    pub fn with_dependency(kind: PlanNodeKind, dependency_kind: PlanNodeKind) -> Pattern {
        Pattern::new(kind).with_dependency(Pattern::new(dependency_kind))
    }

    /// 创建带多个依赖的模式
    pub fn with_dependencies(kind: PlanNodeKind, dependency_kinds: Vec<PlanNodeKind>) -> Pattern {
        let mut pattern = Pattern::new(kind);
        for dep_kind in dependency_kinds {
            pattern = pattern.with_dependency(Pattern::new(dep_kind));
        }
        pattern
    }

    /// 创建过滤操作模式
    pub fn filter() -> Pattern {
        Pattern::new(PlanNodeKind::Filter)
    }

    /// 创建过滤操作带特定依赖的模式
    pub fn filter_with(dependency_kind: PlanNodeKind) -> Pattern {
        Self::with_dependency(PlanNodeKind::Filter, dependency_kind)
    }

    /// 创建限制操作模式
    pub fn limit() -> Pattern {
        Pattern::new(PlanNodeKind::Limit)
    }

    /// 创建限制操作带特定依赖的模式
    pub fn limit_with(dependency_kind: PlanNodeKind) -> Pattern {
        Self::with_dependency(PlanNodeKind::Limit, dependency_kind)
    }

    /// 创建投影操作模式
    pub fn project() -> Pattern {
        Pattern::new(PlanNodeKind::Project)
    }

    /// 创建投影操作带特定依赖的模式
    pub fn project_with(dependency_kind: PlanNodeKind) -> Pattern {
        Self::with_dependency(PlanNodeKind::Project, dependency_kind)
    }

    /// 创建扫描操作模式（顶点或边）
    pub fn scan() -> Pattern {
        Pattern::multi(vec![PlanNodeKind::ScanVertices, PlanNodeKind::ScanEdges])
    }

    /// 创建索引扫描操作模式
    pub fn index_scan() -> Pattern {
        Pattern::new(PlanNodeKind::IndexScan)
    }

    /// 创建连接操作模式
    pub fn join() -> Pattern {
        Pattern::multi(vec![
            PlanNodeKind::InnerJoin,
            PlanNodeKind::HashInnerJoin,
            PlanNodeKind::HashLeftJoin,
        ])
    }

    /// 创建获取操作模式（顶点、边或邻居）
    pub fn get() -> Pattern {
        Pattern::multi(vec![
            PlanNodeKind::GetVertices,
            PlanNodeKind::GetEdges,
            PlanNodeKind::GetNeighbors,
        ])
    }

    /// 创建去重操作模式
    pub fn dedup() -> Pattern {
        Pattern::new(PlanNodeKind::Dedup)
    }

    /// 创建遍历操作模式
    pub fn traverse() -> Pattern {
        Pattern::new(PlanNodeKind::Traverse)
    }

    /// 创建扩展操作模式
    pub fn expand() -> Pattern {
        Pattern::new(PlanNodeKind::Expand)
    }

    /// 创建排序操作模式
    pub fn sort() -> Pattern {
        Pattern::new(PlanNodeKind::Sort)
    }
}

/// 常用的模式组合
pub struct CommonPatterns;

impl CommonPatterns {
    /// 过滤后跟扫描的模式
    pub fn filter_over_scan() -> Pattern {
        PatternBuilder::filter_with(PlanNodeKind::ScanVertices)
    }

    /// 过滤后跟索引扫描的模式
    pub fn filter_over_index_scan() -> Pattern {
        PatternBuilder::filter_with(PlanNodeKind::IndexScan)
    }

    /// 过滤后跟遍历的模式
    pub fn filter_over_traverse() -> Pattern {
        PatternBuilder::filter_with(PlanNodeKind::Traverse)
    }

    /// 过滤后跟连接的模式
    pub fn filter_over_join() -> Pattern {
        PatternBuilder::filter_with(PlanNodeKind::InnerJoin)
    }

    /// 限制后跟扫描的模式
    pub fn limit_over_scan() -> Pattern {
        PatternBuilder::limit_with(PlanNodeKind::ScanVertices)
    }

    /// 限制后跟获取的模式
    pub fn limit_over_get() -> Pattern {
        PatternBuilder::limit_with(PlanNodeKind::GetVertices)
    }

    /// 投影后跟投影的模式
    pub fn project_over_project() -> Pattern {
        PatternBuilder::project_with(PlanNodeKind::Project)
    }

    /// 获取后跟去重的模式
    pub fn get_over_dedup() -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::GetVertices, PlanNodeKind::Dedup)
    }

    /// 获取后跟投影的模式
    pub fn get_over_project() -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::GetVertices, PlanNodeKind::Project)
    }

    /// 排序后跟限制的模式（Top-N查询）
    pub fn sort_over_limit() -> Pattern {
        PatternBuilder::with_dependency(PlanNodeKind::Sort, PlanNodeKind::Limit)
    }
}

/// 模式匹配辅助函数
pub struct PatternMatcher;

impl PatternMatcher {
    /// 检查是否为下推候选模式（过滤操作在数据访问操作之上）
    pub fn is_push_down_candidate(pattern: &Pattern) -> bool {
        match &pattern.node {
            MatchNode::Single(PlanNodeKind::Filter) => true,
            MatchNode::Single(PlanNodeKind::Limit) => true,
            MatchNode::Single(PlanNodeKind::Project) => true,
            _ => false,
        }
    }

    /// 检查是否为合并候选模式（相同类型的连续操作）
    pub fn is_merge_candidate(pattern: &Pattern) -> bool {
        match &pattern.node {
            MatchNode::Single(PlanNodeKind::Filter) => {
                // 检查依赖是否也是过滤操作
                !pattern.dependencies.is_empty()
                    && matches!(
                        &pattern.dependencies[0].node,
                        MatchNode::Single(PlanNodeKind::Filter)
                    )
            }
            MatchNode::Single(PlanNodeKind::Project) => {
                // 检查依赖是否也是投影操作
                !pattern.dependencies.is_empty()
                    && matches!(
                        &pattern.dependencies[0].node,
                        MatchNode::Single(PlanNodeKind::Project)
                    )
            }
            _ => false,
        }
    }

    /// 检查是否为消除候选模式（可能冗余的操作）
    pub fn is_elimination_candidate(pattern: &Pattern) -> bool {
        match &pattern.node {
            MatchNode::Single(PlanNodeKind::Dedup) => true,
            MatchNode::Single(PlanNodeKind::Filter) => true,
            MatchNode::Single(PlanNodeKind::Project) => true,
            _ => false,
        }
    }
}

/// 模式验证器
pub struct PatternValidator;

impl PatternValidator {
    /// 验证模式是否有效
    pub fn validate(pattern: &Pattern) -> Result<(), String> {
        // 检查模式是否有有效的节点类型
        match &pattern.node {
            MatchNode::Single(_) => Ok(()),
            MatchNode::Multi(kinds) => {
                if kinds.is_empty() {
                    Err("Multi pattern must have at least one kind".to_string())
                } else {
                    Ok(())
                }
            }
        }
    }

    /// 验证模式是否适用于下推优化
    pub fn validate_for_push_down(pattern: &Pattern) -> Result<(), String> {
        Self::validate(pattern)?;

        if !PatternMatcher::is_push_down_candidate(pattern) {
            return Err("Pattern is not a push down candidate".to_string());
        }

        if pattern.dependencies.is_empty() {
            return Err("Push down pattern must have at least one dependency".to_string());
        }

        Ok(())
    }

    /// 验证模式是否适用于合并优化
    pub fn validate_for_merge(pattern: &Pattern) -> Result<(), String> {
        Self::validate(pattern)?;

        if !PatternMatcher::is_merge_candidate(pattern) {
            return Err("Pattern is not a merge candidate".to_string());
        }

        Ok(())
    }

    /// 验证模式是否适用于消除优化
    pub fn validate_for_elimination(pattern: &Pattern) -> Result<(), String> {
        Self::validate(pattern)?;

        if !PatternMatcher::is_elimination_candidate(pattern) {
            return Err("Pattern is not an elimination candidate".to_string());
        }

        Ok(())
    }
}
