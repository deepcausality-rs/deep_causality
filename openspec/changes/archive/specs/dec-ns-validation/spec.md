# dec-ns-validation

The validation ladder of `cfd-gap.md` §7 items 4–8: analytic Taylor–Green
rungs, inviscid invariants, and the double shear layer in CI; the Re-1600
flagship as an example program.

## ADDED Requirements

### Requirement: 2D Taylor–Green decay with convergence table (CI)

The test suite SHALL march the 2D Taylor–Green vortex on `square_torus`
lattices of size `[8, 16, 32]` and assert (a) per-grid agreement of the
kinetic-energy envelope with the analytic `E(t) = E(0)·exp(−4νt)` within a
precision-dependent tolerance, and (b) an observed spatial convergence
order of at least `1.9` at f64 on the energy-envelope error. f32 SHALL be
gated at a looser documented tolerance; Float106 at the f64 gate. The test
SHALL document that the projection splitting bounds temporal order, so the
gate is on spatial refinement at fixed CFL.

#### Scenario: Energy envelope tracks the analytic decay

- **WHEN** the 32² run completes at f64 with a CFL-safe `dt`
- **THEN** the relative energy error at the final time is within the
  documented tolerance and the per-grid errors shrink at order ≥ 1.9

#### Scenario: All three precisions complete the ladder

- **WHEN** the same ladder runs at f32, f64, and Float106
- **THEN** each passes its documented gate, and no test weakens the f64
  gate to accommodate f32

### Requirement: 2D-in-3D Taylor–Green exercises every 3D path (CI)

The test suite SHALL march the `w = 0` Taylor–Green field on `cubic_torus`
lattices and assert the same analytic energy envelope, so the 3D wedge,
star, interior product, projection, and CFL paths are all exercised while
a closed-form answer exists. The vertical velocity component SHALL remain
zero to projection tolerance throughout the march.

#### Scenario: 3D march reproduces the 2D envelope

- **WHEN** the 2D-in-3D case runs on a `16³` torus at f64
- **THEN** the energy envelope matches `exp(−4νt)` within the documented
  tolerance and `max|w|` stays at or below the projection tolerance

### Requirement: Inviscid invariants are conserved (CI)

The test suite SHALL march a 3D state with `ν = 0` and no body force and
assert bounded relative drift of kinetic energy and helicity over the
documented horizon, with the bound recorded in the test (measured, not
assumed, per the design). Energy drift SHALL also be asserted in 2D where
helicity does not exist.

#### Scenario: Energy and helicity drift stay within the recorded bound

- **WHEN** the inviscid 3D Taylor–Green runs for the documented number of
  steps at f64
- **THEN** `|E(T) − E(0)| / E(0)` and the corresponding helicity drift are
  at or below the recorded bounds

### Requirement: Double shear layer rolls up with 2D conservation character (CI)

The test suite SHALL march the 2D double shear layer (thin tanh layers
with a small sinusoidal cross-stream perturbation, Brown–Minion form) on a
periodic square lattice at one modest resolution, f64 only, and assert
three structural gates: (a) the cross-stream kinetic energy grows from its
perturbation seed by at least one order of magnitude before the horizon
(the roll-up witness); (b) at `ν > 0`, total kinetic energy and enstrophy
are monotonically non-increasing within a documented tolerance, and every
sampled state is divergence-free at projection tolerance; (c) the existing
Q-criterion kernel from `kernels/fluids`, fed by a test-side
central-difference gradient of the `sharp`-recovered field, reports
positive-Q (vortex-core) cells in the rolled-up state that are absent at
`t = 0`.

#### Scenario: Perturbation energy grows by an order of magnitude

- **WHEN** the shear layer marches to the documented horizon at f64
- **THEN** the cross-stream kinetic energy at the final time is at least
  ten times its initial value

#### Scenario: 2D decay character holds through roll-up

- **WHEN** the same run is sampled at the documented cadence
- **THEN** kinetic energy and enstrophy never increase beyond the
  documented tolerance between samples, and each sampled state's
  divergence residual stays at projection tolerance

#### Scenario: Vortex cores appear where none existed

- **WHEN** the Q-criterion kernel evaluates the initial and final sampled
  states
- **THEN** the initial state reports no positive-Q cells above the
  documented threshold and the final state reports at least one

### Requirement: Re-1600 Taylor–Green as an example program (not CI)

An example binary in `examples/avionics_examples/` (beside the existing
`cfd_taylor_green` harness it extends) SHALL run the standard 3D
Taylor–Green vortex at `Re = 1600` with the grid size as a parameter
(default small; 64³–128³ documented as the reporting resolutions), printing
the time series of kinetic energy and dissipation rate `−dE/dt` as CSV on
stdout for comparison against the published DNS reference curve
(`references.md`). CI SHALL NOT execute this binary; the library tests gate
correctness, the example produces the recognizable artifact.

#### Scenario: Example produces the dissipation time series

- **WHEN** the example binary is run at a small resolution during review
- **THEN** it completes without error and emits monotone time, energy, and
  dissipation columns covering the configured horizon

### Requirement: Solver-path coverage discipline

Every public function, error path, and alternative branch introduced by
this change SHALL be exercised by tests, with any genuinely unreachable
arm (the D3 construction-validated unwraps) documented inline as a coverage
exemption in the Stage 0 tradition. The cross-validation, march, guard,
diagnostic, and seeding paths SHALL each have dedicated failure-path tests,
not only happy-path tests.

#### Scenario: Coverage audit finds no undocumented gap

- **WHEN** `cargo llvm-cov` runs over the new module after the suite
- **THEN** uncovered lines are exactly the documented exemptions
