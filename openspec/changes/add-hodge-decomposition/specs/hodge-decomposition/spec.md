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

### Requirement: HodgeDecompositionFailed error variant

The crate `deep_causality_topology` SHALL extend the existing `TopologyErrorEnum` with a single new stringly-typed variant `HodgeDecompositionFailed(String)`, matching the convention already established by `ManifoldError(String)`, `LinkVariableError(String)`, and the other variants on the same enum. A constructor function `TopologyError::HodgeDecompositionFailed(msg)` SHALL be provided mirroring the existing `TopologyError::ManifoldError(msg)` constructor pattern.

The four documented failure modes of the decomposition — non-convergence of the iterative solve, grade out of range, field-dimension mismatch, and missing metric — SHALL each produce a `HodgeDecompositionFailed(...)` error whose contained message contains a discriminating substring (e.g. `"did not converge"`, `"grade ... exceeds"`, `"field length ... does not match"`, `"no metric attached"`).

Internal to the `hodge_decompose` implementation a private enum `HodgeFailReason<R: RealField>` MAY carry the typed `R`-precision detail for control flow within the function body. That enum MUST NOT escape the module boundary: it is converted to the stringly-typed `TopologyErrorEnum::HodgeDecompositionFailed(String)` at the `Err` construction site via R's `Display` impl. No public surface added by this change set SHALL contain `f64` or any other precision-bearing type other than `R`.

#### Scenario: Caller can pattern-match on the Hodge failure variant

- **WHEN** a `hodge_decompose` call returns `Err(e)` where `e.0` is `TopologyErrorEnum::HodgeDecompositionFailed(msg)`
- **THEN** the caller can distinguish a Hodge decomposition failure from other `TopologyErrorEnum` variants at the variant level, and the `msg` carries the human-readable cause

#### Scenario: Grade-out-of-range failure carries the offending grade and the manifold's max dimension in the message

- **WHEN** the caller invokes `hodge_decompose(field, k)` with `k > self.complex.max_dim()`
- **THEN** the call returns `Err(e)` where `e.0` is `TopologyErrorEnum::HodgeDecompositionFailed(msg)` and `msg` contains both the numeric value of `k` and the numeric value of `self.complex.max_dim()`

#### Scenario: Dimension-mismatch failure carries the expected and actual field lengths in the message

- **WHEN** the caller invokes `hodge_decompose(field, k)` with a `field` whose length does not match `self.complex.num_cells(k)`
- **THEN** the call returns `Err(e)` where `e.0` is `TopologyErrorEnum::HodgeDecompositionFailed(msg)` and `msg` contains both the expected and actual numeric lengths

#### Scenario: Non-convergence failure carries the iteration count and the final residual in the message

- **WHEN** the iterative solve does not reach the convergence tolerance within `max_iterations`
- **THEN** the call returns `Err(e)` where `e.0` is `TopologyErrorEnum::HodgeDecompositionFailed(msg)` and `msg` contains both the iteration count and a `Display`-formatted residual value

#### Scenario: No precision-bearing type leaks into the public error surface

- **WHEN** a reviewer searches the public surface of `TopologyError`, `TopologyErrorEnum`, and any new `pub` item added by this change set for `f64`
- **THEN** zero occurrences are found

### Requirement: Manifold::hodge_decompose method

The crate `deep_causality_topology` SHALL provide a method `Manifold::hodge_decompose(&self, field: &CausalTensor<R>, k: usize) -> Result<HodgeDecomposition<R>, TopologyError>` available on every `Manifold<K, R>` whose `K::Metric: HasHodgeStar<R>` and whose `R: RealField + FromPrimitive + Display`. The additional `Display` bound is required so that the residual carried inside a non-convergence failure can be formatted into the stringly-typed `TopologyErrorEnum::HodgeDecompositionFailed(String)` variant without leaking `R` (or `f64`) into the public error surface. `Display` is satisfied by `f32`, `f64`, and any other reasonable `RealField` implementation. The method MUST be generic over the chain-complex type `K` and the precision type `R`; it MUST NOT be specialised to `SimplicialComplex<R>` or to `f64`.

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
- **THEN** the call returns `Err(e)` where `e.0` is `TopologyErrorEnum::HodgeDecompositionFailed(msg)` whose message indicates a grade-out-of-range cause, and no solve is attempted

