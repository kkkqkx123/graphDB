# GraphDB Project Context

## Language

Always use English in code, comments, logging, error info or other string literal. Use Chinese in docs (except code block)
**Never use any Chinese in any code files or code block.**

## Coding Standards

**Security Assurance**
Always avoid the use of unwrap. In testing, substitute with expect.
Refrain from using unsafe methods except where directly involving low-level operations.
All instances of unsafe usage must be explicitly documented in the unsafe.md file within the docs\archive directory.

**Type Design Guidelines**
Minimise the use of dynamic dispatch forms such as `dyn`, always prioritising deterministic types.
All instances of dynamic dispatch must be explicitly documented in the `dynamic.md` file within the `docs\archive` directory.

## Language

Always use English in code, comments, logging, error info. Use Chinese in docs
**Never use any Chinese in any code files.**

## Project Overview

This is a graph database project reimplemented in Rust, focusing on local single-node deployment scenarios. Unlike the original distributed NebulaGraph, this project removes distributed capabilities and significantly reduces external dependencies, aiming to provide a lightweight, high-performance graph database solution for personal use and small-scale applications.

Key Features:

- Single-node architecture, eliminating distributed complexity
- Written in Rust, ensuring memory safety and concurrency safety
- Minimal external dependencies, leveraging the Rust ecosystem
- Generates a single executable file for straightforward deployment
- Supports fundamental graph database functionality (nodes, edges, properties)

## Architecture

The codebase is organized into several main components:

- `src` - Rust graphDB src director
- `src/core` - core data structure and type definition
- `src/storage` - storage engine
- `src/query` - query engine and parser
- `src/transaction` - transaction management
- `src/index` - index system
- `src/api` - API interfaces layer
- `src/utils` - Utility functions and helpers
- `src/config` - Configuration management

outside crates:

- `crates/inversearch` - Inverted search engine
- `crates/bm25` - BM25 search engine
- `crates/qdrant-client` - HTTP client for qdrant vector database
- `./graphdb-cli` - HTTP CLI client for graphDB(completely independent)

## Key Directories and Files

- `graphDB/Cargo.toml` - Project dependencies and configuration
- `graphDB/src/lib.rs` - Library entry point
- `graphDB/src/main.rs` - Executable entry point

## Building and Running

The graphDB project utilises Cargo as its build system. To build the project:

1. **Prerequisites**:

- rustc: 1.88.0
- cargo: 1.88.0

2. **Compile check**

```shell
# full compile check
& 'D:\softwares\Visual Studio\Common7\Tools\Launch-VsDevShell.ps1'; cargo clippy --all-targets --all-features
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
   cargo test --lib -- --nocapture # Run lib tests
   cargo test --test '*' -- --nocapture # Run integration tests
   cargo test <test_name> # Run specific test(s) matching pattern
   cargo test --test <integration_test_file> # Run specific integration test
   ```

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
