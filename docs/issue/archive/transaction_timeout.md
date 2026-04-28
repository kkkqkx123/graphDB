# Issue: Transaction Timeout Problem

## Problem Description

The transaction functionality in GraphDB server has a timeout issue. When executing transaction-related commands (BEGIN, INSERT within transaction, COMMIT, ROLLBACK), the server frequently times out after 30 seconds, causing test failures.

## Affected Tests

- `test_social_network.TestSocialNetworkTransaction.test_020_transaction_commit`
- `test_social_network.TestSocialNetworkTransaction.test_021_transaction_rollback`

## Error Messages

```
AssertionError: False is not true : INSERT failed: Request timeout after 30s
AssertionError: False is not true : COMMIT failed: Request timeout after 30s
AssertionError: False is not true : ROLLBACK failed: Request timeout after 30s
```

## Root Cause Analysis

The timeout issue may be caused by:

1. **Transaction Lock Contention**: The server may have lock contention issues when multiple transactions are executed concurrently
2. **Inefficient Transaction State Management**: The transaction state machine may have performance bottlenecks
3. **Blocking Operations**: Some operations within transactions may be blocking for extended periods
4. **Resource Leaks**: Previous test runs may leave transactions in an incomplete state, causing subsequent transactions to hang

## Verification Method

Run the following test to verify the issue:

```bash
cd tests/e2e
python -m pytest test_social_network.py::TestSocialNetworkTransaction -v
```

Or run the full test suite:

```bash
python run_tests.py
```

## Expected Behavior

Transactions should complete within a reasonable time (less than 5 seconds for simple operations like inserting a single vertex).

## Actual Behavior

Transactions frequently timeout after 30 seconds, causing test failures.

## Related Code

The test code is located in:
- `tests/e2e/test_social_network.py`, lines 440-500 (TestSocialNetworkTransaction class)

Example test case:

```python
def test_020_transaction_commit(self):
    """TC-020: Basic transaction commit."""
    self.client.execute(f"USE {self.space_name}")

    # Use unique vertex ID with timestamp to avoid conflicts
    import time
    vertex_id = f"tx1_{int(time.time() * 1000)}"

    # Begin transaction
    result = self.client.execute("BEGIN")
    self.assertTrue(result.success, f"BEGIN failed: {result.error}")

    # Insert data
    result = self.client.execute(f'''
        INSERT VERTEX person(name, age) VALUES "{vertex_id}": ("TX_Test", 20)
    ''')
    self.assertTrue(result.success, f"INSERT failed: {result.error}")

    # Commit
    result = self.client.execute("COMMIT")
    self.assertTrue(result.success, f"COMMIT failed: {result.error}")

    # Verify data exists
    result = self.client.execute(f'FETCH PROP ON person "{vertex_id}"')
    self.assertTrue(result.success, f"FETCH failed: {result.error}")
```

## Suggested Fixes

1. **Review Transaction Locking Mechanism**: Check if there are any deadlocks or lock contention issues
2. **Add Transaction Timeout Configuration**: Allow configurable transaction timeouts
3. **Implement Transaction Cleanup**: Ensure transactions are properly cleaned up after test runs
4. **Optimize Transaction State Machine**: Review and optimize the transaction state transitions
5. **Add Logging**: Add detailed logging to trace transaction execution and identify bottlenecks

## Priority

High - Transaction functionality is a core feature of the database

## Related Components

- `src/transaction/` - Transaction management module
- `src/query/` - Query execution engine
- `src/storage/` - Storage engine
