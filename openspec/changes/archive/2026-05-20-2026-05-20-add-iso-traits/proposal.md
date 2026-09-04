## Why

Two pain points the iso trait surface resolves:

1. **Cross-crate isomorphisms cannot currently be expressed cleanly.** `Quaternion<F>` (in `deep_causality_num`) ↔ `CausalMultiVector<F>` (in `deep_causality_multivector`) is a real algebraic correspondence — quaternions are isomorphic to Cl(3,0) rotors. Tier 1 `From`/`Into` cannot express it because the dependency direction is asymmetric: `deep_causality_num` cannot name `CausalMultiVector<F>`, so `From<CausalMultiVector<F>> for Quaternion<F>` has nowhere it compiles. The orphan rule blocks the reverse direction outright. Same problem for any future `Quaternion<F>` ↔ `PauliMatrix<F>` work and other physics-domain cross-crate bridges.

2. **`PropagatingEffect<T>` ↔ `PropagatingProcess<T, (), ()>` is implicitly an isomorphism that the type system doesn't know about.** The lift via `PropagatingProcess::with_state(eff, (), None)` is byte-identical round-trip. Currently consumers hand-roll unwrap-rewrap at every Markovian / non-Markovian boundary in the fluid causal-inference pipeline (see [`openspec/notes/3DCausalFluidDynamics.md`](../../notes/3DCausalFluidDynamics.md) §4.1). Helper libraries written for one carrier cannot be reused on the other without ad-hoc conversion. The type-level relationship is missing.

The broader value is in the foundation. The project's HKT machinery (`deep_causality_haft` with arity-5 `HKT` and `MonadEffect5`) and algebraic-trait hierarchy (`deep_causality_num` with `Magma → Group → Ring → Field → DivisionAlgebra` plus `Module<R> / Algebra<R> / DivisionAlgebra<R>` for vector structures) are already in place. The iso layer composes naturally on top using existing Rust trait inheritance and the witness pattern HAFT already uses; it gives every future structure-preserving conversion a typed home without dependent types, macros, or new external dependencies.

Design note: [`openspec/notes/NumIso.md`](NumIso.md).

## What Changes

This change adds **foundation traits only** in two crates. Concrete iso instances (`PropagatingEffect` ↔ `PropagatingProcess<T, (), ()>` in `deep_causality_core`, `Quaternion<F>` ↔ Cl(3,0) rotor in `deep_causality_multivector`, dense ↔ sparse in `deep_causality_sparse`) are deferred to separate follow-up changes that consume the trait surface. Each is independently scoped and per-crate.

**Part A — Tier 1 marker subtraits in `deep_causality_num`**

- New module `src/iso/` exporting marker subtraits bounded on bidirectional `From`/`Into`: `GroupIso<T>`, `RingIso<T>`, `FieldIso<T>`, `AlgebraIso<T, R>`, `DivisionAlgebraIso<T, R>`.
- Each subtrait has an empty body. The where-clauses (`Self: Group + From<T>`, `T: Group + From<Self>`, etc.) make implementing the subtrait a marker promise: "the bidirectional `From` impls preserve the relevant algebraic structure."
- Property-test helpers in `src/iso/test_support.rs` exercise the round-trip and homomorphism laws. The helpers are `#[cfg(test)]`-gated and require `std`; the trait declarations themselves are `no_std`-compatible.
- No new external dependencies. `proptest` is added as a `dev-dependencies` entry in `deep_causality_num`'s `Cargo.toml`.

**Part B — Tier 2 `Iso<S, T>` trait, marker subtraits, and `StandardIso<S, T>` in `deep_causality_num`**

- New module `src/iso/witness/` exporting `Iso<S, T>` with methods `fn to_target(s: S) -> T;` and `fn to_source(t: T) -> S;`.
- Parallel structure-preserving subtraits `GroupIso<S, T>`, `RingIso<S, T>`, `FieldIso<S, T>`, `AlgebraIso<S, T, R>`, `DivisionAlgebraIso<S, T, R>` with where-clauses on `S` and `T` rather than on `Self`. Empty marker bodies. Property tests on the homomorphism laws via the witness's `to_target` / `to_source`.
- Generic `StandardIso<S, T>(PhantomData<(S, T)>)` zero-sized witness type with blanket impls for `Iso<S, T>` and every Tier 2 marker subtrait. The blanket impls fire automatically when `S` and `T` satisfy bidirectional `From` plus the corresponding algebraic-structure bounds — no manual marker impls required for the common case.
- Public exports from `src/lib.rs`: every Tier 2 trait above, with the witness module path preserved (`pub use iso::witness;`) so consumers disambiguate `iso::GroupIso<T>` (Tier 1) from `iso::witness::GroupIso<S, T>` (Tier 2).
- Method names `to_target` / `to_source` deliberately differ from `forward` / `backward` (which would collide with the EPP framework's temporal "forward-in-time" vocabulary) and from `from` / `into` (which would conflict with std's `From` / `Into` semantics where `Self` is the constructed type).

