//! 核心查询模块
//!
//! 提供查询系统的基础类型定义和通用功能。
//! 此模块定义了 `CoreOperationKind` 枚举和各模块间的类型转换 trait。

mod operation_kind;
mod execution_state;

pub use operation_kind::{CoreOperationKind};
pub use execution_state::{
    QueryExecutionState, ExecutorState, LoopExecutionState,
    RowStatus, OptimizationState, OptimizationPhase,
};

use crate::query::parser::ast::Stmt;
use crate::query::validator::validation_factory::StatementType;

pub trait IntoOperationKind {
    fn into_operation_kind(&self) -> CoreOperationKind;
}

impl IntoOperationKind for Stmt {
    fn into_operation_kind(&self) -> CoreOperationKind {
        match self {
            Stmt::Query(_) => CoreOperationKind::Project,
            Stmt::Create(_) => CoreOperationKind::CreateSpace,
            Stmt::Match(_) => CoreOperationKind::Match,
            Stmt::Delete(_) => CoreOperationKind::Delete,
            Stmt::Update(_) => CoreOperationKind::Update,
            Stmt::Go(_) => CoreOperationKind::Go,
            Stmt::Fetch(_) => CoreOperationKind::GetVertices,
            Stmt::Use(_) => CoreOperationKind::UseSpace,
            Stmt::Show(_) => CoreOperationKind::Show,
            Stmt::Explain(_) => CoreOperationKind::Explain,
            Stmt::Lookup(_) => CoreOperationKind::Lookup,
            Stmt::Subgraph(_) => CoreOperationKind::GetSubgraph,
            Stmt::FindPath(_) => CoreOperationKind::FindPath,
            Stmt::Insert(_) => CoreOperationKind::Insert,
            Stmt::Merge(_) => CoreOperationKind::Merge,
            Stmt::Unwind(_) => CoreOperationKind::Unwind,
            Stmt::Return(_) => CoreOperationKind::Project,
            Stmt::With(_) => CoreOperationKind::Project,
            Stmt::Set(_) => CoreOperationKind::Set,
            Stmt::Remove(_) => CoreOperationKind::Remove,
            Stmt::Pipe(_) => CoreOperationKind::Pipe,
            Stmt::Drop(_) => CoreOperationKind::DropSpace,
            Stmt::Desc(_) => CoreOperationKind::DescribeSpace,
            Stmt::Alter(_) => CoreOperationKind::AlterTag,
            Stmt::CreateUser(_) => CoreOperationKind::CreateUser,
            Stmt::AlterUser(_) => CoreOperationKind::AlterUser,
            Stmt::DropUser(_) => CoreOperationKind::DropUser,
            Stmt::ChangePassword(_) => CoreOperationKind::ChangePassword,
        }
    }
}

