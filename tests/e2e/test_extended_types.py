#!/usr/bin/env python3
"""
E2E Test Suite for Extended Types

Tests extended type functionality including:
- Geography/Geospatial types
- Vector search
- Full-text search
"""

import unittest
import time
import json
from typing import List
from graphdb_client import GraphDBClient


class TestGeography(unittest.TestCase):
    """Geography/Geospatial type tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_geography"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")

        cls.client.execute("""
            CREATE TAG location(
                name: STRING NOT NULL,
                coord: GEOGRAPHY,
                address: STRING,
                category: STRING
            )
        """)
        cls.client.execute("""
            CREATE TAG city(
                name: STRING NOT NULL,
                center: GEOGRAPHY,
                population: INT
            )
        """)
        cls.client.execute("""
            CREATE EDGE nearby(distance_km: DOUBLE)
        """)
        time.sleep(1)

        # Insert cities
        cities = [
            ("Beijing", 116.4074, 39.9042, 21540000),
            ("Shanghai", 121.4737, 31.2304, 24280000),
            ("Guangzhou", 113.2644, 23.1291, 14043500),
        ]

        for i, (name, lon, lat, pop) in enumerate(cities):
            cls.client.execute(f'''
                INSERT VERTEX city(name, center, population) VALUES "city{i+1}":
                    ("{name}", ST_Point({lon}, {lat}), {pop})
            ''')

        # Insert locations
        locations = [
            ("Tiananmen", 116.3974, 39.9093, "attraction"),
            ("Forbidden City", 116.3972, 39.9163, "attraction"),
            ("Wangfujing", 116.4109, 39.9110, "shop"),
        ]

        for i, (name, lon, lat, cat) in enumerate(locations):
            cls.client.execute(f'''
                INSERT VERTEX location(name, coord, address, category) VALUES "loc{i+1:03d}":
                    ("{name}", ST_Point({lon}, {lat}), "Beijing", "{cat}")
            ''')

        time.sleep(1)

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_geo_001_point_creation(self):
        """TC-GEO-001: Create points using ST_Point."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            INSERT VERTEX location(name, coord, category) VALUES "loc_test":
                ("Test Location", ST_Point(116.4, 39.9), "test")
        ''')
        self.assertTrue(result.success, f"Failed to create point: {result.error}")

    def test_geo_002_wkt_creation(self):
        """TC-GEO-002: Create points using WKT format."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            INSERT VERTEX location(name, coord, category) VALUES "loc_wkt":
                ("WKT Location", ST_GeogFromText("POINT(116.5 39.8)"), "test")
        ''')
        self.assertTrue(result.success, f"Failed to create point from WKT: {result.error}")

    def test_geo_003_distance_calculation(self):
        """TC-GEO-003: Calculate distance between points."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            MATCH (a:location {name: "Tiananmen"}), (b:location {name: "Forbidden City"})
            RETURN ST_Distance(a.coord, b.coord) AS distance_km
        ''')
        self.assertTrue(result.success)
        # Distance should be approximately 0.8km

    def test_geo_004_within_distance(self):
        """TC-GEO-004: Find locations within distance."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            MATCH (center:location {name: "Tiananmen"})
            MATCH (loc:location)
            WHERE ST_DWithin(center.coord, loc.coord, 5.0)
            RETURN loc.name, ST_Distance(center.coord, loc.coord) AS distance
            ORDER BY distance
        ''')
        self.assertTrue(result.success)

    def test_geo_005_explain_geography_query(self):
        """TC-GEO-005: EXPLAIN geography query."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            EXPLAIN MATCH (loc:location)
            WHERE ST_DWithin(ST_Point(116.4, 39.9), loc.coord, 10.0)
            RETURN loc.name
        ''')
        self.assertTrue(result.success)


