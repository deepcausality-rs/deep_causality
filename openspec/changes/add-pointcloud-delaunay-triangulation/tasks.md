## 1. Block D0 — Preflight audit

- [ ] 1.1 Verify `add-hodge-decomposition` (H1–H3 + triangulate ambient-dim cap fix) has shipped and is archived. If not, this change set cannot open.
- [ ] 1.2 Audit the existing `PointCloud::triangulate` for the lumped-mass Hodge ⋆ computation that needs to be extracted into a shared helper. Confirm the extracted helper's signature and visibility (private to the `point_cloud` module hierarchy, not part of the crate's public surface).
- [ ] 1.3 Decide whether the cocircular tolerance is hard-coded or exposed as a `triangulate_delaunay_opts` parameter. Document the decision in `design.md` Open Question 1.
- [ ] 1.4 D0-G3 Review: user signs off on the three preflight resolutions before any code lands.

## 2. Block D1 — Bowyer-Watson core + degeneracy guards

- [x] 2.1 ~~Extract the existing Vietoris-Rips lumped-mass Hodge ⋆ construction from `op_triangulate.rs` into a private helper~~ **Done by `harden-simplicial-hodge-degeneracy-detection` task 5.1.** The helper lives at `deep_causality_topology/src/types/simplicial_complex/lazy_hodge_star.rs` as `pub(crate) fn build_lumped_mass_hodge_star<T>(skeletons: &[Skeleton], coords: &[T], dim: usize) -> Result<Vec<CsrMatrix<T>>, TopologyError>` with `T: RealField + FromPrimitive`. Importing it from `op_triangulate_delaunay.rs` is just a `use` statement; the actual call happens lazily via `SimplicialComplex::with_geometry(...)` rather than inline.
- [ ] 2.2 Add the `triangulate_delaunay` method skeleton in a new file `deep_causality_topology/src/types/point_cloud/ops/op_triangulate_delaunay.rs`, gated by the three degeneracy checks per `design.md` Decision 4: `D == 2`, `num_points >= 3`, non-collinear. Each check returns a discriminating `TopologyError::PointCloudError(String)`.
- [ ] 2.3 Implement the super-triangle construction per `design.md` Decision 2 (100x bounding-box expansion). The super-triangle vertices live at virtual indices `num_input_points`, `num_input_points + 1`, `num_input_points + 2`.
- [ ] 2.4 Implement the in-circumcircle predicate as a private helper, taking three triangle-vertex coords and one test-point coord, returning a tri-state `Inside | Outside | OnCircle` enum. The `OnCircle` arm is decided per `design.md` Decision 3 with tolerance `T::epsilon() * 100`.
- [ ] 2.5 Implement the Bowyer-Watson insertion loop: for each input point, identify "bad" triangles (in-circumcircle returns `Inside`), extract the polygonal cavity boundary by collecting edges that appear in exactly one bad triangle, and re-triangulate by fanning from the new point.
- [ ] 2.6 Implement super-triangle vertex removal: drop any output triangle that references a virtual vertex index `>= num_input_points`. Assert the invariant that every surviving triangle's three vertex indices are all `< num_input_points`.
- [ ] 2.7 Build the `SimplicialComplex<T>` from the final triangle list: 0-skeleton is the input vertices, 1-skeleton is the unique edges of the surviving triangles, 2-skeleton is the surviving triangles. Compute boundary and coboundary operators following the same convention as the existing `triangulate`.
- [ ] 2.8 ~~Call `build_lumped_mass_hodge_ops` to populate the Hodge ⋆ operators.~~ **Superseded.** Use `SimplicialComplex::with_geometry(skeletons, boundary, coboundary, coords, dim)` to construct the complex; the Hodge ⋆ vector is populated lazily on first access via `complex.hodge_star_operators()`. No explicit population call at construction.
- [ ] 2.9 Register the new module in `src/types/point_cloud/ops/mod.rs`. No changes to `src/lib.rs` — `triangulate_delaunay` is a method on `PointCloud`, automatically reachable through the existing re-export.
- [ ] 2.10 Create `tests/types/point_cloud/op_triangulate_delaunay_tests.rs`. Register in `tests/types/point_cloud/mod.rs` and `tests/BUILD.bazel`.
- [ ] 2.11 Unit tests covering: degenerate-input rejection (`D != 2`, `n < 3`, collinear), three-point triangle (Delaunay-trivial), four-point cocircular unit square (verify it produces exactly 5 edges + 2 triangles), 5+ random non-degenerate input fuzz test (verify all output triangles satisfy the empty-circumcircle property).
- [ ] 2.12 Unit tests covering the manifold-property check: every output complex of `triangulate_delaunay` succeeds with `Manifold::with_metric` for any non-degenerate input. This is the load-bearing invariant.
- [ ] 2.13 D1-G1 Compilation: `cargo build -p deep_causality_topology` clean (release + debug); `cargo clippy -p deep_causality_topology --all-targets -- -D warnings` clean. Fix lints at root cause; no `#[allow(clippy::...)]` suppressions per `feedback_clippy_lints`.
- [ ] 2.14 D1-G2 Coverage: 100% on every new file in `src/types/point_cloud/ops/`; 100% on `src/types/point_cloud/ops/op_triangulate.rs` (where the Hodge ⋆ extraction landed).
- [ ] 2.15 D1-G3 Review: user reviews the diff, runs `make format && make fix`, signs off, commits.

