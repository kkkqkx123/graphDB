use std::collections::{HashMap, HashSet};
use crate::core::{Value, NullType, Vertex, Edge};
use super::{
    value::Expression,
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

            Expression::Unary { op, operand } => {
                let value = operand.eval(context)?;
                eval_unary_op(*op, value)
            },

            Expression::Binary { op, left, right } => {
                let left_val = left.eval(context)?;
                let right_val = right.eval(context)?;
                eval_binary_op(*op, left_val, right_val)
            },

            Expression::Variable { name } => context.get_variable(name),

            Expression::Property { entity, property } => {
                let entity_val = entity.eval(context)?;
                eval_property_access(entity_val, property)
            },

            Expression::FunctionCall { name, args } => {
                let evaluated_args: Result<Vec<Value>, _> = 
                    args.iter().map(|arg| arg.eval(context)).collect();
                let args = evaluated_args?;
                eval_function_call(name, args)
            },

            Expression::List(items) => eval_list(items, context),

            Expression::Map(items) => eval_map(items, context),

            Expression::Set(items) => eval_set(items, context),

            Expression::Case { conditions, default } => {
                for (condition, result) in conditions {
                    let cond_value = condition.eval(context)?;
                    let should_execute = match cond_value {
                        Value::Bool(true) => true,
                        Value::Int(n) if n != 0 => true,
                        _ => false,
                    };
                    if should_execute {
                        return result.eval(context);
                    }
                }

                match default {
                    Some(default_expr) => default_expr.eval(context),
                    None => Ok(Value::Null(NullType::NaN)),
                }
            },
        }
    }
}