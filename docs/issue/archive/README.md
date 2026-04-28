# GraphDB Issues Documentation

This directory contains documentation of known issues in the GraphDB project, including problem descriptions, root cause analysis, and verification methods.

## Issue Categories

### 1. Transaction Issues

| Issue | Priority | Description |
|-------|----------|-------------|
| [Transaction Timeout](transaction_timeout.md) | High | Transaction commands timeout after 30 seconds |

### 2. Query Execution Issues

| Issue | Priority | Description |
|-------|----------|-------------|
| [MATCH Query Undefined Variable](match_query_undefined_variable.md) | High | MATCH queries fail with undefined variable error |
| [GO Traversal Undefined Variable](go_traversal_undefined_variable.md) | High | GO traversal queries fail with undefined variable error |
| [EXPLAIN Index Schema Manager](explain_index_schema_manager.md) | Medium | EXPLAIN with LOOKUP fails due to schema manager not available |

### 3. Query Optimization Issues

| Issue | Priority | Description |
|-------|----------|-------------|
| [Optimizer Missing Operators](optimizer_tests_missing_operators.md) | Medium | EXPLAIN output missing expected operators |
| [Optimizer TopN Setup Error](optimizer_topn_setup_error.md) | Low | Test setup error using `self` instead of `cls` |

### 4. Extended Types Issues

| Issue | Priority | Description |
|-------|----------|-------------|
| [Extended Types Not Implemented](extended_types_not_implemented.md) | Medium | Geography, Vector, and FullText features not fully implemented |

## Verification Methods

Each issue document includes verification methods using the integration tests in the `tests/e2e/` directory.

### Running Specific Tests

```bash
cd tests/e2e

# Test a specific issue
python -m pytest test_social_network.py::TestSocialNetworkTransaction -v
python -m pytest test_social_network.py::TestSocialNetworkQueries::test_011_match_basic -v
python -m pytest test_optimizer.py::TestOptimizerIndex -v
```

### Running Full Test Suite

```bash
cd tests/e2e
python run_tests.py
```

## Test Results Summary

Based on the latest test run:

| Test Suite | Total | Passed | Failed | Errors | Skipped |
|------------|-------|--------|--------|--------|---------|
| Schema Manager Init | 11 | 11 | 0 | 0 | 0 |
| Social Network | 22 | 14 | 8 | 0 | 0 |
| Optimizer | 9 | 3 | 5 | 1 | 0 |
| Extended Types | 14 | 2 | 12 | 0 | 0 |

## Priority Levels

- **High**: Core functionality issues that block basic operations
- **Medium**: Important features that enhance usability
- **Low**: Minor issues or test code bugs

## Related Components

- `src/query/` - Query parser, planner, and executor
- `src/transaction/` - Transaction management
- `src/schema/` - Schema manager
- `src/index/` - Index system
- `src/storage/` - Storage engine
