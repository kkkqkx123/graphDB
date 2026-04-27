#!/usr/bin/env python3
"""
GraphDB E2E Test Data Generator

This script generates test data for GraphDB E2E testing based on the design
documents in docs/tests/e2e/. It creates GQL statements for:
1. Social network scenario
2. E-commerce scenario
3. Geography/Geospatial scenario
4. Vector search scenario
5. Full-text search scenario
6. Optimizer test scenario

Usage:
    python generate_e2e_data.py --output-dir ../tests/e2e/data
    python generate_e2e_data.py --scenario social --output-file test_data.gql
"""

import argparse
import json
import random
import string
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Dict, Any, Optional


class TestDataGenerator:
    """Base class for test data generators."""

    def __init__(self, seed: int = 42):
        random.seed(seed)
        self.statements: List[str] = []

    def add(self, stmt: str):
        """Add a GQL statement."""
        self.statements.append(stmt)

    def add_comment(self, comment: str):
        """Add a comment line."""
        self.statements.append(f"-- {comment}")

    def add_empty(self):
        """Add an empty line."""
        self.statements.append("")

    def generate(self) -> str:
        """Generate the complete GQL script."""
        return "\n".join(self.statements)

    def save(self, filepath: Path):
        """Save the generated script to file."""
        filepath.parent.mkdir(parents=True, exist_ok=True)
        with open(filepath, 'w', encoding='utf-8') as f:
            f.write(self.generate())
        print(f"Generated: {filepath}")


