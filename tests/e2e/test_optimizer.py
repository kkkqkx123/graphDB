#!/usr/bin/env python3
"""
E2E Test Suite for Query Optimizer

Tests optimizer behavior including:
- Index selection
- Join algorithm selection
- Aggregation strategies
- TopN optimization
- Query plan validation via EXPLAIN
"""

import unittest
import time
import json
from typing import Dict, Any
from graphdb_client import GraphDBClient


class TestOptimizerIndex(unittest.TestCase):
    """Index selection optimization tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_optimizer"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        # Cleanup
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")

        # Create space and schema
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")

        cls.client.execute("""
            CREATE TAG person(
                name: STRING,
                age: INT,
                city: STRING,
                salary: INT
            )
        """)

        # Create indexes
        cls.client.execute("CREATE TAG INDEX idx_person_name ON person(name)")
        cls.client.execute("CREATE TAG INDEX idx_person_age ON person(age)")
        time.sleep(1)

        # Insert test data
        for i in range(100):
            name = f"Person_{i:03d}"
            age = 20 + (i % 40)
            city = ["Beijing", "Shanghai", "Shenzhen"][i % 3]
            salary = 5000 + (i * 100)

            cls.client.execute(f'''
                INSERT VERTEX person(name, age, city, salary) VALUES "p{i:03d}":
                    ("{name}", {age}, "{city}", {salary})
            ''')

        time.sleep(1)

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_idx_001_index_scan_for_equality(self):
        """TC-IDX-001: Equality query should use IndexScan."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            EXPLAIN MATCH (p:person {name: "Person_001"}) RETURN p.age
        ''')
        self.assertTrue(result.success)
        # Plan should contain IndexScan
        plan = json.dumps(result.data) if result.data else ""
        self.assertIn("IndexScan", plan or "")

    def test_idx_002_index_scan_for_range(self):
        """TC-IDX-002: Range query should use IndexScan."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            EXPLAIN MATCH (p:person) WHERE p.age > 25 AND p.age < 35 RETURN p.name
        ''')
        self.assertTrue(result.success)
        plan = json.dumps(result.data) if result.data else ""
        self.assertIn("IndexScan", plan or "")

    def test_idx_003_no_index_full_scan(self):
        """TC-IDX-003: Query on non-indexed field should use SeqScan."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            EXPLAIN MATCH (p:person) WHERE p.salary > 10000 RETURN p.name
        ''')
        self.assertTrue(result.success)
        plan = json.dumps(result.data) if result.data else ""
        # Should use sequential scan since salary has no index
        self.assertIn("Scan", plan or "")


class TestOptimizerJoin(unittest.TestCase):
    """Join optimization tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_optimizer_join"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")

        # Create schema
        cls.client.execute("""
            CREATE TAG company(name: STRING, industry: STRING)
        """)
        cls.client.execute("""
            CREATE TAG employee(name: STRING, salary: INT)
        """)
        cls.client.execute("""
            CREATE EDGE works_at(position: STRING)
        """)
        time.sleep(1)

        # Insert companies (fewer)
        for i in range(10):
            cls.client.execute(f'''
                INSERT VERTEX company(name, industry) VALUES "c{i:02d}":
                    ("Company_{i:02d}", "Tech")
            ''')

        # Insert employees (more)
        for i in range(100):
            cls.client.execute(f'''
                INSERT VERTEX employee(name, salary) VALUES "e{i:03d}":
                    ("Employee_{i:03d}", {5000 + i * 100})
            ''')

        # Create relationships
        for i in range(100):
            company_id = f"c{i % 10:02d}"
            cls.client.execute(f'''
                INSERT EDGE works_at(position) VALUES "e{i:03d}" -> "{company_id}" @0:
                    ("Engineer")
            ''')

        time.sleep(1)

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_join_001_join_algorithm_selection(self):
        """TC-JOIN-001: Verify join algorithm is selected."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            EXPLAIN
            MATCH (e:employee)-[:works_at]->(c:company)
            RETURN e.name, c.name
        ''')
        self.assertTrue(result.success)
        plan = json.dumps(result.data) if result.data else ""
        # Should contain a join node
        self.assertTrue(
            "HashJoin" in plan or "IndexJoin" in plan or "NestedLoop" in plan,
            f"Expected join in plan: {plan}"
        )


class TestOptimizerAggregate(unittest.TestCase):
    """Aggregation optimization tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_optimizer_agg"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")

        cls.client.execute("""
            CREATE TAG sales(product: STRING, amount: INT, category: STRING)
        """)
        time.sleep(1)

        # Insert sales data
        for i in range(1000):
            product = f"Product_{i % 20:02d}"
            amount = random.randint(10, 1000)
            category = ["A", "B", "C"][i % 3]

            cls.client.execute(f'''
                INSERT VERTEX sales(product, amount, category) VALUES "s{i:04d}":
                    ("{product}", {amount}, "{category}")
            ''')

        time.sleep(1)

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_agg_001_hash_aggregate(self):
        """TC-AGG-001: HashAggregate for GROUP BY."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            EXPLAIN
            MATCH (s:sales)
            RETURN s.category, sum(s.amount) AS total
            GROUP BY s.category
        ''')
        self.assertTrue(result.success)
        plan = json.dumps(result.data) if result.data else ""
        self.assertIn("Aggregate", plan or "")


class TestOptimizerTopN(unittest.TestCase):
    """TopN optimization tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_optimizer_topn"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {self.space_name}")

        cls.client.execute("""
            CREATE TAG product(name: STRING, price: INT, sales: INT)
        """)
        time.sleep(1)

        for i in range(100):
            cls.client.execute(f'''
                INSERT VERTEX product(name, price, sales) VALUES "p{i:03d}":
                    ("Product_{i:03d}", {random.randint(10, 1000)}, {random.randint(0, 10000)})
            ''')

        time.sleep(1)

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_topn_001_order_by_limit(self):
        """TC-TOPN-001: ORDER BY + LIMIT should use TopN."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            EXPLAIN
            MATCH (p:product)
            RETURN p.name, p.price
            ORDER BY p.price DESC
            LIMIT 10
        ''')
        self.assertTrue(result.success)
        plan = json.dumps(result.data) if result.data else ""
        # Should use TopN optimization
        self.assertIn("TopN", plan or "")


class TestOptimizerExplainFormat(unittest.TestCase):
    """EXPLAIN format tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_optimizer_explain"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")
        cls.client.execute("CREATE TAG person(name: STRING, age: INT)")
        time.sleep(1)

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_explain_001_text_format(self):
        """TC-EXPLAIN-001: EXPLAIN with text format."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            EXPLAIN MATCH (p:person) RETURN p.name
        ''')
        self.assertTrue(result.success)
        self.assertIsNotNone(result.data)

    def test_explain_002_dot_format(self):
        """TC-EXPLAIN-002: EXPLAIN with DOT format."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            EXPLAIN FORMAT = DOT MATCH (p:person) RETURN p.name
        ''')
        self.assertTrue(result.success)
        # DOT format should contain digraph
        plan = str(result.data) if result.data else ""
        self.assertIn("digraph", plan or "")


class TestOptimizerProfile(unittest.TestCase):
    """PROFILE command tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_optimizer_profile"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")
        cls.client.execute("CREATE TAG person(name: STRING, age: INT)")
        time.sleep(1)

        for i in range(50):
            cls.client.execute(f'''
                INSERT VERTEX person(name, age) VALUES "p{i:03d}":
                    ("Person_{i:03d}", {20 + i})
            ''')

        time.sleep(1)

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_profile_001_basic_profile(self):
        """TC-PROFILE-001: Basic PROFILE execution."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            PROFILE MATCH (p:person) RETURN count(p)
        ''')
        self.assertTrue(result.success)
        # Profile should include execution statistics
        self.assertIsNotNone(result.data)


