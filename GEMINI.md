# Gemini Code Assistant Context

## Project Overview

This project, `deep_causality`, is a Rust-based monorepo for a computational causality library. It enables fast,
context-aware causal reasoning over complex multi-stage causality models. The library is designed for dynamic systems
where time is not linear, causal rules can change, and context is dynamic.

The core of the library is built on the idea of "Causality is a spacetime-agnostic functional dependency." It uses three
main components:

* **Causaloid:** A self-contained unit of causality.
* **Context:** An explicit environment (a hypergraph) where Causaloids operate.
* **Effect Ethos:** A programmable layer for verifying operational rules.

The project is a monorepo containing several sub-crates, including:

* `deep_causality`: The main crate.
* `deep_causality_algorithms`: Provides algorithms for the library.
* `deep_causality_data_structures`: Provides data structures for the library.
* `deep_causality_macros`: Provides macros for the library.
* `deep_causality_uncertain`: Provides functionality for handling uncertainty.
* `examples`: A collection of example code.
* `ultragraph`: A graph library used as a backend.

## Building and Running

The project uses `make` to simplify the execution of common development tasks. The `makefile` in the root of the project
defines the following commands:

* `make build`: Builds the entire project. This is equivalent to running `cargo build`.
* `make test`: Runs all tests across all crates. This is equivalent to running `cargo test`.
* `make bench`: Runs all benchmarks across all crates.
* `make example`: Runs the example code.
* `make fix`: Fixes linting issues as reported by `clippy`.
* `make format`: Formats all code according to the `cargo fmt` style.
* `make check`: Checks the code base for security vulnerabilities.

## Development Conventions


Building and testing a specific crate is preferred over building the entire project. use

`cargo build -p crate_name`

`cargo test -p crate_name`

After a major code change, format and lint the entire code base:

`make format && make fix`