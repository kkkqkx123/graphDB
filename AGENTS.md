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

Always use **Simplified Chinese** in code, comments and docs. Using English in logging, error info.

## Project Overview

This is a graph database project reimplemented in Rust, focusing on local single-node deployment scenarios. Unlike the original distributed NebulaGraph, this project removes distributed capabilities and significantly reduces external dependencies, aiming to provide a lightweight, high-performance graph database solution for personal use and small-scale applications.

Key Features:
- Single-node architecture, eliminating distributed complexity
- Written in Rust, ensuring memory safety and concurrency safety
- Minimal external dependencies, leveraging the Rust ecosystem
- Generates a single executable file for straightforward deployment
- Supports fundamental graph database functionality (nodes, edges, properties)

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
   ```shell
   cd graphDB
   cargo build                 # Debug build (development)
   cargo build --release       # Release build (optimized for performance)
   ```

3. **Type check and compile check**

analyze_cargo is a cli tool that automatically runs `cargo test --lib`, categorizes the errors/warnings, and generates a detailed Markdown report.
default output file is `cargo_errors_report.md` in pwd.
Use it instead of `cargo test --lib` or `cargo check`. 

**Usage**
```shell
analyze_cargo
```

**Examples**

```shell
# Default usage
analyze_cargo

# Filter warnings only
analyze_cargo --filter-warnings
```

## Development Conventions

- **Coding Style**: Employ Rust standard formatting (`cargo fmt`) and adhere to Rust naming conventions
- **IDE Integration**: Utilise Rust-compatible editors such as VS Code (rust-analyzer) or IntelliJ IDEA
- **Testing**: Employ Rust's built-in testing framework (`cargo test`), writing integration tests within the `tests/` directory
- **Code Structure**: Adopt a modular design following Rust conventions

## Testing

The project includes a comprehensive test suite utilising Rust's standard testing framework:

1. **Running tests**:
   ```shell
   cargo test # Run all tests
   cargo test --lib -- --nocapture # Run lib tests
   cargo test <test_name> # Run specific test(s) matching pattern
   cargo test --test <integration_test_file> # Run specific integration test
   ```
It is not recommended to run all tests in one time. 

2. **Test organization**:
   - Unit tests: Located in the same file as the code being tested, marked with `#[cfg(test)]`
   - Unit tests when original file is too large: Add individual test.rs, and add it to `mod.rs`
   - Integration tests: Located in the `tests/` directory
   - Benchmarks: Located in the `benches/` directory
   
## Additional Notes

- The project utilises Rust, employing the ownership system to ensure memory safety
- Does not include distributed functionality, focusing instead on single-machine performance and simplicity
- The architecture aims to minimise external dependencies, leveraging the security and performance of the Rust ecosystem
- Use powershell syntax instead of shell