## 1. Block H0 — Preflight audit

- [ ] 1.1 Verify `add-cubical-regge-calculus-analytical` has shipped with its Block 0 spec refinement complete (proposal + design use `R: RealField`, `HasHodgeStar<R>` is the locked trait shape). If not, this change set cannot open.
- [ ] 1.2 Audit `deep_causality_sparse` and `deep_causality_topology` for any pre-existing CG solver. Record the finding in `design.md` Open Question 1. If a reusable solver exists, plan to consume it; otherwise plan to land the minimal ~80 LOC CG in this change set.
- [ ] 1.3 Verify whether `deep_causality_num::RealField` exposes `default_epsilon()` (or equivalent). Record in `design.md` Open Question 2.
- [ ] 1.4 Decide the crate boundary for the CG solver: `deep_causality_sparse` (preferred if the solver lands fresh) versus `deep_causality_topology` (acceptable if private). Document the decision in `design.md` Risk 2 and confirm at H0 review.
- [ ] 1.5 H0-G3 Review: user signs off on the three preflight resolutions before any code lands. (PyDEC parity is deferred to a follow-up change set per the "Deferred Work" section below; no PyDEC pin is required for H1–H3.)

## 2. Block H1 — Carrier type, error variants, getters

- [ ] 2.1 Create `deep_causality_topology/src/types/hodge_decomposition/mod.rs` with the `HodgeDecomposition<R: RealField>` struct (private fields `exact`, `co_exact`, `harmonic` of type `CausalTensor<R>` and `grade: usize`) and the constructor `pub fn new(exact, co_exact, harmonic, grade) -> Self`.
- [ ] 2.2 Create `deep_causality_topology/src/types/hodge_decomposition/getters.rs` with `exact()`, `co_exact()`, `harmonic()`, `grade()` returning borrowed views.
- [ ] 2.3 Create `deep_causality_topology/src/types/hodge_decomposition/display.rs` implementing `Debug` and `Display` per the existing project convention for `CausalTensor`-carrying types.
- [ ] 2.4 Create `deep_causality_topology/src/types/hodge_decomposition/part_eq.rs` implementing `PartialEq` with the tolerance-based tensor equality used elsewhere in the crate.
- [ ] 2.5 Register the new submodules in `src/types/hodge_decomposition/mod.rs` and re-export `HodgeDecomposition` from `src/lib.rs`.
- [ ] 2.6 Extend `TopologyErrorEnum` (in `src/errors/topology_error.rs`) with a new variant `HodgeDecompositionFailed(String)`, matching the existing stringly-typed convention used by `ManifoldError(String)` and the other variants. Add the constructor function `pub fn HodgeDecompositionFailed<S: Into<String>>(msg: S) -> Self` on `TopologyError`, mirroring the existing `ManifoldError(msg)` constructor pattern. No new file under `src/errors/`; no public `HodgeFailReason` enum is added.
- [ ] 2.7 Add a **private** enum `HodgeFailReason<R: RealField>` inside `src/types/manifold/differential/hodge_decomposition_impl.rs` (created in H2.3) with the four control-flow variants `Nonconvergence { iterations: usize, residual: R }`, `GradeOutOfRange { k: usize, max_dim: usize }`, `DimensionMismatch { expected: usize, actual: usize }`, `MissingMetric`. Provide `fn into_topology_error(self) -> TopologyError where R: Display` that formats each variant into `TopologyError::HodgeDecompositionFailed(...)` with a discriminating substring. The enum MUST stay private to the module; it does NOT cross the crate boundary. (This task is bookkept under H1 even though the file lands in H2, so the gating is honest.)
- [ ] 2.8 Update the existing `tests/errors/topology_error_tests.rs` to cover the new `HodgeDecompositionFailed(String)` variant: constructor returns the right variant; `Display` formats correctly; equality / inequality vs other variants.
- [ ] 2.9 Create test files under `tests/types/hodge_decomposition/` mirroring the source tree: `mod_tests.rs`, `getters_tests.rs`, `display_tests.rs`, `part_eq_tests.rs`. Register in `tests/types/mod.rs` and `tests/BUILD.bazel`.
- [ ] 2.10 (removed — was `HodgeFailReason` standalone test file; no longer applies)
- [ ] 2.11 Unit tests covering: construction with prescribed dimensions; getter return values; `Debug` and `Display` formatting; `PartialEq` reflexivity / symmetry / transitivity; tolerance-based equality boundary. The private `HodgeFailReason<R>` is exercised indirectly through H2's `hodge_decompose` error-path tests.
- [ ] 2.12 H1-G1 Compilation: `cargo build -p deep_causality_topology` clean (release + debug); `cargo clippy -p deep_causality_topology --all-targets -- -D warnings` clean. Fix lints at root cause; no `#[allow(clippy::...)]` suppressions per `feedback_clippy_lints`.
- [ ] 2.13 H1-G2 Coverage: 100% on every new file under `src/types/hodge_decomposition/` and `src/errors/`; 100% on every modified file (`src/lib.rs`, `src/errors/mod.rs`).
- [ ] 2.14 H1-G3 Review: user reviews the diff, runs `make format && make fix`, signs off, commits. Agents never run `git commit`.

