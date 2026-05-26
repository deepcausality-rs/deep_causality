## Why

`PointCloud::triangulate` builds a Vietoris-Rips simplicial complex and populates its lumped-mass Hodge ⋆ operators inline. The Hodge ⋆ construction at [`deep_causality_topology/src/types/point_cloud/ops/op_triangulate.rs:240-312`](../../../deep_causality_topology/src/types/point_cloud/ops/op_triangulate.rs#L240-L312) contains three branches that silently substitute `T::zero()` when input geometry is degenerate:

1. **Zero-length 1-simplex** — `euclidean_distance` returns `0` for duplicate input points. The intermediate-dimension branch at line 305 stores `0` into the diagonal of the edge mass matrix.
2. **Zero-volume top simplex** — the `primal_vol > T::from(1e-12)` guard at line 289 substitutes `0` when the top-dimensional simplex degenerates (e.g. coplanar 4-clique in 3D ambient).
3. **Singular Gram matrix** — `gaussian_determinant` at line 95 returns `T::zero()` whenever any pivot collapses below `1e-12`, masking near-singular geometry.

All three produce a structurally-valid `SimplicialComplex<T>` whose Hodge ⋆ matrix is numerically singular at one or more grades. Downstream consumers — `Manifold::with_metric`, `Manifold::laplacian`, `Manifold::codifferential`, `Manifold::hodge_decompose` — accept the complex and either return NaN, return garbage, or silently agree with another singular complex on a cross-backend comparison. The latter case is the most dangerous: a false-positive cross-backend agreement at strict tolerance, because both sides are zero.

`triangulate` is called from 45 direct sites across 12 modules (per `gitnexus_impact`, risk: **CRITICAL**). The latent flaw is therefore not a corner-case theoretical concern; it is sitting in fixtures across the EM, Quantum, Condensed-matter, Thermodynamics, MHD, Gauge-field, and Regge-geometry test surfaces, plus two end-to-end medicine examples.

The motivating consequence: the strict per-component L2 norm cross-backend test scenario from [`openspec/changes/archive/2026-05-22-add-hodge-decomposition/specs/hodge-decomposition/spec.md`](../archive/2026-05-22-add-hodge-decomposition/specs/hodge-decomposition/spec.md) 4.4 cannot be trusted while the simplicial backend can silently return zero. The follow-up change set `add-pointcloud-delaunay-triangulation` depends on this one — it extracts the lumped-mass Hodge ⋆ block into a shared helper, and that helper must have a sound contract with both callers.

## What Changes

- **Decision point 1:** classify each of the three silent-zero branches as either (a) defined behaviour to keep silently or (b) genuine error to surface via `TopologyError::PointCloudError(String)`. The design doc justifies each classification individually.
- **Decision point 2:** choose the contract style for the future shared helper — fallible-helper signature or witness-newtype precondition. The follow-up change set inherits this decision.
- **Decision point 3 (added mid-implementation):** refactor `SimplicialComplex<T>` to populate Hodge ⋆ operators *lazily* on first access rather than eagerly during `PointCloud::triangulate`. The eager design conflated topological (TDA / clique-complex / Euler-characteristic) consumers with geometric (DEC / Hodge ⋆ / Laplacian) consumers, forcing TDA consumers to satisfy geometric preconditions they neither needed nor used. The lazy refactor moves the degeneracy rejection from `triangulate` to the lazy access path, so TDA consumers succeed on any non-duplicate input while DEC consumers see the same loud error from H1 the moment they touch the Hodge ⋆ surface.
- Make `PointCloud::triangulate` surface every classified error through its existing `Result<SimplicialComplex<T>, TopologyError>` return type. Public signature unchanged; behaviour change is strictly narrowing — inputs that previously produced a silently-singular complex now return `Err`.
- Audit all 45 direct callers identified by `gitnexus_impact upstream`. For each, decide:
  - Generic-position fixture: behaviour preserved, `.unwrap()` and `?` continue to work.
  - Potentially-degenerate fixture: tighten the input, or propagate `Err` deliberately.
- Add a regression test per classified error category, exercising the exact degeneracy that previously slipped through silently.
- Document the precondition contract on `PointCloud::triangulate`: which input classes succeed, which fail loudly with which error, and which are accepted with a documented numerical caveat.
- Update the `add-hodge-decomposition` archived `design.md` Risk 5 entry: the silent-zero risk is now closed, not deferred.
- **Hard precision rule:** the change touches a generic `T: Float + From<f64>` signature surface. No `f64` appears in any new public signature; the existing `From<f64>` bound is preserved.
- **Static dispatch only.**
- **No new external dependencies.**

## Capabilities

### New Capabilities

- `simplicial-hodge-degeneracy-detection`: rejection rules for the three geometric degeneracies that `PointCloud::triangulate` previously masked. Covers the discriminating `TopologyError::PointCloudError` messages, the documented precondition contract, and the regression test fixtures.
- `lazy-hodge-star-population`: `SimplicialComplex<T>` no longer pre-computes Hodge ⋆ operators during construction. The new `SimplicialComplex::with_geometry(...)` constructor stores coordinates and ambient dimension; the new fallible accessor `SimplicialComplex::hodge_star_operators(&self) -> Result<&Vec<CsrMatrix<T>>, TopologyError>` builds and caches the operators on first invocation. TDA consumers that never touch the accessor are unaffected by the degeneracy rejection.

### Modified Capabilities

- None at the OpenSpec spec layer. The behaviour change is a narrowing of `PointCloud::triangulate`'s success domain. No archived spec.md currently asserts the silent-zero behaviour (because it was never intentional); therefore no archived requirement changes.

## Impact

- **Crate affected:** `deep_causality_topology` source (`op_triangulate.rs`). Caller-side ripple lands in `deep_causality_physics` tests and `examples/medicine_examples`.
- **Public surface:**
  - `PointCloud::triangulate` signature unchanged. Behaviour narrows; new `Err` paths added under the existing `TopologyError::PointCloudError(String)` variant. No new error variant.
  - `SimplicialComplex::hodge_star_operators(&self)` signature widens from `-> &Vec<CsrMatrix<T>>` to `-> Result<&Vec<CsrMatrix<T>>, TopologyError>`. All 5 direct callers identified by `gitnexus_impact upstream` migrate (3 propagate `?` through existing `Result` returns; 2 are tests).
  - `SimplicialComplex::new(skeletons, boundary, coboundary, hodge_star_operators)` signature unchanged for backwards compatibility with ~20 test-fixture call sites.
  - New constructor `SimplicialComplex::with_geometry(skeletons, boundary, coboundary, coords, ambient_dim)` for the lazy-construction path.
  - `HasHodgeStar::hodge_star_matrix` trait method widens to `-> Result<Cow<'_, CsrMatrix<R>>, TopologyError>`. Both simplicial and cubical impls update in parallel.
  - No new error variant — every new `Err` path lands under `TopologyError::PointCloudError(String)`.
  - No panics anywhere in the API surface. Every fallible path returns `Result`.
- **Caller audit:** 45 direct call sites. Per the `gitnexus_impact upstream` distribution:
  - `Manifold` test setup helpers: 58 hits (direct)
  - `Em` (electromagnetic kernel tests): 9 hits (indirect)
  - `Gauge_field`: 5 hits (direct)
  - `Quantum` (Klein-Gordon, etc.): 5 hits (indirect)
  - `Condensed` (Föppl-von-Kármán strain): 4 hits (direct)
  - `Thermodynamics` (heat-diffusion): 4 hits (indirect)
  - `Regge_geometry`, `Topology`, `Point_cloud`: 8 hits (direct)
  - `Mhd` (ideal/resistive GRMHD): 2 hits (indirect)
  - `Topological_invariants`, `Differential`: 3 hits (indirect)
  - `examples/medicine_examples/{aneurysm_risk, tissue_classification}`: 2 execution flows
- **Execution flows at risk:** 2 end-to-end medicine demos. If their `build_mock_aneurysm` / `analyze_with_monad` constructs a degenerate complex today, they error at step 1 post-change. The audit decides whether to tighten the fixture or accept the new error path.
- **Effort:** ~80 LOC source + ~150 LOC regression tests + ~100 LOC fixture audit edits across ~10 test files. Estimated 8–14 hours of focused work, the bulk in the per-fixture audit and decision documentation.
- **Unblocks:** `add-pointcloud-delaunay-triangulation`. The Delaunay change set's `build_lumped_mass_hodge_ops` helper extraction is contingent on a sound precondition contract; this change set establishes it.
- **Out of scope:**
  - Adaptive-precision predicates (Shewchuk's exact predicates). The detection thresholds use `T::epsilon()`-scaled rules and document the regime in which they are sound.
  - 3D Delaunay tetrahedralisation, manifold-respecting triangulation, or any new triangulation method. Those land in `add-pointcloud-delaunay-triangulation`.
  - Reworking the Hodge ⋆ scheme itself (barycentric → circumcentric duals). The change preserves the lumped-mass approximation exactly; only the degeneracy boundary changes.
  - Public API additions. No new `pub` items.
- **Agent conduct:** per AGENTS.md golden rule, agents never `git commit`. Per-block gates apply. The 45-caller audit is paced one module at a time; user signs off at each block boundary.
