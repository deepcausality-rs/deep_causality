# Tasks — Plasma-Blackout Corridor (reordered, contract-first, staged)

Each stage builds only on completed lower stages. The only mocks are Stage-0 stubs behind the final interface,
swapped (not rewritten) at Stage 1.

## Stage 0 — Foundations & contracts (promote + extend the seam) — **COMPLETE (verified)**

> **Status:** `deep_causality_physics` 1688 tests + `deep_causality_cfd` 398 tests pass; clippy `--all-targets`
> clean on both; fmt clean. Bazel needs no edit (`kernels/astro/*_tests.rs` is glob-matched). Purely additive.

- [~] 0.1 **Extend the `.couple` seam (④).** `deep_causality_cfd` `types/flow/coupling.rs`. **Done:** the
  `aero_force` (`[R;3]`, the trajectory kick input) + `control_action` (`R`, the corrective command output)
  channels on `CoupledField` with getters/setters; re-exported from the flow module and crate root; consumers
  stay `PhysicsStage` impls (no `dyn`). **Deferred to Stage 3** (added when their consumer lands): the typed
  classifier-input fields (Knudsen / ionization / GNSS) and the provenance/`EffectLog` schema.
- [x] 0.2 **Stub `PhysicsStage`.** `AeroBlackoutStub` (constant mock drag + a windowed `n_e` schedule) satisfies
  the extended contract; swapping it for the real marcher stage changes no consumer. Tests: nav-channel
  default-None + round-trip; the stub publishes the force and the windowed `n_e` (in/out/past the window).
- [x] 0.3 **Promote KS propagator (B1).** `deep_causality_physics` `kernels/astro/ks_propagator.rs`: `KsPropagator`
  (exact 3-D KS matrix-exponential core, singularity-free / near-radial-safe) + `ks_strang_step` (the between-step
  Strang hook; its `accel` closure = the ④ force channel). Tests (f64 + Float106): coast exactness vs the
  independent planar `TwoBodyPropagator` to round-off, one-period closure, semigroup, energy/`|L|` conservation,
  Kepler III, rejections; Strang order ≈ 2 vs an RK4 truth, linear-in-ε, zero-kick identity. Stiefel–Scheifele
  (1971) / Battin (1999) cited in the docstring — **no PDF** (textbooks; matches the `two_body` cite-only precedent).
- [x] 0.4 **Promote Sp(2,R)/KS projection (B2).** `deep_causality_physics` `kernels/astro/ks_constraint.rs`:
  `ks_bilinear_residual` + `ks_project_velocity` (nearest under the fixed gauge that holds `u`). Tests: idempotent,
  restores a perturbed velocity, correction is along the constraint gradient (nearest), zero-position rejection.
  (The across-a-denial-cycle residual bound is a Stage-2 integration gate, not a kernel unit test.)
- [x] 0.5 **Wire the shipped clock (B3).** Confirmed: the shipped `relativistic_clock_drift_rate_kernel` is the
  two-clock (`s ≠ τ`) carry the Stage-2 engine consumes; metric from state; only `G`, `c`, EGM/IERS literal. No
  new kernel, no code this stage.

## Stage 1 — CFD real-fidelity (fill ④ with real data) — **COMPLETE (verified; 1.4 optional, deferred)**

> **Status:** `deep_causality_cfd` 415 tests pass; clippy `--all-targets` clean; fmt clean. Purely additive.

- [x] 1.1 **3-D body-fitted `MetricProvider`** (built as `MetricProvider3d`, the explicit-dimensioned sibling).
  **1.1a** `MetricProvider3d` seam + `CartesianIdentity3d` capture limit; **1.1b** `BodyFittedCoordinate3d`
  (spherical-shell fitted metric — the 3-D inverse-Jacobian, gradient of a radial field = r̂); **1.1c** the
  rank-lever gate (body-fitted O(1) flat vs Cartesian growth); **1.1d** `CompressibleMarcher3dFitted<R, M>` —
  the metric-aware marcher, gated by **exact equivalence to `CompressibleMarcher3d` over `CartesianIdentity3d`**.
- [x] 1.2 **Dynamic marched-rank re-pin** (Res 5 / D9): `run_repinned` on the fitted marcher — front-track +
  rank-preserving ζ-roll + `r0` slide + metric rebuild (reusing the acoustic inverse). Gated: re-pin engages and
  pins the front to the target band; bond bounded. *Named refinement:* the full Rankine–Hugoniot interface
  treatment (re-pin is necessary-but-not-sufficient for Cartesian-flux-through-front, per `qtt_repin_marcher`).
