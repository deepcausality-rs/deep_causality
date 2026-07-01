# qtt-incompressible-2d Specification

## Purpose
TBD - created by archiving change add-cfd-qtt-incompressible-2d. Update Purpose after archive.
## Requirements
### Requirement: Periodic 2-D incompressible Navier–Stokes tensor-train marcher

The `solvers/qtt` module SHALL provide `QttIncompressible2d`, a `Marcher` advancing the periodic 2-D
incompressible Navier–Stokes equations with the velocity pair `(u, v)` held as tensor trains. Each step
SHALL form the nonlinear convection `u·∇u` (via train Hadamard products) plus viscous diffusion,
advance, **recompress**, and apply the [pressure projection](../qtt-projection/spec.md) — so the field
stays divergence-free and low-rank. It SHALL validate against the analytic Taylor–Green vortex.

#### Scenario: Matches the Taylor–Green vortex decay
- **WHEN** the marcher is initialized with the 2-D Taylor–Green vortex and advanced over a fixed horizon
- **THEN** the velocity field matches the analytic decaying solution within discretization + truncation
  error

#### Scenario: Stays divergence-free
- **WHEN** the marcher advances many steps
- **THEN** the divergence of the velocity field remains at or below the projection tolerance each step

#### Scenario: Bounded rank under recompression
- **WHEN** the marcher advances with a fixed round policy
- **THEN** the velocity trains' bond dimensions stay bounded, and tightening the round tolerance reduces
  the error toward the discretization floor

