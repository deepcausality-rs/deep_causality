## ADDED Requirements

### Requirement: Zero hardcoded `f64` or `f32` in the public API of `deep_causality_topology`

The crate's public API SHALL contain no hardcoded `f64` or `f32` in any struct field, function/method signature, trait method, error variant, or trait bound. Every floating-point quantity SHALL be `R: RealField` for some `R` chosen by the caller. The single permitted exception is the internal RNG sample conversion in `metropolis_step` (`<R as FromPrimitive>::from_f64(rng.random::<f64>()).expect("RNG sample fits")`), which is not part of the public API.

#### Scenario: Workspace-wide grep finds no `f64` in topology public signatures

- **WHEN** the command `grep -rn -E '(\bf64\b|\bf32\b)' deep_causality_topology/src/ --include='*.rs'` is run and its output is filtered to lines representing public signatures, struct fields, trait methods, error variants, or trait bounds
- **THEN** the filtered output SHALL be empty, with the documented exception of the one `<R as FromPrimitive>::from_f64(rng.random::<f64>()).expect("RNG sample fits")` call site in the Metropolis step

#### Scenario: No `From<f64>`, `Into<f64>`, or `Mul<f64, Output = T>` trait bounds remain

- **WHEN** the crate's source is searched for the patterns `From<f64>`, `Into<f64>`, and `Mul<f64, Output`
- **THEN** zero matches SHALL appear in any `impl` block or trait bound declaration

### Requirement: `ChainComplex::Metric` is a generic associated type (GAT) parameterized over `R: RealField`

The `ChainComplex` trait SHALL declare its `Metric` associated type as a generic associated type:

```rust
pub trait ChainComplex {
    // ...
    type Metric<R: RealField>;
    // ...
}
```

This places precision as a parameter on the *metric*, not on the chain complex itself. A chain complex is a combinatorial object; the metric is a precision-carrying layer over it. The GAT lets `LatticeComplex<D>` (purely combinatorial, no `R`-typed storage) keep a single type identity while binding `type Metric<R: RealField> = CubicalReggeGeometry<D, R>`. The same `LatticeComplex<D>` instance can be used with metrics at different precisions.

Implementors SHALL provide the GAT binding:

- `impl<const D: usize> ChainComplex for LatticeComplex<D>` → `type Metric<R: RealField> = CubicalReggeGeometry<D, R>;`
- `impl<C> ChainComplex for SimplicialComplex<C>` (with appropriate bounds on `C`) → `type Metric<R: RealField> = ReggeGeometry<R>;`
- `impl<C> ChainComplex for CellComplex<C>` → `type Metric<R: RealField> = ();` (no metric available; works for any `R`)
- Any other `ChainComplex` implementor SHALL similarly provide a `Metric<R: RealField> = ...` binding.

`Manifold<K, F>` SHALL retype its `metric` field from `Option<K::Metric>` to `Option<K::Metric<F>>` where `F` is the manifold's field-data type (already bounded `F: RealField + FromPrimitive` per the propagation policy). Every `K::Metric` use site in the crate (`extensions/hkt_manifold`, `manifold/getters`, etc.) SHALL retype to `K::Metric<R>` with `R` in scope from the surrounding generic context.

#### Scenario: `LatticeComplex<D>` stays combinatorial

- **WHEN** the source is inspected for `LatticeComplex<...>` declarations
- **THEN** `LatticeComplex<D>` SHALL carry exactly one type parameter (`const D: usize`); no `R`, no `PhantomData<R>`, no precision marker SHALL be added to the complex itself

#### Scenario: One lattice, two precisions of metric

- **WHEN** a single `LatticeComplex::<3>::new(...)` instance is constructed
- **AND** that instance is referenced from `Manifold<LatticeComplex<3>, f64>` and `Manifold<LatticeComplex<3>, f32>` in the same program
- **THEN** both manifolds SHALL be valid; the lattice SHALL be the same `LatticeComplex<3>` value used by both; the metrics SHALL be `CubicalReggeGeometry<3, f64>` and `CubicalReggeGeometry<3, f32>` respectively

#### Scenario: `CellComplex<C>` has no metric at any precision

- **WHEN** `Manifold<CellComplex<C>, F>::metric()` is called for any `F: RealField + FromPrimitive`
- **THEN** the field SHALL be `Option<()>` (i.e. the cell complex has no metric; the `()` placeholder satisfies the GAT for any `R`)

#### Scenario: `K::Metric<F>` resolves the GAT at use sites