## 3. Block H2 — Decomposition algorithm

- [ ] 3.1 Decide and land the CG solver per H0.2 / H0.4. If new, create `src/utils/cg_solver.rs` (if private to topology) or coordinate a small additive change to `deep_causality_sparse`. Generic over `R: RealField + FromPrimitive`. ~80 LOC. Includes range-projection of the RHS to handle the singular kernel of `Δ_k` per `design.md` Risk 1.
- [ ] 3.2 Add `pub struct HodgeDecomposeOptions<R: RealField> { tolerance: Option<R>, max_iterations: Option<usize> }` plus a `Default` impl that returns `None` / `None` (defaults resolved inside `hodge_decompose`).
- [ ] 3.3 Implement `Manifold::hodge_decompose` under `src/types/manifold/differential/hodge_decomposition_impl.rs` with the trait bounds `K: ChainComplex, K::Metric: HasHodgeStar<R, Complex = K>, R: RealField + FromPrimitive + Display` (the `Display` bound is consumed exclusively by `HodgeFailReason::into_topology_error`). Signature per `spec.md`: `pub fn hodge_decompose(&self, field: &CausalTensor<R>, k: usize) -> Result<HodgeDecomposition<R>, TopologyError>` plus an `_opts` variant accepting `HodgeDecomposeOptions<R>`. Error construction goes through the private `HodgeFailReason<R>::into_topology_error` shim landed in H1.2.7 — no other error path is permitted.
- [ ] 3.4 Implement the three-step algorithm per `design.md` Decision 2:
  - [ ] 3.4.1 Validate inputs: `k <= max_dim` else return `HodgeFailReason::GradeOutOfRange { k, max_dim }.into_topology_error()`; `field.len() == num_cells(k)` else return `HodgeFailReason::DimensionMismatch { expected, actual }.into_topology_error()`; metric present else return `HodgeFailReason::MissingMetric.into_topology_error()`.
  - [ ] 3.4.2 Build the discrete Laplacian `Δ_k = δ d + d δ` via the existing generic operators landed by `add-cubical-regge-calculus-analytical` R4.
  - [ ] 3.4.3 Compute `δω` (RHS for the α-potential). Project onto the range of `Δ_k` to handle the singular kernel.
  - [ ] 3.4.4 Solve `Δ_k φ_α = δω` via CG. On failure, return `HodgeFailReason::Nonconvergence { iterations, residual }.into_topology_error()`.
  - [ ] 3.4.5 For grade 0, fix the gauge: subtract the mean from `φ_α` per `design.md` Risk 4.
  - [ ] 3.4.6 Compute `α = d φ_α` (the exact component).
  - [ ] 3.4.7 Repeat the analogous steps for `β`: compute `dω`, project, solve `Δ_k ψ_β = dω`, compute `β = δ ψ_β`.
  - [ ] 3.4.8 Compute `h = ω − α − β` (the harmonic residual).
  - [ ] 3.4.9 Wrap in `HodgeDecomposition::new(α, β, h, k)` and return `Ok(...)`.
