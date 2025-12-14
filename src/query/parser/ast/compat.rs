//! 兼容性层 - 新旧 AST 转换
//!
//! 提供新旧 AST 之间的转换功能，支持渐进式迁移。

use crate::core::Value;
use super::types::*;
use super::expr::*;
use super::stmt::*;
use super::pattern::*;

// 从旧 AST 导入类型
use crate::query::parser::ast_old::{
    AstNode as OldAstNode,
    Expression as OldExpression,
    ExpressionType as OldExpressionType,
    Statement as OldStatement,
    Pattern as OldPattern,
    PatternType as OldPatternType,
    BinaryOp as OldBinaryOp,
    UnaryOp as OldUnaryOp,
    PredicateType as OldPredicateType,
    EdgeDirection as OldEdgeDirection,
    Span as OldSpan,
    Position as OldPosition,
};

/// AST 兼容性转换器
pub struct AstCompat;

impl AstCompat {
    /// 将新表达式转换为旧表达式
    pub fn convert_expr_to_old(expr: &Expr) -> Box<dyn OldExpression> {
        match expr {
            Expr::Constant(e) => {
                Box::new(crate::query::parser::ast_old::node::ConstantExpr::new(
                    e.value.clone(),
                    Self::convert_span_to_old(e.span),
                ))
            }
            Expr::Variable(e) => {
                Box::new(crate::query::parser::ast_old::node::VariableExpr::new(
                    e.name.clone(),
                    Self::convert_span_to_old(e.span),
                ))
            }
            Expr::Binary(e) => {
                let left = Self::convert_expr_to_old(&e.left);
                let right = Self::convert_expr_to_old(&e.right);
                Box::new(crate::query::parser::ast_old::node::BinaryExpr::new(
                    left,
                    Self::convert_binary_op_to_old(e.op),
                    right,
                    Self::convert_span_to_old(e.span),
                ))
            }
            Expr::Unary(e) => {
                let operand = Self::convert_expr_to_old(&e.operand);
                Box::new(crate::query::parser::ast_old::node::UnaryExpr::new(
                    Self::convert_unary_op_to_old(e.op),
                    operand,
                    Self::convert_span_to_old(e.span),
                ))
            }
            Expr::FunctionCall(e) => {
                let args: Vec<Box<dyn OldExpression>> = e.args.iter()
                    .map(|arg| Self::convert_expr_to_old(arg))
                    .collect();
                Box::new(crate::query::parser::ast_old::node::FunctionCallExpr::new(
                    e.name.clone(),
                    args,
                    e.distinct,
                    Self::convert_span_to_old(e.span),
                ))
            }
            Expr::PropertyAccess(e) => {
                let object = Self::convert_expr_to_old(&e.object);
                Box::new(crate::query::parser::ast_old::node::PropertyAccessExpr::new(
                    object,
                    e.property.clone(),
                    Self::convert_span_to_old(e.span),
                ))
            }
            Expr::List(e) => {
                let elements: Vec<Box<dyn OldExpression>> = e.elements.iter()
                    .map(|elem| Self::convert_expr_to_old(elem))
                    .collect();
                Box::new(crate::query::parser::ast_old::node::ListExpr::new(
                    elements,
                    Self::convert_span_to_old(e.span),
                ))
            }
            Expr::Map(e) => {
                let pairs: Vec<(String, Box<dyn OldExpression>)> = e.pairs.iter()
                    .map(|(key, value)| (key.clone(), Self::convert_expr_to_old(value)))
                    .collect();
                Box::new(crate::query::parser::ast_old::node::MapExpr::new(
                    pairs,
                    Self::convert_span_to_old(e.span),
                ))
            }
            Expr::Case(e) => {
                let match_expr = e.match_expr.as_ref().map(|expr| Self::convert_expr_to_old(expr));
                let when_then_pairs: Vec<(Box<dyn OldExpression>, Box<dyn OldExpression>)> = e.when_then_pairs.iter()
                    .map(|(when, then)| (Self::convert_expr_to_old(when), Self::convert_expr_to_old(then)))
                    .collect();
                let default = e.default.as_ref().map(|expr| Self::convert_expr_to_old(expr));
                Box::new(crate::query::parser::ast_old::node::CaseExpr::new(
                    match_expr,
                    when_then_pairs,
                    default,
                    Self::convert_span_to_old(e.span),
                ))
            }
            Expr::Subscript(e) => {
                let collection = Self::convert_expr_to_old(&e.collection);
                let index = Self::convert_expr_to_old(&e.index);
                Box::new(crate::query::parser::ast_old::node::SubscriptExpr::new(
                    collection,
                    index,
                    Self::convert_span_to_old(e.span),
                ))
            }
            Expr::Predicate(e) => {
                let list = Self::convert_expr_to_old(&e.list);
                let condition = Self::convert_expr_to_old(&e.condition);
                Box::new(crate::query::parser::ast_old::node::PredicateExpr::new(
                    Self::convert_predicate_type_to_old(e.predicate),
                    list,
                    condition,
                    Self::convert_span_to_old(e.span),
                ))
            }
        }
    }
    
