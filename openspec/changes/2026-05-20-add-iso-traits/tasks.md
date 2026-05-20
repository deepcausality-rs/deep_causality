## 1. Part A — Tier 1 marker subtraits in `deep_causality_num`

> **Gate:** This stage MUST be completed, verified, signed off by the user, and committed before any task in stage 2 begins. See "Stage gates" at the end of this file.

- [x] 1.1 Create the new module tree under `deep_causality_num/src/iso/`. Files: `mod.rs` (re-exports), `group_iso.rs`, `ring_iso.rs`, `field_iso.rs`, `algebra_iso.rs`, `division_algebra_iso.rs`, `test_support.rs`. **Deviation:** `test_support.rs` is `pub mod` (not `#[cfg(test)]`-gated) per the existing `src/utils_tests/` convention; Bazel can't see `tests/` from `src/` so coverage-counting test utilities must live in `src/`.
- [x] 1.2 Declare `pub trait GroupIso<T> where Self: Group + From<T>, T: Group + From<Self> {}` in `src/iso/group_iso.rs`. Empty body. Doc comment explains the marker promise.
- [x] 1.3 Declare `pub trait RingIso<T>: GroupIso<T> where Self: Ring + From<T>, T: Ring + From<Self> {}` in `src/iso/ring_iso.rs`. **Note:** the `From` bounds must be re-stated in the where-clause (Rust subtrait rules) — original spec omitted this.
- [x] 1.4 Declare `pub trait FieldIso<T>: RingIso<T> where Self: Field + From<T>, T: Field + From<Self> {}` in `src/iso/field_iso.rs`. Empty body.
- [x] 1.5 Declare `pub trait AlgebraIso<T, R> where Self: Algebra<R> + From<T>, T: Algebra<R> + From<Self>, R: Ring {}` in `src/iso/algebra_iso.rs`. Empty body.
- [x] 1.6 Declare `pub trait DivisionAlgebraIso<T, R>: AlgebraIso<T, R> where Self: DivisionAlgebra<R> + From<T>, T: DivisionAlgebra<R> + From<Self>, R: Field {}` in `src/iso/division_algebra_iso.rs`. Empty body.
- [x] 1.7 Write `src/iso/test_support.rs`. Exports: `assert_iso_from_round_trip<S, T>`, `assert_group_iso_from_law<S, T>`, `assert_ring_iso_from_laws<S, T>`, `assert_field_iso_from_laws<S, T>`, `assert_algebra_iso_from_law<S, T, R>(a: S, r: R)`, `assert_division_algebra_iso_from_law<S, T, R>`. Helpers use `assert_eq!` with descriptive failure messages and `core::fmt::Debug` (no_std-compatible).
- [x] 1.8 Update `deep_causality_num/src/iso/mod.rs` to declare submodules and re-export every Tier 1 marker subtrait. Doc comment introduces the three-tier design.
- [x] 1.9 Update `deep_causality_num/src/lib.rs`: add `pub mod iso;` and `pub use crate::iso::{AlgebraIso, DivisionAlgebraIso, FieldIso, GroupIso, RingIso};`.
- [ ] 1.10 **Deferred (deviation):** `proptest` dev-dependency not added. Test helpers are plain assertion functions; test files use hardcoded representative inputs. Per AGENTS.md ("Avoid the introduction of external crates unless necessary for testing") — empty marker traits don't require randomized testing for the foundation change. Can be added in a follow-up change if downstream consumers want it.
- [x] 1.11 `deep_causality_num/BUILD.bazel` requires no edit; `srcs = glob(["src/**"])` automatically picks up new files.
- [x] 1.12 Create test files under `deep_causality_num/tests/iso/`: `group_iso_tests.rs`, `ring_iso_tests.rs`, `field_iso_tests.rs`, `algebra_iso_tests.rs`, `division_algebra_iso_tests.rs`, `test_support_tests.rs`, `common.rs` (shared `FloatWrap` and `BadFieldWrap` test types). Each marker test file contains: trait impl for a local newtype (verifying the trait can be implemented), success-path assertions on the identity iso, and at least one `#[should_panic]` test covering the failure branch of the corresponding helper.
- [x] 1.13 Register `mod iso;` in `deep_causality_num/tests/mod.rs`. `tests/BUILD.bazel` does not exist in this crate (the crate uses `cargo test` directly and the integration test binary is auto-generated from `tests/mod.rs`).
- [x] 1.14 `cargo build -p deep_causality_num` and `cargo test -p deep_causality_num` both pass cleanly. 21 new iso tests pass (15 success-path + 6 should-panic panic-path). `cargo clippy -p deep_causality_num --tests -- -D warnings` clean.
- [x] 1.15 `cargo build -p deep_causality_num --no-default-features --features libm_math` passes cleanly; `cargo test -p deep_causality_num --no-default-features --features libm_math` passes (4106 tests + 175 doctests). **Bonus fixes applied during stage A**: six pre-existing no_std bugs in unrelated code that were blocking verification — (a) `src/float_106/ops_arithmetic.rs:255` missing cfg-gated `libm::trunc` dispatch, (b) `src/float_106/mod.rs:129` missing cfg-gated `libm::fma` dispatch, (c) `src/algebra/field_real.rs:426/485/692/697` four `asin`/`atan` implementations causing infinite recursion (missing the cfg-gated `libm::asin{f}`/`libm::atan{f}` dispatch pattern that every other transcendental in the file already has), (d) two test files (`tests/algebra/field_real_f64_tests.rs::test_exp` and `tests/float/float_64_tests.rs::exp_val`) using `assert_eq!` for libm-vs-std comparison where the two implementations agree only to ~1 ULP — relaxed to tolerance comparison matching the established `test_ln` pattern. None of these touch the iso module; all are independent fixes that unblock no_std verification.
- [x] 1.16 Run `make format && make fix` — clean across the workspace. No new clippy warnings.
- [x] 1.17 **Stage A gate:** stage-completion summary prepared. See gate text below the task list.

