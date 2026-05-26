## Context

`PointCloud::triangulate` builds a Vietoris-Rips complex: every k+1-vertex clique whose pairwise edges all fit within the caller's radius becomes a k-simplex in the output. This is the correct construction for topological data analysis (persistent homology, clique complexes) but produces non-manifold complexes on dense planar point sets — e.g. the four coplanar corners of a unit square at radius ≥ √2 yield 6 edges and 4 overlapping triangles (every 3-clique), failing `Manifold::with_metric`'s manifold-property check.

This blocks the strict per-component L2 norm cross-backend equality scenario from `add-hodge-decomposition/specs/hodge-decomposition/spec.md` 4.4. The Hodge decomposition change set ships the relaxed orthogonality + vanishing-component-ratio variant of that scenario; the strict variant is documented in its `tasks.md` Section 6 as deferred to this change set.

The fix is a sibling triangulation method: `PointCloud::triangulate_delaunay` produces a manifold-respecting simplicial complex via 2D Delaunay triangulation. The existing `triangulate` (Vietoris-Rips) stays unchanged — it has 41 direct call sites across the workspace and remains the right tool for TDA use cases.

Stakeholders: the strict cross-backend test in `add-hodge-decomposition`; future DEC pipelines on planar simplicial complexes; downstream consumers of `Manifold::hodge_decompose` who need numerical agreement against a reference simplicial implementation.

## Goals / Non-Goals

**Goals:**

- Deliver `PointCloud::triangulate_delaunay(&self) -> Result<SimplicialComplex<T>, TopologyError>` for planar (2D ambient) point sets. The returned complex MUST satisfy `Manifold::with_metric`'s manifold-property check for any non-degenerate input.
- Compute lumped-mass Hodge ⋆ operators on the Delaunay output identically to the existing Vietoris-Rips path, so the resulting `SimplicialComplex<T>` is drop-in compatible with `Manifold::with_metric`, the differential operators, and `Manifold::hodge_decompose`.
- Document the robustness contract explicitly: which inputs succeed, which fail with a `TopologyError`, and which succeed with a documented tiebreak rule (cocircular points).
- Tighten the H3 cross-backend test in `add-hodge-decomposition` to the strict per-component L2 norm equality variant from `spec.md` 4.4.
- All new public surface is generic over `T: RealField + FromPrimitive`. Zero `f64` in any new public signature.

**Non-Goals:**

