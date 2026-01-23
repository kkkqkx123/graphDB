//! 表达式 AST 定义 (v2)
//!
//! 基于枚举的简化表达式定义，消除动态分发开销。

use super::types::*;
use crate::core::Value;

/// 表达式枚举 - 核心 AST 节点
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Constant(ConstantExpression),
    Variable(VariableExpression),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    FunctionCall(FunctionCallExpression),
    PropertyAccess(PropertyAccessExpression),
    List(ListExpression),
    Map(MapExpression),
    Case(CaseExpression),
    Subscript(SubscriptExpression),
    TypeCast(TypeCastExpression),
    Range(RangeExpression),
    Path(PathExpression),
    Label(LabelExpression),
}

impl Expression {
    /// 获取表达式的位置信息
    pub fn span(&self) -> Span {
        match self {
            Expression::Constant(e) => e.span,
            Expression::Variable(e) => e.span,
            Expression::Binary(e) => e.span,
            Expression::Unary(e) => e.span,
            Expression::FunctionCall(e) => e.span,
            Expression::PropertyAccess(e) => e.span,
            Expression::List(e) => e.span,
            Expression::Map(e) => e.span,
            Expression::Case(e) => e.span,
            Expression::Subscript(e) => e.span,
            Expression::TypeCast(e) => e.span,
            Expression::Range(e) => e.span,
            Expression::Path(e) => e.span,
            Expression::Label(e) => e.span,
        }
    }

    /// 检查表达式是否为常量
    pub fn is_constant(&self) -> bool {
        match self {
            Expression::Constant(_) => true,
            Expression::Binary(e) => e.left.is_constant() && e.right.is_constant(),
            Expression::Unary(e) => e.operand.is_constant(),
            Expression::List(e) => e.elements.iter().all(|elem| elem.is_constant()),
            Expression::Map(e) => e.pairs.iter().all(|(_, value)| value.is_constant()),
            Expression::Case(e) => {
                let match_constant = e
                    .match_expression
                    .as_ref()
                    .map_or(true, |expression| expression.is_constant());
                let when_constant = e
                    .when_then_pairs
                    .iter()
                    .all(|(when, then)| when.is_constant() && then.is_constant());
                let default_constant = e.default.as_ref().map_or(true, |expression| expression.is_constant());
                match_constant && when_constant && default_constant
            }
            Expression::Subscript(e) => e.collection.is_constant() && e.index.is_constant(),
            Expression::Range(e) => {
                let collection_constant = e.collection.is_constant();
                let start_constant = e.start.as_ref().map_or(true, |expression| expression.is_constant());
                let end_constant = e.end.as_ref().map_or(true, |expression| expression.is_constant());
                collection_constant && start_constant && end_constant
            }
            Expression::Path(e) => e.elements.iter().all(|elem| elem.is_constant()),
            _ => false,
        }
    }

