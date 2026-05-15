## ADDED Requirements

### Requirement: A new example crate lives at `examples/finance_examples/`
The repository SHALL contain a new example crate rooted at `examples/finance_examples/` with a binary target `regime_change` that demonstrates causal regime-change detection on a synthetic price series.

#### Scenario: Crate compiles
- **WHEN** a developer runs `cargo build -p finance_examples` from the repo root
- **THEN** the build SHALL succeed without warnings (under the same lint profile the other example crates use)

#### Scenario: Binary runs
- **WHEN** a developer runs `cargo run -p finance_examples --bin regime_change`
- **THEN** the program SHALL run to completion and print at least one regime-change event to stdout

### Requirement: Example uses the current DeepCausality API
The example SHALL build its causal logic from the public API of `deep_causality` and `deep_causality_core`. It SHALL NOT reach into private modules, SHALL NOT depend on unpublished branches, and SHALL NOT invent types that mimic library types.

#### Scenario: Imports come from public crates
- **WHEN** the source files are inspected
- **THEN** every `use` of a DeepCausality type SHALL refer to a publicly exported path from `deep_causality`, `deep_causality_core`, or a sibling published crate

#### Scenario: Effect propagation uses `PropagatingEffect` or `PropagatingProcess`
- **WHEN** the causal logic is traced through `main.rs`
- **THEN** the values flowing between rules SHALL be `PropagatingEffect<T>` or `PropagatingProcess<T, S, C>`, and composition SHALL use the standard `bind` operation, not a hand-rolled chain

### Requirement: Self-contained synthetic data
The example SHALL generate its own input data deterministically from a seeded generator inside the crate. It SHALL NOT read external files at runtime. It SHALL NOT require network access.

#### Scenario: Deterministic output
- **WHEN** the binary is run twice with the same source code
- **THEN** the printed regime-change event stream SHALL be byte-identical

### Requirement: Detector identifies the planted regime shift
The synthetic series SHALL contain two distinct regimes — a low-volatility trending segment followed by a high-volatility mean-reverting segment. The detector SHALL emit at least one `Enter` event in the second half of the series.

#### Scenario: Unit test confirms detection
- **WHEN** a developer runs `cargo test -p finance_examples`
- **THEN** at least one test SHALL assert that the detector emits an `Enter(MeanRevertingHighVol)` event with an index in the back half of the synthetic series

### Requirement: Length and shape constraints
The example's primary entry point `main.rs` SHALL stay under 200 lines. Domain logic SHALL be factored into sibling modules (e.g., `model.rs`) when `main.rs` would exceed that bound. The example SHALL contain at most one `unsafe` block (preferably zero) and SHALL NOT depend on any external crate beyond the workspace's existing dev/example dependencies plus optional `rand`-style utilities already in use elsewhere in `examples/`.

#### Scenario: Length budget honored
- **WHEN** `main.rs` is inspected
- **THEN** it SHALL be 200 lines or fewer

#### Scenario: No new third-party dependencies
- **WHEN** `Cargo.toml` is inspected
- **THEN** every dependency SHALL be either an internal DeepCausality crate, the Rust standard library, or a crate already used by another example crate under `examples/`
