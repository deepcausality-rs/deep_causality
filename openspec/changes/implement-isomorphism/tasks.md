## 1. Part A — Propagating-effect / process `Functor`/`Monad` consistency test (no iso wrapper)

> **Gate:** This stage MUST be completed in full, verified, signed off, and committed before any task in Part B begins. See "Stage gates" at the end.

- [x] 1.1 Create `deep_causality_core/tests/iso/` with `mod.rs` and `effect_process_consistency_tests.rs`. No source-tree changes.
- [x] 1.2 `fmap_same_type_agrees_across_witnesses` test asserts `EffectW::fmap(val, |x| x * 2) == ProcessW::fmap(val, |x| x * 2)` on shared carrier.
- [x] 1.3 `fmap_type_changing_agrees_across_witnesses` test for `|x: i32| x % 2 == 0`.
- [x] 1.4 `bind_agrees_across_witnesses` test pins consistency between the two `Monad::bind` impls. Bonus: also added `pure_agrees_across_witnesses` for the `Pure::pure` lift.
- [x] 1.5 `mod iso;` registered in `tests/mod.rs`. `rust_test_suite(name = "iso", ...)` added to `tests/BUILD.bazel`.
- [x] 1.6 `cargo test -p deep_causality_core`: 115 tests pass (4 new + 111 existing). `cargo clippy -p deep_causality_core --tests -- -D warnings`: clean.
- [x] 1.7 `bazel test //deep_causality_core/tests:iso`: PASSED.
- [x] 1.8 **Bonus fix** applied: `src/types/builder/executable_graph.rs` was missing `use alloc::vec;` for the `vec!` macro under no_std (pre-existing lib bug, not caused by Stage A). One-line cfg-gated import added per the `2026-05-20-add-iso-traits` Stage A precedent. `cargo build -p deep_causality_core --no-default-features --features alloc` now passes. (One pre-existing unused-import warning in `effect_value/mod.rs:8` remains; out of scope for this stage.) Format clean via `cargo fmt`.
- [x] 1.9 **Stage A gate:** stage-completion summary below; awaiting sign-off and commit.

## 2. Part B — `iso-tensor-sparse`: `CausalTensor<F>` <-> `CsrMatrix<F>` (mixed-tier template)

> **Gate:** Part A MUST be signed off and committed before any task here begins.

