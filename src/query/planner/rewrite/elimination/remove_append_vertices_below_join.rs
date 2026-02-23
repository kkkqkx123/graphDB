//! 移除连接下方的添加顶点操作的规则
//!
//! 根据 nebula-graph 的参考实现，此规则匹配以下模式：
//! HashInnerJoin/HashLeftJoin -> ... -> Project -> AppendVertices -> Traverse
//! 当满足特定条件时，可以移除 AppendVertices 节点。
//!
//! # 转换示例
//!
//! Before:
//! ```text
//!   HashInnerJoin({id(v)}, {id(v)})
//!    /         \
//!   /           Project
//!  /               \
//! Left           AppendVertices(v)
//!                     \
//!                   Traverse(e)
//! ```
//!
//! After:
//! ```text
//!   HashInnerJoin({id(v)}, {$-.v})
//!    /         \
//!   /     Project(..., none_direct_dst(e) AS v)
//!  /               \
//! Left          Traverse(e)
//! ```
//!
//! # 适用条件
//!
//! - Join 的右分支为 Project->AppendVertices->Traverse
//! - AppendVertices 的 nodeAlias 只被引用一次
//! - Join 的 hash keys 匹配 id() 或 _joinkey() 模式

use crate::core::Expression;
use crate::core::types::YieldColumn;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::{SingleInputNode, MultipleInputNode, JoinNode};
use crate::query::planner::plan::core::nodes::join_node::{HashInnerJoinNode, HashLeftJoinNode};
use crate::query::planner::plan::core::nodes::project_node::ProjectNode;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::pattern::Pattern;
use crate::query::planner::rewrite::result::{RewriteResult, TransformResult, RewriteError};
use crate::query::planner::rewrite::rule::RewriteRule;

/// 移除连接下方的添加顶点操作的规则
///
/// 当 Join 的右分支包含 AppendVertices 且满足特定条件时，移除 AppendVertices
#[derive(Debug)]
pub struct RemoveAppendVerticesBelowJoinRule;

impl RemoveAppendVerticesBelowJoinRule {
    /// 创建规则实例
    pub fn new() -> Self {
        Self
    }

    /// 检查节点是否为哈希连接节点
    fn is_hash_join(&self, node: &PlanNodeEnum) -> bool {
        matches!(node, PlanNodeEnum::HashInnerJoin(_) | PlanNodeEnum::HashLeftJoin(_))
    }

    /// 从表达式中收集所有属性名
    fn collect_all_property_names(&self, expr: &Expression) -> Vec<String> {
        let mut result = Vec::new();
        self.collect_property_names_recursive(expr, &mut result);
        result
    }

    /// 递归收集属性名
    fn collect_property_names_recursive(&self, expr: &Expression, result: &mut Vec<String>) {
        match expr {
            Expression::Property { property, .. } => {
                if !result.contains(property) {
                    result.push(property.clone());
                }
            }
            Expression::Binary { left, right, .. } => {
                self.collect_property_names_recursive(left, result);
                self.collect_property_names_recursive(right, result);
            }
            Expression::Unary { operand, .. } => {
                self.collect_property_names_recursive(operand, result);
            }
            Expression::Function { args, .. } => {
                for arg in args {
                    self.collect_property_names_recursive(arg, result);
                }
            }
            Expression::Case { conditions, default, .. } => {
                for (when, then) in conditions {
                    self.collect_property_names_recursive(when, result);
                    self.collect_property_names_recursive(then, result);
                }
                if let Some(d) = default {
                    self.collect_property_names_recursive(d, result);
                }
            }
            _ => {}
        }
    }

    /// 检查表达式是否为 id() 或 _joinkey() 函数调用，返回参数表达式
    fn is_id_or_joinkey_function(&self, expr: &Expression) -> Option<Expression> {
        match expr {
            Expression::Function { name, args } if (name == "id" || name == "_joinkey") && args.len() == 1 => {
                Some(args[0].clone())
            }
            _ => None,
        }
    }

    /// 检查表达式是否引用指定属性
    fn expr_references_alias(&self, expr: &Expression, alias: &str) -> bool {
        let properties = self.collect_all_property_names(expr);
        properties.iter().any(|p| p == alias)
    }

    /// 计算 avNodeAlias 在表达式列表中的引用次数
    fn count_alias_references(&self, exprs: &[Expression], alias: &str) -> usize {
        exprs.iter().filter(|e| self.expr_references_alias(e, alias)).count()
    }

    /// 计算 avNodeAlias 在 YieldColumn 列表中的引用次数
    fn count_alias_references_in_columns(&self, columns: &[YieldColumn], alias: &str) -> usize {
        columns.iter().filter(|c| self.expr_references_alias(&c.expression, alias)).count()
    }

