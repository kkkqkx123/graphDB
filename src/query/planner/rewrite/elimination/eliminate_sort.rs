//! 排序消除规则
//!
//! 启发式规则：当输入数据已经有序时，消除不必要的排序操作。
//! 这包括：
//! - 索引扫描返回的数据（按索引键有序）
//! - 已经排序的子查询结果
//! - 有序的数据源扫描
//!
//! # 转换示例
//!
//! Before:
//! ```text
//!   Sort(name ASC)
//!       |
//!   IndexScan(idx_name)  -- 索引已按 name 排序
//! ```
//!
//! After:
//! ```text
//!   IndexScan(idx_name)  -- 直接消除 Sort
//! ```
//!
//! # 注意
//!
//! 此规则是启发式规则，**不依赖代价计算**。
//! 只要检测到输入有序且与排序要求匹配，就直接消除排序。
//! 基于代价的 TopN 转换决策保留在 strategy::sort_elimination 模块中。

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::{SortNode, SortItem};
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};

/// 排序消除规则
///
/// 当检测到输入数据已经有序且满足排序要求时，消除 Sort 节点。
/// 这是启发式规则，不依赖代价计算。
#[derive(Debug)]
pub struct EliminateSortRule;

impl EliminateSortRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查是否可以消除排序
    ///
    /// 返回 true 如果：
    /// 1. 输入是索引扫描，且索引顺序与排序要求匹配
    /// 2. 输入已经是有序的（如来自另一个 Sort 或有序扫描）
    fn can_eliminate_sort(&self, sort_node: &SortNode, input: &PlanNodeEnum) -> bool {
        let sort_items = sort_node.sort_items();

        match input {
            // 索引扫描 - 检查索引是否匹配排序要求
            PlanNodeEnum::IndexScan(index_scan) => {
                // 获取索引的排序列
                // 简化处理：假设索引按第一个属性升序排列
                // 实际实现中应该从索引元数据获取排序信息
                let index_columns = vec![index_scan.index_name().to_string()];
                self.check_order_match(sort_items, &index_columns)
            }
            // 另一个 Sort 节点 - 检查排序键是否兼容
            PlanNodeEnum::Sort(inner_sort) => {
                self.check_sort_compatibility(sort_items, inner_sort.sort_items())
            }
            // TopN 节点 - 输出是有序的
            PlanNodeEnum::TopN(topn) => {
                self.check_sort_compatibility(sort_items, topn.sort_items())
            }
            // 其他情况 - 暂时不能消除
            _ => false,
        }
    }

    /// 检查排序要求与索引顺序是否匹配
    ///
    /// 简化版本：检查排序键是否是索引列的前缀
    fn check_order_match(&self, sort_items: &[SortItem], index_columns: &[String]) -> bool {
        if sort_items.is_empty() {
            return true;
        }

        // 检查排序键是否是索引列的前缀
        for (i, sort_item) in sort_items.iter().enumerate() {
            if i >= index_columns.len() {
                return false;
            }
            // 简化：只检查列名是否匹配，假设都是升序
            // 实际应该检查排序方向
            if sort_item.column != index_columns[i] {
                return false;
            }
        }

        true
    }

    /// 检查两个排序是否兼容
    ///
    /// 如果外层排序是内层排序的前缀，则可以消除外层排序
    fn check_sort_compatibility(
        &self,
        outer_items: &[SortItem],
        inner_items: &[SortItem],
    ) -> bool {
        if outer_items.is_empty() {
            return true;
        }

        // 外层排序必须是内层排序的前缀
        if outer_items.len() > inner_items.len() {
            return false;
        }

        for (i, outer_item) in outer_items.iter().enumerate() {
            let inner_item = &inner_items[i];
            // 列名和方向都必须匹配
            if outer_item.column != inner_item.column
                || outer_item.direction != inner_item.direction
            {
                return false;
            }
        }

        true
    }
}

