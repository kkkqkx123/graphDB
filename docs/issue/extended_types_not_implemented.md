# Issue: Extended Types (Geography, Vector, FullText) Not Fully Implemented

## Problem Description

The extended types functionality including Geography (GEO), Vector, and FullText search are not fully implemented in the GraphDB server. Most tests for these features fail with various errors indicating missing or incomplete implementations.

## Affected Test Suites

- `test_extended_types.TestGeography`
- `test_extended_types.TestVector`
- `test_extended_types.TestFullText`

## Error Summary

### Geography Tests

| Test | Status | Error |
|------|--------|-------|
| test_geo_001_point_creation | FAIL | Various errors |
| test_geo_002_wkt_creation | FAIL | Various errors |
| test_geo_003_distance_calculation | FAIL | Query execution failed |
| test_geo_004_within_distance | FAIL | Query execution failed |
| test_geo_005_explain_geography_query | OK | - |

### Vector Tests

| Test | Status | Error |
|------|--------|-------|
| test_vec_001_vector_insertion | FAIL | Vertex already exists / Type errors |
| test_vec_002_cosine_similarity | FAIL | Query execution failed |
| test_vec_003_filtered_vector_search | FAIL | Query execution failed |
| test_vec_004_explain_vector_query | FAIL | Query execution failed |

### FullText Tests

| Test | Status | Error |
|------|--------|-------|
| test_ft_001_fulltext_index_creation | FAIL | Query execution failed |
| test_ft_002_basic_search | FAIL | Query execution failed |
| test_ft_003_boolean_search | FAIL | Query execution failed |
| test_ft_004_explain_fulltext | FAIL | Query execution failed |

## Root Cause Analysis

### Geography Issues

1. **ST_Point Function**: The `ST_Point` function for creating geographic points may not be implemented
2. **ST_GeogFromText Function**: The `ST_GeogFromText` function for parsing WKT format may not be implemented
3. **ST_Distance Function**: Distance calculation between geographic points may not be implemented
4. **Geographic Data Types**: The `GEOGRAPHY` data type may not be fully supported

### Vector Issues

1. **Vector Data Type**: The `VECTOR` data type may not be fully supported
2. **Cosine Similarity**: Vector similarity search functions may not be implemented
3. **Vector Index**: Vector indexing for efficient similarity search may not be implemented
4. **Data Insertion**: Inserting vertices with vector properties fails

### FullText Issues

1. **FullText Index Creation**: `CREATE FULLTEXT INDEX` syntax may not be supported
2. **FullText Search**: `SEARCH` or `MATCH` with fulltext may not be implemented
3. **Boolean Queries**: Complex boolean queries in fulltext search may not be supported
4. **Text Analysis**: Text tokenization and analysis may not be implemented

## Verification Method

Run the following tests to verify the issues:

```bash
cd tests/e2e
python -m pytest test_extended_types.py::TestGeography -v
python -m pytest test_extended_types.py::TestVector -v
python -m pytest test_extended_types.py::TestFullText -v
```

Or run the full test suite:

```bash
python run_tests.py
```

## Test Data and Queries

### Geography Test Data

```sql
CREATE SPACE geo_test (vid_type=STRING)
USE geo_test
CREATE TAG location(name: STRING, coord: GEOGRAPHY)

-- Failing queries:
INSERT VERTEX location(name, coord) VALUES "loc1": ("Beijing", ST_Point(116.4, 39.9))
INSERT VERTEX location(name, coord) VALUES "loc2": ("Shanghai", ST_GeogFromText("POINT(121.5 31.2)"))
RETURN ST_Distance(ST_Point(116.4, 39.9), ST_Point(121.5, 31.2))
```

### Vector Test Data

```sql
CREATE SPACE vector_test (vid_type=STRING, vector_dimension=128)
USE vector_test
CREATE TAG item(name: STRING, embedding: VECTOR)

-- Failing queries:
INSERT VERTEX item(name, embedding) VALUES "item1": ("test", [0.1, 0.2, 0.3, ...])
SEARCH VECTOR ON item(embedding) TOP 5
```

### FullText Test Data

```sql
CREATE SPACE ft_test (vid_type=STRING)
USE ft_test
CREATE TAG document(title: STRING, content: STRING)
CREATE FULLTEXT INDEX idx_content ON document(content)

-- Failing queries:
SEARCH FULLTEXT ON document WHERE content CONTAINS "keyword"
```

## Related Code

The test code is located in:
- `tests/e2e/test_extended_types.py`

## Suggested Fixes

### Geography

1. Implement `ST_Point` function for creating geographic points
2. Implement `ST_GeogFromText` function for parsing WKT format
3. Implement `ST_Distance` function for calculating distances
4. Add proper `GEOGRAPHY` data type support

### Vector

1. Implement `VECTOR` data type with configurable dimensions
2. Implement vector similarity functions (cosine, euclidean)
3. Implement vector indexing (HNSW, IVF, etc.)
4. Add vector search query syntax

### FullText

1. Implement `CREATE FULLTEXT INDEX` syntax
2. Implement text analysis and tokenization
3. Implement fulltext search query syntax
4. Add boolean query support (AND, OR, NOT)

## Priority

Medium - These are advanced features that enhance the database but are not core functionality

## Related Components

- `src/query/functions/` - Query functions (ST_Point, ST_Distance, etc.)
- `src/query/datatypes/` - Data type definitions
- `src/index/` - Index system (for vector and fulltext indexes)
- `crates/inversearch/` - Inverted search engine (for fulltext)
- `crates/bm25/` - BM25 search engine (for fulltext)
- `crates/qdrant-client/` - Vector database client

## Dependencies

The project includes external crates for extended functionality:
- `crates/inversearch` - Inverted search engine
- `crates/bm25` - BM25 search engine
- `crates/qdrant-client` - HTTP client for qdrant vector database

These crates may need to be integrated properly with the main GraphDB server.
