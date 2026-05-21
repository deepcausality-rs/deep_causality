## 1. `deep_causality_num` — verify `FromPrimitive` is the literal-conversion path

- [x] 1.1 Confirm `deep_causality_num::cast::from_primitive::FromPrimitive` is exposed and implemented for `f32`, `f64`, and `Float106`. (Design pivot: no new methods on `RealField`; topology code uses the existing `FromPrimitive` trait for numeric-literal materialization.)
- [x] 1.2 Confirm the convention: every topology site that previously would have called `R::from_f64(literal)` instead calls `<R as FromPrimitive>::from_f64(literal).expect("documented invariant")`.
- [x] 1.3 No `RealField` trait modification. The `RealField` trait surface stays as it was pre-R0.
- [x] 1.4 No new unit tests in `deep_causality_num`. `FromPrimitive`'s existing test coverage is sufficient.
- [x] 1.5 (Removed — was `from_f64` impl for `f64`; obsolete under the pivot.)
- [x] 1.6 `cargo build --workspace` after the pivot confirms the baseline still compiles.

## 1b. `ChainComplex::Metric` becomes a GAT `Metric<R: RealField>`

Architectural prelude to Phase 2. The `ChainComplex` trait's `Metric` associated type becomes a generic associated type so precision can be a parameter of the metric without forcing it onto the combinatorial complex itself.

- [ ] 1b.1 In `deep_causality_topology/src/traits/chain_complex.rs`, change `type Metric;` to `type Metric<R: RealField>;`. Add `use deep_causality_num::RealField;` if not already imported.
- [ ] 1b.2 In `deep_causality_topology/src/types/lattice_complex/mod.rs`, update the `ChainComplex` impl: `type Metric<R: RealField> = CubicalReggeGeometry<D, R>;`. (Phase 2 retypes `CubicalReggeGeometry` to take this second parameter; sequence the changes so the trait impl compiles by the end of Phase 2.)
- [ ] 1b.3 In `deep_causality_topology/src/types/simplicial_complex/...`, update the `ChainComplex` impl: `type Metric<R: RealField> = ReggeGeometry<R>;`. (Phase 3 retypes `ReggeGeometry`; same sequencing note.)
- [ ] 1b.4 In `deep_causality_topology/src/types/cell_complex/...`, update the `ChainComplex` impl: `type Metric<R: RealField> = ();` (no metric available; works for any `R`).
- [ ] 1b.5 Grep `impl .* ChainComplex for` across the crate; update every additional impl found.
- [ ] 1b.6 In `deep_causality_topology/src/types/manifold/mod.rs`, retype `metric: Option<K::Metric>` to `metric: Option<K::Metric<F>>`. Add `F: RealField + FromPrimitive` bound at the struct level or document why per-impl-block bounding is preferred.
- [ ] 1b.7 In `deep_causality_topology/src/types/manifold/getters/mod.rs`, retype `metric() -> Option<&K::Metric>` to `metric() -> Option<&K::Metric<F>>`.
- [ ] 1b.8 In `deep_causality_topology/src/extensions/hkt_manifold/mod.rs`, retype every `K::Metric` use site to `K::Metric<F>` (or whichever precision parameter is in scope). The `K::Metric: Clone` bound at line 241 becomes `K::Metric<F>: Clone` with `F` in scope.
- [ ] 1b.9 In `deep_causality_topology/src/types/manifold/api/constructors.rs`, retype every constructor that takes a `metric: K::Metric` parameter to `metric: K::Metric<F>`.
- [ ] 1b.10 Grep `K::Metric` (with word boundary) across the crate; verify every use site has been retyped to `K::Metric<F>` (or appropriate precision).
- [ ] 1b.11 Build check: `cargo build -p deep_causality_topology` after Phases 2 and 3 are far enough along that `CubicalReggeGeometry<D, R>` and `ReggeGeometry<R>` exist. Fix any remaining GAT-related errors.

## 2. `CubicalReggeGeometry` — parameterize over `R: RealField`