impl Default for EliminateSortRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for EliminateSortRule {
    fn name(&self) -> &'static str {
        "EliminateSortRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Sort")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 获取 Sort 节点
        let sort_node = match node {
            PlanNodeEnum::Sort(n) => n,
            _ => return Ok(None),
        };

        // 获取输入节点
        let input = sort_node.input();

        // 检查是否可以消除排序
        if self.can_eliminate_sort(sort_node, input) {
            // 消除 Sort 节点，直接返回输入
            let mut result = TransformResult::new();
            result.new_nodes.push(input.clone());
            return Ok(Some(result));
        }

        Ok(None)
    }
}

impl EliminationRule for EliminateSortRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        let sort_node = match node {
            PlanNodeEnum::Sort(n) => n,
            _ => return false,
        };

        let input = sort_node.input();
        self.can_eliminate_sort(sort_node, input)
    }

    fn eliminate(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        let sort_node = match node {
            PlanNodeEnum::Sort(n) => n,
            _ => return Ok(None),
        };

        // 消除 Sort 节点，直接返回输入
        let input = sort_node.input();
        let mut result = TransformResult::new();
        result.new_nodes.push(input.clone());
        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::{
        SortItem, SortNode, StartNode,
    };
    use crate::core::types::graph_schema::OrderDirection;

    fn create_test_sort_node(input: PlanNodeEnum, columns: Vec<&str>) -> SortNode {
        let sort_items: Vec<SortItem> = columns
            .into_iter()
            .map(|c| SortItem::new(c.to_string(), OrderDirection::Asc))
            .collect();
        SortNode::new(input, sort_items).expect("Failed to create SortNode")
    }

    #[test]
    fn test_eliminate_sort_rule_name() {
        let rule = EliminateSortRule::new();
        assert_eq!(rule.name(), "EliminateSortRule");
    }

    #[test]
    fn test_eliminate_sort_rule_pattern() {
        let rule = EliminateSortRule::new();
        assert!(rule.pattern().matches(&PlanNodeEnum::Sort(
            SortNode::new(
                PlanNodeEnum::Start(StartNode::new()),
                vec![SortItem::asc("name".to_string())],
            ).expect("Failed to create SortNode")
        )));
    }

    #[test]
    fn test_check_sort_compatibility_exact_match() {
        let rule = EliminateSortRule::new();

        let outer = vec![
            SortItem::asc("name".to_string()),
            SortItem::asc("age".to_string()),
        ];
        let inner = vec![
            SortItem::asc("name".to_string()),
            SortItem::asc("age".to_string()),
        ];

        assert!(rule.check_sort_compatibility(&outer, &inner));
    }

    #[test]
    fn test_check_sort_compatibility_prefix_match() {
        let rule = EliminateSortRule::new();

        let outer = vec![SortItem::asc("name".to_string())];
        let inner = vec![
            SortItem::asc("name".to_string()),
            SortItem::asc("age".to_string()),
        ];

        assert!(rule.check_sort_compatibility(&outer, &inner));
    }

    #[test]
    fn test_check_sort_compatibility_direction_mismatch() {
        let rule = EliminateSortRule::new();

        let outer = vec![SortItem::desc("name".to_string())];
        let inner = vec![SortItem::asc("name".to_string())];

        assert!(!rule.check_sort_compatibility(&outer, &inner));
    }

    #[test]
    fn test_check_sort_compatibility_column_mismatch() {
        let rule = EliminateSortRule::new();

        let outer = vec![SortItem::asc("name".to_string())];
        let inner = vec![SortItem::asc("age".to_string())];

        assert!(!rule.check_sort_compatibility(&outer, &inner));
    }

    #[test]
    fn test_check_sort_compatibility_empty() {
        let rule = EliminateSortRule::new();

        let outer: Vec<SortItem> = vec![];
        let inner = vec![SortItem::asc("name".to_string())];

        assert!(rule.check_sort_compatibility(&outer, &inner));
    }
}
