# Tasks — add-cubical-regge-calculus-analytical

This change set delivers R4 (cubical Hodge ⋆ + generic differential operators), R5 (Lorentzian signature marker + per-cell metric), and R6 (Regge-action gradient + Metropolis updates), all parameterised over `R: RealField` to align with the existing `CubicalReggeGeometry<const D, R>` and `ReggeGeometry<R>` surface.

## Block 0 — Spec refinement (closes Open Question 7)

Before any code lands, the proposal and design must be reconciled with the `R: RealField` parameterisation that R1–R3 already shipped. This is a documentation-only block.

- [x] 0.1 Rewrite `proposal.md` so every `f64` in a public signature becomes `R: RealField` (with `+ FromPrimitive` where literal construction is required). Affected surfaces:
  - `hodge_star_matrix(...) -> CsrMatrix<R>` (was `CsrMatrix<f64>`).
  - `metric_tensor_at(...) -> CausalTensor<R>` (was `CausalTensor<f64>`).
  - `regge_action_lorentzian(...) -> Complex<R>` (was `Complex<f64>`).
  - `regge_gradient(...) -> Vec<R>` (was `Vec<f64>`).
  - `metropolis_update<Rng>(&mut self, ..., beta: R) -> AcceptReject<R>` (was untyped `beta`).
- [x] 0.2 Promote `CubicalReggeGeometry<D, S = Euclidean>` to `CubicalReggeGeometry<const D, R, S = Euclidean>` throughout the proposal so it matches the live type signature.
- [x] 0.3 Update `design.md` Decision 2 to lock the trait shape as:
  ```rust
  pub trait HasHodgeStar<R: RealField> {
      fn hodge_star_matrix(&self, complex: &impl ChainComplex, k: usize) -> CsrMatrix<R>;
  }
  ```
  with `impl<R: RealField + FromPrimitive> HasHodgeStar<R> for ReggeGeometry<R>` and `impl<const D: usize, R: RealField + FromPrimitive, S: SignatureMarker> HasHodgeStar<R> for CubicalReggeGeometry<D, R, S>`.