- [ ] 2.1 Change struct definition in `deep_causality_topology/src/types/cubical_regge_geometry/mod.rs:74` to `pub struct CubicalReggeGeometry<const D: usize, R: RealField>`.
- [ ] 2.2 Change private enum `EdgeLengths<const D: usize>` to `EdgeLengths<const D: usize, R: RealField>`. Retype the three non-unit variants: `Uniform { length: R }`, `PerAxis { lengths: [R; D] }`, `PerEdge { lengths: Vec<R> }`.
- [ ] 2.3 Retype constructors (lines 105, 115, 126, 138): `unit()`, `uniform(length: R)`, `per_axis(lengths: [R; D])`, `from_edge_lengths(lengths: Vec<R>)`. The `unit()` constructor must compile at any `R: RealField`.
- [ ] 2.4 Retype the `with_timelike_axes` builder (line 151) — no signature change but propagate the `R` parameter through `Self`.
- [ ] 2.5 Retype accessors: `uniform_length() -> Option<R>` (line 178), `axis_lengths() -> Option<[R; D]>` (line 189), `axis_length(axis) -> Option<R>` (line 200), `edge_length_at(edge_id) -> Option<R>` (line 216), `edge_lengths() -> Option<&[R]>` (line 234).
- [ ] 2.6 Replace the four `1.0` literal returns (lines 180, 191, 205, 218) and `[1.0; D]` array (line 191) with `R::one()` / `[R::one(); D]`.
- [ ] 2.7 Add `R: RealField` test pass: explicit `<f64>` turbofish on every existing test in `tests/types/cubical_regge_geometry/`.
- [ ] 2.8 Add `f32` duplicates for the unit-edge, uniform, per-axis, and per-edge accessor tests with `_f32` suffix.

## 3. `ReggeGeometry` — rename `T` to `R: RealField`, drop `From<f64>` bounds

- [ ] 3.1 In `deep_causality_topology/src/types/regge_geometry/mod.rs`, rename the type parameter `T` to `R` and replace the bound `T: Float + Zero + Copy + PartialOrd + From<f64> + Into<f64>` with `R: RealField`.
- [ ] 3.2 In `regge_geometry/curvature.rs:17`, replace the bound `T: Float + Copy + Into<f64> + From<f64>` with `R: RealField`.
- [ ] 3.3 In `curvature.rs:30-33`, retype `calculate_ricci_curvature` to return `Result<CausalTensor<R>, TopologyError>`.
- [ ] 3.4 In `curvature.rs:133`, retype `compute_dihedral_angle` to return `Result<R, TopologyError>`.
- [ ] 3.5 In `curvature.rs:198, 213`, replace the helper bounds `T: Float + Zero + Copy + PartialOrd + From<f64>` with `R: RealField`.
- [ ] 3.6 In `curvature.rs:326, 346, 407, 416, 434`, retype every internal helper (determinant, area, volume) to return `R` instead of `f64`.
- [ ] 3.7 Rewrite every `<T as From<f64>>::from(literal)` call in `regge_geometry/` to `<R as FromPrimitive>::from_f64(literal).expect("...")` (or a `RealField`-native expression where one exists). Workspace grep `From<f64>` in this directory after the rewrite must return zero hits.
- [ ] 3.8 Update every existing `ReggeGeometry<f64>` test to add explicit `::<f64>` where inference fails.
- [ ] 3.9 Add `f32` duplicates for the dihedral-angle, Ricci-curvature, and determinant tests.

## 4. `CurvatureTensor` — parameterize over `R: RealField`

- [ ] 4.1 In `deep_causality_topology/src/types/curvature_tensor/mod.rs`, change every `T: Field + Copy + Default + PartialOrd + Float + From<f64> + Into<f64>` bound to `R: RealField` (rename `T` → `R`).
- [ ] 4.2 Bounds appear at lines 131, 224, 250 — confirm by re-grep after the edit that no `From<f64>` remains.
- [ ] 4.3 Replace every internal `<T as From<f64>>::from(literal)` call (lines 142, 270, 274, 294, 299, 314, 318, 329, 368, 438, 466) with `<R as FromPrimitive>::from_f64(literal).expect("...")` (or a `RealField`-native expression where one exists).
- [ ] 4.4 Retype every public method's return type from `T` to `R` (mechanical rename).
- [ ] 4.5 Retype existing tests with explicit `::<f64>` and add `f32` duplicates for the flat-tensor, index-raise, Ricci, and Kretschmann tests.

## 5. `Manifold` API — generalize covariance / geometry / differential

- [ ] 5.1 In `deep_causality_topology/src/types/manifold/api/covariance.rs:11`, drop the `D: Into<f64> + Copy` bound and replace with `D: RealField`.
- [ ] 5.2 Retype `covariance_matrix() -> Result<Vec<Vec<D>>, TopologyError>` (line 24) — return the manifold's own field-data type.
- [ ] 5.3 Retype `eigen_covariance() -> Result<Vec<D>, TopologyError>` (line 34).
- [ ] 5.4 If `eigen_covariance` internally calls an `f64`-only eigenvalue solver, document the internal precision floor in the doc comment and convert at the boundary via `<D as FromPrimitive>::from_f64(eigenvalue_as_f64).expect("eigenvalue fits")`.
- [ ] 5.5 In `manifold/api/geometry.rs:35`, retype `simplex_volume_squared(simplex) -> Result<C, TopologyError>`.
- [ ] 5.6 In `manifold/api/geometry.rs:17-18`, drop `C: From<f64> + Into<f64>` from the bound; require `C: RealField`.
- [ ] 5.7 In `manifold/geometry/mod.rs:17`, replace the Cayley-Menger bound with `C: RealField`.
- [ ] 5.8 In `manifold/covariance/mod.rs:17`, retype the private `covariance_matrix_impl` return to `Result<Vec<Vec<D>>, _>` with the same generalization.
- [ ] 5.9 In `manifold/differential/laplacian.rs:23` and `manifold/differential/codifferential.rs:22`, drop the `From<f64>` bound and replace internal `1e-12` tolerance constants with `R::epsilon()` or `<R as FromPrimitive>::from_f64(1e-12).expect("epsilon fits")`.
- [ ] 5.10 Update existing manifold tests with explicit `::<f64>` and add `f32` duplicates for covariance, eigen, simplex-volume, and laplacian tests.