- **WHEN** a generic function bounds `K: ChainComplex` and `F: RealField + FromPrimitive` and accepts `metric: Option<K::Metric<F>>`
- **THEN** the code SHALL compile and the resolved `K::Metric<F>` SHALL be the appropriate concrete metric type for `K` at precision `F`

### Requirement: `CubicalReggeGeometry<D>` is parameterized over `R: RealField`

The type SHALL be `CubicalReggeGeometry<const D: usize, R: RealField>`. The private `EdgeLengths<D, R>` enum's `Uniform`, `PerAxis`, and `PerEdge` variants SHALL carry `R` in place of `f64`. The `UnitEdge` variant remains parameterless. All eight constructors and accessors SHALL retype: `uniform(length: R)`, `per_axis(lengths: [R; D])`, `from_edge_lengths(lengths: Vec<R>)`, `uniform_length() -> Option<R>`, `axis_lengths() -> Option<[R; D]>`, `axis_length(axis: usize) -> Option<R>`, `edge_length_at(edge_id: usize) -> Option<R>`, `edge_lengths() -> Option<&[R]>`.

#### Scenario: Construct at `f64` precision

- **WHEN** `CubicalReggeGeometry::<3, f64>::unit()` is called
- **THEN** the result SHALL be a `CubicalReggeGeometry<3, f64>` with unit edge lengths, and downstream methods SHALL accept and return `f64` values

#### Scenario: Construct at `f32` precision

- **WHEN** `CubicalReggeGeometry::<3, f32>::per_axis([1.0_f32, 2.0_f32, 3.0_f32])` is called
- **THEN** the result SHALL be a `CubicalReggeGeometry<3, f32>`, and `axis_lengths()` SHALL return `Some([1.0_f32, 2.0_f32, 3.0_f32])`

#### Scenario: No type alias `CubicalReggeGeometry64<D>` is exposed

- **WHEN** the crate's public API is inspected for type aliases ending in `64`
- **THEN** no such alias SHALL be present (per the no-bridge-code policy)

### Requirement: `ReggeGeometry<T>` is renamed and rebounded as `ReggeGeometry<R: RealField>`

The simplicial type SHALL be `ReggeGeometry<R: RealField>`. The bound `T: Float + Zero + Copy + PartialOrd + From<f64> + Into<f64>` is replaced with `R: RealField`. All curvature helpers (`compute_dihedral_angle`, `calculate_ricci_curvature`, the private determinant / area / volume routines) SHALL return `R` instead of `f64`. Internal computation SHALL operate on `R` end-to-end with no `f64` round-trips.

#### Scenario: Curvature methods return the field's own precision

- **WHEN** `ReggeGeometry::<f64>::new(...).calculate_ricci_curvature(complex)` is called
- **THEN** the result SHALL be `Result<CausalTensor<f64>, TopologyError>`

- **AND WHEN** the same call is made with `ReggeGeometry::<f32>`
- **THEN** the result SHALL be `Result<CausalTensor<f32>, TopologyError>`

### Requirement: `CurvatureTensor` is parameterized over `R: RealField`

The type SHALL be parameterized over `R: RealField` in place of the current `T: Field + Copy + PartialOrd + Float + From<f64> + Into<f64>` bound. Every impl block (flat-tensor construction, index raising / lowering, Ricci, Kretschmann) SHALL operate on `R` without `From<f64>` / `Into<f64>` round-trips. Numeric literals previously materialized via `<T as From<f64>>::from(literal)` SHALL be materialized via `<R as FromPrimitive>::from_f64(literal).expect("...")` or a `RealField`-native expression.

#### Scenario: Flat tensor construction at `f64`

- **WHEN** a flat `CurvatureTensor<f64, /* ... */>` is constructed
- **THEN** every field SHALL be `f64`-typed and every internal coefficient SHALL be derived through `RealField` methods

### Requirement: `Manifold` covariance / geometry / differential APIs return the manifold's scalar field

`Manifold::covariance_matrix` SHALL return `Result<Vec<Vec<R>>, TopologyError>` for an `R` bound to the manifold's existing scalar type parameter. `Manifold::eigen_covariance` SHALL return `Result<Vec<R>, TopologyError>`. `Manifold::simplex_volume_squared` SHALL return `Result<R, TopologyError>`. The `D: Into<f64> + Copy` and `C: From<f64> + Into<f64>` bounds SHALL be removed in favor of `R: RealField` (or wherever applicable, `RealField` on the existing parameter).

#### Scenario: Covariance matrix at `f32`

- **WHEN** `manifold.covariance_matrix()` is called on a `Manifold<SimplicialComplex<f32>, f32>`
- **THEN** the result SHALL be `Result<Vec<Vec<f32>>, TopologyError>`

