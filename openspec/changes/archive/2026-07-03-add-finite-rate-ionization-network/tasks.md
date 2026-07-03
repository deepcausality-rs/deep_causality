## 1. Physics kernels (deep_causality_physics)

- [x] 1.1 Constants: the Park (1990) coefficients for the network in
  `deep_causality_physics/src/constants/` (dissociative-recombination via equilibrium constants,
  electron-impact ionization of N and O with their activation temperatures, the
  equilibrium-constant curve-fit coefficients for `N + O <-> NO+ + e-` and the N2/O2
  dissociation equilibria the atom pool needs). Source PDF added to
  `deep_causality_physics/papers/`; every constant cites its table.
- [x] 1.2 The equilibrium-constant kernel (Park curve-fit form) and the derived
  dissociative-recombination rate kernel (`k_b = k_f / K_eq` inside the kernel; detailed balance
  by construction). Pure pointwise contract, `PropagatingEffect` wrappers, module registration,
  `lib.rs` flattening, full docstring citations.
  -> Landed per amended D4: RP-1232 Table II tabulates the backward rates paired with the
  forwards (its eq. 5a), verified by rendering the scanned table pages from the in-repo PDF and
  reading them directly (the shipped reaction-7 forward constants matched exactly, confirming
  the reading). `no_dissociative_recombination_rate_kernel` (reaction 7 backward) plus
  `n2/o2_dissociation_equilibrium_kernel` (K = k_f/k_b from one table row). New
  `EquilibriumConstant` and `DissociationFraction` newtypes. Constants follow the
  `condensed.rs` real-field accessor mechanism.
- [x] 1.3 The thresholded electron-impact ionization kernels for N and O, rated at the electron
  temperature. Same contract and registration.
- [x] 1.4 The lagged atom-pool kernels: N and O dissociation equilibria at the controller
  temperature as LER targets, with the dissociation-rate clock `tau_pool = 1/(k_d[M])` from the
  Park dissociation rates (design D3: the pool ships lagged; partial equilibrium is the limit it
  relaxes toward, not the closure).
- [x] 1.5 Pointwise validation tests, mirrored and Bazel-registered: equilibrium recovery from
  both sides per reaction, detailed balance across the tabulated temperature range, frozen
  limits at low temperature for every channel, purity (two states, two outputs). Suite green;
  clippy clean.
  -> 12 network tests + wrapper coverage; 1700 physics tests green; clippy 0 (the -3.14
  Table II exponent is written as a quotient to avoid the approx-PI false positive).

## 2. The network stage (deep_causality_cfd)

- [x] 2.1 `FiniteRateIonizationStage` beside `IonizationStage` in `types/flow/`: per-cell
  closed-form fixed point of the three-channel network (quadratic loss through quasi-neutrality),
  LER relaxation with `tau = 1/(k_f[M] + beta * n_e)`, heavy channels at `T_a`, electron
  channels at `T_e = T_ve`, the same per-cell/broadcast/config-fallback input resolution and the
  same optional sheath-renewal mode as the surrogate stage. Writes `"alpha"` and `"n_e"`.
  `IonizationStage` untouched.
  -> Landed with the second-pass revision: the stage reads `"T_tr"` and `"T_ve"` directly and
  builds its own controlling temperatures per channel (ionization at the calibrated geometric
  mean, dissociation at Park's q = 0.7, electron channels at `T_e = T_ve`); the pool clocks are
  `tau_O = 1/(k_d_O2[M])` and `tau_N = 1/(k_d_N2[M] + k_z[O])` (Zeldovich). Also writes
  `"atom_frac_n"`/`"atom_frac_o"` for attribution.
- [x] 2.2 Stage tests, mirrored and Bazel-registered: fixed point matches the kernel equilibrium
  from above and below; recombination decays a hot carried fraction in a cold cell; frozen limit
  leaves the fraction unchanged; per-cell density and temperature fields drive per-cell `n_e`;
  renewal toggle behaves as the surrogate's. Full cfd suite green (the surrogate path
  bit-identical); clippy clean.
  -> 8 stage tests (both-sides convergence, cold-cell recombination decay, frozen limit, per-cell
  drive, lagged pool below equilibrium, renewal statelessness, T_e-field fallback); registered
  in the mirrored tests tree.

## 3. Uncalibrated stagnation-line measurement (sequenced before any corridor re-pin)

- [x] 3.1 Re-measure `qtt_ramc_stagline` with the network and no Saha calibration target,
  **channel by channel** (design D7): first channel 1 plus the lagged atom pool alone, then with
  electron impact enabled, recording both, so any band miss is attributable to a channel. Pin
  the uncalibrated peak-`n_e` band from the full-network measurement with the production-code
  context (DPLR/LAURA/US3D 2x to 3x) recorded in the report. The lever-1 calibrated history
  stays recorded. If the prediction misses the production band, stop and surface the
  attribution (pool clock, electron impact, or rate data) and the Zeldovich decision before
  proceeding.
  -> Measured twice. First pass: 5.46e14 (-4.26 dec), attribution unambiguous (the pool clock;
  electron impact 0.02 percent; rate pairs reproduce the anchor at realistic dissociation).
  Stopped and surfaced per the task; the second ARIZ pass (informed by
  `notes/plasma-blackout/ionization-chemistry.md`) produced the knob-free revision: transit-age
  profile on the stagnation line, Zeldovich exchange in the N-pool clock, dissociation at
  Park's q = 0.7. Second pass: channel 1 + pool 2.60e19 (+0.41 dec), full network 2.99e19
  (+0.48 dec, 3.0x) vs the 1e19 anchor; band pinned at ±0.7 decades; all stagline gates green.
