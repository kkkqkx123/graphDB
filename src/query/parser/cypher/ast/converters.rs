//! Cypher AST转换逻辑

use crate::core::error::QueryError;
use crate::core::types::operators::BinaryOperator;
use crate::core::value::Value;
use crate::core::vertex_edge_path::Tag;
use crate::query::parser::cypher::ast::expressions::Expression;
use crate::query::parser::cypher::ast::statements::CypherStatement;
use crate::query::parser::cypher::ast::{Condition, Query};
use std::collections::HashMap;

/// Cypher语句到查询的转换器
pub struct CypherConverter {
    /// 转换上下文，维护变量和别名
    context: ConversionContext,
}

/// 转换上下文
#[derive(Debug, Default)]
pub struct ConversionContext {
    /// 生成的别名映射
    aliases_generated: HashMap<String, AliasType>,
    /// 变量绑定
    variable_bindings: HashMap<String, Value>,
}

/// 别名类型
#[derive(Debug, Clone, PartialEq)]
pub enum AliasType {
    Node,
    Edge,
    Path,
    NodeList,
    EdgeList,
    Runtime,
}

/// 路径信息
#[derive(Debug, Clone)]
pub struct PathInfo {
    pub alias: Option<String>,
    pub node_infos: Vec<NodeInfo>,
    pub edge_infos: Vec<EdgeInfo>,
    pub direction: PathDirection,
}

/// 路径方向
#[derive(Debug, Clone, PartialEq)]
pub enum PathDirection {
    Outgoing,
    Incoming,
    Undirected,
}

/// 节点信息
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub alias: Option<String>,
    pub labels: Vec<String>,
    pub properties: HashMap<String, Expression>,
    pub anonymous: bool,
}

/// 边信息
#[derive(Debug, Clone)]
pub struct EdgeInfo {
    pub alias: Option<String>,
    pub edge_types: Vec<String>,
    pub properties: HashMap<String, Expression>,
    pub direction: PathDirection,
    pub anonymous: bool,
}

impl CypherConverter {
    /// 创建新的转换器
    pub fn new() -> Self {
        Self {
            context: ConversionContext::default(),
        }
    }

    /// 将Cypher语句转换为查询
    pub fn to_query(&mut self, statement: &CypherStatement) -> Result<Query, QueryError> {
        match statement {
            CypherStatement::Match(match_clause) => self.convert_match_clause(match_clause),
            CypherStatement::Query(query_clause) => self.convert_query_clause(query_clause),
            CypherStatement::Create(create_clause) => self.convert_create_clause(create_clause),
            CypherStatement::Delete(delete_clause) => self.convert_delete_clause(delete_clause),
            CypherStatement::Set(set_clause) => self.convert_set_clause(set_clause),
            CypherStatement::Return(return_clause) => self.convert_return_clause(return_clause),
            CypherStatement::With(with_clause) => self.convert_with_clause(with_clause),
            CypherStatement::Unwind(unwind_clause) => self.convert_unwind_clause(unwind_clause),
            _ => Err(QueryError::InvalidQuery(format!(
                "不支持的Cypher语句类型: {}",
                statement.statement_type()
            ))),
        }
    }

    /// 转换MATCH子句
    fn convert_match_clause(
        &mut self,
        match_clause: &crate::query::parser::cypher::ast::clauses::MatchClause,
    ) -> Result<Query, QueryError> {
        let mut paths = Vec::new();

        // 解析模式
        for pattern in &match_clause.patterns {
            let path_info = self.convert_pattern_to_path(pattern)?;
            paths.push(path_info);
        }

        // 解析WHERE条件
        let conditions = if let Some(where_clause) = &match_clause.where_clause {
            self.convert_where_expression(&where_clause.expression)?
        } else {
            Vec::new()
        };

        // 从路径中提取标签
        let tags = self.extract_tags_from_paths(&paths);

        Ok(Query::MatchNodes { tags, conditions })
    }

    /// 转换复合查询子句
    fn convert_query_clause(
        &mut self,
        query_clause: &crate::query::parser::cypher::ast::statements::QueryClause,
    ) -> Result<Query, QueryError> {
        if let Some(match_clause) = &query_clause.match_clause {
            self.convert_match_clause(match_clause)
        } else {
            Err(QueryError::InvalidQuery(
                "复合查询必须包含MATCH子句".to_string(),
            ))
        }
    }