    /// 获取表达式的字符串表示
    pub fn to_string(&self) -> String {
        match self {
            Expression::Constant(e) => format!("{:?}", e.value),
            Expression::Variable(e) => e.name.clone(),
            Expression::Binary(e) => format!(
                "({} {} {})",
                e.left.to_string(),
                e.op.to_string(),
                e.right.to_string()
            ),
            Expression::Unary(e) => format!("{} {}", e.op.to_string(), e.operand.to_string()),
            Expression::FunctionCall(e) => {
                let args_str = e
                    .args
                    .iter()
                    .map(|arg| arg.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                if e.distinct {
                    format!("{}(DISTINCT {})", e.name, args_str)
                } else {
                    format!("{}({})", e.name, args_str)
                }
            }
            Expression::PropertyAccess(e) => format!("{}.{}", e.object.to_string(), e.property),
            Expression::List(e) => {
                let elements_str = e
                    .elements
                    .iter()
                    .map(|elem| elem.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", elements_str)
            }
            Expression::Map(e) => {
                let pairs_str = e
                    .pairs
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key, value.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", pairs_str)
            }
            Expression::Case(e) => {
                let mut result = String::from("CASE");

                if let Some(ref expression) = e.match_expression {
                    result.push_str(&format!(" {}", expression.to_string()));
                }

                for (when, then) in &e.when_then_pairs {
                    result.push_str(&format!(
                        " WHEN {} THEN {}",
                        when.to_string(),
                        then.to_string()
                    ));
                }

                if let Some(ref default) = e.default {
                    result.push_str(&format!(" ELSE {}", default.to_string()));
                }

                result.push_str(" END");
                result
            }
            Expression::Subscript(e) => format!("{}[{}]", e.collection.to_string(), e.index.to_string()),
            Expression::TypeCast(e) => format!("CAST({} AS {})", e.expression.to_string(), e.target_type),
            Expression::Range(e) => {
                let start_str = e
                    .start
                    .as_ref()
                    .map_or(String::new(), |expression| expression.to_string());
                let end_str = e
                    .end
                    .as_ref()
                    .map_or(String::new(), |expression| expression.to_string());
                format!("{}[{}..{}]", e.collection.to_string(), start_str, end_str)
            }
            Expression::Path(e) => {
                let elements_str = e
                    .elements
                    .iter()
                    .map(|elem| elem.to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                format!("[{}]", elements_str)
            }
            Expression::Label(e) => format!(":{}", e.label),
        }
    }
}

/// 常量表达式
#[derive(Debug, Clone, PartialEq)]
pub struct ConstantExpression {
    pub span: Span,
    pub value: Value,
}

impl ConstantExpression {
    pub fn new(value: Value, span: Span) -> Self {
        Self { span, value }
    }
}

/// 变量表达式
#[derive(Debug, Clone, PartialEq)]
pub struct VariableExpression {
    pub span: Span,
    pub name: String,
}

impl VariableExpression {
    pub fn new(name: String, span: Span) -> Self {
        Self { span, name }
    }
}

/// 二元表达式
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpression {
    pub span: Span,
    pub left: Box<Expression>,
    pub op: BinaryOp,
    pub right: Box<Expression>,
}

impl BinaryExpression {
    pub fn new(left: Expression, op: BinaryOp, right: Expression, span: Span) -> Self {
        Self {
            span,
            left: Box::new(left),
            op,
            right: Box::new(right),
        }
    }
}

/// 一元表达式
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpression {
    pub span: Span,
    pub op: UnaryOp,
    pub operand: Box<Expression>,
}

impl UnaryExpression {
    pub fn new(op: UnaryOp, operand: Expression, span: Span) -> Self {
        Self {
            span,
            op,
            operand: Box::new(operand),
        }
    }
}

/// 函数调用表达式
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallExpression {
    pub span: Span,
    pub name: String,
    pub args: Vec<Expression>,
    pub distinct: bool,
}

impl FunctionCallExpression {
    pub fn new(name: String, args: Vec<Expression>, distinct: bool, span: Span) -> Self {
        Self {
            span,
            name,
            args,
            distinct,
        }
    }
}

/// 属性访问表达式
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyAccessExpression {
    pub span: Span,
    pub object: Box<Expression>,
    pub property: String,
}

impl PropertyAccessExpression {
    pub fn new(object: Expression, property: String, span: Span) -> Self {
        Self {
            span,
            object: Box::new(object),
            property,
        }
    }
}

/// 列表表达式
#[derive(Debug, Clone, PartialEq)]
pub struct ListExpression {
    pub span: Span,
    pub elements: Vec<Expression>,
}

impl ListExpression {
    pub fn new(elements: Vec<Expression>, span: Span) -> Self {
        Self { span, elements }
    }
}

/// 映射表达式
#[derive(Debug, Clone, PartialEq)]
pub struct MapExpression {
    pub span: Span,
    pub pairs: Vec<(String, Expression)>,
}

impl MapExpression {
    pub fn new(pairs: Vec<(String, Expression)>, span: Span) -> Self {
        Self { span, pairs }
    }
}

/// CASE 表达式
#[derive(Debug, Clone, PartialEq)]
pub struct CaseExpression {
    pub span: Span,
    pub match_expression: Option<Box<Expression>>,
    pub when_then_pairs: Vec<(Box<Expression>, Box<Expression>)>,
    pub default: Option<Box<Expression>>,
}

impl CaseExpression {
    pub fn new(
        match_expression: Option<Expression>,
        when_then_pairs: Vec<(Expression, Expression)>,
        default: Option<Expression>,
        span: Span,
    ) -> Self {
        Self {
            span,
            match_expression: match_expression.map(Box::new),
            when_then_pairs: when_then_pairs
                .into_iter()
                .map(|(when, then)| (Box::new(when), Box::new(then)))
                .collect(),
            default: default.map(Box::new),
        }
    }
}

/// 下标表达式
#[derive(Debug, Clone, PartialEq)]
pub struct SubscriptExpression {
    pub span: Span,
    pub collection: Box<Expression>,
    pub index: Box<Expression>,
}

impl SubscriptExpression {
    pub fn new(collection: Expression, index: Expression, span: Span) -> Self {
        Self {
            span,
            collection: Box::new(collection),
            index: Box::new(index),
        }
    }
}

/// 类型转换表达式
#[derive(Debug, Clone, PartialEq)]
pub struct TypeCastExpression {
    pub span: Span,
    pub expression: Box<Expression>,
    pub target_type: String,
}

impl TypeCastExpression {
    pub fn new(expression: Expression, target_type: String, span: Span) -> Self {
        Self {
            span,
            expression: Box::new(expression),
            target_type,
        }
    }
}

/// 范围表达式
#[derive(Debug, Clone, PartialEq)]
pub struct RangeExpression {
    pub span: Span,
    pub collection: Box<Expression>,
    pub start: Option<Box<Expression>>,
    pub end: Option<Box<Expression>>,
}

impl RangeExpression {
    pub fn new(collection: Expression, start: Option<Expression>, end: Option<Expression>, span: Span) -> Self {
        Self {
            span,
            collection: Box::new(collection),
            start: start.map(Box::new),
            end: end.map(Box::new),
        }
    }
}

/// 路径表达式
#[derive(Debug, Clone, PartialEq)]
pub struct PathExpression {
    pub span: Span,
    pub elements: Vec<Expression>,
}

impl PathExpression {
    pub fn new(elements: Vec<Expression>, span: Span) -> Self {
        Self { span, elements }
    }
}

/// 标签表达式
#[derive(Debug, Clone, PartialEq)]
pub struct LabelExpression {
    pub span: Span,
    pub label: String,
}

impl LabelExpression {
    pub fn new(label: String, span: Span) -> Self {
        Self { span, label }
    }
}

// 实现 Display trait 用于格式化输出
impl std::fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Subtract => write!(f, "-"),
            BinaryOp::Multiply => write!(f, "*"),
            BinaryOp::Divide => write!(f, "/"),
            BinaryOp::Modulo => write!(f, "%"),
            BinaryOp::Exponent => write!(f, "**"),
            BinaryOp::And => write!(f, "AND"),
            BinaryOp::Or => write!(f, "OR"),
            BinaryOp::Xor => write!(f, "XOR"),
            BinaryOp::Equal => write!(f, "=="),
            BinaryOp::NotEqual => write!(f, "!="),
            BinaryOp::LessThan => write!(f, "<"),
            BinaryOp::LessThanOrEqual => write!(f, "<="),
            BinaryOp::GreaterThan => write!(f, ">"),
            BinaryOp::GreaterThanOrEqual => write!(f, ">="),
            BinaryOp::StringConcat => write!(f, "||"),
            BinaryOp::Subscript => write!(f, "[]"),
            BinaryOp::Attribute => write!(f, "."),
            BinaryOp::Like => write!(f, "=~"),
            BinaryOp::In => write!(f, "IN"),
            BinaryOp::NotIn => write!(f, "NOT IN"),
            BinaryOp::Contains => write!(f, "CONTAINS"),
            BinaryOp::StartsWith => write!(f, "STARTS WITH"),
            BinaryOp::EndsWith => write!(f, "ENDS WITH"),
            BinaryOp::Union => write!(f, "UNION"),
            BinaryOp::Intersect => write!(f, "INTERSECT"),
            BinaryOp::Except => write!(f, "EXCEPT"),
        }
    }
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "NOT"),
            UnaryOp::Plus => write!(f, "+"),
            UnaryOp::Minus => write!(f, "-"),
            UnaryOp::IsNull => write!(f, "IS NULL"),
            UnaryOp::IsNotNull => write!(f, "IS NOT NULL"),
            UnaryOp::IsEmpty => write!(f, "IS EMPTY"),
            UnaryOp::IsNotEmpty => write!(f, "IS NOT EMPTY"),
        }
    }
}

