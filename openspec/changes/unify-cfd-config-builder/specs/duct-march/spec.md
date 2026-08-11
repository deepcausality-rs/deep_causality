## MODIFIED Requirements

### Requirement: A 1-D compressible duct path in the flow DSL

`deep_causality_cfd` SHALL provide an owned `DuctConfig` and a `DuctConfigBuilder` started by
`CfdConfigBuilder::duct::<R>(name)`, carrying the case name, the area profile (a table or the
analytic converging–diverging variant), the inlet stagnation state, the ratio of specific heats, the
back pressure, the grid resolution, and the stop condition. `CfdFlow::march(&config)` SHALL accept it
through the `MarchDispatch` seam — the same entry every other config family uses — and lower it onto
the existing 1-D compressible Euler solver. The
runner SHALL march to a quasi-steady state under the stop condition and return the standard
`Report` carrying the series `"x"`, `"mach_profile"`, and `"pressure_profile"` and the
scalars `"shock_position"` and `"thrust_coefficient"`. A stop condition that expires before
the residual settles SHALL surface as an error, never as a silently unconverged report.

The inlet stagnation state and the stop condition SHALL be set through builder methods
(`inlet(p0, t0)`, `stop(max_steps, residual_tol)`) rather than through public-field structs, and the
owned config's field constructor SHALL be crate-private with all validation in `build()`. The
returned `Report` SHALL be labelled with the case name rather than a fixed literal, so a swept duct
study distinguishes its cases.

#### Scenario: A choked nozzle reports its shock and thrust

- **WHEN** a converging-diverging duct is marched at a back pressure that places a normal
  shock in the diverging section
- **THEN** the report's Mach profile passes through unity at the throat, the shock position
  falls where the pressure profile steepens, and the thrust coefficient is finite and
  positive

#### Scenario: Convergence failure is loud

- **WHEN** the stop condition expires with the residual above its gate
- **THEN** the run returns an error naming the residual and the step budget

#### Scenario: Each swept duct case is named

- **WHEN** a study sweeps several back pressures, each built through `CfdConfigBuilder::duct` with
  its own name
- **THEN** each returned `Report` carries that case's name

#### Scenario: An invalid duct is refused at build

- **WHEN** a duct is configured with a back pressure at or above the stagnation pressure, a throat
  that is not the strict minimum, fewer than the minimum cells, a zero step budget, or a
  non-positive residual tolerance
- **THEN** `build()` returns `PhysicsError::PhysicalInvariantBroken` naming the violated condition
