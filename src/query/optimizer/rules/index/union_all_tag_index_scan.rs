//! 边索引扫描的UNION ALL规则
//!
//! 该规则识别多个边索引扫描的UNION ALL操作，
//! 并尝试将其合并为更高效的索引扫描。

use crate::query::optimizer::plan::{OptContext, OptGroupNode, OptRule, Pattern, TransformResult, OptimizerError};
use crate::query::optimizer::rule_patterns::PatternBuilder;
use crate::query::optimizer::rule_traits::BaseOptRule;
use crate::query::planner::plan::algorithms::IndexScan;
use std::rc::Rc;
use std::cell::RefCell;

/// 边索引扫描的UNION ALL规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   UnionAll
///   /      \
/// EdgeIndexScan  EdgeIndexScan
/// (type1)       (type2)
/// ```
///
/// After:
/// ```text
///   EdgeIndexScan
///   (type1, type2)
/// ```
///
/// # 适用条件
///
/// - 多个边索引扫描通过UNION ALL连接
/// - 索引扫描的索引类型相同
/// - 可以合并索引扫描条件
#[derive(Debug)]
pub struct UnionAllTagIndexScanRule;

impl OptRule for UnionAllTagIndexScanRule {
    fn name(&self) -> &str {
        "UnionAllTagIndexScanRule"
    }

    fn apply(
        &self,
        ctx: &mut OptContext,
        group_node: &Rc<RefCell<OptGroupNode>>,
    ) -> Result<Option<TransformResult>, OptimizerError> {
        let node_ref = group_node.borrow();
        
        if !node_ref.plan_node.is_union() {
            return Ok(None);
        }

        if node_ref.dependencies.len() < 2 {
            return Ok(None);
        }

        let mut tag_index_scans = Vec::new();
        for &dep_id in &node_ref.dependencies {
            if let Some(dep_node) = ctx.find_group_node_by_id(dep_id) {
                let dep_node_ref = dep_node.borrow();
                if dep_node_ref.plan_node.is_index_scan() {
                    if let Some(index_scan) = dep_node_ref.plan_node.as_index_scan() {
                        if index_scan.is_tag_scan() {
                            tag_index_scans.push((dep_id, index_scan.clone()));
                        }
                    }
                }
            }
        }

        if tag_index_scans.len() < 2 {
            return Ok(None);
        }

        if !can_merge_tag_index_scans(&tag_index_scans) {
            return Ok(None);
        }

        let merged_scan = merge_tag_index_scans(&tag_index_scans);

        let mut merged_scan_group_node = node_ref.clone();
        merged_scan_group_node.plan_node = crate::query::planner::plan::PlanNodeEnum::IndexScan(merged_scan);
        merged_scan_group_node.dependencies = Vec::new();

        let mut result = TransformResult::new();
        result.add_new_group_node(Rc::new(RefCell::new(merged_scan_group_node)));
        
        Ok(Some(result))
    }

    fn pattern(&self) -> Pattern {
        PatternBuilder::union(vec!["UnionAll"])
    }
}

impl BaseOptRule for UnionAllTagIndexScanRule {}

/// 检查是否可以合并标签索引扫描
fn can_merge_tag_index_scans(scans: &[(usize, IndexScan)]) -> bool {
    if scans.is_empty() {
        return false;
    }

    let first_index_name = scans[0].1.index_name();
    scans.iter().all(|(_, scan)| scan.index_name() == *first_index_name)
}

/// 合并标签索引扫描
fn merge_tag_index_scans(scans: &[(usize, IndexScan)]) -> IndexScan {
    let mut merged = scans[0].1.clone();
    
    for (_, scan) in scans.iter().skip(1) {
        merged.scan_limits.extend(scan.scan_limits.clone());
    }

    merged
}
