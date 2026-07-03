## 1. Physics kernels (deep_causality_physics)

- [ ] 1.1 Constants: the Park (1990) coefficients for the network in
  `deep_causality_physics/src/constants/` (dissociative-recombination via equilibrium constants,
  electron-impact ionization of N and O with their activation temperatures, the
  equilibrium-constant curve-fit coefficients for `N + O <-> NO+ + e-` and the N2/O2
  dissociation equilibria the atom pool needs). Source PDF added to
  `deep_causality_physics/papers/`; every constant cites its table.
- [ ] 1.2 The equilibrium-constant kernel (Park curve-fit form) and the derived
  dissociative-recombination rate kernel (`k_b = k_f / K_eq` inside the kernel; detailed balance
  by construction). Pure pointwise contract, `PropagatingEffect` wrappers, module registration,
  `lib.rs` flattening, full docstring citations.
- [ ] 1.3 The thresholded electron-impact ionization kernels for N and O, rated at the electron
  temperature. Same contract and registration.
- [ ] 1.4 The lagged atom-pool kernels: N and O dissociation equilibria at the controller
  temperature as LER targets, with the dissociation-rate clock `tau_pool = 1/(k_d[M])` from the
  Park dissociation rates (design D3: the pool ships lagged; partial equilibrium is the limit it
  relaxes toward, not the closure).
- [ ] 1.5 Pointwise validation tests, mirrored and Bazel-registered: equilibrium recovery from
  both sides per reaction, detailed balance across the tabulated temperature range, frozen
  limits at low temperature for every channel, purity (two states, two outputs). Suite green;
  clippy clean.

## 2. The network stage (deep_causality_cfd)

- [ ] 2.1 `FiniteRateIonizationStage` beside `IonizationStage` in `types/flow/`: per-cell
  closed-form fixed point of the three-channel network (quadratic loss through quasi-neutrality),
  LER relaxation with `tau = 1/(k_f[M] + beta * n_e)`, heavy channels at `T_a`, electron
  channels at `T_e = T_ve`, the same per-cell/broadcast/config-fallback input resolution and the
  same optional sheath-renewal mode as the surrogate stage. Writes `"alpha"` and `"n_e"`.
  `IonizationStage` untouched.
- [ ] 2.2 Stage tests, mirrored and Bazel-registered: fixed point matches the kernel equilibrium
  from above and below; recombination decays a hot carried fraction in a cold cell; frozen limit
  leaves the fraction unchanged; per-cell density and temperature fields drive per-cell `n_e`;
  renewal toggle behaves as the surrogate's. Full cfd suite green (the surrogate path
  bit-identical); clippy clean.

## 3. Uncalibrated stagnation-line measurement (sequenced before any corridor re-pin)

- [ ] 3.1 Re-measure `qtt_ramc_stagline` with the network and no Saha calibration target,
  **channel by channel** (design D7): first channel 1 plus the lagged atom pool alone, then with
  electron impact enabled, recording both, so any band miss is attributable to a channel. Pin
  the uncalibrated peak-`n_e` band from the full-network measurement with the production-code
  context (DPLR/LAURA/US3D 2x to 3x) recorded in the report. The lever-1 calibrated history
  stays recorded. If the prediction misses the production band, stop and surface the
  attribution (pool clock, electron impact, or rate data) and the Zeldovich decision before
  proceeding.
- [ ] 3.2 The sheath-renewal A/B under recombination, on the stagnation line: both modes
  measured, the mode matching the closure kept, both numbers and the reasoning recorded in the
  example's model labels (superseding the first A/B's record).

## 4. Corridor and weather re-pin (examples)

- [ ] 4.1 Swap the network stage into the shared coupling
  (`avionics_examples::blackout::world::corridor_coupling`); update `constants.rs` model labels
  (the forward-only limitation is removed; the renewal decision from 3.2 is recorded; the
  network's rating temperatures labeled).
- [ ] 4.2 Flagship re-pin: anchor gate re-pinned to the measured uncalibrated band; a new gate
  compares the blackout exit altitude against the RAM-C II flight window (25 to 30 km) with a
  band pinned from measurement; onset altitude printed as a prediction; horizons and leg budgets
  re-pinned for the longer dwell; all gates green, exit 0, inside the wall-clock budget.
- [ ] 4.3 Weather-table re-pin: absolute constants (onset spread, drift factors, horizons)
  re-pinned from the new measurement; the mechanism gates (cold-drift factor, statistical
  resolution, worst-draw reacquisition) retained; all gates green, exit 0.
- [ ] 4.4 READMEs updated with the measured numbers and the calibration-to-validation story
  (both corridor examples); prose per `docs/writing_guides/AiStyleguide.md`, no em dashes.

## 5. Finalize

- [ ] 5.1 `make format && make fix`; full physics + cfd suites green; clippy clean (fix, never
  allow); Bazel registration for every new test module; no `unsafe`/`dyn`/lib-macros; float
  literals only in constants (cited), tests, and examples.
- [ ] 5.2 `openspec validate add-finite-rate-ionization-network --strict`; update the
  chemistry-fidelity gap note (lever 3 status) and the finite-rate preparation note (Option A
  shipped, Option B commentary stands); prepare the commit message for review (never commit).