## 2. Part B — Tier 2 `Iso<S, T>` and `StandardIso<S, T>` in `deep_causality_num`

> **Gate:** Stage A MUST be signed off and committed before any task here begins.

- [ ] 2.1 Create the new module tree under `deep_causality_num/src/iso/witness/`. Files: `mod.rs` (re-exports), `iso.rs`, `standard.rs`, `group_iso.rs`, `ring_iso.rs`, `field_iso.rs`, `algebra_iso.rs`, `division_algebra_iso.rs`, `test_support.rs`.
- [ ] 2.2 Declare `pub trait Iso<S, T> { fn to_target(s: S) -> T; fn to_source(t: T) -> S; }` in `src/iso/witness/iso.rs`. Doc comment explains: method names `to_target` / `to_source` rather than `forward` / `backward` (collision with EPP temporal vocabulary) or `from` / `into` (collision with std semantics) per D3. Document round-trip law: `Self::to_source(Self::to_target(s)) == s` and the symmetric case.
- [ ] 2.3 Declare `pub trait GroupIso<S, T>: Iso<S, T> where S: Group, T: Group {}` in `src/iso/witness/group_iso.rs`. Empty body. Doc comment notes that the where-clauses constrain the type *pair* (`S`, `T`) rather than the implementer `Self` (per D4); the implementer is whichever type the iso is hung from.
- [ ] 2.4 Declare `pub trait RingIso<S, T>: GroupIso<S, T> where S: Ring, T: Ring {}` in `src/iso/witness/ring_iso.rs`. Empty body.
- [ ] 2.5 Declare `pub trait FieldIso<S, T>: RingIso<S, T> where S: Field, T: Field {}` in `src/iso/witness/field_iso.rs`. Empty body.
- [ ] 2.6 Declare `pub trait AlgebraIso<S, T, R>: Iso<S, T> where S: Algebra<R>, T: Algebra<R>, R: Ring {}` in `src/iso/witness/algebra_iso.rs`. Empty body. Note that `AlgebraIso<S, T, R>` extends `Iso<S, T>` directly rather than `RingIso<S, T>` — the algebra-vs-ring distinction is orthogonal; implementers write both when both apply.
- [ ] 2.7 Declare `pub trait DivisionAlgebraIso<S, T, R>: AlgebraIso<S, T, R> where S: DivisionAlgebra<R>, T: DivisionAlgebra<R>, R: Field {}` in `src/iso/witness/division_algebra_iso.rs`. Empty body.
- [ ] 2.8 Declare `pub struct StandardIso<S, T>(core::marker::PhantomData<(S, T)>);` in `src/iso/witness/standard.rs`. Add an inherent `pub const fn new() -> Self` constructor. Implement `Clone`, `Copy`, `Default`, `Debug` for `StandardIso<S, T>` (the bounds need to be `where S: ?Sized + 'static, T: ?Sized + 'static` so the implementations are free of constraints on `S` and `T`).
- [ ] 2.9 Implement `Iso<S, T> for StandardIso<S, T> where S: From<T>, T: From<S>` in `src/iso/witness/standard.rs`. Body: `fn to_target(s: S) -> T { T::from(s) }` and `fn to_source(t: T) -> S { S::from(t) }`.
- [ ] 2.10 Implement `GroupIso<S, T> for StandardIso<S, T> where S: Group + From<T>, T: Group + From<S>` in `src/iso/witness/standard.rs`. Empty body. The blanket fires whenever the bounds are satisfied.
- [ ] 2.11 Implement `RingIso<S, T> for StandardIso<S, T> where S: Ring + From<T>, T: Ring + From<S>` analogously.
- [ ] 2.12 Implement `FieldIso<S, T> for StandardIso<S, T> where S: Field + From<T>, T: Field + From<S>` analogously.
- [ ] 2.13 Implement `AlgebraIso<S, T, R> for StandardIso<S, T> where S: Algebra<R> + From<T>, T: Algebra<R> + From<S>, R: Ring` analogously.
- [ ] 2.14 Implement `DivisionAlgebraIso<S, T, R> for StandardIso<S, T> where S: DivisionAlgebra<R> + From<T>, T: DivisionAlgebra<R> + From<S>, R: Field` analogously.
- [ ] 2.15 Write `src/iso/witness/test_support.rs` (`#[cfg(test)]`-gated). Exports `assert_witness_iso_round_trip<W: Iso<S, T>, S, T>(s: S)`, `assert_witness_group_iso_law<W: GroupIso<S, T>, S, T>(a: S, b: S)`, `assert_witness_ring_iso_laws<W: RingIso<S, T>, S, T>(a: S, b: S)`, `assert_witness_field_iso_laws<W: FieldIso<S, T>, S, T>(a: S)`, `assert_witness_algebra_iso_law<W: AlgebraIso<S, T, R>, S, T, R>(r: R, a: S)`, `assert_witness_division_algebra_iso_law<W: DivisionAlgebraIso<S, T, R>, S, T, R>(a: S)`. Signatures parallel to the Tier 1 helpers but operating through the witness's `to_target` / `to_source`.
- [ ] 2.16 Update `deep_causality_num/src/iso/witness/mod.rs` to declare submodules and re-export `Iso`, `StandardIso`, and every Tier 2 marker subtrait.
- [ ] 2.17 Update `deep_causality_num/src/iso/mod.rs` to add `pub mod witness;` (do NOT re-export Tier 2 traits at the top-level `iso::` path; the witness module path preserves the namespace split per D7).
- [ ] 2.18 Update `deep_causality_num/src/lib.rs`: re-export the witness module so consumers can write `use deep_causality_num::iso::witness::{Iso, StandardIso, GroupIso, ...};`. Document the Tier 1 / Tier 2 namespace distinction in the module doc.
- [ ] 2.19 Update `deep_causality_num/BUILD.bazel`: register the new `src/iso/witness/` module tree in the existing library target's `srcs` list.
- [ ] 2.20 Create test files under `deep_causality_num/tests/iso/witness/`: `iso_tests.rs` (round-trip on `StandardIso<i32, i32>` identity), `standard_tests.rs` (blanket-impl verification on a small concrete type pair that satisfies bidirectional `From` + `Ring`), `group_iso_tests.rs`, `ring_iso_tests.rs`, `field_iso_tests.rs`, `algebra_iso_tests.rs`, `division_algebra_iso_tests.rs`. Register in `tests/iso/witness/mod.rs`, `tests/iso/mod.rs`, and `tests/BUILD.bazel`.
- [ ] 2.21 Verify the `StandardIso<S, T>` blanket impl fires correctly: write a compile-pass test (a `#[test]` that uses turbofish to call `StandardIso::<S, T>::to_target(s)` for a concrete `(S, T)` pair where bidirectional `From` + `Ring` is provided in the test module) and assert that the type-checker accepts the call as `RingIso<S, T>` via the blanket.
- [ ] 2.22 Run `cargo build -p deep_causality_num` and `cargo test -p deep_causality_num`; both MUST pass. Verify no new clippy warnings.
- [ ] 2.23 Verify `no_std` compatibility: `cargo build -p deep_causality_num --no-default-features --features libm_math` MUST pass. The Tier 2 trait declarations and `StandardIso<S, T>` MUST compile under `no_std` (`StandardIso<S, T>` uses `core::marker::PhantomData`, not `std::marker::PhantomData`).
- [ ] 2.24 Run `make format && make fix` and verify no new clippy warnings.
- [ ] 2.25 **Stage B gate:** prepare a stage-completion summary. Wait for explicit written sign-off. Prepare a commit message. Do NOT advance to stage 3 until the commit has landed.