class SocialNetworkGenerator(TestDataGenerator):
    """Generator for social network test scenario."""

    FIRST_NAMES = [
        "Alice", "Bob", "Charlie", "David", "Eve", "Frank", "Grace", "Henry",
        "Ivy", "Jack", "Kate", "Leo", "Mary", "Nick", "Olivia", "Paul",
        "Quinn", "Rose", "Sam", "Tom", "Uma", "Victor", "Wendy", "Xavier",
        "Yara", "Zack", "Amy", "Ben", "Cathy", "Dan"
    ]

    CITIES = ["Beijing", "Shanghai", "Shenzhen", "Guangzhou"]

    COMPANIES = [
        ("TechCorp", "Technology", 2010, "Beijing"),
        ("SoftSys", "Software", 2012, "Shanghai"),
        ("CloudNet", "Cloud Services", 2015, "Shenzhen"),
        ("DataWise", "Consulting", 2008, "Beijing"),
        ("WebPlus", "Internet", 2016, "Guangzhou"),
    ]

    POSITIONS = ["Engineer", "Manager", "Director", "Analyst", "Designer"]
    SALARY_RANGES = ["10k-20k", "20k-30k", "30k-50k", "50k-80k", "80k+"]

    def __init__(self, num_persons: int = 20, num_companies: int = 5,
                 num_friend_edges: int = 30, seed: int = 42):
        super().__init__(seed)
        self.num_persons = num_persons
        self.num_companies = num_companies
        self.num_friend_edges = num_friend_edges
        self.person_ids: List[str] = []
        self.company_ids: List[str] = []

    def generate_schema(self):
        """Generate schema creation statements."""
        self.add_comment("Create test space")
        self.add("CREATE SPACE IF NOT EXISTS e2e_social_network (vid_type=STRING)")
        self.add("USE e2e_social_network")
        self.add_empty()

        self.add_comment("Create tags")
        self.add("""CREATE TAG IF NOT EXISTS person(
    name: STRING NOT NULL,
    age: INT,
    email: STRING,
    city: STRING,
    created_at: TIMESTAMP DEFAULT now()
)""")
        self.add_empty()

        self.add("""CREATE TAG IF NOT EXISTS company(
    name: STRING NOT NULL,
    industry: STRING,
    founded_year: INT,
    headquarters: STRING
)""")
        self.add_empty()

        self.add_comment("Create edge types")
        self.add("""CREATE EDGE IF NOT EXISTS friend(
    degree: FLOAT DEFAULT 0.5,
    since: DATE,
    trust_level: INT
)""")
        self.add_empty()

        self.add("""CREATE EDGE IF NOT EXISTS works_at(
    position: STRING,
    since: DATE,
    salary_range: STRING
)""")
        self.add_empty()

        self.add("""CREATE EDGE IF NOT EXISTS lives_in(
    since: DATE,
    address: STRING
)""")
        self.add_empty()

        self.add_comment("Create indexes")
        self.add("CREATE TAG INDEX IF NOT EXISTS idx_person_name ON person(name)")
        self.add("CREATE TAG INDEX IF NOT EXISTS idx_person_age ON person(age)")
        self.add("CREATE TAG INDEX IF NOT EXISTS idx_person_city ON person(city)")
        self.add_empty()

    def generate_persons(self):
        """Generate person vertices."""
        self.add_comment(f"Generate {self.num_persons} persons")

        base_date = datetime(2020, 1, 1)

        for i in range(self.num_persons):
            person_id = f"p{i+1}"
            self.person_ids.append(person_id)

            name = self.FIRST_NAMES[i % len(self.FIRST_NAMES)]
            age = random.randint(24, 36)
            email = f"{name.lower()}@example.com"
            city = random.choice(self.CITIES)
            created_at = (base_date + timedelta(days=random.randint(0, 365))).strftime("%Y-%m-%dT%H:%M:%S")

            self.add(f'''INSERT VERTEX person(name, age, email, city, created_at) VALUES "{person_id}":
    ("{name}", {age}, "{email}", "{city}", datetime("{created_at}"))''')

        self.add_empty()

    def generate_companies(self):
        """Generate company vertices."""
        self.add_comment(f"Generate {self.num_companies} companies")

        for i, (name, industry, founded, hq) in enumerate(self.COMPANIES[:self.num_companies]):
            company_id = f"c{i+1}"
            self.company_ids.append(company_id)

            self.add(f'''INSERT VERTEX company(name, industry, founded_year, headquarters) VALUES "{company_id}":
    ("{name}", "{industry}", {founded}, "{hq}")''')

        self.add_empty()

    def generate_friend_edges(self):
        """Generate friend relationships forming a connected network."""
        self.add_comment(f"Generate {self.num_friend_edges} friend relationships")

        edges = set()
        # Ensure network connectivity - create a spanning tree first
        for i in range(1, len(self.person_ids)):
            src = self.person_ids[i]
            dst = self.person_ids[random.randint(0, i-1)]
            edges.add((src, dst))

        # Add remaining random edges
        while len(edges) < self.num_friend_edges:
            src = random.choice(self.person_ids)
            dst = random.choice(self.person_ids)
            if src != dst and (src, dst) not in edges and (dst, src) not in edges:
                edges.add((src, dst))

        base_date = datetime(2018, 1, 1)
        for src, dst in edges:
            degree = round(random.uniform(0.6, 0.95), 2)
            since = (base_date + timedelta(days=random.randint(0, 1000))).strftime("%Y-%m-%d")
            trust_level = random.randint(1, 5)

            self.add(f'''INSERT EDGE friend(degree, since, trust_level) VALUES "{src}" -> "{dst}" @0:
    ({degree}, date("{since}"), {trust_level})''')

        self.add_empty()

    def generate_work_edges(self):
        """Generate work relationships."""
        self.add_comment("Generate work relationships")

        base_date = datetime(2019, 1, 1)
        for i, person_id in enumerate(self.person_ids):
            company_id = self.company_ids[i % len(self.company_ids)]
            position = random.choice(self.POSITIONS)
            since = (base_date + timedelta(days=random.randint(0, 730))).strftime("%Y-%m-%d")
            salary = random.choice(self.SALARY_RANGES)

            self.add(f'''INSERT EDGE works_at(position, since, salary_range) VALUES "{person_id}" -> "{company_id}" @0:
    ("{position}", date("{since}"), "{salary}")''')

        self.add_empty()

    def generate_lives_in_edges(self):
        """Generate lives_in relationships."""
        self.add_comment("Generate lives_in relationships")

        base_date = datetime(2017, 1, 1)
        for person_id in self.person_ids:
            city = random.choice(self.CITIES)
            since = (base_date + timedelta(days=random.randint(0, 1000))).strftime("%Y-%m-%d")
            address = f"Street {random.randint(1, 100)}, {city}"

            self.add(f'''INSERT EDGE lives_in(since, address) VALUES "{person_id}" -> "{city}" @0:
    (date("{since}"), "{address}")''')

        self.add_empty()

    def generate(self) -> str:
        """Generate complete social network test data."""
        self.add_comment("Social Network E2E Test Data")
        self.add_comment(f"Generated: {datetime.now().isoformat()}")
        self.add_empty()

        self.generate_schema()
        self.generate_persons()
        self.generate_companies()
        self.generate_friend_edges()
        self.generate_work_edges()
        self.generate_lives_in_edges()

        self.add_comment("Verify data")
        self.add('MATCH (p:person) RETURN count(p) AS person_count')
        self.add('MATCH ()-[f:friend]->() RETURN count(f) AS friend_count')
        self.add('MATCH ()-[w:works_at]->() RETURN count(w) AS works_count')

        return super().generate()