    /// 查找包含指定别名的列索引
    fn find_column_with_alias(&self, columns: &[YieldColumn], alias: &str) -> Option<usize> {
        for (idx, col) in columns.iter().enumerate() {
            if let Expression::Variable(var_name) = &col.expression {
                if var_name == alias {
                    return Some(idx);
                }
            }
        }
        None
    }

    /// 查找 probe keys 中匹配 id()/_joinkey() 模式的索引
    fn find_matching_probe_key(&self, probe_keys: &[Expression], av_node_alias: &str) -> Option<usize> {
        for (idx, expr) in probe_keys.iter().enumerate() {
            if let Some(arg) = self.is_id_or_joinkey_function(expr) {
                if self.expr_contains_variable(&arg, av_node_alias) {
                    return Some(idx);
                }
            }
        }
        None
    }

    /// 检查表达式是否包含指定变量引用
    fn expr_contains_variable(&self, expr: &Expression, var_name: &str) -> bool {
        match expr {
            Expression::Variable(name) => name == var_name,
            Expression::Property { object, .. } => self.expr_contains_variable(object, var_name),
            Expression::Binary { left, right, .. } => {
                self.expr_contains_variable(left, var_name) || self.expr_contains_variable(right, var_name)
            }
            Expression::Unary { operand, .. } => self.expr_contains_variable(operand, var_name),
            Expression::Function { args, .. } => {
                args.iter().any(|arg| self.expr_contains_variable(arg, var_name))
            }
            _ => false,
        }
    }

    /// 创建 none_direct_dst 函数调用表达式
    fn create_none_direct_dst_expr(&self, edge_alias: &str, vertex_alias: &str) -> Expression {
        Expression::Function {
            name: "none_direct_dst".to_string(),
            args: vec![
                Expression::Variable(edge_alias.to_string()),
                Expression::Variable(vertex_alias.to_string()),
            ],
        }
    }

    /// 创建变量引用表达式
    fn create_variable_expr(&self, var_name: &str) -> Expression {
        Expression::Variable(var_name.to_string())
    }
}

impl Default for RemoveAppendVerticesBelowJoinRule {
    fn default() -> Self {
        Self::new()
    }
}

