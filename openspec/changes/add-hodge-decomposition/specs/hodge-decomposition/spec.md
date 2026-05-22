## ADDED Requirements

### Requirement: HodgeDecomposition carrier type

The crate `deep_causality_topology` SHALL provide a public type `HodgeDecomposition<R: RealField>` that holds the three pairwise-orthogonal components produced by the discrete Hodge–Helmholtz decomposition of a k-form field on a `Manifold<K, R>`. The components are `exact: CausalTensor<R>`, `co_exact: CausalTensor<R>`, and `harmonic: CausalTensor<R>`, together with the grade `k: usize`.

Fields MUST be private. Read access SHALL be provided by getters `exact()`, `co_exact()`, `harmonic()`, and `grade()`, each returning a borrowed view of the respective field. No setters or interior-mutability surfaces are exposed.

The type SHALL be parameterised over `R: RealField` with no other trait bounds at the type level. Methods that require additional bounds (e.g. `+ FromPrimitive`) MUST declare them at the method site, not on the struct.

#### Scenario: Construct a HodgeDecomposition from three components

- **WHEN** the caller invokes `HodgeDecomposition::new(exact, co_exact, harmonic, k)` with three `CausalTensor<R>` values of matching dimensions and a grade `k`
- **THEN** a `HodgeDecomposition<R>` value is returned with each component accessible via its getter

#### Scenario: Getters return borrowed views, not owned data

- **WHEN** the caller invokes any of `exact()`, `co_exact()`, `harmonic()`, or `grade()` on a `HodgeDecomposition<R>` value
- **THEN** the call returns a reference into the carrier rather than moving the field out

#### Scenario: Fields are not publicly mutable

- **WHEN** any caller attempts to mutate a component field of `HodgeDecomposition<R>` from outside the carrier type's module
- **THEN** the access fails at compile time due to private field visibility

### Requirement: HodgeFailReason error variants

The crate `deep_causality_topology` SHALL extend `ManifoldError` with a new variant `HodgeDecompositionFailed { reason: HodgeFailReason }`. The enum `HodgeFailReason` SHALL enumerate the documented failure modes of the decomposition: `Nonconvergence { iterations: usize, residual: R }`, `GradeOutOfRange { k: usize, max_dim: usize }`, `DimensionMismatch { expected: usize, actual: usize }`, and `MissingMetric`. The `Nonconvergence` variant's `residual` field is generic over `R` via a boxed concrete-type erasure or a type-parameterised error variant; the exact shape is fixed in the H1 implementation.

#### Scenario: Caller can pattern-match on the failure reason

- **WHEN** a `hodge_decompose` call returns `Err(ManifoldError::HodgeDecompositionFailed { reason })`
- **THEN** the caller can pattern-match `reason` against any of the four documented `HodgeFailReason` variants without a catch-all

#### Scenario: GradeOutOfRange carries the offending grade and the manifold's max dimension

- **WHEN** the caller invokes `hodge_decompose(field, k)` with `k > self.complex.max_dim()`
- **THEN** the call returns `Err(ManifoldError::HodgeDecompositionFailed { reason: HodgeFailReason::GradeOutOfRange { k, max_dim } })` where `max_dim == self.complex.max_dim()`

### Requirement: Manifold::hodge_decompose method

The crate `deep_causality_topology` SHALL provide a method `Manifold::hodge_decompose(&self, field: &CausalTensor<R>, k: usize) -> Result<HodgeDecomposition<R>, ManifoldError>` available on every `Manifold<K, R>` whose `K::Metric: HasHodgeStar<R>` and whose `R: RealField + FromPrimitive`. The method MUST be generic over the chain-complex type `K` and the precision type `R`; it MUST NOT be specialised to `SimplicialComplex<R>` or to `f64`.

The method SHALL implement the discrete Hodge–Helmholtz decomposition via the algorithm specified in `design.md` Decision 2: two sequential discrete Poisson solves of the form `Δ_k φ = (boundary operator)(ω)`, followed by a residual computation for the harmonic component.

Convergence of the iterative solve SHALL be controlled by a tolerance `ε_R` derived from `R`'s machine epsilon per `design.md` Decision 6, with an override path via an optional `HodgeDecomposeOptions { tolerance: Option<R>, max_iterations: Option<usize> }` parameter.

#### Scenario: Decompose a pure exact 1-form into its components

- **WHEN** the caller invokes `manifold.hodge_decompose(&pure_exact_1form, 1)` on a `Manifold<LatticeComplex<2>, R, Euclidean>` with trivial topology, where `pure_exact_1form == d f` for some scalar `f`
- **THEN** the returned `HodgeDecomposition<R>` has `‖co_exact‖² < ε_R` and `‖harmonic‖² < ε_R` to numerical tolerance

#### Scenario: Decompose a pure co-exact 1-form into its components

- **WHEN** the caller invokes `manifold.hodge_decompose(&pure_co_exact_1form, 1)` on the same manifold, where `pure_co_exact_1form == δ g` for some 2-form `g`
- **THEN** the returned `HodgeDecomposition<R>` has `‖exact‖² < ε_R` and `‖harmonic‖² < ε_R` to numerical tolerance

#### Scenario: Decompose an arbitrary 1-form into all three components

- **WHEN** the caller invokes `manifold.hodge_decompose(&mixed_1form, 1)` on a manifold whose topology admits a non-trivial harmonic component
- **THEN** the returned `HodgeDecomposition<R>` has all three component L2 norms strictly positive, and `‖exact‖² + ‖co_exact‖² + ‖harmonic‖² == ‖mixed_1form‖²` to numerical tolerance

#### Scenario: Reject decomposition request when grade exceeds the manifold's max dimension