    /// 将旧表达式转换为新表达式
    pub fn convert_expr_from_old(expr: &dyn OldExpression) -> Expr {
        // 获取表达式类型
        let expr_type = expr.expr_type();
        
        match expr_type {
            OldExpr::Constant => {
                let constant = expr.as_any().downcast_ref::<crate::query::parser::ast_old::node::ConstantExpr>()
                    .expect("Failed to downcast to ConstantExpr");
                Expr::Constant(ConstantExpr::new(
                    constant.value.clone(),
                    Self::convert_span_from_old(constant.span()),
                ))
            }
            OldExpr::Variable => {
                let variable = expr.as_any().downcast_ref::<crate::query::parser::ast_old::node::VariableExpr>()
                    .expect("Failed to downcast to VariableExpr");
                Expr::Variable(VariableExpr::new(
                    variable.name.clone(),
                    Self::convert_span_from_old(variable.span()),
                ))
            }
            OldExpr::Binary => {
                let binary = expr.as_any().downcast_ref::<crate::query::parser::ast_old::node::BinaryExpr>()
                    .expect("Failed to downcast to BinaryExpr");
                let left = Self::convert_expr_from_old(&*binary.left);
                let right = Self::convert_expr_from_old(&*binary.right);
                Expr::Binary(BinaryExpr::new(
                    left,
                    Self::convert_binary_op_from_old(binary.op),
                    right,
                    Self::convert_span_from_old(binary.span()),
                ))
            }
            OldExpr::Unary => {
                let unary = expr.as_any().downcast_ref::<crate::query::parser::ast_old::node::UnaryExpr>()
                    .expect("Failed to downcast to UnaryExpr");
                let operand = Self::convert_expr_from_old(&*unary.operand);
                Expr::Unary(UnaryExpr::new(
                    Self::convert_unary_op_from_old(unary.op),
                    operand,
                    Self::convert_span_from_old(unary.span()),
                ))
            }
            OldExpr::FunctionCall => {
                let func = expr.as_any().downcast_ref::<crate::query::parser::ast_old::node::FunctionCallExpr>()
                    .expect("Failed to downcast to FunctionCallExpr");
                let args: Vec<Expr> = func.args.iter()
                    .map(|arg| Self::convert_expr_from_old(&**arg))
                    .collect();
                Expr::FunctionCall(FunctionCallExpr::new(
                    func.name.clone(),
                    args,
                    func.distinct,
                    Self::convert_span_from_old(func.span()),
                ))
            }
            OldExpr::PropertyAccess => {
                let prop = expr.as_any().downcast_ref::<crate::query::parser::ast_old::node::PropertyAccessExpr>()
                    .expect("Failed to downcast to PropertyAccessExpr");
                let object = Self::convert_expr_from_old(&*prop.object);
                Expr::PropertyAccess(PropertyAccessExpr::new(
                    object,
                    prop.property.clone(),
                    Self::convert_span_from_old(prop.span()),
                ))
            }
            OldExpr::List => {
                let list = expr.as_any().downcast_ref::<crate::query::parser::ast_old::node::ListExpr>()
                    .expect("Failed to downcast to ListExpr");
                let elements: Vec<Expr> = list.elements.iter()
                    .map(|elem| Self::convert_expr_from_old(&**elem))
                    .collect();
                Expr::List(ListExpr::new(
                    elements,
                    Self::convert_span_from_old(list.span()),
                ))
            }
            OldExpr::Map => {
                let map = expr.as_any().downcast_ref::<crate::query::parser::ast_old::node::MapExpr>()
                    .expect("Failed to downcast to MapExpr");
                let pairs: Vec<(String, Expr)> = map.pairs.iter()
                    .map(|(key, value)| (key.clone(), Self::convert_expr_from_old(&**value)))
                    .collect();
                Expr::Map(MapExpr::new(
                    pairs,
                    Self::convert_span_from_old(map.span()),
                ))
            }
            OldExpr::Case => {
                let case = expr.as_any().downcast_ref::<crate::query::parser::ast_old::node::CaseExpr>()
                    .expect("Failed to downcast to CaseExpr");
                let match_expr = case.match_expr.as_ref().map(|expr| Self::convert_expr_from_old(&**expr));
                let when_then_pairs: Vec<(Expr, Expr)> = case.when_then_pairs.iter()
                    .map(|(when, then)| (Self::convert_expr_from_old(&**when), Self::convert_expr_from_old(&**then)))
                    .collect();
                let default = case.default.as_ref().map(|expr| Self::convert_expr_from_old(&**expr));
                Expr::Case(CaseExpr::new(
                    match_expr,
                    when_then_pairs,
                    default,
                    Self::convert_span_from_old(case.span()),
                ))
            }
            OldExpr::Subscript => {
                let subscript = expr.as_any().downcast_ref::<crate::query::parser::ast_old::node::SubscriptExpr>()
                    .expect("Failed to downcast to SubscriptExpr");
                let collection = Self::convert_expr_from_old(&*subscript.collection);
                let index = Self::convert_expr_from_old(&*subscript.index);
                Expr::Subscript(SubscriptExpr::new(
                    collection,
                    index,
                    Self::convert_span_from_old(subscript.span()),
                ))
            }
            OldExpr::Predicate => {
                let pred = expr.as_any().downcast_ref::<crate::query::parser::ast_old::node::PredicateExpr>()
                    .expect("Failed to downcast to PredicateExpr");
                let list = Self::convert_expr_from_old(&*pred.list);
                let condition = Self::convert_expr_from_old(&*pred.condition);
                Expr::Predicate(PredicateExpr::new(
                    Self::convert_predicate_type_from_old(pred.predicate),
                    list,
                    condition,
                    Self::convert_span_from_old(pred.span()),
                ))
            }
        }
    }
    
