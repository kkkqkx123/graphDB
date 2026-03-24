# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 106
- **Total Warnings**: 1
- **Total Issues**: 107
- **Unique Error Patterns**: 30
- **Unique Warning Patterns**: 1
- **Files with Issues**: 33

## Error Statistics

**Total Errors**: 106

### Error Type Breakdown

- **error[E0433]**: 100 errors
- **error[E0583]**: 6 errors

### Files with Errors (Top 10)

- `src\query\optimizer\cost\node_estimators\sort_limit.rs`: 19 errors
- `src\query\optimizer\cost\node_estimators\graph_traversal.rs`: 16 errors
- `src\query\optimizer\cost\node_estimators\join.rs`: 13 errors
- `src\query\optimizer\cost\node_estimators\scan.rs`: 8 errors
- `src\query\optimizer\cost\node_estimators\control_flow.rs`: 7 errors
- `src\query\optimizer\cost\node_estimators\data_processing.rs`: 7 errors
- `src\query\parser\mod.rs`: 2 errors
- `src\query\executor\factory\builders\traversal_builder.rs`: 2 errors
- `src\query\optimizer\cost\child_accessor.rs`: 2 errors
- `src\query\executor\factory\builders\join_builder.rs`: 2 errors

## Warning Statistics

**Total Warnings**: 1

### Warning Type Breakdown

- **warning**: 1 warnings

### Files with Warnings (Top 10)

- `src\api\embedded\mod.rs`: 1 warnings

## Detailed Error Categorization

### error[E0433]: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

**Total Occurrences**: 100  
**Unique Files**: 28

#### `src\query\optimizer\cost\node_estimators\sort_limit.rs`: 19 occurrences

- Line 142: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 143: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 144: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- ... 16 more occurrences in this file

#### `src\query\optimizer\cost\node_estimators\graph_traversal.rs`: 16 occurrences

- Line 230: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 231: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 234: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- ... 13 more occurrences in this file

#### `src\query\optimizer\cost\node_estimators\join.rs`: 13 occurrences

- Line 95: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 96: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 116: failed to resolve: use of undeclared type `HashInnerJoinNode`: use of undeclared type `HashInnerJoinNode`
- ... 10 more occurrences in this file

#### `src\query\optimizer\cost\node_estimators\scan.rs`: 8 occurrences

- Line 13: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 14: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 165: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- ... 5 more occurrences in this file

#### `src\query\optimizer\cost\node_estimators\control_flow.rs`: 7 occurrences

- Line 15: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 118: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 119: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- ... 4 more occurrences in this file

#### `src\query\optimizer\cost\node_estimators\data_processing.rs`: 7 occurrences

- Line 19: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 143: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 144: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- ... 4 more occurrences in this file

#### `src\query\executor\mod.rs`: 2 occurrences

- Line 98: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 114: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\executor\factory\builders\data_access_builder.rs`: 2 occurrences

- Line 13: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 14: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\optimizer\cost\node_estimators\graph_algorithm.rs`: 2 occurrences

- Line 77: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 78: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\executor\factory\builders\join_builder.rs`: 2 occurrences

- Line 11: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 12: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\executor\factory\builders\traversal_builder.rs`: 2 occurrences

- Line 14: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 17: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\optimizer\cost\child_accessor.rs`: 2 occurrences

- Line 5: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 193: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\optimizer\analysis\fingerprint.rs`: 2 occurrences

- Line 15: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 85: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\optimizer\strategy\subquery_unnesting.rs`: 2 occurrences

- Line 36: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`
- Line 37: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\executor\factory\executor_factory.rs`: 1 occurrences

- Line 11: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\executor\factory\parsers\vertex_parser.rs`: 1 occurrences

- Line 6: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\executor\factory\builders\transformation_builder.rs`: 1 occurrences

- Line 13: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\optimizer\analysis\integration_test.rs`: 1 occurrences

- Line 13: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\executor\factory\builders\data_processing_builder.rs`: 1 occurrences

- Line 13: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\executor\factory\validators\plan_validator.rs`: 1 occurrences

- Line 6: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\executor\factory\builders\admin_builder.rs`: 1 occurrences

- Line 25: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\executor\factory\builders\data_modification_builder.rs`: 1 occurrences

- Line 11: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\optimizer\analysis\reference_count.rs`: 1 occurrences

- Line 7: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\optimizer\strategy\topn_optimization.rs`: 1 occurrences

- Line 32: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\validator\strategies\expression_strategy_test.rs`: 1 occurrences

- Line 6: failed to resolve: could not find `utils` in `expression`: could not find `utils` in `expression`

#### `src\query\executor\factory\builders\control_flow_builder.rs`: 1 occurrences

- Line 12: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\executor\factory\builders\set_operation_builder.rs`: 1 occurrences

- Line 11: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

#### `src\query\optimizer\strategy\materialization.rs`: 1 occurrences

- Line 34: failed to resolve: could not find `plan` in `planner`: could not find `plan` in `planner`

### error[E0583]: file not found for module `expression`

**Total Occurrences**: 6  
**Unique Files**: 5

#### `src\query\parser\mod.rs`: 2 occurrences

- Line 8: file not found for module `lexer`
- Line 9: file not found for module `parser`

#### `src\core\types\mod.rs`: 1 occurrences

- Line 5: file not found for module `expression`

#### `src\query\mod.rs`: 1 occurrences

- Line 16: file not found for module `planner`

#### `src\api\embedded\mod.rs`: 1 occurrences

- Line 40: file not found for module `statement`

#### `src\core\mod.rs`: 1 occurrences

- Line 3: file not found for module `result`

## Detailed Warning Categorization

### warning: file is loaded as a module multiple times: `src\lib.rs`: first loaded here

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\api\embedded\mod.rs`: 1 occurrences

- Line 40: file is loaded as a module multiple times: `src\lib.rs`: first loaded here

