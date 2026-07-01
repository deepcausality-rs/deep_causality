# qtt-codec-2d Specification

## Purpose
TBD - created by archiving change add-cfd-qtt-incompressible-2d. Update Purpose after archive.
## Requirements
### Requirement: 2-D quantized field codec and lifted axis operators

The `tensor_bridge` module SHALL provide a 2-D quantized codec — `quantize_2d` encoding a
`2^Lx × 2^Ly` periodic lattice field as an `(Lx + Ly)`-mode tensor train (x-modes then y-modes,
most-significant-bit first per axis) and `dequantize_2d` its inverse — round-tripping to working
precision and rejecting non-power-of-two extents. It SHALL also provide the periodic axis operators
`gradient_x`, `gradient_y`, `laplacian_2d`, and `divergence`, built by lifting the 1-D shift stencils
with identity blocks (`∂ₓ = ∂ₓ ⊗ I_y`). The bound is real `R: CfdScalar + ConjugateScalar<Real = R>`.

#### Scenario: 2-D field round-trips
- **WHEN** a `2^Lx × 2^Ly` field is `quantize_2d`d then `dequantize_2d`d
- **THEN** the recovered field equals the original to working precision

#### Scenario: Axis derivative acts on the correct axis
- **WHEN** `gradient_x` (resp. `gradient_y`) is applied to a field that varies only along x (resp. y)
- **THEN** the result matches the periodic centered difference along that axis and is ~zero along the
  constant axis

#### Scenario: 2-D Laplacian matches the dense five-point stencil
- **WHEN** `laplacian_2d` is applied to a smooth field
- **THEN** the result matches the periodic five-point Laplacian `(uᵢ₊₁,ⱼ + uᵢ₋₁,ⱼ + uᵢ,ⱼ₊₁ + uᵢ,ⱼ₋₁ −
  4uᵢ,ⱼ)/Δx²` to discretization error

