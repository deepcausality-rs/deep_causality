## Context

The `add-cubical-regge-calculus-analytical` change set delivers `HasHodgeStar<R>` and the generic differential operators `hodge_star`, `codifferential`, `laplacian` on `Manifold<K, R> where K: ChainComplex, K::Metric: HasHodgeStar<R>`. With those primitives in place, the Hodge–Helmholtz decomposition is implementable in roughly 600 LOC against the existing CSR-matrix machinery from `deep_causality_sparse`.

The decomposition is the classical splitting of any discrete k-form ω into three pairwise-orthogonal components:

```
ω = d α  +  δ β  +  h
    └─┬┘    └┬┘    └┬┘
   exact  co-exact harmonic
```

where `d` is the exterior derivative, `δ` is the codifferential, and `h` lies in the kernel of the Laplacian `Δ_k = δd + dδ`. On a finite simplicial or cubical complex, all three components are uniquely determined by ω up to numerical tolerance.

Stakeholders: the Block B1 work in [`3DCausalFluidDynamics.md`](../add-cubical-regge-calculus-analytical/notes/3DCausalFluidDynamics.md) (consumes `HodgeDecomposition<R>` to build `TopologicalSignature<R>`); future vector-field denoising work; future topological-vortex-identification work; anyone needing a lossless replacement for autoencoder-based feature extraction on field data.

The R1–R3 + R4 + R5 + R6 surface stays untouched. This change set is purely additive on top of the analytical layer.

## Goals / Non-Goals

**Goals:**

- Deliver `HodgeDecomposition<R: RealField>` as a public carrier type holding the three orthogonal components plus the grade `k`, with one-trait-per-file getters following the project's structure convention.
- Deliver `Manifold::hodge_decompose(&self, field: &CausalTensor<R>, k: usize) -> Result<HodgeDecomposition<R>, ManifoldError>` generic over `K: ChainComplex where K::Metric: HasHodgeStar<R>, R: RealField + FromPrimitive`.
- Implement the decomposition via the discrete Poisson solve `Δ_k φ = δ ω` (recovers α via `α = d φ` projection step), then the analogous step for β, and finally `h = ω − d α − δ β` as the residual.
- Use the existing CSR-matrix machinery in `deep_causality_sparse` for assembly and the existing iterative-solver primitives in `deep_causality_topology` (per the simplicial Laplacian-inverse code path that already exists for the `manifold/differential` operators).
- Verify with two-backend property tests: a prescribed test field on a unit square, viewed once as a simplicial complex (two triangles) and once as a cubical complex (one 2-cube), must produce orthogonally-equivalent decompositions to numerical tolerance.
- Verify the Hodge orthogonality identity `‖exact‖² + ‖co-exact‖² + ‖harmonic‖² = ‖field‖²` across at least three lattice sizes and both backends.
- (Deferred) PyDEC parity benchmark — moved to a dedicated follow-up change set `add-hodge-decomposition-pydec-parity`. Internal correctness in this change set is established via the two-backend cross-check + the Hodge orthogonality identity + analytic pure-exact / pure-co-exact tests, with no Python touchpoint.

**Non-Goals:**

- PyDEC parity benchmark on the unit square — moved to follow-up change set `add-hodge-decomposition-pydec-parity` per the goal-list note above.
- Topological signatures (`TopologicalSignature<R>`) — downstream change set `add-3d-causal-fluid-dynamics` Block B1.
- Vector-field denoising or any downstream application of the decomposition.
- Sparse solvers beyond the CSR-backed iterative solve used internally.
- GPU paths.
- Categorical-coherence property tests (functoriality of the decomposition under refinement).
- Higher-grade decompositions on signature-mixed Lorentzian metrics — the Lorentzian `HasHodgeStar<R>` impl from R5 is consumed unchanged, but no Lorentzian-specific validation is part of this change set (the Lorentzian path inherits whatever R5 ships).
- Adaptive iterative-solver tolerance tuning beyond a single configurable convergence threshold.

## Decisions

### Decision 1: One change set, three sequential blocks H1 → H2 → H3

H1 lands the carrier type and error variants (data-only, no algorithm). H2 lands the decomposition algorithm against the discrete Poisson solve. H3 lands the property tests and PyDEC parity benchmark. Each block is independently testable; H1 ships as a passing test surface even before H2's algorithm exists (constructor + getters + error variants all have unit-test coverage).

Gating per block follows the discipline of `notes/3DCausalFluidDynamics.md`:

- G1 (compile clean + `cargo clippy --all-targets -- -D warnings` clean — fix lints at root cause, never `#[allow(clippy::...)]` per `feedback_clippy_lints`).
- G2 (100% coverage on every new and modified file).
- G3 (user review + commit; agents never commit per AGENTS.md golden rule).

