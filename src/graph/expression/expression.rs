//! 表达式类型定义 V2 - 优化版本
//!
//! 使用枚举变体减少装箱，优化内存使用和性能

use serde::{Deserialize, Serialize};

/// 优化后的表达式类型
///
/// 使用枚举变体减少装箱，提高内存效率和性能
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    // 字面量
    Literal(LiteralValue),

    // 变量和属性
    Variable(String),
    Property {
        object: Box<Expression>,
        property: String,
    },

    // 二元操作
    Binary {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },

    // 一元操作
    Unary {
        op: UnaryOperator,
        operand: Box<Expression>,
    },

    // 函数调用
    Function {
        name: String,
        args: Vec<Expression>,
    },

    // 聚合函数
    Aggregate {
        func: AggregateFunction,
        arg: Box<Expression>,
        distinct: bool,
    },

    // 容器类型
    List(Vec<Expression>),
    Map(Vec<(String, Expression)>),

    // 条件表达式
    Case {
        conditions: Vec<(Expression, Expression)>,
        default: Option<Box<Expression>>,
    },

    // 类型转换
    TypeCast {
        expr: Box<Expression>,
        target_type: DataType,
    },

    // 下标访问
    Subscript {
        collection: Box<Expression>,
        index: Box<Expression>,
    },

    // 范围访问
    Range {
        collection: Box<Expression>,
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
    },

    // 路径构建
    Path(Vec<Expression>),

    // 标签
    Label(String),

    // 图数据库特有表达式类型
    TagProperty {
        tag: String,
        prop: String,
    }, // tagName.propName
    EdgeProperty {
        edge: String,
        prop: String,
    }, // edgeName.propName
    InputProperty(String), // $-.propName
    VariableProperty {
        var: String,
        prop: String,
    }, // $varName.propName
    SourceProperty {
        tag: String,
        prop: String,
    }, // $^.tagName.propName
    DestinationProperty {
        tag: String,
        prop: String,
    }, // $$.tagName.propName

    // 一元操作扩展
    UnaryPlus(Box<Expression>),
    UnaryNegate(Box<Expression>),
    UnaryNot(Box<Expression>),
    UnaryIncr(Box<Expression>),
    UnaryDecr(Box<Expression>),
    IsNull(Box<Expression>),
    IsNotNull(Box<Expression>),
    IsEmpty(Box<Expression>),
    IsNotEmpty(Box<Expression>),

    // 类型转换
    TypeCasting {
        expr: Box<Expression>,
        target_type: String,
    },

    // 聚合表达式
    Aggregate {
        func: String,
        arg: Box<Expression>,
        distinct: bool,
    },

    // 列表推导
    ListComprehension {
        generator: Box<Expression>,
        condition: Option<Box<Expression>>,
    },

    // 谓词表达式
    Predicate {
        list: Box<Expression>,
        condition: Box<Expression>,
    },

    // 归约表达式
    Reduce {
        list: Box<Expression>,
        var: String,
        initial: Box<Expression>,
        expr: Box<Expression>,
    },

    // 路径构建表达式
    PathBuild(Vec<Expression>),

    // 文本搜索表达式
    ESQuery(String),

    // UUID表达式
    UUID,

    // 变量表达式
    Variable(String),

    // 下标范围表达式
    SubscriptRange {
        collection: Box<Expression>,
        start: Option<Box<Expression>>,
        end: Option<Box<Expression>>,
    },

    // 匹配路径模式表达式
    MatchPathPattern {
        path_alias: String,
        patterns: Vec<Expression>,
    },
}

/// 字面量值
///
/// 使用独立的枚举来表示字面量，避免嵌套的 Value 类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Null,
}

/// 二元操作符
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // 算术操作
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    // 比较操作
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    // 逻辑操作
    And,
    Or,

    // 字符串操作
    StringConcat,
    Like,
    In,

    // 集合操作
    Union,
    Intersect,
    Except,
}

/// 一元操作符
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    // 算术操作
    Plus,
    Minus,

    // 逻辑操作
    Not,

    // 存在性检查
    IsNull,
    IsNotNull,
    IsEmpty,
    IsNotEmpty,

    // 增减操作
    Increment,
    Decrement,
}

/// 聚合函数
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AggregateFunction {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    Collect,
    Distinct,
}

/// 数据类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Bool,
    Int,
    Float,
    String,
    List,
    Map,
    Vertex,
    Edge,
    Path,
    DateTime,
    // 可以根据需要添加更多类型
}

impl Expression {
    /// 创建字面量表达式
    pub fn literal(value: impl Into<LiteralValue>) -> Self {
        Expression::Literal(value.into())
    }

    /// 创建变量表达式
    pub fn variable(name: impl Into<String>) -> Self {
        Expression::Variable(name.into())
    }