// 表达式工具函数
pub struct ExprUtils;

impl ExprUtils {
    /// 查找表达式中的变量
    pub fn find_variables(expression: &Expression) -> Vec<String> {
        let mut variables = Vec::new();
        Self::find_variables_recursive(expression, &mut variables);
        variables
    }

    fn find_variables_recursive(expression: &Expression, variables: &mut Vec<String>) {
        match expression {
            Expression::Variable(e) => variables.push(e.name.clone()),
            Expression::Binary(e) => {
                Self::find_variables_recursive(&e.left, variables);
                Self::find_variables_recursive(&e.right, variables);
            }
            Expression::Unary(e) => Self::find_variables_recursive(&e.operand, variables),
            Expression::FunctionCall(e) => {
                for arg in &e.args {
                    Self::find_variables_recursive(arg, variables);
                }
            }
            Expression::PropertyAccess(e) => Self::find_variables_recursive(&e.object, variables),
            Expression::List(e) => {
                for elem in &e.elements {
                    Self::find_variables_recursive(elem, variables);
                }
            }
            Expression::Map(e) => {
                for (_, value) in &e.pairs {
                    Self::find_variables_recursive(value, variables);
                }
            }
            Expression::Case(e) => {
                if let Some(ref expression) = e.match_expression {
                    Self::find_variables_recursive(expression, variables);
                }
                for (when, then) in &e.when_then_pairs {
                    Self::find_variables_recursive(when, variables);
                    Self::find_variables_recursive(then, variables);
                }
                if let Some(ref default) = e.default {
                    Self::find_variables_recursive(default, variables);
                }
            }
            Expression::Subscript(e) => {
                Self::find_variables_recursive(&e.collection, variables);
                Self::find_variables_recursive(&e.index, variables);
            }
            Expression::TypeCast(e) => Self::find_variables_recursive(&e.expression, variables),
            Expression::Range(e) => {
                Self::find_variables_recursive(&e.collection, variables);
                if let Some(ref start) = e.start {
                    Self::find_variables_recursive(start, variables);
                }
                if let Some(ref end) = e.end {
                    Self::find_variables_recursive(end, variables);
                }
            }
            Expression::Path(e) => {
                for elem in &e.elements {
                    Self::find_variables_recursive(elem, variables);
                }
            }
            _ => {}
        }
    }

