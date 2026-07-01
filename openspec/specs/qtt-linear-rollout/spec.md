# qtt-linear-rollout Specification

## Purpose
TBD - created by archiving change add-cfd-qtt-tensor-bridge. Update Purpose after archive.
## Requirements
### Requirement: Quasi-1D linear advection–diffusion rollout

The `solvers/qtt` module SHALL provide `QttLinear1d`, a tensor-train marcher for the periodic linear
advection–diffusion equation `∂u/∂t = −c·∂ₓu + ν·∂²ₓu`, advancing a quantized field by applying the
assembled `gradient` / `laplacian` MPOs and **recompressing every step** with a caller-supplied round
policy. It SHALL land behind the existing `FluidTheory` / `Marcher` seam (its state wrapping a
`CausalTensorTrain<R>`), so it composes with the marching machinery unchanged. The rollout SHALL conserve
the field mean for pure advection and SHALL match the analytic solution to discretization + truncation
error.

#### Scenario: Matches the analytic advection–diffusion solution
- **WHEN** a smooth periodic initial profile is marched with `QttLinear1d` over a fixed horizon
- **THEN** the dequantized result matches the analytic advection–diffusion solution within the combined
  discretization and round-tolerance error

#### Scenario: Bounded rank under recompression
- **WHEN** the rollout marches many steps with a fixed round policy
- **THEN** the field train's bond dimension stays bounded (recompression controls rank), and tightening
  the round tolerance reduces the error toward the discretization floor

#### Scenario: Pure advection preserves the mean
- **WHEN** the rollout marches with `ν = 0`
- **THEN** the field mean is conserved to working precision across steps

