# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 911
- **Total Warnings**: 63
- **Total Issues**: 974
- **Unique Error Patterns**: 105
- **Unique Warning Patterns**: 37
- **Files with Issues**: 73

## Error Statistics

**Total Errors**: 911

### Error Type Breakdown

- **error[E0407]**: 407 errors
- **error[E0599]**: 330 errors
- **error[E0412]**: 168 errors
- **error[E0433]**: 3 errors
- **error[E0432]**: 2 errors
- **error[E0046]**: 1 errors

### Files with Errors (Top 10)

- `src\query\visitor\find_visitor.rs`: 56 errors
- `src\query\visitor\extract_group_suite_visitor.rs`: 55 errors
- `src\query\parser\ast\visitor.rs`: 45 errors
- `src\query\visitor\extract_prop_expr_visitor.rs`: 43 errors
- `src\query\optimizer\prune_properties_visitor.rs`: 43 errors
- `src\query\visitor\deduce_alias_type_visitor.rs`: 43 errors
- `src\core\expression_utils.rs`: 42 errors
- `src\query\visitor\deduce_props_visitor.rs`: 40 errors
- `src\query\visitor\deduce_type_visitor.rs`: 37 errors
- `src\query\visitor\property_tracker_visitor.rs`: 37 errors

## Warning Statistics

**Total Warnings**: 63

### Warning Type Breakdown

- **warning**: 63 warnings

### Files with Warnings (Top 10)

- `src\query\executor\result_processing\sort.rs`: 7 warnings
- `src\query\executor\aggregation.rs`: 4 warnings
- `src\query\optimizer\join_optimization.rs`: 2 warnings
- `src\query\parser\lexer\lexer.rs`: 2 warnings
- `src\storage\redb_storage.rs`: 2 warnings
- `src\query\optimizer\plan_validator.rs`: 2 warnings
- `src\query\executor\data_processing\graph_traversal\tests.rs`: 2 warnings
- `src\query\optimizer\scan_optimization.rs`: 2 warnings
- `src\storage\memory_storage.rs`: 2 warnings
- `src\services\session.rs`: 2 warnings

## Detailed Error Categorization

### error[E0407]: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`

**Total Occurrences**: 407  
**Unique Files**: 15

#### `src\query\visitor\deduce_props_visitor.rs`: 28 occurrences

- Line 424: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 429: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 434: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\visitor\deduce_type_visitor.rs`: 28 occurrences

- Line 667: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 671: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 675: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\optimizer\prune_properties_visitor.rs`: 28 occurrences

- Line 265: method `visit_subscript_range` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 311: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 315: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\visitor\validate_pattern_expression_visitor.rs`: 28 occurrences

- Line 240: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 244: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 248: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\visitor\extract_prop_expr_visitor.rs`: 28 occurrences

- Line 293: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 299: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 306: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\visitor\evaluable_expr_visitor.rs`: 28 occurrences

- Line 56: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 61: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 66: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\visitor\extract_group_suite_visitor.rs`: 28 occurrences

- Line 309: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 318: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 327: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 28 occurrences

- Line 203: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 207: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 211: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\visitor\property_tracker_visitor.rs`: 28 occurrences

- Line 318: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 325: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 332: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\visitor\variable_visitor.rs`: 28 occurrences

- Line 65: method `visit_variable_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 170: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 172: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\visitor\deduce_alias_type_visitor.rs`: 28 occurrences

- Line 255: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 260: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 265: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\visitor\fold_constant_expr_visitor.rs`: 28 occurrences

- Line 574: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 579: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 584: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 25 more occurrences in this file

#### `src\query\visitor\find_visitor.rs`: 25 occurrences

- Line 279: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 288: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 297: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 22 more occurrences in this file

#### `src\query\visitor\vid_extract_visitor.rs`: 24 occurrences

- Line 382: method `visit_tag_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 386: method `visit_edge_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 390: method `visit_input_property` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 21 more occurrences in this file

#### `src\query\visitor\rewrite_visitor.rs`: 22 occurrences

- Line 267: method `visit_unary_plus` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 273: method `visit_unary_negate` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- Line 279: method `visit_unary_not` is not a member of trait `ExpressionVisitor`: not a member of trait `ExpressionVisitor`
- ... 19 more occurrences in this file

### error[E0599]: no variant or associated item named `Predicate` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`

