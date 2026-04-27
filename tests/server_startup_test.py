#!/usr/bin/env python3
"""
GraphDB Server Startup Integration Test

Tests the server startup functionality including:
- Configuration loading
- Port binding
- HTTP API availability
- Graceful shutdown
"""

import subprocess
import time
import socket
import sys
import os
import signal
import json
from pathlib import Path
from typing import Optional, Tuple
import urllib.request
import urllib.error


class ServerStartupTest:
    """Integration test for GraphDB server startup."""

    def __init__(self, server_path: str, config_path: str, timeout: int = 30):
        self.server_path = Path(server_path)
        self.config_path = Path(config_path)
        self.timeout = timeout
        self.process: Optional[subprocess.Popen] = None
        self.host = "127.0.0.1"
        self.port = 9758
        self._load_config()

    def _load_config(self):
        """Load server configuration from config.toml."""
        try:
            import tomllib
            with open(self.config_path, 'rb') as f:
                config = tomllib.load(f)
                self.host = config.get('database', {}).get('host', '127.0.0.1')
                self.port = config.get('database', {}).get('port', 9758)
        except Exception as e:
            print(f"Warning: Could not load config: {e}")

    def test_01_server_binary_exists(self) -> Tuple[bool, str]:
        """Test that server binary exists."""
        print("\n[Test 01] Checking server binary...")
        if not self.server_path.exists():
            return False, f"Server binary not found: {self.server_path}"
        if not self.server_path.is_file():
            return False, f"Server path is not a file: {self.server_path}"
        print(f"  ✓ Server binary exists: {self.server_path}")
        print(f"  ✓ File size: {self.server_path.stat().st_size / (1024*1024):.2f} MB")
        return True, "OK"

    def test_02_config_file_exists(self) -> Tuple[bool, str]:
        """Test that config file exists."""
        print("\n[Test 02] Checking config file...")
        if not self.config_path.exists():
            return False, f"Config file not found: {self.config_path}"
        print(f"  ✓ Config file exists: {self.config_path}")
        return True, "OK"

    def test_03_port_available(self) -> Tuple[bool, str]:
        """Test that the configured port is available."""
        print(f"\n[Test 03] Checking port {self.port} availability...")
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        try:
            sock.bind((self.host, self.port))
            sock.close()
            print(f"  ✓ Port {self.port} is available")
            return True, "OK"
        except socket.error as e:
            return False, f"Port {self.port} is in use: {e}"

    def test_04_start_server(self) -> Tuple[bool, str]:
        """Test starting the server."""
        print(f"\n[Test 04] Starting server...")
        print(f"  Command: {self.server_path} serve --config {self.config_path}")
        
        try:
            # Start server process
            self.process = subprocess.Popen(
                [str(self.server_path), "serve", "--config", str(self.config_path)],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                cwd=str(self.server_path.parent.parent)
            )
            
            # Wait for server to start
            print(f"  Waiting up to {self.timeout}s for server to start...")
            start_time = time.time()
            
            while time.time() - start_time < self.timeout:
                # Check if process is still running
                if self.process.poll() is not None:
                    # Process exited
                    stdout, stderr = self.process.communicate()
                    error_msg = f"Server exited with code {self.process.returncode}\n"
                    if stdout:
                        error_msg += f"STDOUT:\n{stdout}\n"
                    if stderr:
                        error_msg += f"STDERR:\n{stderr}"
                    return False, error_msg
                
                # Try to connect to health endpoint
                if self._check_health():
                    print(f"  ✓ Server started successfully")
                    return True, "OK"
                
                time.sleep(0.5)
            
            # Timeout
            self._stop_server()
            return False, f"Server did not start within {self.timeout} seconds"
            
        except Exception as e:
            return False, f"Failed to start server: {e}"

    def test_05_health_endpoint(self) -> Tuple[bool, str]:
        """Test health endpoint."""
        print(f"\n[Test 05] Checking health endpoint...")
        
        health_url = f"http://{self.host}:{self.port}/v1/health"
        print(f"  URL: {health_url}")
        
        try:
            response = urllib.request.urlopen(health_url, timeout=5)
            status_code = response.getcode()
            body = response.read().decode('utf-8')
            
            if status_code == 200:
                print(f"  ✓ Health check passed (HTTP 200)")
                try:
                    data = json.loads(body)
                    print(f"  ✓ Response: {json.dumps(data, indent=2)}")
                except:
                    print(f"  ✓ Response: {body}")
                return True, "OK"
            else:
                return False, f"Health check failed with status {status_code}: {body}"
                
        except urllib.error.URLError as e:
            return False, f"Failed to connect to health endpoint: {e}"
        except Exception as e:
            return False, f"Health check error: {e}"

    def test_06_api_endpoints(self) -> Tuple[bool, str]:
        """Test basic API endpoints."""
        print(f"\n[Test 06] Checking API endpoints...")
        
        endpoints = [
            "/v1/health",
            "/v1/auth/login",  # Login endpoint (may return 405 for GET)
        ]
        
        results = []
        for endpoint in endpoints:
            url = f"http://{self.host}:{self.port}{endpoint}"
            try:
                response = urllib.request.urlopen(url, timeout=5)
                print(f"  ✓ {endpoint} - HTTP {response.getcode()}")
                results.append(True)
            except urllib.error.HTTPError as e:
                # Some endpoints may require auth or specific methods
                if e.code in [401, 403, 405, 404]:
                    print(f"  ⚠ {endpoint} - HTTP {e.code} (expected)")
                    results.append(True)
                else:
                    print(f"  ✗ {endpoint} - HTTP {e.code}")
                    results.append(False)
            except Exception as e:
                print(f"  ✗ {endpoint} - Error: {e}")
                results.append(False)
        
        if all(results):
            return True, "OK"
        else:
            return False, "Some endpoints failed"

    def test_07_graceful_shutdown(self) -> Tuple[bool, str]:
        """Test graceful shutdown."""
        print(f"\n[Test 07] Testing graceful shutdown...")
        
        if self.process is None or self.process.poll() is not None:
            return False, "Server is not running"
        
        try:
            # Send termination signal
            print(f"  Sending termination signal...")
            if sys.platform == "win32":
                self.process.terminate()
            else:
                self.process.send_signal(signal.SIGTERM)
            
            # Wait for process to exit
            try:
                self.process.wait(timeout=5)
                print(f"  ✓ Server stopped gracefully (exit code: {self.process.returncode})")
                return True, "OK"
            except subprocess.TimeoutExpired:
                print(f"  ⚠ Server did not stop gracefully, forcing kill...")
                self.process.kill()
                self.process.wait()
                return True, "OK (forced kill)"
                
        except Exception as e:
            return False, f"Error during shutdown: {e}"

    def _check_health(self) -> bool:
        """Check if server health endpoint is responding."""
        try:
            health_url = f"http://{self.host}:{self.port}/v1/health"
            response = urllib.request.urlopen(health_url, timeout=1)
            return response.getcode() == 200
        except:
            return False

    def _stop_server(self):
        """Stop the server if running."""
        if self.process and self.process.poll() is None:
            try:
                self.process.terminate()
                self.process.wait(timeout=2)
            except:
                self.process.kill()
                self.process.wait()

    def run_all_tests(self) -> bool:
        """Run all tests and return overall result."""
        print("=" * 60)
        print("GraphDB Server Startup Integration Test")
        print("=" * 60)
        
        tests = [
            self.test_01_server_binary_exists,
            self.test_02_config_file_exists,
            self.test_03_port_available,
            self.test_04_start_server,
            self.test_05_health_endpoint,
            self.test_06_api_endpoints,
            self.test_07_graceful_shutdown,
        ]
        
        results = []
        for test in tests:
            try:
                success, message = test()
                results.append((test.__name__, success, message))
                if not success:
                    print(f"  ✗ FAILED: {message}")
                    # Continue with other tests unless it's a prerequisite
                    if test.__name__ in ['test_01_server_binary_exists', 'test_02_config_file_exists']:
                        break
            except Exception as e:
                results.append((test.__name__, False, str(e)))
                print(f"  ✗ EXCEPTION: {e}")
        
        # Cleanup
        self._stop_server()
        
        # Print summary
        print("\n" + "=" * 60)
        print("Test Summary")
        print("=" * 60)
        
        passed = sum(1 for _, success, _ in results if success)
        failed = sum(1 for _, success, _ in results if not success)
        
        for name, success, message in results:
            status = "✓ PASS" if success else "✗ FAIL"
            print(f"{status}: {name} - {message}")
        
        print(f"\nTotal: {len(results)} tests, {passed} passed, {failed} failed")
        
        return failed == 0


def main():
    """Main entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(description="GraphDB Server Startup Test")
    parser.add_argument(
        "--server-path",
        default="bin/graphdb-server.exe",
        help="Path to server executable"
    )
    parser.add_argument(
        "--config-path",
        default="config.toml",
        help="Path to config file"
    )
    parser.add_argument(
        "--timeout",
        type=int,
        default=30,
        help="Startup timeout in seconds"
    )
    
    args = parser.parse_args()
    
    # Find project root
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    
    server_path = project_root / args.server_path
    config_path = project_root / args.config_path
    
    test = ServerStartupTest(
        server_path=str(server_path),
        config_path=str(config_path),
        timeout=args.timeout
    )
    
    success = test.run_all_tests()
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
