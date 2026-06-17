# deep_causality Codebase Guide for Agents
## Core Directives

Communication Style:

Direct and Unfiltered: Do not use polite fillers or "I'm sorry". 
Just state the truth as it is.

Analytical: Use logic, historical precedent, and probability to back up your critiques.

Vigilant: Treat every prompt as a potential trap for cognitive bias or confirmation bias.

Tone: Be respectful regardless of agreement. Its okay to disagree, correct, and call out as long as its respectful.

## Golden Rules

1) Never ever git commit. Prepare a commit message and ask the user t0 commit.
2) Never ever delete files or folders. Ask the user for permission.

## Behavioral Guidelines

These guidelines favor correctness, clarity, and minimal diffs over speed. For trivial tasks, use judgment.

### 1. Think Before You Code

Do not silently guess.

Before making changes:

- State your assumptions clearly.
- If anything is ambiguous, ask instead of choosing one interpretation silently.
- If there are multiple valid approaches, briefly present the tradeoff.
- If the request seems mistaken, inefficient, or overcomplicated, say so.
- If a simpler solution exists, recommend it before implementing.
- If you are confused, stop and explain what is unclear.

- Do not act certain when you are uncertain.

### 2. Keep the Solution Simple

Solve the requested

problem with the minimum necessary code.

- Do not add features that were not asked for.
- Do not introduce abstractions for one-time
- Do not add configurability, extensibility, or generalization unless requested.
- Do not add defensive error handling for unrealistic cases.
- Prefer simple, readable code over clever code.
- If the solution feels too large, step back and simplify it.

Ask yourself:

- Is this the smallest change that solves the problem?
- Would a senior engineer consider this unnecessarily complex?

- If yes, simplify.

### 3. Stay Strictly Within Scope

Only change what the task requires.

When editing existing code:

- Do not refactor unrelated code.
- Do not rewrite comments, formatting, or naming unless necessary for the task.
- Match the existing style and conventions of the codebase.
- Do not fix neighboring issues unless the user asked.
- If you notice unrelated problems, mention them separately instead of changing them.

Every changed line should be easy to justify from the request.

### 4. Make Surgical Diffs

Keep edits local, focused, and easy to review.

- Touch as few files as possible.
- Change as little code as necessary.
- Avoid broad rewrites when a targeted fix is enough.
- Prserve existing structure unless changing it is required.
- Remove only the dead code, imports, or variables created by your own changes.
- Do not delete pre-existing unused code unless asked.

Prefer small and focused diffs over sweeping cleanup.

### 5. Work Toward Verifiable Outcomes

Do not treat "done" as a guess.

Turn requests into clear success criteria whenever possible.

- "Fix the bug" - reproduce it, fix it, then verify the fix
- "Add validation" -> add checks for invalid input and verify behavior
- "Refactor this" - preserve behavior and confirm tests still pass
- "Optimize this" -> improve performance without changing correctness
  For multi-step tasks, make a short plan with verification points.

Examples:

1. Inspect the current behavior -> verify: identify the root cause of the issue
2. Implement the minimal fix -> verify: affected behavior changes as expected
3. Run tests or checks -> verify: no regressions introduced

Prefer tests, existing checks, or concrete validation over verbal confidence.

### 6. Read Before You Write

Understand the surrounding code before editing it.

- Read enough nearby code to understand how the target piece fits in.
- Identify the local conventions before introducing new patterns.
- Do not infer architecture from one file when other relevant files are available.
- If context is missing, say so.

Understand the existing layers of abstraction:

- Identify the various levels of abstraction in the affected codebase.
- Expand the search of the codebase to understand the various levels of abstraction already in place.
- Ensure that new code lands at the right level of abstraction in the existing code base.
- If you find inconsistencies of abstraction, flag them. Stop and say so.
- If you cannot determine where to put new code in the abstraction hierarchy, flag, stop and say so.

Do not patch blindly.

### 7. Preserve Intent

Do not accidentally erase meaning while making changes.

- Preserve
  comments unless they are clearly outdated and directly affected by the task.
- Preserve behavior unless the requested change is meant to alter it.
- Preserve public interfaces unless changing them is necessary.
- Call out any intentional behavior change explicitly.

Do not make hidden product or design decisions on the user's behalf. Instead, ask.

### 8. Ask for Help at the Right Time

Do not continue blindly when the risk is high.

Pause and ask if:

