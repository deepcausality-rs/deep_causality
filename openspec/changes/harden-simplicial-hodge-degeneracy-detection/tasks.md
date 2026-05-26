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

## 5. Block H4 — Lazy Hodge ⋆ population refactor

Added mid-implementation after the original Block H4 (caller audit) surfaced a runtime panic in `examples/medicine_examples/tissue_classification`. The demo is a pure Vietoris-Rips / TDA consumer (Euler characteristic from V-E-F counts) and never accesses the Hodge ⋆ surface, but was nonetheless blocked by the eager Hodge ⋆ degeneracy rejection inside `PointCloud::triangulate`. The architectural fix is to move Hodge ⋆ population from `triangulate` (eager) to a lazy access path on `SimplicialComplex<T>`, separating topological consumers from geometric consumers at the type level rather than forcing the union of their preconditions. See `design.md` Decision 6 for the full rationale.

- [ ] 5.1 Create `deep_causality_topology/src/types/simplicial_complex/lazy_hodge_star.rs`. Move `simplex_volume`, `gaussian_determinant`, and the lumped-mass construction loop from `op_triangulate.rs` into this file. Expose `pub(crate) fn build_lumped_mass_hodge_star<T>(skeletons: &[Skeleton], coords: &[T], dim: usize) -> Result<Vec<CsrMatrix<T>>, TopologyError>`. The top-volume rejection from H1 task 2.2 moves into this helper unchanged; the helper is the single source of truth for the unified `"top-dimensional simplex...below tolerance"` error.
- [ ] 5.2 Update `SimplicialComplex<T>` in `src/types/simplicial_complex/mod.rs`:
  - Change `hodge_star_operators: Vec<CsrMatrix<T>>` to `hodge_star_operators: OnceLock<Vec<CsrMatrix<T>>>`.
  - Add `geometric_data: Option<(Vec<T>, usize)>` (coordinates + ambient dimension).
  - Remove `#[derive(Clone, PartialEq)]` (OnceLock does not implement either). Keep `#[derive(Debug)]`.
  - Implement `Clone` manually: clone the OnceLock's current state if initialized; leave uninitialized otherwise.
  - Implement `PartialEq` manually: compare logical identity (skeletons + boundary + coboundary + geometric_data). Two complexes with the same logical structure but different cache populations are equal because the lazy compute is deterministic.
  - Update `Default` to use `OnceLock::new()` and `geometric_data: None`.
  - Update `map()` to use `OnceLock::into_inner()` for the existing-Hodge-⋆ case and remap coords inside `geometric_data` through the same closure.
- [ ] 5.3 Add new constructor `SimplicialComplex::with_geometry(skeletons, boundary, coboundary, coords, ambient_dim) -> Self` in `src/types/simplicial_complex/api/constructors.rs`. Stores `coords + ambient_dim` in `geometric_data`; leaves `hodge_star_operators: OnceLock::new()` empty.
- [ ] 5.4 Keep the existing `SimplicialComplex::new(skeletons, boundary, coboundary, hodge_star_operators)` constructor with its current signature. The OnceLock is pre-populated via `OnceLock::from(...)` (stable since Rust 1.78) or `let cell = OnceLock::new(); cell.set(...).expect(...); cell`. `geometric_data: None`. Backwards-compatible with ~20 existing test-fixture call sites that pre-supply Hodge ⋆.
- [ ] 5.5 Update the accessor in `src/types/simplicial_complex/getters/mod.rs`. The signature widens to `pub fn hodge_star_operators(&self) -> Result<&Vec<CsrMatrix<T>>, TopologyError>`. Implementation: `self.hodge_star_operators.get_or_try_init(|| { ... lazy build via build_lumped_mass_hodge_star ... })`. For empty complexes (no skeletons), return `Ok(empty_vec)` without attempting to build. For complexes with `geometric_data: None` and an uninitialized cell, return `Err(TopologyError::PointCloudError("Hodge ⋆ operators not available: complex was constructed without geometric data"))`.
- [ ] 5.6 Update `src/types/point_cloud/ops/op_triangulate.rs`:
  - Drop the eager lumped-mass loop (lines populating `hodge_ops` in the current implementation).
  - Drop the top-volume rejection here (it moves to `lazy_hodge_star.rs`).
  - Remove `simplex_volume`, `gaussian_determinant`, intermediate-grade primal-volumes computation.
  - Keep `find_duplicate_points` and the duplicate-point precondition check — duplicate points are a `triangulate` precondition unrelated to Hodge ⋆.
  - Switch the construction call from `SimplicialComplex::new(skeletons, boundary, coboundary, hodge_ops)` to `SimplicialComplex::with_geometry(skeletons, boundary, coboundary, coords, dim)`.
  - Update the doc-comment on `PointCloud::triangulate`: it now rejects only duplicate-point and empty-input cases; the top-volume rejection moves to the Hodge ⋆ access path and is documented there.
