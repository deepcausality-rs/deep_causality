# iso-tensor-sparse Specification

## Purpose
TBD - created by archiving change implement-isomorphism. Update Purpose after archive.
## Requirements
### Requirement: `CausalTensor<F>` (rank-2) -> `CsrMatrix<F>` via Tier 1 `TryFrom`

The crate `deep_causality_sparse` SHALL expose `impl<F> TryFrom<CausalTensor<F>> for CsrMatrix<F>` for any `F: Clone + Copy + Zero + PartialEq`. The conversion SHALL iterate the tensor's data in row-major order and emit a triplet for each non-zero value. The conversion SHALL return `Err(CsrFromTensorError { rank })` (a typed error exposed from the crate root) when the input tensor has rank other than 2. `From` is NOT used because the conversion is intrinsically partial; only rank-2 tensors are matrices, and `From` is reserved for total conversions.

#### Scenario: Forward TryFrom converts dense rank-2 tensor into sparse matrix

- **WHEN** a downstream user invokes `CsrMatrix::<f64>::try_from(tensor)` for a rank-2 `CausalTensor<f64>` with shape `[2, 3]` and data `[1.0, 0.0, 0.0, 4.0, 0.0, 6.0]`
- **THEN** the result SHALL be `Ok(matrix)` where `matrix.shape() == (2, 3)`
- **AND** the stored triplets SHALL be `(0, 0, 1.0)`, `(1, 0, 4.0)`, `(1, 2, 6.0)` in some canonical order
- **AND** no zero values SHALL be stored

#### Scenario: Forward TryFrom returns Err on rank ≠ 2

- **WHEN** a downstream user invokes `CsrMatrix::try_from(tensor)` for a tensor with rank 0, 1, 3, or higher
- **THEN** the result SHALL be `Err(CsrFromTensorError { rank })` with `rank` set to the actual input rank
- **AND** the call SHALL NOT panic

### Requirement: `CsrMatrix<F>` -> `CausalTensor<F>` via Tier 2 `Iso<CsrMatrix<F>, CausalTensor<F>>`

The crate `deep_causality_sparse` SHALL expose `impl<F> Iso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F>` for any `F: Clone + Copy + Zero + PartialEq`. The `to_target` method SHALL materialise a rank-2 dense `CausalTensor<F>` of the matching shape, populating non-zero entries from the triplets and leaving other positions at `F::zero()`. The `to_source` method SHALL delegate to the fallible forward `TryFrom` impl via `.expect(...)`; the Tier 2 trait surface is by contract infallible, so non-rank-2 inputs SHALL panic at this layer with a message directing callers to `TryFrom` for graceful failure. An inherent method `CsrMatrix::to_dense(self) -> CausalTensor<F>` SHALL be provided as an ergonomic alias for `<Self as Iso<CsrMatrix<F>, CausalTensor<F>>>::to_target(self)`.

#### Scenario: to_target materialises a dense tensor from sparse matrix

- **WHEN** a downstream user invokes `sparse.to_dense()` for a `CsrMatrix<f64>` with shape `(2, 3)` and triplets `(0, 0, 1.0), (1, 0, 4.0), (1, 2, 6.0)`
- **THEN** the resulting `CausalTensor<f64>` SHALL have shape `[2, 3]`
- **AND** its data SHALL be `[1.0, 0.0, 0.0, 4.0, 0.0, 6.0]`

#### Scenario: Round-trip holds across both directions

- **WHEN** the test suite runs `assert_witness_iso_round_trip::<CsrMatrix<f64>, CsrMatrix<f64>, CausalTensor<f64>>(sparse, dense)` with matching independent inputs
- **THEN** the assertion SHALL pass in both directions

### Requirement: No algebraic marker subtraits

The iso between `CausalTensor<F>` and `CsrMatrix<F>` SHALL NOT implement `GroupIso`, `RingIso`, `FieldIso`, `AlgebraIso`, or `DivisionAlgebraIso`. Neither type currently implements the corresponding algebraic-structure traits from `deep_causality_num`; adding the marker impls would not type-check.

#### Scenario: No marker impls exist

- **WHEN** a reviewer greps the `deep_causality_sparse` source for `impl.*Iso<CsrMatrix.*CausalTensor`
- **THEN** only the base `Iso<S, T>` impl SHALL appear
- **AND** no `GroupIso`, `RingIso`, `FieldIso`, `AlgebraIso`, or `DivisionAlgebraIso` impls SHALL exist for this type pair

