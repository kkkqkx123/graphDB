//! 计划重写器实现
//!
//! 管理所有启发式重写规则，按顺序应用到计划树

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::rewrite_rule::{RewriteError, RewriteRule};

/// 计划重写器
///
/// 管理所有启发式重写规则，按顺序应用
pub struct PlanRewriter {
    rules: Vec<Box<dyn RewriteRule>>,
}

impl PlanRewriter {
    /// 创建新的计划重写器
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// 添加重写规则
    pub fn add_rule<R: RewriteRule + 'static>(&mut self, rule: R) {
        self.rules.push(Box::new(rule));
    }

    /// 应用所有重写规则
    ///
    /// 递归遍历计划树，对所有匹配的节点应用重写规则
    pub fn rewrite(&self, node: PlanNodeEnum) -> Result<PlanNodeEnum, RewriteError> {
        // 先递归重写子节点
        let mut node = self.rewrite_children(node)?;

        // 尝试应用所有规则
        for rule in &self.rules {
            if rule.matches(&node) {
                // 应用规则，如果成功则更新节点
                if let Some(new_node) = rule.apply(node)? {
                    node = new_node;
                } else {
                    // 规则不匹配，保持原节点继续下一个规则
                }
            }
        }

        Ok(node)
    }

    /// 递归重写子节点
    fn rewrite_children(&self, mut node: PlanNodeEnum) -> Result<PlanNodeEnum, RewriteError> {
        use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

        match &mut node {
            PlanNodeEnum::Filter(n) => {
                let input = n.input().clone_plan_node();
                let new_input = self.rewrite(input)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::Project(n) => {
                let input = n.input().clone_plan_node();
                let new_input = self.rewrite(input)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::HashInnerJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let new_left = self.rewrite(left)?;
                let new_right = self.rewrite(right)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            PlanNodeEnum::HashLeftJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let new_left = self.rewrite(left)?;
                let new_right = self.rewrite(right)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            PlanNodeEnum::CrossJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let new_left = self.rewrite(left)?;
                let new_right = self.rewrite(right)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            PlanNodeEnum::InnerJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let new_left = self.rewrite(left)?;
                let new_right = self.rewrite(right)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            PlanNodeEnum::LeftJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let new_left = self.rewrite(left)?;
                let new_right = self.rewrite(right)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            PlanNodeEnum::FullOuterJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let new_left = self.rewrite(left)?;
                let new_right = self.rewrite(right)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            PlanNodeEnum::Aggregate(n) => {
                let input = n.input().clone_plan_node();
                let new_input = self.rewrite(input)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::Sort(n) => {
                let input = n.input().clone_plan_node();
                let new_input = self.rewrite(input)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::Limit(n) => {
                let input = n.input().clone_plan_node();
                let new_input = self.rewrite(input)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::TopN(n) => {
                let input = n.input().clone_plan_node();
                let new_input = self.rewrite(input)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::Dedup(n) => {
                let input = n.input().clone_plan_node();
                let new_input = self.rewrite(input)?;
                n.set_input(new_input);
            }
            // Union 节点处理 - 使用 dependencies 方法
            PlanNodeEnum::Union(n) => {
                let deps: Vec<PlanNodeEnum> = n
                    .dependencies()
                    .iter()
                    .map(|dep| dep.clone_plan_node())
                    .collect();
                // TODO: UnionNode 需要支持设置新的 dependencies
                // 目前暂时不处理，等待 UnionNode 接口完善
                let _ = deps;
            }
            // 叶子节点无需处理
            _ => {}
        }

        Ok(node)
    }
}

impl Default for PlanRewriter {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建默认的计划重写器
///
/// 包含所有标准的启发式重写规则
pub fn create_default_rewriter() -> PlanRewriter {
    let mut rewriter = PlanRewriter::new();

    // 添加谓词下推规则
    // TODO: 实例化具体规则

    // 添加操作合并规则
    // TODO: 实例化具体规则

    // 添加投影下推规则
    // TODO: 实例化具体规则

    // 添加消除规则
    // TODO: 实例化具体规则

    // 添加LIMIT下推规则
    // TODO: 实例化具体规则

    // 添加聚合优化规则
    // TODO: 实例化具体规则

    rewriter
}
