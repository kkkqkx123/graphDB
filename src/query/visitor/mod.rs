//! 表达式访问器模块
//! 用于表达式分析和转换访问器
//!

mod deduce_type_visitor;
mod extract_group_suite_visitor;
mod fold_constant_expr_visitor;
mod plan_node_visitor;
mod stmt_visitor;
mod ast_traverser;
mod stmt_transformer;
mod ast_transformer;

pub use deduce_type_visitor::{DeduceTypeVisitor, TypeDeductionError};
pub use fold_constant_expr_visitor::{FoldConstantExprVisitor, VisitorError, VisitorResult};
pub use plan_node_visitor::{PlanNodeVisitor, DefaultPlanNodeVisitor};
pub use stmt_visitor::StmtVisitor;
pub use ast_traverser::AstTraverser;
pub use stmt_transformer::StmtTransformer;
pub use ast_transformer::AstTransformer;

// 重新导出工具函数，保持向后兼容
pub use crate::core::types::expression::utils::{
    extract_group_suite, extract_aggregate_functions, has_aggregate_function,
    collect_variables, find_all, is_evaluable, is_constant, GroupSuite,
};
