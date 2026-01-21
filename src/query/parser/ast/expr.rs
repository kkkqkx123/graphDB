//! 表达式 AST 定义 (v2)
//!
//! 基于枚举的简化表达式定义，消除动态分发开销。

use super::types::*;
use crate::core::Value;

/// 表达式枚举 - 核心 AST 节点
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Constant(ConstantExpr),
    Variable(VariableExpr),
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    FunctionCall(FunctionCallExpr),
    PropertyAccess(PropertyAccessExpr),
    List(ListExpr),
    Map(MapExpr),
    Case(CaseExpr),
    Subscript(SubscriptExpr),
    TypeCast(TypeCastExpr),
    Range(RangeExpr),
    Path(PathExpr),
    Label(LabelExpr),
}

impl Expr {
    /// 获取表达式的位置信息
    pub fn span(&self) -> Span {
        match self {
            Expr::Constant(e) => e.span,
            Expr::Variable(e) => e.span,
            Expr::Binary(e) => e.span,
            Expr::Unary(e) => e.span,
            Expr::FunctionCall(e) => e.span,
            Expr::PropertyAccess(e) => e.span,
            Expr::List(e) => e.span,
            Expr::Map(e) => e.span,
            Expr::Case(e) => e.span,
            Expr::Subscript(e) => e.span,
            Expr::TypeCast(e) => e.span,
            Expr::Range(e) => e.span,
            Expr::Path(e) => e.span,
            Expr::Label(e) => e.span,
        }
    }

    /// 检查表达式是否为常量
    pub fn is_constant(&self) -> bool {
        match self {
            Expr::Constant(_) => true,
            Expr::Binary(e) => e.left.is_constant() && e.right.is_constant(),
            Expr::Unary(e) => e.operand.is_constant(),
            Expr::List(e) => e.elements.iter().all(|elem| elem.is_constant()),
            Expr::Map(e) => e.pairs.iter().all(|(_, value)| value.is_constant()),
            Expr::Case(e) => {
                let match_constant = e
                    .match_expr
                    .as_ref()
                    .map_or(true, |expr| expr.is_constant());
                let when_constant = e
                    .when_then_pairs
                    .iter()
                    .all(|(when, then)| when.is_constant() && then.is_constant());
                let default_constant = e.default.as_ref().map_or(true, |expr| expr.is_constant());
                match_constant && when_constant && default_constant
            }
            Expr::Subscript(e) => e.collection.is_constant() && e.index.is_constant(),
            Expr::Range(e) => {
                let collection_constant = e.collection.is_constant();
                let start_constant = e.start.as_ref().map_or(true, |expr| expr.is_constant());
                let end_constant = e.end.as_ref().map_or(true, |expr| expr.is_constant());
                collection_constant && start_constant && end_constant
            }
            Expr::Path(e) => e.elements.iter().all(|elem| elem.is_constant()),
            _ => false,
        }
    }