    /// 创建属性访问表达式
    pub fn property(object: Expression, property: impl Into<String>) -> Self {
        Expression::Property {
            object: Box::new(object),
            property: property.into(),
        }
    }

    /// 创建二元操作表达式
    pub fn binary(left: Expression, op: BinaryOperator, right: Expression) -> Self {
        Expression::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }

    /// 创建一元操作表达式
    pub fn unary(op: UnaryOperator, operand: Expression) -> Self {
        Expression::Unary {
            op,
            operand: Box::new(operand),
        }
    }

    /// 创建函数调用表达式
    pub fn function(name: impl Into<String>, args: Vec<Expression>) -> Self {
        Expression::Function {
            name: name.into(),
            args,
        }
    }

    /// 创建聚合函数表达式
    pub fn aggregate(func: AggregateFunction, arg: Expression, distinct: bool) -> Self {
        Expression::Aggregate {
            func,
            arg: Box::new(arg),
            distinct,
        }
    }

    /// 创建列表表达式
    pub fn list(items: Vec<Expression>) -> Self {
        Expression::List(items)
    }

    /// 创建映射表达式
    pub fn map(pairs: Vec<(impl Into<String>, Expression)>) -> Self {
        Expression::Map(pairs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }

    /// 创建条件表达式
    pub fn case(conditions: Vec<(Expression, Expression)>, default: Option<Expression>) -> Self {
        Expression::Case {
            conditions,
            default: default.map(Box::new),
        }
    }

    /// 创建类型转换表达式
    pub fn cast(expr: Expression, target_type: DataType) -> Self {
        Expression::TypeCast {
            expr: Box::new(expr),
            target_type,
        }
    }

    /// 创建下标访问表达式
    pub fn subscript(collection: Expression, index: Expression) -> Self {
        Expression::Subscript {
            collection: Box::new(collection),
            index: Box::new(index),
        }
    }

    /// 创建范围访问表达式
    pub fn range(
        collection: Expression,
        start: Option<Expression>,
        end: Option<Expression>,
    ) -> Self {
        Expression::Range {
            collection: Box::new(collection),
            start: start.map(Box::new),
            end: end.map(Box::new),
        }
    }

    /// 创建路径表达式
    pub fn path(items: Vec<Expression>) -> Self {
        Expression::Path(items)
    }

    /// 创建标签表达式
    pub fn label(name: impl Into<String>) -> Self {
        Expression::Label(name.into())
    }

    /// 获取表达式的子表达式
    pub fn children(&self) -> Vec<&Expression> {
        match self {
            Expression::Literal(_) => vec![],
            Expression::Variable(_) => vec![],
            Expression::Property { object, .. } => vec![object.as_ref()],
            Expression::Binary { left, right, .. } => vec![left.as_ref(), right.as_ref()],
            Expression::Unary { operand, .. } => vec![operand.as_ref()],
            Expression::Function { args, .. } => args.iter().collect(),
            Expression::Aggregate { arg, .. } => vec![arg.as_ref()],
            Expression::List(items) => items.iter().collect(),
            Expression::Map(pairs) => pairs.iter().map(|(_, expr)| expr).collect(),
            Expression::Case {
                conditions,
                default,
            } => {
                let mut children = Vec::new();
                for (cond, value) in conditions {
                    children.push(cond);
                    children.push(value);
                }
                if let Some(def) = default {
                    children.push(def.as_ref());
                }
                children
            }
            Expression::TypeCast { expr, .. } => vec![expr.as_ref()],
            Expression::Subscript { collection, index } => {
                vec![collection.as_ref(), index.as_ref()]
            }
            Expression::Range {
                collection,
                start,
                end,
            } => {
                let mut children = vec![collection.as_ref()];
                if let Some(s) = start {
                    children.push(s.as_ref());
                }
                if let Some(e) = end {
                    children.push(e.as_ref());
                }
                children
            }
            Expression::Path(items) => items.iter().collect(),
            Expression::Label(_) => vec![],
        }
    }

    /// 获取表达式的类型
    pub fn expression_type(&self) -> ExpressionType {
        match self {
            Expression::Literal(_) => ExpressionType::Literal,
            Expression::Variable(_) => ExpressionType::Variable,
            Expression::Property { .. } => ExpressionType::Property,
            Expression::Binary { .. } => ExpressionType::Binary,
            Expression::Unary { .. } => ExpressionType::Unary,
            Expression::Function { .. } => ExpressionType::Function,
            Expression::Aggregate { .. } => ExpressionType::Aggregate,
            Expression::List(_) => ExpressionType::List,
            Expression::Map(_) => ExpressionType::Map,
            Expression::Case { .. } => ExpressionType::Case,
            Expression::TypeCast { .. } => ExpressionType::TypeCast,
            Expression::Subscript { .. } => ExpressionType::Subscript,
            Expression::Range { .. } => ExpressionType::Range,
            Expression::Path(_) => ExpressionType::Path,
            Expression::Label(_) => ExpressionType::Label,
        }
    }

    /// 检查表达式是否为常量
    pub fn is_constant(&self) -> bool {
        match self {
            Expression::Literal(_) => true,
            Expression::List(items) => items.iter().all(|e| e.is_constant()),
            Expression::Map(pairs) => pairs.iter().all(|(_, e)| e.is_constant()),
            _ => false,
        }
    }

    /// 检查表达式是否包含聚合函数
    pub fn contains_aggregate(&self) -> bool {
        match self {
            Expression::Aggregate { .. } => true,
            _ => self.children().iter().any(|e| e.contains_aggregate()),
        }
    }

    /// 获取表达式中使用的所有变量
    pub fn get_variables(&self) -> Vec<String> {
        let mut variables = Vec::new();
        self.collect_variables(&mut variables);
        variables.sort();
        variables.dedup();
        variables
    }

    fn collect_variables(&self, variables: &mut Vec<String>) {
        match self {
            Expression::Variable(name) => {
                if !variables.contains(name) {
                    variables.push(name.clone());
                }
            }
            _ => {
                for child in self.children() {
                    child.collect_variables(variables);
                }
            }
        }
    }
}

/// 表达式类型分类
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpressionType {
    Literal,
    Variable,
    Property,
    Binary,
    Unary,
    Function,
    Aggregate,
    List,
    Map,
    Case,
    TypeCast,
    Subscript,
    Range,
    Path,
    Label,
}

// 从各种类型到 LiteralValue 的转换
impl From<bool> for LiteralValue {
    fn from(value: bool) -> Self {
        LiteralValue::Bool(value)
    }
}

impl From<i64> for LiteralValue {
    fn from(value: i64) -> Self {
        LiteralValue::Int(value)
    }
}

impl From<f64> for LiteralValue {
    fn from(value: f64) -> Self {
        LiteralValue::Float(value)
    }
}

impl From<String> for LiteralValue {
    fn from(value: String) -> Self {
        LiteralValue::String(value)
    }
}

impl From<&str> for LiteralValue {
    fn from(value: &str) -> Self {
        LiteralValue::String(value.to_string())
    }
}

impl From<LiteralValue> for Expression {
    fn from(value: LiteralValue) -> Self {
        Expression::Literal(value)
    }
}

// 便捷的构建器方法
impl Expression {
    /// 创建布尔字面量
    pub fn bool(value: bool) -> Self {
        Expression::Literal(LiteralValue::Bool(value))
    }

