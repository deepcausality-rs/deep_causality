Coverage convention (AGENTS.md §"Code testing"): every task group targets 100% test
coverage of all added or edited files — including every error variant, every
`Err`/rejection branch, every periodic-vs-open and default-vs-override code path, and
the `Debug`/`Display`/`PartialEq` trait impls of new types. Unreachable code is the
only exemption and must be annotated and justified. Every test file is registered in
its `mod.rs` chain with `#[cfg(test)]` and in `tests/BUILD.bazel`.

## 1. G1+G4 — Wedge product, interior product, pinned conventions (deep_causality_topology)

- [x] 1.1 Write the convention-pinning tests first: `Δ_dR = −∇²` single-mode sine test on a torus; boundary-orientation fixture used by the cup-product tests (design D2)
- [x] 1.2 Implement the cubical cup-product `wedge` on `Manifold<LatticeComplex<D, R>, _>` (new module under `src/types/manifold/differential/`, one op per module), with grade and length validation returning `TopologyError`
- [x] 1.3 Write Leibniz and graded-anticommutativity property tests for `wedge` (sampled smooth fields, refinement sequence, convergence-order assertion); fix the primal–dual averaging convention against these tests (design open question 1)
- [x] 1.4 Error/branch coverage for `wedge`: `k + l > D` rejection; length mismatch on the first argument; length mismatch on the second argument; `k = 0` and `l = 0` scalar-multiplication branches; missing-metric branch if the implementation requires the metric; new `TopologyError` variant(s) covered incl. `Display`/`Debug`
- [x] 1.5 Implement `interior_product` as `(−1)^{k(D−k)} ⋆(⋆ω ∧ X♭)` composing the new wedge with the existing `hodge_star`
- [x] 1.6 Error/branch coverage for `interior_product`: non-1-form `X♭` rejection; `k = 0` input rejection (nothing to contract); length mismatches on both arguments; error propagation from the inner `wedge` and `hodge_star` calls (each inner failure surfaced and asserted separately); even-vs-odd `k(D−k)` sign branches each exercised
- [x] 1.7 Write the Cartan magic-formula test against tangent-functor analytic Lie derivatives of the Taylor–Green field, the `i_X i_X = 0` test, and the convective cross-validation (`i_u du♭` vs. pointwise `∇(|u|²/2) − (u·∇)u`; final assertion wired after 2.3)
- [x] 1.8 Parameterize all new test suites over f32 / f64 / Float106; register test files in mod chains and `tests/BUILD.bazel`; `cargo build -p deep_causality_topology && cargo test -p deep_causality_topology` clean; clippy clean at root cause (no `#[allow]`); verify 100% coverage on every new file

## 2. G2 — De Rham map and sharp map (deep_causality_topology)

- [x] 2.1 Implement the de Rham map (vertex vector field → edge 1-form; tangential component × edge length, plus the exact-line-integral entry point) with orientation consistent with `exterior_derivative`, and length validation
- [x] 2.2 Implement the sharp map (edge 1-form → vertex vector field; metric-weighted incident-edge averaging honoring per-axis periodicity)
- [x] 2.3 Encode the pair with the Tier-2 iso witness; property tests: exact-on-gradients (FTC/orientation pin), constant-field exact round-trip, second-order round-trip convergence on Taylor–Green, naturality via `iso::test_support`; f32/f64/Float106
- [x] 2.4 Error/branch coverage for the transfer pair: de Rham input length ≠ `D × num_vertices` rejection; sharp input length ≠ `num_cells(1)` rejection; **both** the midpoint-sampling and exact-line-integral de Rham paths exercised; sharp's periodic-wrap branch AND open-boundary edge-trimming branch each covered (mixed `[periodic, open]` axes fixture); missing-metric branch if metric-weighted averaging requires it
- [x] 2.5 Register tests, Bazel, clippy, 100% coverage on new files; finish task 1.7's convective cross-validation now that transfer exists

## 3. G6 + leray_project — Projection APIs (deep_causality_topology)