class ECommerceGenerator(TestDataGenerator):
    """Generator for e-commerce test scenario."""

    CATEGORIES = [
        "Electronics", "Clothing", "Food", "Home", "Sports",
        "Books", "Beauty", "Toys", "Automotive", "Garden"
    ]

    PRODUCT_NAMES = [
        "Laptop", "Smartphone", "Headphones", "T-Shirt", "Jeans",
        "Sneakers", "Coffee", "Chocolate", "Chair", "Desk",
        "Bicycle", "Tennis Racket", "Novel", "Textbook", "Shampoo",
        "Face Cream", "LEGO Set", "Action Figure", "Car Charger", "Plant"
    ]

    def __init__(self, num_users: int = 100, num_products: int = 200,
                 num_orders: int = 500, seed: int = 42):
        super().__init__(seed)
        self.num_users = num_users
        self.num_products = num_products
        self.num_orders = num_orders
        self.user_ids: List[str] = []
        self.product_ids: List[str] = []
        self.order_ids: List[str] = []

    def generate_schema(self):
        """Generate schema creation statements."""
        self.add_comment("Create test space")
        self.add("CREATE SPACE IF NOT EXISTS e2e_ecommerce (vid_type=STRING)")
        self.add("USE e2e_ecommerce")
        self.add_empty()

        self.add_comment("Create tags")
        self.add("""CREATE TAG IF NOT EXISTS user(
    user_id: STRING NOT NULL,
    username: STRING,
    email: STRING,
    register_date: DATE,
    level: INT DEFAULT 1
)""")
        self.add_empty()

        self.add("""CREATE TAG IF NOT EXISTS product(
    product_id: STRING NOT NULL,
    name: STRING,
    category: STRING,
    price: DOUBLE,
    stock: INT DEFAULT 0
)""")
        self.add_empty()

        self.add("""CREATE TAG IF NOT EXISTS order(
    order_id: STRING NOT NULL,
    total_amount: DOUBLE,
    status: STRING,
    create_time: TIMESTAMP,
    pay_time: TIMESTAMP
)""")
        self.add_empty()

        self.add_comment("Create edge types")
        self.add("""CREATE EDGE IF NOT EXISTS placed(
    order_time: TIMESTAMP,
    ip_address: STRING
)""")
        self.add_empty()

        self.add("""CREATE EDGE IF NOT EXISTS contains(
    quantity: INT,
    unit_price: DOUBLE,
    discount: DOUBLE DEFAULT 0.0
)""")
        self.add_empty()

        self.add("""CREATE EDGE IF NOT EXISTS views(
    view_time: TIMESTAMP,
    duration_seconds: INT
)""")
        self.add_empty()

    def generate_users(self):
        """Generate user vertices."""
        self.add_comment(f"Generate {self.num_users} users")

        base_date = datetime(2022, 1, 1)

        for i in range(self.num_users):
            user_id = f"u{i+1:04d}"
            self.user_ids.append(user_id)

            username = f"user_{i+1:04d}"
            email = f"{username}@example.com"
            register_date = (base_date + timedelta(days=random.randint(0, 730))).strftime("%Y-%m-%d")
            level = random.randint(1, 5)

            self.add(f'''INSERT VERTEX user(user_id, username, email, register_date, level) VALUES "{user_id}":
    ("{user_id}", "{username}", "{email}", date("{register_date}"), {level})''')

        self.add_empty()

    def generate_products(self):
        """Generate product vertices."""
        self.add_comment(f"Generate {self.num_products} products")

        for i in range(self.num_products):
            product_id = f"prod{i+1:05d}"
            self.product_ids.append(product_id)

            name = random.choice(self.PRODUCT_NAMES)
            category = random.choice(self.CATEGORIES)
            price = round(random.uniform(10, 10000), 2)
            stock = random.randint(0, 1000)

            self.add(f'''INSERT VERTEX product(product_id, name, category, price, stock) VALUES "{product_id}":
    ("{product_id}", "{name}", "{category}", {price}, {stock})''')

        self.add_empty()

    def generate_orders(self):
        """Generate order vertices and relationships."""
        self.add_comment(f"Generate {self.num_orders} orders")

        statuses = ["pending", "paid", "shipped", "completed", "cancelled"]
        base_date = datetime(2023, 1, 1)

        for i in range(self.num_orders):
            order_id = f"ord{i+1:06d}"
            self.order_ids.append(order_id)

            total_amount = round(random.uniform(50, 5000), 2)
            status = random.choice(statuses)
            create_time = (base_date + timedelta(days=random.randint(0, 365))).strftime("%Y-%m-%dT%H:%M:%S")
            pay_time = (datetime.fromisoformat(create_time.replace('Z', '+00:00')) +
                       timedelta(hours=random.randint(1, 24))).strftime("%Y-%m-%dT%H:%M:%S")

            self.add(f'''INSERT VERTEX order(order_id, total_amount, status, create_time, pay_time) VALUES "{order_id}":
    ("{order_id}", {total_amount}, "{status}", datetime("{create_time}"), datetime("{pay_time}"))''')

        self.add_empty()

    def generate_placed_edges(self):
        """Generate placed relationships."""
        self.add_comment("Generate placed relationships")

        base_date = datetime(2023, 1, 1)

        for order_id in self.order_ids:
            user_id = random.choice(self.user_ids)
            order_time = (base_date + timedelta(days=random.randint(0, 365))).strftime("%Y-%m-%dT%H:%M:%S")
            ip_address = f"192.168.{random.randint(0, 255)}.{random.randint(0, 255)}"

            self.add(f'''INSERT EDGE placed(order_time, ip_address) VALUES "{user_id}" -> "{order_id}" @0:
    (datetime("{order_time}"), "{ip_address}")''')

        self.add_empty()

    def generate_contains_edges(self):
        """Generate contains relationships."""
        self.add_comment("Generate contains relationships")

        for order_id in self.order_ids:
            # Each order contains 1-8 products
            num_items = random.randint(1, 8)
            selected_products = random.sample(self.product_ids, min(num_items, len(self.product_ids)))

            for product_id in selected_products:
                quantity = random.randint(1, 5)
                unit_price = round(random.uniform(10, 1000), 2)
                discount = round(random.uniform(0, 0.3), 2)

                self.add(f'''INSERT EDGE contains(quantity, unit_price, discount) VALUES "{order_id}" -> "{product_id}" @0:
    ({quantity}, {unit_price}, {discount})''')

        self.add_empty()

    def generate_views_edges(self):
        """Generate views relationships."""
        self.add_comment("Generate views relationships")

        base_date = datetime(2023, 6, 1)
        view_records = []

        # Each user views 20-80 products on average
        for user_id in self.user_ids:
            num_views = random.randint(20, 80)
            viewed_products = random.sample(self.product_ids, min(num_views, len(self.product_ids)))

            for product_id in viewed_products:
                view_time = (base_date + timedelta(days=random.randint(0, 180))).strftime("%Y-%m-%dT%H:%M:%S")
                duration = random.randint(5, 300)
                view_records.append((user_id, product_id, view_time, duration))

        # Limit total views to avoid too large dataset
        view_records = random.sample(view_records, min(5000, len(view_records)))

        for user_id, product_id, view_time, duration in view_records:
            self.add(f'''INSERT EDGE views(view_time, duration_seconds) VALUES "{user_id}" -> "{product_id}" @0:
    (datetime("{view_time}"), {duration})''')

        self.add_empty()

    def generate(self) -> str:
        """Generate complete e-commerce test data."""
        self.add_comment("E-Commerce E2E Test Data")
        self.add_comment(f"Generated: {datetime.now().isoformat()}")
        self.add_empty()

        self.generate_schema()
        self.generate_users()
        self.generate_products()
        self.generate_orders()
        self.generate_placed_edges()
        self.generate_contains_edges()
        self.generate_views_edges()

        self.add_comment("Verify data")
        self.add('MATCH (u:user) RETURN count(u) AS user_count')
        self.add('MATCH (p:product) RETURN count(p) AS product_count')
        self.add('MATCH (o:order) RETURN count(o) AS order_count')

        return super().generate()