**Part C — Tier 3 `NaturalIso<F, G>` in `deep_causality_haft`**

- New module `src/iso/` exporting `NaturalIso<F, G>` for HKT-witness isomorphisms: `fn to_target<T>(fa: F::Type<T>) -> G::Type<T>;` and `fn to_source<T>(ga: G::Type<T>) -> F::Type<T>;`.
- Higher-arity variant `NaturalIso5<F, G>` for use against the 5-arity propagating-effect carrier in `deep_causality_core`.
- Naturality property-test helpers in `src/iso/test_support.rs` exercising naturality against a fixed bank of test functions (negation, doubling, identity, constant, string-conversion). The helpers are `#[cfg(test)]`-gated.
- Public exports from `src/lib.rs`: `NaturalIso`, `NaturalIso5`.

**Not in scope (deferred to follow-up changes)**

- Concrete `NaturalIso` impl for `PropagatingEffect<T>` ↔ `PropagatingProcess<T, (), ()>` (a separate change to `deep_causality_core`).
- Concrete `Iso<S, T>` impl for `Quaternion<F>` ↔ `CausalMultiVector<F>` (a separate change to `deep_causality_multivector`).
- Concrete `From`/`Into` + Tier 1 markers for dense ↔ sparse (a separate change to `deep_causality_sparse`).
- Coordinate isos on `Manifold<LatticeComplex<3>, F>` in `deep_causality_topology` (specialized trait `CoordinateIso<C1, C2, F>` that carries Jacobian metadata; conceptually parallel but architecturally separate).
- Dedicated iso witness types for the multi-convention scenario. The pattern is structurally available (Rust's coherence rules allow distinct witness types with overlapping target type pairs) but **no current codebase use case requires it**. Reserved for a future change if and when a real multi-convention need emerges.
- Approximate / numerical-equivalence isomorphism with error tracking. Considered, dropped — `Float106` ↔ `f64` round-trips with precision loss tracked through existing `Float106` traits, not via a separate iso variant.
- Theory morphisms in the GATlab sense (non-bijective morphisms requiring dependent types).

## Capabilities

### New Capabilities

- `iso-traits-num`: A three-level trait surface in `deep_causality_num` for isomorphism between algebraic structures. Tier 1 provides structure-preserving marker subtraits (`GroupIso<T>`, `RingIso<T>`, `FieldIso<T>`, `AlgebraIso<T, R>`, `DivisionAlgebraIso<T, R>`) bounded on bidirectional `From`/`Into`. Tier 2 provides the witness-typed `Iso<S, T>` trait, parallel structure-preserving marker subtraits, and the generic `StandardIso<S, T>` witness with blanket impls that auto-derive every marker subtrait from bidirectional `From` plus algebraic structure on `S` and `T`. The two tiers compose with the existing algebraic-trait hierarchy via Rust's standard trait-inheritance mechanism.

- `iso-traits-haft`: A natural-isomorphism trait `NaturalIso<F, G>` for HKT witnesses in `deep_causality_haft`, with arity-1 and arity-5 variants. Bridges the gap that Tier 1 / Tier 2 cannot cover — HKT witnesses are zero-sized types with no instances and therefore no `From`/`Into` analog. Generic code over HKT witnesses can express functor-equivalent type constructors that differ only in fixed-parameter values (e.g. `PropagatingEffect<T>` ≅ `PropagatingProcess<T, (), ()>` at the HKT level).

## Impact

**Affected crates:** `deep_causality_num` and `deep_causality_haft` (two crates touched, both in additive mode — no breaking changes).

**Affected modules:**

- `deep_causality_num/src/iso/` (new module tree):
  - `mod.rs` — re-exports for Tier 1 + Tier 2.
  - `group_iso.rs`, `ring_iso.rs`, `field_iso.rs`, `algebra_iso.rs`, `division_algebra_iso.rs` — Tier 1 marker subtraits.
  - `witness/mod.rs` — re-exports for Tier 2.
  - `witness/iso.rs` — `Iso<S, T>` trait.
  - `witness/standard.rs` — `StandardIso<S, T>` generic witness with blanket impls.
  - `witness/group_iso.rs`, `ring_iso.rs`, `field_iso.rs`, `algebra_iso.rs`, `division_algebra_iso.rs` — Tier 2 marker subtraits.
  - `test_support.rs` (Tier 1) and `witness/test_support.rs` (Tier 2) — property-test helpers, `#[cfg(test)]`-gated.
- `deep_causality_num/src/lib.rs` — additive re-exports.
- `deep_causality_num/Cargo.toml` — add `proptest` to `dev-dependencies`.
- `deep_causality_num/BUILD.bazel` — register the new module tree.
- `deep_causality_num/tests/iso/` (new) — property-test files mirroring `src/iso/` per the project convention.
- `deep_causality_num/tests/BUILD.bazel` — register the test files.
- `deep_causality_haft/src/iso/` (new module tree):
  - `mod.rs` — re-exports.
  - `natural_iso.rs` — `NaturalIso<F, G>`.
  - `natural_iso_5.rs` — `NaturalIso5<F, G>`.
  - `test_support.rs` — naturality-property helpers, `#[cfg(test)]`-gated.
- `deep_causality_haft/src/lib.rs` — additive re-exports.
- `deep_causality_haft/Cargo.toml` — add `proptest` to `dev-dependencies` if not already present.
- `deep_causality_haft/BUILD.bazel`, `deep_causality_haft/tests/BUILD.bazel` — register module and test files.

**External API impact:**

- Purely additive. No renames, no signature changes, no deprecations. Existing call sites are not affected.

**Constraints honored:**

- No `unsafe` introduced.
- No `dyn` / trait objects; all dispatch is static through monomorphization.
- No macros in lib code.
- No new external runtime dependencies. `proptest` is `dev-dependencies` only.
- `no_std` support preserved in both crates. The trait declarations are `no_std`-compatible; test infrastructure is `#[cfg(test)]`-gated and may require `std`.
- Tests mirror the `src/` structure under `tests/` and are registered in `tests/BUILD.bazel`.
- No `pub use` back-compat shims (the change is additive, none would be needed).

**Sequencing inside the change** (three task groups in `tasks.md`, each gated):

1. **Part A** (Tier 1 marker subtraits in `deep_causality_num`) — lands first; the Tier 1 surface is self-contained and depends on nothing.
2. **Part B** (Tier 2 `Iso<S, T>` + `StandardIso<S, T>` + blanket impls in `deep_causality_num`) — depends on Part A only insofar as it lives in the same crate. The Tier 2 surface is independent of Tier 1; the blanket impls on `StandardIso<S, T>` reference Tier 1 marker subtraits (since `StandardIso` derives the Tier 1 markers via the algebraic-structure bounds on `S` and `T`).
3. **Part C** (Tier 3 `NaturalIso<F, G>` + `NaturalIso5<F, G>` in `deep_causality_haft`) — fully independent of Parts A and B; can be sequenced last for review-locality reasons but is otherwise parallelizable.

**Stage gates (binding).** Each stage MUST receive explicit user sign-off before the next stage starts:

- Stage completion criteria: all tasks in the stage checked off; `cargo build -p deep_causality_num` (or `_haft` for Part C) plus `cargo test -p deep_causality_num` (or `_haft`) green; `make format && make fix` clean; no new clippy warnings; the stage's spec scenarios verified.
- Sign-off protocol: agent presents a stage-completion summary (what changed, what was verified, any deviations from spec). User reviews and either (a) signs off in writing ("approved" / "looks good" / explicit go-ahead for next stage) or (b) requests revisions. Implicit approval is not sufficient.
- Commit protocol: per AGENTS.md golden rule §1, the agent NEVER commits. After sign-off, the agent prepares a commit message and the user runs the commit. Only after the commit lands does the next stage begin.
- A stage that fails review returns to in-progress; the gate does not advance until the user re-approves.
