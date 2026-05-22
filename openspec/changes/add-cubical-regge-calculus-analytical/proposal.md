## Why

`add-cubical-regge-calculus-core` (R1–R3) delivers the geometric derivation layer for cubical Regge calculus on `LatticeComplex<D>`: cell volumes, hinge enumeration, dihedral angles, deficit angles, and the Euclidean discrete Einstein–Hilbert action. With that in place, the remaining gap is the **analytical core** — the operators and dynamics that turn the geometry into actual computation on field data.

Three concrete needs justify this change set:

1. **`manifold/differential/{hodge,laplacian}.rs` is still simplicial-only.** Phase R4 of the design note delivers the cubical Hodge ⋆ as the missing input to make these operators generic over `K: ChainComplex`. Once that lands, the Stage-C `cubical_heat_diffusion` example (currently a hand-coded Moore-neighborhood stencil) becomes a one-line `manifold.laplacian(0)` call, and every voxel-grid PDE downstream (heat, wave, Maxwell, FDTD-style methods) becomes a one-liner on `Manifold<LatticeComplex<D>, F>`.
2. **The Lorentzian / Euclidean signature is currently untyped.** `CubicalReggeGeometry<D>` carries an `Option<[bool; D]>` `timelike_axes` field that `add-cubical-regge-calculus-core` silently ignores. Phase R5 promotes this into a type-level marker `CubicalReggeGeometry<D, S>` with `S ∈ {Euclidean, Lorentzian}` so per-cell metric signatures, light-cone-violation detection, and the Wick-rotated `regge_action_lorentzian` are tracked by the compiler rather than at run-time.
3. **There is no way to do lattice quantum gravity on this scaffold.** Phase R6 closes the loop: the local edge-length gradient of the Regge action (O(2^D) per edge) plus a Metropolis update routine make Markov-chain Monte Carlo over edge-length configurations a one-line call against `deep_causality_rand`.

The downstream `add-hodge-decomposition` change set (§7 of the design note) depends on R4 specifically — the per-edge cubical Hodge ⋆ derivation is the one piece with genuine mathematical novelty in this roadmap. Shipping R4–R6 together is justified because R5 reuses R4's per-cell metric machinery, and R6 needs both the Euclidean action gradient (consumes R3) and the Lorentzian-aware variant (consumes R5).

## What Changes

- **R4 — Cubical Hodge ⋆ + generic differential operators.** Add `hodge_star_matrix(&self, complex, k) -> CsrMatrix<f64>` on `CubicalReggeGeometry<D>` returning the diagonal Hodge ⋆ on a lattice complex (entries are dual/primal volume ratios). Introduce a new public capability trait `HasHodgeStar` whose implementors are `ReggeGeometry<T>` (simplicial, already shipped) and `CubicalReggeGeometry<D>` (this change). Promote `manifold/differential/{hodge,laplacian}.rs` from `impl<F> Manifold<SimplicialComplex<C>, F>` to `impl<K: ChainComplex, F> Manifold<K, F> where K::Metric: HasHodgeStar`. `Manifold<CellComplex<_>, _>` (whose `Metric = ()`) silently loses access to these operators — correct behavior, since cell complexes have no metric.
- **R5 — Lorentzian variant + per-cell metric signature.** Add a type-level signature marker so `CubicalReggeGeometry<D, S = Euclidean>` becomes parameterized by `S ∈ {Euclidean, Lorentzian}` (the `Euclidean` default preserves source compatibility with R1–R3). Add `metric_tensor_at(&self, complex, cell_id, grade) -> CausalTensor<f64>` returning the local metric tensor with the correct signature `(+, +, …, +)` or `(−, +, …, +)`. Add `regge_action_lorentzian(&self, complex) -> Complex<f64>` for the Wick-rotated action. Add `LightConeViolation` error variant and detect light-cone-violating edge-length assignments in the `Lorentzian` constructor.
- **R6 — Action gradient + Metropolis updates.** Add `regge_gradient(&self, complex) -> Vec<f64>` returning `∂S_R / ∂(edge_length_i)` indexed by edge id (O(2^D) per edge, fully local). Add `metropolis_update<R: Rng>(&mut self, complex, rng, beta) -> AcceptReject` performing one Metropolis-Hastings step over `EdgeLengths { lengths: Vec<f64> }`, using `deep_causality_rand` for randomness. Add an `AcceptReject` enum (or `Result`-shaped equivalent) for the per-step outcome.
- **Generic differential code path is breaking-shaped at the impl level but source-compatible at the call site.** The trait-bound widening on `manifold/differential/{hodge,laplacian}.rs` (from `SimplicialComplex<C>` to `K: ChainComplex where K::Metric: HasHodgeStar`) is technically a signature change, but all existing call sites resolve identically because `ReggeGeometry<T>: HasHodgeStar` ships in this change. The signature widening is **non-breaking** for downstream code that doesn't impl `Manifold` on a custom complex.
- **`CubicalReggeGeometry<D>` becomes `CubicalReggeGeometry<D, Euclidean>` by default.** The defaulted const generic preserves source compatibility for the R1–R3 surface. Existing constructors (`unit_edge`, `uniform`, `per_axis`, `per_edge`) return the `Euclidean` variant; the `with_timelike_axes` builder returns the `Lorentzian` variant. Code that does `CubicalReggeGeometry::<3>::unit_edge()` continues to compile and produces the same Euclidean geometry as before.