    /// 转换CREATE子句
    fn convert_create_clause(
        &mut self,
        create_clause: &crate::query::parser::cypher::ast::clauses::CreateClause,
    ) -> Result<Query, QueryError> {
        if let Some(pattern) = create_clause.patterns.first() {
            if let Some(part) = pattern.parts.first() {
                let mut tags = Vec::new();

                // 为每个标签创建一个Tag对象
                for label in &part.node.labels {
                    let mut properties = HashMap::new();

                    // 从节点属性中提取属性
                    if let Some(node_props) = &part.node.properties {
                        for (key, expr) in node_props {
                            let evaluator =
                                ExpressionEvaluator::new(&self.context.variable_bindings);
                            let value = evaluator
                                .evaluate(expr)
                                .map_err(|e| QueryError::ExpressionError(e))?;
                            properties.insert(key.clone(), value);
                        }
                    }

                    tags.push(Tag::new(label.clone(), properties));
                }

                // 如果有别名，将其添加到上下文
                if let Some(alias) = &part.node.variable {
                    self.context
                        .aliases_generated
                        .insert(alias.clone(), AliasType::Node);
                }

                Ok(Query::CreateNode { id: None, tags })
            } else {
                Err(QueryError::InvalidQuery(
                    "无法解析CREATE语句：没有有效的节点模式".to_string(),
                ))
            }
        } else {
            Err(QueryError::InvalidQuery(
                "无法解析CREATE语句：没有模式".to_string(),
            ))
        }
    }

    /// 转换DELETE子句
    fn convert_delete_clause(
        &mut self,
        delete_clause: &crate::query::parser::cypher::ast::clauses::DeleteClause,
    ) -> Result<Query, QueryError> {
        if let Some(expr) = delete_clause.expressions.first() {
            let evaluator = ExpressionEvaluator::new(&self.context.variable_bindings);
            let id = evaluator
                .evaluate(expr)
                .map_err(|e| QueryError::ExpressionError(e))?;
            Ok(Query::DeleteNode { id })
        } else {
            Err(QueryError::InvalidQuery(
                "无法解析DELETE语句：没有表达式".to_string(),
            ))
        }
    }

    /// 转换SET子句
    fn convert_set_clause(
        &mut self,
        set_clause: &crate::query::parser::cypher::ast::clauses::SetClause,
    ) -> Result<Query, QueryError> {
        if let Some(set_item) = set_clause.items.first() {
            // 解析SET表达式
            let evaluator = ExpressionEvaluator::new(&self.context.variable_bindings);

            // 从SET项中提取节点ID和属性
            let (id, tags) = self.parse_set_item(set_item, &evaluator)?;

            Ok(Query::UpdateNode { id, tags })
        } else {
            Err(QueryError::InvalidQuery(
                "无法解析SET语句：没有SET项".to_string(),
            ))
        }
    }

    /// 转换RETURN子句
    fn convert_return_clause(
        &mut self,
        _return_clause: &crate::query::parser::cypher::ast::clauses::ReturnClause,
    ) -> Result<Query, QueryError> {
        // RETURN子句通常与前面的子句组合使用
        // 这里简化处理，实际应该与前面的查询结果结合
        Err(QueryError::InvalidQuery(
            "RETURN子句需要与前面的子句组合使用".to_string(),
        ))
    }

    /// 转换WITH子句
    fn convert_with_clause(
        &mut self,
        _with_clause: &crate::query::parser::cypher::ast::clauses::WithClause,
    ) -> Result<Query, QueryError> {
        // WITH子句用于传递结果到下一个查询部分
        Err(QueryError::InvalidQuery("WITH子句暂未实现".to_string()))
    }

    /// 转换UNWIND子句
    fn convert_unwind_clause(
        &mut self,
        _unwind_clause: &crate::query::parser::cypher::ast::clauses::UnwindClause,
    ) -> Result<Query, QueryError> {
        // UNWIND子句用于展开列表
        Err(QueryError::InvalidQuery("UNWIND子句暂未实现".to_string()))
    }

    /// 将模式转换为路径信息
    fn convert_pattern_to_path(
        &mut self,
        pattern: &crate::query::parser::cypher::ast::patterns::Pattern,
    ) -> Result<PathInfo, QueryError> {
        let mut node_infos = Vec::new();
        let mut edge_infos = Vec::new();

        for part in &pattern.parts {
            // 转换节点信息
            let node_info = NodeInfo {
                alias: part.node.variable.clone(),
                labels: part.node.labels.clone(),
                properties: part.node.properties.clone().unwrap_or_default(),
                anonymous: part.node.variable.is_none(),
            };

            // 如果节点有别名，添加到上下文
            if let Some(alias) = &node_info.alias {
                self.context
                    .aliases_generated
                    .insert(alias.clone(), AliasType::Node);
            }

            node_infos.push(node_info);

            // 转换边信息（如果存在）
            for relationship in &part.relationships {
                let edge_info = EdgeInfo {
                    alias: relationship.variable.clone(),
                    edge_types: relationship.types.clone(),
                    properties: relationship.properties.clone().unwrap_or_default(),
                    direction: self.convert_direction(&relationship.direction),
                    anonymous: relationship.variable.is_none(),
                };

                // 如果边有别名，添加到上下文
                if let Some(alias) = &edge_info.alias {
                    self.context
                        .aliases_generated
                        .insert(alias.clone(), AliasType::Edge);
                }

                edge_infos.push(edge_info);
            }
        }

        Ok(PathInfo {
            alias: None,
            node_infos,
            edge_infos,
            direction: PathDirection::Outgoing,
        })
    }

