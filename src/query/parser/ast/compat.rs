//! 兼容性适配层
//!
//! 提供与旧 AST 结构的兼容性，确保现有代码能够正常工作。

use super::*;

// 重新导出旧的类型名称以保持兼容性
pub type Expression = Box<dyn super::Expression>;
pub type Statement = Box<dyn super::Statement>;
pub type Pattern = Box<dyn super::Pattern>;

// 表达式类型别名
pub type ConstantExpr = node::ConstantExpr;
pub type VariableExpr = node::VariableExpr;
pub type BinaryExpr = node::BinaryExpr;
pub type UnaryExpr = node::UnaryExpr;
pub type FunctionCallExpr = node::FunctionCallExpr;
pub type PropertyAccessExpr = node::PropertyAccessExpr;
pub type ListExpr = node::ListExpr;
pub type MapExpr = node::MapExpr;
pub type CaseExpr = node::CaseExpr;
pub type SubscriptExpr = node::SubscriptExpr;
pub type PredicateExpr = node::PredicateExpr;

// 语句类型别名
pub type QueryStatement = statement::QueryStatement;
pub type CreateStatement = statement::CreateStatement;
pub type MatchStatement = statement::MatchStatement;
pub type DeleteStatement = statement::DeleteStatement;
pub type UpdateStatement = statement::UpdateStatement;
pub type GoStatement = statement::GoStatement;
pub type FetchStatement = statement::FetchStatement;
pub type UseStatement = statement::UseStatement;
pub type ShowStatement = statement::ShowStatement;
pub type ExplainStatement = statement::ExplainStatement;

// 模式类型别名
pub type NodePattern = pattern::NodePattern;
pub type EdgePattern = pattern::EdgePattern;
pub type PathPattern = pattern::PathPattern;
pub type VariablePattern = pattern::VariablePattern;

// 辅助类型别名
pub type Property = types::Property;
pub type Assignment = statement::Assignment;
pub type YieldClause = statement::YieldClause;
pub type ReturnClause = statement::ReturnClause;
pub type OrderByClause = statement::OrderByClause;

/// Yield 表达式
#[derive(Debug)]
pub struct YieldExpression {
    pub expr: Box<dyn super::Expression>,
    pub alias: Option<String>,
}
// pub type LimitClause = statement::LimitClause;  // 暂时注释掉
// pub type SkipClause = statement::SkipClause;    // 暂时注释掉
// pub type WhereClause = statement::WhereClause;  // 暂时注释掉
// pub type WithClause = statement::WithClause;    // 暂时注释掉
// pub type UnwindClause = statement::UnwindClause; // 暂时注释掉

// 枚举类型别名
pub use node::{BinaryOp, UnaryOp, PredicateType};
pub use statement::{CreateTarget, DeleteTarget, UpdateTarget, Steps, FromClause, ShowTarget, FetchTarget, DataType, EdgeDirection};
pub use pattern::{EdgeRange, PathElement, RepetitionType};
pub use types::{TagIdentifier, MatchClauseDetail, MatchPath, MatchPathSegment, MatchNode, MatchEdge, Label, WhereClause, WithClause, WithItem, MatchClause};

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{Expression as TraitExpression, ExpressionType};
    
    #[test]
    fn test_compatibility_types() {
        let span = Span::default();
        
        // 测试类型别名是否正常工作
        let expr = Box::new(node::ConstantExpr::new(crate::core::Value::Int(42), span));
        
        assert_eq!(expr.expr_type(), ExpressionType::Constant);
    }
}