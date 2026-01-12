# GraphDB Project Context

## Coding Standards

**Security Assurance**
Always avoid the use of unwrap. In testing, substitute with expect.
Refrain from using unsafe methods except where directly involving low-level operations.
All instances of unsafe usage must be explicitly documented in the unsafe.md file within the docs\archive directory.

**Type Design Guidelines**
Minimise the use of dynamic dispatch forms such as `dyn`, always prioritising deterministic types.
All instances of dynamic dispatch must be explicitly documented in the `dynamic.md` file within the `docs\archive` directory.

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

2. **Build commands**:
   ```bash
   cd graphDB
   cargo build                 # Debug build (development)
   cargo build --release       # Release build (optimized for performance)
   ```

3. **Type check and compile check**

analyze_cargo is a cli tool that automatically runs `cargo test --lib`, categorizes the errors/warnings, and generates a detailed Markdown report.
default output file is `cargo_errors_report.md` in pwd.
Use it instead of `cargo test --lib` or `cargo check`. 

**Usage**
```bash
analyze_cargo
```

**Options**
- `--output <file>`: Specify output file path (default: cargo_errors_report.md)
- `--filter-warnings`: Filter warnings, only show errors
- `--filter-paths <paths>`: Filter errors by file paths (comma-separated)

**Examples**

```bash
# Default usage
analyze_cargo

# Filter warnings only
analyze_cargo --filter-warnings

# Filter by specific paths(folder or file)
analyze_cargo --filter-paths src/main.rs,src/core
```

4. **Run commands**:
   ```bash
   # Start database service
   cargo run --release -- serve --config config.toml

   # Execute query directly
   cargo run --release -- query --query "MATCH (n) RETURN n LIMIT 10"
   ```

5. **Temporary verify**:
   create a rs file, then:
   ```bash
   rustc <script name>.rs && ./<script name>.exe
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
   - Tests relevent to storage: storage in `data/tests` directory, don't place in any other directory
   
## Additional Notes

- The project utilises Rust version 2021, employing the ownership system to ensure memory safety
- Does not include distributed functionality, focusing instead on single-machine performance and simplicity
- The architecture aims to minimise external dependencies, leveraging the security and performance of the Rust ecosystem