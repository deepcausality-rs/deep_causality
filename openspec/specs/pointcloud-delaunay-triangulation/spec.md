# pointcloud-delaunay-triangulation Specification

## Purpose
TBD - created by archiving change add-pointcloud-delaunay-triangulation. Update Purpose after archive.
## Requirements
### Requirement: PointCloud::triangulate_delaunay method

The crate `deep_causality_topology` SHALL provide a method `PointCloud::triangulate_delaunay(&self) -> Result<SimplicialComplex<T>, TopologyError>` available on every `PointCloud<T, D>` whose `T: RealField + FromPrimitive` and whose ambient dimension `D == 2`. The method produces a 2D Delaunay triangulation: a simplicial complex satisfying the empty-circumcircle property (no input vertex lies in the open circumcircle of any 2-simplex) for any non-degenerate input.

The returned `SimplicialComplex<T>` MUST be drop-in compatible with `Manifold::with_metric`: for any non-degenerate input the resulting complex satisfies the manifold-property check (orientability + link condition + no overlapping interior simplices). Lumped-mass Hodge ⋆ operators are populated by the same construction the existing `PointCloud::triangulate` (Vietoris-Rips) method uses, ensuring downstream operators (`Manifold::exterior_derivative`, `Manifold::codifferential`, `Manifold::laplacian`, `Manifold::hodge_decompose`) work identically on both backends.

The existing `PointCloud::triangulate` (Vietoris-Rips) method is left unchanged.

#### Scenario: Triangulate the canonical two-triangle unit square

- **WHEN** the caller invokes `PointCloud::triangulate_delaunay` on a `PointCloud<f64, 2>` with vertex coordinates `(0,0), (1,0), (1,1), (0,1)`
- **THEN** the returned `SimplicialComplex<f64>` has exactly 4 vertices, 5 edges (4 sides + 1 diagonal), and 2 triangles
- **AND** `Manifold::with_metric` accepts the complex without error

#### Scenario: Triangulate a non-degenerate three-point triangle

- **WHEN** the caller invokes `PointCloud::triangulate_delaunay` on a `PointCloud<f64, 2>` with three non-collinear vertex coordinates
- **THEN** the returned `SimplicialComplex<f64>` has exactly 3 vertices, 3 edges, and 1 triangle

#### Scenario: Triangulate a non-degenerate random planar point set

- **WHEN** the caller invokes `PointCloud::triangulate_delaunay` on a `PointCloud<f64, 2>` with `n >= 4` non-collinear, non-cocircular vertex coordinates
- **THEN** every output 2-simplex satisfies the empty-circumcircle property: no input vertex lies strictly inside its circumcircle

#### Scenario: All output triangles reference only real input vertices

- **WHEN** the caller invokes `PointCloud::triangulate_delaunay` on any valid input of `n` points
- **THEN** every output 2-simplex's three vertex indices are all `< n` (no super-triangle artefact survives in the output)

### Requirement: Delaunay triangulation rejects degenerate inputs explicitly

`PointCloud::triangulate_delaunay` SHALL reject three classes of degenerate input with discriminating `TopologyError::PointCloudError(String)` messages:

1. Ambient dimension other than 2 (`D != 2`).
2. Fewer than 3 input points.
3. All input points collinear (zero bounding-box area to within `T::epsilon() * max_extent²`).

#### Scenario: Reject input with ambient dimension other than 2

- **WHEN** the caller invokes `PointCloud::triangulate_delaunay` on a `PointCloud<f64, 3>` (or any `D != 2`)
- **THEN** the call returns `Err(TopologyError::PointCloudError(msg))` whose `msg` indicates the 2D-only constraint

#### Scenario: Reject input with fewer than three points

- **WHEN** the caller invokes `PointCloud::triangulate_delaunay` on a `PointCloud<f64, 2>` with 1 or 2 vertices
- **THEN** the call returns `Err(TopologyError::PointCloudError(msg))` whose `msg` indicates the minimum-vertex requirement

#### Scenario: Reject collinear input

- **WHEN** the caller invokes `PointCloud::triangulate_delaunay` on a `PointCloud<f64, 2>` whose vertices all lie on a single line (within `T::epsilon()`)
- **THEN** the call returns `Err(TopologyError::PointCloudError(msg))` whose `msg` indicates the non-collinearity requirement

### Requirement: Delaunay triangulation accepts cocircular inputs with deterministic tiebreak

For inputs where four or more points are cocircular (lie on a common circle), the Delaunay triangulation is not unique; multiple valid triangulations differ only in which diagonal is chosen for the cocircular sub-quadrilateral. `PointCloud::triangulate_delaunay` SHALL accept such inputs and return one of the valid triangulations, chosen deterministically by input-point insertion order.

