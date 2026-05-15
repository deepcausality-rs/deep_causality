## ADDED Requirements

### Requirement: A new example crate lives at `examples/sre_examples/`
The repository SHALL contain a new example crate rooted at `examples/sre_examples/` with a binary target `cascading_failure_rca` that demonstrates causal root-cause analysis on a synthetic incident window.

#### Scenario: Crate compiles
- **WHEN** a developer runs `cargo build -p sre_examples` from the repo root
- **THEN** the build SHALL succeed without warnings (under the same lint profile the other example crates use)

#### Scenario: Binary runs
- **WHEN** a developer runs `cargo run -p sre_examples --bin cascading_failure_rca`
- **THEN** the program SHALL run to completion and print a structured root-cause attribution to stdout

### Requirement: Example uses the current DeepCausality API
The example SHALL build its causal logic from the public API of `deep_causality` and `deep_causality_core`. It SHALL NOT reach into private modules, SHALL NOT depend on unpublished branches, and SHALL NOT invent types that mimic library types.

#### Scenario: Imports come from public crates
- **WHEN** the source files are inspected
- **THEN** every `use` of a DeepCausality type SHALL refer to a publicly exported path from `deep_causality`, `deep_causality_core`, or a sibling published crate

#### Scenario: Causal logic uses Causaloid composition
- **WHEN** the causal logic is traced through `main.rs`
- **THEN** the four service-level rules SHALL be built as `Causaloid`s and composed via `CausaloidGraph` or via the standard collection/graph constructors, not as a hand-rolled ad-hoc chain

### Requirement: Synthetic incident window
The example SHALL generate its own incident-window input deterministically from a seeded generator inside the crate. It SHALL NOT read external files at runtime. It SHALL NOT require network access.

#### Scenario: Deterministic output
- **WHEN** the binary is run twice with the same source code
- **THEN** the printed attribution SHALL be byte-identical

### Requirement: Detector identifies the planted root cause
The synthetic incident window SHALL bake in a known root cause (DB lag) and let downstream signals fire after a fixed delay. The detector SHALL attribute the cause to `db_lag` (or whichever planted root the seeded fixture specifies).

#### Scenario: Unit test confirms attribution
- **WHEN** a developer runs `cargo test -p sre_examples`
- **THEN** at least one test SHALL assert the detector attributes the root cause to the predicate the fixture planted

### Requirement: Length and shape constraints
The example's primary entry point `main.rs` SHALL stay under 200 lines. Domain logic SHALL be factored into sibling modules when `main.rs` would exceed that bound. The example SHALL contain zero `unsafe` blocks. It SHALL NOT depend on any external crate beyond the workspace's existing dev/example dependencies.

#### Scenario: Length budget honored
- **WHEN** `main.rs` is inspected
- **THEN** it SHALL be 200 lines or fewer

#### Scenario: No new third-party dependencies
- **WHEN** `Cargo.toml` is inspected
- **THEN** every dependency SHALL be either an internal DeepCausality crate, the Rust standard library, or a crate already used by another example crate under `examples/`