- **WHEN** the caller invokes `manifold.hodge_decompose(&field, k)` with `k > self.complex.max_dim()`
- **THEN** the call returns `Err(ManifoldError::HodgeDecompositionFailed { reason: HodgeFailReason::GradeOutOfRange { .. } })` without attempting any solve

#### Scenario: Reject decomposition request when field has the wrong dimension

- **WHEN** the caller invokes `manifold.hodge_decompose(&field, k)` with a `field` whose length does not match `self.complex.num_cells(k)`
- **THEN** the call returns `Err(ManifoldError::HodgeDecompositionFailed { reason: HodgeFailReason::DimensionMismatch { .. } })` without attempting any solve

#### Scenario: Report non-convergence with iteration count and final residual

- **WHEN** the iterative solve does not reach the convergence tolerance within `max_iterations` for some field
- **THEN** the call returns `Err(ManifoldError::HodgeDecompositionFailed { reason: HodgeFailReason::Nonconvergence { iterations, residual } })` where `iterations == max_iterations` and `residual` is the final relative residual

#### Scenario: Method available for both simplicial and cubical backends through the same generic impl

- **WHEN** the caller invokes `hodge_decompose` on a `Manifold<SimplicialComplex<R>, R>` and again on a `Manifold<LatticeComplex<D>, R, Euclidean>`
- **THEN** both calls compile and execute against the same generic method body; no backend-specialised method exists

### Requirement: Hodge orthogonality identity

The decomposition produced by `Manifold::hodge_decompose` SHALL satisfy the Hodge orthogonality identity `‖exact‖² + ‖co_exact‖² + ‖harmonic‖² = ‖field‖²` to numerical tolerance `ε_R` derived from `R`'s machine epsilon, for every valid input field on every manifold whose metric satisfies `HasHodgeStar<R>`.

#### Scenario: Orthogonality holds across lattice sizes

- **WHEN** the caller decomposes the same prescribed 1-form on `LatticeComplex<3>` grids of sizes `4³`, `8³`, and `16³` using `CubicalReggeGeometry<3, R, Euclidean>`
- **THEN** the orthogonality identity holds on each grid to tolerance `ε_R`

#### Scenario: Orthogonality holds across precision backends

- **WHEN** the caller decomposes the same prescribed 1-form on the same grid using `R = f32`, `R = f64`, and `R = DoubleFloat`
- **THEN** the orthogonality identity holds at each backend's natural tolerance `ε_R`

### Requirement: Two-backend cross-check on the unit square

A prescribed 1-form field on the unit square SHALL produce orthogonally-equivalent Hodge decompositions when decomposed via the simplicial backend (`ReggeGeometry<R>` over a complex of two triangles) and via the cubical backend (`CubicalReggeGeometry<2, R, Euclidean>` over a single 2-cube). The two decompositions MUST agree on the L2 norm of each component to tolerance `1e-6` in `f64`.

#### Scenario: Simplicial and cubical decompositions agree on component norms

- **WHEN** the caller decomposes the same prescribed 1-form field once via the simplicial backend on the unit square (two triangles) and once via the cubical backend on the unit square (one 2-cube)
- **THEN** `|‖simplicial.exact()‖ − ‖cubical.exact()‖| < 1e-6`, and likewise for `co_exact` and `harmonic`

### Requirement: PyDEC parity benchmark on the unit square

The change set SHALL ship static test fixtures under `tests/types/hodge_decomposition/pydec_fixtures.rs` capturing the PyDEC reference decomposition values for three prescribed 1-form configurations on the unit square: a pure-exact field, a field with non-trivial co-exact part, and the same fields on the cubical equivalent. The `hodge_decompose` output SHALL agree with the fixture values to ~5 significant figures (`< 1e-5` relative error in `f64`).

The fixture file SHALL record the PyDEC source version and git SHA from which the values were derived. Fixture updates SHALL be manual and version-pinned; no automated regeneration is permitted.

#### Scenario: Simplicial decomposition matches PyDEC fixture for a pure-exact field

- **WHEN** the caller decomposes the fixture-defined pure-exact 1-form on the simplicial unit square
- **THEN** every component L2 norm matches the fixture value to relative error `< 1e-5`

#### Scenario: Simplicial decomposition matches PyDEC fixture for a mixed field

- **WHEN** the caller decomposes the fixture-defined mixed 1-form (non-trivial co-exact part) on the simplicial unit square
- **THEN** every component L2 norm matches the fixture value to relative error `< 1e-5`

#### Scenario: Cubical decomposition matches simplicial decomposition (PyDEC-derived) on the same unit square

- **WHEN** the caller decomposes the fixture-defined mixed 1-form on the cubical unit square (one 2-cube)
- **THEN** every component L2 norm agrees with the simplicial PyDEC fixture value to relative error `< 1e-5`

### Requirement: R: RealField precision parameterisation end-to-end

Every new public signature added by this change set MUST be generic over `R: RealField` (with `+ FromPrimitive` declared at the method site only where literal construction is required). No new public type, method, or trait SHALL use `f64` in its signature. The single exception is documented internally and does not appear in any public surface of this change set.

#### Scenario: No new public signature contains f64

- **WHEN** a reviewer searches the new and modified source files for `f64` in public signatures
- **THEN** zero occurrences are found in `pub` items, `pub fn` parameters, `pub fn` return types, or `pub trait` method signatures

#### Scenario: HodgeDecomposition can be instantiated at multiple precision backends

- **WHEN** the caller constructs `HodgeDecomposition<f32>`, `HodgeDecomposition<f64>`, and `HodgeDecomposition<DoubleFloat>`
- **THEN** all three instantiations compile and pass their respective coverage tests
