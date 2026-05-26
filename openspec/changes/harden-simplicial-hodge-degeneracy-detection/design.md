## Context

`PointCloud::triangulate` returns `Result<SimplicialComplex<T>, TopologyError>` and currently emits exactly one error: empty input. Every other failure mode is masked by silent-zero substitution inside the lumped-mass Hodge ⋆ block. Three branches are involved:

| Site | Trigger | Current behaviour | Caller-visible symptom |
|------|---------|-------------------|------------------------|
| [`op_triangulate.rs:42`](../../../deep_causality_topology/src/types/point_cloud/ops/op_triangulate.rs#L42) | Duplicate input points → zero edge length | Edge mass diagonal contains `0` | `Manifold::laplacian` produces a singular operator at grade 1 |
| [`op_triangulate.rs:289`](../../../deep_causality_topology/src/types/point_cloud/ops/op_triangulate.rs#L289) | Degenerate top simplex (vol ≤ 1e-12) | Top mass diagonal substituted with `0` | `Manifold::codifferential` at grade `max_dim` returns identically zero |
| [`op_triangulate.rs:95`](../../../deep_causality_topology/src/types/point_cloud/ops/op_triangulate.rs#L95) | Singular Gram matrix in volume computation | `gaussian_determinant` returns `T::zero()` | Same as (2), propagated through `simplex_volume` |

`gitnexus_impact upstream` reports **CRITICAL** risk: 131 impacted symbols, 45 direct callers, 12 modules, 2 execution flows. The flaw is therefore live in the workspace today, not theoretical.

The strict per-component L2 norm cross-backend test scenario from the archived `add-hodge-decomposition/spec.md` 4.4 cannot be trusted while the simplicial backend can silently return zero. The cross-backend agreement at strict tolerance could be a false positive driven by both sides returning the zero element. Closing the silent-zero flaw is a prerequisite for `add-pointcloud-delaunay-triangulation`, which depends on a shared `build_lumped_mass_hodge_ops` helper with a sound precondition contract.

Stakeholders: every consumer of `Manifold::with_metric` and downstream DEC operators; the 45 direct fixture call sites identified by impact analysis; the future `add-pointcloud-delaunay-triangulation` change set; future consumers of any planar simplicial Hodge decomposition pipeline.

## Goals / Non-Goals

**Goals:**

- Classify each of the three silent-zero branches as either kept-silent (defined behaviour) or surfaced-as-error (`TopologyError::PointCloudError`). The classification is justified per-branch in Decision 1.
- Make `PointCloud::triangulate` surface every classified error through its existing `Result<_, TopologyError>` return type. Public signature unchanged.
- Audit and update every direct caller surfaced by `gitnexus_impact upstream` so that fixtures that currently succeed silently with a singular complex either tighten the input or propagate the new `Err` deliberately.
- Document the precondition contract on `PointCloud::triangulate` so that downstream callers — including the future shared helper — can rely on it.
- Pick the contract style (fallible-helper vs. witness-newtype) for the future shared helper, so the follow-up Delaunay change set inherits an unambiguous decision.

**Non-Goals:**

- Adaptive-precision predicates (Shewchuk's exact arithmetic). The detection thresholds use `T::epsilon()`-scaled rules and document the regime in which they are sound. Adaptive precision is a follow-up only if a downstream consumer hits the floating-point edge.
- Reworking the lumped-mass Hodge ⋆ scheme itself. The mass formula stays; only the degeneracy boundary changes.
- Adding a new public type (witness newtype) for now. Decision 2 lands on the helper signature as the contract carrier; the newtype option is recorded as a deferred alternative.
- 3D Delaunay tetrahedralisation, planar Delaunay triangulation, or any new triangulation method. Those belong to `add-pointcloud-delaunay-triangulation`.
- Replacement of the existing `PointCloud::triangulate` (Vietoris-Rips). It stays; its success domain narrows but its API and intent are unchanged.

## Decisions

### Decision 1: Classify the three silent-zero branches

Each branch is classified independently. The decision is justified by what the branch *means* geometrically, not by what is easiest to change.

**(a) Zero-length 1-simplex (duplicate input points).** Classification: **surface as error**.

Two identical points in `coords` produce an edge of length zero. The lumped-mass formula for the edge contains a zero diagonal entry. Downstream, the codifferential at grade 1 multiplies through this zero, producing a degenerate Laplacian. There is no geometric interpretation in which a zero-length edge is meaningful for a DEC pipeline: it represents two physically-coincident vertices that should have been merged before triangulation. Surfacing an error forces the caller to deduplicate, which is the correct action.

Error message: `"triangulate: input contains duplicate point at index {i} (distance to point {j} below T::epsilon())"`.

**(b) Zero-volume top simplex (coplanar/collinear k-clique).** Classification: **surface as error**.

A k-clique whose vertices are coplanar in a (k-1)-or-lower-dimensional ambient subspace produces a top simplex of vanishing volume. This is geometrically meaningful information — the input geometry is degenerate at the simplex's location — but the lumped-mass Hodge ⋆ formula at grade `max_dim` has no defined value: the formula divides one by the volume, and zero is not a valid value. Substituting `0` silently is mathematically wrong. The right action is to error and force the caller to either tighten the input or pick a different triangulation strategy.

Error message: `"triangulate: top-dimensional simplex at index {i} has volume below tolerance ({vol} < T::epsilon() * 100), indicating degenerate input geometry"`.

The threshold `T::epsilon() * 100` replaces the current hard-coded `1e-12`, restoring precision-parametric behaviour required by the workspace `RealField` convention.

**(c) Singular Gram matrix in `gaussian_determinant`.** Classification: **surface as error, but only through (b)**.

A singular Gram matrix arises when the vectors forming the simplex are linearly dependent — which is exactly the condition for branch (b). The two branches detect the same underlying degeneracy from different sides: (b) at the volume comparison, (c) at the determinant pivot. Surfacing both as separate errors confuses callers. Instead:

- `gaussian_determinant` is updated to return `Result<T, &'static str>` internally (the `&'static str` is converted to `TopologyError::PointCloudError` at the boundary). The pivot-collapse case returns `Err`, not `T::zero()`.
- `simplex_volume` propagates the error.
- The top-simplex branch in `build_lumped_mass_hodge_ops` reports the volume-degenerate error message (branch (b) wording) regardless of whether the source was pivot collapse or threshold compare. This is honest because both are the same underlying condition.

### Decision 2: Contract style — fallible helper, no witness newtype

The future shared helper introduced by `add-pointcloud-delaunay-triangulation` will be:

```rust
pub(super) fn build_lumped_mass_hodge_ops<T>(
    skeletons: &[Skeleton],
    coords: &[T],
    dim: usize,
) -> Result<Vec<CsrMatrix<T>>, TopologyError>
where T: Float + Sum + From<f64> + Zero + Copy,
```

**Why fallible-helper, not witness-newtype:**

- The helper is `pub(super)`, called from two sites in one module hierarchy. Witness-newtype boilerplate (`NonDegenerateSimplicialGeometry`, fallible `try_new`, separate type for each precondition class) is over-engineered for two callers.
- Rust newtypes don't compose well with the existing `Skeleton`/`Simplex`/coordinate-slice trio that the helper already accepts. Wrapping all three in a newtype either forces a copy or requires lifetime gymnastics for a borrow.
- The fallibility surface is the same in both designs — the type system cannot prove non-degeneracy at compile time, so either the helper checks at call time (b) or a constructor checks at construction time (a). Total runtime cost is identical.
- The error type is already `TopologyError::PointCloudError(String)`. The existing variant absorbs every degeneracy message without API expansion.

**Decision recorded for the follow-up change set:** `add-pointcloud-delaunay-triangulation` extracts the helper with the signature above. The Delaunay caller's upstream guards (D != 2, n < 3, non-collinear) guarantee none of the helper's error paths fire on Delaunay-built complexes; the helper is provably-infallible *at that call site*, while remaining fallible at the boundary.

### Decision 3: Where the duplicate-point check lives

The duplicate-point check is at the input-coordinate level, not at the 1-simplex level. Checking it inside the lumped-mass loop runs in O(N_edges); checking it at coordinate-array entry runs in O(N_points · D). For the workspace's target sizes (≤ 16³ ≈ 4096 points), both are negligible.

**Place the check at the top of `PointCloud::triangulate`**, alongside the existing empty-input check. The check rejects the input before any simplex is built. The 1-simplex branch in `build_lumped_mass_hodge_ops` becomes a debug-only `debug_assert!` that distance is positive; in release builds, the upstream check guarantees the assertion holds.

The check uses `T::epsilon() * max_extent` as the threshold, where `max_extent` is the bounding-box max-axis range. This is scale-invariant.

### Decision 4: Per-fixture audit policy

The 45 direct callers identified by `gitnexus_impact upstream` are audited one module at a time. The policy for each fixture:

1. **Read the fixture's coordinate construction.** Determine whether the input is provably non-degenerate (e.g. axis-aligned unit triangle) or potentially degenerate (e.g. arbitrary user-supplied points).
2. **For provably non-degenerate fixtures:** no source change. `.unwrap()` and `?` continue to work because the new error paths are unreachable on the fixture's input.
3. **For potentially-degenerate fixtures:** decide between (i) tightening the fixture so it is provably non-degenerate (preferred), or (ii) propagating the new `Err` explicitly. Option (i) is preferred because the fixture's intent is usually to test the kernel's *normal* behaviour, not to probe degeneracy handling.
4. **Document the decision** in a one-line comment at the fixture's construction site, of the form `// non-degenerate by construction: unit triangle / axis-aligned lattice / etc.`.

The audit is paced one module per task in `tasks.md`, with a `cargo test -p <crate>` gate after each module's audit lands.

### Decision 5: Tolerance constants are RealField-parametric

The existing `1e-12` literals at [`op_triangulate.rs:95`](../../../deep_causality_topology/src/types/point_cloud/ops/op_triangulate.rs#L95) and [`op_triangulate.rs:289`](../../../deep_causality_topology/src/types/point_cloud/ops/op_triangulate.rs#L289) violate the workspace `RealField` convention from `add-hodge-decomposition`. They are replaced with `T::epsilon() * <T as From<f64>>::from(100.0)` (the workspace-standard 100×ε scaling for "near-zero" comparisons on generic float types).

This affects only the threshold comparison sites; the substituted-zero behaviour goes away entirely (replaced by error returns per Decision 1).

## Risks / Trade-offs

- **[Risk] An audited fixture is misclassified as non-degenerate and silently regresses.** The audit relies on inspection, not proof. A fixture that *looks* generic-position but actually constructs a degenerate sub-clique would now error in production.
  → **Mitigation:** add a regression test per affected module that constructs the module's canonical fixture and asserts `triangulate` returns `Ok`. This is a one-line cost per module and catches regressions at `cargo test` time. The CRITICAL impact severity justifies the extra discipline.

- **[Risk] The `T::epsilon() * 100` threshold rejects geometry that the old `1e-12` threshold accepted (or vice versa) on `T = f64`.** `f64::EPSILON ≈ 2.22e-16`; `100 × f64::EPSILON ≈ 2.22e-14`, two orders of magnitude tighter than `1e-12`. The new threshold is therefore *stricter*: inputs in the gap `[2.22e-14, 1e-12)` now error, where they previously silently zeroed.
  → **Assessment:** this is correct. The gap was exactly the silent-zero regime. Inputs producing volumes in that range were always numerically suspect; surfacing them as errors is the right behaviour. Documented in the precondition contract.

- **[Risk] Two execution flows in `examples/medicine_examples` could break at runtime.** `tissue_classification` and `aneurysm_risk` build mock manifolds via `triangulate`. If either's fixture is degenerate, the demo errors at step 1.
  → **Mitigation:** the demos are audited in their own task block (Block H4). The likely fix is fixture-tightening; both demos use illustrative geometry that can be adjusted without affecting the demo's pedagogical content.

- **[Risk] Behaviour change is invisible to `cargo build` and only surfaces under `cargo test`.** A consumer crate outside the workspace that handles `triangulate`'s `Result` via `.unwrap()` on a degenerate input now panics in production. No workspace-internal caller does this — every internal `.unwrap()` is on a provably-non-degenerate fixture — but external consumers cannot be audited.
  → **Mitigation:** the change is documented in `deep_causality_topology/CHANGELOG.md` under a "Behaviour change" header. Per the workspace convention, this is a minor-version bump for the crate. External consumers reading the changelog will see the narrowed success domain.

- **[Trade-off] The error messages contain integer indices and float values, which complicates structural equality testing.**
  → **Documented:** tests assert error-message substring matches on the discriminating phrase (`"duplicate point"`, `"top-dimensional simplex"`, `"below tolerance"`), not on full string equality.

## Migration Plan

This change set is purely additive on the public type surface:

- **Source compatibility:** `PointCloud::triangulate` signature unchanged. Callers using `?` continue to work; the new `Err` paths propagate through the existing `Result`.
- **Behaviour compatibility:** narrows. Inputs that previously produced silently-singular complexes now return `Err`. No input that previously produced `Ok(complex)` with a *non*-singular complex changes behaviour.
- **Rollback:** revert the change set. The silent-zero behaviour returns. No persisted state, no schema changes.
- **Sequencing:** open before `add-pointcloud-delaunay-triangulation`. The Delaunay change set's task 2.1 (helper extraction) inherits the precondition contract from Decision 2 of this design.
- **Caller audit:** module-by-module per Decision 4, gated at each block boundary in `tasks.md`. The user signs off at each block before the next opens.

## Open Questions

1. **Should the `examples/medicine_examples` demos be audited in this change set or deferred?** They are end-to-end demos, not test fixtures. If their geometry is provably non-degenerate, no change is needed; if not, the fixture-tightening is in scope. Audit them in Block H4 and decide based on what the audit finds.
2. **Should the regression-test fixtures cover all three classified error categories, or only the two that source-changes affect ((a) duplicate-point and (b) volume-below-tolerance)?** Branch (c) (singular Gram matrix) is sub-classified under (b); a dedicated test would assert the unified error message regardless of source. Decide at Block H1 close — a single regression test exercising the unified path is sufficient.
3. **Should `CHANGELOG.md` flag this as a breaking change or a bug fix?** The behaviour narrows, which is technically not source-compatibility-breaking. Workspace convention treats this as a bug fix with a minor-version bump. Confirm at Block H5 (release prep).
