## Context

`add-cubical-complexes` (Stages A–C) shipped a complete type scaffold for cubical geometry in `deep_causality_topology`:

- `LatticeComplex<D>` implements `ChainComplex`, stores `shape: [usize; D]` and `periodic: [bool; D]`, and caches the coboundary operator behind a `Mutex` once it has been computed. Cells are enumerated lazily by `LatticeCellIterator` and encoded as `(position, orientation_bitmask)` pairs.
- `CubicalReggeGeometry<D>` is the associated `ChainComplex::Metric` type. Edge lengths are represented at four uniformity levels — `UnitEdge`, `Uniform { length }`, `PerAxis { lengths: [f64; D] }`, `PerEdge { lengths: Vec<f64> }` — and an optional `timelike_axes: Option<[bool; D]>` is reserved for the (out-of-scope) Lorentzian variant.
- `Manifold<LatticeComplex<D>, F>` wraps complex + field data + optional metric; constructors `from_cubical` / `from_cubical_with_metric` exist.

What is missing is the *geometric derivation layer*: every quantity in the proposal (cell volume, dihedral angle, deficit angle, Regge action) is currently uncomputable on a `LatticeComplex<D>` even though all the inputs are present in `CubicalReggeGeometry<D>`. The forward-looking design note [openspec/notes/CubicalReggeCalculus.md](../../notes/CubicalReggeCalculus.md) lays out the full six-phase roadmap (R1–R6); this change set delivers R1–R3 — the geometric core — and stops at the boundary where the analytical core (Hodge ⋆, Lorentzian, dynamics) would begin.

Stakeholders: anyone using `Manifold<LatticeComplex<D>, F>` for lattice-based scientific computing (sensor fusion on voxel grids, lattice gauge theory + GR, anisotropic spacetime studies, structure-preserving discrete PDE methods on Cartesian grids). The downstream `add-cubical-regge-calculus-analytical` change set (R4–R6) depends on cell volumes (R1) and hinge enumeration (R2) as inputs, so this change set is on the critical path for all of that work.

## Goals / Non-Goals

**Goals:**

- Compute the k-volume of every k-cell in a `LatticeComplex<D>`, across all four edge-length uniformity levels.
- Enumerate the top D-cubes incident to each (D−2)-cell (hinge) by walking the existing coboundary cache.
- Compute the dihedral angle that a given top cube contributes to a given hinge, in closed form for the per-axis case.
- Compute the deficit angle at every hinge and the total discrete Einstein–Hilbert (Regge) action of the complex.
- Verify algebraic invariants by property tests (unit-grid identities, per-axis closed forms, periodic-vs-open boundary differences, locality of edge-length perturbations).
- Keep all additions purely additive: no breaking changes to `ChainComplex`, `Neighborhood`, `Manifold<K, F>`, or the existing `CubicalReggeGeometry<D>` API; no new public traits in this change set.

**Non-Goals:**

- Cubical Hodge ⋆ on `LatticeComplex<D>` and the resulting promotion of `manifold/differential/{hodge,laplacian}.rs` to be generic over `ChainComplex` — deferred to `add-cubical-regge-calculus-analytical` (R4).
- Lorentzian variant of `CubicalReggeGeometry<D>` and per-cell metric-signature computation — deferred to R5. This change set computes the Euclidean Regge action only; the `timelike_axes` field is ignored.
- Action gradient and Metropolis Markov-chain updates over edge lengths — deferred to R6.
- A `HasHodgeStar` capability trait — belongs with R4.
- Sparse-cubical-complex (active-cubes-only) representation, GPU backends, non-cubical regular tilings (hex / kagome / triangular Regge analogs), persistent homology, sheaves, Khovanov-style invariants. Each is a separate change set.
- Performance tuning. Correctness and clean closed forms come first; optimization (parallelization, SIMD, GPU) is a separate perf-track change set.

## Decisions

### Decision 1: Module layout under `src/types/cubical_regge_geometry/`

The Stage-C scaffold lives at `src/types/cubical_regge_geometry/mod.rs` with the type definition, constructors, and the four edge-length variants. Per the AGENTS.md "one type, one module folder; one trait per file when implementing multiple traits" convention, the derived geometric machinery splits into two new sibling files:

