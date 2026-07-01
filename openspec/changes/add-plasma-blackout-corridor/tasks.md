# Tasks — Plasma-Blackout Corridor (reordered, contract-first, staged)

Each stage builds only on completed lower stages. The only mocks are Stage-0 stubs behind the final interface,
swapped (not rewritten) at Stage 1.

## Stage 0 — Foundations & contracts (promote + extend the seam)

- [ ] 0.1 **Extend the `.couple` seam (④).** `deep_causality_cfd` `types/flow/coupling.rs`: add an aero-force
  (Cartesian vector) channel + a control/action channel to `CoupledField` (or `Ambient`), alongside the existing
  named scalars; add the classifier-input fields (Knudsen, ionization fraction, GNSS state) + the provenance/
  `EffectLog` schema. Consumers stay `PhysicsStage` impls on the static cons-tuple (no `dyn`); `BlackoutTrigger`
  keeps deriving the flag from peak `n_e`. **No new seam type.**
- [ ] 0.2 **Stub `PhysicsStage`** (mock drag + scheduled blackout) satisfying the extended contract so Stages 2–3
  build now; swapping it for the Stage-1 marcher stage changes no consumer. Tests: stub satisfies the contract; a
  consumer stage composes against the seam alone.
- [ ] 0.3 **Promote KS propagator (B1).** `deep_causality_physics` `kernels/astro/ks_propagator.rs`: 3-D KS
  matrix-exponential core generalizing the shipped planar `TwoBodyPropagator`; between-step Strang perturbation
  hook (closure = the ④ force channel). Cite Stiefel–Scheifele (1971), Battin (1999); PDF in `papers/`. Tests
  (f64 + Float106, 100% cov): coast round-off, semigroup, conservation, rejection, Strang order ≈ 2, linear-in-ε.
- [ ] 0.4 **Promote Sp(2,R)/KS projection (B2).** `deep_causality_physics` constraint-projection kernel with a
  documented fixed gauge. Tests: idempotent, restores a perturbed state, residuals bounded across a denial cycle.
- [ ] 0.5 **Wire the shipped clock (B3).** Confirm consumption of `relativistic_clock_drift_rate_kernel` with the
  two-clock (`s ≠ τ`) carry; metric from state; only `G`, `c`, EGM/IERS literal. (No new kernel.)

## Stage 1 — CFD real-fidelity (fill ④ with real data)

- [ ] 1.1 **3-D body-fitted `MetricProvider`** — completes the compressible-marcher remainder; curved-shock rank
  `O(10)` not `√side`. Builds on `body-fitted-qtt-coordinate` / `compressible-reacting-qtt-marcher`.
- [ ] 1.2 **Dynamic marched-rank re-pin** (Res 5 / D9) across the flux-through-front.
- [ ] 1.3 **Marcher adapter** implements the Stage-0 ④ interface via `CfdFlow` — real force / heat / transported
  `n_e` / blackout flag; replaces the stub with no consumer change. Tests: real output drives consumers unchanged;
  blackout flag fires from `n_e` × plasma-frequency threshold.
- [ ] 1.4 *(optional)* Chemistry lever 3 (finite-rate ionization network), off by default — firms peak `n_e`
  toward the production band; **not** a flagship blocker (lever 1 already ~1.1×).

## Stage 2 — Trajectory/nav engine (built once, against ④)

- [ ] 2.1 Engine: predict = KS propagate + aero kick from the ④ force channel; correct = 17-state tightly-coupled
  ESKF (pos/vel/att-err/gyro-bias/accel-bias + clock bias/drift) + the B2 projection; two-clock carry. No
  mock/real split (stub and marcher interchangeable behind the contract).
- [ ] 2.2 Synthetic sensors (⑥): strapdown IMU (primary), through-plasma optical (~50 m 1σ), GNSS (denied when
  the ④ blackout flag is set), carried clock. Q from a nav-grade IMU spec (gyro < 0.01 °/hr, accel < 10 µg); R
  from sensor accuracy.
- [ ] 2.3 Encke↔Cowell `select_integrator` switch (B4): ε = a_aero/a_grav from the ④ force vs a config threshold,
  with hysteresis. Consumes stub ε now, real ε after Stage 1.
- [ ] 2.4 Validate engine logic against the Stage-0 stub: coast exactness, GPS clock split, closed-loop nav
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