## 3. Block D2 — Tighten the cross-backend Hodge decomposition test

- [ ] 3.1 Rewrite the simplicial fixture in `deep_causality_topology/tests/types/manifold/hodge_decomposition_cross_backend_tests.rs` to build the canonical two-triangle unit square via `PointCloud::triangulate_delaunay` on the four corner points `(0,0), (1,0), (1,1), (0,1)`. Edge lengths read from the coordinates; `ReggeGeometry<f64>` constructed with the resulting `n_edges` length.
- [ ] 3.2 Replace the current cross-backend assertions (orthogonality identity + vanishing-component-ratio agreement) with the strict per-component L2 norm equality scenario from `add-hodge-decomposition/specs/hodge-decomposition/spec.md` 4.4: `|‖simplicial.exact()‖ − ‖cubical.exact()‖| < 1e-6` and likewise for `co_exact` and `harmonic`.
- [ ] 3.3 Keep the existing relaxed assertions as additional invariants — they still hold and provide defence-in-depth against regression.
- [ ] 3.4 Update the test file header comment to remove the "future-tightening" note that was added when the H3 relaxed variant landed.
- [ ] 3.5 Update `add-hodge-decomposition/specs/hodge-decomposition/spec.md` to remove the relaxed-variant language and restore the strict per-component L2 norm scenario.
- [ ] 3.6 Update `add-hodge-decomposition/tasks.md` Section 6 to mark the strict L2 follow-up as completed (or remove the section entirely if `openspec` workflow prefers).
- [ ] 3.7 Update `add-hodge-decomposition/design.md` Risk 5 to note that the Vietoris-Rips → non-manifold gap has been closed by this change set.
- [ ] 3.8 D2-G1 Compilation: clean across `deep_causality_topology`. Full workspace `make build && make test` since the cross-backend test touches infrastructure with downstream consumers in `deep_causality_physics` and the medicine examples.
- [ ] 3.9 D2-G2 Coverage: 100% on the modified test file.
- [ ] 3.10 D2-G3 Review: user reviews the full diff across D1+D2, signs off, commits. This is the change-set-closing commit; after D2-G3 the change set is ready to archive.

## 4. Deferred Work (not in scope for this change set)

The following are deliberately out of scope per `design.md` Non-Goals and may become follow-up change sets when a downstream consumer needs them:

- 3D Delaunay tetrahedralisation (`triangulate_delaunay` for `D = 3`).
- Constrained-Delaunay with caller-supplied boundary edges (Chew, Ruppert).
- Adaptive-precision in-circumcircle predicate (Shewchuk's robust predicates).
- Voronoi diagram output (the dual of Delaunay).
- `triangulate_delaunay_random_insertion` for `O(n log n)` average behaviour on adversarial inputs.
- Alpha-shapes, weighted Delaunay (power diagrams), GPU paths.

None of these block any downstream consumer; each is opened on demand.