impl IntoOperationKind for StatementType {
    fn into_operation_kind(&self) -> CoreOperationKind {
        match self {
            StatementType::Match => CoreOperationKind::Match,
            StatementType::Go => CoreOperationKind::Go,
            StatementType::FetchVertices => CoreOperationKind::GetVertices,
            StatementType::FetchEdges => CoreOperationKind::GetEdges,
            StatementType::Lookup => CoreOperationKind::Lookup,
            StatementType::FindPath => CoreOperationKind::FindPath,
            StatementType::GetSubgraph => CoreOperationKind::GetSubgraph,
            StatementType::InsertVertices => CoreOperationKind::Insert,
            StatementType::InsertEdges => CoreOperationKind::Insert,
            StatementType::Update => CoreOperationKind::Update,
            StatementType::Delete => CoreOperationKind::Delete,
            StatementType::Unwind => CoreOperationKind::Unwind,
            StatementType::Yield => CoreOperationKind::Project,
            StatementType::OrderBy => CoreOperationKind::Sort,
            StatementType::Limit => CoreOperationKind::Limit,
            StatementType::GroupBy => CoreOperationKind::Aggregate,
            StatementType::CreateSpace => CoreOperationKind::CreateSpace,
            StatementType::CreateTag => CoreOperationKind::CreateTag,
            StatementType::CreateEdge => CoreOperationKind::CreateEdge,
            StatementType::AlterTag => CoreOperationKind::AlterTag,
            StatementType::AlterEdge => CoreOperationKind::AlterEdge,
            StatementType::DropSpace => CoreOperationKind::DropSpace,
            StatementType::DropTag => CoreOperationKind::DropTag,
            StatementType::DropEdge => CoreOperationKind::DropEdge,
            StatementType::DescribeSpace => CoreOperationKind::DescribeSpace,
            StatementType::DescribeTag => CoreOperationKind::DescribeTag,
            StatementType::DescribeEdge => CoreOperationKind::DescribeEdge,
            StatementType::ShowSpaces => CoreOperationKind::ShowSpaces,
            StatementType::ShowTags => CoreOperationKind::ShowTags,
            StatementType::ShowEdges => CoreOperationKind::ShowEdges,
            StatementType::Use => CoreOperationKind::UseSpace,
            StatementType::Assignment => CoreOperationKind::Assignment,
            StatementType::Set => CoreOperationKind::Set,
            StatementType::Pipe => CoreOperationKind::Pipe,
            StatementType::Sequential => CoreOperationKind::Sequential,
            StatementType::Explain => CoreOperationKind::Explain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::parser::ast::stmt::Stmt;
    use crate::query::parser::ast::types::Span;
    
    #[test]
    fn test_stmt_to_operation_kind() {
        let match_stmt = Stmt::Match(crate::query::parser::ast::stmt::MatchStmt {
            span: Span::default(),
            patterns: vec![],
            where_clause: None,
            return_clause: None,
            order_by: None,
            limit: None,
            skip: None,
        });
        assert_eq!(match_stmt.into_operation_kind(), CoreOperationKind::Match);
        
        let go_stmt = Stmt::Go(crate::query::parser::ast::stmt::GoStmt {
            span: Span::default(),
            steps: crate::query::parser::ast::stmt::Steps::Fixed(1),
            from: crate::query::parser::ast::stmt::FromClause {
                span: Span::default(),
                vertices: vec![],
            },
            over: None,
            where_clause: None,
            yield_clause: None,
        });
        assert_eq!(go_stmt.into_operation_kind(), CoreOperationKind::Go);
    }
    
    #[test]
    fn test_operation_kind_properties() {
        let match_op = CoreOperationKind::Match;
        assert!(match_op.is_read_only());
        assert!(!match_op.is_metadata_operation());
        assert!(!match_op.is_dml());
        assert!(!match_op.is_ddl());
        
        let create_space = CoreOperationKind::CreateSpace;
        assert!(!create_space.is_read_only());
        assert!(create_space.is_metadata_operation());
        assert!(!create_space.is_dml());
        assert!(create_space.is_ddl());
        
        let insert = CoreOperationKind::Insert;
        assert!(!insert.is_read_only());
        assert!(!insert.is_metadata_operation());
        assert!(insert.is_dml());
        assert!(!insert.is_ddl());
    }
    
    #[test]
    fn test_operation_kind_display() {
        assert_eq!(format!("{}", CoreOperationKind::Match), "MATCH");
        assert_eq!(format!("{}", CoreOperationKind::CreateSpace), "CREATE_SPACE");
        assert_eq!(format!("{}", CoreOperationKind::InnerJoin), "INNER_JOIN");
    }
    
    #[test]
    fn test_operation_kind_category() {
        assert_eq!(CoreOperationKind::Match.category(), "DATA_QUERY");
        assert_eq!(CoreOperationKind::CreateSpace.category(), "SPACE_MANAGEMENT");
        assert_eq!(CoreOperationKind::Project.category(), "DATA_TRANSFORMATION");
        assert_eq!(CoreOperationKind::InnerJoin.category(), "JOIN");
    }
}