class GeographyGenerator(TestDataGenerator):
    """Generator for geography/geospatial test scenario."""

    CITIES = [
        ("Beijing", 116.4074, 39.9042, 21540000),
        ("Shanghai", 121.4737, 31.2304, 24280000),
        ("Guangzhou", 113.2644, 23.1291, 14043500),
        ("Shenzhen", 114.0579, 22.5431, 12528300),
        ("Chengdu", 104.0668, 30.5728, 16330000),
        ("Hangzhou", 120.1551, 30.2741, 10360000),
        ("Wuhan", 114.3055, 30.5928, 11212000),
        ("Xi'an", 108.9398, 34.3416, 12952900),
        ("Nanjing", 118.7969, 32.0603, 8500000),
        ("Chongqing", 106.5516, 29.5630, 32054100),
    ]

    LOCATION_CATEGORIES = ["restaurant", "hotel", "attraction", "shop", "park"]

    def __init__(self, num_locations: int = 200, seed: int = 42):
        super().__init__(seed)
        self.num_locations = num_locations
        self.location_ids: List[str] = []

    def generate_schema(self):
        """Generate schema creation statements."""
        self.add_comment("Create test space")
        self.add("CREATE SPACE IF NOT EXISTS e2e_geography (vid_type=STRING)")
        self.add("USE e2e_geography")
        self.add_empty()

        self.add_comment("Create tags")
        self.add("""CREATE TAG IF NOT EXISTS location(
    name: STRING NOT NULL,
    coord: GEOGRAPHY,
    address: STRING,
    category: STRING
)""")
        self.add_empty()

        self.add("""CREATE TAG IF NOT EXISTS city(
    name: STRING NOT NULL,
    center: GEOGRAPHY,
    population: INT,
    area_km2: DOUBLE
)""")
        self.add_empty()

        self.add_comment("Create edge types")
        self.add("""CREATE EDGE IF NOT EXISTS nearby(
    distance_km: DOUBLE,
    walking_time_min: INT
)""")
        self.add_empty()

    def generate_cities(self):
        """Generate city vertices."""
        self.add_comment("Generate cities")

        for i, (name, lon, lat, population) in enumerate(self.CITIES):
            city_id = f"city{i+1}"
            area_km2 = round(random.uniform(5000, 35000), 2)

            self.add(f'''INSERT VERTEX city(name, center, population, area_km2) VALUES "{city_id}":
    ("{name}", ST_Point({lon}, {lat}), {population}, {area_km2})''')

        self.add_empty()

    def generate_locations(self):
        """Generate location vertices with coordinates."""
        self.add_comment(f"Generate {self.num_locations} locations")

        for i in range(self.num_locations):
            location_id = f"loc{i+1:03d}"
            self.location_ids.append(location_id)

            # Pick a random city center and add random offset
            city_name, city_lon, city_lat, _ = random.choice(self.CITIES)
            # Generate coordinates within ~20km of city center
            lon = city_lon + random.uniform(-0.2, 0.2)
            lat = city_lat + random.uniform(-0.2, 0.2)

            name = f"Location_{i+1:03d}"
            category = random.choice(self.LOCATION_CATEGORIES)
            address = f"{random.randint(1, 999)} {city_name} Road"

            self.add(f'''INSERT VERTEX location(name, coord, address, category) VALUES "{location_id}":
    ("{name}", ST_Point({lon:.4f}, {lat:.4f}), "{address}", "{category}")''')

        self.add_empty()

    def generate_nearby_edges(self):
        """Generate nearby relationships based on distance."""
        self.add_comment("Generate nearby relationships")

        edges_added = 0
        for i, loc1 in enumerate(self.location_ids):
            for j, loc2 in enumerate(self.location_ids[i+1:], i+1):
                if edges_added >= 500:  # Limit to 500 edges
                    break

                # Randomly decide if locations are nearby (simulated)
                if random.random() < 0.05:  # 5% probability
                    distance_km = round(random.uniform(0.5, 50), 2)
                    walking_time = int(distance_km * 12)  # ~5km/h walking speed

                    self.add(f'''INSERT EDGE nearby(distance_km, walking_time_min) VALUES "{loc1}" -> "{loc2}" @0:
    ({distance_km}, {walking_time})''')
                    edges_added += 1

        self.add_empty()

    def generate(self) -> str:
        """Generate complete geography test data."""
        self.add_comment("Geography E2E Test Data")
        self.add_comment(f"Generated: {datetime.now().isoformat()}")
        self.add_empty()

        self.generate_schema()
        self.generate_cities()
        self.generate_locations()
        self.generate_nearby_edges()

        self.add_comment("Verify data")
        self.add('MATCH (c:city) RETURN count(c) AS city_count')
        self.add('MATCH (l:location) RETURN count(l) AS location_count')
        self.add('MATCH ()-[n:nearby]->() RETURN count(n) AS nearby_count')

        return super().generate()


