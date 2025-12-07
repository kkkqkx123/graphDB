# GraphDB - Rust-based Graph Database

A lightweight, single-node graph database implemented in Rust, designed for personal and small-scale applications. This project is a Rust rewrite of selected functionality from NebulaGraph, focusing on reducing external dependencies while maintaining core graph database capabilities.

## Overview

GraphDB is a simplified, single-node graph database that runs as a single executable. It's designed for:

- Personal projects and development
- Small-scale applications
- Edge computing scenarios
- Educational purposes

## Features

- **Single Executable**: Entire database runs in a single binary
- **Lightweight**: Minimal external dependencies
- **Graph Operations**: Support for nodes, edges, and properties
- **ACID Transactions**: Basic transaction support
- **Query Language**: GQL (Graph Query Language) support
- **Indexing**: Property and label indexing
- **Safe by Default**: Memory safety through Rust's ownership system

## Architecture

### Modules

The system is organized into several key modules:

- `core`: Core data structures like Node, Edge, and Value types
- `storage`: Storage engine with sled backend
- `query`: Query engine and parser
- `transaction`: Transaction management
- `index`: Indexing system for efficient queries
- `api`: API layer and service management
- `utils`: Utility functions and helpers
- `config`: Configuration management

### Design Decisions

1. **Single Node**: No distributed functionality for simplicity
2. **Rust Implementation**: Memory safety and performance
3. **Sled Storage**: Embedded key-value store for persistence
4. **Minimal Dependencies**: Reduced external dependencies for easier maintenance
5. **Module Separation**: Clean separation of concerns between modules

## Building and Running

### Prerequisites

- Rust 1.88 or higher
- Cargo (comes with Rust)

### Build and Run

```bash
# Clone the repository
git clone <repository-url>
cd graphDB

# Build the project
cargo build --release

# Run the database service
./target/release/graphdb serve --config config.toml

# Or execute a query directly
./target/release/graphdb query --query "CREATE NODE (name='John')"
```

### Configuration

Create a `config.toml` file:

```toml
[database]
host = "127.0.0.1"
port = 9758
storage_path = "data/graphdb"
cache_size = 1000
enable_cache = true
max_connections = 10
transaction_timeout = 30
log_level = "info"
```

## Usage Examples

### Creating Nodes

```
CREATE NODE (name='Alice', age=30) LABELS [Person]
```

### Creating Edges

```
CREATE EDGE FROM 1 TO 2 TYPE 'knows' (since=2020)
```

### Querying

```
MATCH (n:Person) WHERE n.age > 25 RETURN n
```

### Deleting

```
DELETE NODE 1
```

## Project Structure

```
graphDB/
├── Cargo.toml          # Project dependencies and metadata
├── README.md           # This file
├── config/
│   └── default.toml    # Default configuration
├── docs/               # Documentation
├── src/
│   ├── main.rs         # Entry point
│   ├── lib.rs          # Library exports
│   ├── core/           # Core data structures
│   ├── storage/        # Storage engine
│   ├── query/          # Query engine
│   ├── transaction/    # Transaction management
│   ├── index/          # Indexing system
│   ├── api/            # API layer
│   ├── utils/          # Utility functions
│   └── config/         # Configuration management
└── tests/              # Integration tests
```

## Development

### Running Tests

```bash
cargo test
```

### Running with Debug Info

```bash
cargo run -- serve --config config.toml
```

### Linting and Formatting

```bash
# Format code
cargo fmt

# Lint code
cargo clippy
```

## Performance Considerations

- GraphDB uses sled as the underlying storage engine, which provides good performance for single-node workloads
- Indexing is available for frequently queried properties
- The in-memory cache can be configured to improve read performance

## Best Practices

1. **Use Labels**: Labels help organize nodes and improve query performance
2. **Index Properties**: Index frequently queried properties
3. **Batch Operations**: When possible, batch multiple operations in a transaction
4. **Connection Management**: Reuse database connections across operations

## Dependencies

GraphDB uses the following key dependencies:

- `tokio`: Async runtime
- `sled`: Embedded key-value store
- `serde`: Serialization/deserialization
- `clap`: Command-line argument parsing
- `log/env_logger`: Logging framework
- `thiserror/anyhow`: Error handling
- `bincode`: Binary serialization
- `toml`: Configuration file format

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass
6. Submit a pull request

## License

This project is licensed under the Apache 2.0 License - see the LICENSE file for details.

## Acknowledgments

- The sled team for the excellent embedded database
- The Rust community for the robust ecosystem
- NebulaGraph for the original inspiration