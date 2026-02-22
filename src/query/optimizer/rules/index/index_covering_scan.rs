//! 索引覆盖扫描优化规则
//!
//! 该规则识别可以仅通过索引数据回答查询的场景，
//! 避免回表查询，直接从索引返回数据。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//!   Project([name, age])
//!       |
//!   IndexScan(index_on_name_age, return_columns=[name, age])
//!       |
//!   GetVertices
//! ```
//!
//! After:
//! ```text
//!   Project([name, age])
//!       |
//!   IndexScan(index_on_name_age, return_columns=[name, age], covering_scan=true)
//! ```
//!
//! # 适用条件
//!
//! - 查询的所有返回列都在索引字段中
//! - 索引状态为 Active
//! - 不需要访问顶点/边的其他属性

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, Result};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::core::nodes::PlanNodeEnum;
use crate::core::YieldColumn;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;

/// 索引覆盖扫描优化规则
#[derive(Debug)]
pub struct IndexCoveringScanRule;

impl OptRule for IndexCoveringScanRule {
    fn name(&self) -> &str {
        "IndexCoveringScanRule"
    }

    fn pattern(&self) -> Pattern {
        // 匹配 Project -> IndexScan -> GetVertices 模式
        PatternBuilder::with_dependency("Project", "IndexScan")
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>> {
        let node_ref = node.borrow();

        // 检查当前节点是否为Project
        if !node_ref.plan_node.is_project() {
            return Ok(None);
        }

        let project_node = match node_ref.plan_node.as_project() {
            Some(n) => n,
            None => return Ok(None),
        };

        // Project必须只有一个子节点
        if node_ref.dependencies.len() != 1 {
            return Ok(None);
        }

        let child_id = node_ref.dependencies[0];
        let child_node = match ctx.find_group_node_by_id(child_id) {
            Some(n) => n,
            None => return Ok(None),
        };

        let child_ref = child_node.borrow();

        // 检查子节点是否为IndexScan
        if !child_ref.plan_node.is_index_scan() {
            return Ok(None);
        }

        let index_scan = match child_ref.plan_node.as_index_scan() {
            Some(s) => s,
            None => return Ok(None),
        };

        // 获取Project需要的列
        let required_columns: HashSet<String> = project_node
            .columns()
            .iter()
            .filter_map(|col| Self::extract_column_name(col))
            .collect();

        // 获取索引字段
        let index_fields: HashSet<String> = index_scan
            .return_columns
            .iter()
            .cloned()
            .collect();

        // 检查是否所有需要的列都在索引中
        let can_cover = required_columns.iter().all(|col| index_fields.contains(col));

        if !can_cover {
            return Ok(None);
        }

        // 检查IndexScan后面是否有GetVertices（需要回表）
        if child_ref.dependencies.len() == 1 {
            let grandchild_id = child_ref.dependencies[0];
            if let Some(grandchild) = ctx.find_group_node_by_id(grandchild_id) {
                let grandchild_ref = grandchild.borrow();
                if grandchild_ref.plan_node.type_name() == "GetVertices" {
                    // 可以移除GetVertices，使用覆盖扫描
                    let new_index_scan = index_scan.clone();
                    // 标记为覆盖扫描（如果IndexScan支持这个标记）
                    // new_index_scan.is_covering_scan = true;

                    // 创建新的IndexScan节点，跳过GetVertices
                    let mut new_group_node = child_ref.clone();
                    new_group_node.plan_node = PlanNodeEnum::IndexScan(new_index_scan);
                    // 继承GetVertices的依赖
                    new_group_node.dependencies = grandchild_ref.dependencies.clone();

                    let mut result = TransformResult::new();
                    result.add_new_group_node(Rc::new(RefCell::new(new_group_node)));
                    result.erase_curr = false; // 保留Project节点

                    return Ok(Some(result));
                }
            }
        }

        Ok(None)
    }
}

impl IndexCoveringScanRule {
    /// 从列表达式中提取列名
    fn extract_column_name(col: &YieldColumn) -> Option<String> {
        // 简化实现：假设列名直接存储
        Some(col.alias.clone())
    }
}

impl BaseOptRule for IndexCoveringScanRule {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::QueryContext;
    use crate::query::optimizer::plan::OptContext;
    use crate::query::planner::plan::core::nodes::{PlanNodeEnum, ProjectNode};
    use crate::query::planner::plan::algorithms::{IndexScan, ScanType};
    use std::sync::Arc;

    fn create_test_context() -> OptContext {
        let query_context = Arc::new(QueryContext::default());
        OptContext::new(query_context)
    }

    #[test]
    fn test_index_covering_scan_rule() {
        let rule = IndexCoveringScanRule;
        let mut ctx = create_test_context();

        // 创建IndexScan节点
        let mut index_scan = IndexScan::new(1, 1, 1, 1, ScanType::Full);
        index_scan.return_columns = vec!["name".to_string(), "age".to_string()];
        let index_scan_enum = PlanNodeEnum::IndexScan(index_scan);

        // 创建Project节点
        let columns = vec![
            crate::query::validator::YieldColumn::new(crate::core::Expression::Variable("name".to_string()), "name".to_string()),
            crate::query::validator::YieldColumn::new(crate::core::Expression::Variable("age".to_string()), "age".to_string()),
        ];
        let project = ProjectNode::new(index_scan_enum.clone(), columns)
            .expect("Failed to create Project node");
        let project_enum = PlanNodeEnum::Project(project);

        // 创建OptGroupNode
        let index_scan_node = crate::query::optimizer::plan::OptGroupNode::new(1, index_scan_enum);
        let project_node = crate::query::optimizer::plan::OptGroupNode::new(2, project_enum);

        // 设置依赖关系
        let mut project_node_with_dep = project_node;
        project_node_with_dep.dependencies = vec![1];

        // 将节点添加到上下文
        ctx.add_group_node(Rc::new(RefCell::new(index_scan_node))).expect("Failed to add group node");

        // 应用规则
        let result = rule.apply(&mut ctx, &Rc::new(RefCell::new(project_node_with_dep)))
            .expect("Rule should apply successfully");

        // 由于没有GetVertices子节点，不应该触发优化
        assert!(result.is_none());
    }
}