- [ ] 5.7 Update `src/traits/has_hodge_star.rs`: change `HasHodgeStar::hodge_star_matrix` signature from `fn hodge_star_matrix(...) -> Cow<'_, CsrMatrix<R>>` to `fn hodge_star_matrix(...) -> Result<Cow<'_, CsrMatrix<R>>, TopologyError>`.
- [ ] 5.8 Update both `HasHodgeStar` impls: `src/types/regge_geometry/has_hodge_star.rs` propagates `?` from the new fallible accessor; `src/types/cubical_regge_geometry/has_hodge_star.rs` wraps its existing infallible computation in `Ok(...)`. No behaviour change on the cubical side.
- [ ] 5.9 Migrate the 5 direct callers identified by `gitnexus_impact upstream`:
  - `deep_causality_physics/src/kernels/mhd/grmhd.rs::relativistic_current_kernel`: propagate `?`.
  - `deep_causality_physics/src/kernels/mhd/ideal.rs::ideal_induction_kernel`: propagate `?`.
  - `deep_causality_topology/src/types/regge_geometry/has_hodge_star.rs::hodge_star_matrix`: handled by 5.7+5.8.
  - `deep_causality_topology/tests/.../op_triangulate_tests.rs::test_point_cloud_triangulate_caps_top_grade_at_ambient_dim_for_coplanar_2d_corners`: `.unwrap()` migration; 4 unit-square corners produce non-degenerate triangles, lazy init succeeds.
  - `deep_causality_topology/tests/.../has_hodge_star_tests.rs::trait_routed_matrix_equals_complex_cache_matrix`: `.unwrap()` migration.
- [ ] 5.10 Audit any other indirect accessor caller in the workspace (grep for `hodge_star_operators()` and `hodge_star_matrix(`). Migrate to fallible handling.
- [ ] 5.11 Update H1's regression tests in `op_triangulate_degeneracy_tests.rs`. The `test_triangulate_rejects_three_collinear_points_in_2d`, `test_triangulate_rejects_four_coplanar_points_in_3d`, and `test_triangulate_rejects_volume_below_threshold` tests previously asserted `triangulate` returned `Err`. After the lazy refactor, `triangulate` returns `Ok` on these inputs and the error surfaces at `complex.hodge_star_operators()` access. Rewrite the tests to call the accessor and assert `Err` there. The unified error message is unchanged.
- [ ] 5.12 H4-G1 Compilation + workspace tests: `cargo build -p deep_causality_topology` + `-p deep_causality_physics` clean. `cargo clippy -p deep_causality_topology --all-targets -- -D warnings` clean. `cargo test -p deep_causality_topology` + `-p deep_causality_physics` pass. The medicine demo `tissue_classification` runs to completion (verify by `cargo run --release --example tissue_classification` from the `examples/medicine_examples` directory).
- [ ] 5.13 H4-G2 Review: user reviews the lazy refactor diff before the cross-crate audit closes.

## 6. Block H5 — Caller audit: `deep_causality_physics` and examples

The original H4 caller audit. After the H4 lazy refactor, most physics fixtures pass without source change because they were already non-degenerate by construction. The audit still runs end-to-end to confirm no fixture relied on the now-removed eager Hodge ⋆ path in `triangulate`.

