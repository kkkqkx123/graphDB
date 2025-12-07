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

            Expression::Vertex { name } => {
                match name.as_str() {
                    "VERTEX" => context.get_current_vertex(),
                    "$^" => context.get_src_vertex(),
                    "$$" => context.get_dst_vertex(),
                    _ => Err(EvaluationError::UndefinedVariable(format!("Vertex identifier: {}", name))),
                }
            },

            Expression::Edge => context.get_current_edge(),

            Expression::PathBuild { items } => {
                // Build a path by evaluating each item and combining them
                let mut evaluated_items = Vec::new();
                for item in items {
                    let item_value = item.eval(context)?;
                    evaluated_items.push(item_value);
                }

                // In a real implementation, we would build an actual Path object
                // For now, we'll return the first vertex and last vertex as a simple path
                if evaluated_items.is_empty() {
                    Ok(Value::Path(crate::core::Path::default()))
                } else {
                    // Create a path from the evaluated items
                    let vertices: Result<Vec<Vertex>, EvaluationError> = evaluated_items
                        .iter()
                        .filter_map(|v| match v {
                            Value::Vertex(vertex_box) => Some(Ok((**vertex_box).clone())),
                            _ => None,
                        })
                        .collect();

                    let edges: Result<Vec<Edge>, EvaluationError> = evaluated_items
                        .iter()
                        .filter_map(|v| match v {
                            Value::Edge(edge) => Some(Ok(edge.clone())),
                            _ => None,
                        })
                        .collect();

                    match (vertices, edges) {
                        (Ok(verts), Ok(edgs)) => {
                            let path = crate::core::Path {
                                src: Box::new(verts.first().cloned().unwrap_or_else(|| Vertex::new(Value::Null(NullType::NaN), vec![]))),
                                steps: edgs.iter().enumerate().map(|(i, edge)| crate::core::Step {
                                    dst: Box::new(verts.get(i + 1).cloned().unwrap_or_else(|| Vertex::new(Value::Null(NullType::NaN), vec![]))),
                                    edge: Box::new(edge.clone()),
                                }).collect(),
                            };
                            Ok(Value::Path(path))
                        }
                        _ => Err(EvaluationError::TypeError("Error constructing path from items".to_string())),
                    }
                }
            },

            Expression::Aggregate { name, arg, distinct } => {
                // For now, we'll create a simple aggregation evaluation
                // In a real implementation, we would need to track aggregation state across multiple evaluations
                // This is a simplified version for demonstration purposes
                match arg {
                    Some(arg_expr) => {
                        let arg_value = arg_expr.eval(context)?;

                        // Get the aggregate function from the manager
                        if let Some(func) = crate::expressions::agg::AggFunctionManager::get(name) {
                            let mut agg_data = crate::expressions::agg::AggData::new();

                            // If distinct is true, we check if the value is unique before applying the function
                            if *distinct {
                                // For distinct, we only add unique values
                                if !agg_data.uniques_mut().contains(&arg_value) {
                                    agg_data.uniques_mut().insert(arg_value.clone());
                                    func(&mut agg_data, &arg_value);
                                }
                            } else {
                                func(&mut agg_data, &arg_value);
                            }

                            Ok(agg_data.result().clone())
                        } else {
                            Err(EvaluationError::Other(format!("Unknown aggregate function: {}", name)))
                        }
                    }
                    None => {
                        // For functions that don't require an argument (like COUNT(*))
                        if name.to_uppercase() == "COUNT" {
                            // For COUNT(*) we return 1 as a placeholder
                            // In a real implementation, this would be handled differently
                            Ok(Value::Int(1))
                        } else {
                            Err(EvaluationError::Other(format!("Aggregate function {} requires an argument", name)))
                        }
                    }
                }
            },

            Expression::ListComprehension { inner_var, collection, filter, mapping } => {
                // Evaluate the collection first
                let collection_value = collection.eval(context)?;

                // Extract list from the collection value
                let source_list = match &collection_value {
                    Value::List(list) => list,
                    _ => return Err(EvaluationError::TypeError("List comprehension requires a list".to_string())),
                };

                // Create a new context with an extended scope for the inner variable
                let mut result_list = Vec::new();

                // Iterate through each element in the source list
                for item in source_list {
                    // Create an extended context with the current item bound to the inner variable
                    // Since we can't easily extend the context, we'll just pass the original context
                    // In a real implementation, we would have a more sophisticated scoping mechanism

                    // Check the filter condition if it exists
                    let should_include = match filter {
                        Some(filter_expr) => {
                            // This is a simplified implementation - in a real system,
                            // we'd need to bind the inner variable before evaluating the filter
                            let filter_result = filter_expr.eval(context)?;
                            matches!(filter_result, Value::Bool(true)) ||
                if let Value::Int(n) = filter_result { n != 0 } else { false }
                        },
                        None => true,  // If no filter, include all items
                    };

                    if should_include {
                        // Apply the mapping expression if it exists, otherwise use the item as is
                        let mapped_value = match mapping {
                            Some(mapping_expr) => {
                                // In a real implementation, we would bind `inner_var` to `item`
                                // and then evaluate the mapping expression in that context
                                mapping_expr.eval(context)?
                            },
                            None => item.clone(),  // If no mapping, use the item as is
                        };

                        result_list.push(mapped_value);
                    }
                }

                Ok(Value::List(result_list))
            },
        }
    }
}