#### Scenario: Simplex volume at `f64`

- **WHEN** `manifold.simplex_volume_squared(&simplex)` is called on a `Manifold<SimplicialComplex<f64>, f64>`
- **THEN** the result SHALL be `Result<f64, TopologyError>` and equal (bit-identically) the pre-R0 result on the same input

### Requirement: `Manifold::differential::{laplacian, codifferential, hodge, exterior}` drop `From<f64>` bounds

These differential operators SHALL operate on `R: RealField` only. The internal tolerance constants (`1e-12` etc.) SHALL be materialized via `R::epsilon()` (where appropriate) or `<R as FromPrimitive>::from_f64(...).expect("...")`.

#### Scenario: Laplacian on a generic manifold

- **WHEN** `manifold.laplacian(k)` is called on a `Manifold<K, F>` where `F: RealField`
- **THEN** the return type SHALL match the manifold's field-data type and SHALL NOT require a `From<f64>` bound on any caller-side type

### Requirement: `DifferentialForm<T>::scale` accepts a generic `R: RealField` scalar

The method SHALL have signature `pub fn scale(&self, scalar: R) -> Self` under an `impl<T, R> DifferentialForm<T> where T: Clone + Default + Mul<R, Output = T>, R: RealField` block. The current `Mul<f64, Output = T>` and `From<f64>` bounds SHALL be removed.

#### Scenario: Scale at `f32`

- **WHEN** a `DifferentialForm<f32>` is scaled by an `f32` scalar
- **THEN** the call SHALL compile and produce a scaled `DifferentialForm<f32>`

#### Scenario: Scale at custom precision

- **WHEN** a `DifferentialForm<f128>` is scaled by an `f128` scalar (once `f128` stabilizes)
- **THEN** no source-level change SHALL be required beyond the precision choice at the construction site

### Requirement: `PointCloud::triangulate` operates on `R: RealField` without `f64` round-trips

The triangulate, Gaussian elimination, Hodge dual, and volume helpers SHALL bound `T: RealField` only. Every literal (epsilon tolerances, fixed coefficients) SHALL be materialized via `<T as FromPrimitive>::from_f64(literal).expect("...")` or a `RealField`-native expression.

#### Scenario: Triangulation at `f32`

- **WHEN** `PointCloud::<f32>::triangulate(...)` is called
- **THEN** all internal computation SHALL run at `f32` precision; no `f64` round-trip SHALL occur

### Requirement: `GaugeGroup::structure_constant` returns `R: RealField`

The trait method SHALL be declared `fn structure_constant<R: RealField>(a: usize, b: usize, c: usize) -> R` with no default implementation. The four in-crate impls (`SU2`, `SU3`, `SE3`, `SO(3,1)`) SHALL each implement the method via `<R as FromPrimitive>::from_f64(literal).expect("structure constant fits")` for their hardcoded coefficient values.

#### Scenario: SU(2) structure constants at `f64`

- **WHEN** `SU2::structure_constant::<f64>(0, 1, 2)` is called
- **THEN** the result SHALL equal the canonical Levi-Civita value `1.0_f64` (bit-identical to the pre-R0 result)

#### Scenario: SU(2) structure constants at `f32`

- **WHEN** `SU2::structure_constant::<f32>(0, 1, 2)` is called
- **THEN** the result SHALL equal `1.0_f32`

#### Scenario: No default implementation

- **WHEN** an external implementor of `GaugeGroup` is compiled without supplying `structure_constant`
- **THEN** the compiler SHALL produce an error requiring the method to be defined

### Requirement: Metropolis acceptance ratio is `R`-typed

`metropolis_step` SHALL return `Result<R, TopologyError>` where `R` is the gauge field's existing real-scalar parameter. The single internal `f64` literal — `let rnd: f64 = rng.random()` — is preserved as the documented RNG-boundary exception; the result is immediately converted via `<R as FromPrimitive>::from_f64(...).expect(...)`.

#### Scenario: Acceptance ratio at `f32`

- **WHEN** `metropolis_step` is called on a `GaugeFieldLattice<G, M, f32>`
- **THEN** the returned acceptance ratio SHALL be `f32`-typed

### Requirement: Test utilities are generic over `R: RealField`

`create_triangle_complex<R: RealField>() -> SimplicialComplex<R>` and `create_line_complex<R: RealField>() -> SimplicialComplex<R>` SHALL be available. No `_f64` aliases SHALL be provided; existing test call sites SHALL update to `create_triangle_complex::<f64>()`.

#### Scenario: Build a complex at `f32` in a test