    /// 类型转换辅助函数
    fn convert_span_to_old(span: Span) -> OldSpan {
        OldSpan::new(
            OldPosition::new(span.start.line, span.start.column),
            OldPosition::new(span.end.line, span.end.column),
        )
    }
    
    fn convert_span_from_old(span: OldSpan) -> Span {
        Span::new(
            Position::new(span.start.line, span.start.column),
            Position::new(span.end.line, span.end.column),
        )
    }
    
    fn convert_binary_op_to_old(op: BinaryOp) -> OldBinaryOp {
        match op {
            BinaryOp::Add => OldBinaryOp::Add,
            BinaryOp::Sub => OldBinaryOp::Sub,
            BinaryOp::Mul => OldBinaryOp::Mul,
            BinaryOp::Div => OldBinaryOp::Div,
            BinaryOp::Mod => OldBinaryOp::Mod,
            BinaryOp::Exp => OldBinaryOp::Exp,
            BinaryOp::And => OldBinaryOp::And,
            BinaryOp::Or => OldBinaryOp::Or,
            BinaryOp::Xor => OldBinaryOp::Xor,
            BinaryOp::Eq => OldBinaryOp::Eq,
            BinaryOp::Ne => OldBinaryOp::Ne,
            BinaryOp::Lt => OldBinaryOp::Lt,
            BinaryOp::Le => OldBinaryOp::Le,
            BinaryOp::Gt => OldBinaryOp::Gt,
            BinaryOp::Ge => OldBinaryOp::Ge,
            BinaryOp::Regex => OldBinaryOp::Regex,
            BinaryOp::In => OldBinaryOp::In,
            BinaryOp::NotIn => OldBinaryOp::NotIn,
            BinaryOp::Contains => OldBinaryOp::Contains,
            BinaryOp::StartsWith => OldBinaryOp::StartsWith,
            BinaryOp::EndsWith => OldBinaryOp::EndsWith,
        }
    }
    
    fn convert_binary_op_from_old(op: OldBinaryOp) -> BinaryOp {
        match op {
            OldBinaryOp::Add => BinaryOp::Add,
            OldBinaryOp::Sub => BinaryOp::Sub,
            OldBinaryOp::Mul => BinaryOp::Mul,
            OldBinaryOp::Div => BinaryOp::Div,
            OldBinaryOp::Mod => BinaryOp::Mod,
            OldBinaryOp::Exp => BinaryOp::Exp,
            OldBinaryOp::And => BinaryOp::And,
            OldBinaryOp::Or => BinaryOp::Or,
            OldBinaryOp::Xor => BinaryOp::Xor,
            OldBinaryOp::Eq => BinaryOp::Eq,
            OldBinaryOp::Ne => BinaryOp::Ne,
            OldBinaryOp::Lt => BinaryOp::Lt,
            OldBinaryOp::Le => BinaryOp::Le,
            OldBinaryOp::Gt => BinaryOp::Gt,
            OldBinaryOp::Ge => BinaryOp::Ge,
            OldBinaryOp::Regex => BinaryOp::Regex,
            OldBinaryOp::In => BinaryOp::In,
            OldBinaryOp::NotIn => BinaryOp::NotIn,
            OldBinaryOp::Contains => BinaryOp::Contains,
            OldBinaryOp::StartsWith => BinaryOp::StartsWith,
            OldBinaryOp::EndsWith => BinaryOp::EndsWith,
        }
    }
    
