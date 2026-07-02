## 0. De-risk (before any host code)

- [x] 0.1 Study run: measure per-step wall-clock and assembly cost of `CompressibleMarcher3dFitted`
  at candidate corridor grids and bond caps (`deep_causality_cfd/studies/`); record the go/no-go
  grid choice and the freestream-rebuild tolerance implied by the assembly cost. If the 3-D budget
  fails the minutes target, record the `CompressibleMarcher2d` fallback decision here.
  → **Measured** (`studies/compressible_carrier_timing`): 3-D fitted at 16³/cap-16 costs 10.7 s
  per step (35.6 min projected — over budget by 3.6×; larger grids strictly worse). The 2-D
  compressible marcher fits with wide margin: 0.026 s/step at 32²/cap-16 (5 s projected) to
  0.174 s/step at 64²/cap-32 (35 s projected). **Fallback taken: the corridor carrier is
  `CompressibleMarcher2d`** (evolved-state provenance preserved; the corridor stage stack is
  already `PhysicsStage<2, _>`); GO configuration 64²/cap-32, fast option 32²/cap-16. Assembly is
  ~free (one rebuild ≈ 0.01 steps), so the freestream rebuild-on-drift tolerance can be tight
  (~10 rebuilds add < 0.05%). The 3-D fitted marcher remains the stagnation/validation tool.

## 1. Marcher seam (one coupled-loop machinery)

- [x] 1.1 Extract the crate-internal `CoupledMarcher` seam (associated `State`; `advance`,
  `publish`, `transport`, `finish`) from the QTT host's loop helpers; generalize
  `MarchPause`/`MarchFork` and the samplers over it. No public-API change; no `dyn`.
  → `types/flow/carrier.rs`: `CoupledCarrier<const D, R>` (build / seed_state / encode_seed / dt /
  advance / publish_and_transport / finish / config accessors, with `coupled_step` as a provided
  method), `BlackoutSampler`, `run_coupled_driver`, `run_until_driver`, and the generic
  `CarrierPause`/`CarrierFork` carrying the CoW, error-capture, and alternation contracts once.
  `MarchPause`/`MarchFork` are now public aliases over `QttCarrier<R>` (D = 2).
- [x] 1.2 Refactor `QttMarchRun` onto the seam. Gate: the existing DSL-equivalence, pause/fork,
  alternation, and full cfd suites pass unchanged before any compressible code lands.
  → Full suite green after the refactor: 559 passed, 0 failed, no warnings.

## 2. Compressible host

- [x] 2.1 `CompressibleMarchConfig` + builder in `types/flow_config/` (owned container: grid/shell
  geometry, gas properties, Park-2T closure inputs, truncation, stop, observe; the config→run
  split). Tests mirror src; Bazel registered.
  → `compressible_march_config.rs`: config + builder (grid/solver/`flight_dt`/`seed_fn` over
  nondimensional primitives/stop/observe/schedule/reference), `AtmosphereRow`, `DescentSchedule`
  (validated, interpolating, clamped), `ReferenceScales`. `dt_flight` is the explicit
  compressed-time constant. Per the D0 outcome the carrier is the 2-D marcher.
- [x] 2.2 The host run type in `types/flow/` over `EulerStateTt` via the seam: `run_coupled`,
  `run_until` + pause/fork. Evolved projections into the `CoupledField` each step.
  → `compressible_march_run.rs`: `CompressibleCarrier` (seam impl over `EulerStateTt2d`),
  `CompressibleMarchRun` with the three verbatim alternation verbs, `CompressiblePause`/`Fork`
  aliases, `CfdFlow::compressible_march`. Projections: `"speed"`, `"T_tr"`, `"n_tot"`, and
  `"pressure_atm"` (all evolved, rescaled by the fixed reference anchors). `T_ve` comes from the
  Millikan-White lag stage driven by the *evolved* per-cell inputs (the marcher state is 5-field
  Euler; see task 3.1), not from an evolved vibrational energy.
- [x] 2.3 Sensed heating on this carrier: resolved per the design's open question in favor of
  Sutton-Graves-form heating on the **evolved density and flight speed** (the example's loads
  stage; lands with task 5). The Brinkman wall integral stays on the QTT carrier; no fitted-shell
  observable is needed for the 2-D carrier.