## 6. `DifferentialForm::scale` — replace `Mul<f64>` with `Mul<R>`

- [ ] 6.1 In `deep_causality_topology/src/types/differential_form/mod.rs:281`, change the impl block from `impl<T: Clone + Default + std::ops::Mul<f64, Output = T>>` to `impl<T, R> DifferentialForm<T> where T: Clone + Default + Mul<R, Output = T>, R: RealField`.
- [ ] 6.2 Retype the method to `pub fn scale(&self, scalar: R) -> Self`.
- [ ] 6.3 In `differential_form/mod.rs:114`, drop the `T: From<f64>` bound on the `zero` constructor; replace with a `T: Default + Zero` shape (or use `Default::default()` directly, which is already the impl content).
- [ ] 6.4 Update existing differential-form tests with explicit `::<f64>` where inference fails.
- [ ] 6.5 Add `f32` duplicates for the scale and zero-construction tests.

## 7. `PointCloud::triangulate` — drop `From<f64>` bounds

- [ ] 7.1 In `deep_causality_topology/src/types/point_cloud/ops/op_triangulate.rs:28`, replace the bound `T: Float + Sum + From<f64>` with `T: RealField + Sum`.
- [ ] 7.2 At lines 90 and 113, repeat the bound replacement.
- [ ] 7.3 Replace every internal `<T as From<f64>>::from(literal)` call (lines 35, 84, 95, 263, 270, 271) with `<T as FromPrimitive>::from_f64(literal).expect("...")` (or a `RealField`-native expression where one exists).
- [ ] 7.4 Update existing triangulate tests with explicit `::<f64>` and add `f32` duplicates for the Gaussian elimination and Hodge dual tests.

## 8. `GaugeGroup` trait — generalize `structure_constant` over `R: RealField`

- [ ] 8.1 In `deep_causality_topology/src/traits/gauge_group.rs:107`, change the trait method signature to `fn structure_constant<R: RealField>(a: usize, b: usize, c: usize) -> R;` with no default implementation.
- [ ] 8.2 In `gauge_groups/su2.rs:40`, implement the method via `<R as FromPrimitive>::from_f64(literal).expect("structure constant fits")` for each hardcoded coefficient.
- [ ] 8.3 In `gauge_groups/se3.rs:55, 57`, same retype.
- [ ] 8.4 In `gauge_groups/so3_1.rs:74, 76`, same retype.
- [ ] 8.5 If a `SU3` impl exists (search), retype the same way.
- [ ] 8.6 Update every internal call site that uses `structure_constant` to either inherit `R` from its generic context or supply `::<R>` explicitly.
- [ ] 8.7 Add a property test per gauge group asserting `structure_constant::<f64>(...)` and `structure_constant::<f32>(...)` produce the expected canonical values bit-identically.

## 9. Metropolis acceptance ratio — generalize to `R`

- [ ] 9.1 In `deep_causality_topology/src/types/gauge/gauge_field_lattice/ops_metropolis.rs:149`, retype `metropolis_step` to return `Result<R, TopologyError>` where `R` is the gauge field's real-scalar parameter.
- [ ] 9.2 At line 100, keep the RNG sample as `let rnd: f64 = rng.random();` (documented exception); add `let rnd: R = <R as FromPrimitive>::from_f64(rnd).expect("f64 RNG sample fits in any RealField");` immediately after.
- [ ] 9.3 Tag the `f64` line as `// PERMITTED-F64: RNG boundary; see design.md Decision 7`.
- [ ] 9.4 Update existing Metropolis tests with explicit `::<f64>` and add an `f32` duplicate test.

## 10. Test utilities — generalize over `R: RealField`

- [ ] 10.1 In `deep_causality_topology/src/utils_tests/mod.rs:16`, retype `create_triangle_complex` to `pub fn create_triangle_complex<R: RealField>() -> SimplicialComplex<R>`.
- [ ] 10.2 At line 82, retype `create_line_complex` the same way.
- [ ] 10.3 Update every existing test call site that uses these helpers to `::<f64>()`.
- [ ] 10.4 Confirm by re-grep that no `SimplicialComplex<f64>` hardcoded type appears in `utils_tests/`.

