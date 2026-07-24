# spectral-diffusion

## Purpose

An opt-in spectral treatment of the viscous term on fully periodic lattices, replacing the
stencil Laplacian with an exact per-wavenumber factor where the geometry permits it.

## Requirements
### Requirement: Opt-in spectral viscous term on fully periodic lattices
On fully periodic uniform Euclidean lattices, the solver SHALL offer an
opt-in evaluation of the viscous term `νΔ₁u♭` by per-edge-family real FFT:
each of the `D` edge families is a shifted torus sub-lattice whose
`Δ₁`-block diagonalizes with the lattice eigenvalues
`λ_k = Σ_d (2 − 2·cos(2π·k_d/N_d)) / h_d²`. The march's time integration
SHALL be unchanged (per-stage drop-in). The option SHALL be off by default
and SHALL NOT become the default unless the validation ladder reproduces
the generic path's observed convergence orders.

#### Scenario: Spectral viscous term matches the operator
- **WHEN** the spectral viscous evaluation and `laplacian(1)` composition are applied to randomized 1-forms on 2D and 3D tori (including anisotropic spacings)
- **THEN** results agree at rounding level for the precision, not merely at a solve tolerance

#### Scenario: Ladder equivalence gates the default
- **WHEN** the 2D Taylor–Green convergence table runs with spectral diffusion enabled
- **THEN** observed orders match the generic path; a mismatch keeps the option opt-in

#### Scenario: Dispatch boundary
- **WHEN** a solver with spectral diffusion enabled is constructed over a mixed-periodicity manifold
- **THEN** construction rejects the option with a typed error (the spectral viscous path is periodic-only)
