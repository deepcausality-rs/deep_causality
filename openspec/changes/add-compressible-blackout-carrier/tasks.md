## 0. De-risk (before any host code)

- [ ] 0.1 Study run: measure per-step wall-clock and assembly cost of `CompressibleMarcher3dFitted`
  at candidate corridor grids and bond caps (`deep_causality_cfd/studies/`); record the go/no-go
  grid choice and the freestream-rebuild tolerance implied by the assembly cost. If the 3-D budget
  fails the minutes target, record the `CompressibleMarcher2d` fallback decision here.

## 1. Marcher seam (one coupled-loop machinery)

- [ ] 1.1 Extract the crate-internal `CoupledMarcher` seam (associated `State`; `advance`,
  `publish`, `transport`, `finish`) from the QTT host's loop helpers; generalize
  `MarchPause`/`MarchFork` and the samplers over it. No public-API change; no `dyn`.
- [ ] 1.2 Refactor `QttMarchRun` onto the seam. Gate: the existing DSL-equivalence, pause/fork,
  alternation, and full cfd suites pass unchanged before any compressible code lands.

## 2. Compressible host

- [ ] 2.1 `CompressibleMarchConfig` + builder in `types/flow_config/` (owned container: grid/shell
  geometry, gas properties, Park-2T closure inputs, truncation, stop, observe; the config→run
  split). Tests mirror src; Bazel registered.
- [ ] 2.2 The host run type in `types/flow/` over `EulerStateTt` via the seam: `run`,
  `run_coupled`, `run_until` + pause/fork. Evolved projections into the `CoupledField` each step:
  `"speed"`, `"T_tr"`, `"T_ve"`, `"n_tot"`.
- [ ] 2.3 Wall heat flux on the fitted shell (Fourier-at-wall from the evolved temperature, or the
  penalization analog — pick per the design's open question) published as `"wall_heat_flux"`;
  sampler/observe wiring reused.
- [ ] 2.4 Freestream schedule: a small cited standard-atmosphere table (checked in), evaluated at
  the truth state each step; rebuild-on-drift with configured tolerance; each rebuild logged to
  provenance. Tests: schedule follows altitude; rebuild count bounded and logged.

## 3. Evolved-state chemistry stages

- [ ] 3.1 `RateControllerStage`: `"T_a" = √(T_tr·T_ve)` from the evolved fields (thin, tested;
  Park 1990/1993 cited). `RecoveryTemperatureStage`/`VibrationalLagStage` remain for the QTT path.
- [ ] 3.2 `IonizationStage::with_density_field("n_tot")`: per-cell density in place of the scalar
  config; scalar constructor unchanged. Tests: per-cell density drives per-cell `n_e`.
- [ ] 3.3 A/B the sheath-renewal mode on the compressible carrier (real through-flow vs explicit
  renewal) against the stagnation-closure calibration; keep the matching mode, document the other
  (the design's open question, settled empirically).

## 4. Bank-steered lift (3-DOF)

- [ ] 4.1 `BankSteeredLift` stage in `corridor.rs`: drag from the dynamic-pressure bundle, lift
  `L = (L/D)·D` rotated about the velocity vector by the clamped control-channel bank (previous
  step; one-step actuation lag documented); lift-plane basis from the local radial at the truth
  position; writes the full 3-vector into the ④ channel. Tests: zero bank = in-plane; ±bank
  curves the trajectory oppositely; clamped (not raw) command actuates. 6-DOF explicitly out of
  scope.
- [ ] 4.2 Terminal truth/navigation states exposed on coupled reports (report finisher); branch
  scoring computes trajectory-derived `miss_distance` to a configured aim point; the t²-law stays
  as a printed cross-check. Tests: distinct banks yield distinct misses.

## 5. Flagship: one continuous descent

- [ ] 5.1 Rewire `examples/avionics_examples/cfd/plasma_blackout_corridor/` onto the compressible
  host: one `run_until`-driven descent over the freestream schedule; the stack drops the
  reconstruction stages (`RateControllerStage` + per-cell ionization instead); station constants
  reduce to freestream + geometry; the branch study forks at the flow-resolved onset; the
  committed branch flies bank-steered lift. Prose per the AI style guide; labels updated
  (compressed-time mapping printed in the banner).
- [ ] 5.2 Re-pin the gates: RAM-C II anchor band held at the 61 km passage (5×); flow-resolved
  onset → nonzero dwell → exit ordering; steered-vs-zero-bank divergence; trajectory-derived miss
  spread across branches; drift → reacquisition; bond ≤ cap; wall-clock inside the minutes budget.
  `exit(1)` on regression, `exit(2)` on setup failure.

## 6. Finalize

- [ ] 6.1 `make format && make fix`; full cfd + examples green; clippy clean (fix, never allow);
  Bazel registration for every new test module; no `unsafe`/`dyn`/lib-macros; float literals only
  in tests and the example.
- [ ] 6.2 `openspec validate add-compressible-blackout-carrier --strict`; update the gap-analysis
  §5 status note to point at this change (the corridor's carrier upgraded from surrogate to
  compressible); prepare the commit message for review (never commit).
