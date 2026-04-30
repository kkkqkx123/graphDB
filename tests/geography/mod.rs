//! Geography Integration Tests Module
//!
//! Test coverage:
//! - Basic operations - create, validate, convert geometries
//! - Geometry types - all 6 geometry types with full operations
//! - Spatial functions - WKT/GeoJSON conversion, measurements, spatial operations
//!
//! Test modules:
//! - common: Common test utilities and helper functions
//! - basic: Basic geometry operations (20 tests)
//! - geometry_types: Tests for each geometry type (20 tests)
//! - spatial_functions: Tests for spatial functions (20 tests)
//! - format_conversion: Tests for WKT and GeoJSON conversion (8 tests)

mod common;
mod basic;
mod geometry_types;
mod spatial_functions;
mod format_conversion;
