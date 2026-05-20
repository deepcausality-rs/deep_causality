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

## 5. Part E — `iso-topology`: simplicial/cell + Poincaré dual — **POSTPONED**

> **Status:** POSTPONED during the implement-isomorphism change. Both sub-isos were dropped before implementation after a feasibility audit revealed structural mismatches with the spec.

### Reason for postponement

**5.A. SimplicialComplex <-> CellComplex<Simplex> is not an iso.** The two types are structurally distinct, not different representations of the same data:

- `SimplicialComplex<T>` carries `Vec<Skeleton>` plus three sets of pre-computed sparse matrices (`boundary_operators: Vec<CsrMatrix<i8>>`, `coboundary_operators: Vec<CsrMatrix<i8>>`, `hodge_star_operators: Vec<CsrMatrix<T>>`).
- `CellComplex<C>` is `Vec<Vec<C>>` — dimension-stratified cell lists, no matrices.

Forward (`From<SimplicialComplex<T>> for CellComplex<Simplex>`) is **lossy**: drops all three matrix sets and the `T` parameter. Reverse (`TryFrom<CellComplex<Simplex>> for SimplicialComplex<T>`) has to **fabricate** the matrices; boundary and coboundary can be computed from simplicial structure, but Hodge-star matrices need T-valued metric data that the cell complex doesn't carry — would produce `SimplicialComplex<()>` at best. Round-trip loses information in both directions; the pair fails the iso bijectivity test.

These are useful **conversions**, but not isomorphisms. They belong in a separate change focused on inter-topology-type conversions, not in `implement-isomorphism`.

**5.B. LatticeComplex<D> <-> DualLatticeComplex<D> Poincaré is too trivial to justify the machinery.** The actual `DualLatticeComplex<D>` is `{ primal: Arc<LatticeComplex<D>> }` — a thin `Arc`-wrapper. There are no materialised cells in either struct; lattice cells are computed on demand from `shape`. The iso at the data level reduces to `wrap` / `unwrap`. The "Poincaré dual k-cell <-> (D-k)-cell mapping" the spec scenarios called for is a statement about how operations on the dual interpret indices — not about data transformation. Implementing the iso adds the witness type but no real algorithmic content; the spec's `assert_poincare_dualizes_boundary` test would be testing operation semantics on the existing API, not iso correctness.

### What was done before postponement

Nothing in source. The feasibility audit happened before any file was created in `deep_causality_topology/`. No code rollback needed.

### Follow-up

The simplicial/cell conversions could land in a separate change focused on inter-topology-type conversions (not iso). The Poincaré-dual iso could land if a future consumer materialises cells separately in either lattice or dual, but the current API doesn't justify it.

The `iso-topology` capability spec file has been removed from this change.

Per-task checklist preserved for the future-change reference:

- [ ] 5.1 - 5.16 (postponed) ~~Stage E tasks~~ — see Stage E section above for the structural-mismatch reason.

## Stage gates

Per AGENTS.md golden rule §1 (NEVER commit) and the protocol established in `2026-05-20-add-iso-traits`:

- Each stage MUST be completed in full before the next stage begins.
- Stage completion criteria (binding): all stage tasks checked off; `cargo build -p <crate>` and `cargo test -p <crate>` green for the affected crate; `bazel test //...` for the new iso test suite green; `make format && make fix` clean; `cargo clippy -p <crate> -- -D warnings` produces no warnings; no_std build passes for trait declarations.
- Sign-off protocol: the agent presents a stage-completion summary listing changes, files touched, test evidence, and any deviations from spec. The user reviews and either (a) signs off in writing or (b) requests revisions.
- Commit protocol: after sign-off, the agent prepares a commit message; the user runs the commit. The next stage begins only after the commit lands.
- A stage that fails review returns to in-progress; the gate does not advance until the user re-approves.

## Sequencing rationale

**Parts C and E were POSTPONED** mid-change after feasibility audits exposed structural mismatches between the spec and the substrate types. The change ships **Parts A, B, and D** — three concrete iso instances plus the propagating-effect consistency test.

1. **Part A** (effect/process consistency test). Smallest diff; test-only; warms up the test infrastructure in `deep_causality_core/tests/iso/`. **DONE.**
2. **Part B** (tensor/sparse). Canonical mixed-tier template. Establishes the feature-gated extension pattern (D2a). Worked example in `IsoFollowUps.md`. **DONE.**
3. ~~**Part C** (num/multivector).~~ **POSTPONED.** `CausalMultiVector<F>` lacks the algebraic-trait stack, and `Field` cannot be honestly impl'd because `Commutative` is metric-dependent (and the metric is a runtime field, not a type parameter). Unlocking needs either phantom-typed metrics or per-algebra newtypes — both out of scope.
4. **Part D** (multifield/tensor). Structural pack/unpack via `StandardIso`. No algebraic markers needed; proceeded independently of the postponed Stage C. **DONE.**
5. ~~**Part E** (topology).~~ **POSTPONED.** `SimplicialComplex<T>` <-> `CellComplex<Simplex>` is lossy in both directions (not an iso); `LatticeComplex<D>` <-> `DualLatticeComplex<D>` reduces to a trivial Arc-wrap because cells aren't materialised in either struct. Neither pair clears the iso bar with the current type definitions. See Stage E section.
