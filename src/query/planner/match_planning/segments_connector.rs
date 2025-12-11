//! 计划段连接器
//! 连接多个计划段形成完整的执行计划

use crate::query::planner::plan::{SubPlan, PlanNodeKind, BinaryInputNode};
use crate::query::planner::plan::PlanNode;
use std::collections::HashSet;

/// 计划段连接器
/// 负责将多个计划段连接成完整的执行计划
#[derive(Debug)]
pub struct SegmentsConnector;

impl SegmentsConnector {
    /// 创建新的段连接器
    pub fn new() -> Self {
        Self
    }

    /// 内连接两个计划
    pub fn inner_join(&self, left: SubPlan, right: SubPlan, intersected_aliases: HashSet<String>) -> SubPlan {
        if left.root.is_none() || right.root.is_none() {
            return if left.root.is_some() { left } else { right };
        }

        let left_root = left.root.unwrap();
        let right_root = right.root.unwrap();

        // 创建内连接节点
        let inner_join_node = Box::new(BinaryInputNode::new(
            PlanNodeKind::HashInnerJoin,
            left_root,
            right_root,
        ));

        // TODO: 设置连接键（hash keys 和 probe keys）
        // 这里需要根据 intersected_aliases 创建相应的表达式

        // 使用新创建的节点作为根节点和尾节点
        SubPlan::new(Some(inner_join_node.clone_plan_node()), Some(inner_join_node))
    }

    /// 左连接两个计划
    pub fn left_join(&self, left: SubPlan, right: SubPlan, intersected_aliases: HashSet<String>) -> SubPlan {
        if left.root.is_none() {
            return right;
        }
        if right.root.is_none() {
            return left;
        }

        let left_root = left.root.unwrap();
        let right_root = right.root.unwrap();

        // 创建左连接节点
        let left_join_node = Box::new(BinaryInputNode::new(
            PlanNodeKind::HashLeftJoin,
            left_root,
            right_root,
        ));

        // TODO: 设置连接键（hash keys 和 probe keys）
        // 这里需要根据 intersected_aliases 创建相应的表达式

        // 使用新创建的节点作为根节点和尾节点
        SubPlan::new(Some(left_join_node.clone_plan_node()), Some(left_join_node))
    }

    /// 笛卡尔积
    pub fn cartesian_product(&self, left: SubPlan, right: SubPlan) -> SubPlan {
        if left.root.is_none() || right.root.is_none() {
            return if left.root.is_some() { left } else { right };
        }

        let left_root = left.root.unwrap();
        let right_root = right.root.unwrap();

        // 创建笛卡尔积节点
        let cartesian_node = Box::new(BinaryInputNode::new(
            PlanNodeKind::CartesianProduct,
            left_root,
            right_root,
        ));

        // 使用新创建的节点作为根节点和尾节点
        SubPlan::new(Some(cartesian_node.clone_plan_node()), Some(cartesian_node))
    }

    /// 添加输入
    pub fn add_input(&self, left: SubPlan, right: SubPlan, copy_col_names: bool) -> SubPlan {
        if left.root.is_none() {
            return right;
        }

        // 使用引用避免移动值
        match (&left.root, &right.tail) {
            (Some(_), Some(_)) => {
                // TODO: 设置输入变量和列名
                // 这里需要根据具体情况设置依赖关系和变量
                SubPlan::new(left.root, right.tail)
            }
            _ => SubPlan::new(left.root, right.tail)
        }
    }

    /// 模式应用（用于模式谓词）
    pub fn pattern_apply(
        &self,
        left: SubPlan,
        right: SubPlan,
        path: &crate::query::validator::structs::Path,
    ) -> SubPlan {
        if left.root.is_none() || right.root.is_none() {
            return if left.root.is_some() { left } else { right };
        }

        let left_root = left.root.unwrap();
        let right_root = right.root.unwrap();

        // 创建模式应用节点
        let pattern_apply_node = Box::new(BinaryInputNode::new(
            PlanNodeKind::PatternApply,
            left_root,
            right_root,
        ));

        // TODO: 设置模式应用相关的参数

        SubPlan::new(Some(pattern_apply_node), None)
    }

    /// 卷起应用（用于路径收集）
    pub fn roll_up_apply(
        &self,
        left: SubPlan,
        right: SubPlan,
        path: &crate::query::validator::structs::Path,
    ) -> SubPlan {
        if left.root.is_none() || right.root.is_none() {
            return if left.root.is_some() { left } else { right };
        }

        let left_root = left.root.unwrap();
        let right_root = right.root.unwrap();

        // 创建卷起应用节点
        let roll_up_apply_node = Box::new(BinaryInputNode::new(
            PlanNodeKind::RollUpApply,
            left_root,
            right_root,
        ));

        // TODO: 设置卷起应用相关的参数

        SubPlan::new(Some(roll_up_apply_node), None)
    }

    /// 连接多个子计划段
    pub fn connect_segments(&self, segments: Vec<SubPlan>) -> SubPlan {
        if segments.is_empty() {
            return SubPlan::new(None, None);
        }

        let mut result = segments[0].clone();
        for i in 1..segments.len() {
            result = self.add_input(result, segments[i].clone(), false);
        }

        result
    }
}

impl Default for SegmentsConnector {
    fn default() -> Self {
        Self::new()
    }
}
