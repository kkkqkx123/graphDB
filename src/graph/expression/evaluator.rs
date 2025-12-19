use super::error::ExpressionError;
use super::operator_conversion;
use super::type_conversion;
use crate::core::Value;
use crate::graph::expression::{Expression, LiteralValue};
use crate::query::context::EvalContext;
use crate::query::parser::cypher::ast::expressions::Expression as CypherExpression;

/// Expression evaluator
#[derive(Debug)]
pub struct ExpressionEvaluator;

impl ExpressionEvaluator {
    /// Create a new ExpressionEvaluator
    pub fn new() -> Self {
        ExpressionEvaluator
    }

    /// Evaluate an expression in the given context
    pub fn evaluate(
        &self,
        expr: &Expression,
        context: &EvalContext,
    ) -> Result<Value, ExpressionError> {
        self.eval_expression(expr, context)
    }

    /// Evaluate an expression in the given context
    pub fn eval_expression(
        &self,
        expr: &Expression,
        context: &EvalContext,
    ) -> Result<Value, ExpressionError> {
        match expr {
            Expression::Literal(literal_value) => {
                // 将 LiteralValue 转换为 Value
                match literal_value {
                    LiteralValue::Bool(b) => Ok(Value::Bool(*b)),
                    LiteralValue::Int(i) => Ok(Value::Int(*i)),
                    LiteralValue::Float(f) => Ok(Value::Float(*f)),
                    LiteralValue::String(s) => Ok(Value::String(s.clone())),
                    LiteralValue::Null => Ok(Value::Null(crate::core::NullType::Null)),
                }
            }
            Expression::TypeCast { expr, target_type } => {
                let value = self.evaluate(expr, context)?;
                type_conversion::cast_value_to_datatype(value, target_type)
            }
            Expression::Property { object, property } => {
                // 先计算 object，然后获取其属性
                let obj_value = self.evaluate(object, context)?;
                // 根据对象类型获取属性
                match obj_value {
                    Value::Map(map) => map
                        .get(property)
                        .cloned()
                        .ok_or_else(|| ExpressionError::PropertyNotFound(property.clone())),
                    _ => Err(ExpressionError::PropertyNotFound(property.clone())),
                }
            }
            Expression::Binary { left, op, right } => {
                // 将 expression::BinaryOperator 转换为 binary::BinaryOperator
                let binary_op = operator_conversion::convert_binary_operator(op);
                super::binary::evaluate_binary_op(left, &binary_op, right, context)
            }
            Expression::Unary { op, operand } => {
                // 将 expression::UnaryOperator 转换为 unary::UnaryOperator
                let unary_op = operator_conversion::convert_unary_operator(op);
                super::unary::evaluate_unary_op(&unary_op, operand, context)
            }
            Expression::Function { name, args } => {
                super::function::evaluate_function(name, args, context)
            }

            // 新增表达式类型的处理
            expr @ Expression::TagProperty { .. }
            | expr @ Expression::EdgeProperty { .. }
            | expr @ Expression::InputProperty(_)
            | expr @ Expression::VariableProperty { .. }
            | expr @ Expression::SourceProperty { .. }
            | expr @ Expression::DestinationProperty { .. } => {
                super::property::evaluate_property_expression(expr, context)
            }

            expr @ Expression::UnaryPlus(_)
            | expr @ Expression::UnaryNegate(_)
            | expr @ Expression::UnaryNot(_)
            | expr @ Expression::UnaryIncr(_)
            | expr @ Expression::UnaryDecr(_)
            | expr @ Expression::IsNull(_)
            | expr @ Expression::IsNotNull(_)
            | expr @ Expression::IsEmpty(_)
            | expr @ Expression::IsNotEmpty(_) => {
                super::unary::evaluate_extended_unary_op(expr, context)
            }

            expr @ Expression::List(_) | expr @ Expression::Map(_) => {
                super::container::evaluate_container(expr, context)
            }

            Expression::TypeCasting { expr, target_type } => {
                let value = self.evaluate(expr, context)?;
                type_conversion::cast_value(value, target_type)
            }

            Expression::Case {
                conditions,
                default,
            } => {
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

            Expression::Aggregate {
                func,
                arg,
                distinct,
            } => {
                // 将 AggregateFunction 转换为字符串
                let func_str = format!("{:?}", func).to_lowercase();
                super::function::evaluate_aggregate(&func_str, arg, *distinct, context)
            }

            Expression::ListComprehension {
                generator,
                condition,
            } => {
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
            }

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
                    }
                    _ => Err(ExpressionError::TypeError(
                        "Predicate requires a list".to_string(),
                    )),
                }
            }