- the request is ambiguous in a way that affects implementation
- the codebase contains conflicting patterns
- the correct behavior is unclear
- the task requires a product or architectural decision
- you are choosing between tradeoffs the user should approve

Do not fabricate certainty to stay moving.

### 9. Final Check Before You Finish

Before considering the task complete, confirm:

- the request was actually addressed
- the change is no larger than necessary
- unrelated code was not modified
- assumptions were surfaced
- affected tests or checks were run when possible
- the final result matches the requested scope

If something could not be verified, say that clearly.

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
* `ultragraph`: Hypergraph data structure used as a backend in deep_causality.

### Algorithm and Discovery Crates
* `deep_causality_algorithms`: Computational causality algorithms (SURD, MRMR) and utils.
* `deep_causality_discovery`: Causality discovery DSL for the DeepCausality project.

### Math and Numerics Crates
* `deep_causality_calculus`: "Arrow-native differentiation and integration operators.
* `deep_causality_num`: Numerical traits and utils used across all crates.
* `deep_causality_rand`: Random number generator and statistical distributions.
* `deep_causality_sparse`: Sparse matrix data structure (CSR format) for deep_causality.
* `deep_causality_tensor`: Tensors 
* `deep_causality_multivector`: Multivector implementation for geometric algebra.
* `deep_causality_uncertain`: A first-order type for uncertain programming.

### Functional Programming Crates
* `deep_causality_haft`: Higher-Order Abstract Functional Traits (HKT).
* `deep_causality_ethos`: Programmable ethics for DeepCausality.

### Topology and Physics Crates
* `deep_causality_topology`: Topological data structures (complexes, manifolds, differential geometry).
* `deep_causality_physics`: Standard library of physics formulas and engineering primitives.

## Project Dependencies

Scope: the 20 library crates that are workspace members. Example crates (`examples/*`),
vendored third-party crates (`thirdparty/crates/*`), and `yanked/*` are excluded.
`deep_causality_effects` exists on disk but is **not** a workspace member, so it is omitted.
`deep_causality_macros` is a member but deprecated and has no dependents.

### Internal Dependencies

Crates are arranged in dependency tiers: a crate depends only on crates in lower tiers.
`→` lists the direct internal (path) dependencies. `(opt)` marks an optional, feature-gated
dependency. Dev/test/bench-only dependencies are shown separately below.

```
Tier 0 — Foundational (no internal runtime dependencies)
  deep_causality_ast
  deep_causality_haft
  deep_causality_metric
  deep_causality_num
  deep_causality_data_structures
  ultragraph

Tier 1
  deep_causality_core        → deep_causality_haft
  deep_causality_calculus    → deep_causality_haft, deep_causality_num
  deep_causality_rand        → deep_causality_num
  deep_causality_tensor      → deep_causality_ast, deep_causality_haft, deep_causality_num

Tier 2
  deep_causality_sparse      → deep_causality_num, deep_causality_haft, deep_causality_tensor (opt)
  deep_causality_multivector → deep_causality_haft, deep_causality_num, deep_causality_tensor,
                               deep_causality_metric
  deep_causality_uncertain   → deep_causality_ast, deep_causality_num, deep_causality_rand

Tier 3
  deep_causality_topology    → deep_causality_num, deep_causality_haft, deep_causality_metric,
                               deep_causality_tensor, deep_causality_multivector,
                               deep_causality_sparse, deep_causality_rand
  deep_causality             → deep_causality_ast, deep_causality_core,
                               deep_causality_data_structures, deep_causality_haft,
                               deep_causality_uncertain, ultragraph

Tier 4
  deep_causality_algorithms  → deep_causality_num, deep_causality_rand,
                               deep_causality_tensor, deep_causality_topology
  deep_causality_physics     → deep_causality_calculus, deep_causality_core,
                               deep_causality_haft, deep_causality_metric,
                               deep_causality_multivector, deep_causality_num,
                               deep_causality_sparse, deep_causality_tensor,
                               deep_causality_topology, deep_causality_rand (opt)
  deep_causality_ethos       → deep_causality, ultragraph

Tier 5
  deep_causality_discovery   → deep_causality_algorithms, deep_causality_haft,
                               deep_causality_num, deep_causality_tensor
```

Internal dev-only dependency (tests/benches, not part of any published runtime):
* `deep_causality_rand` is a dev-dependency of `deep_causality_data_structures`,
  `deep_causality_sparse`, `deep_causality_tensor`, and `ultragraph`.

### External Dependencies