- [ ] 6.1 Audit `tests/kernels/condensed/*` (`create_flat_manifold` used by `test_foppl_von_karman_strain_full` and `test_wrapper_strain_full`).
- [ ] 6.2 Audit `tests/kernels/em/*` (`create_simple_manifold` used by 9 EM kernel tests).
- [ ] 6.3 Audit `tests/kernels/quantum/*` (`create_simple_manifold` used by 5 Klein-Gordon kernel tests).
- [ ] 6.4 Audit `tests/kernels/thermodynamics/*` (`create_temp_manifold` used by 4 heat-diffusion kernel tests).
- [ ] 6.5 Audit `tests/kernels/mhd/*` (`create_dummy_manifold`, `create_test_manifold` used by ideal/resistive GRMHD tests, including the explicit error tests `test_relativistic_current_kernel_low_dim_metric_error` and `test_relativistic_current_kernel_low_skeleton_error`).
- [ ] 6.6 Audit `examples/medicine_examples/tissue_classification/main.rs::analyze_with_monad`. Expected: runs to completion post-H4 lazy refactor with no source change.
- [ ] 6.7 Audit `examples/medicine_examples/aneurysm_risk/main.rs::build_mock_aneurysm`. Expected: runs to completion (unchanged from pre-H4).
- [ ] 6.8 Audit `examples/mathematics_examples/topology/{complex_operators.rs, differential_field.rs}` (2 direct, both call `triangulate` in `main`).
- [ ] 6.9 H5-G1 Compilation + workspace tests: `make build && make test` clean. Both medicine demos run to completion.
- [ ] 6.10 H5-G2 Review: user reviews the cross-crate audit diff and runtime-validates both medicine demos before sign-off.

## 7. Block H6 — Release prep + archive sync

- [ ] 7.1 Update `deep_causality_topology/CHANGELOG.md` with an entry under the next minor version: "Behaviour change: `PointCloud::triangulate` now rejects duplicate-input-point cases with a discriminating `TopologyError::PointCloudError` message. Top-volume degeneracy rejection moves to the Hodge ⋆ access path (`SimplicialComplex::hodge_star_operators`, now fallible). TDA-only consumers unaffected. DEC consumers see a loud `Err` where before they got a silently-singular complex. `HasHodgeStar::hodge_star_matrix` widened to `Result`. New constructor `SimplicialComplex::with_geometry` for the lazy-construction path; existing `SimplicialComplex::new` signature unchanged."
- [ ] 7.2 Update the archived `openspec/changes/archive/2026-05-22-add-hodge-decomposition/design.md` Risk 5 entry: the Vietoris-Rips silent-zero risk is now closed. Cross-reference this change set.
- [ ] 7.3 Update `openspec/changes/add-pointcloud-delaunay-triangulation/design.md` Decision 5 to reflect that the helper signature is `Result<Vec<CsrMatrix<T>>, TopologyError>` per this change set's Decision 2, and that `build_lumped_mass_hodge_star` is now provided by `simplicial_complex/lazy_hodge_star.rs` (not by `op_triangulate.rs` extraction). Remove any lingering "infallible" or "extract from triangulate" language.
- [ ] 7.4 Update `openspec/changes/add-pointcloud-delaunay-triangulation/tasks.md` task 2.1: the helper extraction is already done by this change set's task 5.1; the Delaunay change set's task 2.1 reduces to "import and call the existing `build_lumped_mass_hodge_star` from the lazy-init module via the new `with_geometry` constructor."
- [ ] 7.5 H6-G1 Compilation + workspace tests: `make format && make fix && make build && make test` clean.
- [ ] 7.6 H6-G2 Final review: user reviews the full H0–H6 diff, confirms both medicine demos run, signs off, commits. This is the change-set-closing commit; after H6-G2 the change set is ready to archive.

## 7. Deferred Work (not in scope for this change set)

- Adaptive-precision in-circumcircle predicate (Shewchuk's robust predicates). Mentioned in `design.md` Non-Goals; opens only if a downstream consumer hits floating-point edge behaviour the `T::epsilon() * 100` threshold does not handle.
- Witness-newtype precondition type (`NonDegenerateSimplicialGeometry`). Recorded as deferred alternative in `design.md` Decision 2. Opens only if the helper grows to more than two callers.
- Reworking the lumped-mass Hodge ⋆ scheme itself (barycentric → circumcentric duals). Out of scope; the mass formula stays as-is, only the degeneracy boundary changes.
- Documenting which downstream operators (`Manifold::laplacian`, `Manifold::codifferential`, `Manifold::hodge_decompose`) now have strictly-narrower input domains. The narrowing follows transitively from `triangulate`'s narrowed success domain; explicit per-operator audit is a follow-up if a consumer asks.
