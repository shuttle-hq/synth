# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Synth is a declarative data generator written in Rust. It generates realistic data from JSON schema definitions, supporting import/export to PostgreSQL, MySQL, and MongoDB. Users define "namespaces" (directories of JSON schema files) that describe data models with types, constraints, and relationships.

## Build Commands

```bash
# Build (requires Rust nightly - see rust-toolchain)
cargo build

# Run all tests
cargo test

# Run a single test
cargo test test_name

# Run tests for a specific crate
cargo test -p synth-core
cargo test -p synth-gen
cargo test -p synth

# Clippy (required before commits)
cargo clippy --tests --all-targets --all-features

# Format
cargo fmt

# Run the CLI
cargo run -- generate <namespace_dir> --size <n>
cargo run -- import <namespace_dir> --from <uri>

# Benchmarks
cargo bench -p synth

# License/advisory checks
cargo deny check
```

## Architecture

### Workspace Crates

- **`gen/` (synth-gen)**: Low-level generator primitives. Defines the `Generator` trait and combinators for composing random value producers. The `shared` feature enables cross-generator state sharing; `faker` enables fake data integration.

- **`core/` (synth-core)**: Schema model and compilation. Key subsystems:
  - `schema/` - The data model: content types (strings, numbers, dates, arrays, objects, one_of, series, unique), namespace resolution, inference engine (auto-detecting types/distributions from sample data), and optionalisation.
  - `compile/` - Compiles schemas into executable generator graphs, resolving cross-collection references (`link.rs`) and managing compilation state.
  - `graph/` - Runtime graph of generators for each JSON type, handling the actual data production.

- **`synth/` (synth)**: CLI application and database integration.
  - `cli/` - Command handling (generate, import, version), store management (filesystem-based namespace persistence).
  - `datasource/` - Database connectors: `postgres_datasource.rs`, `mysql_datasource.rs`, plus shared relational logic in `relational_datasource.rs`. MongoDB handled in `cli/mongo.rs`.
  - `sampler.rs` - Orchestrates generation by walking compiled graphs.

- **`test_macros/`**: Proc macros for test utilities.

### Key Data Flow

1. **Import**: Database -> `datasource` introspects schema/data -> `schema::inference` builds content model -> JSON schema files written to namespace dir
2. **Generate**: JSON schema files read from namespace dir -> `compile` resolves references and builds generator graph -> `sampler` walks graph producing JSON -> output to stdout or database via `export`

### Nightly Rust

The project requires **Rust nightly** (configured in `rust-toolchain`). Uses `box_patterns`, `error_iter`, `try_blocks`, and `concat_idents` (though `concat_idents` was removed in Rust 1.90 and needs migration to `${concat(..)}` metavariable expressions).

## Commit Convention

Uses [Angular Commit Guidelines](https://github.com/angular/angular/blob/master/CONTRIBUTING.md#commit). Commits should be squashed before merging to master.

## Testing

- Unit tests are co-located within source files
- Integration tests in `synth/tests/` with example schemas in `synth/tests/examples/`
- Database-specific integration tests require running database instances (see `synth/testing_harness/` for setup scripts for PostgreSQL, MySQL, MongoDB)
- Error scenario tests via `synth/testing_harness/errors/` with an `e2e.sh` end-to-end script
- CI workflows test each database connector separately (`.github/workflows/synth-{postgres,mysql,mongo}.yml`)

## Configuration

- `deny.toml` - cargo-deny configuration for license and advisory checks
- `rustfmt.toml` - Rust 2018 edition formatting
- `default.nix` / `shell.nix` - Nix development environment
