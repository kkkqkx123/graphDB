# Issue: MATCH Query Undefined Variable Error

## Problem Description

MATCH queries fail with "UndefinedVariable" error when trying to reference variables defined in the MATCH pattern. This affects both basic MATCH queries and MATCH queries with path patterns.

## Affected Tests

- `test_social_network.TestSocialNetworkQueries.test_011_match_basic`
- `test_social_network.TestSocialNetworkQueries.test_012_match_with_filter`
- `test_social_network.TestSocialNetworkQueries.test_013_match_path`

## Error Messages

```
AssertionError: False is not true : MATCH failed: Query execution failed: Query error: Execution error: Execution error: Expression error: FunctionError: Failed to evaluate projection expression '': UndefinedVariable: Undefined variable: p

AssertionError: False is not true : MATCH with filter failed: Query execution failed: Query error: Execution error: Execution error: Expression error: FunctionError: Failed to evaluate filter condition: UndefinedVariable: Undefined variable: p

AssertionError: False is not true : MATCH path failed: Query execution failed: Query error: Execution error: Execution error: Expression error: FunctionError: Failed to evaluate filter condition: UndefinedVariable: Undefined variable: p
```

## Root Cause Analysis

The error indicates that the query execution engine is not properly binding variables from the MATCH pattern to the execution context. Possible causes:

1. **Variable Binding Issue**: The query planner or executor is not correctly binding pattern variables (like `p` in `(p:person)`) to the execution context
2. **Scope Management**: The variable scope in MATCH queries may not be properly managed during execution
3. **Expression Evaluation**: The expression evaluator may not have access to variables defined in the MATCH pattern
4. **Query Plan Generation**: The query plan may not correctly propagate variable bindings to downstream operators

## Verification Method

Run the following test to verify the issue:

```bash
cd tests/e2e
python -m pytest test_social_network.py::TestSocialNetworkQueries::test_011_match_basic -v
python -m pytest test_social_network.py::TestSocialNetworkQueries::test_012_match_with_filter -v
python -m pytest test_social_network.py::TestSocialNetworkQueries::test_013_match_path -v
```

Or run the full test suite:

```bash
python run_tests.py
```

## Test Data Setup

The test uses the following schema and data:

```sql
CREATE SPACE e2e_social_network_queries (vid_type=STRING)
USE e2e_social_network_queries
CREATE TAG person(name: STRING, age: INT, city: STRING)
CREATE EDGE friend(degree: FLOAT)

INSERT VERTEX person(name, age, city) VALUES
    "p1": ("Alice", 30, "Beijing"),
    "p2": ("Bob", 25, "Shanghai"),
    "p3": ("Charlie", 35, "Beijing"),
    "p4": ("David", 28, "Shenzhen")

INSERT EDGE friend(degree) VALUES
    "p1" -> "p2": (0.8),
    "p2" -> "p3": (0.7),
    "p1" -> "p3": (0.9)
```

## Failing Queries

### Query 1: Basic MATCH
```sql
MATCH (p:person) RETURN p.name, p.age
```
Error: `UndefinedVariable: Undefined variable: p`

### Query 2: MATCH with Filter
```sql
MATCH (p:person) WHERE p.age > 28 RETURN p.name
```
Error: `UndefinedVariable: Undefined variable: p`

### Query 3: MATCH Path
```sql
MATCH (p:person)-[:friend]->(f:person) RETURN p.name, f.name
```
Error: `UndefinedVariable: Undefined variable: p`

## Expected Behavior

MATCH queries should correctly bind pattern variables and allow them to be referenced in WHERE clauses and RETURN statements.

## Actual Behavior

Variables defined in MATCH patterns are not accessible in the query execution context, causing "UndefinedVariable" errors.

## Related Code

The test code is located in:
- `tests/e2e/test_social_network.py`, lines 270-320 (TestSocialNetworkQueries class)

## Suggested Fixes

1. **Review Variable Binding Logic**: Check how variables are bound in the MATCH query planner
2. **Fix Scope Propagation**: Ensure variable bindings are properly propagated through the query plan
3. **Expression Context**: Verify that the expression evaluator has access to all bound variables
4. **Add Unit Tests**: Add unit tests for variable binding in MATCH queries

## Priority

High - MATCH is a fundamental query pattern in graph databases

## Related Components

- `src/query/` - Query parser and planner
- `src/query/executor/` - Query execution engine
- `src/query/expression/` - Expression evaluation