    /// 转换方向
    fn convert_direction(
        &self,
        direction: &crate::query::parser::cypher::ast::patterns::Direction,
    ) -> PathDirection {
        match direction {
            crate::query::parser::cypher::ast::patterns::Direction::Right => {
                PathDirection::Outgoing
            }
            crate::query::parser::cypher::ast::patterns::Direction::Left => PathDirection::Incoming,
            crate::query::parser::cypher::ast::patterns::Direction::Both => {
                PathDirection::Undirected
            }
        }
    }

    /// 转换WHERE表达式为条件列表
    fn convert_where_expression(&self, expr: &Expression) -> Result<Vec<Condition>, QueryError> {
        let mut conditions = Vec::new();
        self.extract_conditions_from_expression(expr, &mut conditions)?;
        Ok(conditions)
    }

    /// 从表达式中提取条件
    fn extract_conditions_from_expression(
        &self,
        expr: &Expression,
        conditions: &mut Vec<Condition>,
    ) -> Result<(), QueryError> {
        match expr {
            Expression::Binary(binary) => {
                match binary.operator {
                    BinaryOperator::Equal => {
                        if let (Expression::Property(prop), value_expr) =
                            (&*binary.left, &*binary.right)
                        {
                            let evaluator =
                                ExpressionEvaluator::new(&self.context.variable_bindings);
                            let value = evaluator
                                .evaluate(value_expr)
                                .map_err(|e| QueryError::ExpressionError(e))?;
                            conditions
                                .push(Condition::PropertyEquals(prop.property_name.clone(), value));
                        }
                    }
                    BinaryOperator::GreaterThan => {
                        if let (Expression::Property(prop), value_expr) =
                            (&*binary.left, &*binary.right)
                        {
                            let evaluator =
                                ExpressionEvaluator::new(&self.context.variable_bindings);
                            let value = evaluator
                                .evaluate(value_expr)
                                .map_err(|e| QueryError::ExpressionError(e))?;
                            conditions.push(Condition::PropertyGreaterThan(
                                prop.property_name.clone(),
                                value,
                            ));
                        }
                    }
                    BinaryOperator::LessThan => {
                        if let (Expression::Property(prop), value_expr) =
                            (&*binary.left, &*binary.right)
                        {
                            let evaluator =
                                ExpressionEvaluator::new(&self.context.variable_bindings);
                            let value = evaluator
                                .evaluate(value_expr)
                                .map_err(|e| QueryError::ExpressionError(e))?;
                            conditions.push(Condition::PropertyLessThan(
                                prop.property_name.clone(),
                                value,
                            ));
                        }
                    }
                    BinaryOperator::And => {
                        self.extract_conditions_from_expression(&*binary.left, conditions)?;
                        self.extract_conditions_from_expression(&*binary.right, conditions)?;
                    }
                    _ => {
                        // 其他操作符暂不支持
                        return Err(QueryError::ExpressionError(format!(
                            "不支持的操作符: {:?}",
                            binary.operator
                        )));
                    }
                }
            }
            _ => {
                return Err(QueryError::ExpressionError(format!(
                    "不支持的表达式类型: {:?}",
                    expr
                )));
            }
        }
        Ok(())
    }

    /// 从路径中提取标签
    fn extract_tags_from_paths(&self, paths: &[PathInfo]) -> Option<Vec<String>> {
        let mut labels = Vec::new();

        for path in paths {
            for node_info in &path.node_infos {
                labels.extend(node_info.labels.clone());
            }
        }

        if labels.is_empty() {
            None
        } else {
            Some(labels)
        }
    }

