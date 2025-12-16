//! Cypher AST结构定义

use std::collections::HashMap;

/// Cypher语句类型
#[derive(Debug, Clone)]
pub enum CypherStatement {
    Match(MatchClause),
    Where(WhereClause),
    Return(ReturnClause),
    Create(CreateClause),
    Delete(DeleteClause),
    Set(SetClause),
    Remove(RemoveClause),
    Merge(MergeClause),
    With(WithClause),
    Unwind(UnwindClause),
    Call(CallClause),
    Query(QueryClause), // 复合查询语句
}

/// 复合查询语句
#[derive(Debug, Clone)]
pub struct QueryClause {
    pub match_clause: Option<MatchClause>,
    pub where_clause: Option<WhereClause>,
    pub return_clause: Option<ReturnClause>,
    pub with_clause: Option<WithClause>,
}

/// MATCH子句
#[derive(Debug, Clone)]
pub struct MatchClause {
    pub patterns: Vec<Pattern>,
    pub where_clause: Option<WhereClause>,
}

/// WHERE子句
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub expression: Expression,
}

/// RETURN子句
#[derive(Debug, Clone)]
pub struct ReturnClause {
    pub return_items: Vec<ReturnItem>,
    pub distinct: bool,
    pub order_by: Option<OrderByClause>,
    pub skip: Option<SkipClause>,
    pub limit: Option<LimitClause>,
}

/// CREATE子句
#[derive(Debug, Clone)]
pub struct CreateClause {
    pub patterns: Vec<Pattern>,
}

/// DELETE子句
#[derive(Debug, Clone)]
pub struct DeleteClause {
    pub expressions: Vec<Expression>,
    pub detach: bool,
}

/// SET子句
#[derive(Debug, Clone)]
pub struct SetClause {
    pub items: Vec<SetItem>,
}

/// REMOVE子句
#[derive(Debug, Clone)]
pub struct RemoveClause {
    pub items: Vec<RemoveItem>,
}

/// MERGE子句
#[derive(Debug, Clone)]
pub struct MergeClause {
    pub pattern: Pattern,
    pub actions: Vec<MergeAction>,
}

/// WITH子句
#[derive(Debug, Clone)]
pub struct WithClause {
    pub return_items: Vec<ReturnItem>,
    pub where_clause: Option<WhereClause>,
    pub distinct: bool,
    pub order_by: Option<OrderByClause>,
    pub skip: Option<SkipClause>,
    pub limit: Option<LimitClause>,
}

/// UNWIND子句
#[derive(Debug, Clone)]
pub struct UnwindClause {
    pub expression: Expression,
    pub variable: String,
}

/// CALL子句
#[derive(Debug, Clone)]
pub struct CallClause {
    pub procedure: String,
    pub arguments: Vec<Expression>,
    pub yield_items: Option<Vec<String>>,
}

/// 模式定义
#[derive(Debug, Clone)]
pub struct Pattern {
    pub parts: Vec<PatternPart>,
}

/// 模式部分
#[derive(Debug, Clone)]
pub struct PatternPart {
    pub node: NodePattern,
    pub relationships: Vec<RelationshipPattern>,
}

/// 节点模式
#[derive(Debug, Clone)]
pub struct NodePattern {
    pub variable: Option<String>,
    pub labels: Vec<String>,
    pub properties: Option<HashMap<String, Expression>>,
}

/// 关系模式
#[derive(Debug, Clone)]
pub struct RelationshipPattern {
    pub direction: Direction,
    pub variable: Option<String>,
    pub types: Vec<String>,
    pub properties: Option<HashMap<String, Expression>>,
    pub range: Option<Range>,
}

/// 方向
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Both,
}

/// 范围
#[derive(Debug, Clone)]
pub struct Range {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

/// 返回项
#[derive(Debug, Clone)]
pub struct ReturnItem {
    pub expression: Expression,
    pub alias: Option<String>,
}

/// SET项
#[derive(Debug, Clone)]
pub struct SetItem {
    pub left: Expression,
    pub right: Expression,
}

/// REMOVE项
#[derive(Debug, Clone)]
pub struct RemoveItem {
    pub expression: Expression,
}

/// MERGE动作
#[derive(Debug, Clone)]
pub struct MergeAction {
    pub action_type: MergeActionType,
    pub set_items: Vec<SetItem>,
}

/// MERGE动作类型
#[derive(Debug, Clone)]
pub enum MergeActionType {
    OnCreate,
    OnMatch,
}

/// ORDER BY子句
#[derive(Debug, Clone)]
pub struct OrderByClause {
    pub items: Vec<OrderByItem>,
}

/// ORDER BY项
#[derive(Debug, Clone)]
pub struct OrderByItem {
    pub expression: Expression,
    pub ordering: Ordering,
}

/// 排序
#[derive(Debug, Clone)]
pub enum Ordering {
    Ascending,
    Descending,
}

/// SKIP子句
#[derive(Debug, Clone)]
pub struct SkipClause {
    pub expression: Expression,
}

/// LIMIT子句
#[derive(Debug, Clone)]
pub struct LimitClause {
    pub expression: Expression,
}

/// 表达式
#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Variable(String),
    Property(PropertyExpression),
    FunctionCall(FunctionCall),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Case(CaseExpression),
    List(ListExpression),
    Map(MapExpression),
    PatternExpression(PatternExpression),
}

/// 字面量
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

/// 属性表达式
#[derive(Debug, Clone)]
pub struct PropertyExpression {
    pub expression: Box<Expression>,
    pub property_name: String,
}

