## ADDED Requirements

### Requirement: Staircase no-slip / no-penetration on an immersed body
The solver SHALL enforce immersed no-slip and no-penetration by extending the constrained Leray
projection's constraint set with the immersed body's **solid-incident edge set**
(`CutCellRegistry::solid_incident_edges` — every edge incident to at least one `Solid` cell), unioned
with the axis-aligned wall-tangential set in `NoSlipConstraint::new`. This reuses the existing
constrained projector and the symmetric `P_S Δ₁ P_S` restriction with no new solver machinery: the
registry is read off the metric, like the cut star. The partial blockage of `Cut` cells is carried by
the cut Hodge star; the body interior and the fluid↔solid interface are pinned to zero (no flow inside
the body, zero tangential and normal velocity at the staircase boundary).

*Deferred refinements (documented):* slip (tangential-only) on cut faces, moving immersed surfaces
(prescribed values through the existing lift), and the **aperture-resolved** sub-cell cut-face no-slip
(the fragment-normal-directed constraint at the wetted surface). The aperture-resolved refinement is
shipped separately as the `aperture-resolved-noslip` capability.

#### Scenario: No-slip holds on the immersed boundary every step
- **WHEN** a flow is marched with a registered immersed `Solid` body
- **THEN** the velocity on every solid-incident edge is zero (no-slip + no-penetration) at every step
  boundary, the flow goes around the body, and each step output is divergence-free at the solve's
  exactness

### Requirement: Solver wiring is additive and axis-aligned-consistent
`DecNsSolver` SHALL read a `CutCellRegistry` off the geometry, sample a cut-aware CFL bound (the
advective maximum includes cut-adjacent speeds), and seed a field that starts divergence-free and
consistent with the immersed boundary. With an axis-aligned cut registry the results SHALL match the
Stage-3 no-slip solver to rounding, and fully periodic or axis-aligned-wall solves SHALL be unchanged
when no registry is supplied.

#### Scenario: Axis-aligned cut reproduces the Stage-3 wall solver
- **WHEN** a solver is run with a cut registry describing an axis-aligned `Solid` layer coincident
  with a vertex-collocated wall, compared against the Stage-3 wall-bounded solver on the same case
- **THEN** the marched fields agree to rounding (the reduced-height Poiseuille parabola is reproduced
  exactly)

#### Scenario: Empty registry is bit-identical
- **WHEN** the solver runs with no registry, or an empty `CutCellRegistry`
- **THEN** the marched state is bit-identical to the Stage-3 result
