# Issue: GO Traversal Undefined Variable Error

## Problem Description

GO traversal queries fail with "UndefinedVariable" error when trying to reference edge properties in the YIELD clause. The error indicates that the edge variable is not properly defined in the query execution context.

## Affected Tests

- `test_social_network.TestSocialNetworkQueries.test_014_go_traversal`
- `test_social_network.TestSocialNetworkQueries.test_015_go_multiple_steps`

## Error Messages

```
AssertionError: False is not true : GO traversal failed: Query execution failed: Query error: Execution error: Execution error: Expression error: FunctionError: Failed to evaluate projection expression '': UndefinedVariable: Undefined variable: friend

AssertionError: False is not true : GO multi-step failed: Query execution failed: Query error: Execution error: Execution error: Expression error: FunctionError: Failed to evaluate projection expression '': UndefinedVariable: Undefined variable: friend
```

## Root Cause Analysis

The error suggests that when using GO traversal, the edge type name (`friend`) is being treated as a variable in the YIELD clause, but it's not properly bound in the execution context. Possible causes:

1. **Edge Variable Binding**: The GO traversal executor may not be correctly binding edge types to variables
2. **YIELD Clause Processing**: The YIELD clause may not correctly resolve edge properties
3. **Query Plan Generation**: The query plan for GO traversal may not properly set up variable bindings
4. **Syntax Interpretation**: There may be ambiguity in how `friend.name` should be interpreted (as edge property vs. vertex property)

## Verification Method

Run the following test to verify the issue:

```bash
cd tests/e2e
python -m pytest test_social_network.py::TestSocialNetworkQueries::test_014_go_traversal -v
python -m pytest test_social_network.py::TestSocialNetworkQueries::test_015_go_multiple_steps -v
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

### Query 1: GO 1 Step
```sql
GO 1 STEP FROM "p1" OVER friend YIELD friend.name
```
Error: `UndefinedVariable: Undefined variable: friend`

### Query 2: GO 2 Steps
```sql
GO 2 STEPS FROM "p1" OVER friend YIELD friend.name
```
Error: `UndefinedVariable: Undefined variable: friend`

## Expected Behavior

GO traversal queries should correctly resolve edge properties in the YIELD clause. The syntax `friend.name` should refer to the property of the edge being traversed.

## Actual Behavior

The edge type name `friend` is treated as an undefined variable in the YIELD clause.

## Alternative Syntax to Consider

Depending on the intended design, the correct syntax might be:

1. **Current syntax (if supported)**: `GO 1 STEP FROM "p1" OVER friend YIELD friend.name`
2. **Alternative syntax**: `GO 1 STEP FROM "p1" OVER friend YIELD $$[friend].name`
3. **Another alternative**: `GO 1 STEP FROM "p1" OVER friend YIELD edge.name`

## Related Code

The test code is located in:
- `tests/e2e/test_social_network.py`, lines 310-330 (TestSocialNetworkQueries class)

## Suggested Fixes

1. **Review GO Query Syntax**: Clarify the correct syntax for referencing edge properties in GO queries
2. **Fix Variable Binding**: Ensure edge types are properly bound in GO traversal execution
3. **Update YIELD Processing**: Fix the YIELD clause to correctly resolve edge properties
4. **Documentation**: Document the correct syntax for GO traversal queries

## Priority

High - GO traversal is a fundamental graph query pattern

## Related Components

- `src/query/` - Query parser and planner
- `src/query/executor/` - Query execution engine
- `src/query/expression/` - Expression evaluation