    fn convert_unary_op_to_old(op: UnaryOp) -> OldUnaryOp {
        match op {
            UnaryOp::Not => OldUnaryOp::Not,
            UnaryOp::Plus => OldUnaryOp::Plus,
            UnaryOp::Minus => OldUnaryOp::Minus,
            UnaryOp::IsNull => OldUnaryOp::IsNull,
            UnaryOp::IsNotNull => OldUnaryOp::IsNotNull,
            UnaryOp::IsEmpty => OldUnaryOp::IsEmpty,
            UnaryOp::IsNotEmpty => OldUnaryOp::IsNotEmpty,
        }
    }
    
    fn convert_unary_op_from_old(op: OldUnaryOp) -> UnaryOp {
        match op {
            OldUnaryOp::Not => UnaryOp::Not,
            OldUnaryOp::Plus => UnaryOp::Plus,
            OldUnaryOp::Minus => UnaryOp::Minus,
            OldUnaryOp::IsNull => UnaryOp::IsNull,
            OldUnaryOp::IsNotNull => UnaryOp::IsNotNull,
            OldUnaryOp::IsEmpty => UnaryOp::IsEmpty,
            OldUnaryOp::IsNotEmpty => UnaryOp::IsNotEmpty,
        }
    }
    
    fn convert_predicate_type_to_old(op: PredicateType) -> OldPredicateType {
        match op {
            PredicateType::All => OldPredicateType::All,
            PredicateType::Any => OldPredicateType::Any,
            PredicateType::Single => OldPredicateType::Single,
            PredicateType::None => OldPredicateType::None,
            PredicateType::Exists => OldPredicateType::Exists,
        }
    }
    
    fn convert_predicate_type_from_old(op: OldPredicateType) -> PredicateType {
        match op {
            OldPredicateType::All => PredicateType::All,
            OldPredicateType::Any => PredicateType::Any,
            OldPredicateType::Single => PredicateType::Single,
            OldPredicateType::None => PredicateType::None,
            OldPredicateType::Exists => PredicateType::Exists,
        }
    }
    
    fn convert_edge_direction_to_old(dir: EdgeDirection) -> OldEdgeDirection {
        match dir {
            EdgeDirection::Out => OldEdgeDirection::Out,
            EdgeDirection::In => OldEdgeDirection::In,
            EdgeDirection::Both => OldEdgeDirection::Both,
        }
    }
    
    fn convert_edge_direction_from_old(dir: OldEdgeDirection) -> EdgeDirection {
        match dir {
            OldEdgeDirection::Out => EdgeDirection::Out,
            OldEdgeDirection::In => EdgeDirection::In,
            OldEdgeDirection::Both => EdgeDirection::Both,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_expr_conversion_roundtrip() {
        // 创建新表达式
        let new_expr = Expr::Constant(ConstantExpr::new(
            Value::Int(42),
            Span::default(),
        ));
        
        // 转换为旧表达式
        let old_expr = AstCompat::convert_expr_to_old(&new_expr);
        
        // 转换回新表达式
        let converted_expr = AstCompat::convert_expr_from_old(&*old_expr);
        
        // 验证相等性
        assert_eq!(new_expr, converted_expr);
    }
    
    #[test]
    fn test_binary_expr_conversion() {
        let left = Expr::Constant(ConstantExpr::new(Value::Int(5), Span::default()));
        let right = Expr::Constant(ConstantExpr::new(Value::Int(3), Span::default()));
        let new_expr = Expr::Binary(BinaryExpr::new(left, BinaryOp::Add, right, Span::default()));
        
        let old_expr = AstCompat::convert_expr_to_old(&new_expr);
        let converted_expr = AstCompat::convert_expr_from_old(&*old_expr);
        
        assert_eq!(new_expr, converted_expr);
    }
    
    #[test]
    fn test_function_call_conversion() {
        let args = vec![
            Expr::Constant(ConstantExpr::new(Value::Int(1), Span::default())),
            Expr::Constant(ConstantExpr::new(Value::Int(2), Span::default())),
        ];
        let new_expr = Expr::FunctionCall(FunctionCallExpr::new(
            "SUM".to_string(),
            args,
            false,
            Span::default(),
        ));
        
        let old_expr = AstCompat::convert_expr_to_old(&new_expr);
        let converted_expr = AstCompat::convert_expr_from_old(&*old_expr);
        
        assert_eq!(new_expr, converted_expr);
    }
}