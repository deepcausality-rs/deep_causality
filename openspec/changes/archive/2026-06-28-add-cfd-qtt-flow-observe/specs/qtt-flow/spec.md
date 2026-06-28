## ADDED Requirements

### Requirement: QTT marching config container and builder

The crate SHALL provide an owned `QttMarchConfig<R>` container and a `QttMarchConfigBuilder` that holds
the power-of-two grid (`Lx, Ly, dx, dy`), the solver parameters (`dt`, kinematic viscosity, round
policy), the owned seed velocity fields `(u0, v0)`, the march-stop, and the observe set. The builder
SHALL materialize the seed from a closure over the grid (or the analytic Taylor–Green vortex) **at build
time**, and SHALL reject a grid that is not `2^Lx × 2^Ly` or seed fields that do not match it.

#### Scenario: Builds a runnable config from a seed closure
- **WHEN** a grid, solver parameters, a seed closure, a stop, and an observe set are supplied
- **THEN** the builder produces a `QttMarchConfig` whose owned seed fields are the closure evaluated over
  the grid

#### Scenario: Rejects a non-power-of-two grid or mismatched seed
- **WHEN** the grid is not `2^Lx × 2^Ly`, or a supplied seed field's shape does not match the grid
- **THEN** the builder returns a dimension-mismatch error

### Requirement: CfdFlow QTT marching pipeline

`CfdFlow` SHALL provide a `qtt_march(&config)` entry — parallel to `march` — returning a runnable pipeline
that borrows the config, drives `QttIncompressible2d` over the configured horizon, samples the enabled
observables each step into an owned `Report<R>`, and exposes the dequantized final `(u, v)` fields on the
report. It SHALL support a fixed-step and a steady-state (kinetic-energy plateau) stop reusing
`MarchStop<R>`, a per-step hook via `run_with` (a cheap `QttStepView` exposing the step, time, the `(u, v)`
trains, and the TT-native diagnostics), and counterfactual overrides of the seed / stop / observe set.
The pipeline SHALL add no numerics: its result SHALL match the direct `QttIncompressible2d` driver for the
same seed, horizon, and round policy.

#### Scenario: Runs and returns a labeled report
- **WHEN** a QTT march is composed with the kinetic-energy, divergence, and bond observables and run to a
  fixed horizon
- **THEN** the returned `Report` carries one series per enabled observable and the dequantized final
  `(u, v)` fields, and the kinetic-energy series matches the analytic Taylor–Green decay within
  discretization + truncation error

#### Scenario: Matches the direct driver
- **WHEN** the same seed, horizon, and round policy are run through `CfdFlow::qtt_march` and through
  `QttIncompressible2d::run` directly
- **THEN** the final fields are identical

#### Scenario: Steady-state stop terminates on the energy plateau
- **WHEN** the march is configured with a steady-state stop and the kinetic energy stops changing within
  tolerance
- **THEN** the march terminates before the maximum step count

#### Scenario: Per-step hook observes each step
- **WHEN** the march is run with a per-step hook
- **THEN** the hook is invoked once per completed step with a view exposing the step index, elapsed time,
  and the current diagnostics, and the final report is identical to the hookless run