- [x] 0.4 Close Open Question 7 in `design.md` with the decision above.
- [x] 0.5 Update Decision 5 (generic differential operators) to widen `Manifold` impls to `impl<K, R> Manifold<K, R> where K: ChainComplex, K::Metric: HasHodgeStar<R>, R: RealField + FromPrimitive`.
- [x] 0.6 Resolve Open Question 1 (does `deep_causality_num` expose `Complex`?): **Resolved — `deep_causality_num::Complex<T: RealField>` is already exposed at the crate root ([`src/lib.rs:59`](../../../deep_causality_num/src/lib.rs#L59)) with the required generic shape, including `Complex32` / `Complex64` aliases. Reuse unchanged; no coordinated change to `deep_causality_num` is needed.**
- [x] 0.7 Update the proposal's Impact section so `Complex<R>` (or the equivalent reused type) appears with its actual provenance.
- [ ] 0.8 Block-0 gate: user reviews the refined proposal + design, signs off, commits. No code starts until G0 closes.

## Block R4 — Cubical Hodge ⋆ + generic differential operators

Depends on Block 0. Lands the new capability trait, the cubical implementation, and the trait-bound widening of the existing simplicial differential operators so they become generic over `K: ChainComplex`.

### R4.1 Trait scaffolding

- [x] R4.1.1 Create `deep_causality_topology/src/traits/has_hodge_star.rs` with `pub trait HasHodgeStar<R: RealField>` per Decision 2 of the refined design. **Done** with the refined associated-type-on-`Complex` shape and `Cow<'_, CsrMatrix<R>>` return per the [chain_complex.rs](../../../deep_causality_topology/src/traits/chain_complex.rs) precedent — design issue surfaced during implementation, design.md Decision 2 updated to match.
- [x] R4.1.2 Register the trait in `src/traits/mod.rs` and re-export from `src/lib.rs`.
- [x] R4.1.3 Write a one-trait-per-file test stub under `tests/traits/has_hodge_star_tests.rs`; register in `tests/traits/mod.rs` and `tests/BUILD.bazel`. **Bazel registration not needed** — the existing `traits` suite uses `glob(["traits/*_tests.rs"])`, the new file is picked up automatically. 3 tests passing.

### R4.2 Simplicial `HasHodgeStar` impl

- [ ] R4.2.1 Implement `HasHodgeStar<R> for ReggeGeometry<R>` where `R: RealField + FromPrimitive`. The body wraps the existing simplicial Hodge ⋆ computation that currently lives inline in [`manifold/differential/hodge.rs`](../../../deep_causality_topology/src/types/manifold/differential/hodge.rs).
- [ ] R4.2.2 Move the simplicial Hodge ⋆ body out of `Manifold::hodge_star` and into the trait impl, so the manifold-level method becomes a thin call to `self.metric.hodge_star_matrix(...)`.
- [ ] R4.2.3 Property tests: identity-grid Hodge ⋆ is the identity matrix; ⋆ ⋆ = (-1)^(k(n-k)) on Euclidean signature; orthogonality of resulting Laplacian (i.e. `⟨Δω, η⟩ = ⟨ω, Δη⟩` to tolerance) on a small simplicial test complex.

### R4.3 Cubical `HasHodgeStar` impl — unit-edge and per-axis tiers

- [ ] R4.3.1 Implement `hodge_star_matrix` for the `UnitEdge` and `PerAxis` tiers of `CubicalReggeGeometry<D, R, S>`. Diagonal entries are `volume(dual (D-k)-cell) / volume(primal k-cell)`, both available from the existing volume machinery in [`cubical_regge_geometry/volumes.rs`](../../../deep_causality_topology/src/types/cubical_regge_geometry/volumes.rs).
- [ ] R4.3.2 Property tests for the two tiers, with the closed-form expectations given in design.md Decision 4 (unit grid ⇒ identity; 2D per-axis with axes `[a, b]` ⇒ ⋆_0 entry `a·b`, ⋆_1 entries `a/b` or `b/a`, ⋆_2 entry `1/(a·b)`).

### R4.4 Cubical `HasHodgeStar` impl — `PerEdge` tier

- [ ] R4.4.1 Allocate explicit derivation time per Risk 1 of design.md. Derive the dual-cell edge-length formula for `PerEdge` cubical geometry as half-edge averages of the top cubes' edge lengths in the dual directions.
- [ ] R4.4.2 Cross-check the per-edge derivation against the simplicial Hodge ⋆ on a unit square viewed both as two triangles and as one 2-cube; both decompositions must agree on the Hodge decomposition of a prescribed test field to within numerical tolerance.
- [ ] R4.4.3 Implement `hodge_star_matrix` for the `PerEdge` tier.
- [ ] R4.4.4 Property tests: agreement with `PerAxis` on degenerate per-edge inputs (every edge in axis `i` has the same length); independence from cell ordering.
- [ ] R4.4.5 **Risk-mitigation gate:** if R4.4.1–R4.4.2 slip beyond one week, ship R4 with `UnitEdge` + `PerAxis` only and mark `PerEdge` as an explicit follow-up via a `ManifoldError::HodgeStarPerEdgeUnimplemented` return, per design.md Risk 1 mitigation. The decision to defer is taken at R4.4 review, not silently.

### R4.5 Generic differential-operator widening

- [ ] R4.5.1 Widen `impl<R> Manifold<SimplicialComplex<R>, R>` blocks in `src/types/manifold/differential/{hodge,laplacian,codifferential}.rs` to `impl<K, R> Manifold<K, R> where K: ChainComplex, K::Metric: HasHodgeStar<R>, R: RealField + FromPrimitive`.
- [ ] R4.5.2 Audit the workspace for downstream `impl Manifold<...>` blocks. If any exist, the widening is potentially source-breaking; document in `tasks.md` and surface at R4 review.
- [ ] R4.5.3 Add tests on `Manifold<LatticeComplex<3>, R>` exercising `hodge_star`, `codifferential`, and `laplacian` for `k ∈ {0, 1, 2, 3}` on a unit grid and on a per-axis grid. The Stage-C `cubical_heat_diffusion` example becomes a one-line `manifold.laplacian(0)` call per the proposal — extend the existing example or add a new integration test that runs it both ways and asserts agreement.
- [ ] R4.5.4 Discrete Hodge decomposition theorem on a small cubical lattice: any 1-form on a 4³ grid with trivial topology decomposes uniquely into exact + co-exact (no harmonic component on this trivial-topology lattice). Quantitatively verified.

### R4.6 Block R4 gates

- [ ] R4.6.1 R4-G1 Compilation: `cargo build -p deep_causality_topology` clean (release + debug); `cargo clippy -p deep_causality_topology --all-targets -- -D warnings` clean.
- [ ] R4.6.2 R4-G2 Coverage: 100% on every new file (`src/traits/has_hodge_star.rs`, cubical Hodge ⋆ impl files) and every modified file (`src/types/manifold/differential/{hodge,laplacian,codifferential}.rs`, `src/types/regge_geometry/*`, `src/types/cubical_regge_geometry/*`). Unreachable code annotated and justified.
- [ ] R4.6.3 R4-G3 Review: user reviews the diff, runs `make format && make fix`, signs off, commits.

## Block R5 — Lorentzian signature marker + per-cell metric

Depends on R4 (specifically R4.3's per-cell volume machinery, which R5 reuses for the metric tensor). Adds `Euclidean` / `Lorentzian` marker types, promotes the geometry to `CubicalReggeGeometry<const D, R, S = Euclidean>`, and detects light-cone violations at construction.

### R5.1 Signature marker scaffolding

- [ ] R5.1.1 Create `src/types/cubical_regge_geometry/signature/mod.rs` (or equivalent) with `pub struct Euclidean;`, `pub struct Lorentzian;`, and the sealed `pub trait SignatureMarker` per design.md Decision 3.
- [ ] R5.1.2 Seal the trait via the standard `mod sealed { pub trait Sealed {} }` pattern; implement `Sealed` for `Euclidean` and `Lorentzian` only.
- [ ] R5.1.3 Re-export the marker types and trait from `src/lib.rs`.

### R5.2 Promote `CubicalReggeGeometry` to three parameters

- [ ] R5.2.1 Change the struct definition in [`cubical_regge_geometry/mod.rs`](../../../deep_causality_topology/src/types/cubical_regge_geometry/mod.rs) from `CubicalReggeGeometry<const D: usize, R: RealField>` to `CubicalReggeGeometry<const D: usize, R: RealField, S: SignatureMarker = Euclidean>`.
- [ ] R5.2.2 Verify R1–R3 call sites (`CubicalReggeGeometry::<3, f64>::unit_edge()` etc.) continue to compile via the `S = Euclidean` default. Workspace-wide grep + `cargo check` is the test.
- [ ] R5.2.3 Audit project lint config for `#![deny(elided_lifetimes_in_paths)]` or similar (Risk 3 of design.md). Resolve at the type definition site if needed.
- [ ] R5.2.4 Existing `with_timelike_axes` builder is repurposed as the entry to the `Lorentzian` constructor and now returns `Result<CubicalReggeGeometry<D, R, Lorentzian>, LightConeViolation>`.

### R5.3 Per-cell metric tensor

- [ ] R5.3.1 Add `metric_tensor_at(&self, complex: &LatticeComplex<D>, cell_id: CellId, grade: usize) -> CausalTensor<R>` on both `S = Euclidean` and `S = Lorentzian` impl blocks.
- [ ] R5.3.2 The Euclidean tensor is diagonal with `+` entries; the Lorentzian tensor is diagonal with `-` for each timelike axis and `+` otherwise per the East-Coast convention documented in the design.
- [ ] R5.3.3 Property tests: signature matches `S` (eigenvalues check); rotational invariance of trace on Euclidean; Sylvester's-criterion shortcut for the Lorentzian signature check (closed-form for D ≤ 4 per Risk 4 of design.md).

### R5.4 Lorentzian Hodge ⋆ sign factors

- [ ] R5.4.1 Extend the cubical `HasHodgeStar<R>` impl to apply the `(-1)^t` sign factor on the Lorentzian variant, where `t` is the number of timelike axes in the primal cell's active dimensions.
- [ ] R5.4.2 Property test: Euclidean and Lorentzian variants on the *all-spacelike* configuration produce identical Hodge ⋆ matrices (the signature factor degenerates to 1).

### R5.5 Light-cone-violation detection

- [ ] R5.5.1 Add `LightConeViolation { cell_id: CellId, eigenvalues: Vec<R> }` to the crate's error type.
- [ ] R5.5.2 Implement the Sylvester's-criterion signature check (closed-form, O(D) per cube for D ≤ 4) in the `Lorentzian` constructor. Return `Err(LightConeViolation)` on any cube whose local metric has the wrong signature.
- [ ] R5.5.3 Property tests: well-formed Minkowski grid constructs successfully; a deliberately-broken edge-length assignment (timelike edge longer than the spacelike sum it should bound) is rejected.

### R5.6 `regge_action_lorentzian`

- [ ] R5.6.1 Confirm or land `Complex<R>` per task 0.6. If `deep_causality_num::complex_number` is not yet generic over `R`, the work to generalise it is itself a separate micro-change against `deep_causality_num` and is a dependency of R5.6.
- [ ] R5.6.2 Implement `regge_action_lorentzian(&self, complex) -> Complex<R>` on the `Lorentzian` impl block per design.md (Wick-rotated action).
- [ ] R5.6.3 Property test: on the all-spacelike degenerate configuration, the real part equals the Euclidean `regge_action` and the imaginary part is zero to tolerance.

### R5.7 Block R5 gates

- [ ] R5.7.1 R5-G1 Compilation: `deep_causality_topology` (and, if touched, `deep_causality_num`) clean.
- [ ] R5.7.2 R5-G2 Coverage: 100% on every new and modified file.
- [ ] R5.7.3 R5-G3 Review.

## Block R6 — Regge-action gradient + Metropolis updates

Depends on R5 (the gradient and Metropolis update must respect the signature; Lorentzian rejection uses R5.5's light-cone check).

### R6.1 Action gradient — Euclidean

- [ ] R6.1.1 Derive `∂(dihedral)/∂(length)` in closed form per design.md Decision 6 (`d/dx arctan(y/x) = -y/(x²+y²)` style identities).
- [ ] R6.1.2 Implement `regge_gradient(&self, complex: &LatticeComplex<D>) -> Vec<R>` on the `Euclidean` impl block. Length-`num_edges()`, indexed by the existing private `edge_index` helper introduced in R1.
- [ ] R6.1.3 Locality check: each entry depends only on the O(2^D) hinges of D-cubes containing the edge.

### R6.2 Action gradient — Lorentzian

- [ ] R6.2.1 Extend `regge_gradient` to the `Lorentzian` impl block returning `Vec<Complex<R>>` (or document why a real-valued gradient is sufficient and project accordingly).
- [ ] R6.2.2 Property test: on the all-spacelike configuration, the Lorentzian gradient's real part equals the Euclidean gradient to tolerance.

### R6.3 Finite-difference verification

- [ ] R6.3.1 Property test per design.md Decision 6: `(S(L + ε·δ_i) − S(L − ε·δ_i)) / (2ε)` ≈ analytical gradient to ~5 sig figs for ε ~ 1e-5. Run on 2D, 3D, and 4D lattices at unit-edge and per-axis tiers.
- [ ] R6.3.2 Verify the equilibrium check: the unit-edge configuration is a stationary point (gradient near zero in every component, to tolerance).

### R6.4 `AcceptReject` and `metropolis_update`

- [ ] R6.4.1 Add `pub enum AcceptReject<R: RealField> { Accepted { delta_action: R, proposed_length: R }, Rejected { reason: RejectReason } }` (per design.md Decision 7; richer than the bare two-variant version proposed in proposal.md).
- [ ] R6.4.2 Add `RejectReason { NonPositiveLength, LightConeViolation, ProbabilisticReject }`.
- [ ] R6.4.3 Implement `metropolis_update<Rng>(&mut self, complex, rng, sigma: R, beta: R) -> AcceptReject<R>` on the `Euclidean` variant using `deep_causality_rand`'s `Normal` distribution. Reject on `length' ≤ 0` per the detailed-balance preservation argument in design.md Risk 5.
- [ ] R6.4.4 Implement the `Lorentzian` variant; in addition to non-positive rejection, reject on `LightConeViolation` per R5.5.
- [ ] R6.4.5 ΔS computation is *local* (the same O(2^D) locality used in R6.1.2), not a recomputation of the full action.

### R6.5 Detailed-balance verification

- [ ] R6.5.1 Property test per design.md Risk 5: ~10⁶ Metropolis steps on a 2-edge toy lattice. Check that the equilibrium distribution matches `exp(-β · S_R)` to χ² tolerance. Use `deep_causality_rand`'s deterministic seed mode so the test is reproducible.

### R6.6 Block R6 gates

- [ ] R6.6.1 R6-G1 Compilation: clean.
- [ ] R6.6.2 R6-G2 Coverage: 100% on every new file. The detailed-balance test counts as a long-running test and is feature-gated behind `--features long-running-tests` if its runtime exceeds the project's test-runtime budget.
- [ ] R6.6.3 R6-G3 Review. After R6-G3, this change set is complete and unblocks `add-hodge-decomposition`.

## Out-of-scope reminder

The following are explicitly NOT part of this change set (per design.md "Non-Goals"):

- The Hodge–Helmholtz decomposition (`hodge_decompose`) — separate change set `add-hodge-decomposition`.
- The 3D causal-fluid pipeline (TopologicalSignature, RollingHistory, FluidContext, SURD wiring, NS kernels) — separate change set sequence per [`notes/3DCausalFluidDynamics.md`](notes/3DCausalFluidDynamics.md).
- Causal-graph analysis of turbulent flows.
- Sparse cubical complexes.
- GPU paths.
- Non-cubical regular tilings.
- Performance tuning beyond the algorithmically-natural O(2^D) per-edge gradient.
- Adaptive Metropolis step-size tuning.
- HMC sampling.
- Categorical-coherence proptest suite.

## Total effort

- Block 0 (spec refinement): ~3 hours, 0 LOC, documentation only.
- Block R4 (Hodge ⋆ + generic operators): ~400 LOC, ~18 tests, ~6 hours.
- Block R5 (signature + per-cell metric + Lorentzian variant): ~300 LOC, ~14 tests, ~5 hours.
- Block R6 (gradient + Metropolis): ~450 LOC, ~12 tests, ~7 hours.

**Total: ~1150 LOC, ~44 tests, ~21 hours focused work** (~6 hours above the proposal.md ~15h estimate; the delta is the R: RealField refinement work and the explicit per-edge derivation budget in R4.4).
