# qtt-differential-operators Specification

## Purpose
TBD - created by archiving change add-cfd-qtt-tensor-bridge. Update Purpose after archive.
## Requirements
### Requirement: Periodic grid-shift MPO

The `tensor_bridge` module SHALL build the periodic grid-shift operator `S₊` (cyclic `+1` with carry) on
a `2^L` grid as a low-bond MPO via `CausalTensorTrainOperator::from_cores`, with `S₋` its transpose.
Applying `S₊` (resp. `S₋`) to a quantized field SHALL cyclically shift it by one grid point, matching the
dense cyclic-shift matrix to working precision.

#### Scenario: Shift MPO matches the cyclic-shift matrix
- **WHEN** `S₊` is applied to a quantized field on a `2^L` grid
- **THEN** the dequantized result equals the field shifted cyclically by one point, to working precision

#### Scenario: S₋ inverts S₊
- **WHEN** `S₊` then `S₋` are applied to a field
- **THEN** the result equals the original field to working precision

### Requirement: Finite-difference stencil assembly

The module SHALL assemble periodic finite-difference operators from the shift MPOs using the operator
algebra: a centered first derivative `gradient = (S₊ − S₋)/(2Δx)` and a second derivative
`laplacian = (S₊ + S₋ − 2·I)/Δx²`, each recompressed with `round`. Densifying an assembled operator SHALL
equal the corresponding standard periodic finite-difference matrix to working precision.

#### Scenario: Assembled Laplacian matches the dense stencil
- **WHEN** the `laplacian` MPO is assembled at grid spacing `Δx` and densified
- **THEN** it equals the periodic second-difference matrix `(uᵢ₊₁ + uᵢ₋₁ − 2uᵢ)/Δx²` to working precision

#### Scenario: Gradient annihilates a constant and differentiates a ramp
- **WHEN** `gradient` is applied to a constant field, and to a smooth periodic profile
- **THEN** the constant maps to ~zero and the profile maps to its analytic derivative within discretization error