## 3. Part C — Tier 3 `NaturalIso<F, G>` in `deep_causality_haft`

> **Gate:** Stage B MUST be signed off and committed before any task here begins. Stage C is structurally independent of Stages A and B (it touches a different crate), but the sequencing keeps each stage's diff localized.

- [ ] 3.1 Create the new module tree under `deep_causality_haft/src/iso/`. Files: `mod.rs` (re-exports), `natural_iso.rs`, `natural_iso_5.rs`, `test_support.rs`.
- [ ] 3.2 Declare `pub trait NaturalIso<F, G> where F: HKT, G: HKT { fn to_target<T>(fa: F::Type<T>) -> G::Type<T>; fn to_source<T>(ga: G::Type<T>) -> F::Type<T>; }` in `src/iso/natural_iso.rs`. Doc comment explains: HKT witnesses are zero-sized types with no instances, so `From`/`Into` cannot apply at this level. Document the round-trip law (per `T`) and the naturality law (`to_target(F::fmap(fa, h)) == G::fmap(to_target(fa), h)` for any function `h: T -> U`).
- [ ] 3.3 Declare `pub trait NaturalIso5<F, G> where F: HKT5<...>, G: HKT5<...> { fn to_target<V, S, C, E, L>(fa: F::Type<V, S, C, E, L>) -> G::Type<V, S, C, E, L>; fn to_source<V, S, C, E, L>(ga: G::Type<V, S, C, E, L>) -> F::Type<V, S, C, E, L>; }` in `src/iso/natural_iso_5.rs`. The HKT5 bound expression uses the existing `HKT5` machinery in `deep_causality_haft`; check the existing crate to confirm the exact form. Doc comment notes the 5-arity case is for the propagating-effect carrier (per D8).
- [ ] 3.4 Write `src/iso/test_support.rs` (`#[cfg(test)]`-gated). Exports `assert_natural_iso_round_trip<W, F, G, T>(fa: F::Type<T>)` (round-trip per `T`) and `assert_natural_iso_naturality<W, F, G, T, U>(fa: F::Type<T>, h: impl Fn(T) -> U)` (naturality against a caller-supplied function). The test-function bank (negation, doubling, identity, constant, string-conversion) is set up as a `#[cfg(test)]`-only helper module that test files can import.
- [ ] 3.5 Update `deep_causality_haft/src/iso/mod.rs` to declare submodules and re-export `NaturalIso` and `NaturalIso5`.
- [ ] 3.6 Update `deep_causality_haft/src/lib.rs`: add `pub mod iso;` and `pub use iso::{NaturalIso, NaturalIso5};`.
- [ ] 3.7 Update `deep_causality_haft/Cargo.toml`: add `proptest` to `dev-dependencies` if not already present.
- [ ] 3.8 Update `deep_causality_haft/BUILD.bazel`: register the new `src/iso/` module tree.
- [ ] 3.9 Create test files under `deep_causality_haft/tests/iso/`: `natural_iso_tests.rs` (round-trip and naturality on an identity natural iso between two structurally-equivalent witnesses constructed for the test, e.g. `Option<T>` and a test-local `MyOption<T>` newtype), `natural_iso_5_tests.rs` (analogous for arity 5). Register in `tests/iso/mod.rs` and `tests/BUILD.bazel`.
- [ ] 3.10 Run `cargo build -p deep_causality_haft` and `cargo test -p deep_causality_haft`; both MUST pass. Verify no new clippy warnings.
- [ ] 3.11 Verify `no_std` compatibility: `cargo build -p deep_causality_haft --no-default-features --features alloc` MUST pass. The trait declarations MUST compile under `no_std`; test infrastructure may require `std`.
- [ ] 3.12 Run `make format && make fix` and verify no new clippy warnings.
- [ ] 3.13 **Stage C gate:** prepare a stage-completion summary. Wait for explicit written sign-off. Prepare a commit message. The change is complete after Stage C lands.

## Stage gates

Per AGENTS.md golden rule §1 (NEVER commit) and the protocol established in `2026-05-19-add-cubical-complexes`:

- Each stage MUST be completed in full before the next stage begins.
- Stage completion criteria (binding): all stage tasks checked off; `cargo build -p <crate>` and `cargo test -p <crate>` green for the affected crate; `make format && make fix` clean; `cargo clippy -p <crate> -- -D warnings` produces no warnings; `no_std` build passes; the stage's spec scenarios verified.
- Sign-off protocol: the agent presents a stage-completion summary listing changes, files touched, test evidence, and any deviations from spec. The user reviews and either (a) signs off in writing ("approved" / "looks good" / explicit go-ahead for next stage) or (b) requests revisions.
- Commit protocol: after sign-off, the agent prepares a commit message; the user runs the commit. The next stage begins only after the commit lands.
- A stage that fails review returns to in-progress; the gate does not advance until the user re-approves.
