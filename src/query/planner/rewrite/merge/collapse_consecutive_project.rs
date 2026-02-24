//! 合并连续投影规则
//!
//! 当多个 Project 节点连续出现时，合并为一个 Project 节点
//! 减少不必要的中间结果生成
//!
//! 示例:
//! ```
//! Project(a, b) -> Project(c, d)  =>  Project(c, d)
//! ```
//!
//! 适用条件:
//! - 两个 Project 节点连续出现
//! - 上层 Project 不依赖下层 Project 的别名解析

use crate::core::{Expression, YieldColumn};
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult};
use crate::query::planner::rewrite::rule::{MergeRule, RewriteRule};
use std::collections::HashMap;

/// 合并连续投影规则
///
/// # 转换示例
///
/// Before:
/// ```text
///   Project(col2)
///       |
///   Project(col1)
///       |
///   ScanVertices
/// ```
///
/// After:
/// ```text
///   Project(col2)
///       |
///   ScanVertices
/// ```
///
/// # 适用条件
///
/// - 当前节点为Project节点
/// - 子节点也为Project节点
/// - 上层Project的列引用可以解析为下层Project的输入
#[derive(Debug)]
pub struct CollapseConsecutiveProjectRule;

impl CollapseConsecutiveProjectRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 重写表达式，将属性引用替换为实际表达式
    fn rewrite_expression(
        expr: &Expression,
        rewrite_map: &HashMap<String, Expression>,
    ) -> Expression {
        match expr {
            Expression::Variable(name) => {
                if let Some(new_expr) = rewrite_map.get(name) {
                    new_expr.clone()
                } else {
                    expr.clone()
                }
            }
            Expression::Property { object, property } => {
                if let Expression::Variable(obj_name) = object.as_ref() {
                    let full_name = format!("{}.{}", obj_name, property);
                    if let Some(new_expr) = rewrite_map.get(&full_name) {
                        return new_expr.clone();
                    }
                    if let Some(new_expr) = rewrite_map.get(property) {
                        return Expression::Property {
                            object: Box::new(new_expr.clone()),
                            property: property.clone(),
                        };
                    }
                }
                Expression::Property {
                    object: Box::new(Self::rewrite_expression(object, rewrite_map)),
                    property: property.clone(),
                }
            }
            Expression::Binary { left, op, right } => Expression::Binary {
                left: Box::new(Self::rewrite_expression(left, rewrite_map)),
                op: *op,
                right: Box::new(Self::rewrite_expression(right, rewrite_map)),
            },
            Expression::Unary { op, operand } => Expression::Unary {
                op: *op,
                operand: Box::new(Self::rewrite_expression(operand, rewrite_map)),
            },
            Expression::Function { name, args } => Expression::Function {
                name: name.clone(),
                args: args
                    .iter()
                    .map(|arg| Self::rewrite_expression(arg, rewrite_map))
                    .collect(),
            },
            Expression::Aggregate { func, arg, distinct } => Expression::Aggregate {
                func: func.clone(),
                arg: Box::new(Self::rewrite_expression(arg, rewrite_map)),
                distinct: *distinct,
            },
            Expression::List(list) => Expression::List(
                list.iter()
                    .map(|item| Self::rewrite_expression(item, rewrite_map))
                    .collect(),
            ),
            Expression::Map(map) => Expression::Map(
                map.iter()
                    .map(|(k, v)| (k.clone(), Self::rewrite_expression(v, rewrite_map)))
                    .collect(),
            ),
            Expression::Case {
                test_expr,
                conditions,
                default,
            } => Expression::Case {
                test_expr: test_expr
                    .as_ref()
                    .map(|e| Box::new(Self::rewrite_expression(e, rewrite_map))),
                conditions: conditions
                    .iter()
                    .map(|(w, t)| {
                        (
                            Self::rewrite_expression(w, rewrite_map),
                            Self::rewrite_expression(t, rewrite_map),
                        )
                    })
                    .collect(),
                default: default
                    .as_ref()
                    .map(|e| Box::new(Self::rewrite_expression(e, rewrite_map))),
            },
            Expression::TypeCast { expression, target_type } => Expression::TypeCast {
                expression: Box::new(Self::rewrite_expression(expression, rewrite_map)),
                target_type: target_type.clone(),
            },
            Expression::Subscript { collection, index } => Expression::Subscript {
                collection: Box::new(Self::rewrite_expression(collection, rewrite_map)),
                index: Box::new(Self::rewrite_expression(index, rewrite_map)),
            },
            _ => expr.clone(),
        }
    }

    /// 执行合并操作
    fn merge_projects(
        &self,
        parent_proj: &ProjectNode,
        child_proj: &ProjectNode,
    ) -> Option<ProjectNode> {
        // 构建列名到表达式的映射（从子Project）
        let mut rewrite_map = HashMap::new();
        for col in child_proj.columns() {
            if !col.alias.is_empty() {
                rewrite_map.insert(col.alias.clone(), col.expression.clone());
            }
        }

        // 重写父Project的列表达式
        let new_columns: Vec<YieldColumn> = parent_proj
            .columns()
            .iter()
            .map(|col| YieldColumn {
                expression: Self::rewrite_expression(&col.expression, &rewrite_map),
                alias: col.alias.clone(),
                is_matched: col.is_matched,
            })
            .collect();

        // 创建新的Project节点，输入为子Project的输入
        let child_input = child_proj.input().clone();
        ProjectNode::new(child_input, new_columns).ok()
    }
}

impl Default for CollapseConsecutiveProjectRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for CollapseConsecutiveProjectRule {
    fn name(&self) -> &'static str {
        "CollapseConsecutiveProjectRule"
    }

    fn pattern(&self) -> Pattern {
        Pattern::new_with_name("Project").with_dependency_name("Project")
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为Project节点
        let parent_proj = match node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 获取子节点
        let child_node = parent_proj.input();
        let child_proj = match child_node {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 执行合并
        if let Some(new_proj) = self.merge_projects(parent_proj, child_proj) {
            let mut result = TransformResult::new();
            result.erase_curr = true;
            result.add_new_node(PlanNodeEnum::Project(new_proj));
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

impl MergeRule for CollapseConsecutiveProjectRule {
    fn can_merge(&self, parent: &PlanNodeEnum, child: &PlanNodeEnum) -> bool {
        parent.is_project() && child.is_project()
    }

    fn create_merged_node(
        &self,
        ctx: &mut RewriteContext,
        parent: &PlanNodeEnum,
        _child: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        self.apply(ctx, parent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_rule_name() {
        let rule = CollapseConsecutiveProjectRule::new();
        assert_eq!(rule.name(), "CollapseConsecutiveProjectRule");
    }

    #[test]
    fn test_rule_pattern() {
        let rule = CollapseConsecutiveProjectRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_collapse_consecutive_projects() {
        // 创建起始节点
        let start = PlanNodeEnum::Start(StartNode::new());

        // 创建下层Project节点
        let child_columns = vec![
            YieldColumn {
                expression: Expression::Variable("a".to_string()),
                alias: "col_a".to_string(),
                is_matched: false,
            },
            YieldColumn {
                expression: Expression::Variable("b".to_string()),
                alias: "col_b".to_string(),
                is_matched: false,
            },
        ];
        let child_proj = ProjectNode::new(start, child_columns).expect("创建下层Project失败");
        let child_node = PlanNodeEnum::Project(child_proj);

        // 创建上层Project节点，引用下层Project的别名
        let parent_columns = vec![YieldColumn {
            expression: Expression::Variable("col_a".to_string()),
            alias: "result".to_string(),
            is_matched: false,
        }];
        let parent_proj = ProjectNode::new(child_node, parent_columns).expect("创建上层Project失败");
        let parent_node = PlanNodeEnum::Project(parent_proj);

        // 应用规则
        let rule = CollapseConsecutiveProjectRule::new();
        let mut ctx = RewriteContext::new();
        let result = rule.apply(&mut ctx, &parent_node).expect("应用规则失败");

        assert!(
            result.is_some(),
            "应该成功合并连续的Project节点"
        );

        // 验证结果
        let transform_result = result.expect("Failed to apply rewrite rule");
        assert!(transform_result.erase_curr);
        assert_eq!(transform_result.new_nodes.len(), 1);

        // 验证新的Project节点
        if let PlanNodeEnum::Project(ref new_proj) = transform_result.new_nodes[0] {
            let columns = new_proj.columns();
            assert_eq!(columns.len(), 1);
            assert_eq!(columns[0].alias, "result");
            // 验证表达式已被重写为原始引用
            if let Expression::Variable(name) = &columns[0].expression {
                assert_eq!(name, "a");
            } else {
                panic!("表达式应该是Variable");
            }
        } else {
            panic!("转换结果应该是Project节点");
        }
    }
}