- [x] 2.4 Freestream schedule: `DescentSchedule` evaluated at the truth state each `pre_step`
  (new seam hook); the exact `FittedNormalShock` RH jump feeds the Dirichlet **inflow strip** (the
  shock-fitted boundary of the marched layer); flight scalars (`"flight_altitude"`,
  `"flight_speed"`, `"flight_mach"`, `"freestream_n"`) published for downstream stages;
  rebuild-on-drift of the acoustic envelope with each rebuild logged. Tests: projections
  published; the strip holds the RH post-shock temperature (within 15% at Mach 24); rebuild
  logged; schedule inert without a truth state; fork/alternation smoke on the new host.

## 3. Evolved-state chemistry stages

- [x] 3.1 `VibrationalLagStage::with_pressure_field("pressure_atm")`: the Millikan-White clock
  reads the evolved per-cell pressure instead of the config constant (the planned
  `RateControllerStage` is subsumed — the lag stage already writes `"T_a"`). The config-pressure
  constructor remains for the QTT path. Tests: per-cell pressure drives per-cell relaxation.
  → Per-cell / single-cell-broadcast / config-fallback resolution inside `apply`; tests cover
  per-cell divergence and the absent-field fallback equivalence.
- [x] 3.2 `IonizationStage::with_density_field("n_tot")`: per-cell density in place of the scalar
  config; scalar constructor unchanged. Tests: per-cell density drives per-cell `n_e`.
  → The per-cell density enters all three uses: the Saha surrogate target `α_eq(T, n_i)`, the
  clock `τ_ion = 1/(k_f·[M_i])`, and both `n_e = α·n_i` write-backs (sheath-renewal and carried
  paths). Tests: dense cell ionizes more; a broadcast single-cell field reproduces the scalar
  config exactly. Suite green: 573 passed, clippy clean.