Only crates with at least one external (crates.io) runtime dependency are listed.
The other 16 library crates have no external runtime dependencies.

| Crate | External dependency | Status |
|-------|---------------------|--------|
| `deep_causality_num` | `libm` | optional — `libm_math` / `no-std` feature |
| `deep_causality_rand` | `getrandom`, `chacha20poly1305`, `zeroize` | all optional — `aead-random` / `os-random` features |
| `deep_causality_algorithms` | `rayon` | optional — `parallel` feature |
| `deep_causality_discovery` | `csv`, `parquet` | required (runtime) |

External dev-only dependencies (tests/benches, not part of any published runtime):
* `criterion` — benchmarks in `deep_causality`, `deep_causality_algorithms`,
  `deep_causality_data_structures`, `deep_causality_multivector`, `deep_causality_sparse`,
  `deep_causality_tensor`, `deep_causality_uncertain`, `ultragraph`.
* `tempfile` — `deep_causality_discovery` tests.
* `rusty-fork` — `deep_causality_uncertain` tests.


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

## Code testing

You aim for one hundred percent test coverage of all added or edited code files.
The only exception is if, for some reason, some code is impossible to reach. Then you skip testing that dead code.

Code examples under `examples/*` are exempt from the coverage requirement. They are runnable demonstrations, not library code, and are verified by running them (`cargo run -p <crate> --example <name>`) rather than by unit tests. Do not add test files or test modules for example binaries.

If tests find any bug, you fix the implementation so that the test pass. Because the testing exists to ensure that the API is correct, and if the API is not correct, you fix the API so that the test is passing.

Never ever fix any test to make a broken or incorrect API pass a bogus test. Never.

If you encounter a severe bug that requires refactoring beyond the first level implementation, and you suspect a large blast radius of breaking changes, ask the user how to proceed.

If you can derive a cleaner or better architecture, please do so and refactor downstream tests. This is okay. However, do not compromise any architecture for test compatibility, because remember, test exists to decide if the API is correct or not. If the API changes, so do the tests to veify that the new architectue is correct. 


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

Every single test files must be registered to the correspoding mod file and that module must be registered with its higher up module. ensure the corrext #[cfg(test)] annotation is set for each registeres test file. 

Also, ensure the folder modules are correctly declared in the Bazel configration undre crate_name/tests/BUILD.bazel. 

The tests folder replicates the exact src folder structure, for for example:

tests/errors/mod.r  # contains tests for each error type in a separate file
tests/traits/mod.rs # Optional contains tests for each  trait in a separate file
tests/type/mod.rs # contains tests for each  type in a separate file

Test files replicate the source file name with an appended _tests. For example,
a source file

`src/errors/normal_error/normal_error.rs`

is matched with the test file under the tests folder:

`test/errors/normal_error/normal_error_tests.rs`

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
* Private types: Public fields may be used, provided they do not
  leak outside their defined scope.

Static Dispatch:
* Use static dispatch 
* Avoid usage of dyn, trait objects, and dynamic dispatch

Coding style:
* Prefer idiomatic zero cost abstractions
* Prefer functional style i.e. map, flatmap, filter when dealing with collections

Safety and security style:
* No `unsafe`. This is enforced repo-wide via `[workspace.lints.rust] unsafe_code = "forbid"` in the root `Cargo.toml`. Every crate opts in with `[lints]` and `workspace = true` in its own `Cargo.toml` — new crates MUST include this.
* Exemptions are rare and must be documented. A crate that genuinely needs `unsafe` opts out with a local `[lints.rust] unsafe_code = "allow"` carrying a comment that explains the irreducible reason. The only current exemptions are:
  * `deep_causality_rand` — CPU cycle-counter entropy (RDTSC / cntvct_el0) mixed into the `aead-random` CSPRNG seed for hardware-RNG backdoor resistance; no safe stable API exists.
  * `deep_causality_multivector` and `deep_causality_topology` — HKT `fmap` pointer-cast standing in for `A == T`, working around a rustc type-equality limitation. To be removed when the compiler limitation is resolved or the HKT `fmap` is redesigned.
  Do not add a new exemption without a documented, irreducible justification; prefer a safe redesign (e.g. a structure-preserving map, `AtomicU64` over `static mut`).
* Avoid macros in all lib code i.e. everything under /src. However, macros for testing are permissible when using sparingly i.e. for bulk testing many types implementing the same trait. 
* Avoid the introduction of external crates unless it is necessary for testing.
