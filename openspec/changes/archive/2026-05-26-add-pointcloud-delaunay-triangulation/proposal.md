## Why

`PointCloud::triangulate` is a Vietoris-Rips complex constructor: it builds every k+1-vertex clique whose pairwise edges all fit within a caller-supplied radius, regardless of whether the resulting simplicial complex is a manifold. For a planar point set with K-cliques larger than 3 (e.g. four coplanar corners of a unit square at radius ≥ √2), the resulting complex contains overlapping interior simplices and is rejected by `Manifold::with_metric`'s manifold-property check.

This blocks the strict per-component L2 norm cross-backend equality scenario in [`openspec/changes/add-hodge-decomposition/specs/hodge-decomposition/spec.md`](../add-hodge-decomposition/specs/hodge-decomposition/spec.md) "Two-backend cross-check on the unit square". The Hodge decomposition change set ships with the relaxed orthogonality + vanishing-component-ratio variant of that scenario; the strict variant (`|‖simplicial.exact()‖ − ‖cubical.exact()‖| < 1e-6`) is documented in `tasks.md` Section 6 of that change set and depends on this one.

Beyond the immediate spec.md 4.4 gap, a manifold-respecting triangulation is the canonical input for any DEC-style numerical pipeline on a planar simplicial complex: Hodge ⋆ operators, codifferential, Laplacian, and downstream consumers of `Manifold::hodge_decompose` all assume the input complex is a 2-manifold. Vietoris-Rips is appropriate for topological data analysis (persistent homology, clique complexes), not for DEC.

## What Changes

- Add `PointCloud::triangulate_delaunay(&self) -> Result<SimplicialComplex<T>, TopologyError>` — a 2D Delaunay triangulation producing a manifold-respecting simplicial complex from a planar point set. Generic over `T: RealField + FromPrimitive` with all the bounds the existing `triangulate` carries plus what the in-circumcircle predicate needs.
- The new method MUST produce, for any non-degenerate planar input, a simplicial complex that satisfies `Manifold::with_metric`'s manifold-property check (oriented, link condition, no overlapping interior simplices).
- Preserve the existing `PointCloud::triangulate` (Vietoris-Rips) API unchanged. It has 41 direct callers across the workspace and is the right tool for topological data analysis. The new `triangulate_delaunay` is a sibling method, not a replacement.
- Compute lumped-mass Hodge ⋆ operators on the Delaunay output identically to how the Vietoris-Rips `triangulate` does, so the resulting `SimplicialComplex<T>` is drop-in compatible with `Manifold::with_metric`, the differential operators, and `Manifold::hodge_decompose`.
- Document the robustness strategy explicitly: which degeneracies are handled, which fail loudly with a `TopologyError`, and which are accepted with a documented arbitrary-tiebreak rule.
- Tighten the H3 cross-backend test in `add-hodge-decomposition` from the relaxed orthogonality + vanishing-component-ratio variant to the strict per-component L2 norm equality variant from `spec.md` 4.4. This is the test-only follow-up tracked in `add-hodge-decomposition/tasks.md` Section 6.
- **Hard precision rule:** every new public signature is generic over `T: RealField`. No `f64` appears in the new public surface. Mirrors the convention enforced by `add-hodge-decomposition`.
- **Static dispatch only.** No `dyn`, no trait objects.
- **No new external dependencies.** Bowyer-Watson is ~250-350 LOC of pure numerical code with no external crates required.

## Capabilities

### New Capabilities

- `pointcloud-delaunay-triangulation`: 2D Delaunay triangulation on `PointCloud<T, D>` producing a manifold-respecting `SimplicialComplex<T>`. Covers the `PointCloud::triangulate_delaunay` method, its error variants for degenerate inputs (collinear, fewer-than-3-vertex), the cocircular tiebreak rule, and a documented robustness guarantee against the unit-square corner case.

### Modified Capabilities

- `hodge-decomposition`: the cross-backend cross-check scenario in `spec.md` 4.4 is tightened from "orthogonality + vanishing-component-ratio agreement to 1e-6" to "strict per-component L2 norm equality to 1e-6". No code in `Manifold::hodge_decompose` changes; only the cross-backend test fixture in `hodge_decomposition_cross_backend_tests.rs` is rewritten to use the new Delaunay-built two-triangle simplicial unit square.

## Impact

- **Crate affected:** `deep_causality_topology` only.
- **New public method:** `PointCloud::triangulate_delaunay`.
- **New public error path:** `TopologyErrorEnum::PointCloudError(String)` (existing variant) extended with discriminating messages for the documented Delaunay failure modes — no new variant.
- **Modified test:** `tests/types/manifold/hodge_decomposition_cross_backend_tests.rs` rewrites its simplicial fixture from a single right triangle to the canonical two-triangle unit square, and tightens the cross-backend assertions to the strict per-component L2 norm equality variant from `spec.md` 4.4.
- **Dependencies:** `add-hodge-decomposition` (H1–H3 + the triangulate ambient-dim cap fix) must ship first. The strict cross-backend test depends on `Manifold::hodge_decompose` (H2) and on the existing simplicial Hodge ⋆ machinery; neither changes in this change set.
- **Effort:** ~400 LOC (Bowyer-Watson + Hodge ⋆ population + degeneracy handling) + ~150 LOC tests + ~50 LOC for the tightened cross-backend test. Estimated 6–10 hours of focused work, the bulk of which is in the robustness/degeneracy test sweep.
- **Unblocks:** the strict variant of `add-hodge-decomposition/specs/hodge-decomposition/spec.md` task 4.4. Does not block any current downstream consumer; `add-3d-causal-fluid-dynamics` Block B1 has shipped with the relaxed cross-backend variant.
- **Out of scope:**
  - 3D Delaunay tetrahedralisation. Defer until a downstream consumer needs it.
  - Constrained-Delaunay with boundary edges. Defer.
  - Adaptive-precision in-circumcircle predicate (e.g. Shewchuk's). The first version uses naive `f64` predicates with a documented degeneracy-handling rule; if a downstream consumer hits precision issues, that motivates the adaptive-precision follow-up.
  - GPU paths.
  - Alpha-shapes or weighted Delaunay variants.
- **Agent conduct:** per AGENTS.md golden rule, agents never `git commit`. Per-block gates apply.
