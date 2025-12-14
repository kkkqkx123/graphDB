//! 类型定义模块
//!
//! 定义 AST 中使用的各种辅助类型和结构。

use crate::core::Value;
use super::{Expression, EdgeRange};
use std::fmt;
use std::collections::HashMap;

/// 数据类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Bool,
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Float,
    Double,
    String,
    FixedString(usize),
    Timestamp,
    Date,
    Time,
    DateTime,
    Duration,
    Geography,
    Point,
    LineString,
    Polygon,
    List(Box<DataType>),
    Map(Box<DataType>, Box<DataType>),
    Set(Box<DataType>),
}

impl DataType {
    /// 检查是否是数值类型
    pub fn is_numeric(&self) -> bool {
        matches!(self, 
            DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
            DataType::Float | DataType::Double
        )
    }
    
    /// 检查是否是字符串类型
    pub fn is_string(&self) -> bool {
        matches!(self, DataType::String | DataType::FixedString(_))
    }
    
    /// 检查是否是时间类型
    pub fn is_temporal(&self) -> bool {
        matches!(self, 
            DataType::Timestamp | DataType::Date | DataType::Time | DataType::DateTime | DataType::Duration
        )
    }
    
    /// 检查是否是几何类型
    pub fn is_geometric(&self) -> bool {
        matches!(self, DataType::Geography | DataType::Point | DataType::LineString | DataType::Polygon)
    }
    
    /// 检查是否是集合类型
    pub fn is_collection(&self) -> bool {
        matches!(self, DataType::List(_) | DataType::Map(_, _) | DataType::Set(_))
    }
    
    /// 获取类型的默认零值
    pub fn default_value(&self) -> Value {
        match self {
            DataType::Bool => Value::Bool(false),
            DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 => {
                Value::Int(0)
            }
            DataType::Float | DataType::Double => Value::Float(0.0),
            DataType::String | DataType::FixedString(_) => Value::String(String::new()),
            DataType::Timestamp => Value::Int(0),
            DataType::Date => Value::String("1970-01-01".to_string()),
            DataType::Time => Value::String("00:00:00".to_string()),
            DataType::DateTime => Value::String("1970-01-01T00:00:00".to_string()),
            DataType::Duration => Value::Int(0),
            DataType::Geography | DataType::Point | DataType::LineString | DataType::Polygon => {
                Value::Null(crate::core::NullType::Null)
            }
            DataType::List(inner_type) => {
                // 创建包含默认值的列表
                Value::List(vec![inner_type.default_value()])
            }
            DataType::Map(key_type, value_type) => {
                // 创建包含默认键值对的映射
                let key = key_type.default_value();
                let value = value_type.default_value();
                let mut map = HashMap::new();
                map.insert(key.to_string(), value);
                Value::Map(map)
            }
            DataType::Set(inner_type) => {
                // 创建包含默认值的集合
                Value::List(vec![inner_type.default_value()])
            }
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::Bool => write!(f, "BOOL"),
            DataType::Int => write!(f, "INT"),
            DataType::Int8 => write!(f, "INT8"),
            DataType::Int16 => write!(f, "INT16"),
            DataType::Int32 => write!(f, "INT32"),
            DataType::Int64 => write!(f, "INT64"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::String => write!(f, "STRING"),
            DataType::FixedString(len) => write!(f, "FIXED_STRING({})", len),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
            DataType::Date => write!(f, "DATE"),
            DataType::Time => write!(f, "TIME"),
            DataType::DateTime => write!(f, "DATETIME"),
            DataType::Duration => write!(f, "DURATION"),
            DataType::Geography => write!(f, "GEOGRAPHY"),
            DataType::Point => write!(f, "POINT"),
            DataType::LineString => write!(f, "LINESTRING"),
            DataType::Polygon => write!(f, "POLYGON"),
            DataType::List(inner) => write!(f, "LIST<{}>", inner),
            DataType::Map(key, value) => write!(f, "MAP<{}, {}>", key, value),
            DataType::Set(inner) => write!(f, "SET<{}>", inner),
        }
    }
}

/// 属性定义
#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default_value: Option<Value>,
    pub comment: Option<String>,
}

impl Property {
    pub fn new(name: String, data_type: DataType) -> Self {
        Self {
            name,
            data_type,
            nullable: true,
            default_value: None,
            comment: None,
        }
    }
    
    pub fn with_nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }
    
    pub fn with_default(mut self, default: Value) -> Self {
        self.default_value = Some(default);
        self
    }
    
    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// 标签定义
