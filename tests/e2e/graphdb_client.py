#!/usr/bin/env python3
"""
GraphDB HTTP Client for E2E Testing

Provides a Python client for interacting with GraphDB HTTP API.
This client is used by E2E test scripts to execute GQL statements.
"""

import json
import time
import requests
from dataclasses import dataclass
from typing import Optional, Dict, Any, List
from urllib.parse import urljoin


@dataclass
class TestResult:
    """Result of a GQL execution."""
    success: bool
    data: Optional[Any] = None
    error: Optional[str] = None
    execution_time_ms: Optional[float] = None


class GraphDBClient:
    """HTTP client for GraphDB."""

    def __init__(
        self,
        host: str = "127.0.0.1",
        port: int = 9758,
        timeout: int = 30,
        retry_count: int = 3
    ):
        self.host = host
        self.port = port
        self.timeout = timeout
        self.retry_count = retry_count
        self.base_url = f"http://{host}:{port}"
        self.session: Optional[requests.Session] = None
        self.current_space: Optional[str] = None

    def connect(self) -> bool:
        """Establish connection to GraphDB server."""
        self.session = requests.Session()
        self.session.headers.update({
            "Content-Type": "application/json",
            "Accept": "application/json"
        })

        # Test connection
        for attempt in range(self.retry_count):
            try:
                response = self.session.get(
                    f"{self.base_url}/v1/health",
                    timeout=self.timeout
                )
                if response.status_code == 200:
                    return True
            except requests.exceptions.ConnectionError:
                if attempt < self.retry_count - 1:
                    time.sleep(1)
                continue

        return False

    def disconnect(self):
        """Close connection."""
        if self.session:
            self.session.close()
            self.session = None

    def is_connected(self) -> bool:
        """Check if connected to server."""
        if not self.session:
            return False
        try:
            response = self.session.get(
                f"{self.base_url}/v1/health",
                timeout=5
            )
            return response.status_code == 200
        except:
            return False

    def execute(self, gql: str, params: Optional[Dict[str, Any]] = None) -> TestResult:
        """Execute a GQL statement."""
        if not self.session:
            return TestResult(
                success=False,
                error="Not connected to server"
            )

        start_time = time.time()

        try:
            payload = {
                "gql": gql,
                "params": params or {}
            }

            response = self.session.post(
                f"{self.base_url}/v1/execute",
                json=payload,
                timeout=self.timeout
            )

            execution_time = (time.time() - start_time) * 1000

            if response.status_code == 200:
                result = response.json()
                return TestResult(
                    success=result.get("success", False),
                    data=result.get("data"),
                    error=result.get("error"),
                    execution_time_ms=execution_time
                )
            else:
                return TestResult(
                    success=False,
                    error=f"HTTP {response.status_code}: {response.text}",
                    execution_time_ms=execution_time
                )

        except requests.exceptions.Timeout:
            return TestResult(
                success=False,
                error=f"Request timeout after {self.timeout}s",
                execution_time_ms=(time.time() - start_time) * 1000
            )
        except Exception as e:
            return TestResult(
                success=False,
                error=str(e),
                execution_time_ms=(time.time() - start_time) * 1000
            )

    def execute_script(self, script_path: str) -> List[TestResult]:
        """Execute a GQL script file."""
        results = []

        with open(script_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # Split by semicolons, handling multiline statements
        statements = []
        current = []
        for line in content.split('\n'):
            line = line.strip()
            if not line or line.startswith('--'):
                continue

            current.append(line)
            if line.endswith(';'):
                stmt = ' '.join(current).rstrip(';').strip()
                if stmt:
                    statements.append(stmt)
                current = []

        # Handle last statement if not terminated
        if current:
            stmt = ' '.join(current).rstrip(';').strip()
            if stmt:
                statements.append(stmt)

        for stmt in statements:
            result = self.execute(stmt)
            results.append(result)
            if not result.success:
                print(f"Warning: Statement failed: {stmt[:50]}...")
                print(f"  Error: {result.error}")

        return results

    def explain(self, query: str, format_type: str = "text") -> TestResult:
        """Execute EXPLAIN command."""
        if format_type == "dot":
            gql = f"EXPLAIN FORMAT = DOT {query}"
        else:
            gql = f"EXPLAIN {query}"
        return self.execute(gql)

    def profile(self, query: str) -> TestResult:
        """Execute PROFILE command."""
        gql = f"PROFILE {query}"
        return self.execute(gql)

    def get_spaces(self) -> TestResult:
        """Get list of spaces."""
        return self.execute("SHOW SPACES")

    def create_space(self, name: str, vid_type: str = "STRING") -> TestResult:
        """Create a new space."""
        return self.execute(f"CREATE SPACE {name} (vid_type={vid_type})")

    def drop_space(self, name: str) -> TestResult:
        """Drop a space."""
        return self.execute(f"DROP SPACE IF EXISTS {name}")

    def use_space(self, name: str) -> TestResult:
        """Switch to a space."""
        result = self.execute(f"USE {name}")
        if result.success:
            self.current_space = name
        return result

    def get_tags(self) -> TestResult:
        """Get list of tags."""
        return self.execute("SHOW TAGS")

    def get_edges(self) -> TestResult:
        """Get list of edges."""
        return self.execute("SHOW EDGES")

    def get_indexes(self) -> TestResult:
        """Get list of indexes."""
        return self.execute("SHOW INDEXES")

    def begin_transaction(self) -> TestResult:
        """Begin a transaction."""
        return self.execute("BEGIN")

    def commit_transaction(self) -> TestResult:
        """Commit current transaction."""
        return self.execute("COMMIT")

    def rollback_transaction(self) -> TestResult:
        """Rollback current transaction."""
        return self.execute("ROLLBACK")

    def wait_for_server(self, timeout: int = 60) -> bool:
        """Wait for server to be ready."""
        start = time.time()
        while time.time() - start < timeout:
            if self.is_connected():
                return True
            time.sleep(1)
        return False


class TestDataLoader:
    """Helper class to load test data from generated GQL files."""

    def __init__(self, client: GraphDBClient):
        self.client = client

    def load_from_file(self, filepath: str, batch_size: int = 10) -> Dict[str, Any]:
        """Load test data from a GQL file in batches."""
        results = {
            "total": 0,
            "success": 0,
            "failed": 0,
            "errors": []
        }

        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()

        # Parse statements
        statements = []
        current = []
        for line in content.split('\n'):
            line = line.strip()
            if not line or line.startswith('--'):
                continue

            current.append(line)
            if line.endswith(';'):
                stmt = ' '.join(current).rstrip(';').strip()
                if stmt and not stmt.startswith('RETURN'):
                    statements.append(stmt)
                current = []

        if current:
            stmt = ' '.join(current).rstrip(';').strip()
            if stmt and not stmt.startswith('RETURN'):
                statements.append(stmt)

        # Execute in batches
        batch = []
        for stmt in statements:
            batch.append(stmt)

            if len(batch) >= batch_size:
                self._execute_batch(batch, results)
                batch = []

        if batch:
            self._execute_batch(batch, results)

        return results

    def _execute_batch(self, statements: List[str], results: Dict[str, Any]):
        """Execute a batch of statements."""
        for stmt in statements:
            results["total"] += 1
            result = self.client.execute(stmt)

            if result.success:
                results["success"] += 1
            else:
                results["failed"] += 1
                results["errors"].append({
                    "statement": stmt[:100],
                    "error": result.error
                })


def create_client_from_env() -> GraphDBClient:
    """Create client from environment variables."""
    import os
    host = os.environ.get("GRAPHDB_HOST", "127.0.0.1")
    port = int(os.environ.get("GRAPHDB_PORT", "9758"))
    return GraphDBClient(host=host, port=port)


if __name__ == "__main__":
    # Simple test
    client = GraphDBClient()
    if client.connect():
        print("Connected to GraphDB")

        result = client.execute("SHOW SPACES")
        print(f"Spaces: {result.data}")

        client.disconnect()
    else:
        print("Failed to connect")
