# no-slip-viscous

## Purpose

No-slip enforcement on wall-tangential edges, and the symmetry the constrained viscous
operator must retain under it — the property the CG solve and the energy argument both
depend on.

## Requirements
### Requirement: No-slip constraints on wall-tangential edges
The solver stack SHALL enforce no-slip on wall-bounded lattices by
constraining 1-form coefficients on wall-tangential edges to zero. Because
the plain Leray projector and the coordinate constraint do not commute,
the constraint SHALL be enforced through the **constrained Leray
projection** — the M-orthogonal projection onto the intersection of the
divergence-free subspace with the no-slip subspace (masked edge masses in
the grade-0 solve, masked gradient correction) — applied to every
projected rate evaluation, the seeding projection, and the step re-entry,
with an explicit chain stage re-asserting the exact constraint values at
each step boundary. Wall-normal flux is the projection's Neumann
condition, not an edge constraint (boundary-crossing normal edges do not
exist on an open lattice). Prescribed nonzero tangential wall values (a
moving wall) SHALL be supported as a lift re-applied after each
projection; the projector ignores constrained-edge input values, so the
lift commutes with it exactly.

#### Scenario: Tangential wall edges are exactly zero
- **WHEN** a wall-bounded march advances any number of steps
- **THEN** every wall-tangential edge coefficient is exactly zero at every step boundary, and the divergence residual is simultaneously at the solve's exactness

#### Scenario: Moving-wall values are held exactly
- **WHEN** a wall carries a prescribed tangential velocity through the moving-wall lift
- **THEN** the lift edges hold their prescribed edge integrals exactly at every step boundary while the remaining wall edges stay pinned to zero

#### Scenario: Periodic solver path unchanged
- **WHEN** the solver runs on a fully periodic manifold
- **THEN** the constrained-edge set is empty and results are bit-identical to the pre-change periodic path

### Requirement: Constrained viscous operator symmetry
The effective viscous operator on wall-bounded lattices SHALL be
symmetric in the corrected M-inner product; it is the subspace
restriction `P_S Δ₁ P_S` of the boundary-corrected grade-1 Laplacian to
the no-slip subspace. (Realized as the symmetric restriction rather than
ghost-coefficient row surgery: the diagonal mass and the coordinate
projector make `M₁ · P_S Δ₁ P_S` symmetric whenever `M₁ Δ₁` is, which the
wall-hodge-star capability pins.)

#### Scenario: Boundary rows are M-symmetric
- **WHEN** the constrained viscous operator is assembled on walled 2D and 3D lattices
- **THEN** M-symmetry holds to rounding

#### Scenario: Couette diffusion sanity
- **WHEN** a march runs between two walls with one moving-lid tangential boundary value
- **THEN** the velocity profile relaxes to the linear Couette solution **exactly at stationarity rounding** — the linear shear lies in the discrete viscous kernel and the convective residual of an x-uniform shear is a discrete gradient the constrained projector removes (a stronger result than the originally drafted discretization-order gate)