class TestVector(unittest.TestCase):
    """Vector search tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_vector"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")

        cls.client.execute("""
            CREATE TAG product_vector(
                product_id: STRING NOT NULL,
                name: STRING,
                category: STRING,
                embedding: VECTOR(128),
                price: DOUBLE
            )
        """)
        time.sleep(1)

        # Insert products with vectors
        import random
        for i in range(100):
            vector = [round(random.gauss(0, 0.1), 4) for _ in range(128)]
            vector_str = ", ".join(str(v) for v in vector)

            cls.client.execute(f'''
                INSERT VERTEX product_vector(product_id, name, category, embedding, price) VALUES "pv{i:03d}":
                    ("PROD{i:03d}", "Product {i}", "electronics", [{vector_str}], {random.uniform(10, 1000):.2f})
            ''')

        time.sleep(1)

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_vec_001_vector_insertion(self):
        """TC-VEC-001: Insert vertex with vector."""
        self.client.execute(f"USE {self.space_name}")

        vector = [0.1] * 128
        vector_str = ", ".join(str(v) for v in vector)

        result = self.client.execute(f'''
            INSERT VERTEX product_vector(product_id, name, category, embedding, price) VALUES "pv_test":
                ("TEST001", "Test Product", "test", [{vector_str}], 99.99)
        ''')
        self.assertTrue(result.success, f"Failed to insert vector: {result.error}")

    def test_vec_002_cosine_similarity(self):
        """TC-VEC-002: Cosine similarity search."""
        self.client.execute(f"USE {self.space_name}")

        query_vector = [0.1] * 128
        vector_str = ", ".join(str(v) for v in query_vector)

        result = self.client.execute(f'''
            MATCH (p:product_vector)
            ORDER BY cosine_similarity(p.embedding, [{vector_str}]) DESC
            LIMIT 10
        ''')
        self.assertTrue(result.success)

    def test_vec_003_filtered_vector_search(self):
        """TC-VEC-003: Vector search with filter."""
        self.client.execute(f"USE {self.space_name}")

        query_vector = [0.1] * 128
        vector_str = ", ".join(str(v) for v in query_vector)

        result = self.client.execute(f'''
            MATCH (p:product_vector)
            WHERE p.price < 500
            ORDER BY cosine_similarity(p.embedding, [{vector_str}]) DESC
            LIMIT 5
        ''')
        self.assertTrue(result.success)

    def test_vec_004_explain_vector_query(self):
        """TC-VEC-004: EXPLAIN vector query."""
        self.client.execute(f"USE {self.space_name}")

        query_vector = [0.1] * 128
        vector_str = ", ".join(str(v) for v in query_vector)

        result = self.client.execute(f'''
            EXPLAIN MATCH (p:product_vector)
            ORDER BY cosine_similarity(p.embedding, [{vector_str}]) DESC
            LIMIT 10
        ''')
        self.assertTrue(result.success)


class TestFullText(unittest.TestCase):
    """Full-text search tests."""

    @classmethod
    def setUpClass(cls):
        cls.client = GraphDBClient()
        cls.client.connect()
        cls.space_name = "e2e_fulltext"
        cls._setup_data()

    @classmethod
    def _setup_data(cls):
        cls.client.execute(f"DROP SPACE IF EXISTS {cls.space_name}")
        cls.client.execute(f"CREATE SPACE {cls.space_name} (vid_type=STRING)")
        cls.client.execute(f"USE {cls.space_name}")

        cls.client.execute("""
            CREATE TAG article(
                doc_id: STRING NOT NULL,
                title: STRING,
                content: STRING,
                author: STRING
            )
        """)

        # Create fulltext index
        cls.client.execute('''
            CREATE FULLTEXT INDEX idx_article_content ON article(content)
                WITH (engine=bm25, analyzer=standard)
        ''')
        time.sleep(2)  # Wait for index to be ready

        # Insert articles
        articles = [
            ("art001", "Graph Database Introduction", "Graph databases are designed for connected data", "Alice"),
            ("art002", "Query Optimization", "Optimizing queries improves performance significantly", "Bob"),
            ("art003", "Index Design", "Proper index design is crucial for database performance", "Charlie"),
        ]

        for doc_id, title, content, author in articles:
            cls.client.execute(f'''
                INSERT VERTEX article(doc_id, title, content, author) VALUES "{doc_id}":
                    ("{doc_id}", "{title}", "{content}", "{author}")
            ''')

        time.sleep(1)

    @classmethod
    def tearDownClass(cls):
        cls.client.disconnect()

    def test_ft_001_fulltext_index_creation(self):
        """TC-FT-001: Create fulltext index."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute("SHOW INDEXES")
        self.assertTrue(result.success)
        # Should contain the fulltext index
        indexes = str(result.data)
        self.assertIn("idx_article_content", indexes or "")

    def test_ft_002_basic_search(self):
        """TC-FT-002: Basic fulltext search."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            SEARCH IN article.content FOR "database"
            RETURN article.doc_id, article.title, score()
        ''')
        self.assertTrue(result.success)

    def test_ft_003_boolean_search(self):
        """TC-FT-003: Boolean query search."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            SEARCH IN article.content FOR "graph AND database"
            RETURN article.doc_id, article.title
        ''')
        self.assertTrue(result.success)

    def test_ft_004_explain_fulltext(self):
        """TC-FT-004: EXPLAIN fulltext search."""
        self.client.execute(f"USE {self.space_name}")

        result = self.client.execute('''
            EXPLAIN SEARCH IN article.content FOR "performance"
            RETURN article.doc_id, score()
        ''')
        self.assertTrue(result.success)


class TestExtendedTypesCleanup(unittest.TestCase):
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
        spaces = ["e2e_geography", "e2e_vector", "e2e_fulltext"]
        for space in spaces:
            result = self.client.execute(f"DROP SPACE IF EXISTS {space}")
            self.assertTrue(result.success or "not exist" in str(result.error).lower())


def run_tests():
    """Run all extended type tests."""
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()

    suite.addTests(loader.loadTestsFromTestCase(TestGeography))
    suite.addTests(loader.loadTestsFromTestCase(TestVector))
    suite.addTests(loader.loadTestsFromTestCase(TestFullText))
    suite.addTests(loader.loadTestsFromTestCase(TestExtendedTypesCleanup))

    runner = unittest.TextTestRunner(verbosity=2)
    result = runner.run(suite)

    return result.wasSuccessful()


if __name__ == "__main__":
    import random
    success = run_tests()
    exit(0 if success else 1)
