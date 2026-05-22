## Why

`add-cubical-complexes` (issue #487) shipped the type scaffold for cubical geometry — `LatticeComplex<D>` implementing `ChainComplex`, and `CubicalReggeGeometry<D>` carrying edge-length storage at four uniformity levels plus optional `timelike_axes` — but no derived geometric machinery. There is no way today to compute cell volumes, hinge dihedral angles, deficit angles, or the discrete Einstein–Hilbert (Regge) action on a `LatticeComplex<D>`, even though the scaffold carries all the inputs needed for it. This blocks every downstream use case that depends on metric-aware quantities on lattice complexes: lattice quantum gravity, anisotropic-spacetime studies, structure-preserving discrete PDE methods on Cartesian grids, and the cubical Hodge ⋆ (which needs cell-volume ratios as inputs).

This change set delivers the geometric core (phases R1–R3 of the design note) so that follow-up work — Hodge ⋆ generic over `ChainComplex`, Lorentzian variant, Metropolis dynamics — has a foundation to build on.

## What Changes

- Add cell-volume computation on `CubicalReggeGeometry<D, R>` for k-cells of every grade, across all four edge-length uniformity levels (`UnitEdge`, `Uniform`, `PerAxis`, `PerEdge`). Returns `R` (the generic real-field precision), not `f64`.
- Add hinge enumeration: a `hinge_top_cube_neighbors` helper on `LatticeComplex<D, R>` that walks the existing coboundary cache to enumerate top D-cubes incident to a given (D−2)-cell.
- Add dihedral-angle computation on `CubicalReggeGeometry<D, R>`: the angle a top cube contributes to a given hinge (returns `R`), in closed form for the per-axis case and `R::pi() / (R::one() + R::one())` on unit/uniform grids.
- Add deficit-angle computation: `ε(h) = 2π − Σ θᵢ(h)` (computed as `(R::pi() + R::pi()) − Σ θᵢ(h)`) summed over incident top cubes.
- Add the discrete Einstein–Hilbert (Regge) action: `S_R = Σ_h volume(h) · deficit_angle(h)` over all hinges (returns `R`).
- Convert `LatticeComplex<D, R>::coboundary_cache` from `Mutex<HashMap<usize, CsrMatrix<i8>>>` to `Box<[OnceLock<CsrMatrix<i8>>]>` of length `D + 1`, indexed by grade. Lock-free reads after first init; eliminates the `.clone()` in the cache-hit path; the `coboundary_matrix` method now returns `Cow::Borrowed`. This is a precursor step landed in §1 of tasks, before R1.
- Property-test invariants: unit-grid identities (every k-cube has k-volume `R::one()`; every dihedral angle equals `R::pi() / (R::one() + R::one())`; every deficit angle is `R::zero()`; total Regge action is `R::zero()`), per-axis closed forms, periodic-vs-open boundary action differences (measured on a perturbed metric), and locality of edge-length perturbations.
- No breaking changes to public trait surfaces. All additions are new methods on `CubicalReggeGeometry<D, R>` or new helpers on `LatticeComplex<D, R>`. The `ChainComplex`, `Neighborhood`, and `Manifold<K, F>` trait surfaces shipped in `add-cubical-complexes` Stages A–C are untouched. The `coboundary_cache` field is private; the swap to `OnceLock` is internal.

## Capabilities

### New Capabilities

- `cubical-regge-calculus-core`: Geometric derivation layer for cubical Regge calculus on `LatticeComplex<D>` — cell volumes, hinge enumeration, dihedral angles, deficit angles, and the discrete Einstein–Hilbert action — built on top of the `CubicalReggeGeometry<D>` scaffold shipped by `add-cubical-complexes`.

### Modified Capabilities

<!-- None. This change is purely additive against the type scaffold shipped by add-cubical-complexes. No existing requirement is changed. -->

## Impact

- **Crate affected:** `deep_causality_topology` only.
- **New source modules:**
  - `src/types/cubical_regge_geometry/volumes.rs`
  - `src/types/cubical_regge_geometry/curvature.rs`
  - Helpers added under `src/types/lattice_complex/`.
- **New methods on `CubicalReggeGeometry<D, R>`:**
  - `cell_volume(&self, complex, cell_id, grade) -> R`
  - `top_cell_volume(&self, complex, cell_id) -> R`
  - `dihedral_angle(&self, complex, top_cube_id, hinge_id) -> R`
  - `deficit_angle(&self, complex, hinge_id) -> R`
  - `regge_action(&self, complex) -> R`
- **New helper on `LatticeComplex<D, R>`:**
  - `hinge_top_cube_neighbors(&self, hinge_id) -> impl Iterator<Item = CellId>`
- **Internal cache change on `LatticeComplex<D, R>`:**
  - `coboundary_cache: Box<[OnceLock<CsrMatrix<i8>>]>` (was `Mutex<HashMap<usize, CsrMatrix<i8>>>`). Coboundary reads become lock-free and zero-copy (`Cow::Borrowed`).
- **Trait surface:** unchanged. No new public traits in this change set. (The `HasHodgeStar` capability trait belongs to the follow-up `add-cubical-regge-calculus-analytical` change set covering R4–R6.)
- **Dependencies:** no new external crates. All work uses existing infrastructure (`deep_causality_sparse` for the coboundary cache, `deep_causality_tensor` for any field views, the existing `CellId` / `Manifold` types).
- **Tests:** ~28 property tests across the three phases, registered under `tests/types/cubical_regge_geometry/` mirroring the source-tree layout, with corresponding entries in `deep_causality_topology/tests/BUILD.bazel`.
- **Effort:** ~570 LOC of source, ~28 tests, ~7 hours of focused work per the design note.
- **Out of scope for this change set (covered by a separate later proposal `add-cubical-regge-calculus-analytical`):** the cubical Hodge ⋆ on `LatticeComplex<D>` and the resulting promotion of `manifold/differential/{hodge,laplacian}.rs` to be generic over `ChainComplex` (R4); the Lorentzian variant and per-cell metric signature (R5); the action gradient and Metropolis updates (R6). Sparse cubical complexes, GPU backends, non-cubical regular tilings, and persistent-homology extensions are also out of scope.
- **Reference:** [openspec/notes/CubicalReggeCalculus.md](CubicalReggeCalculus.md), §§3.R1–R3.
