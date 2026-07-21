# wall-hodge-star

## Purpose

Boundary-corrected dual volumes in the cubical Regge Hodge star: dual volumes are clipped at
open (wall) boundaries so the star stays diagonal, positive and SPD-preserving on
wall-bounded lattices, while fully periodic lattices are unchanged.

## Requirements
### Requirement: Boundary-corrected dual volumes on open axes
The cubical Regge geometry's Hodge star SHALL clip dual volumes at open
(wall) boundaries: a cell's dual volume is scaled by `2^{-b}` where `b`
counts its open-axis boundary incidences (wall face → 1, wall edge → 2, 3D
wall corner → 3). The correction SHALL apply across the unit, uniform,
per-axis, and per-edge tiers; fully periodic lattices SHALL be unchanged.
The corrected star SHALL remain diagonal and positive (Euclidean
signature), preserving the SPD structure the CG and energy arguments
require.

#### Scenario: Interior entries unchanged
- **WHEN** the corrected star is computed on an open lattice
- **THEN** entries for cells with no boundary incidence equal the previous (interior) values exactly

#### Scenario: Clip exponents at faces, edges, and corners
- **WHEN** the grade-0 star is computed on an open 3D lattice with uniform spacing `h`
- **THEN** boundary-face vertices carry `h³/2`, boundary-edge vertices `h³/4`, and corner vertices `h³/8`

#### Scenario: Periodic axes are not clipped
- **WHEN** the star is computed on a mixed lattice (periodic-x, wall-y)
- **THEN** only y-boundary incidences contribute clip factors

#### Scenario: Corrected Laplacians stay M-symmetric
- **WHEN** `Δ₀` and `Δ₁` are assembled with the corrected star on a walled lattice
- **THEN** the operators are symmetric in the corrected M-inner product to rounding (pinned by a dedicated test)
