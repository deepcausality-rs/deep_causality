# iso-multifield-tensor Specification

## Purpose
TBD - created by archiving change implement-isomorphism. Update Purpose after archive.
## Requirements
### Requirement: `CausalMultiField<T>` <-> tensor-carrier tuple round-trips byte-identically

The crate `deep_causality_multivector` SHALL expose bidirectional conversion between `CausalMultiField<T>` and its underlying carrier shape `(CausalTensor<T>, Metric, [T; 3], [usize; 3])`. The forward direction `From<CausalMultiField<T>> for (CausalTensor<T>, Metric, [T; 3], [usize; 3])` SHALL unpack the multifield's four fields without copying allocated data. The reverse direction `From<(CausalTensor<T>, Metric, [T; 3], [usize; 3])> for CausalMultiField<T>` SHALL pack the tuple back into a multifield without validation (the caller is responsible for shape consistency). The pair SHALL satisfy `Iso<CausalMultiField<T>, (CausalTensor<T>, Metric, [T; 3], [usize; 3])>` via the `StandardIso<S, T>` blanket impl. No algebraic marker subtraits apply (neither `CausalMultiField` nor the tuple carrier is a `Group`/`Ring`/`Field`).

#### Scenario: Forward From unpacks multifield into tuple

- **WHEN** a downstream user invokes `<(_, _, _, _)>::from(multifield)` for a `CausalMultiField<f64>` value
- **THEN** the resulting tuple's first element SHALL be the multifield's tensor data
- **AND** the second element SHALL be the multifield's metric
- **AND** the third element SHALL be the multifield's grid spacing `[dx, dy, dz]`
- **AND** the fourth element SHALL be the multifield's grid shape `[Nx, Ny, Nz]`

#### Scenario: Reverse From packs tuple into multifield

- **WHEN** a downstream user invokes `CausalMultiField::<f64>::from((tensor, metric, dx, shape))`
- **THEN** the resulting multifield's fields SHALL equal the tuple components in order

#### Scenario: Round-trip via StandardIso witness

- **WHEN** the test suite runs `assert_witness_iso_round_trip::<StandardIso<CausalMultiField<f64>, _>, CausalMultiField<f64>, _>` with a representative multifield and its matching tuple
- **THEN** the assertion SHALL pass in both directions

