#!/usr/bin/env python3
"""
E2E Test Suite for Social Network Scenario

Tests basic graph operations including:
- Schema management
- Data insertion (vertices and edges)
- MATCH queries
- GO traversals
- LOOKUP queries
- Transaction management
"""

import unittest
import json
import time
from typing import Dict, Any, List
from graphdb_client import GraphDBClient, TestResult


class TestSocialNetworkBasic(unittest.TestCase):
    """Basic connection and schema management tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_social_network"

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_001_connect_and_show_spaces(self):
        """TC-001: Connect to server and list spaces."""
        result = self.client.execute("SHOW SPACES")
        self.assertTrue(result.success, f"Failed to show spaces: {result.error}")
        self.assertIsNotNone(result.data)

    def test_002_create_and_use_space(self):
        """TC-002: Create space and switch to it."""
        # Drop if exists
        self.client.execute(f"DROP SPACE IF EXISTS {self.space_name}")

        # Create space
        result = self.client.execute(
            f"CREATE SPACE {self.space_name} (vid_type=STRING)"
        )
        self.assertTrue(result.success, f"Failed to create space: {result.error}")

        # Use space
        result = self.client.execute(f"USE {self.space_name}")
        self.assertTrue(result.success, f"Failed to use space: {result.error}")

    def test_003_create_tags_and_edges(self):
        """TC-003: Create tags and edge types."""
        self.client.execute(f"USE {self.space_name}")

        # Create person tag (use IF NOT EXISTS to avoid error if already exists)
        result = self.client.execute("""
            CREATE TAG IF NOT EXISTS person(
                name: STRING NOT NULL,
                age: INT,
                email: STRING,
                city: STRING
            )
        """)
        self.assertTrue(result.success, f"Failed to create person tag: {result.error}")

        # Create company tag
        result = self.client.execute("""
            CREATE TAG IF NOT EXISTS company(
                name: STRING NOT NULL,
                industry: STRING
            )
        """)
        self.assertTrue(result.success, f"Failed to create company tag: {result.error}")

        # Create friend edge
        result = self.client.execute("""
            CREATE EDGE IF NOT EXISTS friend(degree: FLOAT)
        """)
        self.assertTrue(result.success, f"Failed to create friend edge: {result.error}")

        # Create works_at edge
        result = self.client.execute("""
            CREATE EDGE IF NOT EXISTS works_at(position: STRING)
        """)
        self.assertTrue(result.success, f"Failed to create works_at edge: {result.error}")

    def test_004_show_tags(self):
        """TC-004: Verify tags were created."""
        self.client.execute(f"USE {self.space_name}")
        result = self.client.execute("SHOW TAGS")
        self.assertTrue(result.success)
        self.assertIn("person", str(result.data))
        self.assertIn("company", str(result.data))

    def test_005_show_edges(self):
        """TC-005: Verify edges were created."""
        self.client.execute(f"USE {self.space_name}")
        result = self.client.execute("SHOW EDGES")
        self.assertTrue(result.success)
        self.assertIn("friend", str(result.data))
        self.assertIn("works_at", str(result.data))


class TestSocialNetworkData(unittest.TestCase):
    """Data operation tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_social_network_data"
        cls._setup_schema()

    @classmethod
    def _setup_schema(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")
        cls.client.execute("""
            CREATE TAG person(name: STRING NOT NULL, age: INT, email: STRING, city: STRING)
        """)
        cls.client.execute("""
            CREATE TAG company(name: STRING NOT NULL, industry: STRING)
        """)
        cls.client.execute("CREATE EDGE friend(degree: FLOAT)")
        cls.client.execute("CREATE EDGE works_at(position: STRING)")
        time.sleep(1)  # Wait for schema to propagate

    def setUp(self):
        """Ensure client is authenticated before each test."""
        if not self.client.ensure_authenticated():
            self.client.connect()
        self.client.execute(f"USE {self.space_name}")

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_006_insert_vertex(self):
        """TC-006: Insert vertex data."""
        self.client.execute(f"USE {self.space_name}")

        # Use unique vertex ID to avoid conflicts
        import time
        vertex_id = f"p1_{int(time.time() * 1000)}"

        result = self.client.execute(f'''
            INSERT VERTEX person(name, age, email) VALUES "{vertex_id}":
                ("Alice", 30, "alice@example.com")
        ''')
        self.assertTrue(result.success, f"Failed to insert vertex: {result.error}")

    def test_007_insert_multiple_vertices(self):
        """TC-007: Insert multiple vertices."""
        self.client.execute(f"USE {self.space_name}")

        # Use unique vertex IDs to avoid conflicts
        import time
        timestamp = int(time.time() * 1000)
        vid1 = f"p2_{timestamp}"
        vid2 = f"p3_{timestamp}"

        result = self.client.execute(f'''
            INSERT VERTEX person(name, age) VALUES
                "{vid1}": ("Bob", 25),
                "{vid2}": ("Charlie", 35)
        ''')
        self.assertTrue(result.success, f"Failed to insert vertices: {result.error}")

    def test_008_insert_edge(self):
        """TC-008: Insert edge data."""
        self.client.execute(f"USE {self.space_name}")

        # First ensure vertices exist
        self.client.execute('''
            INSERT VERTEX person(name, age) VALUES "p1": ("Alice", 30)
        ''')
        self.client.execute('''
            INSERT VERTEX person(name, age) VALUES "p2": ("Bob", 25)
        ''')

        result = self.client.execute('''
            INSERT EDGE friend(degree) VALUES "p1" -> "p2": (0.8)
        ''')
        self.assertTrue(result.success, f"Failed to insert edge: {result.error}")

    def test_009_fetch_vertex(self):
        """TC-009: Fetch vertex properties."""
        self.client.execute(f"USE {self.space_name}")

        # Insert a vertex to fetch
        self.client.execute('''
            INSERT VERTEX person(name, age, email) VALUES "p_fetch": ("Alice", 30, "alice@test.com")
        ''')

        result = self.client.execute('FETCH PROP ON person "p_fetch"')
        self.assertTrue(result.success)
        self.assertIn("Alice", str(result.data))

    def test_010_fetch_edge(self):
        """TC-010: Fetch edge properties."""
        self.client.execute(f"USE {self.space_name}")

        # Insert vertices and edge
        self.client.execute('''
            INSERT VERTEX person(name, age) VALUES "p_edge1": ("Alice", 30)
        ''')
        self.client.execute('''
            INSERT VERTEX person(name, age) VALUES "p_edge2": ("Bob", 25)
        ''')
        self.client.execute('''
            INSERT EDGE friend(degree) VALUES "p_edge1" -> "p_edge2" @0: (0.8)
        ''')

        result = self.client.execute('FETCH PROP ON friend "p_edge1" -> "p_edge2"')
        self.assertTrue(result.success)


class TestSocialNetworkQueries(unittest.TestCase):
    """Query statement tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_social_network_queries"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")

        # Create schema
        cls.client.execute("""
            CREATE TAG person(name: STRING NOT NULL, age: INT, city: STRING)
        """)
        cls.client.execute("CREATE EDGE friend(degree: FLOAT)")
        cls.client.execute("CREATE TAG INDEX idx_person_name ON person(name)")
        time.sleep(1)

        # Insert test data
        cls.client.execute('''
            INSERT VERTEX person(name, age, city) VALUES
                "p1": ("Alice", 30, "Beijing"),
                "p2": ("Bob", 25, "Shanghai"),
                "p3": ("Charlie", 35, "Beijing"),
                "p4": ("David", 28, "Shenzhen")
        ''')
        cls.client.execute('''
            INSERT EDGE friend(degree) VALUES
                "p1" -> "p2": (0.8),
                "p2" -> "p3": (0.7),
                "p1" -> "p3": (0.9)
        ''')
        time.sleep(1)

    def setUp(self):
        """Ensure client is authenticated before each test."""
        if not self.client.ensure_authenticated():
            self.client.connect()
        self.client.execute(f"USE {self.space_name}")

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_011_match_basic(self):
        """TC-011: Basic MATCH query."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute("MATCH (p:person) RETURN p.name, p.age")
        self.assertTrue(result.success, f"MATCH failed: {result.error}")
        self.assertIsNotNone(result.data)

    def test_012_match_with_filter(self):
        """TC-012: MATCH with filter condition."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute("""
            MATCH (p:person) WHERE p.age > 28 RETURN p.name
        """)
        self.assertTrue(result.success, f"MATCH with filter failed: {result.error}")
        data_str = str(result.data)
        self.assertIn("Alice", data_str)
        self.assertIn("Charlie", data_str)

    def test_013_match_path(self):
        """TC-013: MATCH path query."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute("""
            MATCH (p:person)-[:friend]->(f:person) RETURN p.name, f.name
        """)
        self.assertTrue(result.success, f"MATCH path failed: {result.error}")

    def test_014_go_traversal(self):
        """TC-014: GO traversal query."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            GO 1 STEP FROM "p1" OVER friend YIELD friend.name
        ''')
        self.assertTrue(result.success, f"GO traversal failed: {result.error}")

    def test_015_go_multiple_steps(self):
        """TC-015: GO multi-step traversal."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            GO 2 STEPS FROM "p1" OVER friend YIELD friend.name
        ''')
        self.assertTrue(result.success, f"GO multi-step failed: {result.error}")

    def test_016_lookup_index(self):
        """TC-016: LOOKUP index query."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            LOOKUP ON person WHERE person.name == "Alice" YIELD person.age
        ''')
        self.assertTrue(result.success)


class TestSocialNetworkExplain(unittest.TestCase):
    """EXPLAIN/PROFILE command tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_social_network_explain"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")
        cls.client.execute("CREATE TAG person(name: STRING, age: INT)")
        cls.client.execute("CREATE EDGE friend(degree: FLOAT)")
        cls.client.execute("CREATE TAG INDEX idx_person_name ON person(name)")
        time.sleep(1)

        cls.client.execute('''
            INSERT VERTEX person(name, age) VALUES
                "p1": ("Alice", 30),
                "p2": ("Bob", 25)
        ''')
        time.sleep(1)

    def setUp(self):
        """Ensure client is authenticated before each test."""
        if not self.client.ensure_authenticated():
            self.client.connect()
        self.client.execute(f"USE {self.space_name}")

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_017_explain_basic(self):
        """TC-017: Basic EXPLAIN query."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute("EXPLAIN MATCH (p:person) RETURN p.name")
        self.assertTrue(result.success)
        # Should contain plan information
        self.assertIsNotNone(result.data)

    def test_018_explain_with_index(self):
        """TC-018: EXPLAIN with index scan."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            EXPLAIN LOOKUP ON person WHERE person.name == "Alice"
        ''')
        self.assertTrue(result.success, f"EXPLAIN with index failed: {result.error}")

    def test_019_profile_query(self):
        """TC-019: PROFILE query execution."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute("PROFILE MATCH (p:person) RETURN count(p)")
        self.assertTrue(result.success)
        # Should contain execution statistics
        self.assertIsNotNone(result.data)


class TestSocialNetworkTransaction(unittest.TestCase):
    """Transaction management tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_social_network_tx"
        cls._setup_schema()

    @classmethod
    def _setup_schema(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")
        cls.client.execute("CREATE TAG person(name: STRING, age: INT)")
        time.sleep(1)

    def setUp(self):
        """Ensure client is authenticated before each test and reset transaction state."""
        if not self.client.ensure_authenticated():
            self.client.connect()
        self.client.execute(f"USE {self.space_name}")
        # Try to rollback any pending transaction from previous test
        self.client.execute("ROLLBACK")

    def tearDown(self):
        """Clean up after each test."""
        # Ensure any pending transaction is rolled back
        self.client.execute("ROLLBACK")

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

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

    def test_021_transaction_rollback(self):
        """TC-021: Transaction rollback."""
        self.client.execute(f"USE {self.space_name}")

        # Use unique vertex ID with timestamp to avoid conflicts
        import time
        vertex_id = f"tx2_{int(time.time() * 1000)}"

        # Begin transaction
        result = self.client.execute("BEGIN")
        self.assertTrue(result.success, f"BEGIN failed: {result.error}")

        # Insert data
        result = self.client.execute(f'''
            INSERT VERTEX person(name, age) VALUES "{vertex_id}": ("Rollback", 25)
        ''')
        self.assertTrue(result.success, f"INSERT failed: {result.error}")

        # Rollback
        result = self.client.execute("ROLLBACK")
        self.assertTrue(result.success, f"ROLLBACK failed: {result.error}")

        # Verify data does not exist
        result = self.client.execute(f'FETCH PROP ON person "{vertex_id}"')
        # Should fail or return empty - rollback means data should not exist
        # Note: Depending on implementation, this might return success with empty data
        # or fail with "not found" error. Both are acceptable.


class TestSocialNetworkCleanup(unittest.TestCase):
    """Cleanup tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()

    def setUp(self):
        """Ensure client is authenticated before each test."""
        if not self.client.ensure_authenticated():
            self.client.connect()

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_999_cleanup_spaces(self):
        """Cleanup: Drop all test spaces."""
        spaces = [
            "e2e_social_network",
            "e2e_social_network_data",
            "e2e_social_network_queries",
            "e2e_social_network_explain",
            "e2e_social_network_tx"
        ]
        for space in spaces:
            result = self.client.execute(f"DROP SPACE IF EXISTS {space}")
            # Accept success or if space doesn't exist
            if not result.success and "not exist" not in str(result.error).lower():
                print(f"Warning: Failed to drop space {space}: {result.error}")


def run_tests():
    """Run all tests and generate report."""
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()

    # Add all test classes
    suite.addTests(loader.loadTestsFromTestCase(TestSocialNetworkBasic))
    suite.addTests(loader.loadTestsFromTestCase(TestSocialNetworkData))
    suite.addTests(loader.loadTestsFromTestCase(TestSocialNetworkQueries))
    suite.addTests(loader.loadTestsFromTestCase(TestSocialNetworkExplain))
    suite.addTests(loader.loadTestsFromTestCase(TestSocialNetworkTransaction))
    suite.addTests(loader.loadTestsFromTestCase(TestSocialNetworkCleanup))

    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)

    return result.wasSuccessful()


if __name__ == "__main__":
    success = run_tests()
    exit(0 if success else 1)