The unit square corners `(0,0), (1,0), (1,1), (0,1)` are the canonical cocircular case (cocircular on the unit circle around `(0.5, 0.5)`). The method MUST succeed on this input and produce a 5-edge, 2-triangle complex.

#### Scenario: Cocircular unit-square corners produce a deterministic two-triangle complex

- **WHEN** the caller invokes `PointCloud::triangulate_delaunay` on a `PointCloud<f64, 2>` with the four unit-square corner coordinates in the order `(0,0), (1,0), (1,1), (0,1)`
- **THEN** the call returns `Ok(complex)` where `complex` has 4 vertices, 5 edges, and 2 triangles
- **AND** repeated invocations on the same input produce identical complexes (deterministic output)

### Requirement: R: RealField precision parameterisation end-to-end

Every new public signature added by this change set MUST be generic over `T: RealField` (with `+ FromPrimitive` declared at the method site where literal construction is required). No `f64` SHALL appear in any new public type, method, or trait signature.

#### Scenario: No new public signature contains f64

- **WHEN** a reviewer searches the new and modified source files for `f64` in public signatures
- **THEN** zero occurrences are found in `pub` items, `pub fn` parameters, `pub fn` return types, or `pub trait` method signatures

#### Scenario: triangulate_delaunay can be instantiated at multiple precision backends

- **WHEN** the caller invokes `PointCloud::<f32, 2>::triangulate_delaunay`, `PointCloud::<f64, 2>::triangulate_delaunay`, and `PointCloud::<Float106, 2>::triangulate_delaunay` on the same logical input
- **THEN** all three calls compile and produce structurally identical complexes (same vertex count, same edge count, same triangle count)

### Requirement: Cross-backend Hodge-decomposition cross-check on the Delaunay unit square

The cross-backend cross-check SHALL use the canonical two-triangle Delaunay unit square on the simplicial side and the 2×2 lattice on the cubical side. The simplicial fixture is constructed via `PointCloud::triangulate_delaunay` on the four unit-square corners. This requirement supersedes the relaxed cross-check that shipped with `add-hodge-decomposition` H3 (single-right-triangle simplicial fixture); the test file `hodge_decomposition_cross_backend_tests.rs` is rewritten to consume the Delaunay fixture instead.

**Resolution of the deferred strict-equality scenario.** The `add-hodge-decomposition` change set deferred a `|‖simplicial.exact()‖ − ‖cubical.exact()‖| < 1e-6` raw-norm equality scenario to this change set. That scenario as stated is unachievable on any coarse discretization with different edge counts on the two sides: the simplicial Delaunay unit square has 5 edges while the cubical 2×2 lattice has 12, so for any non-trivial ω the raw `‖α‖` differs by O(1) between backends. The strict test in this change set is the per-component agreement of *normalized component fractions*, not raw L2 norms. Both backends use the same prescribed ω with the same logical content; the normalized fractions are dimensionless ratios that agree at 1e-6 on coarse discretizations.

A prescribed pure-exact 1-form `ω = df` on the unit square SHALL produce decompositions whose per-component fractional energies agree across the simplicial backend (`ReggeGeometry<f64>` over the two-triangle Delaunay unit square) and the cubical backend (`CubicalReggeGeometry<2, f64, Euclidean>` over the 2×2 lattice unit square) to tolerance `1e-6`, with each vanishing component individually below the noise floor.

#### Scenario: Simplicial and cubical decompositions agree on each component fraction to 1e-6

- **WHEN** the caller decomposes a prescribed pure-exact 1-form `ω = df` once via the simplicial backend on the Delaunay-built two-triangle unit square and once via the cubical backend on the 2×2 lattice unit square
- **THEN** for each component `c ∈ {exact, co_exact, harmonic}` the per-backend fraction `‖c‖² / ‖ω‖²` agrees across backends: `|frac_simplicial(c) − frac_cubical(c)| < 1e-6`

#### Scenario: Per-component vanishing on each backend individually

- **WHEN** the caller decomposes the same prescribed `ω = df` on each backend
- **THEN** each backend individually reports `‖co_exact‖² / ‖ω‖² < 1e-6` and `‖harmonic‖² / ‖ω‖² < 1e-6` (the two vanishing components are at machine noise on each side)

#### Scenario: The relaxed scenarios from add-hodge-decomposition H3 still hold

- **WHEN** the caller runs the existing relaxed scenarios (orthogonality identity per backend; summed-vanishing-ratio cross-backend agreement) on the new Delaunay-built simplicial fixture
- **THEN** both scenarios pass at their original `1e-6` tolerance, providing defence-in-depth alongside the new per-component scenario

