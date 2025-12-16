//! Cypher AST转换逻辑

use crate::core::value::Value;
use crate::core::vertex_edge_path::Tag;
use crate::query::types::{Condition, Query};
use crate::query::parser::cypher::ast::statements::CypherStatement;
use std::collections::HashMap;

/// Cypher语句到查询的转换器
pub struct CypherConverter;

impl CypherConverter {
    /// 将Cypher语句转换为查询
    pub fn to_query(statement: &CypherStatement) -> Result<Query, String> {
        match statement {
            CypherStatement::Match(match_clause) => {
                let mut tags = None;
                let mut conditions = Vec::new();

                // 从模式中提取标签
                for pattern in &match_clause.patterns {
                    for part in &pattern.parts {
                        if !part.node.labels.is_empty() {
                            tags = Some(part.node.labels.clone());
                            break;
                        }
                    }
                    if tags.is_some() {
                        break;
                    }
                }

                // 从WHERE子句中提取条件
                if let Some(where_clause) = &match_clause.where_clause {
                    // 简化处理：将表达式转换为条件
                    // 这里需要更复杂的逻辑来解析WHERE表达式
                    conditions.push(Condition::PropertyGreaterThan(
                        "age".to_string(),
                        Value::Int(18),
                    ));
                }

                Ok(Query::MatchNodes { tags, conditions })
            }
            CypherStatement::Query(query_clause) => {
                // 处理复合查询语句
                if let Some(match_clause) = &query_clause.match_clause {
                    let mut tags = None;
                    let mut conditions = Vec::new();

                    // 从模式中提取标签
                    for pattern in &match_clause.patterns {
                        for part in &pattern.parts {
                            if !part.node.labels.is_empty() {
                                tags = Some(part.node.labels.clone());
                                break;
                            }
                        }
                        if tags.is_some() {
                            break;
                        }
                    }

                    // 从WHERE子句中提取条件
                    if let Some(where_clause) = &match_clause.where_clause {
                        // 简化处理：将表达式转换为条件
                        conditions.push(Condition::PropertyGreaterThan(
                            "age".to_string(),
                            Value::Int(18),
                        ));
                    }

                    Ok(Query::MatchNodes { tags, conditions })
                } else {
                    Err("复合查询必须包含MATCH子句".to_string())
                }
            }
            CypherStatement::Create(create_clause) => {
                // 简化处理：从第一个节点模式创建节点
                if let Some(pattern) = create_clause.patterns.first() {
                    if let Some(part) = pattern.parts.first() {
                        let mut tags = Vec::new();
                        
                        // 为每个标签创建一个Tag对象
                        for label in &part.node.labels {
                            let mut properties = HashMap::new();
                            
                            // 从节点属性中提取属性
                            if let Some(node_props) = &part.node.properties {
                                for (key, expr) in node_props {
                                    properties.insert(key.clone(), expr.to_value());
                                }
                            }
                            
                            tags.push(Tag::new(label.clone(), properties));
                        }
                        
                        Ok(Query::CreateNode { id: None, tags })
                    } else {
                        Err("无法解析CREATE语句：没有有效的节点模式".to_string())
                    }
                } else {
                    Err("无法解析CREATE语句：没有模式".to_string())
                }
            }
            CypherStatement::Delete(delete_clause) => {
                // 简化处理：假设第一个表达式是节点ID
                if let Some(expr) = delete_clause.expressions.first() {
                    let id = expr.to_value();
                    Ok(Query::DeleteNode { id })
                } else {
                    Err("无法解析DELETE语句：没有表达式".to_string())
                }
            }
            CypherStatement::Set(set_clause) => {
                // SET语句转换为更新节点
                // 简化处理：假设第一个SET项包含节点ID
                if let Some(_set_item) = set_clause.items.first() {
                    // 这里需要更复杂的逻辑来解析SET表达式
                    let id = Value::String("some_id".to_string());
                    let tags = Vec::new(); // 简化处理
                    
                    Ok(Query::UpdateNode { id, tags })
                } else {
                    Err("无法解析SET语句：没有SET项".to_string())
                }
            }
            _ => Err(format!("不支持的Cypher语句类型: {}", statement.statement_type())),
        }
    }
}

/// 表达式求值器
pub struct ExpressionEvaluator {
    context: HashMap<String, Value>,
}

