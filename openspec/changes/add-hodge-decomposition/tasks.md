## 1. Block H0 — Preflight audit

- [ ] 1.1 Verify `add-cubical-regge-calculus-analytical` has shipped with its Block 0 spec refinement complete (proposal + design use `R: RealField`, `HasHodgeStar<R>` is the locked trait shape). If not, this change set cannot open.
- [ ] 1.2 Audit `deep_causality_sparse` and `deep_causality_topology` for any pre-existing CG solver. Record the finding in `design.md` Open Question 1. If a reusable solver exists, plan to consume it; otherwise plan to land the minimal ~80 LOC CG in this change set.
- [ ] 1.3 Verify whether `deep_causality_num::RealField` exposes `default_epsilon()` (or equivalent). Record in `design.md` Open Question 2.
- [ ] 1.4 Decide the crate boundary for the CG solver: `deep_causality_sparse` (preferred if the solver lands fresh) versus `deep_causality_topology` (acceptable if private). Document the decision in `design.md` Risk 2 and confirm at H0 review.
- [ ] 1.5 Pin a specific PyDEC release (version + git SHA) to source the fixture values from. Record in `design.md` Open Question 4.
- [ ] 1.6 H0-G3 Review: user signs off on the four preflight resolutions before any code lands.

## 2. Block H1 — Carrier type, error variants, getters

- [ ] 2.1 Create `deep_causality_topology/src/types/hodge_decomposition/mod.rs` with the `HodgeDecomposition<R: RealField>` struct (private fields `exact`, `co_exact`, `harmonic` of type `CausalTensor<R>` and `grade: usize`) and the constructor `pub fn new(exact, co_exact, harmonic, grade) -> Self`.
- [ ] 2.2 Create `deep_causality_topology/src/types/hodge_decomposition/getters.rs` with `exact()`, `co_exact()`, `harmonic()`, `grade()` returning borrowed views.
- [ ] 2.3 Create `deep_causality_topology/src/types/hodge_decomposition/display.rs` implementing `Debug` and `Display` per the existing project convention for `CausalTensor`-carrying types.
- [ ] 2.4 Create `deep_causality_topology/src/types/hodge_decomposition/part_eq.rs` implementing `PartialEq` with the tolerance-based tensor equality used elsewhere in the crate.
- [ ] 2.5 Register the new submodules in `src/types/hodge_decomposition/mod.rs` and re-export `HodgeDecomposition` from `src/lib.rs`.
- [ ] 2.6 Add `HodgeFailReason` enum under `src/errors/hodge_fail_reason.rs` with the four variants from `spec.md` (`Nonconvergence`, `GradeOutOfRange`, `DimensionMismatch`, `MissingMetric`). Generic over `R` if `Nonconvergence` carries an `R`-typed residual; otherwise the variant boxes the residual via a precision-erased shim documented at the call site.
- [ ] 2.7 Extend the existing `ManifoldError` enum with `HodgeDecompositionFailed { reason: HodgeFailReason }`. Add `From<HodgeFailReason> for ManifoldError`.
- [ ] 2.8 Register `HodgeFailReason` in `src/errors/mod.rs` and re-export from `src/lib.rs`.
- [ ] 2.9 Create test files under `tests/types/hodge_decomposition/` mirroring the source tree: `mod_tests.rs`, `getters_tests.rs`, `display_tests.rs`, `part_eq_tests.rs`. Register in `tests/types/mod.rs` and `tests/BUILD.bazel`.
- [ ] 2.10 Create `tests/errors/hodge_fail_reason_tests.rs`; register in `tests/errors/mod.rs` and `tests/BUILD.bazel`.
- [ ] 2.11 Unit tests covering: construction with prescribed dimensions; getter return values; `Debug` and `Display` formatting; `PartialEq` reflexivity / symmetry / transitivity; tolerance-based equality boundary; every `HodgeFailReason` variant constructs, prints, and pattern-matches without a catch-all.
- [ ] 2.12 H1-G1 Compilation: `cargo build -p deep_causality_topology` clean (release + debug); `cargo clippy -p deep_causality_topology --all-targets -- -D warnings` clean. Fix lints at root cause; no `#[allow(clippy::...)]` suppressions per `feedback_clippy_lints`.
- [ ] 2.13 H1-G2 Coverage: 100% on every new file under `src/types/hodge_decomposition/` and `src/errors/`; 100% on every modified file (`src/lib.rs`, `src/errors/mod.rs`).
- [ ] 2.14 H1-G3 Review: user reviews the diff, runs `make format && make fix`, signs off, commits. Agents never run `git commit`.

## 3. Block H2 — Decomposition algorithm

