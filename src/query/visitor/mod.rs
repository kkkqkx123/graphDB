//! 表达式访问器模块
//! 用于表达式分析和转换访问器

mod deduce_alias_type_visitor;
mod deduce_props_visitor;
mod deduce_type_visitor;
mod evaluable_expr_visitor;
mod extract_filter_expr_visitor;
mod extract_group_suite_visitor;
mod extract_prop_expr_visitor;
mod find_visitor;
mod property_tracker_visitor;
mod rewrite_visitor;
mod validate_pattern_expression_visitor;
mod variable_visitor;
mod vid_extract_visitor;
mod fold_constant_expr_visitor;

pub use deduce_alias_type_visitor::{AliasType, DeduceAliasTypeVisitor};
pub use deduce_props_visitor::{DeducePropsVisitor, ExpressionProps};
pub use deduce_type_visitor::{DeduceTypeVisitor, TypeDeductionError};
pub use evaluable_expr_visitor::EvaluableExprVisitor;
pub use extract_filter_expr_visitor::ExtractFilterExprVisitor;
pub use extract_group_suite_visitor::{ExtractGroupSuiteVisitor, GroupSuite};
pub use extract_prop_expr_visitor::{ExtractPropExprVisitor, ExtractedProps};
pub use find_visitor::FindVisitor;
pub use fold_constant_expr_visitor::FoldConstantExprVisitor;
pub use property_tracker_visitor::{PropertyTracker, PropertyTrackerVisitor};
pub use rewrite_visitor::{RewriteVisitor, Matcher, Rewriter};
pub use validate_pattern_expression_visitor::ValidatePatternExpressionVisitor;
pub use variable_visitor::VariableVisitor;
pub use vid_extract_visitor::{VidExtractVisitor, VidPattern};