## Capabilities

### New Capabilities

- `cubical-regge-calculus-analytical`: Analytical layer on top of the cubical Regge geometric core — cubical Hodge ⋆ on `LatticeComplex<D>` and the resulting promotion of `manifold/differential/{hodge,laplacian}.rs` to be generic over `ChainComplex`, the Lorentzian variant with per-cell metric signatures and light-cone-violation detection, and the local Regge-action gradient plus Metropolis updates that enable lattice-quantum-gravity-style Monte Carlo simulations.

### Modified Capabilities

- `cubical-regge-calculus-core`: `regge_action`'s contract is extended. The R1–R3 spec said the method ignores `timelike_axes` and always computes the Euclidean action; after this change, the `Euclidean` signature variant computes the Euclidean action (unchanged from R1–R3) and the new `Lorentzian` variant computes the Wick-rotated action returning `Complex<f64>`. The "ignored field" scenario is replaced by a "signature is tracked at the type level" scenario. All R1–R3 call sites compile and behave identically because the `Euclidean` default preserves source compatibility.

## Impact

- **Crate affected:** `deep_causality_topology` only.
- **New public trait:** `HasHodgeStar` (capability trait gating the Hodge-dependent differential operators).
- **New public type-level marker:** `Euclidean` and `Lorentzian` signature types (or const-generic equivalents) parameterizing `CubicalReggeGeometry<D, S>`.
- **New public methods on `CubicalReggeGeometry<D, S>`:** `hodge_star_matrix`, `metric_tensor_at`, `regge_action_lorentzian` (Lorentzian only), `regge_gradient`, `metropolis_update`.
- **New public error variant:** `LightConeViolation`.
- **New public enum:** `AcceptReject` (or equivalent shape).
- **Generic-widening of existing methods:** `Manifold::hodge_star`, `Manifold::laplacian`, `Manifold::codifferential` widen their `where` clauses from `Self = Manifold<SimplicialComplex<C>, F>` to `K: ChainComplex, K::Metric: HasHodgeStar`. Call-site behavior is preserved for simplicial users; lattice users gain access to these methods for the first time.
- **Trait impls added:** `impl<T> HasHodgeStar for ReggeGeometry<T>` and `impl<const D: usize, S> HasHodgeStar for CubicalReggeGeometry<D, S>`.
- **Dependencies:** uses `deep_causality_rand` (already in workspace) for Metropolis randomness; uses `num::Complex<f64>` style only if not already exported — otherwise a small internal `Complex<f64>` shim suffices (decision in design.md).
- **Effort per the design note:** R4 ~350 LOC / ~15 tests / 5 h, R5 ~250 LOC / ~12 tests / 4 h, R6 ~400 LOC / ~10 tests / 6 h. Total ~1000 LOC, ~37 tests, ~15 hours of focused work.
- **Sequencing:** depends on `add-cubical-regge-calculus-core` having shipped (cell volumes, hinge enumeration, dihedral angles, deficit angle, Euclidean Regge action are inputs to R4 and R6). Unblocks `add-hodge-decomposition` (§7 of the design note), which builds the uniform discrete Hodge–Helmholtz decomposition on top of R4's `HasHodgeStar` trait.
- **Out of scope:** the Hodge–Helmholtz decomposition itself (separate change set `add-hodge-decomposition`); causal-flow analysis (`add-causal-flow-analysis`); sparse cubical complexes; GPU paths; non-cubical regular tilings; categorical-coherence property tests via `proptest`.
- **Reference:** [openspec/notes/CubicalReggeCalculus.md](../add-cubical-regge-calculus-core/CubicalReggeCalculus.md), §§3.R4–R6.
