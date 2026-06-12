## ADDED Requirements

### Requirement: No-slip constraints on wall-tangential edges
The solver stack SHALL enforce no-slip on wall-bounded lattices by
constraining 1-form coefficients on wall-tangential edges to zero, applied
as a typed chain stage after each projected rate application and at
seeding. Wall-normal flux is the projection's Neumann condition, not an
edge constraint (boundary-crossing normal edges do not exist on an open
lattice).

#### Scenario: Tangential wall edges are exactly zero
- **WHEN** a wall-bounded march advances any number of steps
- **THEN** every wall-tangential edge coefficient is exactly zero at every step boundary

#### Scenario: Periodic solver path unchanged
- **WHEN** the solver runs on a fully periodic manifold
- **THEN** the no-slip stage is absent and results are bit-identical to the pre-change periodic path

### Requirement: Mirror-consistent viscous boundary rows
The grade-1 viscous operator on wall-bounded lattices SHALL use boundary
rows consistent with the mirror (ghost) condition for interior edges
adjacent to a wall, expressed without ghost storage by folding the
reflection into the stencil coefficients, and the constrained operator
SHALL remain symmetric in the corrected M-inner product.

#### Scenario: Boundary rows are M-symmetric
- **WHEN** the constrained viscous operator is assembled on walled 2D and 3D lattices
- **THEN** M-symmetry holds to rounding

#### Scenario: Couette diffusion sanity
- **WHEN** a pure-diffusion march (no advection) runs between two walls with one moving-lid tangential boundary value
- **THEN** the velocity profile converges to the linear Couette solution at the discretization's order