    /// 解析SET项
    fn parse_set_item(
        &self,
        set_item: &crate::query::parser::cypher::ast::clauses::SetItem,
        evaluator: &ExpressionEvaluator,
    ) -> Result<(Value, Vec<Tag>), QueryError> {
        // 解析左侧表达式（通常是属性访问）
        let variable_name = if let Expression::Property(prop_expr) = &set_item.left {
            if let Expression::Variable(var) = &*prop_expr.expression {
                var.clone()
            } else {
                return Err(QueryError::ExpressionError(
                    "SET左侧必须是变量属性".to_string(),
                ));
            }
        } else {
            return Err(QueryError::ExpressionError(
                "SET左侧必须是属性表达式".to_string(),
            ));
        };

        // 解析变量引用
        let id = if let Some(value) = self.context.variable_bindings.get(&variable_name) {
            value.clone()
        } else {
            Value::String(variable_name)
        };

        // 解析右侧表达式
        let value = evaluator
            .evaluate(&set_item.right)
            .map_err(|e| QueryError::ExpressionError(e))?;

        // 创建包含该属性的Tag
        let mut properties = HashMap::new();
        if let Expression::Property(prop_expr) = &set_item.left {
            properties.insert(prop_expr.property_name.clone(), value);
        }
        let tag = Tag::new("default".to_string(), properties);

        Ok((id, vec![tag]))
    }
}

impl Default for CypherConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// 表达式求值器
pub struct ExpressionEvaluator {
    variable_bindings: HashMap<String, Value>,
}

impl ExpressionEvaluator {
    /// 创建新的表达式求值器
    pub fn new(variable_bindings: &HashMap<String, Value>) -> Self {
        Self {
            variable_bindings: variable_bindings.clone(),
        }
    }

