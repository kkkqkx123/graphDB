# Issue: Optimizer TopN Test - Setup Error

## Problem Description

The TestOptimizerTopN test class fails during setUpClass with a NameError: 'self' is not defined. This is a coding error in the test setup method where `self` is being used inside a class method instead of `cls`.

## Affected Tests

- `test_optimizer.TestOptimizerTopN` (entire test class)

## Error Messages

```
ERROR: setUpClass (test_optimizer.TestOptimizerTopN)
----------------------------------------------------------------------
Traceback (most recent call last):
  File "D:\项目\database\graphDB\tests\e2e\test_optimizer.py", line 243, in setUpClass
    cls._setup_data()
    ~~~~~~~~~~~~~~~^^
  File "D:\项目\database\graphDB\tests\e2e\test_optimizer.py", line 249, in _setup_data
    cls.client.execute(f"USE {self.space_name}")
                              ^^^^
NameError: name 'self' is not defined
```

## Root Cause Analysis

This is a simple coding error in the test file. In the `_setup_data` class method, `self` is being used instead of `cls`. Class methods (decorated with `@classmethod`) should use `cls` to refer to the class, not `self`.

The problematic code:
```python
@classmethod
def _setup_data(cls):
    cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
    cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
    cls.client.execute(f"USE {self.space_name}")  # BUG: should be cls.space_name
    # ...
```

## Verification Method

Run the following test to verify the issue:

```bash
cd tests/e2e
python -m pytest test_optimizer.py::TestOptimizerTopN -v
```

Or run the full test suite:

```bash
python run_tests.py
```

## Related Code

The test code is located in:
- `tests/e2e/test_optimizer.py`, lines 240-260 (TestOptimizerTopN class)

Problematic code:

```python
class TestOptimizerTopN(unittest.TestCase):
    """TopN optimization tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "optimizer_topn_test"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {self.space_name}")  # BUG HERE
        cls.client.execute("CREATE TAG user(name: STRING, age: INT)")
        time.sleep(1)
        # ...
```

## Fix

The fix is simple - change `self.space_name` to `cls.space_name`:

```python
@classmethod
def _setup_data(cls):
    cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
    cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
    cls.client.execute(f"USE {cls.space_name}")  # FIXED
    cls.client.execute("CREATE TAG user(name: STRING, age: INT)")
    time.sleep(1)
    # ...
```

## Priority

Low - This is a test code bug, not a production code issue

## Related Components

- `tests/e2e/test_optimizer.py` - Test file with the bug