- `src/types/cubical_regge_geometry/volumes.rs` — cell-volume methods (R1).
- `src/types/cubical_regge_geometry/curvature.rs` — dihedral angles, deficit angles, Regge action (R2 + R3).

Both files contain `impl<const D: usize> CubicalReggeGeometry<D> { … }` blocks, not free functions. The hinge-enumeration helper goes on `LatticeComplex<D>` itself (it walks the coboundary cache, which is private to that module) at `src/types/lattice_complex/hinge_neighbors.rs` (or extended into the existing coboundary file if it stays small).

**Alternatives considered:**
- Single file `cubical_regge.rs` with all R1–R3 methods. Rejected: the design note lists 570+ LOC and 28 tests across three phases; one file per phase keeps reviews scoped and matches the project convention.
- Free functions in a `regge` submodule rather than methods on `CubicalReggeGeometry<D>`. Rejected: the `&self` parameter is needed to read edge lengths and (in R3) the Lorentzian flag (even though we don't yet use it); method form keeps the API discoverable via `cargo doc`.

### Decision 2: Cell volume from edge lengths — closed form per uniformity level

For a k-cube whose orientation bitmask has active dimensions `{i₁, …, iₖ}`, the k-volume in the cubical setting (where edges meeting at any vertex are mutually orthogonal under the lattice's per-axis metric) reduces to the product of edge lengths along the active dims. Concretely:

- `UnitEdge`: `cell_volume = 1.0` for every grade. No traversal needed.
- `Uniform { length: L }`: `cell_volume = L.powi(grade as i32)`.
- `PerAxis { lengths }`: `cell_volume = active_dims.iter().map(|i| lengths[*i]).product()`.
- `PerEdge { lengths }`: walk the cube's k incident edges along the active dims (using `position` + axis to index into `lengths`) and take the product. The per-edge Gram-matrix determinant collapses to this product because cross-terms vanish under axis alignment — documented in the closed-form derivation in §3.R1 of the design note.

**Rationale:** the cubical case admits these clean closed forms precisely because cubical cells are axis-aligned. We do *not* introduce a general Gram-determinant routine here. If a future change set ever needs sheared cubical cells (where the Gram matrix has off-diagonal entries), it can replace this implementation without touching the public method signature.

**Alternatives considered:** a generic Gram-determinant path that handles arbitrary edge vectors. Rejected: it adds ~80 LOC of linear algebra, runs ~10× slower in the common case, and provides no additional capability under the cubical-axis-aligned assumption. Out of scope for this change set.

### Decision 3: Hinge enumeration via the existing coboundary cache

Hinges in a D-dim cubical complex are (D−2)-cells. The top D-cubes incident to a hinge are exactly the support of the coboundary-of-coboundary operator `δ ∘ δ` applied to that hinge. `LatticeComplex<D>` already builds and caches the `coboundary_matrix(k)` lazily per grade, so:

```
hinge_top_cube_neighbors(complex, hinge_id) =
    coboundary_matrix(D-1).row_indices_of_columns_in(
        coboundary_matrix(D-2).row_indices_of_column(hinge_id)
    )
```

returned as an iterator (no allocation in the steady state — the matrix rows are sparse, the iterator yields D-cube CellIds). The result is deduplicated; on a regular grid each (D−2)-hinge has exactly 2(D−1) = 4 incident D-cubes in 3D and 4 in 4D. (Note: 2(D−1) is the count of (D−1)-faces incident to a (D−2)-cell *inside one D-cube*. For the global incidence count of D-cubes per (D−2)-hinge, the value depends on the lattice — 2 in 2D, 4 in 3D, 4 in 4D — and the property tests pin it down.)

**Rationale:** reuses infrastructure already shipped in Stage B. Zero new caching, zero new sparse-matrix code. The compute is one sparse matrix-vector product against the cached coboundary matrices.

**Alternatives considered:** a closed-form coordinate-walk that enumerates incident D-cubes from `(position, orientation_bitmask)` without consulting the coboundary cache. Faster in the worst case (~4× constant-factor) but duplicates the cache's invariant and breaks if a future change set introduces an irregular cubical complex (e.g. sparse active-cubes-only representation). Rejected for now; can be added as a `#[inline]` fast path later.

### Decision 4: Dihedral angle closed form

A dihedral angle at a hinge `h` from a top cube `c` is the angle between the two faces of `c` that share `h`. The two faces correspond to the two axes of the dihedral's normal plane — the axes inactive in `h` but active in `c`. Call them `i` and `j`.

- `UnitEdge` and `Uniform`: every dihedral angle is exactly π/2. The implementation returns `std::f64::consts::FRAC_PI_2` without touching the lattice.
- `PerAxis { lengths }`: `dihedral_angle(c, h) = arctan2(lengths[j], lengths[i])` (the angle the face along axis `j` makes with the face along axis `i` at the shared hinge). For a unit cube `lengths[i] == lengths[j]` so the result is π/4 + π/4 = π/2 by symmetry — which is consistent with the unit case.
- `PerEdge`: same shape as per-axis, but `lengths[i]` is read at the specific edge of `c` along axis `i` adjacent to `h`. Closed form is identical otherwise.

**Rationale:** the cubical-axis alignment turns dihedral angles into 2-argument arctangents — no Cayley-Menger, no eigenvalue solve, no nonlinear root-finding. This is the single biggest practical advantage of the cubical Regge approach over simplicial.

**Note on convention:** the design note lists a property test ("per-axis grid with axes `[a, b]` in 2D: dihedral angle from a stretched cube is `arctan(b/a) + arctan(a/b) = π/2`") that confirms angles around a hinge sum to π/2 *per cube* and 2π over all four incident cubes on a flat grid. Implementation honors this: the angle a cube contributes is between its two faces at the hinge, and the sum is checked in R3.

### Decision 5: Deficit angle and Regge action on the Euclidean case only

Deficit angle: `deficit_angle(h) = 2π − Σ_{c incident to h} dihedral_angle(c, h)`. The factor of 2π is the full angle in the 2D normal plane to a (D−2)-hinge. Regge action: `regge_action() = Σ_h cell_volume(h, D-2) · deficit_angle(h)`.

**Scope choice:** this change set computes the Euclidean Regge action only. The `timelike_axes` field on `CubicalReggeGeometry<D>` is read-only for now — if it is `Some(...)`, `regge_action` returns the Euclidean action computed by treating all axes as spacelike (the field is *not* validated). The Lorentzian variant (`regge_action_lorentzian` returning `Complex<f64>`) is deferred to R5, where the type system will be extended to track the signature choice (the `S = Euclidean | Lorentzian` marker in §3.R5 of the design note).

**Rationale:** keeping R3 Euclidean-only lets us ship the geometric core without dragging in the type-level signature machinery R5 requires. The Lorentzian variant is non-trivial (imaginary dihedral angles in normal planes containing the timelike axis, Wick rotation conventions, light-cone-violation detection) and belongs in its own reviewable change.

**Alternatives considered:** validate `timelike_axes.is_none()` at the entry of `regge_action` and panic / return `Err` otherwise. Rejected: the field is `Option<[bool; D]>` and may be set by user code that doesn't intend a Lorentzian computation. We document the Euclidean assumption clearly in the doc comment and move on.

### Decision 6: No new public traits

The follow-up change set (R4 / Hodge ⋆ generic over `ChainComplex`) introduces a `HasHodgeStar` capability trait. This change set introduces *no* new public traits, in keeping with the proposal's "no breaking changes / no trait surface modifications" guarantee. All new functionality is delivered via inherent methods on `CubicalReggeGeometry<D>` and one helper on `LatticeComplex<D>`.

**Rationale:** trait surfaces are part of the public contract; once added, they're hard to evolve. By keeping this change set inherent-method-only, R4 can introduce `HasHodgeStar` later with full freedom to shape the trait around what the analytical core actually needs.

### Decision 7: Test layout mirrors source layout, registered in BUILD.bazel

Per AGENTS.md ("test folder replicates the exact src folder structure"), the test files live at:

- `tests/types/cubical_regge_geometry/volumes_tests.rs`
- `tests/types/cubical_regge_geometry/curvature_tests.rs`
- `tests/types/lattice_complex/hinge_neighbors_tests.rs`

Each is registered in `tests/types/cubical_regge_geometry/mod.rs` / `tests/types/lattice_complex/mod.rs` with `#[cfg(test)]`, and the folder modules are declared in `deep_causality_topology/tests/BUILD.bazel`. Any shared test fixtures (small lattices with known geometry) go under `src/utils_tests/` so Bazel can reach them.

## Risks / Trade-offs

- **[Risk] Coboundary-cache contention under multi-threaded use.** `LatticeComplex<D>` caches `coboundary_matrix(k)` behind a `Mutex`. Repeated hinge enumeration from many threads serializes on the cache mutex on first compute, then runs lock-free for reads.
  → **Mitigation:** acceptable in this change set; the cache is already in place and the first-compute serialization is a one-time cost per `(complex, grade)` pair. If contention is observed downstream, swap the `Mutex` for `OnceLock` in a separate perf change.

- **[Risk] Per-edge volume / dihedral formulas assume axis-aligned cells.** The closed forms rely on cubical cells having mutually orthogonal edges at every vertex. If a future change set introduces sheared cubical cells (where the per-edge Gram matrix has off-diagonal entries), the closed-form routines silently produce wrong answers.
  → **Mitigation:** document the assumption prominently in doc comments. Add a `debug_assert!` in the per-edge path that the local Gram matrix has zero off-diagonal entries. (This is essentially free under the current `PerEdge { lengths: Vec<f64> }` representation, which has no way to express off-diagonal terms in the first place — but the assertion guards the assumption explicitly.)

- **[Risk] Floating-point error accumulation in the Regge-action sum.** On a periodic unit-edge lattice, every deficit angle is mathematically zero; the implementation must avoid summing `2π − (π/2 + π/2 + π/2 + π/2)` term-by-term in a way that produces machine-epsilon noise scaling with the number of hinges.
  → **Mitigation:** detect the unit / uniform / per-axis-with-equal-lengths case at the entry of `regge_action` and short-circuit to `0.0` exactly. The property test "every deficit angle is 0 on a unit grid" then passes by construction.

- **[Risk] `timelike_axes` field is ignored.** Users who set `timelike_axes = Some(...)` and call `regge_action()` get the Euclidean action, not the Lorentzian one they may have intended.
  → **Mitigation:** the doc comment on `regge_action` states explicitly that the field is ignored and points to the follow-up `regge_action_lorentzian` method that will land in R5. Possibly add a `#[deprecated_in_next = "..."]`-style note if cargo supports it; otherwise documentation only.

- **[Trade-off] No generic Gram-determinant path.** We accept that this change set cannot handle sheared cubical cells. The benefit is ~80 LOC saved and a ~10× faster per-edge volume computation in the common case.

- **[Trade-off] Inherent methods instead of a trait.** Hodge ⋆ work in R4 will need to dispatch through a `HasHodgeStar` trait; that's fine because R4's analytical operators (`Manifold::laplacian`, `Manifold::hodge_star`) compose with the inherent methods landed here without requiring them to be trait-dispatched.

## Migration Plan

This change is purely additive. No migration steps for downstream code.

- After landing: existing `Manifold<LatticeComplex<D>, F>` users see new methods on the metric type and a new helper on the complex type. Existing calls continue to work unchanged.
- Rollback: revert the change. No persisted state, no public-API removals, no schema changes.
- Sequencing: this change set must land before `add-cubical-regge-calculus-analytical` (R4–R6), which depends on `cell_volume` (R1) and `hinge_top_cube_neighbors` (R2) as inputs to the cubical Hodge ⋆ construction.

## Open Questions

1. Should `regge_action` return `f64` (the action value) or `ReggeActionResult { value: f64, hinges_evaluated: usize, max_deficit: f64 }` for downstream introspection? **Recommendation:** start with `f64`. Add the richer return type in R6 when the gradient computation needs the same intermediate quantities.
2. Should `dihedral_angle` on `UnitEdge` / `Uniform` short-circuit to a `const FRAC_PI_2` without consulting the complex, or always walk the lattice for consistency? **Recommendation:** short-circuit — it's measurably faster in the only case where the difference matters (unit-grid sanity-check tests), and the doc comment will note the optimization.
3. Per-edge case: how is the edge-to-`lengths`-index map encoded? `LatticeComplex<D>` enumerates cells as `(position, orientation_bitmask)`; mapping a (position, axis) pair to a flat index in `PerEdge { lengths: Vec<f64> }` is its own piece of bookkeeping. **Recommendation:** the mapping should be a private associated function on `LatticeComplex<D>` (e.g. `edge_index(position, axis) -> usize`) shared by R1, R2, and any future per-edge consumer. Land it as part of R1.