- [x] 1.3 **Marcher adapter** (`AeroForceCoupling`): publishes the flow-derived aero force (∝ dynamic pressure)
  into the ④ channel; the reacting stack (`RecoveryTemperature → Ionization → AeroForceCoupling`) fills ④ with
  real `n_e` + force, replacing `AeroBlackoutStub` with no consumer change.
- [~] 1.4 *(NOT adopted — ruled out by the experiment, started then reverted)* Chemistry lever 3 (finite-rate
  ionization network). Per
  [gap-three-resolution-2](../../notes/plasma-blackout/gap-3/gap-three-resolution-2-tve-controlled-ionization.md)
  §"Lever 2" + §C: **lever 1 already lands peak `n_e` at ~1.1× of RAM-C II — inside the production band; "there
  is nothing to improve."** Lever 2 (3-T) was prototyped and rejected for exactly this (it brackets low, buys
  nothing on the calibrated point estimate); lever 3 falls under the same principle — a further chemistry-
  fidelity lever overcorrects an already-calibrated model. Pursue **only if the anchor itself is tightened**
  beyond the ~2–3× the surrogate already achieves. (A detailed-balance finite-rate stage was drafted and
  reverted after this finding.)

## Stage 2 — Trajectory/nav engine (built once, against ④)

> **Crate placement:** the whole navigation layer (INS error-state, ESKF, synthetic IMU, re-entry nav
> engine, integrator regime switch) lives in `deep_causality_cfd/src/navigation/`, **not** in
> `deep_causality_physics`. Navigation/INS/guidance is aerospace *engineering*, not a force of nature; it
> *composes* the physics kernels (KS conformal propagator, relativistic forward clock, KS constraint
> projection) that stay in `deep_causality_physics`. CFD already depends on physics, so the nav layer
> consumes the kernels cleanly.

- [x] 2.1 Engine: predict = KS propagate + aero kick from the ④ force channel; correct = 17-state tightly-coupled
  ESKF (pos/vel/att-err/gyro-bias/accel-bias + clock bias/drift) + the B2 projection; two-clock carry. No
  mock/real split (stub and marcher interchangeable behind the contract).
- [x] 2.2 Synthetic sensors (⑥): strapdown IMU (primary), through-plasma optical (~50 m 1σ), GNSS (denied when
  the ④ blackout flag is set), carried clock. Q from a nav-grade IMU spec (gyro < 0.01 °/hr, accel < 10 µg); R
  from sensor accuracy.
- [x] 2.3 Encke↔Cowell `select_integrator` switch (B4): ε = a_aero/a_grav from the ④ force vs a config threshold,
  with hysteresis. Consumes stub ε now, real ε after Stage 1.
- [x] 2.4 Validate engine logic against the Stage-0 stub: coast exactness, GPS clock split, closed-loop nav
  (`t²`/`t³` drift → reacquire), overlap-band + hysteresis on the switch.

## Stage 3 — Composition (fill the Stage-0 seams)

- [ ] 3.1 Regime classifier (Knudsen + `n_e`/ionization + GNSS state → governing-model selection); thresholds
  config, indicators from state; logs regime changes.
- [ ] 3.2 Counterfactual bank-angle branches (`continue_with`) — each a coupled rollout returning (peak heat,
  thermal load, miss distance, blackout dwell), predict-only through the window.
- [ ] 3.3 Cybernetic bounded-correction gate (`deep_causality_haft::CyberneticLoop::control_step`): S = sensed
  coupled state, B = trajectory/thermal-margin belief (`observe_fn`), C = the verified safety envelope
  (thermal/g-load/physiological/ROE), A = bounded bank-angle correction (`decide_fn`, clamped into C), E =
  unrecoverable breach. Deterministic, no Effect-monad allocation on the hot path. Tests: correction clamps to the
  envelope; no-safe-action returns E; identical inputs → identical Action. Provenance to `EffectLog` per the
  Stage-0 schema. (Effect Ethos stays for non-real-time deontic checks, not this gate.)

## Stage 4 — CFD Flow DSL (re)design

