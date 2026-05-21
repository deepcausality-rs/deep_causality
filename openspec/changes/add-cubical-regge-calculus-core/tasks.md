## 1. Scaffolding and shared helpers

- [ ] 1.1 Add `src/types/cubical_regge_geometry/volumes.rs` and `curvature.rs` as new submodules; register both in `src/types/cubical_regge_geometry/mod.rs`
- [ ] 1.2 Add private associated function `LatticeComplex<D>::edge_index(position, axis) -> usize` mapping a `(position, axis)` pair to a flat `Vec<f64>` index for `PerEdge` metrics; cover with unit tests for open and periodic boundaries
- [ ] 1.3 Add `tests/types/cubical_regge_geometry/{mod.rs, volumes_tests.rs, curvature_tests.rs}` skeletons; register in `tests/types/cubical_regge_geometry/mod.rs` and `tests/types/mod.rs`
- [ ] 1.4 Update `deep_causality_topology/tests/BUILD.bazel` to include the new test folder module
- [ ] 1.5 Add any shared test fixtures (small lattices with known geometry) under `src/utils_tests/cubical_regge_fixtures.rs`

## 2. Phase R1 — Cell volumes

- [ ] 2.1 Implement `CubicalReggeGeometry<D>::cell_volume(&self, complex, cell_id, grade) -> f64` in `volumes.rs`, dispatching on the edge-length variant (`UnitEdge`, `Uniform`, `PerAxis`, `PerEdge`)
- [ ] 2.2 Implement `CubicalReggeGeometry<D>::top_cell_volume(&self, complex, cell_id) -> f64` as the `grade = D` convenience
- [ ] 2.3 Add `debug_assert!` in the `PerEdge` path documenting the axis-aligned-cell assumption (cross-terms vanish)
- [ ] 2.4 Write unit-edge property tests: every k-cube has volume `1.0` exactly for every grade
- [ ] 2.5 Write per-axis property tests: top-cube volume equals `axis_lengths.iter().product()`; k-cell volume equals product of active-axis lengths
- [ ] 2.6 Write per-edge property test: a `PerEdge` metric with uniform-per-axis lengths reproduces the per-axis result to `1e-12`

## 3. Phase R2 — Hinge enumeration + dihedral angles

- [ ] 3.1 Implement `LatticeComplex<D>::hinge_top_cube_neighbors(&self, hinge_id) -> impl Iterator<Item = CellId>` by walking the cached `coboundary_matrix(D-2)` and `coboundary_matrix(D-1)`; deduplicate the result
- [ ] 3.2 Add unit tests for hinge enumeration: 2D interior vertex → 4 squares; 4D interior 2-cell → 4 4-cubes; periodic-boundary wrap-around; determinism / no duplicates
- [ ] 3.3 Implement `CubicalReggeGeometry<D>::dihedral_angle(&self, complex, top_cube_id, hinge_id) -> f64` in `curvature.rs`, dispatching on the edge-length variant
- [ ] 3.4 Short-circuit `UnitEdge` and `Uniform` to `FRAC_PI_2` without touching the lattice
- [ ] 3.5 Implement per-axis case using `arctan2(lengths[j], lengths[i])` where `i, j` are the two axes inactive in the hinge but active in the top cube
- [ ] 3.6 Implement per-edge case: read the two edge lengths of the top cube at the hinge along axes `i` and `j` (via `edge_index`) and apply the same `arctan2` formula
- [ ] 3.7 Document the contract for non-incident `(top_cube_id, hinge_id)` pairs in the doc comment (return `NaN`, `Err`, or panic — choose one and document it)
- [ ] 3.8 Write unit-edge property test: every dihedral angle equals `FRAC_PI_2` to `1e-15`
- [ ] 3.9 Write per-axis property test: dihedral angles around any interior vertex on a 2D lattice sum to `2π` to `1e-12`
- [ ] 3.10 Write per-edge / per-axis agreement test: matching uniform-per-axis edge lengths produce identical dihedral angles to `1e-12`

## 4. Phase R3 — Deficit angles + Regge action

- [ ] 4.1 Implement `CubicalReggeGeometry<D>::deficit_angle(&self, complex, hinge_id) -> f64` in `curvature.rs` as `2π − Σ dihedral_angle(c, h)` summed over `hinge_top_cube_neighbors(hinge_id)`
- [ ] 4.2 Short-circuit `UnitEdge` (and `Uniform`, and `PerAxis` with all-equal lengths) at the entry of `deficit_angle` to return exact `0.0`, avoiding floating-point noise
- [ ] 4.3 Implement `CubicalReggeGeometry<D>::regge_action(&self, complex) -> f64` as `Σ_h cell_volume(h, D-2) · deficit_angle(h)` over every (D−2)-hinge
- [ ] 4.4 Apply the same flat-case short-circuit to `regge_action`, returning exact `0.0`
- [ ] 4.5 Document in the doc comment that `timelike_axes` is ignored in this change set; the Lorentzian variant is deferred to a follow-up change
- [ ] 4.6 Write property test: unit-edge open lattice → every deficit angle is exactly `0.0`
- [ ] 4.7 Write property test: unit-edge periodic lattice → every deficit angle is exactly `0.0`; total `regge_action` is exactly `0.0`
- [ ] 4.8 Write property test: single-edge perturbation has bounded support — hinges not in any incident top cube of the perturbed edge still report `deficit_angle == 0.0`
- [ ] 4.9 Write property test: periodic vs. open boundary action difference equals the boundary-hinge contribution to within `1e-10`
- [ ] 4.10 Write property test: `regge_action` on a metric with `timelike_axes = Some(...)` equals the same metric with `timelike_axes = None` (Lorentzian path is deferred)

## 5. Public API export and documentation

- [ ] 5.1 Re-export new public methods from `deep_causality_topology/src/lib.rs` as appropriate (per the project's "all public items exported from lib.rs" convention)
- [ ] 5.2 Verify no new public traits were added; the crate's `pub trait` set MUST match pre-change (assertion: visual diff of `src/lib.rs` and `src/traits/`)
- [ ] 5.3 Write doc comments on every new public method, including the assumption (axis-aligned cubical cells) and the deferred-work pointers (Lorentzian variant, gradient, Metropolis updates)
- [ ] 5.4 Add a short module-level doc comment to `src/types/cubical_regge_geometry/mod.rs` summarizing what R1–R3 cover and pointing at `openspec/notes/CubicalReggeCalculus.md` for the full roadmap

## 6. Verification

- [ ] 6.1 `cargo build -p deep_causality_topology` succeeds
- [ ] 6.2 `cargo test -p deep_causality_topology` passes, including all new tests
- [ ] 6.3 `cargo clippy -p deep_causality_topology -- -D warnings` is clean (no lint suppressions — fix any issues by rewriting code)
- [ ] 6.4 `cargo fmt --check` is clean
- [ ] 6.5 `bazel test //deep_causality_topology/...` passes (confirms `BUILD.bazel` registration)
- [ ] 6.6 Spot-check that downstream crates (`deep_causality`, `deep_causality_physics`, any example crates depending on `deep_causality_topology`) still compile unchanged: `make build`
- [ ] 6.7 Prepare a commit message summarizing R1–R3 contents; do not commit — leave the commit for the user per AGENTS.md Golden Rule 1