No block opens with an unclosed prior-block gate.

**Alternatives considered:**

- Ship as a single block. Rejected: H2's iterative solve is the only algorithmically interesting work; isolating it behind H1's data scaffolding makes review tractable.
- Combine H3 with H2 (algorithm + tests in one block). Rejected: separating the property tests into their own gated block enforces that the H2 implementation is reviewable on the algorithmic merits before the test suite is built around it, which catches "the test was written to fit the bug" failure modes.

### Decision 2: Decomposition algorithm — three sequential Poisson solves

For a k-form ω on a `Manifold<K, R>` with Hodge ⋆ available:

1. Solve `Δ_k φ_α = δ ω` for the (k-1)-form potential `φ_α`. Set `α = d φ_α`; this is the exact component.
2. Solve `Δ_k ψ_β = d ω` for the (k+1)-form potential `ψ_β`. Set `β = δ ψ_β`; this is the co-exact component.
3. `h = ω − α − β` is the harmonic residual.

The two Poisson solves are independent and could in principle run in parallel; this change set ships them sequentially for simplicity. Parallelisation is a future perf change.

The solver is the same iterative-solver primitive that the existing simplicial Laplacian code path uses, exposed as a private helper in `src/types/manifold/differential/`. If no such helper exists today, this change set lands a minimal CG (conjugate-gradient) solver on `CsrMatrix<R>` since the discrete Laplacian is symmetric positive semi-definite. CG is generic over `R: RealField + FromPrimitive` and requires no external dependency.

**Convergence:** the solver runs to a relative residual `‖r‖ / ‖b‖ < ε_R` where `ε_R = R::from_f64(1e-10).unwrap_or_else(R::default_epsilon)`. The threshold is exposed as a function parameter with a default; callers needing tighter or looser tolerances can override.

**Failure modes (encoded in `HodgeFailReason`):**

- `Nonconvergence { iterations: usize, residual: R }` — CG did not converge in the iteration budget.
- `GradeOutOfRange { k: usize, max_dim: usize }` — `k > max_dim` or `k > 3` for grades the manifold does not carry.
- `DimensionMismatch { expected: usize, actual: usize }` — input field has the wrong number of entries for grade `k`.
- `MissingMetric` — `self.metric` is `None`. This is structurally impossible if `K::Metric: HasHodgeStar<R>` is satisfied at the type level, but the check stays as a defensive guard since `Manifold::metric` is `Option<...>` by current design.

**Alternatives considered:**

- Direct sparse LU solve. Rejected: pulls in either `sprs`/`nalgebra-sparse` (external dep, against AGENTS.md) or ~1500 LOC of in-house sparse-LU implementation. CG is ~80 LOC and adequate for the lattice sizes the downstream fluid pipeline targets (≤ 256³).
- Reuse an existing iterative solver if one ships in the workspace. Audit at H2 kickoff: if `deep_causality_sparse` or `deep_causality_topology` already has a CG implementation, reuse it; otherwise land the minimal version here.
- Compute the harmonic component first via the kernel of `Δ_k` (eigen-decomposition or null-space projection). Rejected: more expensive than the residual approach for lattices where the harmonic dimension is small (the typical case for trivial-topology grids).

### Decision 3: `HodgeDecomposition<R>` is a value type, not a borrow type

The struct owns its three `CausalTensor<R>` components. Callers that need to keep the input field separate can clone or borrow it externally. This matches the existing `CausalTensor<R>` ownership conventions in `deep_causality_topology`.

Fields are private per the AGENTS.md visibility rule. Getters are one-per-file under `src/types/hodge_decomposition/`:

- `src/types/hodge_decomposition/mod.rs` — struct definition + constructor.
- `src/types/hodge_decomposition/getters.rs` — `exact()`, `co_exact()`, `harmonic()`, `grade()`.
- `src/types/hodge_decomposition/display.rs` — `Debug` and `Display`.
- `src/types/hodge_decomposition/part_eq.rs` — `PartialEq` (tensor-equality with tolerance, per existing convention).

**Alternatives considered:**

- Return a tuple `(α, β, h)`. Rejected: loses type identity and the grade context. Once Block B1 lands, the `TopologicalSignature::from(decomposition)` constructor signature is far cleaner with a named carrier type.
- Make the carrier generic over the manifold reference (`HodgeDecomposition<'a, K, R>`). Rejected: forces a lifetime through every downstream consumer; the gain (avoiding three tensor clones) is not worth the complexity for the lattice sizes targeted.

### Decision 4: PyDEC parity benchmark — deferred to follow-up change set

