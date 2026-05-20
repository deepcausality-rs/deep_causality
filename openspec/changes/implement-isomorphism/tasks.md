## 1. Part A — Propagating-effect / process `Functor`/`Monad` consistency test (no iso wrapper)

> **Gate:** This stage MUST be completed in full, verified, signed off, and committed before any task in Part B begins. See "Stage gates" at the end.

- [ ] 1.1 Create `deep_causality_core/tests/iso/` with `mod.rs` and `effect_process_consistency_tests.rs`. No source-tree changes.
- [ ] 1.2 Write a test that constructs a representative `CausalEffectPropagationProcess<i32, (), (), CausalityError, EffectLog>` value (e.g. via `PropagatingEffectWitness::pure(42)`) and asserts `<PropagatingEffectWitness<CausalityError, EffectLog>>::fmap(val.clone(), |x| x * 2)` is byte-identical to `<PropagatingProcessWitness<(), ()>>::fmap(val, |x| x * 2)`. The assertion message names both witnesses so the relationship is discoverable.
- [ ] 1.3 Add a second test for the type-changing case (`|x: i32| x % 2 == 0`).
- [ ] 1.4 Add a third test for `Monad::bind` (the `bind` impls are also written independently).
- [ ] 1.5 Register `mod iso;` in `deep_causality_core/tests/mod.rs`. Add `rust_test_suite(name = "iso", srcs = glob(["iso/*_tests.rs"]), ...)` to `deep_causality_core/tests/BUILD.bazel`.
- [ ] 1.6 `cargo test -p deep_causality_core` passes. `cargo clippy -p deep_causality_core --tests -- -D warnings` clean.
- [ ] 1.7 `bazel test //deep_causality_core/tests:iso` passes.
- [ ] 1.8 Run `make format && make fix` — clean across the workspace.
- [ ] 1.9 **Stage A gate:** prepare a stage-completion summary listing the test files added and the consistency properties pinned. Wait for sign-off and commit before proceeding.

## 2. Part B — `iso-tensor-sparse`: `CausalTensor<F>` <-> `CsrMatrix<F>` (mixed-tier template)

> **Gate:** Part A MUST be signed off and committed before any task here begins.

