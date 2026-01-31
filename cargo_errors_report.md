# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 116
- **Total Warnings**: 0
- **Total Issues**: 116
- **Unique Error Patterns**: 24
- **Unique Warning Patterns**: 0
- **Files with Issues**: 20

## Error Statistics

**Total Errors**: 116

### Error Type Breakdown

- **error[E0308]**: 74 errors
- **error[E0053]**: 18 errors
- **error[E0599]**: 6 errors
- **error[E0107]**: 3 errors
- **error[E0603]**: 2 errors
- **error[E0433]**: 2 errors
- **error[E0502]**: 2 errors
- **error[E0277]**: 2 errors
- **error[E0061]**: 2 errors
- **error[E0505]**: 2 errors
- **error[E0432]**: 1 errors
- **error[E0506]**: 1 errors
- **error[E0596]**: 1 errors

### Files with Errors (Top 10)

- `src\query\optimizer\predicate_pushdown.rs`: 22 errors
- `src\query\optimizer\index_optimization.rs`: 21 errors
- `src\query\optimizer\operation_merge.rs`: 12 errors
- `src\query\optimizer\plan\node.rs`: 11 errors
- `src\query\optimizer\engine\optimizer.rs`: 11 errors
- `src\query\optimizer\scan_optimization.rs`: 7 errors
- `src\query\optimizer\join_optimization.rs`: 4 errors
- `src\query\optimizer\projection_pushdown.rs`: 4 errors
- `src\query\optimizer\limit_pushdown.rs`: 3 errors
- `src\query\optimizer\rule_traits.rs`: 3 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: mismatched types: expected `usize`, found `&usize`

**Total Occurrences**: 74  
**Unique Files**: 15

#### `src\query\optimizer\predicate_pushdown.rs`: 22 occurrences

- Line 191: mismatched types: expected `TransformResult`, found `Rc<RefCell<OptGroupNode>>`
- Line 1018: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`
- Line 1040: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`
- ... 19 more occurrences in this file

#### `src\query\optimizer\index_optimization.rs`: 15 occurrences

- Line 45: mismatched types: expected `usize`, found `&usize`
- Line 1063: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`
- Line 1064: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`
- ... 12 more occurrences in this file

#### `src\query\optimizer\operation_merge.rs`: 12 occurrences

- Line 516: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`
- Line 519: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`
- Line 556: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`
- ... 9 more occurrences in this file

#### `src\query\optimizer\scan_optimization.rs`: 5 occurrences

- Line 30: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`
- Line 69: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`
- Line 119: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`
- ... 2 more occurrences in this file

#### `src\query\optimizer\join_optimization.rs`: 3 occurrences

- Line 150: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`
- Line 151: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`
- Line 154: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

#### `src\query\optimizer\rule_traits.rs`: 3 occurrences

- Line 754: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`
- Line 755: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`
- Line 756: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`

#### `src\query\optimizer\engine\optimizer.rs`: 3 occurrences

- Line 163: mismatched types: expected `&mut Rc<RefCell<OptGroup>>`, found `&mut OptGroup`
- Line 188: mismatched types: expected `&mut Rc<RefCell<OptGroup>>`, found `&mut OptGroup`
- Line 688: mismatched types: expected `Option<PlanNodeEnum>`, found `PlanNodeEnum`

#### `src\query\optimizer\limit_pushdown.rs`: 3 occurrences

- Line 1103: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`
- Line 1112: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`
- Line 1120: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

#### `src\query\optimizer\projection_pushdown.rs`: 2 occurrences

- Line 218: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`
- Line 240: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 477: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

#### `src\query\optimizer\subquery_optimization.rs`: 1 occurrences

- Line 146: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

#### `src\query\optimizer\push_filter_down_aggregate.rs`: 1 occurrences

- Line 109: mismatched types: expected `OptGroupNode`, found `&OptGroupNode`

#### `src\query\optimizer\loop_unrolling.rs`: 1 occurrences

- Line 380: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

#### `src\query\optimizer\constant_folding.rs`: 1 occurrences

- Line 485: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

#### `src\query\optimizer\predicate_reorder.rs`: 1 occurrences

- Line 178: mismatched types: expected `&Rc<RefCell<OptGroupNode>>`, found `&OptGroupNode`

### error[E0053]: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`

**Total Occurrences**: 18  
**Unique Files**: 11

#### `src\query\optimizer\index_optimization.rs`: 6 occurrences

- Line 133: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`
- Line 236: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`
- Line 278: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`
- ... 3 more occurrences in this file

#### `src\query\optimizer\scan_optimization.rs`: 2 occurrences

- Line 21: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`
- Line 61: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`

#### `src\query\optimizer\projection_pushdown.rs`: 2 occurrences

- Line 75: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`
- Line 140: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`

#### `src\query\optimizer\join_optimization.rs`: 1 occurrences

- Line 21: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`

#### `src\query\optimizer\predicate_reorder.rs`: 1 occurrences

- Line 25: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`

#### `src\query\optimizer\rule_registry.rs`: 1 occurrences

- Line 119: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`

#### `src\query\optimizer\transformation_rules.rs`: 1 occurrences