#### Scenario: Reject decomposition request when field has the wrong dimension

- **WHEN** the caller invokes `manifold.hodge_decompose(&field, k)` with a `field` whose length does not match `self.complex.num_cells(k)`
- **THEN** the call returns `Err(e)` where `e.0` is `TopologyErrorEnum::HodgeDecompositionFailed(msg)` whose message indicates a dimension-mismatch cause, and no solve is attempted

#### Scenario: Report non-convergence with iteration count and final residual in the message

- **WHEN** the iterative solve does not reach the convergence tolerance within `max_iterations` for some field
- **THEN** the call returns `Err(e)` where `e.0` is `TopologyErrorEnum::HodgeDecompositionFailed(msg)` and `msg` reports both the iteration count and the final relative residual via R's `Display` impl

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

A prescribed pure-exact 1-form field on a non-degenerate planar domain SHALL produce orthogonally-equivalent Hodge decompositions when decomposed via the simplicial backend (`ReggeGeometry<R>`) and via the cubical backend (`CubicalReggeGeometry<2, R, Euclidean>`). The two decompositions MUST agree on the algebraic structure of the result to tolerance `1e-6` in `f64`:

- The Hodge orthogonality identity `‖α‖² + ‖β‖² + ‖h‖² = ‖ω‖²` holds on each backend at `1e-6`.
- For `ω = df` (pure exact), both backends report `(‖β‖² + ‖h‖²) / ‖ω‖² < 1e-6` (the vanishing components are individually at noise floor on each backend).
- The cross-backend disagreement on the vanishing-component ratio is itself `< 1e-6`.

The strict per-component L2 norm equality scenario `|‖simplicial.exact()‖ − ‖cubical.exact()‖| < 1e-6` is deferred to a follow-up change set `add-hodge-decomposition-delaunay-cross-backend` (see `tasks.md` Section 6 and `design.md` Risk 5) because the canonical two-triangle simplicial unit square requires a manifold-respecting (Delaunay or constrained-Delaunay) triangulation that `PointCloud::triangulate` does not currently provide.

#### Scenario: Both backends satisfy the Hodge orthogonality identity on the unit square

- **WHEN** the caller decomposes a prescribed pure-exact 1-form `ω = df` on the simplicial backend and on the cubical unit square
- **THEN** each backend's `‖α‖² + ‖β‖² + ‖h‖²` matches `‖ω‖²` to relative error `< 1e-6`

#### Scenario: Both backends agree on the vanishing-component ratio for a pure-exact 1-form

- **WHEN** the caller decomposes the same prescribed `ω = df` on both backends
- **THEN** each backend individually reports `(‖β‖² + ‖h‖²) / ‖ω‖² < 1e-6`, and the absolute disagreement between the two ratios is itself `< 1e-6`

### Requirement: R: RealField precision parameterisation end-to-end

Every new public signature added by this change set MUST be generic over `R: RealField` (with `+ FromPrimitive` declared at the method site only where literal construction is required). No new public type, method, or trait SHALL use `f64` in its signature. The single exception is documented internally and does not appear in any public surface of this change set.

#### Scenario: No new public signature contains f64

- **WHEN** a reviewer searches the new and modified source files for `f64` in public signatures
- **THEN** zero occurrences are found in `pub` items, `pub fn` parameters, `pub fn` return types, or `pub trait` method signatures

#### Scenario: HodgeDecomposition can be instantiated at multiple precision backends

- **WHEN** the caller constructs `HodgeDecomposition<f32>`, `HodgeDecomposition<f64>`, and `HodgeDecomposition<DoubleFloat>`
- **THEN** all three instantiations compile and pass their respective coverage tests
