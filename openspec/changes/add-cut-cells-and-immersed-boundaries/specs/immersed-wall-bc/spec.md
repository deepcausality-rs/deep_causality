## ADDED Requirements

### Requirement: No-slip and slip on immersed cut faces
The solver SHALL enforce no-slip (Dirichlet) and slip boundary conditions on cut-face
fragments by extending the Stage-3 symmetric restriction and the constrained Leray
projection's constraint set to the edges adjacent to cut faces, with the constraint
direction set by the fragment outward normal. A moving immersed surface SHALL be supported
by prescribing the boundary velocity on the constrained edges, reusing the moving-wall lift.

#### Scenario: No-slip holds on the immersed boundary every step
- **WHEN** a flow is marched with no-slip cut faces over a registered cut geometry
- **THEN** the wall-tangential velocity on the cut-adjacent edges is zero (or the prescribed surface velocity for a moving surface) at every step boundary, and each step output is divergence-free at the solve's exactness

### Requirement: Solver wiring is additive and axis-aligned-consistent
`DecNsSolver` SHALL accept a `CutCellRegistry`, sample a cut-aware CFL bound (the advective
maximum includes cut-adjacent speeds), and seed a field that starts divergence-free and
consistent with the immersed boundary. With an axis-aligned cut registry the results SHALL
match the Stage-3 no-slip solver to rounding, and fully periodic or axis-aligned-wall solves
SHALL be unchanged when no registry is supplied.

#### Scenario: Axis-aligned cut reproduces the Stage-3 wall solver
- **WHEN** a solver is run with a cut registry describing an axis-aligned wall and compared against the Stage-3 wall-bounded solver on the same case
- **THEN** the marched fields agree to rounding