**Total Occurrences**: 330  
**Unique Files**: 28

#### `src\core\expression_utils.rs`: 42 occurrences

- Line 25: no variant named `EdgeProperty` found for enum `core::types::expression::Expression`
- Line 42: no variant named `TagProperty` found for enum `core::types::expression::Expression`
- Line 131: no variant named `EdgeProperty` found for enum `core::types::expression::Expression`
- ... 39 more occurrences in this file

#### `src\query\parser\expressions\expression_converter.rs`: 30 occurrences

- Line 28: no variant or associated item named `Predicate` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`
- Line 29: no variant or associated item named `TagProperty` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`
- Line 30: no variant or associated item named `EdgeProperty` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`
- ... 27 more occurrences in this file

#### `src\query\validator\order_by_validator.rs`: 28 occurrences

- Line 263: no variant named `TagProperty` found for enum `core::types::expression::Expression`
- Line 264: no variant named `EdgeProperty` found for enum `core::types::expression::Expression`
- Line 265: no variant or associated item named `InputProperty` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`
- ... 25 more occurrences in this file

#### `src\query\validator\strategies\alias_strategy.rs`: 23 occurrences

- Line 105: no variant named `ListComprehension` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 115: no variant named `Predicate` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 119: no variant named `Reduce` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- ... 20 more occurrences in this file

#### `src\query\validator\strategies\expression_strategy.rs`: 21 occurrences

- Line 267: no variant or associated item named `UUID` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`
- Line 268: no variant or associated item named `ESQuery` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`
- Line 269: no variant named `TagProperty` found for enum `core::types::expression::Expression`
- ... 18 more occurrences in this file

#### `src\expression\visitor.rs`: 21 occurrences

- Line 46: no variant named `TagProperty` found for enum `core::types::expression::Expression`
- Line 47: no variant named `EdgeProperty` found for enum `core::types::expression::Expression`
- Line 48: no variant or associated item named `InputProperty` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`
- ... 18 more occurrences in this file

#### `src\query\visitor\find_visitor.rs`: 21 occurrences

- Line 280: no variant or associated item named `TagProperty` found for enum `core::types::expression::ExpressionType` in the current scope: variant or associated item not found in `ExpressionType`
- Line 281: no variant named `TagProperty` found for enum `core::types::expression::Expression`
- Line 289: no variant or associated item named `EdgeProperty` found for enum `core::types::expression::ExpressionType` in the current scope: variant or associated item not found in `ExpressionType`
- ... 18 more occurrences in this file

#### `src\query\validator\strategies\aggregate_strategy.rs`: 18 occurrences

- Line 44: no variant named `ListComprehension` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 53: no variant named `Predicate` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 57: no variant named `Reduce` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- ... 15 more occurrences in this file

#### `src\query\visitor\extract_group_suite_visitor.rs`: 18 occurrences

- Line 135: no variant named `TagProperty` found for enum `core::types::expression::Expression`
- Line 136: no variant named `EdgeProperty` found for enum `core::types::expression::Expression`
- Line 137: no variant or associated item named `InputProperty` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`
- ... 15 more occurrences in this file

#### `src\query\optimizer\optimizer.rs`: 17 occurrences

- Line 958: no variant named `TagProperty` found for enum `core::types::expression::Expression`
- Line 961: no variant named `EdgeProperty` found for enum `core::types::expression::Expression`
- Line 964: no variant named `VariableProperty` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- ... 14 more occurrences in this file

#### `src\query\optimizer\predicate_pushdown.rs`: 15 occurrences

- Line 284: no variant named `TagProperty` found for enum `core::types::expression::Expression`
- Line 288: no variant named `SourceProperty` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 319: no variant named `EdgeProperty` found for enum `core::types::expression::Expression`
- ... 12 more occurrences in this file

#### `src\query\parser\ast\expr.rs`: 13 occurrences

- Line 41: no variant or associated item named `Predicate` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`
- Line 42: no variant or associated item named `TagProperty` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`
- Line 43: no variant or associated item named `EdgeProperty` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`
- ... 10 more occurrences in this file

#### `src\query\parser\ast\visitor.rs`: 9 occurrences

- Line 27: no variant or associated item named `Predicate` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`
- Line 28: no variant or associated item named `TagProperty` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`
- Line 29: no variant or associated item named `EdgeProperty` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`
- ... 6 more occurrences in this file