#[derive(Debug, Clone, PartialEq)]
pub struct TagDefinition {
    pub name: String,
    pub properties: Vec<Property>,
    pub comment: Option<String>,
}

impl TagDefinition {
    pub fn new(name: String) -> Self {
        Self {
            name,
            properties: Vec::new(),
            comment: None,
        }
    }
    
    pub fn with_properties(mut self, properties: Vec<Property>) -> Self {
        self.properties = properties;
        self
    }
    
    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// 边类型定义
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeTypeDefinition {
    pub name: String,
    pub source_tags: Vec<String>,
    pub target_tags: Vec<String>,
    pub properties: Vec<Property>,
    pub comment: Option<String>,
}

impl EdgeTypeDefinition {
    pub fn new(name: String) -> Self {
        Self {
            name,
            source_tags: Vec::new(),
            target_tags: Vec::new(),
            properties: Vec::new(),
            comment: None,
        }
    }
    
    pub fn with_source_tags(mut self, tags: Vec<String>) -> Self {
        self.source_tags = tags;
        self
    }
    
    pub fn with_target_tags(mut self, tags: Vec<String>) -> Self {
        self.target_tags = tags;
        self
    }
    
    pub fn with_properties(mut self, properties: Vec<Property>) -> Self {
        self.properties = properties;
        self
    }
    
    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// 索引定义
#[derive(Debug, Clone, PartialEq)]
pub struct IndexDefinition {
    pub name: String,
    pub on_type: String, // 标签或边类型名称
    pub on_property: String,
    pub index_type: IndexType,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IndexType {
    Single,
    Composite(Vec<String>),
    FullText,
    Unique,
}

impl IndexDefinition {
    pub fn new(name: String, on_type: String, on_property: String) -> Self {
        Self {
            name,
            on_type,
            on_property,
            index_type: IndexType::Single,
            comment: None,
        }
    }
    
    pub fn with_index_type(mut self, index_type: IndexType) -> Self {
        self.index_type = index_type;
        self
    }
    
    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// 函数定义
#[derive(Debug)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<DataType>,
    pub body: Option<Expr>,
    pub comment: Option<String>,
}

impl Clone for FunctionDefinition {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            parameters: self.parameters.clone(),
            return_type: self.return_type.clone(),
            body: self.body.as_ref().map(|expr| super::Expression::clone_box(expr)),
            comment: self.comment.clone(),
        }
    }
}

impl PartialEq for FunctionDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name &&
        self.parameters == other.parameters &&
        self.return_type == other.return_type &&
        self.body.is_some() == other.body.is_some() &&
        self.comment == other.comment
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub data_type: DataType,
    pub default_value: Option<Value>,
}

impl FunctionDefinition {
    pub fn new(name: String) -> Self {
        Self {
            name,
            parameters: Vec::new(),
            return_type: None,
            body: None,
            comment: None,
        }
    }
    
    pub fn with_parameters(mut self, parameters: Vec<Parameter>) -> Self {
        self.parameters = parameters;
        self
    }
    
    pub fn with_return_type(mut self, return_type: DataType) -> Self {
        self.return_type = Some(return_type);
        self
    }
    
    pub fn with_body(mut self, body: Expr) -> Self {
        self.body = Some(body);
        self
    }
    
    pub fn with_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }
}

/// 类型工具函数
pub struct TypeUtils;

impl TypeUtils {
    /// 检查两个类型是否兼容
    pub fn are_compatible(type1: &DataType, type2: &DataType) -> bool {
        match (type1, type2) {
            (DataType::Int, DataType::Int8) | (DataType::Int8, DataType::Int) => true,
            (DataType::Int, DataType::Int16) | (DataType::Int16, DataType::Int) => true,
            (DataType::Int, DataType::Int32) | (DataType::Int32, DataType::Int) => true,
            (DataType::Int, DataType::Int64) | (DataType::Int64, DataType::Int) => true,
            (DataType::Float, DataType::Double) | (DataType::Double, DataType::Float) => true,
            _ => type1 == type2,
        }
    }
    
    /// 获取类型的优先级
    pub fn get_precedence(data_type: &DataType) -> u8 {
        match data_type {
            DataType::Bool => 1,
            DataType::Int8 => 2,
            DataType::Int16 => 3,
            DataType::Int32 => 4,
            DataType::Int64 | DataType::Int => 5,
            DataType::Float => 6,
            DataType::Double => 7,
            DataType::String | DataType::FixedString(_) => 8,
            DataType::Date => 9,
            DataType::Time => 10,
            DataType::DateTime => 11,
            DataType::Timestamp => 12,
            DataType::Duration => 13,
            DataType::Geography => 14,
            DataType::Point => 15,
            DataType::LineString => 16,
            DataType::Polygon => 17,
            DataType::List(_) => 18,
            DataType::Map(_, _) => 19,
            DataType::Set(_) => 20,
        }
    }
    