    /// 求值表达式
    pub fn evaluate(&self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Literal(literal) => self.evaluate_literal(literal),
            Expression::Variable(name) => self.evaluate_variable(name),
            Expression::Binary(binary) => self.evaluate_binary(binary),
            Expression::Unary(unary) => self.evaluate_unary(unary),
            Expression::FunctionCall(call) => self.evaluate_function_call(call),
            Expression::Property(prop) => self.evaluate_property(prop),
            Expression::List(list) => self.evaluate_list(list),
            Expression::Map(map) => self.evaluate_map(map),
            Expression::Case(case_expr) => self.evaluate_case(case_expr),
            _ => Err(format!("不支持的表达式类型: {:?}", expr)),
        }
    }

    /// 求值字面量
    fn evaluate_literal(
        &self,
        literal: &crate::query::parser::cypher::ast::expressions::Literal,
    ) -> Result<Value, String> {
        match literal {
            crate::query::parser::cypher::ast::expressions::Literal::String(s) => {
                Ok(Value::String(s.clone()))
            }
            crate::query::parser::cypher::ast::expressions::Literal::Integer(i) => {
                Ok(Value::Int(*i))
            }
            crate::query::parser::cypher::ast::expressions::Literal::Float(f) => {
                Ok(Value::Float(*f))
            }
            crate::query::parser::cypher::ast::expressions::Literal::Boolean(b) => {
                Ok(Value::Bool(*b))
            }
            crate::query::parser::cypher::ast::expressions::Literal::Null => {
                Ok(Value::Null(crate::core::value::NullType::Null))
            }
        }
    }

    /// 求值变量
    fn evaluate_variable(&self, name: &str) -> Result<Value, String> {
        self.variable_bindings
            .get(name)
            .cloned()
            .ok_or_else(|| format!("未定义的变量: {}", name))
    }

    /// 求值属性表达式
    fn evaluate_property(
        &self,
        prop: &crate::query::parser::cypher::ast::expressions::PropertyExpression,
    ) -> Result<Value, String> {
        // 简化处理：返回属性名作为字符串
        // 实际应该从对象中获取属性值
        Ok(Value::String(prop.property_name.clone()))
    }

    /// 求值列表表达式
    fn evaluate_list(
        &self,
        list: &crate::query::parser::cypher::ast::expressions::ListExpression,
    ) -> Result<Value, String> {
        let elements: Result<Vec<Value>, String> = list
            .elements
            .iter()
            .map(|elem| self.evaluate(elem))
            .collect();

        Ok(Value::List(elements?))
    }

    /// 求值Map表达式
    fn evaluate_map(
        &self,
        map: &crate::query::parser::cypher::ast::expressions::MapExpression,
    ) -> Result<Value, String> {
        let mut properties = HashMap::new();
        for (key, expr) in &map.properties {
            let value = self.evaluate(expr)?;
            properties.insert(key.clone(), value);
        }
        Ok(Value::Map(properties))
    }

    /// 求值二元表达式
    fn evaluate_binary(
        &self,
        binary: &crate::query::parser::cypher::ast::expressions::BinaryExpression,
    ) -> Result<Value, String> {
        let left = self.evaluate(&binary.left)?;
        let right = self.evaluate(&binary.right)?;

        match binary.operator {
            BinaryOperator::Add => left.add(&right),
            BinaryOperator::Subtract => left.sub(&right),
            BinaryOperator::Multiply => left.mul(&right),
            BinaryOperator::Divide => left.div(&right),
            BinaryOperator::Modulo => left.rem(&right),
            BinaryOperator::Exponent => match (&left, &right) {
                (Value::Int(l), Value::Int(r)) => {
                    let result = l.pow(*r as u32);
                    Ok(Value::Int(result))
                }
                (Value::Float(l), Value::Float(r)) => {
                    let result = l.powf(*r);
                    Ok(Value::Float(result))
                }
                (Value::Int(l), Value::Float(r)) => {
                    let result = (*l as f64).powf(*r);
                    Ok(Value::Float(result))
                }
                (Value::Float(l), Value::Int(r)) => {
                    let result = l.powf(*r as f64);
                    Ok(Value::Float(result))
                }
                _ => Err("指数操作符只能应用于数字".to_string()),
            },
            BinaryOperator::Equal => Ok(Value::Bool(left == right)),
            BinaryOperator::NotEqual => Ok(Value::Bool(left != right)),
            BinaryOperator::GreaterThan => Ok(Value::Bool(left > right)),
            BinaryOperator::LessThan => Ok(Value::Bool(left < right)),
            BinaryOperator::GreaterThanOrEqual => Ok(Value::Bool(left >= right)),
            BinaryOperator::LessThanOrEqual => Ok(Value::Bool(left <= right)),
            BinaryOperator::And => match (left, right) {
                (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l && r)),
                _ => Err("AND操作符只能应用于布尔值".to_string()),
            },
            BinaryOperator::Or => match (left, right) {
                (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l || r)),
                _ => Err("OR操作符只能应用于布尔值".to_string()),
            },
            BinaryOperator::Xor => match (left, right) {
                (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l ^ r)),
                _ => Err("XOR操作符只能应用于布尔值".to_string()),
            },
            BinaryOperator::In => match right {
                Value::List(list) => Ok(Value::Bool(list.contains(&left))),
                _ => Err("IN操作符的右侧必须是列表".to_string()),
            },
            BinaryOperator::NotIn => match right {
                Value::List(list) => Ok(Value::Bool(!list.contains(&left))),
                _ => Err("NOT IN操作符的右侧必须是列表".to_string()),
            },
            BinaryOperator::StartsWith => match (&left, &right) {
                (Value::String(s), Value::String(prefix)) => Ok(Value::Bool(s.starts_with(prefix))),
                _ => Err("STARTS WITH操作符只能应用于字符串".to_string()),
            },
            BinaryOperator::EndsWith => match (&left, &right) {
                (Value::String(s), Value::String(suffix)) => Ok(Value::Bool(s.ends_with(suffix))),
                _ => Err("ENDS WITH操作符只能应用于字符串".to_string()),
            },
            BinaryOperator::Contains => match (&left, &right) {
                (Value::String(s), Value::String(substr)) => Ok(Value::Bool(s.contains(substr))),
                _ => Err("CONTAINS操作符只能应用于字符串".to_string()),
            },
            BinaryOperator::Like => {
                match (&left, &right) {
                    (Value::String(s), Value::String(pattern)) => {
                        // 简化处理：使用基本的字符串包含来模拟正则匹配
                        // 实际应该使用regex crate
                        Ok(Value::Bool(s.contains(pattern)))
                    }
                    _ => Err("正则匹配操作符只能应用于字符串".to_string()),
                }
            }
            BinaryOperator::StringConcat => match (&left, &right) {
                (Value::String(s1), Value::String(s2)) => {
                    Ok(Value::String(format!("{}{}", s1, s2)))
                }
                _ => Err("字符串连接操作符只能应用于字符串".to_string()),
            },
            BinaryOperator::Subscript => match (&left, &right) {
                (Value::List(list), Value::Int(index)) => {
                    if *index >= 0 && (*index as usize) < list.len() {
                        Ok(list[*index as usize].clone())
                    } else {
                        Err("下标越界".to_string())
                    }
                }
                (Value::Map(map), Value::String(key)) => {
                    if let Some(value) = map.get(key) {
                        Ok(value.clone())
                    } else {
                        Err("键不存在".to_string())
                    }
                }
                _ => Err("下标操作符只能应用于列表或映射".to_string()),
            },
            BinaryOperator::Attribute => match (&left, &right) {
                (Value::Map(map), Value::String(key)) => {
                    if let Some(value) = map.get(key) {
                        Ok(value.clone())
                    } else {
                        Err("属性不存在".to_string())
                    }
                }
                _ => Err("属性访问操作符只能应用于映射".to_string()),
            },
            BinaryOperator::Union => match (&left, &right) {
                (Value::List(l1), Value::List(l2)) => {
                    let mut result = l1.clone();
                    result.extend(l2.clone());
                    Ok(Value::List(result))
                }
                _ => Err("并集操作符只能应用于列表".to_string()),
            },
            BinaryOperator::Intersect => match (&left, &right) {
                (Value::List(l1), Value::List(l2)) => {
                    let result: Vec<_> = l1
                        .iter()
                        .filter(|item| l2.contains(item))
                        .cloned()
                        .collect();
                    Ok(Value::List(result))
                }
                _ => Err("交集操作符只能应用于列表".to_string()),
            },
            BinaryOperator::Except => match (&left, &right) {
                (Value::List(l1), Value::List(l2)) => {
                    let result: Vec<_> = l1
                        .iter()
                        .filter(|item| !l2.contains(item))
                        .cloned()
                        .collect();
                    Ok(Value::List(result))
                }
                _ => Err("差集操作符只能应用于列表".to_string()),
            },
        }
    }

    /// 求值一元表达式
    fn evaluate_unary(
        &self,
        unary: &crate::query::parser::cypher::ast::expressions::UnaryExpression,
    ) -> Result<Value, String> {
        let expr = self.evaluate(&unary.expression)?;

        match unary.operator {
            crate::core::types::operators::UnaryOperator::Not => match expr {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err("NOT操作符只能应用于布尔值".to_string()),
            },
            crate::core::types::operators::UnaryOperator::Minus => expr.negate(),
            crate::core::types::operators::UnaryOperator::Plus => Ok(expr),
            crate::core::types::operators::UnaryOperator::IsNull => {
                Ok(Value::Bool(matches!(expr, Value::Null(_))))
            }
            crate::core::types::operators::UnaryOperator::IsNotNull => {
                Ok(Value::Bool(!matches!(expr, Value::Null(_))))
            }
            crate::core::types::operators::UnaryOperator::IsEmpty => {
                Err("IsEmpty操作符还未实现".to_string())
            }
            crate::core::types::operators::UnaryOperator::IsNotEmpty => {
                Err("IsNotEmpty操作符还未实现".to_string())
            }
            crate::core::types::operators::UnaryOperator::Increment => {
                Err("Increment操作符在此上下文中不支持".to_string())
            }
            crate::core::types::operators::UnaryOperator::Decrement => {
                Err("Decrement操作符在此上下文中不支持".to_string())
            }
        }
    }

    /// 求值函数调用
    fn evaluate_function_call(
        &self,
        call: &crate::query::parser::cypher::ast::expressions::FunctionCall,
    ) -> Result<Value, String> {
        let args: Result<Vec<Value>, String> = call
            .arguments
            .iter()
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
            "size" => {
                if args.len() != 1 {
                    return Err("size函数需要一个参数".to_string());
                }
                args[0].length()
            }
            "toString" => {
                if args.len() != 1 {
                    return Err("toString函数需要一个参数".to_string());
                }
                Ok(Value::String(format!("{:?}", args[0])))
            }
            "toInt" => {
                if args.len() != 1 {
                    return Err("toInt函数需要一个参数".to_string());
                }
                match &args[0] {
                    Value::String(s) => s
                        .parse::<i64>()
                        .map(Value::Int)
                        .map_err(|_| format!("无法将字符串 '{}' 转换为整数", s)),
                    Value::Float(f) => Ok(Value::Int(*f as i64)),
                    Value::Int(_) => Ok(args[0].clone()),
                    _ => Err("toInt函数只能应用于字符串或数字".to_string()),
                }
            }
            "toFloat" => {
                if args.len() != 1 {
                    return Err("toFloat函数需要一个参数".to_string());
                }
                match &args[0] {
                    Value::String(s) => s
                        .parse::<f64>()
                        .map(Value::Float)
                        .map_err(|_| format!("无法将字符串 '{}' 转换为浮点数", s)),
                    Value::Int(i) => Ok(Value::Float(*i as f64)),
                    Value::Float(_) => Ok(args[0].clone()),
                    _ => Err("toFloat函数只能应用于字符串或数字".to_string()),
                }
            }
            "toBoolean" => {
                if args.len() != 1 {
                    return Err("toBoolean函数需要一个参数".to_string());
                }
                match &args[0] {
                    Value::String(s) => match s.to_lowercase().as_str() {
                        "true" | "1" | "yes" | "on" => Ok(Value::Bool(true)),
                        "false" | "0" | "no" | "off" => Ok(Value::Bool(false)),
                        _ => Err(format!("无法将字符串 '{}' 转换为布尔值", s)),
                    },
                    Value::Int(i) => Ok(Value::Bool(*i != 0)),
                    Value::Float(f) => Ok(Value::Bool(*f != 0.0)),
                    Value::Bool(_) => Ok(args[0].clone()),
                    _ => Err("toBoolean函数只能应用于字符串或数字".to_string()),
                }
            }
            "substring" => {
                if args.len() < 2 || args.len() > 3 {
                    return Err("substring函数需要2或3个参数".to_string());
                }
                match &args[0] {
                    Value::String(s) => {
                        let start = if let Value::Int(i) = &args[1] {
                            *i as usize
                        } else {
                            return Err("substring函数的第二个参数必须是整数".to_string());
                        };

                        let end = if args.len() == 3 {
                            if let Value::Int(i) = &args[2] {
                                *i as usize
                            } else {
                                return Err("substring函数的第三个参数必须是整数".to_string());
                            }
                        } else {
                            s.len()
                        };

                        if start > end || end > s.len() {
                            return Err("substring函数的参数范围无效".to_string());
                        }

                        Ok(Value::String(s[start..end].to_string()))
                    }
                    _ => Err("substring函数只能应用于字符串".to_string()),
                }
            }
            "replace" => {
                if args.len() != 3 {
                    return Err("replace函数需要3个参数".to_string());
                }
                match &args[0] {
                    Value::String(s) => {
                        if let (Value::String(old), Value::String(new)) = (&args[1], &args[2]) {
                            Ok(Value::String(s.replace(old, new)))
                        } else {
                            Err("replace函数的第二和第三个参数必须是字符串".to_string())
                        }
                    }
                    _ => Err("replace函数只能应用于字符串".to_string()),
                }
            }
            "concat" => {
                if args.is_empty() {
                    return Err("concat函数至少需要一个参数".to_string());
                }
                let mut result = String::new();
                for arg in &args {
                    match arg {
                        Value::String(s) => result.push_str(s),
                        _ => result.push_str(&format!("{:?}", arg)),
                    }
                }
                Ok(Value::String(result))
            }
            "coalesce" => {
                if args.is_empty() {
                    return Err("coalesce函数至少需要一个参数".to_string());
                }
                for arg in &args {
                    if !arg.is_null() {
                        return Ok(arg.clone());
                    }
                }
                Ok(Value::Null(crate::core::value::NullType::Null))
            }
            "head" => {
                if args.len() != 1 {
                    return Err("head函数需要一个参数".to_string());
                }
                match &args[0] {
                    Value::List(list) => {
                        if list.is_empty() {
                            Ok(Value::Null(crate::core::value::NullType::Null))
                        } else {
                            Ok(list[0].clone())
                        }
                    }
                    _ => Err("head函数只能应用于列表".to_string()),
                }
            }
            "last" => {
                if args.len() != 1 {
                    return Err("last函数需要一个参数".to_string());
                }
                match &args[0] {
                    Value::List(list) => {
                        if list.is_empty() {
                            Ok(Value::Null(crate::core::value::NullType::Null))
                        } else {
                            Ok(list[list.len() - 1].clone())
                        }
                    }
                    _ => Err("last函数只能应用于列表".to_string()),
                }
            }
            "reverse" => {
                if args.len() != 1 {
                    return Err("reverse函数需要一个参数".to_string());
                }
                match &args[0] {
                    Value::List(list) => {
                        let mut reversed = list.clone();
                        reversed.reverse();
                        Ok(Value::List(reversed))
                    }
                    Value::String(s) => Ok(Value::String(s.chars().rev().collect())),
                    _ => Err("reverse函数只能应用于列表或字符串".to_string()),
                }
            }
            "keys" => {
                if args.len() != 1 {
                    return Err("keys函数需要一个参数".to_string());
                }
                match &args[0] {
                    Value::Map(map) => {
                        let keys: Vec<Value> =
                            map.keys().map(|k| Value::String(k.clone())).collect();
                        Ok(Value::List(keys))
                    }
                    _ => Err("keys函数只能应用于Map".to_string()),
                }
            }
            "values" => {
                if args.len() != 1 {
                    return Err("values函数需要一个参数".to_string());
                }
                match &args[0] {
                    Value::Map(map) => {
                        let values: Vec<Value> = map.values().cloned().collect();
                        Ok(Value::List(values))
                    }
                    _ => Err("values函数只能应用于Map".to_string()),
                }
            }
            "properties" => {
                if args.len() != 1 {
                    return Err("properties函数需要一个参数".to_string());
                }
                // 简化处理：直接返回输入值
                // 实际应该返回节点的属性Map
                Ok(args[0].clone())
            }
            _ => Err(format!("不支持的函数: {}", call.function_name)),
        }
    }

    /// 求值CASE表达式
    fn evaluate_case(
        &self,
        case_expr: &crate::query::parser::cypher::ast::expressions::CaseExpression,
    ) -> Result<Value, String> {
        // 如果有表达式，先求值表达式
        let test_value = if let Some(expr) = &case_expr.expression {
            Some(self.evaluate(expr)?)
        } else {
            None
        };

        // 检查每个WHEN-THEN分支
        for alternative in &case_expr.alternatives {
            let condition_match = if let Some(test_val) = &test_value {
                // 简单CASE形式：CASE test WHEN value1 THEN result1 WHEN value2 THEN result2
                let when_value = self.evaluate(&alternative.when_expression)?;
                test_val == &when_value
            } else {
                // 搜索CASE形式：CASE WHEN condition1 THEN result1 WHEN condition2 THEN result2
                match self.evaluate(&alternative.when_expression)? {
                    Value::Bool(b) => b,
                    _ => return Err("CASE WHEN条件必须返回布尔值".to_string()),
                }
            };

            if condition_match {
                return self.evaluate(&alternative.then_expression);
            }
        }

        // 如果没有分支匹配，返回ELSE表达式或NULL
        if let Some(else_expr) = &case_expr.default_alternative {
            self.evaluate(else_expr)
        } else {
            Ok(Value::Null(crate::core::value::NullType::Null))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::cypher::ast::clauses::*;
    use crate::query::parser::cypher::ast::expressions::*;
    use crate::query::parser::cypher::ast::patterns::*;

    #[test]
    fn test_simple_match_conversion() {
        let mut converter = CypherConverter::new();

        // 创建简单的MATCH语句: MATCH (n:Person) WHERE n.age > 18
        let match_clause = MatchClause {
            patterns: vec![Pattern {
                parts: vec![PatternPart {
                    node: NodePattern {
                        variable: Some("n".to_string()),
                        labels: vec!["Person".to_string()],
                        properties: None,
                    },
                    relationships: vec![],
                }],
            }],
            where_clause: Some(WhereClause {
                expression: Expression::Binary(BinaryExpression {
                    left: Box::new(Expression::Property(PropertyExpression {
                        expression: Box::new(Expression::Variable("n".to_string())),
                        property_name: "age".to_string(),
                    })),
                    operator: BinaryOperator::GreaterThan,
                    right: Box::new(Expression::Literal(Literal::Integer(18))),
                }),
            }),
            optional: false,
        };

        let statement = CypherStatement::Match(match_clause);
        let result = converter.to_query(&statement);

        assert!(result.is_ok());
        if let Ok(Query::MatchNodes { tags, conditions }) = result {
            assert_eq!(tags, Some(vec!["Person".to_string()]));
            assert_eq!(conditions.len(), 1);
            assert!(matches!(
                conditions[0],
                Condition::PropertyGreaterThan(_, _)
            ));
        }
    }

    #[test]
    fn test_create_conversion() {
        let mut converter = CypherConverter::new();

        // 创建CREATE语句: CREATE (p:Person {name: "Alice", age: 30})
        let create_clause = CreateClause {
            patterns: vec![Pattern {
                parts: vec![PatternPart {
                    node: NodePattern {
                        variable: Some("p".to_string()),
                        labels: vec!["Person".to_string()],
                        properties: Some({
                            let mut props = HashMap::new();
                            props.insert(
                                "name".to_string(),
                                Expression::Literal(Literal::String("Alice".to_string())),
                            );
                            props.insert(
                                "age".to_string(),
                                Expression::Literal(Literal::Integer(30)),
                            );
                            props
                        }),
                    },
                    relationships: vec![],
                }],
            }],
        };

        let statement = CypherStatement::Create(create_clause);
        let result = converter.to_query(&statement);

        assert!(result.is_ok());
        if let Ok(Query::CreateNode { id, tags }) = result {
            assert!(id.is_none());
            assert_eq!(tags.len(), 1);
            assert_eq!(tags[0].name, "Person");
            assert_eq!(tags[0].properties.len(), 2);
        }
    }

    #[test]
    fn test_expression_evaluator() {
        let mut bindings = HashMap::new();
        bindings.insert("x".to_string(), Value::Int(10));

        let evaluator = ExpressionEvaluator::new(&bindings);

        // 测试变量求值
        let var_expr = Expression::Variable("x".to_string());
        assert_eq!(
            evaluator
                .evaluate(&var_expr)
                .expect("Evaluator should evaluate variable expression"),
            Value::Int(10)
        );

        // 测试二元表达式
        let binary_expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(5))),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Literal::Integer(3))),
        });
        assert_eq!(
            evaluator
                .evaluate(&binary_expr)
                .expect("Evaluator should evaluate binary expression"),
            Value::Int(8)
        );

        // 测试函数调用
        let func_expr = Expression::FunctionCall(FunctionCall {
            function_name: "abs".to_string(),
            arguments: vec![Expression::Literal(Literal::Integer(-5))],
        });
        assert_eq!(
            evaluator
                .evaluate(&func_expr)
                .expect("Evaluator should evaluate function expression"),
            Value::Int(5)
        );
    }
}