#### `src\query\validator\strategies\type_inference.rs`: 9 occurrences

- Line 714: no variant named `ListComprehension` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 721: no variant named `Predicate` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 724: no variant named `Reduce` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- ... 6 more occurrences in this file

#### `src\query\optimizer\prune_properties_visitor.rs`: 6 occurrences

- Line 128: no variant named `TagProperty` found for enum `core::types::expression::Expression`
- Line 131: no variant named `EdgeProperty` found for enum `core::types::expression::Expression`
- Line 134: no variant or associated item named `InputProperty` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`
- ... 3 more occurrences in this file

#### `src\query\visitor\deduce_alias_type_visitor.rs`: 6 occurrences

- Line 492: no method named `visit_tag_property` found for mutable reference `&mut DeduceAliasTypeVisitor` in the current scope
- Line 496: no method named `visit_edge_property` found for mutable reference `&mut DeduceAliasTypeVisitor` in the current scope
- Line 500: no method named `visit_input_property` found for mutable reference `&mut DeduceAliasTypeVisitor` in the current scope
- ... 3 more occurrences in this file

#### `src\query\visitor\extract_prop_expr_visitor.rs`: 6 occurrences

- Line 519: no method named `visit_tag_property` found for mutable reference `&mut ExtractPropExprVisitor` in the current scope
- Line 523: no method named `visit_edge_property` found for mutable reference `&mut ExtractPropExprVisitor` in the current scope
- Line 527: no method named `visit_input_property` found for mutable reference `&mut ExtractPropExprVisitor` in the current scope
- ... 3 more occurrences in this file

#### `src\query\optimizer\index_optimization.rs`: 6 occurrences

- Line 496: no variant named `TagProperty` found for enum `core::types::expression::Expression`
- Line 499: no variant named `VariableProperty` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 500: no variant named `EdgeProperty` found for enum `core::types::expression::Expression`
- ... 3 more occurrences in this file

#### `src\query\validator\set_validator.rs`: 3 occurrences

- Line 190: no variant named `ListComprehension` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 196: no variant named `Predicate` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 200: no variant named `Reduce` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`

#### `src\query\visitor\deduce_props_visitor.rs`: 3 occurrences

- Line 787: no variant named `TagProperty` found for enum `core::types::expression::Expression`
- Line 803: no variant named `SourceProperty` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 819: no variant named `EdgeProperty` found for enum `core::types::expression::Expression`

#### `src\query\optimizer\plan_validator.rs`: 3 occurrences

- Line 346: no variant named `ListComprehension` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 352: no variant named `Predicate` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 356: no variant named `Reduce` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`

#### `src\query\parser\ast\utils.rs`: 3 occurrences

- Line 70: no variant or associated item named `Predicate` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`
- Line 502: no variant or associated item named `Predicate` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`
- Line 507: no variant or associated item named `Predicate` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`

#### `src\query\validator\strategies\expression_operations.rs`: 3 occurrences

- Line 71: no variant named `Reduce` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 80: no variant named `Predicate` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`
- Line 88: no variant named `ListComprehension` found for enum `core::types::expression::Expression`: variant not found in `core::types::expression::Expression`

#### `src\expression\evaluator\expression_evaluator.rs`: 2 occurrences

- Line 181: no variant or associated item named `InputProperty` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`
- Line 352: no variant or associated item named `InputProperty` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`

#### `src\query\parser\ast\tests.rs`: 1 occurrences

- Line 156: no variant or associated item named `Predicate` found for enum `expr::Expr` in the current scope: variant or associated item not found in `expr::Expr`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 488: no variant or associated item named `InputProperty` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`

#### `src\query\executor\result_processing\sort.rs`: 1 occurrences

