use std::collections::{HashMap, HashSet};
use crate::core::{Value, NullType, Vertex, Edge};
use crate::graph::expression::Expression;
use super::{
    base::{EvaluationError, ExpressionContext},
    operations::{eval_unary_op, eval_binary_op},
    function_call::eval_function_call,
    property_access::eval_property_access,
    container::{eval_list, eval_map, eval_set},
};

impl Expression {
    /// Evaluate the expression within the given context
    pub fn eval(&self, context: &dyn ExpressionContext) -> Result<Value, EvaluationError> {
        match self {
            Expression::Constant(value) => Ok(value.clone()),

            Expression::UnaryOp(op, operand) => {
                let value = operand.eval(context)?;
                eval_unary_op(*op, value)
            },

            Expression::BinaryOp(left, op, right) => {
                let left_val = left.eval(context)?;
                let right_val = right.eval(context)?;
                eval_binary_op(*op, left_val, right_val)
            },

            Expression::Property(name) => context.get_variable(name),

            Expression::Function(name, args) => {
                let evaluated_args: Result<Vec<Value>, _> =
                    args.iter().map(|arg| arg.eval(context)).collect();
                let args = evaluated_args?;
                eval_function_call(name, args)
            },
        }
    }
}