impl ExpressionEvaluator {
    /// 创建新的表达式求值器
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
        }
    }

    /// 设置上下文变量
    pub fn set_context(&mut self, context: HashMap<String, Value>) {
        self.context = context;
    }

    /// 求值表达式
    pub fn evaluate(&self, expr: &crate::query::parser::cypher::ast::expressions::Expression) -> Result<Value, String> {
        use crate::query::parser::cypher::ast::expressions::Expression;
        
        match expr {
            Expression::Literal(literal) => self.evaluate_literal(literal),
            Expression::Variable(name) => self.evaluate_variable(name),
            Expression::Binary(binary) => self.evaluate_binary(binary),
            Expression::Unary(unary) => self.evaluate_unary(unary),
            Expression::FunctionCall(call) => self.evaluate_function_call(call),
            _ => Err(format!("不支持的表达式类型: {:?}", expr)),
        }
    }

    /// 求值字面量
    fn evaluate_literal(&self, literal: &crate::query::parser::cypher::ast::expressions::Literal) -> Result<Value, String> {
        use crate::query::parser::cypher::ast::expressions::Literal;
        
        match literal {
            Literal::String(s) => Ok(Value::String(s.clone())),
            Literal::Integer(i) => Ok(Value::Int(*i)),
            Literal::Float(f) => Ok(Value::Float(*f)),
            Literal::Boolean(b) => Ok(Value::Bool(*b)),
            Literal::Null => Ok(Value::Null(crate::core::value::NullType::Null)),
        }
    }

    /// 求值变量
    fn evaluate_variable(&self, name: &str) -> Result<Value, String> {
        self.context.get(name)
            .cloned()
            .ok_or_else(|| format!("未定义的变量: {}", name))
    }

    /// 求值二元表达式
    fn evaluate_binary(&self, binary: &crate::query::parser::cypher::ast::expressions::BinaryExpression) -> Result<Value, String> {
        use crate::query::parser::cypher::ast::expressions::BinaryOperator;
        
        let left = self.evaluate(&binary.left)?;
        let right = self.evaluate(&binary.right)?;
        
        match binary.operator {
            BinaryOperator::Add => left.add(&right),
            BinaryOperator::Subtract => left.sub(&right),
            BinaryOperator::Multiply => left.mul(&right),
            BinaryOperator::Divide => left.div(&right),
            BinaryOperator::Equal => Ok(Value::Bool(left.equals(&right))),
            BinaryOperator::NotEqual => Ok(Value::Bool(!left.equals(&right))),
            BinaryOperator::GreaterThan => Ok(Value::Bool(left.greater_than(&right))),
            BinaryOperator::LessThan => Ok(Value::Bool(left.less_than(&right))),
            _ => Err(format!("不支持的操作符: {:?}", binary.operator)),
        }
    }

    /// 求值一元表达式
    fn evaluate_unary(&self, unary: &crate::query::parser::cypher::ast::expressions::UnaryExpression) -> Result<Value, String> {
        use crate::query::parser::cypher::ast::expressions::UnaryOperator;
        
        let expr = self.evaluate(&unary.expression)?;
        
        match unary.operator {
            UnaryOperator::Not => {
                match expr {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => Err("NOT操作符只能应用于布尔值".to_string()),
                }
            }
            UnaryOperator::Negative => expr.negate(),
            _ => Err(format!("不支持的一元操作符: {:?}", unary.operator)),
        }
    }

    /// 求值函数调用
    fn evaluate_function_call(&self, call: &crate::query::parser::cypher::ast::expressions::FunctionCall) -> Result<Value, String> {
        let args: Result<Vec<Value>, String> = call.arguments.iter()
            .map(|arg| self.evaluate(arg))
            .collect();
        
        let args = args?;
        
        match call.function_name.as_str() {
            "abs" => {
                if args.len() != 1 {
                    return Err("abs函数需要一个参数".to_string());
                }
                args[0].abs()
            }
            "length" => {
                if args.len() != 1 {
                    return Err("length函数需要一个参数".to_string());
                }
                args[0].length()
            }
            _ => Err(format!("不支持的函数: {}", call.function_name)),
        }
    }
}

impl Default for ExpressionEvaluator {
    fn default() -> Self {
        Self::new()
    }
}