- [ ] 3.5 Register the new module file in `src/types/manifold/differential/mod.rs`.
- [ ] 3.6 Create `tests/types/manifold/differential/hodge_decomposition_impl_tests.rs`. Register in the corresponding `mod.rs` and `tests/BUILD.bazel`.
- [ ] 3.7 Unit tests covering every input-validation branch (`GradeOutOfRange`, `DimensionMismatch`, `MissingMetric`, `Nonconvergence` triggered by an artificially low iteration cap).
- [ ] 3.8 Unit tests for the CG solver (if landed in this change set): convergence on a small symmetric positive-definite system; singular-kernel projection on a known-rank-deficient system; behaviour at the iteration cap.
- [ ] 3.9 Smoke test: decompose a trivial pure-exact 1-form on a `LatticeComplex<2>` and verify each component's L2 norm matches the analytic expectation (exact ≈ ‖field‖, co-exact ≈ 0, harmonic ≈ 0). Full property tests live in H3.
- [ ] 3.10 H2-G1 Compilation: clean across `deep_causality_topology` (and `deep_causality_sparse` if the CG solver landed there).
- [ ] 3.11 H2-G2 Coverage: 100% on every new and modified file. Every `match` arm and every error path is exercised.
- [ ] 3.12 H2-G3 Review: user reviews, signs off, commits.

## 4. Block H3 — Property tests, two-backend cross-check

- [ ] 4.1 Property test: pure exact 1-form decomposes with `‖co_exact‖² < ε_R` and `‖harmonic‖² < ε_R`. Tested on `LatticeComplex<2>`, `LatticeComplex<3>` (`Euclidean`) and on `SimplicialComplex` of equivalent topology. Run across `R ∈ {f32, f64, DoubleFloat}`.
- [ ] 4.2 Property test: pure co-exact 1-form decomposes with `‖exact‖² < ε_R` and `‖harmonic‖² < ε_R`. Same backend / precision matrix as 4.1.
- [ ] 4.3 Property test: Hodge orthogonality identity `‖exact‖² + ‖co_exact‖² + ‖harmonic‖² == ‖field‖²` on grids `4³`, `8³`, `16³`, across all three precision backends.
- [ ] 4.4 Property test: two-backend cross-check on the unit square. Same prescribed scalar field f(x, y) = 2x + 3y, ω = df, decomposed via `ReggeGeometry<R>` (single right triangle) and via `CubicalReggeGeometry<2, R, Euclidean>` (2x2 lattice). Both backends must satisfy the orthogonality identity at `1e-6`, agree on the vanishing-component ratio `(‖β‖² + ‖h‖²) / ‖ω‖²` to `1e-6`, and individually report that ratio at `< 1e-6` for the pure-exact input.
  - The strict per-component L2 norm equality variant of this scenario (`|‖simplicial.exact()‖ − ‖cubical.exact()‖| < 1e-6` etc.) remains future work pending a Delaunay/manifold-respecting triangulation in `PointCloud`. The H3 ambient-dim cap on `PointCloud::triangulate` (commit `fix(point_cloud): cap triangulate top-grade at ambient dimension`) closes the related degenerate-3-simplex / lumped-mass-M_0 collapse bug; the remaining Vietoris-Rips → non-manifold gap is documented in `design.md` Risk 5.