impl RewriteRule for RemoveAppendVerticesBelowJoinRule {
    fn name(&self) -> &'static str {
        "RemoveAppendVerticesBelowJoinRule"
    }

    fn pattern(&self) -> Pattern {
        // 匹配 HashInnerJoin 或 HashLeftJoin
        Pattern::multi(vec!["HashInnerJoin", "HashLeftJoin"])
    }

    fn apply(
        &self,
        _ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<Option<TransformResult>> {
        // 检查是否为哈希连接节点，并获取 JoinNode trait 对象
        let join_node: &dyn JoinNode = match node {
            PlanNodeEnum::HashInnerJoin(n) => n,
            PlanNodeEnum::HashLeftJoin(n) => n,
            _ => return Ok(None),
        };

        // 获取左右输入节点
        let left_input = join_node.left_input();
        let right_input = join_node.right_input();

        // 检查右输入是否为 Project
        let project = match right_input {
            PlanNodeEnum::Project(n) => n,
            _ => return Ok(None),
        };

        // 获取 Project 的输入节点
        let project_input = project.input();

        // 检查是否为 AppendVertices
        let append_vertices = match project_input {
            PlanNodeEnum::AppendVertices(n) => n,
            _ => return Ok(None),
        };

        // 获取 AppendVertices 的 node_alias
        let av_node_alias = match append_vertices.node_alias() {
            Some(alias) => alias,
            None => return Ok(None),
        };

        // 获取 AppendVertices 的输入节点
        let append_inputs = append_vertices.inputs();
        if append_inputs.is_empty() {
            return Ok(None);
        }

        // 检查是否为 Traverse
        let traverse = match &*append_inputs[0] {
            PlanNodeEnum::Traverse(n) => n,
            _ => return Ok(None),
        };

        // 获取 Traverse 的 edge_alias 和 vertex_alias
        let tv_edge_alias = match traverse.edge_alias() {
            Some(alias) => alias,
            None => return Ok(None),
        };
        let _tv_node_alias = match traverse.vertex_alias() {
            Some(alias) => alias,
            None => return Ok(None),
        };

        // 获取 Join 的 hash keys 和 probe keys
        let hash_keys = join_node.hash_keys();
        let probe_keys = join_node.probe_keys();

        // 检查 avNodeAlias 在 probe keys 中的引用次数
        let probe_ref_count = self.count_alias_references(probe_keys, av_node_alias);
        if probe_ref_count > 1 {
            // 如果被引用多次，不能移除 AppendVertices
            return Ok(None);
        }

        // 查找匹配的 probe key 索引
        let probe_key_idx = match self.find_matching_probe_key(probe_keys, av_node_alias) {
            Some(idx) => idx,
            None => return Ok(None),
        };

        // 检查对应的 hash key 是否匹配
        if probe_key_idx >= hash_keys.len() {
            return Ok(None);
        }
        let corresponding_hash_key = &hash_keys[probe_key_idx];
        let probe_key = &probe_keys[probe_key_idx];
        if corresponding_hash_key != probe_key {
            return Ok(None);
        }

        // 检查 avNodeAlias 在 Project columns 中的引用次数
        let columns = project.columns();
        let col_ref_count = self.count_alias_references_in_columns(columns, av_node_alias);
        if col_ref_count > 1 {
            return Ok(None);
        }

        // 查找 Project 中包含 avNodeAlias 的列索引
        let prj_idx = match self.find_column_with_alias(columns, av_node_alias) {
            Some(idx) => idx,
            None => return Ok(None),
        };

        // 创建新的 Project 列
        let mut new_columns: Vec<YieldColumn> = columns.iter().cloned().collect();
        let none_direct_dst_expr = self.create_none_direct_dst_expr(tv_edge_alias, _tv_node_alias);
        new_columns[prj_idx] = YieldColumn {
            expression: none_direct_dst_expr,
            alias: av_node_alias.clone(),
            is_matched: false,
        };

        // 创建新的 Project 节点
        let new_project = ProjectNode::new(
            append_inputs[0].as_ref().clone(),
            new_columns,
        ).map_err(|e| RewriteError::InvalidPlanStructure(e.to_string()))?;

        // 创建新的 probe keys
        let mut new_probe_keys: Vec<Expression> = probe_keys.iter().cloned().collect();
        new_probe_keys[probe_key_idx] = self.create_variable_expr(av_node_alias);

        // 创建新的 Join 节点
        let new_join: PlanNodeEnum = match node {
            PlanNodeEnum::HashInnerJoin(_) => {
                PlanNodeEnum::HashInnerJoin(
                    HashInnerJoinNode::new(
                        left_input.clone(),
                        PlanNodeEnum::Project(new_project),
                        hash_keys.iter().cloned().collect(),
                        new_probe_keys,
                    ).map_err(|e| RewriteError::InvalidPlanStructure(e.to_string()))?
                )
            }
            PlanNodeEnum::HashLeftJoin(_) => {
                PlanNodeEnum::HashLeftJoin(
                    HashLeftJoinNode::new(
                        left_input.clone(),
                        PlanNodeEnum::Project(new_project),
                        hash_keys.iter().cloned().collect(),
                        new_probe_keys,
                    ).map_err(|e| RewriteError::InvalidPlanStructure(e.to_string()))?
                )
            }
            _ => unreachable!(),
        };

        // 构建转换结果
        let mut result = TransformResult::new();
        result.erase_curr = true;
        result.add_new_node(new_join);

        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::rewrite::rule::RewriteRule;

    #[test]
    fn test_remove_append_vertices_below_join_rule_name() {
        let rule = RemoveAppendVerticesBelowJoinRule::new();
        assert_eq!(rule.name(), "RemoveAppendVerticesBelowJoinRule");
    }

    #[test]
    fn test_remove_append_vertices_below_join_rule_pattern() {
        let rule = RemoveAppendVerticesBelowJoinRule::new();
        let pattern = rule.pattern();
        assert!(pattern.node.is_some());
    }

    #[test]
    fn test_is_hash_join() {
        let rule = RemoveAppendVerticesBelowJoinRule::new();
        
        // 创建测试节点
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_enum = PlanNodeEnum::Start(start_node);
        
        // HashInnerJoin 应该返回 true
        let hash_inner = HashInnerJoinNode::new(
            start_enum.clone(),
            start_enum.clone(),
            vec![],
            vec![],
        ).expect("Failed to create HashInnerJoinNode");
        assert!(rule.is_hash_join(&PlanNodeEnum::HashInnerJoin(hash_inner)));
        
        // HashLeftJoin 应该返回 true
        let hash_left = HashLeftJoinNode::new(
            start_enum.clone(),
            start_enum.clone(),
            vec![],
            vec![],
        ).expect("Failed to create HashLeftJoinNode");
        assert!(rule.is_hash_join(&PlanNodeEnum::HashLeftJoin(hash_left)));
        
        // Start 节点应该返回 false
        assert!(!rule.is_hash_join(&start_enum));
    }
}
