## ADDED Requirements

### Requirement: Prescribed inflow zone
The solver SHALL provide an `Inflow` boundary zone that prescribes a wall-normal Dirichlet
velocity on a tagged boundary face: it contributes the face's normal edges to the projection
constraint set, supplies their prescribed `u·n` as the inhomogeneous lift, and reports
`FluxRole::Source` so the net-flux projection injects the inflow flux into the compatibility
balance. The inflow profile SHALL support a uniform value and a per-edge profile, precision-generic
over `R: RealField`.

#### Scenario: Uniform inflow into a channel
- **WHEN** a uniform `Inflow` is applied to the inlet face of a straight channel that has an outflow face
- **THEN** the steady interior flow is uniform at the prescribed speed and is mass-conservative to the solve tolerance

#### Scenario: Inflow value can be time-dependent
- **WHEN** the inflow lift value varies per step
- **THEN** the prescribed normal velocity updates each step through the zone's time-dependent lift hook