class VectorGenerator(TestDataGenerator):
    """Generator for vector search test scenario."""

    CATEGORIES = ["electronics", "clothing", "food", "home", "sports"]

    def __init__(self, num_products: int = 1000, num_images: int = 500,
                 num_texts: int = 2000, seed: int = 42):
        super().__init__(seed)
        self.num_products = num_products
        self.num_images = num_images
        self.num_texts = num_texts

    def generate_schema(self):
        """Generate schema creation statements."""
        self.add_comment("Create test space")
        self.add("CREATE SPACE IF NOT EXISTS e2e_vector (vid_type=STRING)")
        self.add("USE e2e_vector")
        self.add_empty()

        self.add_comment("Create tags")
        self.add("""CREATE TAG IF NOT EXISTS product_vector(
    product_id: STRING NOT NULL,
    name: STRING,
    category: STRING,
    embedding: VECTOR(128),
    price: DOUBLE,
    tags: LIST<STRING>
)""")
        self.add_empty()

        self.add("""CREATE TAG IF NOT EXISTS image_vector(
    image_id: STRING NOT NULL,
    url: STRING,
    feature: VECTOR(512),
    labels: LIST<STRING>
)""")
        self.add_empty()

        self.add("""CREATE TAG IF NOT EXISTS text_embedding(
    doc_id: STRING NOT NULL,
    content: STRING,
    embedding: VECTOR(768),
    category: STRING
)""")
        self.add_empty()

    def generate_product_vectors(self):
        """Generate product vertices with 128-dim vectors."""
        self.add_comment(f"Generate {self.num_products} product vectors")

        for i in range(self.num_products):
            product_id = f"pv{i+1:05d}"
            name = f"Product_{i+1:05d}"
            category = random.choice(self.CATEGORIES)
            price = round(random.uniform(10, 10000), 2)
            tags = json.dumps(random.sample(["new", "sale", "hot", "limited"], random.randint(0, 3)))

            # Generate 128-dim vector
            vector = [round(random.gauss(0, 0.1), 4) for _ in range(128)]
            vector_str = ", ".join(str(v) for v in vector)

            self.add(f'''INSERT VERTEX product_vector(product_id, name, category, embedding, price, tags) VALUES "{product_id}":
    ("{product_id}", "{name}", "{category}", [{vector_str}], {price}, {tags})''')

        self.add_empty()

    def generate_image_vectors(self):
        """Generate image vertices with 512-dim vectors."""
        self.add_comment(f"Generate {self.num_images} image vectors")

        for i in range(self.num_images):
            image_id = f"img{i+1:04d}"
            url = f"https://example.com/images/{image_id}.jpg"
            labels = json.dumps(random.sample(["person", "car", "building", "nature"], random.randint(1, 3)))

            # Generate 512-dim vector
            vector = [round(random.gauss(0, 0.1), 4) for _ in range(512)]
            vector_str = ", ".join(str(v) for v in vector)

            self.add(f'''INSERT VERTEX image_vector(image_id, url, feature, labels) VALUES "{image_id}":
    ("{image_id}", "{url}", [{vector_str}], {labels})''')

        self.add_empty()

    def generate_text_embeddings(self):
        """Generate text vertices with 768-dim vectors."""
        self.add_comment(f"Generate {self.num_texts} text embeddings")

        for i in range(self.num_texts):
            doc_id = f"doc{i+1:05d}"
            content = f"This is sample document content for testing vector search. Document number {i+1}."
            category = random.choice(["tech", "news", "blog", "paper"])

            # Generate 768-dim vector
            vector = [round(random.gauss(0, 0.1), 4) for _ in range(768)]
            vector_str = ", ".join(str(v) for v in vector)

            self.add(f'''INSERT VERTEX text_embedding(doc_id, content, embedding, category) VALUES "{doc_id}":
    ("{doc_id}", "{content}", [{vector_str}], "{category}")''')

        self.add_empty()

    def generate(self) -> str:
        """Generate complete vector test data."""
        self.add_comment("Vector Search E2E Test Data")
        self.add_comment(f"Generated: {datetime.now().isoformat()}")
        self.add_empty()

        self.generate_schema()
        self.generate_product_vectors()
        self.generate_image_vectors()
        self.generate_text_embeddings()

        self.add_comment("Verify data")
        self.add('MATCH (p:product_vector) RETURN count(p) AS product_count')
        self.add('MATCH (i:image_vector) RETURN count(i) AS image_count')
        self.add('MATCH (t:text_embedding) RETURN count(t) AS text_count')

        return super().generate()


