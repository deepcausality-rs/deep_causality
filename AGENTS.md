# Gemini Code Assistant Context

## Project Overview

This project, `deep_causality`, is a Rust-based monorepo for a computational causality library. It enables fast,
context-aware causal reasoning over complex multi-stage causality models. The library is designed for dynamic systems
where time is not linear, causal rules can change, and context is dynamic.

The core of the library is built on the idea of "Causality is a spacetime-agnostic functional dependency."
It uses three main components:

* **Causaloid:** A self-contained unit of causality.
* **Context:** An explicit environment (a hypergraph) where Causaloids operate.
* **Effect Ethos:** A programmable layer for verifying operational rules.

## Project Structure

The project is a monorepo containing several sub-crates, including:

* `deep_causality`: The main crate.
* `deep_causality_algorithms`: Provides algorithms for the library.
* `deep_causality_data_structures`: Provides data structures for the library.
* `deep_causality_discovery`: A custom DSL for causal discovery.
* `deep_causality_macros`: Provides macros for the library.
* `deep_causality_num`: Numerical traits and utils used in other crates.
* `deep_causality_rand`: Random number generator and statistical distributions used in deep_causality_tensor and other
* `deep_causality_tensor`: A custom tensor type used in deep_causality_algorithms and deep_causality_discovery
* `deep_causality_uncertain`: Provides functionality for handling uncertainty.
* `examples`: A collection of example code.
* `ultragraph`: A graph library used as a backend.


## Building and Running

The project uses `make` to simplify the execution of common development tasks. The `makefile` in the root of the project
defines the following commands:

* `make build`: Builds the entire mono-repo
* `make test`: Tests the entire mono-repo (Slow)
* `make fix`: Fixes linting issues as reported by `clippy`.
* `make format`: Formats all code according to the `cargo fmt` style.
* `make check`: Checks the code base for security vulnerabilities.

## Development Conventions

Building and testing a specific crate is preferred over building the entire project.
Use the following commands by default.

`cargo build -p crate_name`

`cargo test -p crate_name`

After a major code change, format and lint the entire code base:

`make format && make fix`

Only when multiple crate (3 or more) have changed at once, you run:

`make format && make fix` Format and fix lints

`make build`: Builds the entire mono-repo

`make test`: Tests the entire mono-repo

To rebuild and test the entire repo

## Code structure

Each crate adheres to the following base structure

src/errors/mod.rs  # contains each error type in a separate file
src/traits/mod.rs # contains each trait in a separate file
src/type/mod.rs # contains each type in a separate file

As a rule, one type, one file. 

Optional src folders
src/extensions/mod.rs # contains type extensions i.e. a default impl for a trait
src/utils/mod.rs # contains utils

One notable exception is the deep_causality_num crate that uses a different structure
due to the particularities of modelling numerical properties in Rust.

## Test structure

The tests folder replicates the exact src folder structure, for for example:

tests/errors/mod.r  # contains tests for each error type in a separate file
tests/traits/mod.rs # Optional contains tests for each  trait in a separate file
tests/type/mod.rs # contains tests for each  type in a separate file

Test files replicate the source file name with an appended _tests. For example,
a source file

`src/errors/normal_error/normal_error.rs`

is matched with the test file under the tests folder:

`test/errors/normal_error/normal_error_tests.rs`

Test files do not contain enclosing test modules, just the tests itself. 

Shared utils used for testing are actually stored in the src tree under:

`src/utils_tests/mod.rs # contains utils`

The reason is, Bazel cannot access util files from within the test folder, but it
can access the full src folder during testing. As a result, test utils have to be fully
tested to count towards the code coverage score.

The usage of a prelude file is prohibited. 

## Code export

* All public errors, traits, and errors are exported from src/lib.rs
* internal modules remain private at the root level.

## Code import

When importing from a crate, always import directly from the root level, for example:

`use deep_causality_discovery::{ConsoleFormatter, ProcessAnalysis, ProcessResultFormatter};`

## Code conventions

Field visibility:
* Public types: All fields will be private, and access will be provided through
  constructors, getters, and setters as appropriate.
* Private or temporary types: Public fields may be used, provided they do not
  leak outside their defined scope.

Static Dispatch:
* Use static dispatch 
* Avoid usage of dyn, trait objects, and dynamic dispatch

Coding style:
* Prefer idiomatic zero cost abstractions
* Prefer functional style i.e. map, flatmap, filter when dealing with collections

Safety and security style:
* Avoid unsafe in all crates
* Avoid macros in all lib code i.e. everything under /src. However, macros for testing are permissible when using sparingly i.e. for bulk testing many types implementing the same trait. 
* Avoid the introduction of external crates unless it is necessary for testing. 