- [x] 3.2 The sheath-renewal A/B under recombination, on the stagnation line: both modes
  measured, the mode matching the closure kept, both numbers and the reasoning recorded in the
  example's model labels (superseding the first A/B's record).
  -> Measured over the same transit-age profile: renewal 2.99e19 (+0.48 dec), carried 4.70e18
  (-0.33 dec). Renewal kept: its fixed-point clock equals the true Riccati relaxation rate
  sqrt(production*beta) near equilibrium and realizes the transit-age closure; the carried arm
  under-relaxes young parcels but self-limits at or below the closed-form arm, which is the
  property recombination was added for (the forward-only surrogate ran away). Gate added
  (carried arm inside the earned band, at or below renewal); recorded in the stagline README
  and baseline.txt; all 7 gates green, exit 0.

## 4. Corridor and weather re-pin (examples)

- [x] 4.1 Swap the network stage into the shared coupling
  (`avionics_examples::blackout::world::corridor_coupling`); update `constants.rs` model labels
  (the forward-only limitation is removed; the renewal decision from 3.2 is recorded; the
  network's rating temperatures labeled).
  -> Swapped; the stage reads the marched "T_tr"/"T_ve" directly (the "T_a" hop is gone). The
  first run failed the anchor gate 60x low: renewal at one residence time reproduces the
  starved-pool regime the first stagline pass measured. Resolved by mirroring the stagline
  closure: both sheath clocks (vibrational bath, network renewal) run at the transit-age
  profile's observable peak, `SHEATH_PEAK_AGE_S = RESIDENCE_TIME_S * 4.174` (ln 65; derived,
  not tuned). Model labels updated with the network, its rating temperatures, and the A/B.
- [x] 4.2 Flagship re-pin: anchor gate re-pinned to the measured uncalibrated band; a new gate
  compares the blackout exit altitude against the RAM-C II flight window (25 to 30 km) with a
  band pinned from measurement; onset altitude printed as a prediction; horizons and leg budgets
  re-pinned for the longer dwell; all gates green, exit 0, inside the wall-clock budget.
  -> The 5x anchor band already equals the stagline-earned ±0.7 decades (10^0.7 ≈ 5.0); the
  gate label now says so. New gate (2b): exit at 46.9 km inside [40, 50] km, reported against
  the RAM-C II 25-30 km window (the offset is the probe's light ballistic bundle); onset at
  74.7 km printed as a prediction (no onset constant exists). Measured: peak n_e 3.35e19 at
  the 60.9 km passage; dwell 58.4 s. The higher onset forks the branches in thinner air, so
  the aim cross-range was re-pinned 45 -> 20 m to keep the sweep's interior optimum (committed
  15 deg, 3.12 m, 6.4x better than ballistic; clamped 40 deg overshoots). 12 gates green,
  exit 0, 35 s wall-clock; output.txt refreshed.
- [x] 4.3 Weather-table re-pin: absolute constants (onset spread, drift factors, horizons)
  re-pinned from the new measurement; the mechanism gates (cold-drift factor, statistical
  resolution, worst-draw reacquisition) retained; all gates green, exit 0.
  -> All 8 mechanism gates passed unchanged under the network (onset spread 4.2 s vs the 2 s
  gate; cold drift 1.50x vs 1.2x; 4.0 sigma vs 2; worst draw 0.24 m vs 1 m; 203 s vs 600 s
  budget). The measured-spread comment and the shared-module label were re-pinned; output.txt
  refreshed.
- [x] 4.4 READMEs updated with the measured numbers and the calibration-to-validation story
  (both corridor examples); prose per `docs/writing_guides/AiStyleguide.md`, no em dashes.
  -> Corridor README: onset/exit altitudes, the new miss landscape, the network section with
  the calibration-to-validation story, RP-1232 anchor, updated limitations (the surrogate
  limitation replaced by the single-point closure honesty), gate count 12. Weather README: new
  table, drift-law check re-derived (within ~5 percent), 4.0 sigma. The stagline verification
  README gained the network and A/B sections in 3.2.

## 5. Finalize

- [x] 5.1 `make format && make fix`; full physics + cfd suites green; clippy clean (fix, never
  allow); Bazel registration for every new test module; no `unsafe`/`dyn`/lib-macros; float
  literals only in constants (cited), tests, and examples.
  -> 1711 physics + 590 cfd tests green; clippy 0 across physics, cfd, and avionics_examples
  (all targets); both new test modules are covered by the existing Bazel globs
  (`kernels/hypersonic/*_tests.rs`, `types/flow/**/*_tests.rs`); both corridor examples and the
  stagline verification run green (exit 0) with refreshed output.txt/baseline.txt.
- [x] 5.2 `openspec validate add-finite-rate-ionization-network --strict`; update the
  chemistry-fidelity gap note (lever 3 status) and the finite-rate preparation note (Option A
  shipped, Option B commentary stands); prepare the commit message for review (never commit).
  -> Validates strict. Gap note: lever 3 marked shipped with both measurements and the A/B
  record. Preparation note: STATUS flipped to shipped with the two design revisions the build
  taught (lagged pool, transit-age profile); Option B commentary stands. Commit message
  prepared and handed to the user (never committed).
