# qtt-imex-time-integration Specification

## Purpose
TBD - created by archiving change add-cfd-compressible-qtt-marcher. Update Purpose after archive.
## Requirements
### Requirement: IMEX acoustic time step

`deep_causality_cfd` SHALL integrate the compressible system with an **implicit–explicit (IMEX)** scheme: the
slow convective/entropy transport explicit, and the **fast acoustic (pressure) mode implicit** via a
tensor-train linear solve (`solve::linear`, AMEn). The stable timestep SHALL be governed by the convective
CFL, not the acoustic CFL, so a micrometre grid does not force an acoustic-limited timestep.

#### Scenario: Stability beyond the acoustic CFL
- **WHEN** the marcher is run at a timestep above the explicit acoustic CFL but within the convective CFL
- **THEN** the solution stays bounded and stable (a fully explicit scheme at the same timestep diverges)

#### Scenario: Diffusion-CFL honesty
- **WHEN** physical/artificial viscosity is applied
- **THEN** the dissipation is integrated implicitly or kept within its explicit diffusion-CFL limit — naive
  over-thickening (which the `qtt_rank_nonlinear` study showed blows up to full rank) is not used

### Requirement: Conservation-preserving rounding

Because `round` minimizes Frobenius error and is **not** integral-conserving, the marcher SHALL preserve the
discrete conserved quantities across rounding by carrying the conserved totals and applying a low-rank
(rank-1) projection/fixup after each round. The conserved integrals SHALL stay at the rounding floor over a
run.

#### Scenario: Rounding does not leak conserved quantities
- **WHEN** many steps are taken with per-step rounding on a periodic, source-free domain
- **THEN** `∫ρ`, `∫ρu`, `∫ρE` remain conserved to the rounding floor (no secular drift from rounding)

### Requirement: Positivity-preserving formulation

The marcher SHALL keep `ρ`, `p`, internal energy, and species fractions positive — via entropy / log-variable
evolution (so positivity is structural) and/or a positivity limiter — with no negative values produced near
shocks or strong expansions.

#### Scenario: Positivity through a strong expansion
- **WHEN** the marcher is run through a strong rarefaction / near-vacuum state
- **THEN** density, pressure, and internal energy stay strictly positive throughout