    /// 获取两个类型中更通用的类型
    pub fn get_common_type(type1: &DataType, type2: &DataType) -> DataType {
        if type1 == type2 {
            return type1.clone();
        }
        
        let prec1 = Self::get_precedence(type1);
        let prec2 = Self::get_precedence(type2);
        
        if prec1 > prec2 {
            type1.clone()
        } else {
            type2.clone()
        }
    }
    
    /// 检查值是否符合数据类型
    pub fn is_value_compatible(value: &Value, data_type: &DataType) -> bool {
        match (value, data_type) {
            (Value::Bool(_), DataType::Bool) => true,
            (Value::Int(_), DataType::Int | DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64) => true,
            (Value::Float(_), DataType::Float | DataType::Double) => true,
            (Value::String(_), DataType::String | DataType::FixedString(_)) => true,
            (Value::List(_), DataType::List(_)) => true,
            (Value::Map(_), DataType::Map(_, _)) => true,
            (Value::Null(_), _) => true, // NULL 可以赋值给任何类型
            _ => false,
        }
    }
}

/// 类型工厂
pub struct TypeFactory;

impl TypeFactory {
    /// 创建基本类型
    pub fn bool() -> DataType {
        DataType::Bool
    }
    
    pub fn int() -> DataType {
        DataType::Int
    }
    
    pub fn float() -> DataType {
        DataType::Float
    }
    
    pub fn string() -> DataType {
        DataType::String
    }
    
    /// 创建列表类型
    pub fn list(inner_type: DataType) -> DataType {
        DataType::List(Box::new(inner_type))
    }
    
    /// 创建映射类型
    pub fn map(key_type: DataType, value_type: DataType) -> DataType {
        DataType::Map(Box::new(key_type), Box::new(value_type))
    }
    
    /// 创建集合类型
    pub fn set(inner_type: DataType) -> DataType {
        DataType::Set(Box::new(inner_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_data_type_creation() {
        let int_type = DataType::Int;
        assert!(int_type.is_numeric());
        assert!(!int_type.is_string());
        
        let string_type = DataType::String;
        assert!(!string_type.is_numeric());
        assert!(string_type.is_string());
    }
    
    #[test]
    fn test_type_compatibility() {
        let int_type = DataType::Int;
        let int8_type = DataType::Int8;
        
        assert!(TypeUtils::are_compatible(&int_type, &int8_type));
        assert!(TypeUtils::are_compatible(&int8_type, &int_type));
        
        let string_type = DataType::String;
        assert!(!TypeUtils::are_compatible(&int_type, &string_type));
    }
    
    #[test]
    fn test_type_precedence() {
        let int_type = DataType::Int;
        let float_type = DataType::Float;
        
        let prec_int = TypeUtils::get_precedence(&int_type);
        let prec_float = TypeUtils::get_precedence(&float_type);
        
        assert!(prec_float > prec_int);
    }
    
    #[test]
    fn test_common_type() {
        let int_type = DataType::Int;
        let float_type = DataType::Float;
        
        let common = TypeUtils::get_common_type(&int_type, &float_type);
        assert_eq!(common, DataType::Float);
    }
}

/// 标签标识符
pub type TagIdentifier = String;

/// MATCH 语句子句详情
#[derive(Debug)]
pub struct MatchClause {
    pub patterns: Vec<Pattern>,
    pub where_clause: Option<WhereClause>,
    pub with_clause: Option<WithClause>,
}

impl Clone for MatchClause {
    fn clone(&self) -> Self {
        Self {
            patterns: self.patterns.clone(),
            where_clause: self.where_clause.clone(),
            with_clause: self.with_clause.clone(),
        }
    }
}

impl PartialEq for MatchClause {
    fn eq(&self, other: &Self) -> bool {
        self.patterns == other.patterns &&
        self.where_clause == other.where_clause &&
        self.with_clause == other.with_clause
    }
}

/// MATCH 路径
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub path: Vec<PatternSegment>,
}

/// MATCH 路径段
#[derive(Debug, Clone, PartialEq)]
pub enum PatternSegment {
    Node(MatchNode),
    Edge(MatchEdge),
}

/// MATCH 节点
#[derive(Debug)]
pub struct MatchNode {
    pub identifier: Option<String>,
    pub labels: Vec<Label>,
    pub properties: Option<Expr>,
    pub predicates: Vec<Expr>,
}

impl Clone for MatchNode {
    fn clone(&self) -> Self {
        Self {
            identifier: self.identifier.clone(),
            labels: self.labels.clone(),
            properties: self.properties.as_ref().map(|expr| super::Expression::clone_box(expr)),
            predicates: self.predicates.iter().map(|expr| super::Expression::clone_box(expr)).collect(),
        }
    }
}

impl PartialEq for MatchNode {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier &&
        self.labels == other.labels &&
        self.properties.is_some() == other.properties.is_some() &&
        self.predicates.len() == other.predicates.len()
    }
}

/// MATCH 边
#[derive(Debug)]
pub struct MatchEdge {
    pub direction: crate::query::parser::ast::stmt::EdgeDirection,
    pub identifier: Option<String>,
    pub types: Vec<String>,
    pub relationship: Option<String>,
    pub properties: Option<Expr>,
    pub predicates: Vec<Expr>,
    pub range: Option<EdgeRange>,
}

impl Clone for MatchEdge {
    fn clone(&self) -> Self {
        Self {
            direction: self.direction.clone(),
            identifier: self.identifier.clone(),
            types: self.types.clone(),
            relationship: self.relationship.clone(),
            properties: self.properties.as_ref().map(|expr| super::Expression::clone_box(expr)),
            predicates: self.predicates.iter().map(|expr| super::Expression::clone_box(expr)).collect(),
            range: self.range.clone(),
        }
    }
}

impl PartialEq for MatchEdge {
    fn eq(&self, other: &Self) -> bool {
        self.direction == other.direction &&
        self.identifier == other.identifier &&
        self.types == other.types &&
        self.relationship == other.relationship &&
        self.properties.is_some() == other.properties.is_some() &&
        self.predicates.len() == other.predicates.len() &&
        self.range == other.range
    }
}

/// 标签
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub name: String,
}