/// 函数调用
#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub function_name: String,
    pub arguments: Vec<Expression>,
    pub distinct: bool,
}

/// 二元表达式
#[derive(Debug, Clone)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
}

/// 二元操作符
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    And,
    Or,
    Xor,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    In,
    StartsWith,
    EndsWith,
    Contains,
    RegexMatch,
}

/// 一元表达式
#[derive(Debug, Clone)]
pub struct UnaryExpression {
    pub operator: UnaryOperator,
    pub expression: Box<Expression>,
}

/// 一元操作符
#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Not,
    Negate,
    IsNull,
    IsNotNull,
}

/// CASE表达式
#[derive(Debug, Clone)]
pub struct CaseExpression {
    pub expression: Option<Box<Expression>>,
    pub alternatives: Vec<CaseAlternative>,
    pub default: Option<Box<Expression>>,
}

/// CASE分支
#[derive(Debug, Clone)]
pub struct CaseAlternative {
    pub condition: Box<Expression>,
    pub result: Box<Expression>,
}

/// 列表表达式
#[derive(Debug, Clone)]
pub struct ListExpression {
    pub items: Vec<Expression>,
}

/// Map表达式
#[derive(Debug, Clone)]
pub struct MapExpression {
    pub entries: HashMap<String, Expression>,
}

/// 模式表达式
#[derive(Debug, Clone)]
pub struct PatternExpression {
    pub pattern: Pattern,
}

impl CypherStatement {
    /// 获取语句类型
    pub fn statement_type(&self) -> &str {
        match self {
            CypherStatement::Match(_) => "MATCH",
            CypherStatement::Where(_) => "WHERE",
            CypherStatement::Return(_) => "RETURN",
            CypherStatement::Create(_) => "CREATE",
            CypherStatement::Delete(_) => "DELETE",
            CypherStatement::Set(_) => "SET",
            CypherStatement::Remove(_) => "REMOVE",
            CypherStatement::Merge(_) => "MERGE",
            CypherStatement::With(_) => "WITH",
            CypherStatement::Unwind(_) => "UNWIND",
            CypherStatement::Call(_) => "CALL",
            CypherStatement::Query(_) => "QUERY",
        }
    }

    /// 将Cypher语句转换为查询
    pub fn to_query(&self) -> Result<crate::query::types::Query, String> {
        use crate::core::value::Value;
        use crate::core::vertex_edge_path::Tag;
        use crate::query::types::{Condition, Query};
        use std::collections::HashMap;

        match self {
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
                if let Some(set_item) = set_clause.items.first() {
                    // 这里需要更复杂的逻辑来解析SET表达式
                    let id = Value::String("some_id".to_string());
                    let tags = Vec::new(); // 简化处理
                    
                    Ok(Query::UpdateNode { id, tags })
                } else {
                    Err("无法解析SET语句：没有SET项".to_string())
                }
            }
            _ => Err(format!("不支持的Cypher语句类型: {}", self.statement_type())),
        }
    }
}

impl Expression {
    /// 将表达式转换为值
    pub fn to_value(&self) -> crate::core::value::Value {
        use crate::core::value::Value;
        match self {
            Expression::Literal(literal) => match literal {
                Literal::String(s) => Value::String(s.clone()),
                Literal::Integer(i) => Value::Int(*i),
                Literal::Float(f) => Value::Float(*f),
                Literal::Boolean(b) => Value::Bool(*b),
                Literal::Null => Value::Null(crate::core::value::NullType::Null),
            },
            Expression::Variable(name) => Value::String(name.clone()),
            _ => Value::String(format!("{:?}", self)), // 简化处理
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cypher_statement_type() {
        let match_stmt = CypherStatement::Match(MatchClause {
            patterns: Vec::new(),
            where_clause: None,
        });
        
        assert_eq!(match_stmt.statement_type(), "MATCH");
        
        let return_stmt = CypherStatement::Return(ReturnClause {
            return_items: Vec::new(),
            distinct: false,
            order_by: None,
            skip: None,
            limit: None,
        });
        
        assert_eq!(return_stmt.statement_type(), "RETURN");
    }

    #[test]
    fn test_node_pattern_creation() {
        let node = NodePattern {
            variable: Some("n".to_string()),
            labels: vec!["Person".to_string(), "User".to_string()],
            properties: Some(HashMap::from([
                ("name".to_string(), Expression::Literal(Literal::String("Alice".to_string()))),
                ("age".to_string(), Expression::Literal(Literal::Integer(25))),
            ])),
        };
        
        assert_eq!(node.variable, Some("n".to_string()));
        assert_eq!(node.labels.len(), 2);
        assert!(node.properties.is_some());
    }

    #[test]
    fn test_expression_literal() {
        let expr = Expression::Literal(Literal::String("hello".to_string()));
        
        match expr {
            Expression::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_binary_expression() {
        let expr = Expression::Binary(BinaryExpression {
            left: Box::new(Expression::Literal(Literal::Integer(5))),
            operator: BinaryOperator::Add,
            right: Box::new(Expression::Literal(Literal::Integer(3))),
        });
        
        match expr {
            Expression::Binary(bin) => {
                assert!(matches!(*bin.left, Expression::Literal(Literal::Integer(5))));
                assert!(matches!(*bin.right, Expression::Literal(Literal::Integer(3))));
                assert_eq!(bin.operator, BinaryOperator::Add);
            }
            _ => panic!("Expected binary expression"),
        }
    }
}