- [ ] 2.1 Create module `deep_causality_sparse/src/iso/` (or under `src/types/sparse_matrix/`) with `mod.rs`, `from_tensor.rs`, `iso.rs`, and `to_dense.rs`.
- [ ] 2.2 Declare `impl<F> From<CausalTensor<F>> for CsrMatrix<F> where F: Zero + PartialEq + Clone` in `from_tensor.rs`. Iterate row-major; emit triplets for non-zero values; panic on rank ≠ 2.
- [ ] 2.3 Declare `impl<F> Iso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F> where F: Zero + Clone` in `iso.rs`. `to_target` materialises the dense tensor; `to_source` delegates to the forward `From`.
- [ ] 2.4 Add `impl<F> CsrMatrix<F> { pub fn to_dense(self) -> CausalTensor<F> { ... } }` ergonomic alias in `to_dense.rs`.
- [ ] 2.5 Update `deep_causality_sparse/src/lib.rs` re-exports.
- [ ] 2.6 Create `deep_causality_sparse/tests/iso/` with `mod.rs` and `tensor_sparse_tests.rs`. Tests cover: forward `From`, reverse `to_dense()` / `Iso::to_target`, `assert_witness_iso_round_trip` with independent `(sparse, dense)` inputs, `#[should_panic]` test for rank ≠ 2 input on the forward direction.
- [ ] 2.7 Register `mod iso;` in `deep_causality_sparse/tests/mod.rs`. Add `rust_test_suite` entry to `deep_causality_sparse/tests/BUILD.bazel`.
- [ ] 2.8 `cargo build -p deep_causality_sparse` and `cargo test -p deep_causality_sparse` pass. Verify no new clippy warnings.
- [ ] 2.9 `bazel test //deep_causality_sparse/tests:iso` passes.
- [ ] 2.10 `cargo build -p deep_causality_sparse --no-default-features --features alloc` passes (or whatever the crate's no_std flag is).
- [ ] 2.11 Run `make format && make fix` — clean.
- [ ] 2.12 **Stage B gate:** stage-completion summary; sign-off; commit.

## 3. Part C — `iso-num-multivector`: Complex <-> Cl(0,1) and Quaternion <-> Cl(3,0)-even

> **Gate:** Part B MUST be signed off and committed before any task here begins.

### 3.A. Complex <-> Cl(0,1) multivector

- [ ] 3.1 Create module `deep_causality_multivector/src/iso/` with `mod.rs`, `complex_iso.rs`.
- [ ] 3.2 Declare `impl<F> From<Complex<F>> for CausalMultiVector<F>` in `complex_iso.rs`. Hard-code the Cl(0,1) metric. Embed: scalar = `re`, e₁ = `im`.
- [ ] 3.3 Declare `pub struct ComplexCl01Iso;` and `impl<F> Iso<CausalMultiVector<F>, Complex<F>> for ComplexCl01Iso`. `to_target` reads the scalar and e₁ coefficients; panics on wrong metric. `to_source` delegates to the forward `From`.
- [ ] 3.4 Declare marker impls: `impl<F> GroupIso<CausalMultiVector<F>, Complex<F>> for ComplexCl01Iso`, `impl<F> RingIso<...>`, `impl<F> FieldIso<...>`, `impl<F> AlgebraIso<..., F>`, `impl<F> DivisionAlgebraIso<..., F>`. Each empty body.
- [ ] 3.5 Tests in `deep_causality_multivector/tests/iso/complex_iso_tests.rs`: direct call assertions, `assert_witness_iso_round_trip`, `assert_witness_group_iso_law`, `assert_witness_ring_iso_laws`, `assert_witness_field_iso_laws`, `assert_witness_algebra_iso_law`, `assert_witness_division_algebra_iso_law`. Plus a `#[should_panic]` for wrong-metric input on `to_target`.

### 3.B. Quaternion <-> Cl(3,0)-even rotor

- [ ] 3.6 Create `deep_causality_multivector/src/iso/quaternion_iso.rs`.
- [ ] 3.7 Declare `impl<F> From<Quaternion<F>> for CausalMultiVector<F>`. Hard-code Cl(3,0) metric. Lift `w` to the scalar, `x/y/z` to the e₂e₃ / e₃e₁ / e₁e₂ bivectors (east-coast convention; doc-comment the choice).
- [ ] 3.8 Declare `impl<F> TryFrom<CausalMultiVector<F>> for Quaternion<F>`. Error variant for non-rotor inputs (any non-zero coefficient on vector or pseudoscalar basis blades). Returns `Err`, does NOT panic.
- [ ] 3.9 Declare `pub struct QuaternionRotorIso;` and `impl<F> Iso<CausalMultiVector<F>, Quaternion<F>> for QuaternionRotorIso`. `to_target` extracts the rotor coefficients (panics on non-rotor input — caller must have filtered via `TryFrom`); `to_source` delegates to forward `From`.
- [ ] 3.10 Declare marker impls: `GroupIso`, `RingIso`, `AlgebraIso<..., F>`, `DivisionAlgebraIso<..., F>`. Do NOT impl `FieldIso` (quaternions are non-commutative).
- [ ] 3.11 Tests in `deep_causality_multivector/tests/iso/quaternion_iso_tests.rs`: forward `From` direct calls, `TryFrom` success path, `TryFrom` failure path on non-rotor input, `assert_witness_iso_round_trip` on rotor input, `assert_witness_ring_iso_laws`, `assert_witness_algebra_iso_law`, `assert_witness_division_algebra_iso_law`. Plus a `#[should_panic]` for non-rotor input on the Tier 2 `to_target`.

### 3.C. Wiring

- [ ] 3.12 Update `deep_causality_multivector/src/iso/mod.rs` to re-export `ComplexCl01Iso` and `QuaternionRotorIso`.
- [ ] 3.13 Update `deep_causality_multivector/src/lib.rs` re-exports.
- [ ] 3.14 Register `mod iso;` in `deep_causality_multivector/tests/mod.rs`. Add `rust_test_suite` entry to `deep_causality_multivector/tests/BUILD.bazel`.
- [ ] 3.15 `cargo build -p deep_causality_multivector` and `cargo test -p deep_causality_multivector` pass. Verify no new clippy warnings.
- [ ] 3.16 `bazel test //deep_causality_multivector/tests:iso` passes.
- [ ] 3.17 `cargo build -p deep_causality_multivector --no-default-features` passes (or the crate's no_std flag).
- [ ] 3.18 Run `make format && make fix` — clean.
- [ ] 3.19 **Stage C gate:** stage-completion summary; sign-off; commit.

## 4. Part D — `iso-multifield-tensor`: `CausalMultiField<T>` <-> tuple

> **Gate:** Part C MUST be signed off and committed before any task here begins.

- [ ] 4.1 Add `multifield_iso.rs` under `deep_causality_multivector/src/iso/`.
- [ ] 4.2 Declare `impl<T> From<CausalMultiField<T>> for (CausalTensor<T>, Metric, [T; 3], [usize; 3])`. Unpack the four fields directly (move semantics; no copying).
- [ ] 4.3 Declare `impl<T> From<(CausalTensor<T>, Metric, [T; 3], [usize; 3])> for CausalMultiField<T>`. Pack the tuple into the multifield without validation.
- [ ] 4.4 The pair satisfies `Iso<...>` via the existing `StandardIso<S, T>` blanket impl; no manual marker impls required.
- [ ] 4.5 Tests in `deep_causality_multivector/tests/iso/multifield_iso_tests.rs`: direct unpack/pack, `assert_witness_iso_round_trip::<StandardIso<...>, _, _>` with representative inputs.
- [ ] 4.6 Update `deep_causality_multivector/src/iso/mod.rs` re-exports.
- [ ] 4.7 `cargo build -p deep_causality_multivector` and `cargo test -p deep_causality_multivector` pass. Clippy clean.
- [ ] 4.8 `bazel test //deep_causality_multivector/tests:iso` passes.
- [ ] 4.9 Run `make format && make fix` — clean.
- [ ] 4.10 **Stage D gate:** stage-completion summary; sign-off; commit.

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

Order chosen by ascending difficulty and dependency:

1. **Part A** (effect/process consistency test). Smallest diff; test-only; warms up the test infrastructure in `deep_causality_core/tests/iso/`.
2. **Part B** (tensor/sparse). Canonical mixed-tier template. Worked example in `IsoFollowUps.md`.
3. **Part C** (num/multivector). First real exercise of full marker-subtrait coverage (`FieldIso`, `DivisionAlgebraIso`). Two pairs; Cl(0,1) is the simpler warm-up before the Cl(3,0)-rotor.
4. **Part D** (multifield/tensor). Trivial pack/unpack; uses `StandardIso`. Could land earlier but kept after C so the multivector crate's iso surface is established first.
5. **Part E** (topology). The Poincaré dual is the only sub-task with non-trivial body algorithm; landed last so all the template patterns are established.