- Line 169: no variant or associated item named `InputProperty` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`

#### `src\query\executor\data_processing\transformations\rollup_apply.rs`: 1 occurrences

- Line 551: no variant or associated item named `InputProperty` found for enum `core::types::expression::Expression` in the current scope: variant or associated item not found in `Expression`

### error[E0412]: cannot find type `PredicateExpr` in this scope: not found in this scope

**Total Occurrences**: 168  
**Unique Files**: 17

#### `src\query\parser\ast\visitor.rs`: 36 occurrences

- Line 74: cannot find type `PredicateExpr` in this scope: not found in this scope
- Line 77: cannot find type `TagPropertyExpr` in this scope: not found in this scope
- Line 80: cannot find type `EdgePropertyExpr` in this scope: not found in this scope
- ... 33 more occurrences in this file

#### `src\query\visitor\rewrite_visitor.rs`: 9 occurrences

- Line 480: cannot find type `PredicateExpr` in this scope: not found in this scope
- Line 487: cannot find type `TagPropertyExpr` in this scope: not found in this scope
- Line 492: cannot find type `EdgePropertyExpr` in this scope: not found in this scope
- ... 6 more occurrences in this file

#### `src\query\visitor\extract_prop_expr_visitor.rs`: 9 occurrences

- Line 512: cannot find type `PredicateExpr` in this scope: not found in this scope
- Line 518: cannot find type `TagPropertyExpr` in this scope: not found in this scope
- Line 522: cannot find type `EdgePropertyExpr` in this scope: not found in this scope
- ... 6 more occurrences in this file

#### `src\query\visitor\extract_filter_expr_visitor.rs`: 9 occurrences

- Line 405: cannot find type `PredicateExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 411: cannot find type `TagPropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 415: cannot find type `EdgePropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- ... 6 more occurrences in this file

#### `src\query\visitor\deduce_type_visitor.rs`: 9 occurrences

- Line 927: cannot find type `PredicateExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 936: cannot find type `TagPropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 944: cannot find type `EdgePropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- ... 6 more occurrences in this file

#### `src\query\visitor\variable_visitor.rs`: 9 occurrences

- Line 344: cannot find type `PredicateExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 352: cannot find type `TagPropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 359: cannot find type `EdgePropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- ... 6 more occurrences in this file

#### `src\query\optimizer\prune_properties_visitor.rs`: 9 occurrences

- Line 501: cannot find type `PredicateExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 509: cannot find type `TagPropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 516: cannot find type `EdgePropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- ... 6 more occurrences in this file

#### `src\query\visitor\validate_pattern_expression_visitor.rs`: 9 occurrences

- Line 460: cannot find type `PredicateExpr` in this scope: not found in this scope
- Line 466: cannot find type `TagPropertyExpr` in this scope: not found in this scope
- Line 470: cannot find type `EdgePropertyExpr` in this scope: not found in this scope
- ... 6 more occurrences in this file

#### `src\query\visitor\evaluable_expr_visitor.rs`: 9 occurrences

- Line 192: cannot find type `PredicateExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 197: cannot find type `TagPropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 202: cannot find type `EdgePropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- ... 6 more occurrences in this file

#### `src\query\visitor\property_tracker_visitor.rs`: 9 occurrences

- Line 535: cannot find type `PredicateExpr` in this scope: not found in this scope
- Line 541: cannot find type `TagPropertyExpr` in this scope: not found in this scope
- Line 546: cannot find type `EdgePropertyExpr` in this scope: not found in this scope
- ... 6 more occurrences in this file

#### `src\query\visitor\fold_constant_expr_visitor.rs`: 9 occurrences

- Line 786: cannot find type `PredicateExpr` in this scope: not found in this scope
- Line 792: cannot find type `TagPropertyExpr` in this scope: not found in this scope
- Line 797: cannot find type `EdgePropertyExpr` in this scope: not found in this scope
- ... 6 more occurrences in this file

#### `src\query\visitor\find_visitor.rs`: 9 occurrences

- Line 504: cannot find type `PredicateExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 512: cannot find type `TagPropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- Line 517: cannot find type `EdgePropertyExpr` in module `crate::query::parser::ast::expr`: not found in `crate::query::parser::ast::expr`
- ... 6 more occurrences in this file

#### `src\query\visitor\deduce_alias_type_visitor.rs`: 9 occurrences

- Line 485: cannot find type `PredicateExpr` in this scope: not found in this scope
- Line 491: cannot find type `TagPropertyExpr` in this scope: not found in this scope
- Line 495: cannot find type `EdgePropertyExpr` in this scope: not found in this scope
- ... 6 more occurrences in this file