- 3D Delaunay tetrahedralisation. The 3D-ambient generalisation is well-studied but ~5x the implementation surface; defer until a downstream consumer needs it. The new method explicitly requires `D = 2`.
- Constrained-Delaunay with boundary edges (Chew, Ruppert). The unit square has no required interior edges that would not naturally be in the Delaunay output, so plain Delaunay suffices for `spec.md` 4.4.
- Adaptive-precision in-circumcircle predicate (Shewchuk's robust predicates, ExactPredicates equivalents). The first version uses naive `f64` predicates with documented tiebreak rules. If a downstream consumer hits precision issues on a specific input class, the adaptive-precision upgrade lands as a separate follow-up.
- Alpha-shapes or weighted Delaunay (power diagrams).
- GPU paths.
- Replacement of the existing `PointCloud::triangulate` (Vietoris-Rips). The two methods coexist; callers pick by use case.

## Decisions

### Decision 1: Bowyer-Watson over divide-and-conquer

Bowyer-Watson is the incremental Delaunay algorithm: start with a super-triangle containing all input points, insert each point one at a time, identify the triangles whose circumcircle contains the new point, remove them, and re-triangulate the resulting polygonal cavity.

**Why Bowyer-Watson:**

- ~250-350 LOC vs ~400-500 for Guibas–Stolfi divide-and-conquer.
- The bookkeeping is straightforward: a single `Vec<Simplex>` of current triangles, an in-circumcircle predicate, and a "polygonal cavity boundary" extraction step. No quad-edge data structure.
- Worst-case `O(n²)` vs `O(n log n)` for divide-and-conquer. For the lattice sizes the downstream Hodge decomposition pipeline targets (≤ 16³ ≈ 4 096 points), the quadratic constant is a few milliseconds.
- The cocircular-degeneracy handling is local to the in-circumcircle predicate; divide-and-conquer's merge step has the same issue distributed across more code.

**Alternatives considered:**

- **Guibas–Stolfi divide-and-conquer.** Rejected: 1.5x the LOC, the quad-edge structure is dense for the workspace's idiomatic style, and the asymptotic speedup is irrelevant for current consumer sizes. If a future workload needs sub-millisecond triangulation of 10⁵+ points, lift to D&C in a follow-up.
- **Fortune's sweep-line.** Rejected: produces the Voronoi diagram, requires a separate Voronoi→Delaunay dual step, and degeneracy handling is harder.
- **Watson's original variant** (no super-triangle, build the convex hull as you go). Rejected: more complex initialisation, doesn't simplify the in-circumcircle code.

### Decision 2: Super-triangle bounding box rule

Bowyer-Watson initialises with a "super-triangle" — a triangle large enough to contain all input points. After triangulation, simplices containing super-triangle vertices are discarded.

**Sizing rule:** compute the axis-aligned bounding box of input points, expand by a factor of 100x in each direction, and place the super-triangle vertices at `(min_x - 100·dx, min_y - 100·dy)`, `(max_x + 200·dx, min_y - 100·dy)`, `(min_x - 100·dx, max_y + 200·dy)` where `dx = max_x - min_x`, `dy = max_y - min_y`. The 100x expansion is large enough that no input point lies inside the circumcircle of any super-triangle edge with a real input vertex (avoiding spurious circumcircle hits), and small enough that floating-point precision is not catastrophically lost.

**Alternatives considered:**

- **Unit-bounded super-triangle** at, say, `(±10, ±10)`. Rejected: brittle for inputs at non-unit scale.
- **Symbolic perturbation** for the super-triangle (treat super-vertices as conceptually at infinity). Rejected: ~50 LOC of bookkeeping for a problem that the 100x bounding-box approach solves cleanly.

### Decision 3: Cocircular tiebreak rule

Four points are cocircular iff they lie on a common circle. For cocircular inputs the Delaunay triangulation is not unique; there are two valid triangulations differing only in which diagonal is chosen. The unit square's four corners are the canonical example (cocircular on the unit circle around `(0.5, 0.5)`).

**Tiebreak rule:** when the in-circumcircle predicate returns "on the circle" (within `T::epsilon() * 100`), accept the existing triangulation and do not flip. This produces a deterministic output dependent on input-point order.

**For the unit-square test fixture:** the four corners are inserted in lexicographic order `(0,0), (1,0), (1,1), (0,1)`. The Bowyer-Watson sequence produces a specific diagonal choice (either `(0,2)` or `(1,3)`); the test asserts properties invariant under that choice (orthogonality identity, per-component L2 norm to spec.md 4.4 tolerance), not the specific diagonal.

**Alternatives considered:**

- **Symbolic perturbation** (Edelsbrunner-Mücke). Rejected: ~100 LOC of careful bookkeeping for a problem the simple tiebreak solves at the cost of accepting one valid triangulation over the other.
- **Random tiebreak.** Rejected: violates determinism, which the test suite relies on.
- **Reject cocircular inputs** with a `TopologyError`. Rejected: the unit square is the canonical test case and must succeed.

### Decision 4: Degenerate-input handling

Three classes of degeneracy:

1. **Fewer than 3 input points.** Return `Err(TopologyError::PointCloudError("triangulate_delaunay requires at least 3 points"))`.
2. **All input points collinear.** Return `Err(TopologyError::PointCloudError("triangulate_delaunay requires non-collinear input"))`. Detected by checking whether the bounding-box area is `< T::epsilon() * max_extent²`.
3. **D > 2 ambient dimension.** Return `Err(TopologyError::PointCloudError("triangulate_delaunay requires 2D ambient (D == 2)"))`. 3D Delaunay is non-goal.

All three are checked at the top of the method; the Bowyer-Watson core runs only after these guards pass.

### Decision 5: Hodge ⋆ operator computation

**Updated (post-`harden-simplicial-hodge-degeneracy-detection`):** the helper extraction is already done. `harden-simplicial-hodge-degeneracy-detection` task 5.1 created `deep_causality_topology/src/types/simplicial_complex/lazy_hodge_star.rs` containing `pub(crate) fn build_lumped_mass_hodge_star<T>(skeletons: &[Skeleton], coords: &[T], dim: usize) -> Result<Vec<CsrMatrix<T>>, TopologyError>` with `T: RealField + FromPrimitive`. The helper is fallible (zero-volume top simplex returns the unified `"top-dimensional simplex below tolerance"` error). The `triangulate_delaunay` constructor calls `SimplicialComplex::with_geometry(...)`, which stores coordinates and triggers the helper lazily on first read of `complex.hodge_star_operators()`. No further helper extraction is needed in this change set.

Why extract:

- Avoids duplicating ~80 LOC of mass-matrix construction.
- Makes the Hodge ⋆ choice a single decision point reviewable independently of the triangulation choice.
- If a future change set needs a different Hodge ⋆ scheme (geometric instead of barycentric duals), the swap is local.

**Alternatives considered:**

- **Reimplement the mass-matrix code in `triangulate_delaunay`.** Rejected: duplicates ~80 LOC and creates a divergence risk where the two triangulation paths could produce different mass matrices for the same complex.
- **Move the Hodge ⋆ computation to `SimplicialComplexBuilder::build`.** Rejected: that constructor is used in places where the geometric coordinates are not available; the lumped-mass formula requires top-simplex volumes which require coordinates.

### Decision 6: Tightened cross-backend test fixture

The H3 cross-backend test (`hodge_decomposition_cross_backend_tests.rs`) currently uses a single right triangle on the simplicial side. After this change set, the simplicial fixture becomes the canonical two-triangle unit square via `PointCloud::triangulate_delaunay` on the four corners.

The cross-backend assertions tighten from:

- (current) Orthogonality identity at `1e-6` on each backend; vanishing-component ratio agreement at `1e-6` cross-backend.

To:

- (post-fix) Strict per-component L2 norm equality at `1e-6` cross-backend: `|‖simplicial.exact()‖ − ‖cubical.exact()‖| < 1e-6`, and likewise for `co_exact` and `harmonic`.

The H3 relaxed scenarios remain in the test file as additional invariants; the strict scenario is added as the new primary assertion.

## Risks / Trade-offs

- **[Risk] Naive `f64` predicates can produce inconsistent triangulations on inputs near the cocircular boundary.** The in-circumcircle determinant has terms of order `coord⁴`; on inputs scaled to `1e6` or `1e-6`, the predicate can flip sign due to floating-point rounding.
  → **Mitigation:** the public contract documents the tested input regime (coordinates `O(1)` in magnitude). Inputs outside that regime are accepted but with no robustness guarantee. If a downstream consumer needs scale-invariant robustness, the adaptive-precision predicate is a follow-up. Test coverage explicitly includes the unit-scale unit-square case (the spec.md 4.4 target) plus randomized fuzz tests in the `[-10, 10]²` range.

- **[Risk] Quadratic-worst-case insertion order for adversarial inputs.** Bowyer-Watson's `O(n²)` cost is realised when nearly all points fall inside the circumcircle of each insertion's predecessor. Random or lexicographic insertion produces `O(n log n)` average.
  → **Mitigation:** insertion order is the input point-array order. The method documents that callers wanting `O(n log n)` average performance should pre-randomise their input order if it is known to be adversarial. For lattice-aligned inputs (the spec.md 4.4 case), the order is fine.

- **[Risk] Super-triangle removal leaves "dangling" output triangles.** A triangle containing a super-triangle vertex must be discarded post-triangulation. Forgetting to also discard simplices that *only* reference real input vertices but were created by the super-triangle's presence is a classic Bowyer-Watson bug.
  → **Mitigation:** explicit invariant check after the super-triangle removal pass: every output triangle has all three vertex indices `< num_input_points`. Tested directly.

- **[Trade-off] Two triangulation paths in `PointCloud`** — Vietoris-Rips and Delaunay — increase the API surface. Callers must understand which to pick for which use case.
  → **Documentation:** the `triangulate_delaunay` doc comment explicitly contrasts the two and points each at its appropriate use case (DEC vs TDA).

## Migration Plan

This change set is purely additive on the public API:

- **Source compatibility:** `PointCloud::triangulate` is unchanged. Existing call sites continue to work.
- **Rollback:** revert the change set. No persisted state, no schema changes. The H3 cross-backend test reverts to the relaxed scenario; nothing else downstream breaks.
- **Sequencing:** depends on `add-hodge-decomposition` having shipped fully (H1–H3 + the triangulate ambient-dim cap fix). Does not block any downstream consumer.

## Open Questions

1. ~~**Should `triangulate_delaunay` accept a configurable cocircular-tolerance parameter, or is the `T::epsilon() * 100` hard-coded default adequate?**~~ **Resolved at D0-1.3 (2026-05-26): hard-code `T::epsilon() * 100`.** Matches the workspace convention established by `harden-simplicial-hodge-degeneracy-detection` Decision 5 (every near-zero comparison uses the same scaling). The motivating use case (`spec.md` 4.4 unit square cocircular on the unit circle) has margin orders-of-magnitude greater than the threshold on `f64`. Adding `triangulate_delaunay_opts { cocircular_tol: T }` doubles the public API surface for a knob no current consumer needs; defer until one asks. If precision becomes a real concern, the followup is the adaptive-precision predicate (Shewchuk's exact arithmetic) which subsumes the tolerance knob entirely.
2. **Should the new method also produce the dual Voronoi diagram?** Voronoi-Delaunay duality is one of the cleanest mathematical structures in computational geometry, and the dual is needed for some advanced DEC schemes (circumcentric duals). For this change set: no. Voronoi is an out-of-scope future change set.
3. **Should we ship a `triangulate_delaunay_random_insertion` variant for `O(n log n)` average behaviour?** Probably not until a downstream consumer hits the quadratic worst case. Decide at H3 follow-up review.