- [ ] 3.1 Decide and land the CG solver per H0.2 / H0.4. If new, create `src/utils/cg_solver.rs` (if private to topology) or coordinate a small additive change to `deep_causality_sparse`. Generic over `R: RealField + FromPrimitive`. ~80 LOC. Includes range-projection of the RHS to handle the singular kernel of `Δ_k` per `design.md` Risk 1.
- [ ] 3.2 Add `pub struct HodgeDecomposeOptions<R: RealField> { tolerance: Option<R>, max_iterations: Option<usize> }` plus a `Default` impl that returns `None` / `None` (defaults resolved inside `hodge_decompose`).
- [ ] 3.3 Implement `Manifold::hodge_decompose` under `src/types/manifold/differential/hodge_decomposition_impl.rs` with the trait bounds `K: ChainComplex, K::Metric: HasHodgeStar<R>, R: RealField + FromPrimitive`. Signature per `spec.md`: `pub fn hodge_decompose(&self, field: &CausalTensor<R>, k: usize) -> Result<HodgeDecomposition<R>, ManifoldError>` plus an `_opts` variant accepting `HodgeDecomposeOptions<R>`.
- [ ] 3.4 Implement the three-step algorithm per `design.md` Decision 2:
  - [ ] 3.4.1 Validate inputs: `k <= max_dim` else `Err(GradeOutOfRange)`; `field.len() == num_cells(k)` else `Err(DimensionMismatch)`; metric present else `Err(MissingMetric)`.
  - [ ] 3.4.2 Build the discrete Laplacian `Δ_k = δ d + d δ` via the existing generic operators landed by `add-cubical-regge-calculus-analytical` R4.
  - [ ] 3.4.3 Compute `δω` (RHS for the α-potential). Project onto the range of `Δ_k` to handle the singular kernel.
  - [ ] 3.4.4 Solve `Δ_k φ_α = δω` via CG. On failure, return `Err(Nonconvergence { iterations, residual })`.
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

## 4. Block H3 — Property tests, two-backend cross-check, PyDEC parity

- [ ] 4.1 Property test: pure exact 1-form decomposes with `‖co_exact‖² < ε_R` and `‖harmonic‖² < ε_R`. Tested on `LatticeComplex<2>`, `LatticeComplex<3>` (`Euclidean`) and on `SimplicialComplex` of equivalent topology. Run across `R ∈ {f32, f64, DoubleFloat}`.
- [ ] 4.2 Property test: pure co-exact 1-form decomposes with `‖exact‖² < ε_R` and `‖harmonic‖² < ε_R`. Same backend / precision matrix as 4.1.
- [ ] 4.3 Property test: Hodge orthogonality identity `‖exact‖² + ‖co_exact‖² + ‖harmonic‖² == ‖field‖²` on grids `4³`, `8³`, `16³`, across all three precision backends.
- [ ] 4.4 Property test: two-backend cross-check on the unit square. Same prescribed 1-form decomposed via `ReggeGeometry<R>` (two triangles) and via `CubicalReggeGeometry<2, R, Euclidean>` (one 2-cube). Agreement on each component L2 norm to tolerance `1e-6` at `f64`.
- [ ] 4.5 Derive PyDEC reference values by hand from the pinned PyDEC source (version + SHA captured in H0.5) for the three configurations defined in `spec.md` ("PyDEC parity benchmark on the unit square"):
  - [ ] 4.5.1 Pure-exact 1-form on simplicial unit square.
  - [ ] 4.5.2 Mixed 1-form (non-trivial co-exact) on simplicial unit square.
  - [ ] 4.5.3 Mixed 1-form on cubical unit square (reference values derived from the simplicial PyDEC fixture per the two-backend cross-check; documented as such).
- [ ] 4.6 Create `tests/types/hodge_decomposition/pydec_fixtures.rs` holding the reference values as `const` arrays. File header records the PyDEC version + SHA + the date the fixture was hand-derived.
- [ ] 4.7 Property test: `hodge_decompose` output matches each PyDEC fixture value to relative error `< 1e-5` at `f64` precision.
- [ ] 4.8 Property test: every new public signature is generic over `R: RealField`. Verified by a small `compile-pass` test that instantiates `HodgeDecomposition<f32>`, `HodgeDecomposition<f64>`, `HodgeDecomposition<DoubleFloat>` and calls `hodge_decompose` at each precision. Failure means a precision constraint regressed somewhere.
- [ ] 4.9 Static check: grep the diff for `f64` in public signatures (`pub fn`, `pub struct`, `pub trait`, `pub enum`). Zero hits is a hard gate.
- [ ] 4.10 H3-G1 Compilation: clean across `deep_causality_topology` (and `deep_causality_sparse` if touched).
- [ ] 4.11 H3-G2 Coverage: 100% on every new and modified file. Long-running property tests (e.g. the 16³ grid) are feature-gated behind `--features long-running-tests` if their runtime exceeds the project's per-test budget.
- [ ] 4.12 H3-G3 Review: user reviews the full diff across H1+H2+H3, runs `make format && make fix && make build && make test` (since multiple files are touched), signs off, commits. This is the change-set-closing commit; after H3-G3 the change set is ready to archive and `add-3d-causal-fluid-dynamics` Block B1 can open.
