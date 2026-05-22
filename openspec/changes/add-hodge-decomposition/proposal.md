## Why

The `add-cubical-regge-calculus-analytical` change set delivers the `HasHodgeStar<R>` capability trait and the generic differential operators (`hodge_star`, `codifferential`, `laplacian`) on `Manifold<K, R>`. With those primitives in place, the Hodge–Helmholtz decomposition — the canonical splitting of any k-form field into orthogonal exact, co-exact, and harmonic components — becomes implementable in a few hundred lines and unlocks downstream work that is otherwise blocked.

Specifically, Block B1 of [`openspec/changes/add-cubical-regge-calculus-analytical/notes/3DCausalFluidDynamics.md`](../add-cubical-regge-calculus-analytical/notes/3DCausalFluidDynamics.md) consumes a `HodgeDecomposition<R>` carrier type and a `Manifold::hodge_decompose(...)` method as its entry point. The 3D causal-fluid pipeline cannot start until both exist. Beyond fluids, the same decomposition underpins vector-field denoising, topological vortex identification, and any downstream causal-inference work that needs a lossless replacement for autoencoder-based feature extraction.

This change set provides the decomposition itself, scoped narrowly: the carrier type, the manifold method, two-backend property tests (simplicial and cubical), and a PyDEC parity benchmark on the canonical unit square.

## What Changes

- Add a new public type `HodgeDecomposition<R: RealField>` under `deep_causality_topology/src/types/hodge_decomposition/` holding the three orthogonal components `exact: CausalTensor<R>`, `co_exact: CausalTensor<R>`, `harmonic: CausalTensor<R>` plus the grade `k`.
- Add the method `Manifold::hodge_decompose(&self, field: &CausalTensor<R>, k: usize) -> Result<HodgeDecomposition<R>, ManifoldError>` generic over `K: ChainComplex where K::Metric: HasHodgeStar<R>, R: RealField + FromPrimitive`. Implements the discrete Hodge–Helmholtz decomposition by solving the discrete Poisson problem `(δd + dδ) ω_harmonic = 0` and recovering the exact and co-exact parts via the standard projection.
- Add `ManifoldError::HodgeDecompositionFailed { reason: HodgeFailReason }` for the documented failure modes of the iterative solve (non-convergence, grade out of range, dimension mismatch, missing metric).
- Add `getters` per field on `HodgeDecomposition<R>` (one-trait-per-file under `src/types/hodge_decomposition/`).
- Verify with two-backend property tests: same field on a unit square decomposed via `ReggeGeometry<R>` (simplicial, two triangles) and via `CubicalReggeGeometry<2, R, Euclidean>` (one 2-cube) must agree on the decomposition to numerical tolerance.
- Verify the orthogonality identity `‖exact‖² + ‖co-exact‖² + ‖harmonic‖² = ‖field‖²` as a property test across at least three lattice sizes and both backends.
- Record a PyDEC parity benchmark on the unit square — the canonical reference implementation in the DEC literature — to ~5 significant figures. The benchmark inputs and expected outputs ship as static test fixtures; the agent reproduces them by hand from the PyDEC source against a known field, no Python dependency is introduced.
- **Hard precision rule:** every new public signature is generic over `R: RealField` (with `+ FromPrimitive` only where literal construction is required). No `f64` appears anywhere in the new surface. Mirrors the convention of `CubicalReggeGeometry<const D, R>`, `ReggeGeometry<R>`, and `Manifold<K, R>`.
- **Static dispatch only.** No `dyn`, no trait objects, per AGENTS.md.
- **No new external dependencies.** The iterative solve uses the existing CSR-matrix machinery in `deep_causality_sparse`.

## Capabilities

### New Capabilities

- `hodge-decomposition`: Discrete Hodge–Helmholtz decomposition on any `Manifold<K, R>` whose `K::Metric: HasHodgeStar<R>`. Covers the `HodgeDecomposition<R>` carrier type, the `Manifold::hodge_decompose` method, the documented failure modes, and the cross-backend equivalence guarantee.

### Modified Capabilities

(none — this is a pure additive change set; it consumes `cubical-regge-calculus-analytical`'s `HasHodgeStar<R>` trait unchanged.)

## Impact

- **Crate affected:** `deep_causality_topology` only. No new workspace crates. No changes to `deep_causality_core`, `deep_causality_num`, `deep_causality_sparse`, or `deep_causality_tensor`.
- **New public type:** `HodgeDecomposition<R: RealField>` and the documented getters.
- **New public method:** `Manifold::hodge_decompose`.
- **New public error variant:** `ManifoldError::HodgeDecompositionFailed { reason: HodgeFailReason }`.
- **Dependencies:** `add-cubical-regge-calculus-analytical` (R4 + R5 + R6) must ship first, refined per its `tasks.md` Block 0 to use `R: RealField`. The `HasHodgeStar<R>` capability trait is the load-bearing input. Without it, `hodge_decompose` has nothing to call.
- **Effort:** ~600 LOC, ~25 tests, ~12 hours of focused work after the prerequisite ships.
- **Unblocks:** Block B1 of `add-3d-causal-fluid-dynamics` (`TopologicalSignature<R>` extractor consumes `HodgeDecomposition<R>`). All downstream fluid-pipeline blocks depend transitively.
- **Out of scope:**
  - Topological signatures (downstream change set, B1).
  - Vector-field denoising applications (downstream consumers).
  - Sparse solvers beyond the CSR-backed iterative solve used internally.
  - GPU paths.
  - Adaptive iterative-solver tolerance tuning beyond a single configurable convergence threshold.
- **Agent conduct:** per AGENTS.md golden rule, agents never `git commit`. The `tasks.md` artifact lists explicit user-review checkpoints (G3 gates per block); the user is the only entity that commits.