## 11. Internal cleanup — remove all surviving `f64` references in `deep_causality_topology/src/`

- [ ] 11.1 Run `grep -rn -E '\bf64\b' deep_causality_topology/src/ --include='*.rs'`. Every hit must fall into one of: (a) the documented Metropolis RNG-boundary line, (b) `<R as FromPrimitive>::from_f64(...).expect(...)` calls, (c) doc-comments / module-level `//!` strings, (d) test code that explicitly tests `f64`-precision behavior. No other hits are permitted.
- [ ] 11.2 Run `grep -rn 'From<f64>' deep_causality_topology/src/`. Zero hits required.
- [ ] 11.3 Run `grep -rn 'Into<f64>' deep_causality_topology/src/`. Zero hits required.
- [ ] 11.4 Run `grep -rn 'Mul<f64' deep_causality_topology/src/`. Zero hits required.
- [ ] 11.5 Run `grep -rn 'as f64' deep_causality_topology/src/`. Allowed only inside the Metropolis RNG-boundary line (one hit); confirm.
- [ ] 11.6 Run `grep -rn '.into()' deep_causality_topology/src/` and audit every hit for hidden `f64` round-trips; eliminate the genuine `f64` round-trips.

## 12. Downstream library temporary pins — keep workspace compiling

- [ ] 12.1 Run `cargo build --workspace`. The compile errors enumerate every `deep_causality_physics` and `deep_causality_effects` call site that needs a temporary pin.
- [ ] 12.2 For each error in `deep_causality_physics/`, add `::<f64>` at the construction site and tag the line `// TEMP: removed by generalize-physics-over-realfield`.
- [ ] 12.3 For each error in `deep_causality_effects/`, add `::<f64>` at the construction site and tag the line `// TEMP: removed by generalize-effects-over-realfield`.
- [ ] 12.4 Confirm `cargo build --workspace` succeeds.
- [ ] 12.5 Confirm `cargo test --workspace` succeeds (every existing test passes; bit-identical at `R = f64`).

## 13. Verification

- [ ] 13.1 `cargo build -p deep_causality_topology` and `cargo build -p deep_causality_num` succeed.
- [ ] 13.2 `cargo test -p deep_causality_topology` passes — every existing test plus every new `_f32` duplicate.
- [ ] 13.3 `cargo clippy -p deep_causality_topology -- -D warnings` is clean (no lint suppressions — rewrite any flagged code).
- [ ] 13.4 `cargo fmt --check` is clean.
- [ ] 13.5 `bazel test //deep_causality_topology/... //deep_causality_num/...` passes (confirms `BUILD.bazel` registration of any new test files).
- [ ] 13.6 Run the existing `deep_causality_topology` benchmark suite at `R = f64` and confirm no regression >2% vs. pre-R0 baseline.
- [ ] 13.7 Run `make build` and `make test` to verify the full workspace.
- [ ] 13.8 Run the four invariant greps from task 11 and verify zero hits where expected.

## 14. Downstream-proposal editorial pass

- [ ] 14.1 Update `openspec/changes/add-cubical-regge-calculus-core/proposal.md` to declare the surface generic over `R: RealField` throughout.
- [ ] 14.2 Update `openspec/changes/add-cubical-regge-calculus-core/design.md` accordingly; resolve the open question on `regge_action` return type by adopting `ReggeActionResult<R: RealField> { value: R, hinges_evaluated: usize, max_deficit: R }`.
- [ ] 14.3 Update `openspec/changes/add-cubical-regge-calculus-core/specs/cubical-regge-calculus-core/spec.md` to retype every scenario's expected return / parameter to `R`.
- [ ] 14.4 Update `openspec/changes/add-cubical-regge-calculus-core/tasks.md` to thread the `R` parameter through every implementation task.
- [ ] 14.5 Update `openspec/changes/add-cubical-regge-calculus-analytical/proposal.md` to remove the `Complex<f64>` shim discussion (Decision 8 in its design.md).
- [ ] 14.6 Update `openspec/changes/add-cubical-regge-calculus-analytical/design.md` Decision 8 to "Lorentzian Regge action returns `C: ComplexField<R>` from `deep_causality_num`; no shim needed".
- [ ] 14.7 Re-grep the four `f64` invariant patterns (task 11) inside the two cubical-Regge proposal directories to confirm zero hits.

## 15. Commit prep

- [ ] 15.1 Stage the changes per AGENTS.md (Golden Rule 1 — agent does not commit; user commits).
- [ ] 15.2 Draft a commit message summarizing R0's scope, the breaking changes, the two temporary pin tags for follow-up change sets, and the downstream-proposal editorial pass.
- [ ] 15.3 Leave the commit for the user to inspect and run.