class FullTextGenerator(TestDataGenerator):
    """Generator for full-text search test scenario."""

    TECH_KEYWORDS = [
        "database", "graph", "index", "query", "optimization", "performance",
        "storage", "memory", "cache", "distributed", "transaction", "ACID",
        "SQL", "NoSQL", "schema", "model", "design", "architecture"
    ]

    def __init__(self, num_articles: int = 500, num_products: int = 1000, seed: int = 42):
        super().__init__(seed)
        self.num_articles = num_articles
        self.num_products = num_products

    def generate_schema(self):
        """Generate schema creation statements."""
        self.add_comment("Create test space")
        self.add("CREATE SPACE IF NOT EXISTS e2e_fulltext (vid_type=STRING)")
        self.add("USE e2e_fulltext")
        self.add_empty()

        self.add_comment("Create tags")
        self.add("""CREATE TAG IF NOT EXISTS article(
    doc_id: STRING NOT NULL,
    title: STRING,
    content: STRING,
    author: STRING,
    publish_date: TIMESTAMP,
    tags: LIST<STRING>
)""")
        self.add_empty()

        self.add("""CREATE TAG IF NOT EXISTS product_desc(
    sku: STRING NOT NULL,
    name: STRING,
    description: STRING,
    category: STRING
)""")
        self.add_empty()

        self.add_comment("Create fulltext indexes")
        self.add("CREATE FULLTEXT INDEX IF NOT EXISTS idx_article_content ON article(content) WITH (engine=bm25, analyzer=standard)")
        self.add("CREATE FULLTEXT INDEX IF NOT EXISTS idx_article_title ON article(title) WITH (engine=inversearch, analyzer=cjk)")
        self.add_empty()

    def generate_articles(self):
        """Generate article vertices."""
        self.add_comment(f"Generate {self.num_articles} articles")

        authors = ["Alice", "Bob", "Charlie", "David", "Eve"]
        base_date = datetime(2023, 1, 1)

        for i in range(self.num_articles):
            doc_id = f"art{i+1:04d}"
            title = f"Article about {random.choice(self.TECH_KEYWORDS)} and {random.choice(self.TECH_KEYWORDS)}"

            # Generate content with tech keywords
            content_words = random.choices(self.TECH_KEYWORDS, k=random.randint(20, 50))
            content = " ".join(content_words)

            author = random.choice(authors)
            publish_date = (base_date + timedelta(days=random.randint(0, 365))).strftime("%Y-%m-%dT%H:%M:%S")
            tags = json.dumps(random.sample(self.TECH_KEYWORDS, random.randint(2, 5)))

            self.add(f'''INSERT VERTEX article(doc_id, title, content, author, publish_date, tags) VALUES "{doc_id}":
    ("{doc_id}", "{title}", "{content}", "{author}", datetime("{publish_date}"), {tags})''')

        self.add_empty()

    def generate_products(self):
        """Generate product description vertices."""
        self.add_comment(f"Generate {self.num_products} product descriptions")

        categories = ["Electronics", "Clothing", "Home", "Sports"]

        for i in range(self.num_products):
            sku = f"SKU{i+1:06d}"
            name = f"Product {i+1:06d}"

            # Generate description
            desc_words = random.choices(self.TECH_KEYWORDS, k=random.randint(10, 30))
            description = " ".join(desc_words)

            category = random.choice(categories)

            self.add(f'''INSERT VERTEX product_desc(sku, name, description, category) VALUES "{sku}":
    ("{sku}", "{name}", "{description}", "{category}")''')

        self.add_empty()

    def generate(self) -> str:
        """Generate complete full-text test data."""
        self.add_comment("Full-Text Search E2E Test Data")
        self.add_comment(f"Generated: {datetime.now().isoformat()}")
        self.add_empty()

        self.generate_schema()
        self.generate_articles()
        self.generate_products()

        self.add_comment("Verify data")
        self.add('MATCH (a:article) RETURN count(a) AS article_count')
        self.add('MATCH (p:product_desc) RETURN count(p) AS product_count')

        return super().generate()


