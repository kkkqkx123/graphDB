//! Cypher语句类型定义

use crate::query::parser::cypher::ast::clauses::*;

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cypher_statement_type() {
        let match_stmt = CypherStatement::Match(MatchClause {
            patterns: Vec::new(),
            where_clause: None,
            optional: false,
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
}
