## 1. Scaffolding and shared helpers

- [ ] 1.1 Convert `LatticeComplex<D, R>::coboundary_cache` from `Mutex<HashMap<usize, CsrMatrix<i8>>>` to `Box<[OnceLock<CsrMatrix<i8>>]>` of length `D + 1` (one slot per grade `0..=D`). Initialize in `new()` via `(0..=D).map(|_| OnceLock::new()).collect::<Vec<_>>().into_boxed_slice()`. Update `coboundary_matrix(k)` to use `self.coboundary_cache[k].get_or_init(...)` and return `Cow::Borrowed`. Update `Clone` impl to construct fresh empty `OnceLock`s. Drop the `Mutex` import.
- [ ] 1.2 Add `src/types/cubical_regge_geometry/volumes.rs` and `curvature.rs` as new submodules; register both in `src/types/cubical_regge_geometry/mod.rs`
- [ ] 1.3 Add `tests/types/cubical_regge_geometry/{volumes_tests.rs, curvature_tests.rs}` skeletons (SPDX-header only; populated in R1–R3); register in `tests/types/cubical_regge_geometry/mod.rs`
- [ ] 1.4 Update `deep_causality_topology/tests/BUILD.bazel` to add a `types/cubical_regge_geometry` `rust_test_suite` entry
- [ ] 1.5 *Deferred:* `LatticeComplex::edge_index` and `src/utils_tests/cubical_regge_fixtures.rs` are introduced in §2 (R1) where they have a real consumer (`cell_volume`'s `PerEdge` arm). Introducing them in §1 generates `dead_code` warnings on `edge_index`, `edges_along`, `valid_positions` until R1 lands, and the project policy forbids `#[allow(dead_code)]` suppression.

## 2. Phase R1 — Cell volumes

- [ ] 2.1 Add `pub(crate) fn LatticeComplex<D, R>::edge_index(position: [usize; D], axis: usize) -> usize` mapping a `(position, axis)` pair to a flat `Vec<R>` index in the canonical `iter_cells(1)` ordering, with `pub(crate)` helpers `edges_along(axis)` and `valid_positions(d, orientation)`. Cover with an inline `#[cfg(test)] mod edge_index_tests` checking open/periodic/mixed-boundary lattices and end-to-end agreement with `iter_cells(1)` enumeration.
- [ ] 2.2 Implement `CubicalReggeGeometry<D, R>::cell_volume(&self, complex: &LatticeComplex<D, R>, cell: &LatticeCell<D>) -> R` in `volumes.rs`, dispatching on the edge-length variant (`UnitEdge`, `Uniform`, `PerAxis`, `PerEdge`). Grade is derived from `cell.cell_dim()` — dropping the redundant `grade: usize` argument from the original design-note signature.
- [ ] 2.3 Implement `CubicalReggeGeometry<D, R>::top_cell_volume(&self, complex, cell) -> R` as the `cell.cell_dim() == D` convenience (debug-asserts the dimension)
- [ ] 2.4 Add `debug_assert!` in the `PerEdge` path documenting the axis-aligned-cell assumption (cross-terms vanish)
- [ ] 2.5 Write unit-edge property tests: every k-cube has volume exactly `R::one()` for every grade (instantiate over `f64` and `f32`)
- [ ] 2.6 Write per-axis property tests: top-cube volume equals the fold-product of `axis_lengths` over `R`; k-cell volume equals product of active-axis lengths
- [ ] 2.7 Write per-edge property test: a `PerEdge` metric with uniform-per-axis lengths reproduces the per-axis result to within `R::epsilon() * <small constant>`
- [ ] 2.8 Add `src/utils_tests/cubical_regge_fixtures.rs` (and register in `src/utils_tests/mod.rs`) with shared fixtures for the R1–R3 tests (small open/periodic lattices with known geometry).

## 3. Phase R2 — Hinge enumeration + dihedral angles

- [ ] 3.1 Implement `LatticeComplex<D, R>::hinge_top_cube_neighbors(&self, hinge_id) -> impl Iterator<Item = CellId>` by walking the cached `coboundary_matrix(D-2)` and `coboundary_matrix(D-1)`; deduplicate the result. Borrows the cached matrices directly (cache returns `Cow::Borrowed` after the §1.1 conversion).
- [ ] 3.2 Add unit tests for hinge enumeration: 2D interior vertex → 4 squares; 3D interior edge → 4 cubes; 4D interior 2-cell → 4 4-cubes; periodic-boundary wrap-around; open-boundary corner / face hinges produce the correct reduced counts (1 / 2); determinism / no duplicates
- [ ] 3.3 Implement `CubicalReggeGeometry<D, R>::dihedral_angle(&self, complex, top_cube: &LatticeCell<D>, hinge: &LatticeCell<D>) -> R` in `curvature.rs`. Returns `R::pi() / (R::one() + R::one())` uniformly — the dihedral on an axis-aligned cubical complex is π/2 regardless of edge-length variant (see design.md Decision 4 correction).
- [ ] 3.4 Add debug-assertions that `top_cube.cell_dim() == D` and `hinge.cell_dim() == D - 2`. The function does not validate incidence — callers are responsible for enumerating only incident pairs.
- [ ] 3.5 *Superseded by 3.3.* The arctan2 per-axis formula in the original design note was geometrically incorrect; no per-axis-specific code path is needed.
- [ ] 3.6 *Superseded by 3.3.* No per-edge-specific dihedral code path under the axis-aligned assumption.
- [ ] 3.7 Document the non-incident-pair contract in the doc comment: misuse degrades to the geometric constant `π/2` rather than NaN or panic in release; debug builds catch grade mismatches via assertions.
- [ ] 3.8 Write unit-edge property test: every dihedral angle equals `R::pi() / (R::one() + R::one())` to `R::epsilon()` tolerance
- [ ] 3.9 Write per-axis property test: dihedral angles around any interior vertex on a periodic 2D lattice sum to `R::pi() + R::pi()` (i.e. 2π) to a few `R::epsilon()`
- [ ] 3.10 Write per-edge / per-axis agreement test: matching uniform-per-axis edge lengths produce identical dihedral angles to within a few `R::epsilon()` (trivially true now — both return π/2 — but kept as a regression guard against any future variant-specific code path)

## 4. Phase R3 — Deficit angles + Regge action

- [ ] 4.1 Implement `CubicalReggeGeometry<D, R>::deficit_angle(&self, complex, hinge_id) -> R` in `curvature.rs` as `(R::pi() + R::pi()) − Σ dihedral_angle(c, h)` summed over `hinge_top_cube_neighbors(hinge_id)`. Per the R2 correction, dihedral is the constant `R::pi() / (R::one() + R::one())`, so this reduces to `(4 − n) · π/2` where `n` is the incident-top-cube count.
- [ ] 4.2 Short-circuit `n == 4` (full incidence) at the entry of `deficit_angle` to return exact `R::zero()`, avoiding floating-point noise from `2π − 2π`. This replaces the original "variant-based" short-circuit, which the R2 correction makes redundant — deficit no longer depends on edge-length variant, only on hinge incidence count.
- [ ] 4.3 Implement `CubicalReggeGeometry<D, R>::regge_action(&self, complex) -> R` as `Σ_h cell_volume(h) · deficit_angle(h)` over every (D−2)-hinge enumerated by `iter_cells(D − 2)`, accumulated from `R::zero()`. Returns `R::zero()` for `D < 2`.
- [ ] 4.4 Skip the volume×deficit multiplication when `deficit_angle` returns `R::zero()` (saves a `cell_volume` call per interior hinge — a meaningful win on large periodic lattices where most hinges are interior).
- [ ] 4.5 Document in the doc comment that `timelike_axes` is ignored in this change set; the Lorentzian variant is deferred to a follow-up change
- [ ] 4.6 Write property test: unit-edge open lattice → interior hinges (with 4 incident cubes) have deficit exactly `R::zero()`; boundary hinges have deficit equal to `(4 − n) · π/2` where `n` is the incidence count (`1` at a 2D corner, `2` on a 2D edge, etc.). The original task expecting "every deficit is 0 on an open lattice" was based on the incorrect arctan2 dihedral formula; open boundaries carry real intrinsic curvature that the corrected geometry preserves.
- [ ] 4.7 Write property test: unit-edge periodic lattice → every deficit angle is exactly `R::zero()`; total `regge_action` is exactly `R::zero()`
- [ ] 4.8 Write property test: edge-length perturbation does NOT change any deficit angle (under the axis-aligned cubical assumption, deficit depends only on the lattice's topology, not on its metric). Verifies the R2 geometric simplification holds end-to-end.
- [ ] 4.9 Write property test: on a 2D unit-edge open 3×3 lattice the regge_action equals the explicit closed-form sum `4 · 1 · 3π/2 + 4 · 1 · π + 1 · 1 · 0 = 10π` (4 corners with deficit 3π/2, 4 edge-vertices with deficit π, 1 interior vertex with deficit 0). On a 2D periodic 3×3 lattice it equals exactly 0. Difference = 10π = boundary-hinge contribution.
- [ ] 4.10 Write property test: `regge_action` on a metric with `timelike_axes = Some(...)` equals the same metric with `timelike_axes = None` (Lorentzian path is deferred)
- [ ] 4.11 Write property test: on a 3D `PerAxis` lattice with stretched axis lengths, regge_action picks up the per-axis edge volumes on boundary hinges — verifying the volume factor flows through to the action (the only edge-length sensitivity the axis-aligned R3 implementation has).

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