class OptimizerGenerator(TestDataGenerator):
    """Generator for optimizer test scenario."""

    DEPARTMENTS = ["Engineering", "Sales", "Marketing", "HR", "Finance"]
    CITIES = ["Beijing", "Shanghai", "Shenzhen", "Guangzhou", "Hangzhou"]

    def __init__(self, num_persons: int = 10000, num_companies: int = 100,
                 seed: int = 42):
        super().__init__(seed)
        self.num_persons = num_persons
        self.num_companies = num_companies

    def generate_schema(self):
        """Generate schema creation statements."""
        self.add_comment("Create test space")
        self.add("CREATE SPACE IF NOT EXISTS e2e_optimizer (vid_type=STRING)")
        self.add("USE e2e_optimizer")
        self.add_empty()

        self.add_comment("Create tags")
        self.add("""CREATE TAG IF NOT EXISTS person(
    name: STRING,
    age: INT,
    city: STRING,
    salary: INT,
    department: STRING
)""")
        self.add_empty()

        self.add("""CREATE TAG IF NOT EXISTS company(
    name: STRING,
    industry: STRING,
    size: INT
)""")
        self.add_empty()

        self.add_comment("Create edge types")
        self.add("""CREATE EDGE IF NOT EXISTS works_at(
    position: STRING,
    salary: INT
)""")
        self.add_empty()

        self.add("""CREATE EDGE IF NOT EXISTS manages(
    since: DATE
)""")
        self.add_empty()

        self.add_comment("Create indexes")
        self.add("CREATE TAG INDEX IF NOT EXISTS idx_person_name ON person(name)")
        self.add("CREATE TAG INDEX IF NOT EXISTS idx_person_age ON person(age)")
        self.add("CREATE TAG INDEX IF NOT EXISTS idx_person_city ON person(city)")
        self.add_empty()

    def generate_persons(self):
        """Generate person vertices."""
        self.add_comment(f"Generate {self.num_persons} persons")

        first_names = ["Alice", "Bob", "Charlie", "David", "Eve", "Frank", "Grace", "Henry"]

        for i in range(self.num_persons):
            person_id = f"person{i+1:05d}"
            name = random.choice(first_names) + f"_{i+1}"
            age = random.randint(22, 60)
            city = random.choice(self.CITIES)
            salary = random.randint(5000, 100000)
            department = random.choice(self.DEPARTMENTS)

            self.add(f'''INSERT VERTEX person(name, age, city, salary, department) VALUES "{person_id}":
    ("{name}", {age}, "{city}", {salary}, "{department}")''')

        self.add_empty()

    def generate_companies(self):
        """Generate company vertices."""
        self.add_comment(f"Generate {self.num_companies} companies")

        industries = ["Tech", "Finance", "Healthcare", "Retail", "Manufacturing"]

        for i in range(self.num_companies):
            company_id = f"comp{i+1:03d}"
            name = f"Company_{i+1:03d}"
            industry = random.choice(industries)
            size = random.randint(10, 10000)

            self.add(f'''INSERT VERTEX company(name, industry, size) VALUES "{company_id}":
    ("{name}", "{industry}", {size})''')

        self.add_empty()

    def generate_works_at_edges(self):
        """Generate works_at relationships."""
        self.add_comment("Generate works_at relationships")

        positions = ["Engineer", "Manager", "Director", "Analyst", "Specialist"]
        base_date = datetime(2020, 1, 1)

        for i in range(self.num_persons):
            person_id = f"person{i+1:05d}"
            company_id = f"comp{random.randint(1, self.num_companies):03d}"
            position = random.choice(positions)
            salary = random.randint(5000, 100000)

            self.add(f'''INSERT EDGE works_at(position, salary) VALUES "{person_id}" -> "{company_id}" @0:
    ("{position}", {salary})''')

        self.add_empty()

    def generate(self) -> str:
        """Generate complete optimizer test data."""
        self.add_comment("Optimizer E2E Test Data")
        self.add_comment(f"Generated: {datetime.now().isoformat()}")
        self.add_empty()

        self.generate_schema()
        self.generate_persons()
        self.generate_companies()
        self.generate_works_at_edges()

        self.add_comment("Verify data")
        self.add('MATCH (p:person) RETURN count(p) AS person_count')
        self.add('MATCH (c:company) RETURN count(c) AS company_count')
        self.add('MATCH ()-[w:works_at]->() RETURN count(w) AS works_count')

        return super().generate()


