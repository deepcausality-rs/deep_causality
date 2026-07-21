# dec-ns-validation

## Purpose

The validation ladder of `cfd-gap.md` §7 items 4–8: analytic Taylor–Green
rungs, inviscid invariants, and the double shear layer in CI; the Re-1600
flagship as an example program.

## Requirements

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

An example binary in `deep_causality_cfd/verification/` SHALL run the standard 3D
Taylor–Green vortex at `Re = 1600` (the `dec_taylor_green_re1600_verification` example, beside the
`mms_taylor_green_verification` harness it extends) with the grid size as a parameter
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

Every public function, error path, and alternative branch SHALL be exercised
by tests, with any genuinely unreachable
arm (the D3 construction-validated unwraps) documented inline as a coverage
exemption in the Stage 0 tradition. The cross-validation, march, guard,
diagnostic, and seeding paths SHALL each have dedicated failure-path tests,
not only happy-path tests.

#### Scenario: Coverage audit finds no undocumented gap

- **WHEN** `cargo llvm-cov` runs over the new module after the suite
- **THEN** uncovered lines are exactly the documented exemptions

### Requirement: Poiseuille channel rung (CI)
The validation ladder SHALL include body-force-driven laminar Poiseuille
flow (periodic-x, wall-y) marched to steady state, compared against the
exact parabolic profile over a refinement ladder — the analytic-first
gate for the wall substrate (corrected star, Neumann projection, no-slip
rows) before any reference-data comparison.

#### Scenario: Profile is exact over the refinement ladder
- **WHEN** the steady-state centerplane profile error is measured over the refinement ladder at f64
- **THEN** the profile reproduces the exact parabola at rounding on every rung — with vertex-collocated walls the Dirichlet rows sit exactly on the boundary and the 3-point viscous stencil is exact on quadratics, while the convective term of an x-uniform shear vanishes under the constrained projection (a stronger result than the originally drafted ≥ 1.9 observed-order gate, which has no resolvable h-dependent error left to fit)

#### Scenario: Steady state is wall-consistent
- **WHEN** the Poiseuille march reaches its steady-state criterion
- **THEN** wall-tangential edges are exactly zero and the divergence residual is at the solve's exactness

### Requirement: Lid-driven cavity rung (coarse CI + example)
The ladder SHALL include the Re-1000 lid-driven cavity, split by cost
per the tests-are-fast / examples-verify division: a single coarse rung
in CI compared against the Ghia et al. (1982) centerline tables with a
pinned RMSE gate at a short spin-up horizon (tests stay fast), and an
example program carrying the thorough verification — the
refinement-trend mode (coarse → finer RMSE strictly decreasing at
time-converged horizons, gated, nonzero exit on violation) and the
full-resolution run emitting centerline CSVs and the detected
vortex-center table (primary and corner eddies) against Ghia's values.

The pinned RMSE bounds carry headroom measured from their own pinning run and are therefore regression
tripwires, not agreement claims against Ghia; they SHALL be labelled `tripwire` per the evidence-class
requirement. The committed baseline artifact SHALL be a complete run carrying the RMSE, the vortex-center
table and the verdict line, and its header configuration SHALL match the configuration whose numbers the
harness and `verification/README.md` report.

#### Scenario: Coarse cavity gates in CI
- **WHEN** the coarse cavity rung completes in CI
- **THEN** centerline RMSE against the Ghia tables is within the pinned gate

#### Scenario: Example verifies the refinement trend
- **WHEN** the cavity example runs in its trend mode
- **THEN** the finer grid's time-converged centerline RMSE is within its pinned gate and strictly below the coarse grid's, with a nonzero exit on violation

#### Scenario: Example emits centerlines and the vortex table
- **WHEN** the cavity example runs at full resolution
- **THEN** it writes centerline CSVs and the detected vortex centers alongside the Ghia reference values

#### Scenario: The committed baseline is a finished run
- **WHEN** the committed cavity `baseline.txt` is read
- **THEN** it carries the centerline RMSE, the vortex-center table and the verdict line, rather than
  terminating mid-march as a progress trace

#### Scenario: Pinned bounds are not presented as Ghia agreement
- **WHEN** the cavity gate block is printed or documented
- **THEN** each pinned RMSE bound is marked `tripwire`, while the Ghia table values it is compared against
  are shown separately as the reference

### Requirement: Stencil-path coverage in the existing ladder
The validation ladder SHALL run every existing CI rung (2D Taylor–Green
table, 2D-in-3D, inviscid invariants, double shear layer) through the
compiled stencil pipeline with results matching the generic path at
tolerance, so the ladder gates both evaluation strategies permanently.

#### Scenario: Ladder is strategy-agnostic
- **WHEN** the CI ladder runs with the stencil pipeline enabled
- **THEN** every rung passes with observed orders and conservation gates equal to the generic path's

### Requirement: Isolated-cylinder rung gates its published references

The `dec_cylinder_verification` example SHALL gate the shedding Strouhal number `St = f·D/U` and the
cycle-mean drag coefficient `C_d` against the published laminar benchmarks it already cites (Williamson
1996 for `St`; Dröge & Verstappen 2005 and the Lehmkuhl lineage for `C_d`), and SHALL exit non-zero when a
gate breaks or when the solver returns an error.

The harness currently contains no assertion and no `process::exit` call: on a solver `Err` it prints,
breaks the march, then reports `St` and `C_d` computed from the *truncated* series and returns zero. That
behaviour contradicts the convention `verification/README.md` advertises for every program in the suite,
and it is the crate's only isolated-cylinder validation.

Because the affordable default grid (8 cells/D) is below reference-grid quality, the `St` and `C_d` bounds
MAY be pinned tripwires rather than reference gates at the default configuration. Whichever class is used
SHALL be declared per the evidence-class requirement, and the reference values SHALL remain printed
alongside the measured ones so the offset stays visible.

#### Scenario: A solver error fails the run

- **WHEN** `solver.step` returns `Err` during the march
- **THEN** the harness reports the failure and exits non-zero, and does not report `St` or `C_d` derived
  from the truncated series

#### Scenario: Strouhal and drag are gated, not merely reported

- **WHEN** the harness completes a full march
- **THEN** `St` and `C_d` are each compared against a declared bound, the comparison result is printed as
  PASS or FAIL with its evidence class, and any FAIL exits non-zero

#### Scenario: Reference values stay visible next to the measurement

- **WHEN** the gate block is printed
- **THEN** the Williamson `St` and the Dröge–Verstappen / Lehmkuhl `C_d` band appear next to the measured
  values, with the grid resolution stated, so an under-resolved pass is not read as reference agreement
