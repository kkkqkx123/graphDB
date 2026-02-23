//! 移除无操作投影的规则
//!
//! 根据 nebula-graph 的参考实现，此规则检查 Project 节点是否只是简单地传递子节点的列，
//! 如果是，则可以移除 Project 节点。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//!   Project(v1, v2, v3)  // 列名和子节点输出列名相同
//!       |
//!   ScanVertices (输出 v1, v2, v3)
//! ```
//!
//! After:
//! ```text
//!   ScanVertices
//! ```
//!
//! # 适用条件
//!
//! - Project 节点的输出列与子节点的输出列完全相同
//! - Project 的列表达式为简单的属性引用（VarProperty 或 InputProperty）
//! - 子节点在允许列表中（某些节点类型不允许移除 Project）

use crate::core::Expression;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{RewriteRule, EliminationRule};
use std::collections::HashSet;

/// 移除无操作投影的规则
///
/// 当 Project 节点只是简单地传递子节点的列时，直接移除 Project 节点
#[derive(Debug)]
pub struct RemoveNoopProjectRule {
    /// 允许移除 Project 的子节点类型集合
    allowed_child_types: HashSet<&'static str>,
}

impl RemoveNoopProjectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        let mut allowed_child_types = HashSet::new();
        
        // 允许移除 Project 的子节点类型
        // 参考 nebula-graph 的 kQueries 集合
        allowed_child_types.insert("GetNeighbors");
        allowed_child_types.insert("GetVertices");
        allowed_child_types.insert("GetEdges");
        allowed_child_types.insert("Traverse");
        allowed_child_types.insert("AppendVertices");
        allowed_child_types.insert("IndexScan");
        allowed_child_types.insert("ScanVertices");
        allowed_child_types.insert("ScanEdges");
        allowed_child_types.insert("EdgeIndexScan");
        allowed_child_types.insert("Union");
        allowed_child_types.insert("Project");
        allowed_child_types.insert("Unwind");
        allowed_child_types.insert("Sort");
        allowed_child_types.insert("TopN");
        allowed_child_types.insert("Sample");
        allowed_child_types.insert("Aggregate");
        allowed_child_types.insert("Assign");
        allowed_child_types.insert("InnerJoin");
        allowed_child_types.insert("HashInnerJoin");
        allowed_child_types.insert("HashLeftJoin");
        allowed_child_types.insert("CrossJoin");
        allowed_child_types.insert("DataCollect");
        allowed_child_types.insert("Argument");
        
        Self {
            allowed_child_types,
        }
    }

    /// 检查子节点类型是否允许移除 Project
    fn is_allowed_child_type(&self, node: &PlanNodeEnum) -> bool {
        self.allowed_child_types.contains(node.name())
    }

    /// 检查是否为无操作投影
    fn is_noop_projection(
        &self,
        project: &ProjectNode,
        child_col_names: &[String],
    ) -> bool {
        let proj_col_names = project.col_names();
        
        // 列数必须相同
        if proj_col_names.len() != child_col_names.len() {
            return false;
        }

        let columns = project.columns();
        
        // 检查每一列
        for (i, col) in columns.iter().enumerate() {
            let expr = &col.expression;
            
            // 表达式必须是简单的属性引用
            match expr {
                Expression::Variable(var_name) => {
                    // 变量名必须与 Project 的列名匹配
                    if var_name != &proj_col_names[i] {
                        return false;
                    }
                }
                Expression::Property { property, .. } => {
                    // 属性名必须与 Project 的列名匹配
                    if property != &proj_col_names[i] {
                        return false;
                    }
                }
                _ => {
                    // 其他表达式类型，不是无操作投影
                    return false;
                }
            }
            
            // 检查列名是否与输入列名匹配
            if proj_col_names[i] != child_col_names[i] {
                return false;
            }
        }

        true
    }
}

impl Default for RemoveNoopProjectRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for RemoveNoopProjectRule {
    fn name(&self) -> &'static str {
        "RemoveNoopProjectRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Project")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为 Project 节点
        let project = match node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 获取输入节点
        let input = project.input();
        
        // 检查子节点类型是否允许
        if !self.is_allowed_child_type(input) {
            return Ok(None);
        }

        // 获取子节点的列名
        let child_col_names = input.col_names();
        
        // 检查是否为无操作投影
        if !self.is_noop_projection(project, child_col_names) {
            return Ok(None);
        }

        // 创建转换结果，用输入节点替换当前 Project 节点
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(input.clone());

        Ok(Some(result))
    }
}

impl EliminationRule for RemoveNoopProjectRule {
    fn can_eliminate(&self, node: &PlanNodeEnum) -> bool {
        match node {
            PlanNodeEnum::Project(n) => {
                let input = n.input();
                if !self.is_allowed_child_type(input) {
                    return false;
                }
                self.is_noop_projection(n, input.col_names())
            }
            _ => false,
        }
    }

    fn eliminate(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

    #[test]
    fn test_remove_noop_project_rule_name() {
        let rule = RemoveNoopProjectRule::new();
        assert_eq!(rule.name(), "RemoveNoopProjectRule");
    }

    #[test]
    fn test_remove_noop_project_rule_pattern() {
        let rule = RemoveNoopProjectRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_is_allowed_child_type() {
        let rule = RemoveNoopProjectRule::new();
        
        // 测试允许的子节点类型
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        // Start 不在允许列表中
        assert!(!rule.is_allowed_child_type(&PlanNodeEnum::Start(start_node.clone())));
    }
}