- [x] 2.1 Module created at `deep_causality_sparse/src/extensions/ext_iso.rs`. **Deviation**: per code-review feedback, moved to `src/extensions/` following the existing `ext_hkt.rs` convention (anything bridging sparse to another DC crate is an `ext_*` extension). Gated behind the `tensor-iso` Cargo feature so default sparse consumers don't pay the tensor-dep cost.
- [x] 2.2 `impl<F> From<CausalTensor<F>> for CsrMatrix<F> where F: Clone + Copy + Zero + PartialEq` added. Iterates row-major, drops zeros, panics on rank ≠ 2 with descriptive message.
- [x] 2.3 `impl<F> Iso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F>` added. `to_target` walks CSR row-pointer / col-index arrays to populate the dense buffer; `to_source` delegates to forward `From`.
- [x] 2.4 Inherent `CsrMatrix::to_dense(self) -> CausalTensor<F>` ergonomic alias added (also gated behind `tensor-iso`).
- [x] 2.5 No new lib.rs re-export needed; both `CsrMatrix` and the trait surface are already public via existing paths. **Cargo.toml** adds `deep_causality_tensor` as an `optional = true` dep plus a new `tensor-iso` feature `["dep:deep_causality_tensor"]` (NOT in default). Default builds compile zero tensor code.
- [x] 2.6 9 tests at `deep_causality_sparse/tests/extensions/ext_iso_tests.rs`: forward (3 incl. 2 panic-paths for rank 1 and rank 3), reverse (3 incl. iso vs. alias consistency), round-trip with independent inputs (2 incl. all-zero case). **Deviation**: tests live under `tests/extensions/` (alongside `ext_hkt_tests.rs`) following the same `ext_*` convention; cfg-gated behind `feature = "tensor-iso"`.
- [x] 2.7 `mod ext_iso_tests;` registered (cfg-gated) in `tests/extensions/mod.rs`. `rust_test_suite(name = "extensions", ...)` added to `tests/BUILD.bazel` with `crate_features = ["tensor-iso"]` and the matching deps. (Pre-existing gap: `extensions/` test_suite did not exist; this change adds it for both `ext_hkt_tests` and `ext_iso_tests`.)
- [x] 2.8 `cargo test -p deep_causality_sparse` (default, no feature): 86 + 19 = 105 tests pass; **zero tensor dep pulled** (verified via `cargo tree`). `cargo test -p deep_causality_sparse --features tensor-iso`: 95 + 19 + 9 new = 123 tests pass. `cargo clippy -p deep_causality_sparse --tests --features tensor-iso -- -D warnings`: clean.
- [x] 2.9 `bazel test //deep_causality_sparse/tests:extensions`: both `ext_hkt_tests_test` and `ext_iso_tests_test` PASS. `bazel test //deep_causality_sparse/tests:all`: 10/10 pass. **Bazel BUILD.bazel update**: `crate_features = ["std", "tensor-iso"]` on the lib (Bazel users get the iso unconditionally; this is consistent with Bazel's explicit-deps model where the dep graph is hand-curated anyway).
- [x] 2.10 `cargo build -p deep_causality_sparse --no-default-features` passes (no iso, no tensor dep).
- [x] 2.11 `cargo fmt -p deep_causality_sparse --check`: clean.
- [x] 2.12 **Stage B gate:** stage-completion summary below; awaiting sign-off.

## 3. Part C — `iso-num-multivector`: Complex <-> Cl(0,1) and Quaternion <-> Cl(3,0)-even — **POSTPONED**

> **Status:** POSTPONED during the implement-isomorphism change. Stage was rolled back without landing.

### Reason for postponement

The Stage C spec assumed `CausalMultiVector<F>` already implemented the `deep_causality_num` algebraic-trait hierarchy (`Group`, `Ring`, `Field`, `Algebra<F>`, `DivisionAlgebra<F>`, plus the marker traits `Associative`, `Commutative`, `Distributive`, `AbelianGroup`). During implementation it became clear that:

1. None of these impls exist on `CausalMultiVector<F>` today. Adding the full stack honestly is ~300-500 LoC of new trait impls (Neg, MulAssign<Self>, Div<Self>/DivAssign<Self>, Zero, One, conjugate, infallible inverse, plus the markers).
2. **`Field` cannot be honestly impl'd on `CausalMultiVector<F>` at all**, because `Field` requires `Commutative`, and the geometric product is non-commutative for most metrics (Cl(3,0), Cl(3,1), etc.) and commutative only for a few (Cl(0,1), Cl(1,0), Cl(0,0)). `Commutative` is a marker trait — either impl'd or not — and the metric is a **runtime** field on `CausalMultiVector<F>`, not a type parameter. We cannot make the impl conditional on runtime state.

Therefore the original spec requirement that `ComplexCl01Iso` satisfy `FieldIso<CausalMultiVector<F>, Complex<F>>` cannot be met without either:

- **Phantom-typed metrics** (`CausalMultiVector<F, M: MetricMarker>`): breaking change to every existing call site. Out of scope.
- **Per-algebra newtype wrappers** (`Cl01<F>`, `Cl30Even<F>`, etc.): cleaner mathematically but a separate design exercise. Out of scope for this change.

### What was done before rollback

Files were drafted at `deep_causality_multivector/src/extensions/iso_complex/mod.rs` and `iso_quaternion/mod.rs`. Compile failures uncovered the algebraic-trait gap. All files were removed; `src/extensions/mod.rs` and `src/lib.rs` reverted to the state before Stage C.

### Follow-up

The Cl(0,1)-Complex and Cl(3,0)-even-Quaternion isos remain real and useful. They should land in a future change that first introduces either (a) phantom-typed metrics on `CausalMultiVector`, or (b) per-algebra newtype wrappers. The decision between (a) and (b) belongs to that future change.

Per-task checklist preserved for the future change reference:

- [ ] 3.1 (postponed) ~~Create module~~ — pending newtype/phantom-metric prerequisite.
- [ ] 3.2 - 3.19 (postponed) ~~Stage C tasks 3.2-3.19~~ — see future change.

The `iso-num-multivector` capability spec file has been removed from this change. It belongs to the future change that lands the prerequisite work.

## 4. Part D — `iso-multifield-tensor`: `CausalMultiField<T>` <-> tuple

> **Gate:** Part B MUST be signed off and committed before any task here begins. (Part C was POSTPONED; this stage proceeds independently.)

- [x] 4.1 Module created at `deep_causality_multivector/src/extensions/iso_multifield/mod.rs`. **Deviation**: placed under `src/extensions/` per the established `iso_*` extension convention (mirrors `hkt_multifield/`); not under `src/iso/`.
- [x] 4.2 `impl<T> From<CausalMultiField<T>> for MultiFieldCarrier<T>` added. Move semantics; no copy.
- [x] 4.3 `impl<T> From<MultiFieldCarrier<T>> for CausalMultiField<T>` added. Struct-literal pack via `pub(crate)` fields.
- [x] 4.4 `Iso<CausalMultiField<T>, MultiFieldCarrier<T>>` satisfied via the `StandardIso<S, T>` blanket impl in `deep_causality_num`. No manual `Iso` or marker impls. **Bonus**: exposed `MultiFieldCarrier<T>` type alias for ergonomics.
- [x] 4.5 5 tests in `deep_causality_multivector/tests/extensions/iso_multifield/iso_multifield_tests.rs`: forward (2), reverse (1), round-trip via `assert_witness_iso_round_trip::<StandardIso<...>>` (1), byte-identity round-trip on a self-pair (1).
- [x] 4.6 `extensions/mod.rs` declares `pub mod iso_multifield;`. `lib.rs` re-exports `MultiFieldCarrier`. **Prerequisite**: added `#[derive(PartialEq)]` to `CausalMultiField<T>` (was previously `#[derive(Debug, Clone)]` only) so the round-trip helper's `S/T: PartialEq` bound can be satisfied.
- [x] 4.7 `cargo build -p deep_causality_multivector` and `cargo test -p deep_causality_multivector`: 347 + 3 = 350 tests pass (5 new + 345 existing). `cargo clippy -p deep_causality_multivector --tests -- -D warnings`: clean.
- [x] 4.8 `bazel test //deep_causality_multivector/tests:extensions`: 6/6 tests pass (5 prior + new `iso_multifield_tests_test`). Bazel `tests/BUILD.bazel` updated: added `extensions/iso_multifield/*_tests.rs` glob and `//deep_causality_metric`, `//deep_causality_num`, `//deep_causality_tensor` to deps.
- [x] 4.9 `cargo fmt -p deep_causality_multivector --check`: clean.
- [x] 4.10 **Stage D gate:** stage-completion summary below; awaiting sign-off.

## 5. Part E — `iso-topology`: simplicial/cell + Poincaré dual

> **Gate:** Part D MUST be signed off and committed before any task here begins.

### 5.A. SimplicialComplex <-> CellComplex<Simplex>

- [ ] 5.1 Create module `deep_causality_topology/src/iso/` with `mod.rs` and `simplicial_cell_iso.rs`.
- [ ] 5.2 Declare `impl<T> From<SimplicialComplex<T>> for CellComplex<Simplex>`. Iterate simplices; append each as a cell.
- [ ] 5.3 Declare `impl<T> TryFrom<CellComplex<Simplex>> for SimplicialComplex<T>`. Error variant for non-simplex cells. No panic.
- [ ] 5.4 Tests in `deep_causality_topology/tests/iso/simplicial_cell_tests.rs`: forward `From` success, `TryFrom` success path, `TryFrom` failure path on non-simplex input.

### 5.B. LatticeComplex<D> <-> DualLatticeComplex<D> (Poincaré)

- [ ] 5.5 Add `poincare_iso.rs` under `deep_causality_topology/src/iso/`.
- [ ] 5.6 Declare `pub struct PoincareIso<const D: usize>;` and `impl<const D: usize> Iso<LatticeComplex<D>, DualLatticeComplex<D>> for PoincareIso<D>`.
- [ ] 5.7 Implement `to_target`: for each k-cell in the primal, emit a (D-k)-cell in the dual; preserve incidence relations.
- [ ] 5.8 Implement `to_source`: inverse of `to_target` (same algorithm, dimensions reversed).
- [ ] 5.9 Tests in `deep_causality_topology/tests/iso/poincare_iso_tests.rs`: `assert_witness_iso_round_trip` for D ∈ {1, 2, 3}, plus a domain-specific `assert_poincare_dualizes_boundary` helper that validates the boundary <-> coboundary correspondence.

### 5.C. Wiring

- [ ] 5.10 Update `deep_causality_topology/src/iso/mod.rs` to re-export `PoincareIso`.
- [ ] 5.11 Update `deep_causality_topology/src/lib.rs` re-exports.
- [ ] 5.12 Register `mod iso;` in `deep_causality_topology/tests/mod.rs`. Add `rust_test_suite` entry to `deep_causality_topology/tests/BUILD.bazel`.
- [ ] 5.13 `cargo build -p deep_causality_topology` and `cargo test -p deep_causality_topology` pass. Clippy clean.
- [ ] 5.14 `bazel test //deep_causality_topology/tests:iso` passes.
- [ ] 5.15 Run `make format && make fix` — clean.
- [ ] 5.16 **Stage E gate (final):** stage-completion summary; sign-off; commit. Change is complete after Stage E lands.

## Stage gates

Per AGENTS.md golden rule §1 (NEVER commit) and the protocol established in `2026-05-20-add-iso-traits`:

- Each stage MUST be completed in full before the next stage begins.
- Stage completion criteria (binding): all stage tasks checked off; `cargo build -p <crate>` and `cargo test -p <crate>` green for the affected crate; `bazel test //...` for the new iso test suite green; `make format && make fix` clean; `cargo clippy -p <crate> -- -D warnings` produces no warnings; no_std build passes for trait declarations.
- Sign-off protocol: the agent presents a stage-completion summary listing changes, files touched, test evidence, and any deviations from spec. The user reviews and either (a) signs off in writing or (b) requests revisions.
- Commit protocol: after sign-off, the agent prepares a commit message; the user runs the commit. The next stage begins only after the commit lands.
- A stage that fails review returns to in-progress; the gate does not advance until the user re-approves.

## Sequencing rationale

Order chosen by ascending difficulty and dependency. **Part C was POSTPONED** mid-change after discovering an unmet prerequisite (no algebraic-trait impls on `CausalMultiVector`, and `Field` cannot be honestly impl'd without phantom-typed metrics or per-algebra newtype wrappers). The change ships Parts A, B, D, E.

1. **Part A** (effect/process consistency test). Smallest diff; test-only; warms up the test infrastructure in `deep_causality_core/tests/iso/`. **DONE.**
2. **Part B** (tensor/sparse). Canonical mixed-tier template. Establishes the feature-gated extension pattern (D2a). Worked example in `IsoFollowUps.md`. **DONE.**
3. ~~**Part C** (num/multivector).~~ **POSTPONED.** See Part C section above for the reason. Deferred to a future change that lands the algebraic-trait prerequisite (phantom metrics or per-algebra newtypes).
4. **Part D** (multifield/tensor). Trivial pack/unpack; uses `StandardIso`. No algebraic markers needed; proceeds independently of the postponed Stage C.
5. **Part E** (topology). The Poincaré dual is the only sub-task with non-trivial body algorithm; landed last so all the template patterns are established.