#### `src\query\visitor\extract_group_suite_visitor.rs`: 9 occurrences

- Line 538: cannot find type `PredicateExpr` in this scope: not found in this scope
- Line 544: cannot find type `TagPropertyExpr` in this scope: not found in this scope
- Line 548: cannot find type `EdgePropertyExpr` in this scope: not found in this scope
- ... 6 more occurrences in this file

#### `src\query\visitor\deduce_props_visitor.rs`: 9 occurrences

- Line 630: cannot find type `PredicateExpr` in this scope: not found in this scope
- Line 636: cannot find type `TagPropertyExpr` in this scope: not found in this scope
- Line 641: cannot find type `EdgePropertyExpr` in this scope: not found in this scope
- ... 6 more occurrences in this file

#### `src\query\visitor\vid_extract_visitor.rs`: 5 occurrences

- Line 584: cannot find type `PredicateExpr` in this scope: not found in this scope
- Line 590: cannot find type `TagPropertyExpr` in this scope: not found in this scope
- Line 594: cannot find type `EdgePropertyExpr` in this scope: not found in this scope
- ... 2 more occurrences in this file

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 69: cannot find type `PredicateType` in this scope: not found in this scope

### error[E0433]: failed to resolve: use of undeclared type `PredicateExpr`: use of undeclared type `PredicateExpr`

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\query\parser\ast\tests.rs`: 2 occurrences

- Line 156: failed to resolve: use of undeclared type `PredicateExpr`: use of undeclared type `PredicateExpr`
- Line 157: failed to resolve: use of undeclared type `PredicateType`: use of undeclared type `PredicateType`

#### `src\query\parser\ast\utils.rs`: 1 occurrences

- Line 70: failed to resolve: use of undeclared type `PredicateExpr`: use of undeclared type `PredicateExpr`

### error[E0432]: unresolved imports `crate::query::parser::ast::DestinationPropertyExpr`, `crate::query::parser::ast::EdgePropertyExpr`, `crate::query::parser::ast::InputPropertyExpr`, `crate::query::parser::ast::ListComprehensionExpr`, `crate::query::parser::ast::PredicateExpr`, `crate::query::parser::ast::ReduceExpr`, `crate::query::parser::ast::SourcePropertyExpr`, `crate::query::parser::ast::TagPropertyExpr`, `crate::query::parser::ast::VariablePropertyExpr`: no `DestinationPropertyExpr` in `query::parser::ast`, no `EdgePropertyExpr` in `query::parser::ast`, no `InputPropertyExpr` in `query::parser::ast`, no `ListComprehensionExpr` in `query::parser::ast`, no `PredicateExpr` in `query::parser::ast`, no `ReduceExpr` in `query::parser::ast`, no `SourcePropertyExpr` in `query::parser::ast`, no `TagPropertyExpr` in `query::parser::ast`, no `VariablePropertyExpr` in `query::parser::ast`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\parser\expressions\expression_converter.rs`: 2 occurrences

- Line 8: unresolved imports `crate::query::parser::ast::DestinationPropertyExpr`, `crate::query::parser::ast::EdgePropertyExpr`, `crate::query::parser::ast::InputPropertyExpr`, `crate::query::parser::ast::ListComprehensionExpr`, `crate::query::parser::ast::PredicateExpr`, `crate::query::parser::ast::ReduceExpr`, `crate::query::parser::ast::SourcePropertyExpr`, `crate::query::parser::ast::TagPropertyExpr`, `crate::query::parser::ast::VariablePropertyExpr`: no `DestinationPropertyExpr` in `query::parser::ast`, no `EdgePropertyExpr` in `query::parser::ast`, no `InputPropertyExpr` in `query::parser::ast`, no `ListComprehensionExpr` in `query::parser::ast`, no `PredicateExpr` in `query::parser::ast`, no `ReduceExpr` in `query::parser::ast`, no `SourcePropertyExpr` in `query::parser::ast`, no `TagPropertyExpr` in `query::parser::ast`, no `VariablePropertyExpr` in `query::parser::ast`
- Line 457: unresolved imports `crate::query::parser::ast::DestinationPropertyExpr`, `crate::query::parser::ast::EdgePropertyExpr`, `crate::query::parser::ast::InputPropertyExpr`, `crate::query::parser::ast::ListComprehensionExpr`, `crate::query::parser::ast::PredicateExpr`, `crate::query::parser::ast::ReduceExpr`, `crate::query::parser::ast::SourcePropertyExpr`, `crate::query::parser::ast::TagPropertyExpr`, `crate::query::parser::ast::VariablePropertyExpr`: no `DestinationPropertyExpr` in `query::parser::ast`, no `EdgePropertyExpr` in `query::parser::ast`, no `InputPropertyExpr` in `query::parser::ast`, no `ListComprehensionExpr` in `query::parser::ast`, no `PredicateExpr` in `query::parser::ast`, no `ReduceExpr` in `query::parser::ast`, no `SourcePropertyExpr` in `query::parser::ast`, no `TagPropertyExpr` in `query::parser::ast`, no `VariablePropertyExpr` in `query::parser::ast`