- [x] 3.3 A/B the sheath-renewal mode on the compressible carrier (real through-flow vs explicit
  renewal) against the stagnation-closure calibration; keep the matching mode, document the other
  (the design's open question, settled empirically at the flagship re-pin).
  → **Explicit renewal kept.** Measured on the full descent: with renewal the peak `n_e` at the
  61 km passage is 1.43e19 (anchor 1e19, well inside the 5× band); without it the carried
  fraction accumulates to equilibrium — peak 2.68e21 (268× over), premature onset at 82 km, and
  no exit within the 700-step horizon. The marched strip renews the *thermodynamic* state, but
  the ionization fraction is a carried scalar with no through-flow on this carrier. Documented
  in the flagship's `constants.rs` model labels.

## 4. Bank-steered lift (3-DOF)

- [x] 4.1 `BankSteeredLift` stage in `corridor.rs`: drag from the dynamic-pressure bundle, lift
  `L = (L/D)·D` rotated about the velocity vector by the clamped control-channel bank (previous
  step; one-step actuation lag documented); lift-plane basis from the local radial at the truth
  position; writes the full 3-vector into the ④ channel. Tests: zero bank = in-plane; ±bank
  curves the trajectory oppositely; clamped (not raw) command actuates. 6-DOF explicitly out of
  scope.
  → Drag anti-parallel to the truth velocity; lift `a_lift·(cos φ·n̂ + sin φ·v̂×n̂)` with `n̂` the
  radial projected off `v̂`; `with_speed_field` for the carrier's `"flight_speed"`. Degenerate
  fallbacks (no speed → no-op; no truth state → axis drag; radial velocity → pure drag)
  documented. Five tests including the gate-clamp actuation path.
- [x] 4.2 Terminal truth/navigation states exposed on coupled reports (report finisher); branch
  scoring computes trajectory-derived `miss_distance` to a configured aim point; the t²-law stays
  as a printed cross-check. Tests: distinct banks yield distinct misses.
  → `finish_report` (carrier.rs) publishes `"final_truth_state"` (6 cells) and
  `"final_nav_position"` (3 cells) when the field carries them — both hosts inherit it.
  `BranchAccumulator::finish_at(terminal, aim)` closes a branch with the Euclidean
  trajectory-derived miss; the t²-law close (`finish`) remains for the printed cross-check.
  Suite green: 580 passed, clippy clean.

## 4b. Parallel counterfactual fan-out (addendum, user-requested)

- [x] 4.3 Scoped fork-join over slices in `deep_causality_par` (`scoped_map`: order-preserving
  map on `std::thread::scope` threads under `parallel`, inline map without; no Rayon, no thread
  pool, no new dependency) + `CarrierPause::continue_branches(worlds, steps)` on the seam, with
  `MaybeParallel` bounds so serial builds carry no `Send + Sync` obligations. The flagship's
  branch study now fans out through it (avionics examples enable `cfd/parallel`).
  → Tests: `scoped_map` suite green in both feature modes; `continue_branches` matches the
  manual fork chain bit-identically in both modes. Flagship gates unchanged and bit-identical
  (peak `n_e` 1.427e19, separation 56.52 m, spread [20.63, 70.00] m); wall-clock 42.2 s → 34.2 s.
  Spec delta: "Parallel counterfactual fan-out" requirement added to
  `blackout-coupling-interface`; validated strict.

## 5. Flagship: one continuous descent

- [x] 5.1 Rewire `examples/avionics_examples/cfd/plasma_blackout_corridor/` onto the compressible
  host: one `run_until`-driven descent over the freestream schedule; the stack drops the
  reconstruction stages (`RateControllerStage` + per-cell ionization instead); station constants
  reduce to freestream + geometry; the branch study forks at the flow-resolved onset; the
  committed branch flies bank-steered lift. Prose per the AI style guide; labels updated
  (compressed-time mapping printed in the banner).
  → One continuous descent (90 → ~46 km, 32²/cap-16, `dt_flight` 0.1 s/step): the truth vehicle
  drives the schedule, the RH inflow strip is the shock-fitted boundary, and chemistry runs on
  the evolved per-cell state (`RecoveryTemperatureStage`/`EosStage` gone; `FlightCondition`
  stations gone). Candidate banks ride as world-published constants (`publish_constant`, new
  config knob) so forked branches differ only in the command; `BankSteeredLift` flies the
  gate-clamped bank (the committed 40° world is bounded to the 0.5 rad cap every step, logged).
  Diagnostic pauses (onset / 61 km passage / exit / reacquisition) never change the world.
  Support fixes surfaced by the rewire: ESKF covariance update moved to the Joseph form
  (the simple form corrupts `P` under long precise-fix sequences and the injected corrections
  diverge), and the example's flat `P0`/`Q` priors replaced with per-block diagonals.
- [x] 5.2 Re-pin the gates: RAM-C II anchor band held at the 61 km passage (5×); flow-resolved
  onset → nonzero dwell → exit ordering; steered-vs-zero-bank divergence; trajectory-derived miss
  spread across branches; drift → reacquisition; bond ≤ cap; wall-clock inside the minutes budget.
  `exit(1)` on regression, `exit(2)` on setup failure.
  → All ten gates pass, exit 0: onset found at 71.6 km, denied through the 60.9 km passage at
  Mach 24.9 with `n_e` = 1.43e19 (anchor 1e19, 5× band), flow-resolved exit at 45.6 km after a
  55.9 s dwell, drift 0.03 → 0.95 m dead-reckoning and 0.23 m after reacquisition, slip →
  continuum regime events logged, committed-vs-ballistic terminal separation 56.5 m with miss
  spread [20.6, 70.0] m, bond 16 ≤ cap 16, 0 rebuilds ≤ 3, ~42 s wall-clock (budget 600 s).

## 6. Finalize

- [x] 6.1 `make format && make fix`; full cfd + examples green; clippy clean (fix, never allow);
  Bazel registration for every new test module; no `unsafe`/`dyn`/lib-macros; float literals only
  in tests and the example.
  → 581 cfd tests green, clippy 0 warnings across cfd + examples, flagship exits 0 in ~42 s.
  Bazel: the `types_flow`/`types_flow_config` suites glob `**/*_tests.rs`, so the two new test
  files are registered automatically; the timing study is a Cargo example target.
- [x] 6.2 `openspec validate add-compressible-blackout-carrier --strict`; update the gap-analysis
  §5 status note to point at this change (the corridor's carrier upgraded from surrogate to
  compressible); prepare the commit message for review (never commit).
  → Validated (strict). Gap-analysis §5 carries a second status block pointing here. Commit
  message prepared for review.