    /// 检查表达式是否包含聚合函数
    pub fn contains_aggregate(expression: &Expression) -> bool {
        Self::contains_aggregate_recursive(expression)
    }

    fn contains_aggregate_recursive(expression: &Expression) -> bool {
        match expression {
            Expression::FunctionCall(e) => {
                let func_name = e.name.to_uppercase();
                matches!(
                    func_name.as_str(),
                    "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "COLLECT" | "AGGREGATE"
                )
            }
            Expression::Binary(e) => {
                Self::contains_aggregate_recursive(&e.left)
                    || Self::contains_aggregate_recursive(&e.right)
            }
            Expression::Unary(e) => Self::contains_aggregate_recursive(&e.operand),
            Expression::List(e) => e.elements.iter().any(Self::contains_aggregate_recursive),
            Expression::Map(e) => e
                .pairs
                .iter()
                .any(|(_, value)| Self::contains_aggregate_recursive(value)),
            Expression::Case(e) => {
                let match_contains = e
                    .match_expression
                    .as_ref()
                    .map_or(false, |expression| Self::contains_aggregate_recursive(expression));
                let when_contains = e.when_then_pairs.iter().any(|(when, then)| {
                    Self::contains_aggregate_recursive(when)
                        || Self::contains_aggregate_recursive(then)
                });
                let default_contains = e
                    .default
                    .as_ref()
                    .map_or(false, |expression| Self::contains_aggregate_recursive(expression));
                match_contains || when_contains || default_contains
            }
            Expression::Subscript(e) => {
                Self::contains_aggregate_recursive(&e.collection)
                    || Self::contains_aggregate_recursive(&e.index)
            }
            Expression::TypeCast(e) => Self::contains_aggregate_recursive(&e.expression),
            Expression::Range(e) => {
                let collection_contains = Self::contains_aggregate_recursive(&e.collection);
                let start_contains = e
                    .start
                    .as_ref()
                    .map_or(false, |expression| Self::contains_aggregate_recursive(expression));
                let end_contains = e
                    .end
                    .as_ref()
                    .map_or(false, |expression| Self::contains_aggregate_recursive(expression));
                collection_contains || start_contains || end_contains
            }
            Expression::Path(e) => e.elements.iter().any(Self::contains_aggregate_recursive),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_expression() {
        let expression = Expression::Constant(ConstantExpression::new(Value::Int(42), Span::default()));
        assert!(expression.is_constant());
        assert_eq!(expression.to_string(), "Int(42)");
    }

    #[test]
    fn test_variable_expression() {
        let expression = Expression::Variable(VariableExpression::new("x".to_string(), Span::default()));
        assert!(!expression.is_constant());
        assert_eq!(expression.to_string(), "x");
    }

    #[test]
    fn test_binary_expression() {
        let left = Expression::Constant(ConstantExpression::new(Value::Int(5), Span::default()));
        let right = Expression::Constant(ConstantExpression::new(Value::Int(3), Span::default()));
        let expression = Expression::Binary(BinaryExpression::new(left, BinaryOp::Add, right, Span::default()));

        assert!(expression.is_constant());
        assert_eq!(expression.to_string(), "(Int(5) + Int(3))");
    }

    #[test]
    fn test_find_variables() {
        let expression = Expression::Variable(VariableExpression::new("test_var".to_string(), Span::default()));
        let variables = ExprUtils::find_variables(&expression);
        assert_eq!(variables, vec!["test_var"]);
    }

    #[test]
    fn test_contains_aggregate() {
        let func_expression = Expression::FunctionCall(FunctionCallExpression::new(
            "COUNT".to_string(),
            vec![Expression::Variable(VariableExpression::new(
                "x".to_string(),
                Span::default(),
            ))],
            false,
            Span::default(),
        ));
        assert!(ExprUtils::contains_aggregate(&func_expression));
    }
}
