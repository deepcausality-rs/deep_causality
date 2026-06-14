## ADDED Requirements

### Requirement: Convective outflow zone
The solver SHALL provide an `Outflow` boundary zone that advances its boundary edges by a
convective / zero-gradient time-update (upwinded extrapolation from the interior) before the
projection each step, and reports `FluxRole::Reference` so the projection lets its normal flux
adjust to conserve mass. The outflow SHALL NOT be pinned in the projection constraint set (which
would over-specify it and reflect waves).

#### Scenario: Steady channel with inflow and outflow
- **WHEN** a channel driven by a uniform inflow is marched with a convective outflow far downstream
- **THEN** the flow reaches a steady uniform state with mass in equal to mass out each step and no spurious reflection corrupting the interior

#### Scenario: Outflow is not over-specified
- **WHEN** an outflow zone is configured
- **THEN** its boundary edges are advanced by the time-update and made divergence-compatible by the projection's reference role, rather than pinned to a fixed value
