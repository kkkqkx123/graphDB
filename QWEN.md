# GraphDB Project Context

## Language

Always use **Simplified Chinese** in code, comments and docs

## Project Overview

This is a graph database project reimplemented in Rust, focusing on local single-node deployment scenarios. Unlike the original distributed NebulaGraph, this project removes distributed capabilities and significantly reduces external dependencies, aiming to provide a lightweight, high-performance graph database solution for personal use and small-scale applications.

Key Features:
- Single-node architecture, eliminating distributed complexity
- Written in Rust, ensuring memory safety and concurrency safety
- Minimal external dependencies, leveraging the Rust ecosystem
- Generates a single executable file for straightforward deployment
- Supports fundamental graph database functionality (nodes, edges, properties)

## nebula-graph Architecture

The nebula-graph codebase is organized into several main components:
root path: `nebula-3.8.0`
- `src/clients` - Client libraries for connecting to NebulaGraph
- `src/common` - Common utilities and shared code
- `src/graph` - Graph query engine and execution
- `src/storage` - Storage engine implementation
- `src/meta` - Metadata management
- `src/kvstore` - Key-value store layer
- `src/daemons` - Service daemon implementations
- `src/parser` - Query parsing and processing
- `src/webservice` - Web service interfaces

## Architecture-GraphDB

The new codebase is organized into several main components:
- `graphDB/src` - Rust graphDB src director
- `graphDB/src/core` - core data structure and type definition
- `graphDB/src/storage` - storage engine
- `graphDB/src/query` - query engine and parser
- `graphDB/src/transaction` - transaction management
- `graphDB/src/index` - index system
- `graphDB/src/api` - API interfaces layer
- `graphDB/src/utils` - Utility functions and helpers
- `graphDB/src/config` - Configuration management

## Building and Running

The graphDB project utilises Cargo as its build system. To build the project:

1. **Prerequisites**: 

- rustc: 1.88.0
- cargo: 1.88.0
- rustup:
Default host: x86_64-pc-windows-msvc
rustup home: D:\Source\.rustup
installed toolchains: stable-x86_64-pc-windows-msvc (active, default)
installed targets: x86_64-pc-windows-msvc

2. **Build commands**:
   ```bash
   cd graphDB-rust
   cargo build                 # Debug build (development)
   cargo build --release       # Release build (optimized for performance)
   ```

3. **Run commands**:
   ```bash
   # Start database service
   cargo run --release -- serve --config config.toml

   # Execute query directly
   cargo run --release -- query --query "MATCH (n) RETURN n LIMIT 10"
   ```

4. **Compilation checking and linting**:
   ```bash
   cargo check                         # Quick compilation check without building
   cargo clippy                        # Linting for code quality and best practices
   cargo clippy --fix                  # Automatically fix some linting issues
   cargo fmt                           # Format code according to Rust standards
   cargo fmt -- --check                # Check if code is properly formatted
   ```

5. **Testing commands**:
   ```bash
   cargo test                          # Run all tests
   cargo test -- --nocapture           # Run tests with output visible
   cargo test -- --test-threads=1      # Run tests sequentially (useful for debugging)
   cargo test <test_name>              # Run specific test(s) matching pattern
   cargo test --test <integration_test_file>  # Run specific integration test
   cargo test --release                # Run tests in release mode
   cargo test --doc                    # Run documentation examples as tests
   ```

6. **File-specific operations and filtering**:
   ```bash
   # Check specific file(s)
   cargo check --manifest-path graphDB-rust/Cargo.toml

   # Target specific file in compilation
   cargo build --package <package_name> --lib <specific_file.rs>

   # Filter test results
   cargo test <test_pattern>           # Run tests matching a pattern
   cargo test -- --skip <pattern>      # Skip tests matching a pattern
   cargo test -- <test_filter> --exact # Run only tests with exact name match

   # Compilation for specific targets
   cargo build --target x86_64-unknown-linux-musl  # For static linking
   cargo build --target x86_64-apple-darwin
   cargo build --target x86_64-pc-windows-msvc
   ```

## Development Conventions

- **Coding Style**: Employ Rust standard formatting (`cargo fmt`) and adhere to Rust naming conventions
- **IDE Integration**: Utilise Rust-compatible editors such as VS Code (rust-analyzer) or IntelliJ IDEA
- **Testing**: Employ Rust's built-in testing framework (`cargo test`), writing integration tests within the `tests/` directory
- **Code Structure**: Adopt a modular design following Rust conventions

## File Structure

- `Cargo.toml` - Rust project configuration
- `graphDB-rust/` - Main directory for Rust implementation
- `graphDB-rust/src/` - Rust source code
- `graphDB-rust/tests/` - Integration test files
- `graphDB-rust/benches/` - Performance benchmarks
- `graphDB-rust/examples/` - Example applications
- `docs/` - Documentation (including design analysis)
- `config/` - Configuration file examples
- `target/` - Compiled artifacts (generated during build)

## Key Directories and Files

- `graphDB-rust/Cargo.toml` - Project dependencies and configuration
- `graphDB-rust/src/lib.rs` - Library entry point
- `graphDB-rust/src/main.rs` - Executable entry point
- `docs/rust-architecture-design.md` - Rust architecture design documentation
- `docs/rust-rewrite-dependency-analysis.md` - Dependency analysis documentation

## Testing

The project includes a comprehensive test suite utilising Rust's standard testing framework:

1. **Running tests**:
   ```bash
   cargo test                          # Run all tests
   cargo test -- --nocapture           # Run tests with output visible
   cargo test -- --test-threads=1      # Run tests sequentially (useful for debugging)
   cargo test <test_name>              # Run specific test(s) matching pattern
   cargo test --test <integration_test_file>  # Run specific integration test
   cargo test --release                # Run tests in release mode
   cargo test --doc                    # Run documentation examples as tests
   ```

2. **Test organization**:
   - Unit tests: Located in the same file as the code being tested, marked with `#[cfg(test)]`
   - Integration tests: Located in the `tests/` directory
   - Benchmarks: Located in the `benches/` directory

3. **Test-specific operations**:
   ```bash
   cargo test --workspace              # Run tests across all workspace members
   cargo test --lib                    # Run only library tests (not binary/tests/)
   cargo test --bins                   # Run tests for binary targets only
   cargo test --benches                # Run benchmark tests
   cargo test --features <feature>     # Run tests with specific feature flags
   ```

## Additional Notes

- The project utilises Rust version 2021, employing the ownership system to ensure memory safety
- Does not include distributed functionality, focusing instead on single-machine performance and simplicity
- The architecture aims to minimise external dependencies, leveraging the security and performance of the Rust ecosystem