- [ ] 4.5 Property test: every new public signature is generic over `R: RealField`. Verified by a small `compile-pass` test that instantiates `HodgeDecomposition<f32>`, `HodgeDecomposition<f64>`, `HodgeDecomposition<DoubleFloat>` and calls `hodge_decompose` at each precision. Failure means a precision constraint regressed somewhere.
- [ ] 4.6 Static check: grep the diff for `f64` in public signatures (`pub fn`, `pub struct`, `pub trait`, `pub enum`). Zero hits is a hard gate.
- [ ] 4.7 H3-G1 Compilation: clean across `deep_causality_topology` (and `deep_causality_sparse` if touched).
- [ ] 4.8 H3-G2 Coverage: 100% on every new and modified file. Long-running property tests (e.g. the 16³ grid) are feature-gated behind `--features long-running-tests` if their runtime exceeds the project's per-test budget.
- [ ] 4.9 H3-G3 Review: user reviews the full diff across H1+H2+H3, runs `make format && make fix && make build && make test` (since multiple files are touched), signs off, commits. This is the change-set-closing commit; after H3-G3 the change set is ready to archive and `add-3d-causal-fluid-dynamics` Block B1 can open.

## 5. Deferred Work — PyDEC parity benchmark (future change set)

PyDEC parity is deferred to a dedicated follow-up change set `add-hodge-decomposition-pydec-parity` opened after H3-G3 closes. Rationale: PyDEC parity requires a one-time off-line Python run to hand-derive reference values, which is an orthogonal workstream from the algorithm + property-test work in H1–H3. Splitting it out keeps this change set Python-touchless and lets the algorithm land on internal consistency gates (two-backend cross-check + Hodge orthogonality identity + analytic pure-exact / pure-co-exact tests) alone.

The follow-up change set will cover:

- [ ] 5.1 Pin a specific PyDEC release (version + git SHA) to source the fixture values from.
- [ ] 5.2 Derive PyDEC reference values by hand from the pinned PyDEC source for three configurations on the unit square: pure-exact 1-form (simplicial), mixed 1-form (simplicial), mixed 1-form (cubical, derived via two-backend correspondence).
- [ ] 5.3 Create `tests/types/hodge_decomposition/pydec_fixtures.rs` holding the reference values as `const` arrays. File header records the PyDEC version + SHA + the date the fixture was hand-derived.
- [ ] 5.4 Property test: `hodge_decompose` output matches each PyDEC fixture value to relative error `< 1e-5` at `f64` precision.

This change set does not block; `add-3d-causal-fluid-dynamics` Block B1 may open immediately after H3-G3 of the present change set.

## 6. Deferred Work — strict per-component L2 cross-backend agreement (future change set)

The original `spec.md` task 4.4 scenario `|‖simplicial.exact()‖ − ‖cubical.exact()‖| < 1e-6` (and likewise for `co_exact` and `harmonic`) requires a manifold-respecting two-triangle simplicial decomposition of the unit square. The present change set's H3 cross-backend test verifies the orthogonality identity and the vanishing-component-ratio scenario at `1e-6`, both on a single-right-triangle simplicial fixture, because `PointCloud::triangulate`'s Vietoris-Rips clique expansion produces a non-manifold 4-triangle complex on the 4 coplanar corners of the unit square. See `design.md` Risk 5.

The follow-up change set `add-hodge-decomposition-delaunay-cross-backend` would cover:

- [ ] 6.1 Add a Delaunay (or constrained-Delaunay) triangulation method to `PointCloud`, producing a manifold-respecting simplicial complex from a planar point set.
- [ ] 6.2 Replace the H3 single-right-triangle simplicial fixture with the canonical two-triangle simplicial unit square.
- [ ] 6.3 Tighten the cross-backend test to assert the strict per-component L2 norm equality variant of `spec.md` task 4.4.

This change set does not block downstream consumers; the H3 cross-backend coverage already satisfies "discretisation independence" for orthogonality and vanishing-component structure.
