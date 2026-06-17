# open-boundary-flux-projection Specification

## Purpose
TBD - created by archiving change add-boundary-zone-abstraction. Update Purpose after archive.
## Requirements
### Requirement: The Leray projection admits nonzero net boundary flux under mixed BCs
The constrained Leray / pressure-Poisson projection SHALL support boundaries tagged with a flux
role — `Closed` (zero normal flux), `Source` (prescribed normal flux), or `Reference` (a pinned
pressure face that lets its normal flux adjust). With at least one `Source` and exactly one
`Reference`, the discrete projection SHALL be non-singular and globally mass-conservative: the
`Reference` face's net flux equals the negative sum of all `Source` fluxes. The operator SHALL be
precision-generic over `R: RealField` with no narrowing to `f64`.

#### Scenario: Prescribed inflow balanced by a reference outflow
- **WHEN** a domain has an inflow face with prescribed normal flux and an outflow face tagged as the pressure reference
- **THEN** the projected field is divergence-free in the interior, the inflow normal flux is held, and the outflow normal flux adjusts so that total mass in equals total mass out

#### Scenario: Unbalanced configuration is rejected
- **WHEN** a domain declares a `Source` face but no `Reference` face
- **THEN** the projection returns an explicit error rather than solving a singular or non-conservative system

### Requirement: Closed-domain reduction is exact
When no boundary reports a flux role other than `Closed`, the net-flux projection SHALL be
bit-identical to the existing closed-domain constrained Leray projection at both the operator and
the marched-solver level.

#### Scenario: No open boundary reproduces the existing solver
- **WHEN** a case with only closed (wall/periodic) boundaries is projected and marched
- **THEN** the result is bit-identical to the pre-change projection (the source and reference code paths are not taken)

