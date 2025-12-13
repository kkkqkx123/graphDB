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
- `src` - Rust graphDB src director
- `src/core` - core data structure and type definition
- `src/storage` - storage engine
- `src/query` - query engine and parser
- `src/transaction` - transaction management
- `src/index` - index system
- `src/api` - API interfaces layer
- `src/utils` - Utility functions and helpers
- `src/config` - Configuration management


## Key Directories and Files

- `graphDB/Cargo.toml` - Project dependencies and configuration
- `graphDB/src/lib.rs` - Library entry point
- `graphDB/src/main.rs` - Executable entry point

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
   cd graphDB
   cargo build                 # Debug build (development)
   cargo build --release       # Release build (optimized for performance)
   ```

3. **Type check and compile check**
```bash
cargo check --message-format=short # Default Type check
cargo check # Detailed Type check(Only use it when you need that. when use this, always add filter logic, like `cargo check 2>&1 | Select-String "error\[E" | Select-Object -First 10`)
```   

4. **Run commands**:
   ```bash
   # Start database service
   cargo run --release -- serve --config config.toml

   # Execute query directly
   cargo run --release -- query --query "MATCH (n) RETURN n LIMIT 10"
   ```

## Development Conventions

- **Coding Style**: Employ Rust standard formatting (`cargo fmt`) and adhere to Rust naming conventions
- **IDE Integration**: Utilise Rust-compatible editors such as VS Code (rust-analyzer) or IntelliJ IDEA
- **Testing**: Employ Rust's built-in testing framework (`cargo test`), writing integration tests within the `tests/` directory
- **Code Structure**: Adopt a modular design following Rust conventions

## Testing

The project includes a comprehensive test suite utilising Rust's standard testing framework:

1. **Running tests**:
   ```bash
   cargo test # Run all tests
   cargo test -- --test-threads=1 # Run tests sequentially (useful for debugging)
   cargo test <test_name> # Run specific test(s) matching pattern
   cargo test --test <integration_test_file> # Run specific integration test
   cargo test --release  # Run tests in release mode
   cargo test --doc # Run documentation examples as tests
   ```
It is not recommended to run all tests in one time. 

2. **Test organization**:
   - Unit tests: Located in the same file as the code being tested, marked with `#[cfg(test)]`
   - Unit tests when original file is too large: Add individual test.rs, and add it to `mod.rs`
   - Integration tests: Located in the `tests/` directory
   - Benchmarks: Located in the `benches/` directory
   - Tests relevent to storage: use `pub mod test_config;`, don't use individual path
   
## Additional Notes

- The project utilises Rust version 2021, employing the ownership system to ensure memory safety
- Does not include distributed functionality, focusing instead on single-machine performance and simplicity
- The architecture aims to minimise external dependencies, leveraging the security and performance of the Rust ecosystem
- Due to the excessive length of cargo command output, always execute commands in the format: `<cargo command> 2>&1 | Select-Object -Last 60`(or more line. When use `cargo check`, use `-First` instead to capture errors first)