### error[E0046]: not all trait items implemented, missing: `visit_list_comprehension`, `visit_predicate`, `visit_reduce`: missing `visit_list_comprehension`, `visit_predicate`, `visit_reduce` in implementation

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\visitor\find_visitor.rs`: 1 occurrences

- Line 91: not all trait items implemented, missing: `visit_list_comprehension`, `visit_predicate`, `visit_reduce`: missing `visit_list_comprehension`, `visit_predicate`, `visit_reduce` in implementation

## Detailed Warning Categorization

### warning: unused imports: `EdgeId` and `TagId`

**Total Occurrences**: 63  
**Unique Files**: 44

#### `src\query\executor\result_processing\sort.rs`: 7 occurrences

- Line 688: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- Line 712: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- Line 736: unused variable: `test_config`: help: if this is intentional, prefix it with an underscore: `_test_config`
- ... 4 more occurrences in this file

#### `src\query\executor\aggregation.rs`: 4 occurrences

- Line 530: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- Line 558: unused variable: `test_path`: help: if this is intentional, prefix it with an underscore: `_test_path`
- Line 559: unused variable: `executor`: help: if this is intentional, prefix it with an underscore: `_executor`
- ... 1 more occurrences in this file

#### `src\storage\memory_storage.rs`: 2 occurrences

- Line 5: unused imports: `EdgeId` and `TagId`
- Line 175: variable does not need to be mutable

#### `src\query\optimizer\scan_optimization.rs`: 2 occurrences

- Line 101: unused import: `crate::query::planner::plan::core::nodes::ScanVerticesNode`
- Line 104: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\optimizer\projection_pushdown.rs`: 2 occurrences

- Line 121: unused import: `crate::query::planner::plan::core::nodes::ProjectNode`
- Line 124: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\parser\lexer\lexer.rs`: 2 occurrences

- Line 1041: variable does not need to be mutable
- Line 1055: variable does not need to be mutable

#### `src\storage\redb_storage.rs`: 2 occurrences

- Line 286: unused variable: `edge_type_bytes`: help: if this is intentional, prefix it with an underscore: `_edge_type_bytes`
- Line 336: unused variable: `edge_type_bytes`: help: if this is intentional, prefix it with an underscore: `_edge_type_bytes`

#### `src\query\executor\data_processing\graph_traversal\tests.rs`: 2 occurrences

- Line 3: unused import: `crate::config::test_config::test_config`
- Line 9: unused import: `crate::query::executor::data_processing::graph_traversal::traverse::TraverseExecutor`

#### `src\query\optimizer\plan_validator.rs`: 2 occurrences

- Line 454: unused import: `crate::api::session::session_manager::SessionInfo`
- Line 456: unused import: `OptGroup`

#### `src\query\validator\strategies\variable_validator.rs`: 2 occurrences

- Line 253: unused import: `std::collections::HashMap`
- Line 257: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\services\session.rs`: 2 occurrences

- Line 50: unused variable: `client_info`: help: if this is intentional, prefix it with an underscore: `_client_info`
- Line 50: unused variable: `connection_info`: help: if this is intentional, prefix it with an underscore: `_connection_info`

#### `src\query\optimizer\join_optimization.rs`: 2 occurrences