    /// 获取表达式的字符串表示
    pub fn to_string(&self) -> String {
        match self {
            Expr::Constant(e) => format!("{:?}", e.value),
            Expr::Variable(e) => e.name.clone(),
            Expr::Binary(e) => format!(
                "({} {} {})",
                e.left.to_string(),
                e.op.to_string(),
                e.right.to_string()
            ),
            Expr::Unary(e) => format!("{} {}", e.op.to_string(), e.operand.to_string()),
            Expr::FunctionCall(e) => {
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
            Expr::PropertyAccess(e) => format!("{}.{}", e.object.to_string(), e.property),
            Expr::List(e) => {
                let elements_str = e
                    .elements
                    .iter()
                    .map(|elem| elem.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("[{}]", elements_str)
            }
            Expr::Map(e) => {
                let pairs_str = e
                    .pairs
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key, value.to_string()))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{{}}}", pairs_str)
            }
            Expr::Case(e) => {
                let mut result = String::from("CASE");

                if let Some(ref expr) = e.match_expr {
                    result.push_str(&format!(" {}", expr.to_string()));
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
            Expr::Subscript(e) => format!("{}[{}]", e.collection.to_string(), e.index.to_string()),
            Expr::TypeCast(e) => format!("CAST({} AS {})", e.expr.to_string(), e.target_type),
            Expr::Range(e) => {
                let start_str = e
                    .start
                    .as_ref()
                    .map_or(String::new(), |expr| expr.to_string());
                let end_str = e
                    .end
                    .as_ref()
                    .map_or(String::new(), |expr| expr.to_string());
                format!("{}[{}..{}]", e.collection.to_string(), start_str, end_str)
            }
            Expr::Path(e) => {
                let elements_str = e
                    .elements
                    .iter()
                    .map(|elem| elem.to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                format!("[{}]", elements_str)
            }
            Expr::Label(e) => format!(":{}", e.label),
        }
    }
}

/// 常量表达式
#[derive(Debug, Clone, PartialEq)]
pub struct ConstantExpr {
    pub span: Span,
    pub value: Value,
}

impl ConstantExpr {
    pub fn new(value: Value, span: Span) -> Self {
        Self { span, value }
    }
}

/// 变量表达式
#[derive(Debug, Clone, PartialEq)]
pub struct VariableExpr {
    pub span: Span,
    pub name: String,
}

impl VariableExpr {
    pub fn new(name: String, span: Span) -> Self {
        Self { span, name }
    }
}

/// 二元表达式
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub span: Span,
    pub left: Box<Expr>,
    pub op: BinaryOp,
    pub right: Box<Expr>,
}

impl BinaryExpr {
    pub fn new(left: Expr, op: BinaryOp, right: Expr, span: Span) -> Self {
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
pub struct UnaryExpr {
    pub span: Span,
    pub op: UnaryOp,
    pub operand: Box<Expr>,
}

impl UnaryExpr {
    pub fn new(op: UnaryOp, operand: Expr, span: Span) -> Self {
        Self {
            span,
            op,
            operand: Box::new(operand),
        }
    }
}

/// 函数调用表达式
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallExpr {
    pub span: Span,
    pub name: String,
    pub args: Vec<Expr>,
    pub distinct: bool,
}

impl FunctionCallExpr {
    pub fn new(name: String, args: Vec<Expr>, distinct: bool, span: Span) -> Self {
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
pub struct PropertyAccessExpr {
    pub span: Span,
    pub object: Box<Expr>,
    pub property: String,
}

impl PropertyAccessExpr {
    pub fn new(object: Expr, property: String, span: Span) -> Self {
        Self {
            span,
            object: Box::new(object),
            property,
        }
    }
}

/// 列表表达式
#[derive(Debug, Clone, PartialEq)]
pub struct ListExpr {
    pub span: Span,
    pub elements: Vec<Expr>,
}

impl ListExpr {
    pub fn new(elements: Vec<Expr>, span: Span) -> Self {
        Self { span, elements }
    }
}

/// 映射表达式
#[derive(Debug, Clone, PartialEq)]
pub struct MapExpr {
    pub span: Span,
    pub pairs: Vec<(String, Expr)>,
}

impl MapExpr {
    pub fn new(pairs: Vec<(String, Expr)>, span: Span) -> Self {
        Self { span, pairs }
    }
}

/// CASE 表达式
#[derive(Debug, Clone, PartialEq)]
pub struct CaseExpr {
    pub span: Span,
    pub match_expr: Option<Box<Expr>>,
    pub when_then_pairs: Vec<(Box<Expr>, Box<Expr>)>,
    pub default: Option<Box<Expr>>,
}

impl CaseExpr {
    pub fn new(
        match_expr: Option<Expr>,
        when_then_pairs: Vec<(Expr, Expr)>,
        default: Option<Expr>,
        span: Span,
    ) -> Self {
        Self {
            span,
            match_expr: match_expr.map(Box::new),
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
pub struct SubscriptExpr {
    pub span: Span,
    pub collection: Box<Expr>,
    pub index: Box<Expr>,
}

impl SubscriptExpr {
    pub fn new(collection: Expr, index: Expr, span: Span) -> Self {
        Self {
            span,
            collection: Box::new(collection),
            index: Box::new(index),
        }
    }
}

/// 类型转换表达式
#[derive(Debug, Clone, PartialEq)]
pub struct TypeCastExpr {
    pub span: Span,
    pub expr: Box<Expr>,
    pub target_type: String,
}

impl TypeCastExpr {
    pub fn new(expr: Expr, target_type: String, span: Span) -> Self {
        Self {
            span,
            expr: Box::new(expr),
            target_type,
        }
    }
}

/// 范围表达式
#[derive(Debug, Clone, PartialEq)]
pub struct RangeExpr {
    pub span: Span,
    pub collection: Box<Expr>,
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
}

impl RangeExpr {
    pub fn new(collection: Expr, start: Option<Expr>, end: Option<Expr>, span: Span) -> Self {
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
pub struct PathExpr {
    pub span: Span,
    pub elements: Vec<Expr>,
}

impl PathExpr {
    pub fn new(elements: Vec<Expr>, span: Span) -> Self {
        Self { span, elements }
    }
}

/// 标签表达式
#[derive(Debug, Clone, PartialEq)]
pub struct LabelExpr {
    pub span: Span,
    pub label: String,
}

impl LabelExpr {
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
            UnaryOp::Increment => write!(f, "++"),
            UnaryOp::Decrement => write!(f, "--"),
        }
    }
}

// 表达式工具函数
pub struct ExprUtils;

impl ExprUtils {
    /// 查找表达式中的变量
    pub fn find_variables(expr: &Expr) -> Vec<String> {
        let mut variables = Vec::new();
        Self::find_variables_recursive(expr, &mut variables);
        variables
    }

    fn find_variables_recursive(expr: &Expr, variables: &mut Vec<String>) {
        match expr {
            Expr::Variable(e) => variables.push(e.name.clone()),
            Expr::Binary(e) => {
                Self::find_variables_recursive(&e.left, variables);
                Self::find_variables_recursive(&e.right, variables);
            }
            Expr::Unary(e) => Self::find_variables_recursive(&e.operand, variables),
            Expr::FunctionCall(e) => {
                for arg in &e.args {
                    Self::find_variables_recursive(arg, variables);
                }
            }
            Expr::PropertyAccess(e) => Self::find_variables_recursive(&e.object, variables),
            Expr::List(e) => {
                for elem in &e.elements {
                    Self::find_variables_recursive(elem, variables);
                }
            }
            Expr::Map(e) => {
                for (_, value) in &e.pairs {
                    Self::find_variables_recursive(value, variables);
                }
            }
            Expr::Case(e) => {
                if let Some(ref expr) = e.match_expr {
                    Self::find_variables_recursive(expr, variables);
                }
                for (when, then) in &e.when_then_pairs {
                    Self::find_variables_recursive(when, variables);
                    Self::find_variables_recursive(then, variables);
                }
                if let Some(ref default) = e.default {
                    Self::find_variables_recursive(default, variables);
                }
            }
            Expr::Subscript(e) => {
                Self::find_variables_recursive(&e.collection, variables);
                Self::find_variables_recursive(&e.index, variables);
            }
            Expr::TypeCast(e) => Self::find_variables_recursive(&e.expr, variables),
            Expr::Range(e) => {
                Self::find_variables_recursive(&e.collection, variables);
                if let Some(ref start) = e.start {
                    Self::find_variables_recursive(start, variables);
                }
                if let Some(ref end) = e.end {
                    Self::find_variables_recursive(end, variables);
                }
            }
            Expr::Path(e) => {
                for elem in &e.elements {
                    Self::find_variables_recursive(elem, variables);
                }
            }
            _ => {}
        }
    }

    /// 检查表达式是否包含聚合函数
    pub fn contains_aggregate(expr: &Expr) -> bool {
        Self::contains_aggregate_recursive(expr)
    }

    fn contains_aggregate_recursive(expr: &Expr) -> bool {
        match expr {
            Expr::FunctionCall(e) => {
                let func_name = e.name.to_uppercase();
                matches!(
                    func_name.as_str(),
                    "COUNT" | "SUM" | "AVG" | "MIN" | "MAX" | "COLLECT" | "AGGREGATE"
                )
            }
            Expr::Binary(e) => {
                Self::contains_aggregate_recursive(&e.left)
                    || Self::contains_aggregate_recursive(&e.right)
            }
            Expr::Unary(e) => Self::contains_aggregate_recursive(&e.operand),
            Expr::List(e) => e.elements.iter().any(Self::contains_aggregate_recursive),
            Expr::Map(e) => e
                .pairs
                .iter()
                .any(|(_, value)| Self::contains_aggregate_recursive(value)),
            Expr::Case(e) => {
                let match_contains = e
                    .match_expr
                    .as_ref()
                    .map_or(false, |expr| Self::contains_aggregate_recursive(expr));
                let when_contains = e.when_then_pairs.iter().any(|(when, then)| {
                    Self::contains_aggregate_recursive(when)
                        || Self::contains_aggregate_recursive(then)
                });
                let default_contains = e
                    .default
                    .as_ref()
                    .map_or(false, |expr| Self::contains_aggregate_recursive(expr));
                match_contains || when_contains || default_contains
            }
            Expr::Subscript(e) => {
                Self::contains_aggregate_recursive(&e.collection)
                    || Self::contains_aggregate_recursive(&e.index)
            }
            Expr::TypeCast(e) => Self::contains_aggregate_recursive(&e.expr),
            Expr::Range(e) => {
                let collection_contains = Self::contains_aggregate_recursive(&e.collection);
                let start_contains = e
                    .start
                    .as_ref()
                    .map_or(false, |expr| Self::contains_aggregate_recursive(expr));
                let end_contains = e
                    .end
                    .as_ref()
                    .map_or(false, |expr| Self::contains_aggregate_recursive(expr));
                collection_contains || start_contains || end_contains
            }
            Expr::Path(e) => e.elements.iter().any(Self::contains_aggregate_recursive),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_expr() {
        let expr = Expr::Constant(ConstantExpr::new(Value::Int(42), Span::default()));
        assert!(expr.is_constant());
        assert_eq!(expr.to_string(), "Int(42)");
    }

    #[test]
    fn test_variable_expr() {
        let expr = Expr::Variable(VariableExpr::new("x".to_string(), Span::default()));
        assert!(!expr.is_constant());
        assert_eq!(expr.to_string(), "x");
    }

    #[test]
    fn test_binary_expr() {
        let left = Expr::Constant(ConstantExpr::new(Value::Int(5), Span::default()));
        let right = Expr::Constant(ConstantExpr::new(Value::Int(3), Span::default()));
        let expr = Expr::Binary(BinaryExpr::new(left, BinaryOp::Add, right, Span::default()));

        assert!(expr.is_constant());
        assert_eq!(expr.to_string(), "(Int(5) + Int(3))");
    }

    #[test]
    fn test_find_variables() {
        let expr = Expr::Variable(VariableExpr::new("test_var".to_string(), Span::default()));
        let variables = ExprUtils::find_variables(&expr);
        assert_eq!(variables, vec!["test_var"]);
    }

    #[test]
    fn test_contains_aggregate() {
        let func_expr = Expr::FunctionCall(FunctionCallExpr::new(
            "COUNT".to_string(),
            vec![Expr::Variable(VariableExpr::new(
                "x".to_string(),
                Span::default(),
            ))],
            false,
            Span::default(),
        ));
        assert!(ExprUtils::contains_aggregate(&func_expr));
    }
}