            Expression::Reduce {
                list,
                var,
                initial,
                expr,
            } => {
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
                    }
                    _ => Err(ExpressionError::TypeError(
                        "Reduce requires a list".to_string(),
                    )),
                }
            }

            Expression::PathBuild(items) => {
                // 路径构建的简化实现
                let mut path_items = Vec::new();
                for item in items {
                    path_items.push(self.evaluate(item, context)?);
                }
                Ok(Value::List(path_items)) // 简化为列表形式
            }

            Expression::ESQuery(query) => {
                // 文本搜索表达式，返回查询字符串
                Ok(Value::String(query.clone()))
            }

            Expression::UUID => {
                // 生成UUID的简化实现
                use uuid::Uuid;
                Ok(Value::String(Uuid::new_v4().to_string()))
            }

            Expression::Variable(var_name) => {
                // 从上下文变量中获取值
                if let Some(value) = context.vars.get(var_name) {
                    Ok(value.clone())
                } else {
                    Err(ExpressionError::PropertyNotFound(format!(
                        "Variable ${}",
                        var_name
                    )))
                }
            }

            Expression::Subscript { collection, index } => {
                let coll_value = self.evaluate(collection, context)?;
                let idx_value = self.evaluate(index, context)?;

                super::binary::subscript_values(coll_value, idx_value)
            }

            Expression::SubscriptRange {
                collection,
                start,
                end,
            } => {
                let coll_value = self.evaluate(collection, context)?;

                match coll_value {
                    Value::List(items) => {
                        let start_idx = if let Some(start_expr) = start {
                            let val = self.evaluate(start_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => {
                                    return Err(ExpressionError::TypeError(
                                        "Range start index must be an integer".to_string(),
                                    ))
                                }
                            }
                        } else {
                            0
                        };

                        let end_idx = if let Some(end_expr) = end {
                            let val = self.evaluate(end_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => {
                                    return Err(ExpressionError::TypeError(
                                        "Range end index must be an integer".to_string(),
                                    ))
                                }
                            }
                        } else {
                            items.len()
                        };

                        if start_idx > end_idx || end_idx > items.len() {
                            return Err(ExpressionError::InvalidOperation(
                                "Invalid range".to_string(),
                            ));
                        }

                        let result = items[start_idx..end_idx].to_vec();
                        Ok(Value::List(result))
                    }
                    Value::String(s) => {
                        let start_idx = if let Some(start_expr) = start {
                            let val = self.evaluate(start_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => {
                                    return Err(ExpressionError::TypeError(
                                        "Range start index must be an integer".to_string(),
                                    ))
                                }
                            }
                        } else {
                            0
                        };

                        let end_idx = if let Some(end_expr) = end {
                            let val = self.evaluate(end_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => {
                                    return Err(ExpressionError::TypeError(
                                        "Range end index must be an integer".to_string(),
                                    ))
                                }
                            }
                        } else {
                            s.len()
                        };

                        if start_idx > end_idx || end_idx > s.len() {
                            return Err(ExpressionError::InvalidOperation(
                                "Invalid range".to_string(),
                            ));
                        }

                        let result = s[start_idx..end_idx].to_string();
                        Ok(Value::String(result))
                    }
                    _ => Err(ExpressionError::TypeError(
                        "Subscript range requires a list or string".to_string(),
                    )),
                }
            }

            Expression::Label(label_name) => {
                // 标签表达式，返回标签名
                Ok(Value::String(label_name.clone()))
            }

            Expression::MatchPathPattern {
                path_alias,
                patterns: _,
            } => {
                // 匹配路径模式表达式，简化实现返回路径别名
                Ok(Value::String(path_alias.clone()))
            }

            Expression::Range {
                collection,
                start,
                end,
            } => {
                let coll_value = self.evaluate(collection, context)?;

                match coll_value {
                    Value::List(items) => {
                        let start_idx = if let Some(start_expr) = start {
                            let val = self.evaluate(start_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => {
                                    return Err(ExpressionError::TypeError(
                                        "Range start index must be an integer".to_string(),
                                    ))
                                }
                            }
                        } else {
                            0
                        };

                        let end_idx = if let Some(end_expr) = end {
                            let val = self.evaluate(end_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => {
                                    return Err(ExpressionError::TypeError(
                                        "Range end index must be an integer".to_string(),
                                    ))
                                }
                            }
                        } else {
                            items.len()
                        };

                        if start_idx > end_idx || end_idx > items.len() {
                            return Err(ExpressionError::InvalidOperation(
                                "Invalid range".to_string(),
                            ));
                        }

                        let result = items[start_idx..end_idx].to_vec();
                        Ok(Value::List(result))
                    }
                    Value::String(s) => {
                        let start_idx = if let Some(start_expr) = start {
                            let val = self.evaluate(start_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => {
                                    return Err(ExpressionError::TypeError(
                                        "Range start index must be an integer".to_string(),
                                    ))
                                }
                            }
                        } else {
                            0
                        };

                        let end_idx = if let Some(end_expr) = end {
                            let val = self.evaluate(end_expr, context)?;
                            match val {
                                Value::Int(n) => n as usize,
                                _ => {
                                    return Err(ExpressionError::TypeError(
                                        "Range end index must be an integer".to_string(),
                                    ))
                                }
                            }
                        } else {
                            s.len()
                        };

                        if start_idx > end_idx || end_idx > s.len() {
                            return Err(ExpressionError::InvalidOperation(
                                "Invalid range".to_string(),
                            ));
                        }

                        let result = s[start_idx..end_idx].to_string();
                        Ok(Value::String(result))
                    }
                    _ => Err(ExpressionError::TypeError(
                        "Range requires a list or string".to_string(),
                    )),
                }
            }

            Expression::Path(items) => {
                // 路径表达式，计算所有项并返回为列表
                let mut path_items = Vec::new();
                for item in items {
                    path_items.push(self.evaluate(item, context)?);
                }
                Ok(Value::List(path_items))
            }
        }
    }

    /// 直接评估Cypher表达式
    pub fn evaluate_cypher(
        &self,
        cypher_expr: &CypherExpression,
        context: &EvalContext,
    ) -> Result<Value, ExpressionError> {
        super::cypher::CypherEvaluator::evaluate_cypher(cypher_expr, context)
    }

    /// 批量评估Cypher表达式
    pub fn evaluate_cypher_batch(
        &self,
        cypher_exprs: &[CypherExpression],
        context: &EvalContext,
    ) -> Result<Vec<Value>, ExpressionError> {
        super::cypher::CypherEvaluator::evaluate_cypher_batch(cypher_exprs, context)
    }

    /// 检查Cypher表达式是否为常量
    pub fn is_cypher_constant(&self, cypher_expr: &CypherExpression) -> bool {
        super::cypher::CypherEvaluator::is_cypher_constant(cypher_expr)
    }

    /// 获取Cypher表达式中使用的所有变量
    pub fn get_cypher_variables(&self, cypher_expr: &CypherExpression) -> Vec<String> {
        super::cypher::CypherEvaluator::get_cypher_variables(cypher_expr)
    }

    /// 检查Cypher表达式是否包含聚合函数
    pub fn contains_cypher_aggregate(&self, cypher_expr: &CypherExpression) -> bool {
        super::cypher::CypherEvaluator::contains_cypher_aggregate(cypher_expr)
    }

    /// 优化Cypher表达式
    pub fn optimize_cypher_expression(
        &self,
        cypher_expr: &CypherExpression,
    ) -> CypherExpression {
        super::cypher::CypherExpressionOptimizer::optimize_cypher_expression(cypher_expr)
    }
}
