# boundary-zone-abstraction Specification

## Purpose
TBD - created by archiving change add-boundary-zone-abstraction. Update Purpose after archive.
## Requirements
### Requirement: Boundary conditions are first-class composable zones with layered dispatch
The solver SHALL model every boundary condition as a `BoundaryZone` term that declares which
solver stage(s) it affects through one hook per stage (metric/Hodge-star overlay, rate source,
projection constraint set, time-dependent inhomogeneous lift, projection flux role, and boundary
time-update), each hook defaulting to a no-op so a zone overrides only the layers it touches. The
solver SHALL accept a set of zones and, at each stage, combine the contributions of the zones that
override that stage. A single uniform `apply()` SHALL NOT be used, because zones act at different
stages of a step (geometry, rate, projection constraint, projection flux, boundary time-update).

Zone composition SHALL be **static** — a typed, monomorphized composition on the HKT foundation
(`deep_causality_haft`), with the solver generic over the composed zone type. Trait objects, `dyn`,
and dynamic dispatch SHALL NOT be used (per the repo's static-dispatch convention).

#### Scenario: A zone contributes only to the stages it declares
- **WHEN** a solver is configured with a set of boundary zones
- **THEN** each zone affects only the solver stages whose hooks it overrides, and is a no-op on every other stage

#### Scenario: Existing boundary conditions reduce bit-identically
- **WHEN** the no-slip wall, moving wall, body force, and immersed-body conditions are expressed as zones and a case (Poiseuille, lid-driven cavity, immersed solid block) is marched
- **THEN** the marched field is bit-identical to the pre-abstraction solver for that case

### Requirement: Time-dependent zone values without solver state
A zone's inhomogeneous lift and boundary time-update hooks SHALL receive the step index and time
step so a zone can supply time-varying boundary values, while the solver remains stateless and the
zone remains immutable configuration; any evolving value (such as a last-good fallback) SHALL be
carried in the monad state, not in the solver or the zone.

#### Scenario: A time-varying boundary value drives the march
- **WHEN** a zone supplies a step-dependent lift value across a march
- **THEN** the boundary value updates each step and neither the solver nor the zone retains mutable simulation state

