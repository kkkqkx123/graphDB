use crate::core::Value;
use crate::graph::expression::{Expression, BinaryOperator, UnaryOperator};
use super::context::EvalContext;
use super::error::ExpressionError;

/// Expression evaluator
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// Evaluate an expression in the given context
    pub fn evaluate(&self, expr: &Expression, context: &EvalContext) -> Result<Value, ExpressionError> {
        self.eval_expression(expr, context)
    }

    /// Evaluate an expression in the given context
    pub fn eval_expression(&self, expr: &Expression, context: &EvalContext) -> Result<Value, ExpressionError> {
        match expr {
            Expression::Constant(value) => Ok(value.clone()),
            Expression::Property(prop_name) => {
                // Try to find the property in vertex
                if let Some(vertex) = context.vertex {
                    for tag in &vertex.tags {
                        if let Some(value) = tag.properties.get(prop_name) {
                            return Ok(value.clone());
                        }
                    }
                }

                // Try to find the property in edge
                if let Some(edge) = context.edge {
                    if let Some(value) = edge.props.get(prop_name) {
                        return Ok(value.clone());
                    }
                }

                // Try to find the property in variables
                if let Some(value) = context.vars.get(prop_name) {
                    return Ok(value.clone());
                }

                Err(ExpressionError::PropertyNotFound(prop_name.clone()))
            },
            Expression::BinaryOp(left, op, right) => {
                super::binary::evaluate_binary_op(left, op, right, context)
            },
            Expression::UnaryOp(op, operand) => {
                super::unary::evaluate_unary_op(op, operand, context)
            },
            Expression::Function(func_name, args) => {
                super::function::evaluate_function(func_name, args, context)
            },
            
            // 新增表达式类型的处理
            expr @ Expression::TagProperty { .. } |
            expr @ Expression::EdgeProperty { .. } |
            expr @ Expression::InputProperty(_) |
            expr @ Expression::VariableProperty { .. } |
            expr @ Expression::SourceProperty { .. } |
            expr @ Expression::DestinationProperty { .. } => {
                super::property::evaluate_property_expression(expr, context)
            }
            
            expr @ Expression::UnaryPlus(_) |
            expr @ Expression::UnaryNegate(_) |
            expr @ Expression::UnaryNot(_) |
            expr @ Expression::UnaryIncr(_) |
            expr @ Expression::UnaryDecr(_) |
            expr @ Expression::IsNull(_) |
            expr @ Expression::IsNotNull(_) |
            expr @ Expression::IsEmpty(_) |
            expr @ Expression::IsNotEmpty(_) => {
                super::unary::evaluate_extended_unary_op(expr, context)
            }
            
            expr @ Expression::List(_) |
            expr @ Expression::Set(_) |
            expr @ Expression::Map(_) => {
                super::container::evaluate_container(expr, context)
            }
            
            Expression::TypeCasting { expr, target_type: _ } => {
                // 类型转换暂时返回原值，实际实现需要根据目标类型进行转换
                self.evaluate(expr, context)
            }
            
            Expression::Case { conditions, default } => {
                for (condition, value) in conditions {
                    let cond_result = self.evaluate(condition, context)?;
                    if super::unary::value_to_bool(&cond_result) {
                        return self.evaluate(value, context);
                    }
                }

                if let Some(default_expr) = default {
                    self.evaluate(default_expr, context)
                } else {
                    Ok(Value::Null(crate::core::NullType::Null))
                }
            }

            Expression::Aggregate { func, arg, distinct } => {
                super::function::evaluate_aggregate(func, arg, *distinct, context)
            }

            Expression::ListComprehension { generator, condition } => {
                // 简化实现：返回生成器的结果
                if let Some(cond) = condition {
                    let cond_result = self.evaluate(cond, context)?;
                    if super::unary::value_to_bool(&cond_result) {
                        self.evaluate(generator, context)
                    } else {
                        Ok(Value::List(vec![]))
                    }
                } else {
                    self.evaluate(generator, context)
                }
            },

            Expression::Predicate { list, condition } => {
                let list_value = self.evaluate(list, context)?;
                let condition_clone = (*condition).clone();

                // 简化实现：检查列表中的元素是否满足条件
                match list_value {
                    Value::List(items) => {
                        for item in items {
                            // 创建一个临时上下文，将当前元素作为变量
                            let mut temp_context = context.clone();
                            temp_context.vars.insert("__item".to_string(), item);

                            let cond_result = self.evaluate(&condition_clone, &temp_context)?;
                            if super::unary::value_to_bool(&cond_result) {
                                return Ok(Value::Bool(true));
                            }
                        }
                        Ok(Value::Bool(false))
                    },
                    _ => Err(ExpressionError::TypeError("Predicate requires a list".to_string())),
                }
            },

            Expression::Reduce { list, var, initial, expr } => {
                let list_value = self.evaluate(list, context)?;
                let initial_value = self.evaluate(initial, context)?;

                match list_value {
                    Value::List(items) => {
                        let mut accumulator = initial_value;
                        for item in items {
                            let mut temp_context = context.clone();
                            temp_context.vars.insert(var.clone(), item);

                            // 这里需要使用当前累加器值，但在简化实现中，我们只计算一次
                            accumulator = self.evaluate(expr, &temp_context)?;
                        }
                        Ok(accumulator)
                    },
                    _ => Err(ExpressionError::TypeError("Reduce requires a list".to_string())),
                }
            },

            Expression::PathBuild(items) => {
                // 路径构建的简化实现
                let mut path_items = Vec::new();
                for item in items {
                    path_items.push(self.evaluate(item, context)?);
                }
                Ok(Value::List(path_items))  // 简化为列表形式
            },

            Expression::ESQuery(query) => {
                // 文本搜索表达式，返回查询字符串
                Ok(Value::String(query.clone()))
            },

            Expression::UUID => {
                // 生成UUID的简化实现
                use uuid::Uuid;
                Ok(Value::String(Uuid::new_v4().to_string()))
            },

            Expression::Variable(var_name) => {
                // 从上下文变量中获取值
                if let Some(value) = context.vars.get(var_name) {
                    Ok(value.clone())
                } else {
                    Err(ExpressionError::PropertyNotFound(format!("Variable ${}", var_name)))
                }
            },

            Expression::Subscript { collection, index } => {
                let coll_value = self.evaluate(collection, context)?;
                let idx_value = self.evaluate(index, context)?;

                super::binary::subscript_values(coll_value, idx_value)
            },

            Expression::SubscriptRange { collection, start, end } => {
                let coll_value = self.evaluate(collection, context)?;

                match coll_value {
                    Value::List(items) => {
                        let start_idx = if let Some(start_expr) = start {
                            let val = self.evaluate(start_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => return Err(ExpressionError::TypeError("Range start index must be an integer".to_string())),
                            }
                        } else {
                            0
                        };

                        let end_idx = if let Some(end_expr) = end {
                            let val = self.evaluate(end_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => return Err(ExpressionError::TypeError("Range end index must be an integer".to_string())),
                            }
                        } else {
                            items.len()
                        };

                        if start_idx > end_idx || end_idx > items.len() {
                            return Err(ExpressionError::InvalidOperation("Invalid range".to_string()));
                        }

                        let result = items[start_idx..end_idx].to_vec();
                        Ok(Value::List(result))
                    },
                    Value::String(s) => {
                        let start_idx = if let Some(start_expr) = start {
                            let val = self.evaluate(start_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => return Err(ExpressionError::TypeError("Range start index must be an integer".to_string())),
                            }
                        } else {
                            0
                        };

                        let end_idx = if let Some(end_expr) = end {
                            let val = self.evaluate(end_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => return Err(ExpressionError::TypeError("Range end index must be an integer".to_string())),
                            }
                        } else {
                            s.len()
                        };

                        if start_idx > end_idx || end_idx > s.len() {
                            return Err(ExpressionError::InvalidOperation("Invalid range".to_string()));
                        }

                        let result = s[start_idx..end_idx].to_string();
                        Ok(Value::String(result))
                    },
                    _ => Err(ExpressionError::TypeError("Subscript range requires a list or string".to_string())),
                }
            },

            Expression::Label(label_name) => {
                // 标签表达式，返回标签名
                Ok(Value::String(label_name.clone()))
            },

            Expression::MatchPathPattern { path_alias, patterns: _ } => {
                // 匹配路径模式表达式，简化实现返回路径别名
                Ok(Value::String(path_alias.clone()))
            },
        }
    }
}