## 1. Sampler (deep_causality_cfd)

- [x] 1.1 `DescentSchedule::sample` interpolates `ln n_tot` linearly; temperature and sound speed stay linear; document the contract
- [x] 1.2 Rewrite `schedule_interpolates_and_clamps` to the log-linear contract and add the exponential-layer test from the spec
- [x] 1.3 `cargo test -p deep_causality_cfd` and clippy green

## 2. Atmosphere and anchor (examples/avionics_examples/src/shared)

- [x] 2.1 Generate the US-1976 table (0–90 km, 1 km) from Tables 2 and 4; check the generator against transcribed Table I rows (61, 71, 72 km)
- [x] 2.2 Replace `ATMOSPHERE`; cite the standard; state the 86–90 km isothermal extension
- [x] 2.3 Replace `RAMC_NE_REFERENCE` with the station-1 Ka-band anchor (value, altitude, derivation, citation) and add the lower-bound altitude span
- [x] 2.4 Restate `T_REF`, `N_REF`, `U_REF`, `FALLBACK_*` and the model labels without the 61 km claims; values unchanged
- [x] 2.5 Correct the stale US-1976 values quoted in the comments of the low-altitude rows

## 3. Stagnation-line harness (deep_causality_cfd/verification/qtt_ramc_stagline)

- [x] 3.1 Fly the cited 71 km freestream: Mach 25.9, 217.9 K, q 2.28 kPa; derive velocity and number density
- [x] 3.2 Compare against the station-1 Ka-band anchor; state the 0.93 km offset; keep the ±0.70 allowance and its `[tripwire]` class; add the 61 km station and its `[reference]` lower-bound gate
- [x] 3.3 Re-pin the Park-2T controller and T₂-band tripwires from the corrected run; report any gate the corrected physics fails
- [x] 3.4 Regenerate `baseline.txt`; update the harness README

## 4. Corridor

- [x] 4.1 Re-size `TRUTH_V0` at fixed flight-path angle so the vehicle crosses 71.93 km at 7.66 km/s; record the measured crossing speed
- [x] 4.2 Gate (2): `n_e` at the 71.93 km crossing (from the trace) against the anchor ±0.70 dec `[tripwire]`
- [x] 4.3 Report `n_e` and speed at the 61 km passage beside the flight lower bound (not gated; trajectories differ below the anchor)
- [x] 4.4 Report the descent's peak `n_e` and altitude; gate that it lies inside the denied window `[tripwire]`
- [x] 4.5 Re-pin moved tripwires (exit-altitude band, aim offset sizing) from the corrected run, each with its measured value
- [x] 4.6 Regenerate `output.txt` and the trace CSVs

## 5. Weather and retropropulsion

- [x] 5.1 Rerun the weather table; re-pin moved tripwires; regenerate `output.txt`, `weather_table.csv`, traces and audit logs
- [x] 5.2 Rerun the retropropulsion descent on the new table; re-pin moved tripwires or report failures; regenerate `output.txt` and CSVs
- [x] 5.3 Rerun the Park-2T harness (cross-reference text) and the four other CFD examples; regenerate their artifacts

## 6. Documentation

- [x] 6.1 Crate README and `verification/README.md` (stagline, Park-2T, summary table, references) from the regenerated baselines
- [x] 6.2 Plasma-blackout READMEs (family, corridor, weather, retropropulsion) from the regenerated outputs
- [x] 6.3 Website data and pages that quote corridor, weather, retropropulsion or RAM-C numbers
- [x] 6.4 Preprint notes and tables under `papers/counterfactual_fluid_dynamics/`
- [x] 6.5 List remaining downstream consumers that quote changed numbers (tutorial video text) for the owner

## 7. Verification

- [x] 7.1 `cargo build`/`test`/`clippy` for `deep_causality_cfd` and `avionics_examples`; `make check_examples`
- [x] 7.2 Grep for the superseded figures (61 km anchor, 1.3e21 row, 73.2 km, 2.07 m, 44.4 s, +0.35 dec) outside archives
- [x] 7.3 `openspec validate fix-ramc-anchor-freestream --strict`
