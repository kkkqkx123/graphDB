#!/usr/bin/env python3
"""
E2E Test Suite for Schema Manager Initialization

Tests that verify schema manager is properly initialized in various scenarios:
1. Basic query operations work when vector search is disabled
2. Basic query operations work when vector search is enabled but fails to initialize
3. Schema validation works correctly
"""

import unittest
from graphdb_client import GraphDBClient


class TestSchemaManagerInitialization(unittest.TestCase):
    """Test schema manager initialization in different configurations."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        connected = cls.client.connect()
        if not connected:
            raise unittest.SkipTest("Cannot connect to GraphDB server")
        cls.test_space = "schema_manager_test_space"

    @classmethod
    def tearDownClass(cls):
        # Cleanup
        try:
            cls.client.execute(f"DROP SPACE IF EXISTS {cls.test_space}")
        except:
            pass
        cls.client.disconnect()

    def test_001_basic_connection(self):
        """TC-SCHEMA-001: Verify basic connection works."""
        result = self.client.execute("SHOW SPACES")
        self.assertTrue(result.success, f"Basic connection failed: {result.error}")
        self.assertIsNotNone(result.data)

    def test_002_create_space_without_vector(self):
        """TC-SCHEMA-002: Create space should work regardless of vector config."""
        # Drop if exists
        self.client.execute(f"DROP SPACE IF EXISTS {self.test_space}")

        # Create space - this should work even if schema_manager is not initialized
        result = self.client.execute(
            f"CREATE SPACE IF NOT EXISTS {self.test_space} (vid_type=STRING)"
        )
        self.assertTrue(
            result.success,
            f"CREATE SPACE failed - schema_manager may not be initialized: {result.error}"
        )

    def test_003_use_space(self):
        """TC-SCHEMA-003: Use space should work."""
        result = self.client.execute(f"USE {self.test_space}")
        self.assertTrue(
            result.success,
            f"USE SPACE failed - schema_manager may not be initialized: {result.error}"
        )

    def test_004_create_tag(self):
        """TC-SCHEMA-004: Create tag should work with schema_manager."""
        self.client.execute(f"USE {self.test_space}")

        result = self.client.execute("""
            CREATE TAG IF NOT EXISTS test_person(
                name STRING NOT NULL,
                age INT
            )
        """)
        self.assertTrue(
            result.success,
            f"CREATE TAG failed - schema_manager may not be initialized: {result.error}"
        )

    def test_005_show_tags(self):
        """TC-SCHEMA-005: Show tags should work."""
        self.client.execute(f"USE {self.test_space}")

        result = self.client.execute("SHOW TAGS")
        self.assertTrue(
            result.success,
            f"SHOW TAGS failed - schema_manager may not be initialized: {result.error}"
        )
        self.assertIn("test_person", str(result.data))

    def test_006_insert_vertex(self):
        """TC-SCHEMA-006: Insert vertex should work."""
        self.client.execute(f"USE {self.test_space}")

        result = self.client.execute("""
            INSERT VERTEX test_person(name, age) VALUES 'p1': ('Alice', 30)
        """)
        self.assertTrue(
            result.success,
            f"INSERT VERTEX failed - schema_manager may not be initialized: {result.error}"
        )

    def test_007_fetch_vertex(self):
        """TC-SCHEMA-007: Fetch vertex should work."""
        self.client.execute(f"USE {self.test_space}")

        result = self.client.execute("FETCH PROP ON test_person 'p1'")
        self.assertTrue(
            result.success,
            f"FETCH PROP failed - schema_manager may not be initialized: {result.error}"
        )

    def test_008_match_query(self):
        """TC-SCHEMA-008: MATCH query should work."""
        self.client.execute(f"USE {self.test_space}")

        result = self.client.execute("MATCH (v:test_person) RETURN v LIMIT 1")
        # MATCH might not be fully implemented, so we just check it doesn't crash
        # and doesn't return schema_manager error
        if not result.success:
            self.assertNotIn(
                "schema manager not initialized",
                str(result.error).lower(),
                "MATCH query failed due to schema_manager not initialized"
            )

    def test_009_drop_space(self):
        """TC-SCHEMA-009: Drop space should work."""
        result = self.client.execute(f"DROP SPACE IF EXISTS {self.test_space}")
        self.assertTrue(
            result.success,
            f"DROP SPACE failed: {result.error}"
        )


class TestSchemaManagerErrorHandling(unittest.TestCase):
    """Test error handling when schema_manager is not available."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        connected = cls.client.connect()
        if not connected:
            raise unittest.SkipTest("Cannot connect to GraphDB server")

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_error_message_clarity(self):
        """TC-SCHEMA-ERR-001: Error messages should be clear when operations fail."""
        # Try to use a non-existent space
        result = self.client.execute("USE non_existent_space_xyz")

        # Should fail, but error should not be "schema manager not initialized"
        if not result.success:
            error_msg = str(result.error).lower()
            self.assertNotIn(
                "schema manager not initialized",
                error_msg,
                "Error message indicates schema_manager not initialized - this is a server config issue"
            )

    def test_show_spaces_always_works(self):
        """TC-SCHEMA-ERR-002: SHOW SPACES should always work."""
        result = self.client.execute("SHOW SPACES")
        self.assertTrue(
            result.success,
            f"SHOW SPACES should always work but failed: {result.error}"
        )


if __name__ == "__main__":
    unittest.main()