- [x] 3.1 Implement `leray_project` (grade-0 half-decomposition: `δω` → gauge-fixed `Δ₀` CG solve → `ω − dφ`), returning the projected 1-form with the grade-0 potential retrievable; `CgFailure` propagates as a typed error
- [x] 3.2 Tests for `leray_project`: gradient annihilation, divergence ≤ CG tolerance at three precisions, idempotency, harmonic mean-flow retention on `square_torus`/`cubic_torus`, success on lattices where the β-step would be singular
- [x] 3.3 Error/branch coverage for `leray_project`: CG non-convergence surfaced as the typed error (forced via a one-iteration budget); input grade ≠ 1 rejection; field-length mismatch rejection; missing-metric rejection; default-tolerance branch AND caller-supplied-options branch both exercised (incl. the per-backend epsilon clamping floor at f32)
- [x] 3.4 ~~Implement harmonic-kernel deflation~~ **Revised during apply (see design D6):** tests falsified the singularity premise (consistent RHS ⟂ kernel); pinned well-posedness on tori with tests instead (incl. 16×16 drift canary); deflation documented as fallback; module docs supersede Risk 1; shared `resolve_cg_tolerance` extracted for `leray_project`
- [x] 3.5 Tests for the deflated decomposition: convergence on 2D/3D tori, pairwise component orthogonality, open-lattice regression suite bit-compatibility, half-vs-full gradient-part agreement; update the superseded Risk 1 note reference in the module docs
- [x] 3.6 Branch coverage (revised with 3.4): torus path (β_k > 0) and contractible path each asserted; mixed-periodicity `[periodic, open]` lattice covered; CG-failure propagation covered by the existing `hodge_decompose_reports_nonconvergence_under_artificially_low_iteration_cap` test plus `leray_surfaces_cg_nonconvergence`
- [x] 3.7 Register tests, Bazel, clippy, 100% coverage on new and edited files (the edited `hodge_decompose` internals included); `make format && make fix`

## 4. G3 — Typed fluid forms (deep_causality_physics)

- [x] 4.1 Verify/establish the `deep_causality_topology` dependency edge in `deep_causality_physics` `Cargo.toml` and `BUILD.bazel` (document if new)
- [x] 4.2 Implement `VelocityOneForm<R>`, `VorticityTwoForm<R>`, `PressureZeroForm<R>`, `BodyForceOneForm<R>` under `src/fluids/` (one type per folder module; private fields; grade/length/finiteness-validating constructors; getters; `Debug`/`Display`/`PartialEq` in per-trait files per project convention)
- [x] 4.3 Constructor rejection coverage, per type and per branch: grade mismatch; length mismatch; NaN coefficient; +∞ and −∞ coefficients (separate cases); each emitted `PhysicsError` variant asserted, incl. `Display`/`Debug` output; happy-path getters covered
- [x] 4.4 Implement `Clone + Add + Mul<R>` on `VelocityOneForm<R>` only; test it satisfies the `Rk4` arrow bounds with a linear-decay rate closure reproducing analytic decay at three precisions
- [x] 4.5 Implement the `SolenoidalField<R>` type-state: private fields, `pub(crate)` construction restricted to the Leray-projection path and `from_hodge_projection`, read-only `as_one_form()`, deliberately no `Add`/`Mul`
- [x] 4.6 Type-state coverage: divergence-at-construction ≤ tolerance at three precisions via **both** construction paths (Leray per-step AND `from_hodge_projection` per-snapshot); error propagation when the underlying projection fails; `as_one_form()` read-only access; compile-fail tests for external construction and for `Add`/`Mul` on the projected type; trait-impl files (`Debug`/`Display`/`PartialEq`) covered
- [x] 4.7 Register tests in mod chains and Bazel, clippy clean, 100% coverage on every new file

## 5. Integration verification and closeout

- [x] 5.1 End-to-end MMS cross-check (note §6): DEC RHS (`−i_u du♭ + ν Δu♭`) vs. pointwise-kernel RHS via tangent-functor derivatives on the same sampled Taylor–Green field, agreement at second order, wired into CI alongside the existing `cfd_taylor_green` harness
- [x] 5.2 Coverage audit across the change: run the project's coverage tooling over every new or edited file in both crates and confirm 100%, with any genuinely unreachable line annotated and justified inline (AGENTS.md exemption rule); no `#[allow(dead_code)]` workarounds
- [x] 5.3 Run `make format && make fix && make build && make test` (3+ crates touched); fix failures at root cause, never by weakening a test against a correct API or suppressing a lint
- [x] 5.4 Update `openspec/notes/cfd/cfd-gap.md` gap statuses (G1–G4, G6 → closed) and the roadmap Stage 0 row; prepare the commit message and hand off to the user for commit (agents never commit)
