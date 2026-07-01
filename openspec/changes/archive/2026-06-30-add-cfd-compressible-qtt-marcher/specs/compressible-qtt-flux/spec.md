<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: Conservative compressible state and flux

`deep_causality_cfd` SHALL represent the compressible state in **conservative variables** `(ρ, ρu, ρv, ρw,
ρE, {ρY_s})` as tensor trains, and SHALL compute the conservative flux divergence `∇·F(U)` in tensor-train
form using the `qtt-codec-3d` / body-fitted operators. The scheme SHALL be **conservative** (the discrete flux
divergence telescopes), so captured/fitted jumps satisfy the Rankine–Hugoniot conditions.

#### Scenario: Conservation of total mass / momentum / energy
- **WHEN** the compressible update is marched on a periodic domain with no sources
- **THEN** the integrated `∫ρ`, `∫ρu`, `∫ρE` are conserved to the rounding floor over the run

#### Scenario: Free-stream preservation
- **WHEN** a uniform free-stream state is marched
- **THEN** it is preserved exactly (the flux of a constant state is zero), including in the body-fitted
  coordinate (metric identities hold discretely)

### Requirement: Approximate Riemann flux

The flux SHALL use an approximate Riemann solver (Rusanov/local Lax–Friedrichs as the baseline, HLLC as the
sharper option) expressed through the existing tensor-train operations, with a wave-speed estimate from the
state. On a smooth field the flux SHALL reduce to a centred flux to the scheme's order.

#### Scenario: Sod shock tube matches the exact Riemann solution
- **WHEN** the 1-D compressible flux is marched on the Sod shock-tube initial data
- **THEN** the density / velocity / pressure profiles match the exact Riemann solution within a recorded
  tolerance, with the shock, contact, and expansion at the correct speeds

### Requirement: EOS pressure closure via TT-cross

The marcher SHALL evaluate the pressure and temperature closure (`p, T` from density, internal energy, and
species fractions — ideal-gas baseline; the Tier-A two-temperature mixture EOS as the reacting option)
pointwise on the tensor-train state via TT-cross (`apply_nonlinear` / `cross`) at a controlled rank on smooth
fields.

#### Scenario: EOS recovers pressure on a smooth field
- **WHEN** the EOS closure is evaluated on a smooth density/energy train
- **THEN** the decoded pressure matches the pointwise analytic EOS within tolerance, at bounded rank