- Line 111: unused import: `crate::query::planner::plan::core::nodes::LimitNode`
- Line 114: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\executor\data_processing\join\mod.rs`: 1 occurrences

- Line 252: unused imports: `Direction` and `Value`

#### `src\query\parser\ast\tests.rs`: 1 occurrences

- Line 460: unused import: `super::*`

#### `src\query\executor\data_processing\loops.rs`: 1 occurrences

- Line 550: unused import: `crate::core::value::NullType`

#### `src\query\executor\factory.rs`: 1 occurrences

- Line 568: variable does not need to be mutable

#### `src\query\validator\strategies\expression_operations.rs`: 1 occurrences

- Line 537: unused variable: `validator`: help: if this is intentional, prefix it with an underscore: `_validator`

#### `src\query\optimizer\elimination_rules.rs`: 1 occurrences

- Line 574: unused import: `SortNode`

#### `src\query\planner\statements\path_planner.rs`: 1 occurrences

- Line 75: unused variable: `min_hops`: help: if this is intentional, prefix it with an underscore: `_min_hops`

#### `src\query\executor\result_processing\aggregation.rs`: 1 occurrences

- Line 982: unused import: `crate::core::value::NullType`

#### `src\common\thread.rs`: 1 occurrences

- Line 89: unused variable: `id`: help: if this is intentional, prefix it with an underscore: `_id`

#### `src\query\validator\strategies\aggregate_strategy.rs`: 1 occurrences

- Line 351: unused import: `UnaryOperator`

#### `src\query\optimizer\rule_traits.rs`: 1 occurrences

- Line 726: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\executor\result_processing\projection.rs`: 1 occurrences

- Line 322: unused import: `crate::storage::StorageEngine`

#### `src\query\executor\result_processing\dedup.rs`: 1 occurrences

- Line 493: unused import: `crate::core::value::NullType`

#### `src\query\context\ast\base.rs`: 1 occurrences

- Line 230: unused variable: `query_text`: help: if this is intentional, prefix it with an underscore: `_query_text`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 91: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\context\managers\transaction.rs`: 1 occurrences

- Line 342: unused variable: `tx2`: help: if this is intentional, prefix it with an underscore: `_tx2`

#### `src\index\binary.rs`: 1 occurrences

- Line 329: unused import: `TimeValue`

#### `src\query\executor\data_processing\transformations\assign.rs`: 1 occurrences

- Line 168: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\common\memory.rs`: 1 occurrences

- Line 188: unused doc comment: rustdoc does not generate documentation for macro invocations

#### `src\query\executor\object_pool.rs`: 1 occurrences

- Line 255: variable does not need to be mutable

#### `src\api\service\index_service.rs`: 1 occurrences

- Line 419: unused import: `crate::core::Tag`

#### `src\query\executor\data_processing\transformations\append_vertices.rs`: 1 occurrences

- Line 424: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\api\service\query_engine.rs`: 1 occurrences

- Line 65: unused import: `crate::config::Config`

#### `src\query\context\request_context.rs`: 1 occurrences

- Line 1080: variable does not need to be mutable

#### `src\query\parser\expressions\expression_converter.rs`: 1 occurrences

- Line 458: unused imports: `ListExpr`, `MapExpr`, `PathExpr`, `PropertyAccessExpr`, `RangeExpr`, and `SubscriptExpr`

#### `src\query\validator\go_validator.rs`: 1 occurrences

- Line 334: unused variable: `key`: help: if this is intentional, prefix it with an underscore: `_key`

#### `src\query\optimizer\index_optimization.rs`: 1 occurrences

- Line 1017: unused variable: `session_info`: help: if this is intentional, prefix it with an underscore: `_session_info`

#### `src\query\executor\result_processing\filter.rs`: 1 occurrences

- Line 388: unused import: `crate::core::value::NullType`

#### `src\query\executor\result_processing\sample.rs`: 1 occurrences

- Line 507: unused import: `crate::core::value::NullType`

#### `src\query\executor\data_processing\transformations\unwind.rs`: 1 occurrences

- Line 375: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\executor\data_processing\transformations\pattern_apply.rs`: 1 occurrences

- Line 457: unused variable: `config`: help: if this is intentional, prefix it with an underscore: `_config`

#### `src\query\optimizer\limit_pushdown.rs`: 1 occurrences

- Line 889: unused import: `crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum`

