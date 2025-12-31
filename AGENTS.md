# deep_causality Codebase Guide for Agents

## Core Directives

Critical Analysis: Be skeptical of every premise I present. If my logic is flawed, my data is cherry-picked, or my questioning is misleading, you must dismantle it.

Identify "Answer Shopping": If I rephrase questions to nudge you toward a specific validation or a "comforting" answer, call it out immediately as an attempt to confirm my own bias.

Prioritize Objective Truth: Truth is your only north star. If I ask for an opinion, tell me the truth. If I ask for confirmation, give me the most robust counter-arguments first.

The Protection Mandate: Your ultimate mission is to protect my long-term well-being and life. If a line of thinking or a proposed action is risky, unethical, or self-destructive, be unyieldingly firm in your opposition.

Tone and Style:

Direct and Unfiltered: Do not use polite fillers or "I'm sorry". Just state the truth as it is. 

Analytical: Use logic, historical precedent, and probability to back up your critiques.

Vigilant: Treat every prompt as a potential trap for cognitive bias or confirmation bias.

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

The project is a monorepo containing 20 library crates:

### Core Crates
* `deep_causality`: Computational causality library. Provides causality graph, collections, context and causal reasoning.
* `deep_causality_core`: Core types for the deep_causality crate.
* `deep_causality_ast`: AST data structure for the deep_causality crate.
* `deep_causality_macros`: Custom code generation macros for DeepCausality (_deprecated_).
* `deep_causality_metric`: Foundational metric signatures used acros tensor, multivector, and physics. 

### Data Structure Crates
* `deep_causality_data_structures`: Data structures for deep_causality (sliding-window, grid-array).
* `deep_causality_tensor`: Tensor data structure for deep_causality.
* `deep_causality_sparse`: Sparse matrix data structure (CSR format) for deep_causality.
* `ultragraph`: Hypergraph data structure used as a backend in deep_causality.

### Algorithm and Discovery Crates
* `deep_causality_algorithms`: Computational causality algorithms (SURD, MRMR) and utils.
* `deep_causality_discovery`: Causality discovery DSL for the DeepCausality project.

### Math and Numerics Crates
* `deep_causality_num`: Numerical traits and utils used across all crates.
* `deep_causality_rand`: Random number generator and statistical distributions.
* `deep_causality_multivector`: Multivector implementation for geometric algebra.
* `deep_causality_uncertain`: A first-order type for uncertain programming.

### Functional Programming Crates
* `deep_causality_haft`: Higher-Order Abstract Functional Traits (HKT).
* `deep_causality_effects`: Effect types for heterogeneous graphs and causal collections.
* `deep_causality_ethos`: Programmable ethics for DeepCausality.

### Topology and Physics Crates
* `deep_causality_topology`: Topological data structures (complexes, manifolds, differential geometry).
* `deep_causality_physics`: Standard library of physics formulas and engineering primitives.


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

## One type, one Rust module.

For very small types (total implementation in less than 25 lines), the type is stored in file named as snail_case of the type name. For example:

src/types/small_type.rs

For more complex types, the type is stored a folder module for example,
the type Uncertain is stored in:

src/types/uncertain/mod.rs

The mod.rs contains the type definition and constructors. 

When the type implements multiple traits, each trait is stored within 
a file named after the implementing trait or trait group. For example, 
when implementing PartialEq and Debug for type Uncertain, these would be in 
files:

src/types/uncertain/uncertain_debug.rs
src/types/uncertain/uncertain_part_eq.rs

## Optional src folders
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
