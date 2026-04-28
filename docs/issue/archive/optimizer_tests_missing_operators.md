# Issue: Optimizer Tests - Missing Query Plan Operators

## Problem Description

The optimizer tests expect specific operators (IndexScan, SeqScan, HashJoin, Aggregate, etc.) to appear in the query execution plan, but the actual plans only show a "Limit" operator. This suggests that either:
1. The query optimizer is not generating the expected plans
2. The EXPLAIN output format has changed
3. The queries are being executed differently than expected

## Affected Tests

- `test_optimizer.TestOptimizerIndex.test_idx_001_index_scan_for_equality`
- `test_optimizer.TestOptimizerIndex.test_idx_002_index_scan_for_range`
- `test_optimizer.TestOptimizerIndex.test_idx_003_no_index_full_scan`
- `test_optimizer.TestOptimizerJoin.test_join_001_join_algorithm_selection`
- `test_optimizer.TestOptimizerAggregate.test_agg_001_hash_aggregate`

## Error Messages

```
AssertionError: 'IndexScan' not found in '{"columns": ["plan"], "rows": [{"plan": "...| 4036 | Limit | 4037 |..."}], "row_count": 1}'

AssertionError: 'Scan' not found in '{"columns": ["plan"], "rows": [{"plan": "...| 4310 | Limit | 4311 |..."}], "row_count": 1}'

AssertionError: False is not true : Expected join in plan: {"columns": ["plan"], "rows": [{"plan": "...| 5754 | Limit | 5755 |..."}], "row_count": 1}

AssertionError: 'Aggregate' not found in '{"columns": ["plan"], "rows": [{"plan": "...| 13061 | Limit | 13062 |..."}], "row_count": 1}'
```

## Root Cause Analysis

The EXPLAIN output only shows a "Limit" operator, which suggests:

1. **Query Simplification**: The optimizer may be simplifying queries to just a LIMIT operation
2. **Plan Generation Issue**: The query plan generator may not be creating the expected operator tree
3. **EXPLAIN Format Change**: The EXPLAIN output format may have changed, hiding the actual operators
4. **Query Execution Short-circuit**: The queries may be short-circuited before reaching the expected operators

## Verification Method

Run the following tests to verify the issue:

```bash
cd tests/e2e
python -m pytest test_optimizer.py::TestOptimizerIndex -v
python -m pytest test_optimizer.py::TestOptimizerJoin -v
python -m pytest test_optimizer.py::TestOptimizerAggregate -v
```

Or run the full test suite:

```bash
python run_tests.py
```

## Test Data Setup

The tests use various schemas. Example from TestOptimizerIndex:

```sql
CREATE SPACE optimizer_index_test (vid_type=STRING)
USE optimizer_index_test
CREATE TAG user(name: STRING, age: INT, city: STRING)
CREATE TAG INDEX idx_user_name ON user(name)
CREATE TAG INDEX idx_user_age ON user(age)

INSERT VERTEX user(name, age, city) VALUES
    "u1": ("Alice", 25, "Beijing"),
    "u2": ("Bob", 30, "Shanghai"),
    "u3": ("Charlie", 35, "Beijing")
```

## Failing Queries and Expected Plans

### Test 1: Index Scan for Equality
```sql
EXPLAIN MATCH (u:user) WHERE u.name == "Alice" RETURN u.age
```
Expected: Plan should contain "IndexScan"
Actual: Plan only contains "Limit"

### Test 2: Index Scan for Range
```sql
EXPLAIN MATCH (u:user) WHERE u.age > 25 RETURN u.name
```
Expected: Plan should contain "IndexScan"
Actual: Plan only contains "Limit"

### Test 3: Full Scan (No Index)
```sql
EXPLAIN MATCH (u:user) WHERE u.city == "Beijing" RETURN u.name
```
Expected: Plan should contain "SeqScan" or similar scan operator
Actual: Plan only contains "Limit"

### Test 4: Join Algorithm Selection
```sql
EXPLAIN MATCH (u1:user)-[:knows]->(u2:user) RETURN u1.name, u2.name
```
Expected: Plan should contain "HashJoin", "IndexJoin", or "NestedLoop"
Actual: Plan only contains "Limit"

### Test 5: Hash Aggregate
```sql
EXPLAIN MATCH (u:user) RETURN u.city, count(*) AS cnt
```
Expected: Plan should contain "Aggregate" or "HashAggregate"
Actual: Plan only contains "Limit"

## Sample EXPLAIN Output

Current output format:
```
| id   | name  | deps | profiling_data | operator_info        | output_var |
----------------------------------------------------------------------------
| 4036 | Limit | 4037 | -              | count:10000,offset:0 | -          |
----------------------------------------------------------------------------
```

Expected output should include additional operators like:
- IndexScan
- SeqScan
- HashJoin
- Aggregate
- Filter
- Project

## Related Code

The test code is located in:
- `tests/e2e/test_optimizer.py`

Example test case:

```python
def test_idx_001_index_scan_for_equality(self):
    """TC-IDX-001: Equality query should use IndexScan."""
    self.client.execute(f"USE {self.space_name}")

    result = self.client.execute('''
        EXPLAIN MATCH (u:user) WHERE u.name == "Alice" RETURN u.age
    ''')
    self.assertTrue(result.success)
    plan = json.dumps(result.data)
    self.assertIn("IndexScan", plan or "")
```

## Suggested Fixes

1. **Review Query Plan Generation**: Check if the query planner is correctly generating operator trees
2. **Fix EXPLAIN Output**: Ensure EXPLAIN shows all operators in the plan, not just Limit
3. **Check Index Usage**: Verify that indexes are being used when expected
4. **Verify Operator Implementation**: Ensure operators like IndexScan, HashJoin, Aggregate are implemented
5. **Add Logging**: Add debug logging to trace query plan generation

## Priority

Medium - Query optimization is important for performance but basic queries still work

## Related Components

- `src/query/planner/` - Query planner
- `src/query/optimizer/` - Query optimizer
- `src/query/executor/` - Query execution engine
- `src/index/` - Index system
