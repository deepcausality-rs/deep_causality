# Tasks: add-dec-periodic-ns-solver

Coverage discipline (carried over from Stage 0): every group ends with its
error and branch paths tested; 100% line coverage on new files, with
genuinely unreachable arms documented inline as exemptions. Tests run at
f32, f64, and Float106 wherever a numeric gate exists.

## 1. Plumbing and rate assembly (`dec-ns-rate`)

- [ ] 1.1 Promote `deep_causality_calculus` from dev-dependency to runtime
  dependency in `deep_causality_physics/Cargo.toml` (path + version), update
  `BUILD.bazel` deps, and add the edge to AGENTS.md's dependency table.
- [ ] 1.2 Create the folder module
  `src/theories/fluid_dynamics/dec/` (mod.rs with module doc stating the
  governing formulation and the Chorin placement) and register it in
  `theories/fluid_dynamics/mod.rs`; export the public types from `lib.rs`.
- [ ] 1.3 Implement the rate evaluator (one type, own file): construction
  validates metric presence, edge-count match for velocity and optional
  body force, and finite `ν ≥ 0`; evaluation composes
  `exterior_derivative(1)` → `interior_product` → `laplacian(1)` with the
  pinned viscous sign and additive body force.
- [ ] 1.4 Tests: convective term vs. the Stage 0 oracle over `[8, 16, 32]`
  (reuse the capstone comparison machinery), viscous-sign decay pin,
  body-force additivity (exact), evaluation at f32/f64/Float106.
- [ ] 1.5 Tests (errors/branches): mismatched velocity length, mismatched
  body-force length, NaN/`+∞`/negative `ν`, metric-free manifold rejection;
  document the construction-validated unwraps as coverage exemptions.

## 2. March step and run loop (`dec-ns-march`)

- [ ] 2.1 Implement `StepOutput<R>` (state, max speed, divergence residual,
  accumulated time or step count) as its own small type with getters.
- [ ] 2.2 Implement `DecNsSolver<'m, D, R>` (folder module): configuration
  (ν, dt, optional `BodyForceOneForm`, `HodgeDecomposeOptions`, two CFL
  safety factors defaulting to 0.9), constructor validating via the rate
  evaluator's construction path.
- [ ] 2.3 Implement `step`: `as_one_form` → `Rk4::run` →
  `SolenoidalField::from_leray_projection` → CFL guard → `StepOutput`;
  advective and diffusive limits per design D6 with the skip conditions and
  the dedicated error carrying limit and actual `dt`.
- [ ] 2.4 Implement `run_n` and `run_until` carrying the `Result` chain and
  the failing step index; predicate checked on each produced state.
- [ ] 2.5 Implement initial-condition seeding: vertex vectors through
  `de_rham` (and an exact-integrals variant through
  `de_rham_from_integrals`) into `VelocityOneForm`, then one projection to
  the starting `SolenoidalField`.
- [ ] 2.6 Implement the `PropagatingEffect` wrapper in `wrappers.rs`
  following the existing kernel-wrapper convention.
- [ ] 2.7 Tests (happy paths): step divergence-free at all three
  precisions; zero-state inviscid fixed point; seeded Taylor–Green state
  divergence-free with energy at the analytic value; `run_n` step count;
  `run_until` predicate stop with step count.
- [ ] 2.8 Tests (errors/branches): CG starvation short-circuit (one
  iteration), advective CFL violation message naming both numbers,
  diffusive violation at rest, failing-step index in `run_n` and
  `run_until`, `run_until` bound exhaustion, wrapper success and failure
  conversion, constructor rejections delegated from the rate path.

## 3. Diagnostics (`dec-ns-diagnostics`)

- [ ] 3.1 Implement kinetic energy, enstrophy, and 3D-only helicity per
  design D8, with the dimension guard on helicity; standalone max-speed and
  divergence-residual functions shared with the step.
- [ ] 3.2 Implement `pressure_diagnostic`: one Leray projection of the
  unprojected RHS at the given state; return Bernoulli and static
  `PressureZeroForm`s; document the extra-solve cost and `ρ = 1`.
- [ ] 3.3 Tests: energy convergence to analytic `E(0)` over the ladder;
  enstrophy zero on a constant field; helicity finite on a 3D state and
  rejected on 2D; step-output residual identical to direct evaluation;
  Taylor–Green static pressure vs. analytic field up to gauge over the
  ladder; the two conventions differing by exactly the kinetic 0-form;
  diagnostic CG starvation returns the error; all numeric gates at three
  precisions.

## 4. Validation ladder (`dec-ns-validation`)

- [ ] 4.1 CI test: 2D Taylor–Green decay on `[8, 16, 32]` `square_torus`
  — per-grid envelope tolerance and observed spatial order ≥ 1.9 at f64;
  documented looser f32 gate; Float106 at the f64 gate; doc comment stating
  the splitting-bounded temporal order.
- [ ] 4.2 CI test: 2D-in-3D Taylor–Green on `cubic_torus` (16³ at f64) —
  envelope agreement plus `max|w|` at projection tolerance throughout.
- [ ] 4.3 CI test: inviscid invariants — 3D energy and helicity drift
  bounds over the documented horizon, recorded in the test; 2D energy
  drift bound.
- [ ] 4.4 CI test: double shear layer (Brown–Minion form, one modest 2D
  resolution, f64) — roll-up witness (cross-stream energy ≥ 10× seed),
  monotone energy and enstrophy decay within documented tolerance,
  divergence-free at every sample, and the Q-criterion vortex-core gate via
  the existing kernel fed by a test-side central-difference gradient of the
  `sharp`-recovered field.
- [ ] 4.5 Example binary in `examples/avionics_examples/` (beside the
  existing `cfd_taylor_green` harness):
  parameterized-resolution Re-1600 3D Taylor–Green emitting
  time/energy/dissipation CSV; smoke-tested at the small default by its
  own crate test, never run at reporting resolution in CI.
- [ ] 4.6 Register all new test files in their `mod.rs` chain with
  `#[cfg(test)]`, and extend the Bazel test suites
  (`deep_causality_physics/tests/BUILD.bazel`, example build files) for the
  new folders.

## 5. Verification and closeout

- [ ] 5.1 `make format && make fix`; `cargo build -p deep_causality_physics`
  and `cargo test -p deep_causality_physics` green; clippy clean with no
  suppressions.
- [ ] 5.2 Coverage audit via `cargo llvm-cov -p deep_causality_physics`:
  100% lines on new files except inline-documented exemptions; close any
  undocumented gap with tests.
- [ ] 5.3 Full workspace `make build && make test` green (calculus
  promotion touches the dependency graph).
- [ ] 5.4 Update `openspec/notes/cfd/cfd-roadmap.md` Stage 1 status and
  note any deviations discovered during assembly (upstream defects were
  fixed at their source with their own tests, per the monorepo rule);
  prepare the commit message and hand off to the user for commit (agents
  never commit).
