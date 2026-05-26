## 1. Block H0 — Preflight

- [ ] 1.1 Confirm `gitnexus_impact upstream` against `triangulate` is current. If the index is stale per the GitNexus tool's freshness warning, run `npx gitnexus analyze` first. Capture the impact snapshot (45 direct callers, 12 modules, 2 execution flows) in this change set's `design.md` Context section.
- [ ] 1.2 Confirm Decision 2 — fallible-helper signature, not witness-newtype — is the contract style for the future shared helper. Cross-reference with `add-pointcloud-delaunay-triangulation` task 2.1 to ensure the proposed helper signature matches.
- [ ] 1.3 Resolve `design.md` Open Question 2: a single regression test for the unified volume-degenerate error message (covering both branch (b) threshold-compare and branch (c) Gram-matrix singular) is sufficient. Document the decision inline.
- [ ] 1.4 H0-G1 Review: user signs off on the four preflight resolutions before any code lands.

## 2. Block H1 — Source-side detection logic

- [ ] 2.1 Add duplicate-point detection at the top of `PointCloud::triangulate`. Loop over `coords` pairs; reject the first pair whose Euclidean distance is below `T::epsilon() * max_extent` with `Err(TopologyError::PointCloudError(...))` whose message contains `"duplicate point"`, `"index {i}"`, `"index {j}"`. The check runs in O(N²·D) and is acceptable for the workspace's target sizes (≤ 4096 points).
- [ ] 2.2 Replace the hard-coded `1e-12` threshold at [`op_triangulate.rs:289`](../../../deep_causality_topology/src/types/point_cloud/ops/op_triangulate.rs#L289) with `T::epsilon() * <T as From<f64>>::from(100.0)`. When the top simplex volume falls below the threshold, return `Err` instead of substituting `T::zero()`. Error message contains `"top-dimensional simplex"`, `"index {i}"`, `"below tolerance"`.
- [ ] 2.3 Change `gaussian_determinant`'s pivot-collapse branch at [`op_triangulate.rs:95`](../../../deep_causality_topology/src/types/point_cloud/ops/op_triangulate.rs#L95) to return a sentinel (e.g. an `Option<T>` or a dedicated `Result<T, ()>`) instead of `T::zero()`. Update `simplex_volume` to propagate the sentinel. The top-simplex branch in the lumped-mass loop converts the sentinel into the same `"top-dimensional simplex...below tolerance"` error message — both detection paths produce the unified error per `design.md` Decision 1(c).
- [ ] 2.4 Add a debug-only `debug_assert!(primal_vol > T::zero(), "...")` in the intermediate-dimension branch (edges in 2D/3D). The upstream duplicate-point check guarantees the assertion holds in release builds; the assertion catches future regressions where degenerate edges sneak in.
- [ ] 2.5 Update the documentation comment on `PointCloud::triangulate` to document the precondition contract: the three rejection rules, their messages, and the `T::epsilon() * 100` scaling convention. Cross-reference `design.md` Decisions 1 and 5.
- [ ] 2.6 H1-G1 Compilation: `cargo build -p deep_causality_topology` clean (release + debug); `cargo clippy -p deep_causality_topology --all-targets -- -D warnings` clean. Fix lints at root cause; no `#[allow(clippy::...)]` suppressions per `feedback_clippy_lints`.

## 3. Block H2 — Regression test fixtures

- [ ] 3.1 Add `tests/types/point_cloud/op_triangulate_degeneracy_tests.rs`. Register in `tests/types/point_cloud/mod.rs` and `tests/BUILD.bazel`.
- [ ] 3.2 Unit test: duplicate-point rejection. Construct a `PointCloud<f64, 2>` with two identical points; assert `triangulate` returns `Err` whose message contains `"duplicate point"`.
- [ ] 3.3 Unit test: collinear 3-point input. Construct three points on a line in 2D; assert `triangulate` with `radius >= max_pairwise_distance` returns `Err` whose message contains `"top-dimensional simplex"` and `"below tolerance"`.
- [ ] 3.4 Unit test: coplanar 4-clique in 3D ambient. Construct four points on the z=0 plane in `PointCloud<f64, 3>`; assert `triangulate` with sufficient radius returns `Err` whose message contains `"top-dimensional simplex"` (covers the previously-capped clique-expansion path).
- [ ] 3.5 Unit test: non-degenerate generic-position input still succeeds. Construct three points forming a unit triangle; assert `triangulate` returns `Ok(complex)` with the expected vertex/edge/triangle counts. This is the regression-prevention test for accidental over-rejection.
- [ ] 3.6 Unit test: the `T::epsilon() * 100` threshold is honoured. Construct an input whose top simplex volume falls just above and just below the threshold; assert the boundary behaviour matches the documented contract.
- [ ] 3.7 H2-G1 Coverage: 100% on the new regression test file; 100% on the new source paths in `op_triangulate.rs`. Verify via `make coverage` or equivalent.
- [ ] 3.8 H2-G2 Compilation + own-crate tests: `cargo test -p deep_causality_topology` passes (no caller-side audit yet, so callers may still break — that's expected and addressed in Block H3+).
- [ ] 3.9 H2-G3 Review: user reviews H1+H2 diff, confirms the source-side detection logic is sound before the caller audit opens.

## 4. Block H3 — Caller audit: `deep_causality_topology` internal

- [ ] 4.1 Audit `tests/types/manifold/*` fixtures. The `gitnexus_impact` snapshot identifies 9 direct setup helpers (`setup_triangle_manifold`, `setup_valid_manifold_parts`, `setup_manifold_with_metric`, `setup_manifold_no_metric`, `setup_manifold_with_data`, `single_triangle_manifold`, `simplicial_triangle_manifold`, etc.). For each: read the construction, classify as provably-non-degenerate or potentially-degenerate, document the classification with a one-line comment, and tighten the input if needed.
- [ ] 4.2 Audit `tests/types/point_cloud/op_triangulate_tests.rs` fixtures (6 tests including `test_point_cloud_triangulate_caps_top_grade_at_ambient_dim_for_coplanar_2d_corners`). Note: this test explicitly exercises the coplanar 4-clique case that now errors — it must be updated to assert the new `Err` behaviour or split into a separate "this used to silently work, now errors" regression test.
- [ ] 4.3 Audit `tests/types/point_cloud/point_cloud_tests.rs` (`test_triangulate_varying_radius`, `test_triangulate_zero_radius`).
- [ ] 4.4 Audit `tests/types/gauge/gauge_field/*` (1 direct + 9 indirect via `create_u1_gauge_field`).
- [ ] 4.5 Audit `tests/types/regge_geometry/has_hodge_star_tests.rs` (2 direct).
- [ ] 4.6 Audit `tests/types/topology/topology_tests.rs` (1 direct, `create_simple_topology`).
- [ ] 4.7 H3-G1 Compilation + own-crate tests: `cargo test -p deep_causality_topology` passes clean.
- [ ] 4.8 H3-G2 Review: user reviews the per-fixture decisions before downstream crates are touched.

## 5. Block H4 — Caller audit: `deep_causality_physics` and examples

- [ ] 5.1 Audit `tests/kernels/condensed/*` (`create_flat_manifold` used by `test_foppl_von_karman_strain_full` and `test_wrapper_strain_full`).
- [ ] 5.2 Audit `tests/kernels/em/*` (`create_simple_manifold` used by 9 EM kernel tests).
- [ ] 5.3 Audit `tests/kernels/quantum/*` (`create_simple_manifold` used by 5 Klein-Gordon kernel tests).
- [ ] 5.4 Audit `tests/kernels/thermodynamics/*` (`create_temp_manifold` used by 4 heat-diffusion kernel tests).
- [ ] 5.5 Audit `tests/kernels/mhd/*` (`create_dummy_manifold`, `create_test_manifold` used by ideal/resistive GRMHD tests, including the explicit error tests `test_relativistic_current_kernel_low_dim_metric_error` and `test_relativistic_current_kernel_low_skeleton_error`).
- [ ] 5.6 Audit `examples/medicine_examples/tissue_classification/main.rs::analyze_with_monad`. If the demo fixture is degenerate, tighten it; if it is generic-position, no change.
- [ ] 5.7 Audit `examples/medicine_examples/aneurysm_risk/main.rs::build_mock_aneurysm`. Same audit policy as 5.6.
- [ ] 5.8 Audit `examples/mathematics_examples/topology/{complex_operators.rs, differential_field.rs}` (2 direct, both call `triangulate` in `main`).
- [ ] 5.9 H4-G1 Compilation + workspace tests: `make build && make test` clean. The medicine examples must run to completion without runtime error.
- [ ] 5.10 H4-G2 Review: user reviews the cross-crate audit diff and runtime-validates both medicine demos before sign-off.

## 6. Block H5 — Release prep + archive sync

- [ ] 6.1 Update `deep_causality_topology/CHANGELOG.md` with an entry under the next minor version: "Behaviour change: `PointCloud::triangulate` now rejects three previously-masked degeneracy classes (duplicate input points, zero-volume top simplex, singular Gram matrix) with discriminating `TopologyError::PointCloudError` messages. Inputs that previously produced a silently-singular complex now return `Err`. Document the precondition contract in the method's doc-comment."
- [ ] 6.2 Update the archived `openspec/changes/archive/2026-05-22-add-hodge-decomposition/design.md` Risk 5 entry: the Vietoris-Rips silent-zero risk is now closed. Cross-reference this change set.
- [ ] 6.3 Update `openspec/changes/add-pointcloud-delaunay-triangulation/design.md` Decision 5 to reflect that the helper signature is `Result<Vec<CsrMatrix<T>>, TopologyError>` per this change set's Decision 2. Remove any lingering "infallible" language.
- [ ] 6.4 Update `openspec/changes/add-pointcloud-delaunay-triangulation/tasks.md` task 2.1 helper signature to match Decision 2.
- [ ] 6.5 H5-G1 Compilation + workspace tests: `make format && make fix && make build && make test` clean.
- [ ] 6.6 H5-G2 Final review: user reviews the full H0–H5 diff, confirms the medicine demos run, signs off, commits. This is the change-set-closing commit; after H5-G2 the change set is ready to archive.

## 7. Deferred Work (not in scope for this change set)

- Adaptive-precision in-circumcircle predicate (Shewchuk's robust predicates). Mentioned in `design.md` Non-Goals; opens only if a downstream consumer hits floating-point edge behaviour the `T::epsilon() * 100` threshold does not handle.
- Witness-newtype precondition type (`NonDegenerateSimplicialGeometry`). Recorded as deferred alternative in `design.md` Decision 2. Opens only if the helper grows to more than two callers.
- Reworking the lumped-mass Hodge ⋆ scheme itself (barycentric → circumcentric duals). Out of scope; the mass formula stays as-is, only the degeneracy boundary changes.
- Documenting which downstream operators (`Manifold::laplacian`, `Manifold::codifferential`, `Manifold::hodge_decompose`) now have strictly-narrower input domains. The narrowing follows transitively from `triangulate`'s narrowed success domain; explicit per-operator audit is a follow-up if a consumer asks.
