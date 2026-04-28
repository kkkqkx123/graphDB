# Issue: EXPLAIN with Index - Schema Manager Not Available

## Problem Description

When executing EXPLAIN with LOOKUP query that uses an index, the query fails with "Schema manager not available" error. This suggests that the EXPLAIN/LOOKUP execution path is not properly initializing or accessing the schema manager.

## Affected Tests

- `test_social_network.TestSocialNetworkExplain.test_018_explain_with_index`

## Error Messages

```
AssertionError: False is not true : EXPLAIN with index failed: Query execution failed: Query error: Invalid query: Semantic error: Schema manager not available
```

## Root Cause Analysis

The error indicates that when executing an EXPLAIN query with LOOKUP, the semantic analyzer cannot access the schema manager. Possible causes:

1. **Schema Manager Initialization**: The schema manager may not be properly initialized in the EXPLAIN query execution path
2. **Context Propagation**: The query execution context may not be properly propagating the schema manager to the semantic analyzer
3. **LOOKUP Query Processing**: The LOOKUP query type may have a different code path that doesn't initialize the schema manager
4. **Session State**: The session state may not be properly set up when executing EXPLAIN queries

## Verification Method

Run the following test to verify the issue:

```bash
cd tests/e2e
python -m pytest test_social_network.py::TestSocialNetworkExplain::test_018_explain_with_index -v
```

Or run the full test suite:

```bash
python run_tests.py
```

## Test Data Setup

The test uses the following schema and data:

```sql
CREATE SPACE e2e_social_network_explain (vid_type=STRING)
USE e2e_social_network_explain
CREATE TAG person(name: STRING, age: INT)
CREATE EDGE friend(degree: FLOAT)
CREATE TAG INDEX idx_person_name ON person(name)

INSERT VERTEX person(name, age) VALUES
    "p1": ("Alice", 30),
    "p2": ("Bob", 25)
```

## Failing Query

```sql
EXPLAIN LOOKUP ON person WHERE person.name == "Alice"
```

Error: `Semantic error: Schema manager not available`

## Expected Behavior

EXPLAIN with LOOKUP should generate a query execution plan that shows the index scan operation.

## Actual Behavior

The query fails during semantic analysis because the schema manager is not available.

## Working Query (for comparison)

The following EXPLAIN query works correctly:

```sql
EXPLAIN MATCH (p:person) RETURN p.name
```

This suggests the issue is specific to the LOOKUP query type or the combination of EXPLAIN + LOOKUP.

## Related Code

The test code is located in:
- `tests/e2e/test_social_network.py`, lines 375-385 (TestSocialNetworkExplain class)

Example test case:

```python
def test_018_explain_with_index(self):
    """TC-018: EXPLAIN with index scan."""
    self.client.execute(f"USE {self.space_name}")

    result = self.client.execute('''
        EXPLAIN LOOKUP ON person WHERE person.name == "Alice"
    ''')
    self.assertTrue(result.success, f"EXPLAIN with index failed: {result.error}")
```

## Suggested Fixes

1. **Review EXPLAIN + LOOKUP Code Path**: Check how EXPLAIN handles LOOKUP queries differently from other query types
2. **Initialize Schema Manager**: Ensure the schema manager is properly initialized in the EXPLAIN/LOOKUP execution path
3. **Context Setup**: Verify that the query execution context is properly set up with schema manager access
4. **Session Validation**: Check if the session is properly authenticated and has access to the schema manager

## Priority

Medium - EXPLAIN is useful for query optimization but not a core functionality

## Related Components

- `src/query/` - Query parser and planner
- `src/query/executor/` - Query execution engine
- `src/query/semantic/` - Semantic analyzer
- `src/schema/` - Schema manager
