//! Geography Integration Tests
//!
//! Test coverage:
//! - Geometry types - Point, LineString, Polygon, MultiPoint, MultiLineString, MultiPolygon
//! - Spatial functions - construction, conversion, properties, measurements, operations
//! - Format support - WKT (Well-Known Text), GeoJSON
//! - Spatial relations - intersects, contains, within, covers, crosses, touches, overlaps
//! - Edge cases - invalid coordinates, empty geometries, degenerate cases
//! - Error handling - type errors, null handling, invalid inputs
//!
//! Test modules:
//! - basic: Basic geometry operations (20 tests)
//! - geometry_types: Tests for each geometry type (20 tests)
//! - spatial_functions: Tests for spatial functions (30 tests)
//! - format_conversion: Tests for WKT and GeoJSON conversion (8 tests)
//! - spatial_relations: Tests for spatial relationship functions (8 tests)
//! - measurements: Tests for measurement functions (10 tests)
//! - edge_cases: Tests for edge cases and error handling (15 tests)
//!
//! Total: ~110 test cases

mod common;
mod geography;
