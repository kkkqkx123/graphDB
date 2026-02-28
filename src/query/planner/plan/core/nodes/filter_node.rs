//! 过滤节点实现
//!
//! FilterNode 用于根据指定的条件过滤输入数据流

use std::sync::Arc;

use crate::define_plan_node_with_deps;
use crate::core::types::{ContextualExpression, SerializableExpression, ExpressionContext};
use crate::core::Expression;
use super::plan_node_enum::PlanNodeEnum;

define_plan_node_with_deps! {
    pub struct FilterNode {
        condition: ContextualExpression,
        condition_serializable: Option<SerializableExpression>,
    }
    enum: Filter
    input: SingleInputNode
}

impl FilterNode {
    /// 创建新的过滤节点
    pub fn new(
        input: PlanNodeEnum,
        condition: ContextualExpression,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            condition,
            condition_serializable: None,
            output_var: None,
            col_names,
        })
    }
    
    /// 创建新的过滤节点（从Expression）
    pub fn from_expression(
        input: PlanNodeEnum,
        condition: Expression,
        ctx: Arc<ExpressionContext>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(condition);
        let id = ctx.register_expression(expr_meta);
        let ctx_expr = ContextualExpression::new(id, ctx);

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            condition: ctx_expr,
            condition_serializable: None,
            output_var: None,
            col_names,
        })
    }

    /// 获取过滤条件
    pub fn condition(&self) -> &ContextualExpression {
        &self.condition
    }

    /// 设置过滤条件
    pub fn set_condition(&mut self, condition: ContextualExpression) {
        self.condition = condition;
        self.condition_serializable = None;
    }
    
    /// 设置过滤条件（从Expression）
    pub fn set_condition_expression(&mut self, condition: Expression, ctx: Arc<ExpressionContext>) {
        let expr_meta = crate::core::types::expression::ExpressionMeta::new(condition);
        let id = ctx.register_expression(expr_meta);
        self.condition = ContextualExpression::new(id, ctx);
        self.condition_serializable = None;
    }
    
    pub fn prepare_for_serialization(&mut self) {
        self.condition_serializable = Some(SerializableExpression::from_contextual(&self.condition));
    }
    
    pub fn after_deserialization(&mut self, ctx: Arc<ExpressionContext>) {
        if let Some(ref ser_expr) = self.condition_serializable {
            self.condition = ser_expr.clone().to_contextual(ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use std::sync::Arc;
    use crate::core::types::{ExpressionContext, ExpressionMeta};

    #[test]
    fn test_filter_node_creation() {
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_node_enum =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                start_node,
            );

        let ctx = Arc::new(ExpressionContext::new());
        let expr = Expression::Variable("test".to_string());
        let expr_meta = ExpressionMeta::new(expr);
        let id = ctx.register_expression(expr_meta);
        let condition = ContextualExpression::new(id, ctx);
        
        let filter_node = FilterNode::new(start_node_enum, condition)
            .expect("Filter node should be created successfully");

        assert_eq!(filter_node.type_name(), "FilterNode");
        assert!(filter_node.condition().is_variable());
    }
}
