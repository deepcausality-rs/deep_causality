# simplicial-hodge-degeneracy-detection Specification

## Purpose
TBD - created by archiving change harden-simplicial-hodge-degeneracy-detection. Update Purpose after archive.
## Requirements
### Requirement: PointCloud::triangulate rejects duplicate input points

The crate `deep_causality_topology` SHALL detect duplicate input points in `PointCloud::triangulate` and return a discriminating `TopologyError::PointCloudError(String)` rather than silently producing a simplicial complex with a zero-length 1-simplex and a singular edge mass matrix.

A duplicate is defined as a pair of input points whose Euclidean distance falls below `T::epsilon() * max_extent`, where `max_extent` is the maximum axis-aligned bounding-box extent of the input. The threshold is scale-invariant.

The error message MUST contain the literal substring `"duplicate point"` and the two offending point indices.

#### Scenario: Two identical points are rejected

- **WHEN** the caller invokes `PointCloud::triangulate` on a `PointCloud<f64, 2>` whose vertex coordinates contain two identical points at indices `i` and `j`
- **THEN** the call returns `Err(TopologyError::PointCloudError(msg))` whose `msg` contains the substring `"duplicate point"` and references both indices `i` and `j`

#### Scenario: Near-duplicate within `T::epsilon() * max_extent` is rejected

- **WHEN** the caller invokes `PointCloud::triangulate` on a `PointCloud<f64, 2>` containing two points whose Euclidean distance is less than `f64::EPSILON * max_extent`
- **THEN** the call returns `Err(TopologyError::PointCloudError(msg))` whose `msg` contains the substring `"duplicate point"`

#### Scenario: Distinct points above threshold succeed

- **WHEN** the caller invokes `PointCloud::triangulate` on a `PointCloud<f64, 2>` containing three points forming a non-degenerate triangle with edge lengths above `f64::EPSILON * max_extent`
- **THEN** the call returns `Ok(complex)` and the complex contains three vertices, three edges, and one triangle

### Requirement: PointCloud::triangulate rejects zero-volume top-dimensional simplices

The crate `deep_causality_topology` SHALL detect degenerate top-dimensional simplices in `PointCloud::triangulate` and return a discriminating `TopologyError::PointCloudError(String)` rather than silently substituting `T::zero()` into the top mass matrix diagonal.

A top simplex is degenerate when its computed volume falls below `T::epsilon() * <T as From<f64>>::from(100.0)`. This applies whether the degeneracy is detected at the volume comparison or at the singular-Gram-matrix branch inside `gaussian_determinant`; both detection paths produce a unified error message.

The error message MUST contain the literal substrings `"top-dimensional simplex"` and `"below tolerance"` and the offending simplex index.

#### Scenario: Three collinear points produce an error

- **WHEN** the caller invokes `PointCloud::triangulate` on a `PointCloud<f64, 2>` with three points lying on a single line
- **AND** `radius` is at least the largest pairwise distance among the input points
- **THEN** the call returns `Err(TopologyError::PointCloudError(msg))` whose `msg` contains the substrings `"top-dimensional simplex"` and `"below tolerance"`

#### Scenario: Four coplanar points in 3D ambient produce an error

- **WHEN** the caller invokes `PointCloud::triangulate` on a `PointCloud<f64, 3>` with four points all sharing the same `z` coordinate
- **AND** `radius` is at least the largest pairwise distance among the input points
- **THEN** the call returns `Err(TopologyError::PointCloudError(msg))` whose `msg` contains the substring `"top-dimensional simplex"`

#### Scenario: Volume just above threshold succeeds

- **WHEN** the caller invokes `PointCloud::triangulate` on input whose top simplex volume falls just above `T::epsilon() * 100`
- **THEN** the call returns `Ok(complex)` and no top mass diagonal entry is substituted with zero

### Requirement: PointCloud::triangulate uses RealField-parametric tolerance constants

All numerical tolerance comparisons inside `PointCloud::triangulate` SHALL be expressed as functions of `T::epsilon()` and not as hard-coded `f64` literals. The hard-coded `1e-12` literals previously present at the singular-Gram-matrix and zero-volume branches SHALL be replaced with `T::epsilon() * <T as From<f64>>::from(100.0)`.

This restores precision-parametric behaviour across the `RealField` family (`f32`, `f64`, `Float106`).

#### Scenario: triangulate's tolerance scales with T's epsilon

- **WHEN** a reviewer searches `op_triangulate.rs` for hard-coded `1e-12` literals
- **THEN** zero occurrences are found inside the `triangulate` method body or the helpers it calls (`simplex_volume`, `gaussian_determinant`)

#### Scenario: triangulate behaves consistently across precision backends

- **WHEN** the caller invokes `PointCloud::<f32, 2>::triangulate`, `PointCloud::<f64, 2>::triangulate`, and `PointCloud::<Float106, 2>::triangulate` on logically-identical non-degenerate input
- **THEN** all three calls return `Ok(complex)` with the same vertex, edge, and triangle counts