    /// 创建整数字面量
    pub fn int(value: i64) -> Self {
        Expression::Literal(LiteralValue::Int(value))
    }

    /// 创建浮点数字面量
    pub fn float(value: f64) -> Self {
        Expression::Literal(LiteralValue::Float(value))
    }

    /// 创建字符串字面量
    pub fn string(value: impl Into<String>) -> Self {
        Expression::Literal(LiteralValue::String(value.into()))
    }

    /// 创建空值
    pub fn null() -> Self {
        Expression::Literal(LiteralValue::Null)
    }

    /// 创建等于比较
    pub fn eq(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Equal, right)
    }

    /// 创建不等于比较
    pub fn ne(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::NotEqual, right)
    }

    /// 创建小于比较
    pub fn lt(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::LessThan, right)
    }

    /// 创建小于等于比较
    pub fn le(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::LessThanOrEqual, right)
    }

    /// 创建大于比较
    pub fn gt(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::GreaterThan, right)
    }

    /// 创建大于等于比较
    pub fn ge(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::GreaterThanOrEqual, right)
    }

    /// 创建加法
    pub fn add(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Add, right)
    }

    /// 创建减法
    pub fn sub(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Subtract, right)
    }

    /// 创建乘法
    pub fn mul(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Multiply, right)
    }

    /// 创建除法
    pub fn div(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Divide, right)
    }

    /// 创建逻辑与
    pub fn and(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::And, right)
    }

    /// 创建逻辑或
    pub fn or(left: Expression, right: Expression) -> Self {
        Self::binary(left, BinaryOperator::Or, right)
    }

    /// 创建逻辑非
    pub fn not(expr: Expression) -> Self {
        Self::unary(UnaryOperator::Not, expr)
    }

    /// 创建空值检查
    pub fn is_null(expr: Expression) -> Self {
        Self::unary(UnaryOperator::IsNull, expr)
    }

    /// 创建非空值检查
    pub fn is_not_null(expr: Expression) -> Self {
        Self::unary(UnaryOperator::IsNotNull, expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_creation() {
        // 测试字面量创建
        let expr = Expression::int(42);
        assert_eq!(expr, Expression::Literal(LiteralValue::Int(42)));

        // 测试变量创建
        let expr = Expression::variable("x");
        assert_eq!(expr, Expression::Variable("x".to_string()));

        // 测试属性访问创建
        let expr = Expression::property(Expression::variable("a"), "name");
        assert_eq!(
            expr,
            Expression::Property {
                object: Box::new(Expression::Variable("a".to_string())),
                property: "name".to_string(),
            }
        );
    }

    #[test]
    fn test_binary_operations() {
        let left = Expression::int(10);
        let right = Expression::int(20);

        // 测试加法
        let expr = Expression::add(left.clone(), right.clone());
        assert_eq!(
            expr,
            Expression::Binary {
                left: Box::new(left.clone()),
                op: BinaryOperator::Add,
                right: Box::new(right.clone()),
            }
        );

        // 测试比较
        let expr = Expression::lt(left, right);
        assert_eq!(
            expr,
            Expression::Binary {
                left: Box::new(Expression::int(10)),
                op: BinaryOperator::LessThan,
                right: Box::new(Expression::int(20)),
            }
        );
    }

    #[test]
    fn test_unary_operations() {
        let expr = Expression::not(Expression::bool(true));
        assert_eq!(
            expr,
            Expression::Unary {
                op: UnaryOperator::Not,
                operand: Box::new(Expression::bool(true)),
            }
        );
    }

    #[test]
    fn test_function_calls() {
        let expr = Expression::function("count", vec![Expression::variable("x")]);
        assert_eq!(
            expr,
            Expression::Function {
                name: "count".to_string(),
                args: vec![Expression::Variable("x".to_string())],
            }
        );
    }

    #[test]
    fn test_aggregate_functions() {
        let expr =
            Expression::aggregate(AggregateFunction::Count, Expression::variable("x"), false);
        assert_eq!(
            expr,
            Expression::Aggregate {
                func: AggregateFunction::Count,
                arg: Box::new(Expression::Variable("x".to_string())),
                distinct: false,
            }
        );
    }

    #[test]
    fn test_containers() {
        // 测试列表
        let list = Expression::list(vec![
            Expression::int(1),
            Expression::int(2),
            Expression::int(3),
        ]);
        assert_eq!(
            list,
            Expression::List(vec![
                Expression::Literal(LiteralValue::Int(1)),
                Expression::Literal(LiteralValue::Int(2)),
                Expression::Literal(LiteralValue::Int(3)),
            ])
        );

        // 测试映射
        let map = Expression::map(vec![
            ("a", Expression::int(1)),
            ("b", Expression::string("hello")),
        ]);
        assert_eq!(
            map,
            Expression::Map(vec![
                ("a".to_string(), Expression::Literal(LiteralValue::Int(1))),
                (
                    "b".to_string(),
                    Expression::Literal(LiteralValue::String("hello".to_string()))
                ),
            ])
        );
    }

    #[test]
    fn test_expression_properties() {
        // 测试常量检查
        assert!(Expression::int(42).is_constant());
        assert!(Expression::bool(true).is_constant());
        assert!(!Expression::variable("x").is_constant());

        // 测试聚合函数检查
        let agg_expr =
            Expression::aggregate(AggregateFunction::Count, Expression::variable("x"), false);
        assert!(agg_expr.contains_aggregate());

        let simple_expr = Expression::add(Expression::int(1), Expression::int(2));
        assert!(!simple_expr.contains_aggregate());

        // 测试变量提取
        let complex_expr = Expression::add(
            Expression::variable("x"),
            Expression::mul(Expression::variable("y"), Expression::int(2)),
        );
        let vars = complex_expr.get_variables();
        assert_eq!(vars, vec!["x", "y"]);
    }

    #[test]
    fn test_type_conversions() {
        // 测试从基本类型到 LiteralValue 的转换
        let lit: LiteralValue = 42i64.into();
        assert_eq!(lit, LiteralValue::Int(42));

        let lit: LiteralValue = 3.14f64.into();
        assert_eq!(lit, LiteralValue::Float(3.14));

        let lit: LiteralValue = true.into();
        assert_eq!(lit, LiteralValue::Bool(true));

        let lit: LiteralValue = "hello".into();
        assert_eq!(lit, LiteralValue::String("hello".to_string()));

        // 测试从 LiteralValue 到 Expression 的转换
        let expr: Expression = LiteralValue::Int(42).into();
        assert_eq!(expr, Expression::Literal(LiteralValue::Int(42)));
    }
}
