#!/usr/bin/env python3
"""Debug script to check detailed error messages."""

import sys
sys.path.insert(0, 'tests/e2e')

from graphdb_client import GraphDBClient

def debug_test():
    # Use correct credentials from config.toml
    client = GraphDBClient()
    
    # Connect
    print("Connecting to server...")
    if not client.connect():
        print("Failed to connect!")
        return
    print("Connected successfully!")
    print(f"Session ID: {client.session_id}")
    
    # Test 1: Show spaces
    print("\n[Test 1] SHOW SPACES")
    result = client.execute("SHOW SPACES")
    print(f"  Success: {result.success}")
    print(f"  Data: {result.data}")
    print(f"  Error: {result.error}")
    
    # Test 2: Create space
    space_name = "debug_test_space"
    print(f"\n[Test 2] DROP SPACE IF EXISTS {space_name}")
    result = client.execute(f"DROP SPACE IF EXISTS {space_name}")
    print(f"  Success: {result.success}")
    print(f"  Error: {result.error}")
    
    print(f"\n[Test 3] CREATE SPACE {space_name}")
    result = client.execute(f"CREATE SPACE {space_name} (vid_type=STRING)")
    print(f"  Success: {result.success}")
    print(f"  Error: {result.error}")
    
    # Test 4: Use space
    print(f"\n[Test 4] USE {space_name}")
    result = client.execute(f"USE {space_name}")
    print(f"  Success: {result.success}")
    print(f"  Error: {result.error}")
    
    # Test 5: Create tag
    print("\n[Test 5] CREATE TAG person(name: STRING)")
    result = client.execute("CREATE TAG person(name: STRING)")
    print(f"  Success: {result.success}")
    print(f"  Error: {result.error}")
    
    # Test 6: Show tags
    print("\n[Test 6] SHOW TAGS")
    result = client.execute("SHOW TAGS")
    print(f"  Success: {result.success}")
    print(f"  Data: {result.data}")
    print(f"  Error: {result.error}")
    
    # Test 7: Insert vertex
    print("\n[Test 7] INSERT VERTEX person(name) VALUES 'p1': ('Alice')")
    result = client.execute("INSERT VERTEX person(name) VALUES 'p1': ('Alice')")
    print(f"  Success: {result.success}")
    print(f"  Error: {result.error}")
    
    # Test 8: Fetch vertex
    print("\n[Test 8] FETCH PROP ON person 'p1'")
    result = client.execute("FETCH PROP ON person 'p1'")
    print(f"  Success: {result.success}")
    print(f"  Data: {result.data}")
    print(f"  Error: {result.error}")
    
    # Cleanup
    print(f"\n[Cleanup] DROP SPACE {space_name}")
    result = client.execute(f"DROP SPACE {space_name}")
    print(f"  Success: {result.success}")
    print(f"  Error: {result.error}")
    
    client.disconnect()
    print("\nDebug test completed!")

if __name__ == "__main__":
    debug_test()