- Line 23: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`

#### `src\query\optimizer\loop_unrolling.rs`: 1 occurrences

- Line 26: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`

#### `src\query\optimizer\push_filter_down_aggregate.rs`: 1 occurrences

- Line 29: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`

#### `src\query\optimizer\constant_folding.rs`: 1 occurrences

- Line 26: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`

#### `src\query\optimizer\subquery_optimization.rs`: 1 occurrences

- Line 24: method `apply` has an incompatible type for trait: expected `Rc<RefCell<OptGroupNode>>`, found `node::OptGroupNode`

### error[E0599]: no variant or associated item named `default` found for enum `plan_node_enum::PlanNodeEnum` in the current scope: variant or associated item not found in `PlanNodeEnum`

**Total Occurrences**: 6  
**Unique Files**: 2

#### `src\query\optimizer\plan\node.rs`: 5 occurrences

- Line 42: no variant or associated item named `default` found for enum `plan_node_enum::PlanNodeEnum` in the current scope: variant or associated item not found in `PlanNodeEnum`
- Line 580: no variant or associated item named `default` found for enum `plan_node_enum::PlanNodeEnum` in the current scope: variant or associated item not found in `PlanNodeEnum`
- Line 587: no variant or associated item named `default` found for enum `plan_node_enum::PlanNodeEnum` in the current scope: variant or associated item not found in `PlanNodeEnum`
- ... 2 more occurrences in this file

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 683: no method named `add_input` found for enum `plan_node_enum::PlanNodeEnum` in the current scope: method not found in `PlanNodeEnum`

### error[E0107]: type alias takes 1 generic argument but 2 generic arguments were supplied: expected 1 generic argument

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\query\optimizer\plan\node.rs`: 3 occurrences

- Line 198: type alias takes 1 generic argument but 2 generic arguments were supplied: expected 1 generic argument
- Line 204: type alias takes 1 generic argument but 2 generic arguments were supplied: expected 1 generic argument
- Line 219: type alias takes 1 generic argument but 2 generic arguments were supplied: expected 1 generic argument

### error[E0603]: struct import `OptimizerError` is private: private struct import

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\core\error.rs`: 2 occurrences

- Line 762: struct import `OptimizerError` is private: private struct import
- Line 763: struct import `OptimizerError` is private: private struct import

### error[E0061]: this function takes 0 arguments but 1 argument was supplied

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\optimizer\rule_registry.rs`: 1 occurrences

- Line 125: this function takes 0 arguments but 1 argument was supplied

#### `src\query\optimizer\plan\node.rs`: 1 occurrences

- Line 589: this function takes 4 arguments but 2 arguments were supplied

### error[E0502]: cannot borrow `*self` as mutable because it is also borrowed as immutable: mutable borrow occurs here

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\engine\optimizer.rs`: 2 occurrences

- Line 472: cannot borrow `*self` as mutable because it is also borrowed as immutable: mutable borrow occurs here
- Line 568: cannot borrow `*ctx` as mutable because it is also borrowed as immutable: mutable borrow occurs here

### error[E0277]: the trait bound `OptimizationPhase: Default` is not satisfied: the trait `Default` is not implemented for `OptimizationPhase`

**Total Occurrences**: 2  
**Unique Files**: 2

#### `src\query\optimizer\core\config.rs`: 1 occurrences

- Line 82: the trait bound `OptimizationPhase: Default` is not satisfied: the trait `Default` is not implemented for `OptimizationPhase`

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 533: can't compare `&str` with `str`: no implementation for `&str == str`

### error[E0505]: cannot move out of `new_node` because it is borrowed: move out of `new_node` occurs here

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\engine\optimizer.rs`: 2 occurrences

- Line 615: cannot move out of `new_node` because it is borrowed: move out of `new_node` occurs here
- Line 619: cannot move out of `new_node` because it is borrowed: move out of `new_node` occurs here

### error[E0433]: failed to resolve: could not find `project` in `nodes`: could not find `project` in `nodes`

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\query\optimizer\plan\node.rs`: 2 occurrences

- Line 610: failed to resolve: could not find `project` in `nodes`: could not find `project` in `nodes`
- Line 613: failed to resolve: could not find `filter` in `nodes`: could not find `filter` in `nodes`

### error[E0432]: unresolved import `super::node`: could not find `node` in `super`

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\plan_validator.rs`: 1 occurrences

- Line 7: unresolved import `super::node`: could not find `node` in `super`

### error[E0506]: cannot assign to `ctx.stats.rules_applied` because it is borrowed: `ctx.stats.rules_applied` is assigned to here but it was already borrowed

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 571: cannot assign to `ctx.stats.rules_applied` because it is borrowed: `ctx.stats.rules_applied` is assigned to here but it was already borrowed

### error[E0596]: cannot borrow `group_mut.explored_rules` as mutable, as it is behind a `&` reference: `group_mut` is a `&` reference, so the data it refers to cannot be borrowed as mutable

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\query\optimizer\engine\optimizer.rs`: 1 occurrences

- Line 576: cannot borrow `group_mut.explored_rules` as mutable, as it is behind a `&` reference: `group_mut` is a `&` reference, so the data it refers to cannot be borrowed as mutable

