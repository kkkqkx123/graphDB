//! Cypher子句结构定义

use crate::query::parser::cypher::ast::expressions::Expression;
use crate::query::parser::cypher::ast::patterns::Pattern;

/// MATCH子句
#[derive(Debug, Clone)]
pub struct MatchClause {
    pub patterns: Vec<Pattern>,
    pub where_clause: Option<WhereClause>,
    pub optional: bool,
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

/// 返回项
#[derive(Debug, Clone)]
pub struct ReturnItem {
    pub expression: Expression,
    pub alias: Option<String>,
}

/// SET操作符
#[derive(Debug, Clone, PartialEq)]
pub enum SetOperator {
    Replace,  // =
    Add,      // +=
    Subtract, // -=
}

/// SET项
#[derive(Debug, Clone)]
pub struct SetItem {
    pub left: Expression,
    pub operator: SetOperator,
    pub right: Expression,
}

/// REMOVE项类型
#[derive(Debug, Clone)]
pub enum RemoveItemType {
    Property,
    Label,
}

/// REMOVE项
#[derive(Debug, Clone)]
pub struct RemoveItem {
    pub expression: Expression,
    pub item_type: RemoveItemType,
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
#[derive(Debug, Clone, PartialEq)]
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