def main():
    parser = argparse.ArgumentParser(
        description="Generate E2E test data for GraphDB"
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default="tests/e2e/data",
        help="Output directory for generated files"
    )
    parser.add_argument(
        "--scenario",
        type=str,
        choices=["social", "ecommerce", "geography", "vector", "fulltext", "optimizer", "all"],
        default="all",
        help="Which scenario to generate (default: all)"
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Random seed for reproducibility"
    )

    args = parser.parse_args()
    output_dir = Path(args.output_dir)

    generators = {
        "social": (SocialNetworkGenerator, "social_network_data.gql"),
        "ecommerce": (ECommerceGenerator, "ecommerce_data.gql"),
        "geography": (GeographyGenerator, "geography_data.gql"),
        "vector": (VectorGenerator, "vector_data.gql"),
        "fulltext": (FullTextGenerator, "fulltext_data.gql"),
        "optimizer": (OptimizerGenerator, "optimizer_data.gql"),
    }

    if args.scenario == "all":
        scenarios = list(generators.keys())
    else:
        scenarios = [args.scenario]

    for scenario in scenarios:
        gen_class, filename = generators[scenario]
        generator = gen_class(seed=args.seed)
        output_path = output_dir / filename
        generator.save(output_path)

    print(f"\nAll test data generated in: {output_dir.absolute()}")


if __name__ == "__main__":
    main()