PyDEC parity is moved out of this change set into a dedicated follow-up `add-hodge-decomposition-pydec-parity`. The benchmark requires a one-time off-line Python run to hand-derive reference values from the [PyDEC](https://github.com/hirani/pydec) source, which is orthogonal to the algorithm + property-test work in H1–H3 and would gate this change set on a Python-touching task. Splitting it out keeps H1–H3 Python-touchless and lets the algorithm land on internal consistency gates alone.

Internal correctness in this change set is established via three independent checks:

1. The Hodge orthogonality identity `‖α‖² + ‖β‖² + ‖h‖² = ‖ω‖²` across precision backends and lattice sizes (the algebraic invariant that any correct decomposition must satisfy).
2. Analytic pure-exact and pure-co-exact field tests where the expected component norms are known in closed form.
3. The two-backend cross-check on the unit square (simplicial vs cubical) at tolerance `1e-6` in `f64`.

The follow-up change set will add the external PyDEC check on top once H1–H3 land.

**Alternatives considered:**

- Keep PyDEC parity in-scope and ship the fixtures with H3. Rejected by user direction: the Python dependency is orthogonal to the algorithmic work and is cleaner to land separately.
- Generate fixtures via a Python script at test time. Rejected: introduces a Python runtime dependency, violates AGENTS.md "avoid external crates / runtimes" rule.
- Drop PyDEC parity entirely and never run the external check. Rejected: the simplicial and cubical paths could in principle agree with each other while both disagreeing with the canonical DEC reference. Hence the deferral to a follow-up rather than outright removal.

### Decision 5: Two-backend cross-check on the unit square

A prescribed 1-form field on the unit square is decomposed twice: once with the simplicial backend (`ReggeGeometry<R>` over a complex of two triangles) and once with the cubical backend (`CubicalReggeGeometry<2, R, Euclidean>` over a single 2-cube). The two decompositions must agree on the L2 norms of each component to tolerance `ε_R = R::from_f64(1e-6).unwrap()`.

This is the property-test analog of "discretisation independence" for the Hodge decomposition. If the two backends disagree, either the simplicial Hodge ⋆ (R4.2 of the prerequisite) or the cubical Hodge ⋆ (R4.3 / R4.4) is wrong, and the decomposition itself cannot be the locus of the bug. This separation of concerns is the load-bearing reason for the two-backend test design.

**Current coverage (post-H3).** The H3 cross-backend tests verify:

1. Orthogonality identity `‖α‖² + ‖β‖² + ‖h‖² = ‖ω‖²` on both backends at `1e-6`.
2. Vanishing-component agreement: for `ω = df` (pure exact), both backends report `(‖β‖² + ‖h‖²) / ‖ω‖² < 1e-6`, and the cross-backend disagreement on that ratio is itself `< 1e-6`.

Full per-component L2 norm equality (the original scenario in `spec.md`) requires a manifold-respecting two-triangle simplicial unit square. `PointCloud::triangulate` is Vietoris-Rips — it builds every clique of every grade — so on the four coplanar corners of the unit square at radius 1.5 it produces 4 overlapping triangles (every 3-clique), not the 2 non-overlapping triangles of a Delaunay triangulation. `Manifold::with_metric` rejects the resulting complex because four interior-sharing triangles do not form a 2-manifold. The single-right-triangle simplicial fixture used in `hodge_decomposition_cross_backend_tests.rs` sidesteps this; tightening to the full unit-square fixture awaits a Delaunay (or other manifold-respecting) triangulation algorithm in `PointCloud`. See Risk 5 below.

### Decision 6: Convergence tolerance is `R`-derived, not hard-coded

The CG convergence threshold is `R::from_f64(1e-10).unwrap_or_else(R::default_epsilon)`. For `f32` this falls back to `f32::EPSILON ≈ 1.19e-7`; for `f64` to `f64::EPSILON ≈ 2.22e-16`; for `DoubleFloat` to whatever that type defines. Tests assert convergence at each precision backend's natural tolerance, not at a single shared one.

**Alternatives considered:**

- Hard-code `1e-10`. Rejected: meaningless for `f32` (below its representable epsilon) and overly loose for `DoubleFloat`.
- Take the tolerance as a constructor parameter only. Adopted as an additional override path: the `hodge_decompose` method accepts an optional `HodgeDecomposeOptions { tolerance: Option<R>, max_iterations: Option<usize> }` parameter; defaults are derived as above.

## Risks / Trade-offs

- **[Risk] CG may not converge for ill-conditioned Laplacians.** The discrete Laplacian `Δ_k` is symmetric positive semi-definite but has a null space corresponding to the harmonic forms. CG on a singular system needs a regularisation step (project the RHS onto the range of `Δ_k`) or it diverges silently.
  → **Mitigation:** before each Poisson solve, project the RHS onto the range of `Δ_k` by subtracting its projection onto the kernel. The kernel is enumerated via the existing Betti-number machinery (`ChainComplex::betti_number`) and a small Krylov basis. Documented in the H2 task list. If the kernel is empty (trivial topology), the projection is a no-op.

- **[Risk] The minimal in-house CG solver may need to land in `deep_causality_sparse` rather than `deep_causality_topology`.** The crate boundary question is open. CG is a sparse-linear-algebra primitive, not a topology primitive.
  → **Mitigation:** decide at H2 kickoff. If `deep_causality_sparse` is the natural home, the CG solver lands there as a small additive change to that crate, exposed as a public function. If it lands in `deep_causality_topology`, it stays private. Either path is fine; the decision is documented at H2-G3 review.

- **[Risk] The Laplacian inverse on grade-0 forms is degenerate by exactly one dimension (constant functions are always harmonic).** This is the canonical "Neumann problem" non-uniqueness.
  → **Mitigation:** at grade 0, fix the gauge by subtracting the mean from φ_α before computing `α = d φ_α`. This is standard practice in DEC literature and adds three lines. Documented in H2.

- **[Risk 5] `PointCloud::triangulate` is Vietoris-Rips, not Delaunay.** It builds every k+1-vertex clique whose pairwise edges all fit within the radius, regardless of whether the resulting complex is a manifold. For four coplanar corners of the unit square at radius ≥ √2 it produces 6 edges and 4 overlapping triangles (every 3-clique), which is not a 2-manifold and is rejected by `Manifold::with_metric`. The H3 ambient-dimension cap on `triangulate` removes one related bug (degenerate higher simplices collapsing lumped-mass M_0) but does not fix the Vietoris-Rips → non-manifold gap.
  → **Mitigation:** the H3 cross-backend test fixture uses a single right triangle on the simplicial side rather than the full unit square. The orthogonality identity and vanishing-component agreement scenarios still hold at the spec.md `1e-6` tolerance; only the strict per-component L2 norm equality scenario from `spec.md` 4.4 remains future work. The clean fix is a Delaunay (or constrained-Delaunay) triangulation algorithm in `PointCloud`, scoped as a separate change set.
  → **Update (2026-05-26, post-archive):** the silent-zero half of this risk is closed by `harden-simplicial-hodge-degeneracy-detection`. The `triangulate` lumped-mass loop no longer silently zeroes degenerate top simplices; the rejection moved to a fallible `SimplicialComplex::hodge_star_operators` accessor with a discriminating `TopologyError::PointCloudError`. The Vietoris-Rips → non-manifold gap itself (the canonical four-coplanar-corners case) remains, and is the motivating problem for the `add-pointcloud-delaunay-triangulation` change set.

- **[Trade-off] One configurable convergence tolerance, not a full preconditioner / step-size tuning surface.** Adequate for the lattice sizes the downstream fluid pipeline targets (≤ 256³). Larger or stiffer problems may need a preconditioned CG; that is a future perf change, not a blocker for H1–H3.

- **[Trade-off] In-house CG instead of an external sparse-linalg crate.** ~80 LOC, no external dep, generic over `R: RealField`. The cost is performance: an external SIMD-accelerated CG would be faster, but the downstream consumers do not need that performance today.

- **[Trade-off] Sequential rather than parallel Poisson solves in H2.** The two solves are independent and could run in parallel via `rayon`. Deferred to a perf change set; the algorithm is correct as-written.

## Migration Plan

This change set is purely additive. There is no migration.

- **Source compatibility:** the new `Manifold::hodge_decompose` method is added to existing `impl<K, R> Manifold<K, R>` blocks. No existing methods change signature.
- **Rollback:** revert the change set. No persisted state, no schema changes, no downstream code depends on it yet (Block B1 is the first consumer and ships separately).
- **Sequencing:** depends on `add-cubical-regge-calculus-analytical` (R4 + R5 + R6) having shipped, refined per its `tasks.md` Block 0 to use `R: RealField`. Unblocks Block B1 of `add-3d-causal-fluid-dynamics`.

## Open Questions

1. **Does a CG solver already exist in `deep_causality_sparse` or elsewhere in the workspace?** Audit at H2 kickoff. If yes, reuse; if no, land the minimal ~80 LOC version per Decision 2.
2. **Does `deep_causality_num::RealField` expose `default_epsilon()` or an equivalent?** Used in Decision 6 to derive the CG tolerance. If not, the fallback is `R::from_f64(1e-7)` or similar. Verify at H1 kickoff.
3. **Should the change set live entirely in `deep_causality_topology` or split the CG solver into `deep_causality_sparse`?** See Risk 2; decision at H2 kickoff.
4. **PyDEC version pinning.** Deferred to the follow-up change set `add-hodge-decomposition-pydec-parity` per Decision 4. Not an open question for the present change set.
