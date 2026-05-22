## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Two-backend cross-check on the unit square

The strict per-component L2 norm equality scenario for the cross-backend cross-check, deferred at the close of `add-hodge-decomposition` H3, is now achievable and SHALL be the primary assertion of the cross-backend test. The relaxed orthogonality + vanishing-component-ratio scenarios are retained as additional defence-in-depth invariants.

A prescribed 1-form field on the unit square SHALL produce decompositions whose per-component L2 norms agree across the simplicial backend (`ReggeGeometry<f64>` over the two-triangle Delaunay unit square) and the cubical backend (`CubicalReggeGeometry<2, f64, Euclidean>` over the unit-square lattice) to tolerance `1e-6`.

#### Scenario: Simplicial and cubical decompositions agree on each component L2 norm to 1e-6

- **WHEN** the caller decomposes a prescribed pure-exact 1-form `ω = df` once via the simplicial backend on the Delaunay-built two-triangle unit square and once via the cubical backend on the lattice unit square
- **THEN** `|‖simplicial.exact()‖ − ‖cubical.exact()‖| < 1e-6`, and likewise for `co_exact` and `harmonic`