/// WHERE 子句
#[derive(Debug)]
pub struct WhereClause {
    pub expression: Expr,
}

impl Clone for WhereClause {
    fn clone(&self) -> Self {
        Self {
            expression: super::Expression::clone_box(&self.expression),
        }
    }
}

impl PartialEq for WhereClause {
    fn eq(&self, other: &Self) -> bool {
        // 简化比较，只检查类型
        true
    }
}

/// WITH 子句
#[derive(Debug)]
pub struct WithClause {
    pub items: Vec<WithItem>,
}

impl Clone for WithClause {
    fn clone(&self) -> Self {
        Self {
            items: self.items.iter().map(|item| item.clone_box()).collect(),
        }
    }
}

impl PartialEq for WithClause {
    fn eq(&self, other: &Self) -> bool {
        self.items.len() == other.items.len()
    }
}

/// WITH 项
#[derive(Debug)]
pub struct WithItem {
    pub expression: Expr,
    pub alias: Option<String>,
}

impl Clone for WithItem {
    fn clone(&self) -> Self {
        Self {
            expression: super::Expression::clone_box(&self.expression),
            alias: self.alias.clone(),
        }
    }
}

impl PartialEq for WithItem {
    fn eq(&self, other: &Self) -> bool {
        self.alias == other.alias
    }
}

impl WithItem {
    pub fn clone_box(&self) -> WithItem {
        self.clone()
    }
}

/// MATCH 子句数据
#[derive(Debug, Clone, PartialEq)]
pub struct MatchClauseData {
    pub patterns: Vec<Pattern>,
    pub where_clause: Option<WhereClause>,
    pub with_clause: Option<WithClause>,
}

/// MATCH 子句枚举
#[derive(Debug, Clone, PartialEq)]
pub enum MatchClauseType {
    Match(MatchClauseData),
    Return(crate::query::parser::ast::compat::ReturnClause),
    With(WithClause),
    Where(WhereClause),
}

impl MatchClause {
    pub fn clone_box(&self) -> MatchClause {
        match self {
            MatchClause::Match(detail) => MatchClause::Match(detail.clone()),
            MatchClause::Return(ret) => MatchClause::Return(ret.clone()),
            MatchClause::With(with) => MatchClause::With(with.clone()),
            MatchClause::Where(where_clause) => MatchClause::Where(where_clause.clone()),
        }
    }
}