class TestOptimizerCleanup(unittest.TestCase):
    """Cleanup tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_999_cleanup(self):
        """Cleanup: Drop all test spaces."""
        spaces = [
            "e2e_optimizer",
            "e2e_optimizer_join",
            "e2e_optimizer_agg",
            "e2e_optimizer_topn",
            "e2e_optimizer_explain",
            "e2e_optimizer_profile"
        ]
        for space in spaces:
            result = self.client.execute(f"DROP SPACE IF EXISTS {space}")
            self.assertTrue(result.success or "not exist" in str(result.error).lower())


import random


def run_tests():
    """Run all optimizer tests."""
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()

    suite.addTests(loader.loadTestsFromTestCase(TestOptimizerIndex))
    suite.addTests(loader.loadTestsFromTestCase(TestOptimizerJoin))
    suite.addTests(loader.loadTestsFromTestCase(TestOptimizerAggregate))
    suite.addTests(loader.loadTestsFromTestCase(TestOptimizerTopN))
    suite.addTests(loader.loadTestsFromTestCase(TestOptimizerExplainFormat))
    suite.addTests(loader.loadTestsFromTestCase(TestOptimizerProfile))
    suite.addTests(loader.loadTestsFromTestCase(TestOptimizerCleanup))

    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)

    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_tests()
    exit(0 if success else 1)
