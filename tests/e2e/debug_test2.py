#!/usr/bin/env python3
"""Debug script to check detailed error messages."""

import requests
import json

base_url = "http://127.0.0.1:9758"

# Test health endpoint
print("Testing health endpoint...")
try:
    response = requests.get(f"{base_url}/v1/health", timeout=5)
    print(f"  Status: {response.status_code}")
    print(f"  Response: {response.text}")
except Exception as e:
    print(f"  Error: {e}")

# Test auth endpoint
print("\nTesting auth endpoint...")
try:
    response = requests.post(
        f"{base_url}/v1/auth/login",
        json={"username": "root", "password": "root"},
        timeout=5
    )
    print(f"  Status: {response.status_code}")
    print(f"  Response: {response.text}")
except Exception as e:
    print(f"  Error: {e}")

# Test query endpoint
print("\nTesting query endpoint...")
try:
    # First authenticate
    auth_response = requests.post(
        f"{base_url}/v1/auth/login",
        json={"username": "root", "password": "root"},
        timeout=5
    )
    session_id = auth_response.json().get("session_id")
    print(f"  Session ID: {session_id}")
    
    # Then execute query
    response = requests.post(
        f"{base_url}/v1/query",
        json={"query": "SHOW SPACES", "session_id": session_id, "parameters": {}},
        headers={"X-Session-ID": str(session_id)},
        timeout=5
    )
    print(f"  Status: {response.status_code}")
    print(f"  Response: {response.text}")
except Exception as e:
    print(f"  Error: {e}")

print("\nDebug test completed!")