- **WHEN** a test calls `create_triangle_complex::<f32>()`
- **THEN** the result SHALL be a `SimplicialComplex<f32>`

### Requirement: Use the existing `FromPrimitive` trait for numeric-literal conversions

`deep_causality_num` already exposes a `FromPrimitive` trait (`deep_causality_num/src/cast/from_primitive/mod.rs`) with fallible `-> Option<Self>` constructors for every primitive numeric type (`from_f64`, `from_f32`, `from_i64`, `from_i32`, plus `i8`/`i16`/`i128`/`u*`/`isize`/`usize`). The topology refactor SHALL use this existing trait for any site that needs to materialize a numeric literal as `R`.

No new methods SHALL be added to the `RealField` trait. Topology generic-code sites that materialize literals SHALL bound `R: RealField + FromPrimitive` (or thread `FromPrimitive` as an additional bound where appropriate) and SHALL call `<R as FromPrimitive>::from_f64(literal).expect("constant fits in R")` (or equivalent).

The `expect` message SHALL be a short documented invariant: the conversion is infallible for the concrete `R` types in production use (`f32`, `f64`, `Float106`), but the `Option`-returning shape is preserved because `FromPrimitive` is the established trait surface.

#### Scenario: `RealField` trait surface is unchanged

- **WHEN** the `RealField` trait is inspected after R0 is applied
- **THEN** the trait SHALL have the same method set as before R0; no `from_f64`, `from_f32`, `from_i64`, or `from_i32` methods SHALL be added

#### Scenario: Topology generic code uses `FromPrimitive` for literals

- **WHEN** a generic function or method in `deep_causality_topology` materializes a numeric literal as `R`
- **THEN** the call site SHALL use `<R as FromPrimitive>::from_X(literal).expect("...")` (or matching `.unwrap()` / `.ok_or(...)` shape) — not a hypothetical `R::from_X` on `RealField`

#### Scenario: `FromPrimitive` is implemented for every `R` in production use

- **WHEN** the workspace is compiled at `R = f64`, `R = f32`, or `R = Float106`
- **THEN** each concrete `R` SHALL already implement `FromPrimitive`; no new `FromPrimitive` impls SHALL be needed for these types

#### Scenario: The literal-fits-in-R invariant is documented

- **WHEN** a topology call site uses `<R as FromPrimitive>::from_f64(0.5).expect("0.5 is representable in every RealField")`
- **THEN** the `expect` message SHALL state the invariant clearly; an `unwrap` without a message SHALL NOT be used

### Requirement: Behavior at `R = f64` is bit-identical to the pre-R0 baseline

Every test that passed before R0 at `f64` precision SHALL pass after R0 at `f64` precision with bit-identical numerical results. No algorithm's computational content changes; only types and trait bounds.

#### Scenario: Existing simplicial-complex test passes unchanged

- **WHEN** the existing simplicial-complex / Manifold / ReggeGeometry test suite is run at `R = f64`
- **THEN** every test SHALL pass; every floating-point comparison SHALL match the pre-R0 result bit-for-bit

### Requirement: Property-test pass at `R = f32`

Every algorithmically-meaningful test (cell volume, dihedral angle, Cayley-Menger volume, curvature tensor invariants, Gaussian elimination determinants, structure-constant returns) SHALL be duplicated against `R = f32` with widened tolerances appropriate to `f32::EPSILON`.

#### Scenario: `f32` duplicate exists for every algorithmic `f64` test

- **WHEN** the test suite is enumerated for tests of the categories listed above
- **THEN** every `f64` test SHALL have a corresponding `f32` test in the same file with `_f32` suffix on the test name

#### Scenario: `f32` test catches a hidden `f64` round-trip

- **WHEN** a hypothetical refactor regression introduces a `<T as From<f64>>::from(...)` round-trip in an internal helper
- **THEN** the corresponding `f32` property test SHALL fail with a precision discrepancy exceeding the widened tolerance

### Requirement: `cargo build --workspace` succeeds after R0 ships

The workspace SHALL compile cleanly after R0 lands. `deep_causality_physics` and `deep_causality_effects` SHALL temporarily pin their topology call sites to `::<f64>` (tagged `// TEMP: removed by generalize-{physics,effects}-over-realfield`) so the workspace remains green between R0 and the follow-up library change sets.

#### Scenario: Workspace builds with one command

- **WHEN** `cargo build --workspace` is run after R0 is applied
- **THEN** the build SHALL succeed with no errors

#### Scenario: Temporary pins are greppable

- **WHEN** the source is searched for `// TEMP: removed by generalize-`
- **THEN** every pin SHALL be tagged and the tag SHALL name the follow-up change set that will remove it