- [ ] 4.1 Design pass: reconcile the preliminary design (design.md) with what Stages 0–3 shipped. Confirm the
  approach is **compose the per-step coupling stack** (`Coupling::between_steps().then(..)`) run by
  `run_coupled` — **not** a new linear phase builder. Minor revision from the preliminary is expected.
- [ ] 4.2 Implement the corridor stages as `PhysicsStage` impls in `deep_causality_cfd`: `RegimeClassify` [2],
  `TrajectoryNav` [4] (KS predict + ESKF; reads the ④ force channel, GNSS gated by the blackout flag; nav state
  threads through `CoupledField`), `CyberneticCorrect` [6]. Evolve `fluiddynamics-dsl` / `qtt-flow`; keep the
  config→run split, the cons-tuple `.then()` composition, and the `seed_with`/`march_with` counterfactuals.
- [ ] 4.3 `CyberneticCorrect` = a `PhysicsStage` wrapping a direct `CyberneticLoop::control_step`: clamp the
  Action into the envelope (mutate the control channel), return `Err(Entropy)` on an unrecoverable breach (reuse
  the coupling's `?` short-circuit). No Effect-monad allocation on the hot path.
- [ ] 4.4 Emit the provenance schema from the loop into `EffectLog` [7]; optional thin convenience over
  `qtt_march`/`run_coupled` if it improves readability.
- [ ] 4.5 Tests: a composed stack + `run_coupled` equals the hand-written `Coupling`/`run_coupled` (same result,
  no `dyn`, no extra hot-path allocation); the `CyberneticCorrect` breach short-circuits with `Err`; the marcher
  reuses its `EndoArrow` step unchanged. (100% coverage of the new stages / DSL surface.)
- [ ] 4.6 **Counterfactual alternation (verbatim core vocabulary).** Add `alternate_context` / `alternate_state`
  / `alternate_value` combinators to `QttMarchRun` (thin wrappers over the `Alternatable` ops; each appends the
  `!!*Alternation!!` audit entry; error channel never alternated). **Pre-run** attach point subsumes
  `seed_with`/`march_with`. Context alternation swaps a **whole** `QttMarchConfig`; alternate worlds are
  checked-in named configs (`config::nominal_reentry()`, `steep_reentry()`, …) — no delta builder.
- [ ] 4.7 **Mid-march fork (resumable loop).** `run_until(predicate) -> MarchPause`, `MarchPause::fork()`,
  `MarchFork::alternate_*`, `continue_march(steps)` — rebuild the solver from the (alternated) context, resume
  from the branch state. Corridor [5] bank-angle branches = context alternations forked from one shared
  blackout-onset state.
- [ ] 4.8 **Arc + copy-on-write marching state.** Wrap the threaded state (`fluid`, `field`) in `Arc`; `fork` /
  `alternate_state` share by reference (O(1)); a writing stage clones via `Arc::make_mut` (cost only on
  divergence). Tests: read-only fork copies no tensor data; first write triggers exactly one CoW clone;
  alternation on an errored run is a no-op with only the audit entry.

## Stage 5 — Flagship example

- [ ] 5.1 `examples/avionics_examples/<flagship>/` wiring corridor §4 chain [1]–[7] in one `CausalFlow`, **written
  in the Stage-4 DSL** — the central control loop within the ~10–30-line budget; over the RAM-C trajectory;
  main-at-top, utils_print, single `FloatType`. Labels every simplification.
- [ ] 5.2 **Coupled validation gate** (the milestone that needed Stage 1): real `n_e` → real blackout window →
  real INS drift → reacquisition; ~2–3× honest bands; all four required elements (regime change, multiphysics,
  counterfactuals, tensor compression) exercised; exits nonzero on regression.

## Finalize (each stage)

- [ ] F.1 `make format && make fix`; per-crate tests green; examples run their gates. No `unsafe`/`dyn`/lib-macros;
  float literals only in test code; `[lints] workspace = true`; Bazel registration updated.
- [ ] F.2 `openspec validate add-plasma-blackout-corridor --strict` passes; update gap-analysis §5 + the Gap-3
  resolutions to point at this reordered plan.

## Out of scope

- [ ] Bars `(4,2)` conformal packaging (optional per FS-1); geopotential harmonics > J2; IERS 2PN clock; full
  6-DOF entry; GPU/parallel acceleration (gated behind the tensor-network acceleration survey); promoting the
  ESKF to a library estimator (YAGNI until a second consumer).
