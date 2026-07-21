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

Every hook the trait documents SHALL be one the solver actually folds, and every hook the solver folds
SHALL be documented. A hook documented as folded but never called is worse than an absent one: a zone
implementing it applies no boundary condition, silently, while the documentation asserts the opposite.
The converse — a hook wired but undocumented — leaves a stage of the abstraction invisible to the
implementor of the next zone.

Zone-supplied constrained edges SHALL compose with the structural constraint set by **union**: a
constrained edge is one pinned to zero rate, so pinning it from two sources is idempotent and there is
no value for the sources to disagree about. The union SHALL be taken after any free-slip un-pin, so a
constraint a zone supplies explicitly outranks a structural relaxation.

Agreement between the documented and folded hook sets SHALL be verified **behaviourally** — by
observing that a zone overriding a hook changes the marched result — rather than by inspecting source
text. A textual check passes on a hook named only in a comment and cannot detect a hook that is folded
but ignored.

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
- **WHEN** a zone overrides any hook the trait documents and a case is marched
- **THEN** the marched result differs from the same case without that zone, so the solver demonstrably
  folds the contribution at the matching stage

#### Scenario: The documented hook set matches the folded hook set
- **WHEN** each documented hook is exercised in turn by a zone that overrides only that hook
- **THEN** every one of them changes the march, with no hook documented as folded that the solver
  never calls and none folded that the trait does not document

#### Scenario: Hooks the solver folds jointly are still isolated
- **WHEN** two hooks cannot be exercised independently because the solver folds them together and
  rejects one without the other — as a prescribed inflow is rejected without an outflow reference
- **THEN** each is isolated by varying it while the other is held fixed, rather than left unverified

#### Scenario: Overlapping constraint sources are idempotent
- **WHEN** a zone supplies constrained edges that the structural constraint set already contains
- **THEN** the marched field is unchanged, the union having pinned each edge once

### Requirement: Time-dependent zone values without solver state
A zone's inhomogeneous lift and boundary time-update hooks SHALL receive the step index and time
step so a zone can supply time-varying boundary values, while the solver remains stateless and the
zone remains immutable configuration; any evolving value (such as a last-good fallback) SHALL be
carried in the monad state, not in the solver or the zone.

#### Scenario: A time-varying boundary value drives the march
- **WHEN** a zone supplies a step-dependent lift value across a march
- **THEN** the boundary value updates each step and neither the solver nor the zone retains mutable simulation state

