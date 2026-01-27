//! 常用模式匹配逻辑
//! 提供可复用的模式匹配组件，简化规则实现

use super::plan::{MatchNode, Pattern};

/// 常用的模式匹配构建器
pub struct PatternBuilder;

impl PatternBuilder {
    /// 创建基本的单节点模式
    pub fn single(node_name: &'static str) -> Pattern {
        Pattern::new(node_name)
    }

    /// 创建多节点模式（匹配任一）
    pub fn multi(node_names: Vec<&'static str>) -> Pattern {
        Pattern::multi(node_names)
    }

    /// 创建带单个依赖的模式
    pub fn with_dependency(node_name: &'static str, dependency_name: &'static str) -> Pattern {
        Pattern::new(node_name).with_dependency(Pattern::new(dependency_name))
    }

    /// 创建带多个依赖的模式
    pub fn with_dependencies(
        node_name: &'static str,
        dependency_names: Vec<&'static str>,
    ) -> Pattern {
        let mut pattern = Pattern::new(node_name);
        for dep_name in dependency_names {
            pattern = pattern.with_dependency(Pattern::new(dep_name));
        }
        pattern
    }

    /// 创建过滤操作模式
    pub fn filter() -> Pattern {
        Pattern::new("Filter")
    }

    /// 创建过滤操作带特定依赖的模式
    pub fn filter_with(dependency_name: &'static str) -> Pattern {
        Self::with_dependency("Filter", dependency_name)
    }

    /// 创建限制操作模式
    pub fn limit() -> Pattern {
        Pattern::new("Limit")
    }

    /// 创建限制操作带特定依赖的模式
    pub fn limit_with(dependency_name: &'static str) -> Pattern {
        Self::with_dependency("Limit", dependency_name)
    }

    /// 创建投影操作模式
    pub fn project() -> Pattern {
        Pattern::new("Project")
    }

    /// 创建投影操作带特定依赖的模式
    pub fn project_with(dependency_name: &'static str) -> Pattern {
        Self::with_dependency("Project", dependency_name)
    }

    /// 创建扫描操作模式（顶点或边）
    pub fn scan() -> Pattern {
        Pattern::multi(vec!["ScanVertices", "ScanEdges"])
    }

    /// 创建索引扫描操作模式
    pub fn index_scan() -> Pattern {
        Pattern::new("IndexScan")
    }

    /// 创建连接操作模式
    pub fn join() -> Pattern {
        Pattern::multi(vec!["InnerJoin", "HashInnerJoin", "HashLeftJoin"])
    }

    /// 创建获取操作模式（顶点、边或邻居）
    pub fn get() -> Pattern {
        Pattern::multi(vec!["GetVertices", "GetEdges", "GetNeighbors"])
    }

    /// 创建去重操作模式
    pub fn dedup() -> Pattern {
        Pattern::new("Dedup")
    }

    /// 创建遍历操作模式
    pub fn traverse() -> Pattern {
        Pattern::new("Traverse")
    }

    /// 创建扩展操作模式
    pub fn expand() -> Pattern {
        Pattern::new("Expand")
    }

    /// 创建循环节点模式
    pub fn loop_pattern() -> Pattern {
        Pattern::multi(vec!["Loop", "ForLoop", "WhileLoop"])
    }

    /// 创建排序操作模式
    pub fn sort() -> Pattern {
        Pattern::new("Sort")
    }
}

/// 常用的模式组合
pub struct CommonPatterns;

impl CommonPatterns {
    /// 过滤后跟扫描的模式
    pub fn filter_over_scan() -> Pattern {
        PatternBuilder::filter_with("ScanVertices")
    }

    /// 过滤后跟索引扫描的模式
    pub fn filter_over_index_scan() -> Pattern {
        PatternBuilder::filter_with("IndexScan")
    }

    /// 过滤后跟遍历的模式
    pub fn filter_over_traverse() -> Pattern {
        PatternBuilder::filter_with("Traverse")
    }

    /// 过滤后跟连接的模式
    pub fn filter_over_join() -> Pattern {
        PatternBuilder::filter_with("InnerJoin")
    }

    /// 限制后跟扫描的模式
    pub fn limit_over_scan() -> Pattern {
        PatternBuilder::limit_with("ScanVertices")
    }

    /// 限制后跟获取的模式
    pub fn limit_over_get() -> Pattern {
        PatternBuilder::limit_with("GetVertices")
    }

    /// 过滤后跟过滤的模式
    pub fn filter_over_filter() -> Pattern {
        PatternBuilder::with_dependency("Filter", "Filter")
    }

    /// 投影后跟投影的模式
    pub fn project_over_project() -> Pattern {
        PatternBuilder::project_with("Project")
    }

    /// 获取后跟去重的模式
    pub fn get_over_dedup() -> Pattern {
        PatternBuilder::with_dependency("GetVertices", "Dedup")
    }

    /// 获取后跟投影的模式
    pub fn get_over_project() -> Pattern {
        PatternBuilder::with_dependency("GetVertices", "Project")
    }

    /// 排序后跟限制的模式（Top-N查询）
    pub fn sort_over_limit() -> Pattern {
        PatternBuilder::with_dependency("Sort", "Limit")
    }
}

/// 模式匹配辅助函数
pub struct PatternMatcher;

impl PatternMatcher {
    /// 检查是否为下推候选模式（过滤操作在数据访问操作之上）
    pub fn is_push_down_candidate(pattern: &Pattern) -> bool {
        match &pattern.node {
            MatchNode::Single("Filter") => true,
            MatchNode::Single("Limit") => true,
            MatchNode::Single("Project") => true,
            _ => false,
        }
    }

    /// 检查是否为合并候选模式（相同类型的连续操作）
    pub fn is_merge_candidate(pattern: &Pattern) -> bool {
        match &pattern.node {
            MatchNode::Single("Filter") => {
                // 检查依赖是否也是过滤操作
                !pattern.dependencies.is_empty()
                    && matches!(&pattern.dependencies[0].node, MatchNode::Single("Filter"))
            }
            MatchNode::Single("Project") => {
                // 检查依赖是否也是投影操作
                !pattern.dependencies.is_empty()
                    && matches!(&pattern.dependencies[0].node, MatchNode::Single("Project"))
            }
            _ => false,
        }
    }

    /// 检查是否为消除候选模式（可能冗余的操作）
    pub fn is_elimination_candidate(pattern: &Pattern) -> bool {
        match &pattern.node {
            MatchNode::Single("Dedup") => true,
            MatchNode::Single("Filter") => true,
            MatchNode::Single("Project") => true,
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
