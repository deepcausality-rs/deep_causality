## MODIFIED Requirements

### Requirement: Boundary conditions are first-class composable zones with layered dispatch
The solver SHALL model every boundary condition as a `BoundaryZone` term that declares which
solver stage(s) it affects through one hook per stage (metric/Hodge-star overlay, rate source,
projection constraint set, time-dependent inhomogeneous lift, projection flux role, and boundary
time-update), each hook defaulting to a no-op so a zone overrides only the layers it touches. The
solver SHALL accept a set of zones and, at each stage, combine the contributions of the zones that
override that stage. A single uniform `apply()` SHALL NOT be used, because zones act at different
stages of a step (geometry, rate, projection constraint, projection flux, boundary time-update).

Every hook the trait documents SHALL be one the solver actually folds. A hook that is documented as
folded but never called is worse than an absent one: a zone that implements it applies no boundary
condition, silently, and the trait's documentation asserts the opposite. Four of the five `collect_*`
hooks are wired; `collect_constrained_edges` has no call site and no implementor outside the trait
definition, while the no-slip constraint set it appears designed to feed is instead enumerated
directly in `dec_ns_solver/no_slip.rs`. The hook SHALL therefore be wired into the constrained
projection, or removed and its documentation corrected.

Zone composition SHALL be **static** — a typed, monomorphized composition on the HKT foundation
(`deep_causality_haft`), with the solver generic over the composed zone type. Trait objects, `dyn`,
and dynamic dispatch SHALL NOT be used (per the repo's static-dispatch convention).

#### Scenario: A zone contributes only to the stages it declares
- **WHEN** a solver is configured with a set of boundary zones
- **THEN** each zone affects only the solver stages whose hooks it overrides, and is a no-op on every other stage

#### Scenario: Existing boundary conditions reduce bit-identically
- **WHEN** the no-slip wall, moving wall, body force, and immersed-body conditions are expressed as zones and a case (Poiseuille, lid-driven cavity, immersed solid block) is marched
- **THEN** the marched field is bit-identical to the pre-abstraction solver for that case

#### Scenario: A zone implementing a documented hook has an effect
- **WHEN** a zone overrides any hook the trait documents
- **THEN** the solver folds that contribution at the matching stage, and the zone's boundary condition
  is applied

#### Scenario: The documented hook set matches the folded hook set
- **WHEN** the trait's hook documentation is compared against the solver's fold sites
- **THEN** the two agree, with no hook documented as folded that the solver never calls