### Requirement: SimplicialComplex populates Hodge ⋆ operators lazily

`SimplicialComplex<T>` SHALL NOT eagerly populate Hodge ⋆ operators at construction. The operators SHALL be built on first invocation of the `hodge_star_operators(&self) -> Result<&Vec<CsrMatrix<T>>, TopologyError>` accessor, using coordinates and ambient dimension supplied to the new `with_geometry(...)` constructor. The build result SHALL be cached so subsequent calls return without recomputation.

Topological consumers (Vietoris-Rips / clique-complex / Euler characteristic) that never invoke the accessor SHALL NOT pay the lazy-build cost and SHALL NOT see Hodge ⋆ degeneracy errors. The H1 degeneracy rejection (zero-volume top simplex) moves from `PointCloud::triangulate` to the lazy-build path; the unified `"top-dimensional simplex...below tolerance"` error surfaces only when the accessor is called on a complex whose top simplices are degenerate.

The accessor signature is fallible. No infallible convenience variant SHALL exist. No code path in the API SHALL panic on degenerate Hodge ⋆ input.

#### Scenario: TDA consumer succeeds on geometrically-degenerate input

- **WHEN** the caller invokes `PointCloud::triangulate` on a `PointCloud<f64, 3>` whose Vietoris-Rips clique complex contains at least one zero-volume top simplex, and the caller only reads `complex.skeletons()`, `complex.num_elements_at_grade(_)`, `complex.dimension()`, `complex.boundary_operators()`, or `complex.coboundary_operators()`
- **THEN** `triangulate` returns `Ok(complex)` and every read operation returns valid data
- **AND** no `Err` is propagated and no panic occurs

#### Scenario: DEC consumer sees the degeneracy error at first Hodge ⋆ access

- **WHEN** the caller invokes `complex.hodge_star_operators()` on a complex whose top simplices include at least one of volume below `T::epsilon() * 100`
- **THEN** the call returns `Err(TopologyError::PointCloudError(msg))` whose `msg` contains the substrings `"top-dimensional simplex"` and `"below tolerance"`
- **AND** the cell remains uninitialised so a subsequent call retries the build deterministically

#### Scenario: Hodge ⋆ is computed once and cached

- **WHEN** the caller invokes `complex.hodge_star_operators()` twice on a complex with non-degenerate geometry
- **THEN** both calls return `Ok(&vec)` referencing the same underlying allocation
- **AND** the lazy build runs exactly once

#### Scenario: Empty complex returns Ok(empty Vec) without attempting to build

- **WHEN** the caller invokes `complex.hodge_star_operators()` on a `SimplicialComplex<f64>` whose `skeletons` is empty
- **THEN** the call returns `Ok(&empty_vec)` without invoking the lazy-build helper

#### Scenario: Complex constructed without geometric data returns a discriminating error

- **WHEN** the caller invokes `complex.hodge_star_operators()` on a `SimplicialComplex<f64>` constructed via `SimplicialComplex::new(skeletons, boundary, coboundary, hodge_star_operators)` with an empty `hodge_star_operators` argument and no geometric data
- **THEN** the call returns `Err(TopologyError::PointCloudError(msg))` whose `msg` indicates that geometric data is not available

### Requirement: HasHodgeStar trait method is fallible

The trait `HasHodgeStar` SHALL declare `hodge_star_matrix` with a fallible signature:

```rust
fn hodge_star_matrix(...) -> Result<Cow<'_, CsrMatrix<R>>, TopologyError>;
```

Both implementations (`ReggeGeometry<R>` on the simplicial side, `CubicalReggeGeometry<D, R, M>` on the cubical side) SHALL implement this signature. The cubical impl wraps its existing infallible computation in `Ok(...)`; the simplicial impl propagates the error from the lazy Hodge ⋆ accessor.

#### Scenario: Cubical impl never produces an Err

- **WHEN** the caller invokes `cubical_geometry.hodge_star_matrix(k)` for any valid `k`
- **THEN** the call returns `Ok(matrix)`

#### Scenario: Simplicial impl propagates the lazy-init error

- **WHEN** the caller invokes `regge_geometry.hodge_star_matrix(k)` on a `ReggeGeometry<f64>` backed by a `SimplicialComplex` with degenerate top geometry
- **THEN** the call returns `Err(TopologyError::PointCloudError(msg))` with the same message the lazy accessor produces

### Requirement: PointCloud::triangulate documents its precondition contract

The doc-comment on `PointCloud::triangulate` SHALL document the three rejection rules, the `T::epsilon() * 100` scaling convention, and the regime in which the floating-point predicates are sound. The contract MUST be readable by downstream consumers without reading the implementation.

#### Scenario: Documentation enumerates the three rejection categories

- **WHEN** a reviewer reads the doc-comment on `PointCloud::triangulate`
- **THEN** the doc-comment names all three rejection categories — duplicate input points, zero-volume top simplex, singular Gram matrix — and the unified `T::epsilon() * 100` threshold convention

