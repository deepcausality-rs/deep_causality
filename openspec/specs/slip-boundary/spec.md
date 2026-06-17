# slip-boundary Specification

## Purpose
TBD - created by archiving change add-slip-boundaries-and-surface-forces. Update Purpose after archive.
## Requirements
### Requirement: Free-slip / far-field boundary zone
The solver SHALL provide a `SlipWall` boundary zone declaring a lattice face as **free-slip** (a
far-field lateral boundary): zero wall-normal penetration with a **free tangential** velocity (zero
tangential shear), in contrast to the auto-derived no-slip that pins the tangential edges. The zone
SHALL remove its face's wall-tangential edges from the no-slip constraint set (the un-pin seam), so
the no-slip set is `(auto walls ∪ constrained_edges) \ slip_edges`; the viscous operator SHALL give
the zero-shear tangential condition through the boundary-clipped Hodge star. With no face declared
slip, the constraint set is unchanged and the march is bit-identical to the closed/no-slip path.

#### Scenario: Free-slip preserves a uniform tangential flow
- **WHEN** a uniform plug flow tangential to a free-slip wall is marched
- **THEN** it is preserved with no boundary layer forming at that wall, unlike a no-slip wall (which develops a Poiseuille-type profile)

#### Scenario: No slip face declared is bit-identical
- **WHEN** a case declares no `SlipWall` zone
- **THEN** the no-slip constraint set and the marched result are bit-identical to the pre-change solver

