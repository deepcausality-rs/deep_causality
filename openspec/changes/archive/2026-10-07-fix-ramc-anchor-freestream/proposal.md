## Why

The RAM-C II validation anchor of the plasma-blackout examples and the stagnation-line harness rests on
a freestream condition that does not exist. The shared atmosphere's "61 km" row carries the US-1976
number density of 71.9 km (1.3e21 m⁻³) with a temperature near 61 km (250 K). The stagnation-line
harness calls the same condition "~71 km" and pairs it with an independent Mach 25. The flight datum
it is compared against, ~1e19 m⁻³, is the station-1 Ka-band critical-density crossing that Grantham
(1970, p. 18) places at 236 000 ft (71.93 km); below that altitude the nose plasma is overdense, so at
61 km the flight gives only a lower bound. The mid-atmosphere rows between 30 and 75 km are scaled
down from US-1976 by up to 4.4×, and `DescentSchedule::sample` interpolates number density linearly
across rows 14–16 km apart, overstating it by about 2× between rows. The reported agreement
(+0.35 decades) is therefore a comparison at matched density under a wrong altitude, inside a descent
flown through a distorted atmosphere.

## What Changes

- **BREAKING (behaviour):** `DescentSchedule::sample` interpolates `n_tot` log-linearly between rows
  (exact for an isothermal hydrostatic layer); temperature and sound speed stay linear.
- The shared `ATMOSPHERE` table is replaced by U.S. Standard Atmosphere 1976 rows at 1 km spacing,
  0–90 km, generated from the standard's defining constants (Tables 2 and 4) and checked against its
  Table I.
- The RAM-C II anchor becomes the station-1 (x/D = 0.15) Ka-band crossing: N_e,pk = 0.63 ×
  1.287e-8 × f² cm⁻³ with f = 3.5e10 Hz (35 GHz) = 9.93e12 cm⁻³ = 9.93e18 m⁻³ at 71.93 km, with the same value as a lower bound at lower
  altitudes down to the end of the primary data period (56.39 km).
- The stagnation-line harness flies the cited RAM-C II 71 km freestream (Mach 25.9, 217.9 K,
  q = 2.28 kPa; velocity and number density derived) instead of the hybrid condition.
- The stagnation-line harness also flies the cited 61 km freestream and gates the flight's lower bound
  there (`[reference]`). The corridor gates `n_e` at its 71.93 km crossing against the anchor, reports
  its 61 km value beside the lower bound, and reports and gates the descent's peak `n_e` and its
  altitude. The truth entry velocity is re-sized so the vehicle crosses 71.93 km at RAM-C's
  7.66 km/s.
- `[tripwire]` bounds that the corrected physics moves are re-pinned from the corrected run and
  recorded; `[reference]` gates are derived from cited data and are not tuned.
- All plasma-blackout examples, the four other CFD examples, the stagnation-line and Park-2T
  harnesses, and every downstream document and website data file are regenerated from the new
  outputs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `full-descent-atmosphere`: the table becomes US-1976 throughout, and the sampler interpolates
  number density log-linearly.
- `plasma-blackout-flagship`: the coupled validation gate compares at the 71.93 km anchor, adds the
  61 km lower bound, and gates the descent's peak.
- `compressible-qtt-validation`: the stagnation-line verification runs at the cited RAM-C II 71 km
  freestream and compares against the station-1 Ka-band anchor.

## Impact

- `deep_causality_cfd/src/types/flow_config/compressible_march_config.rs` (sampler) and its test.
- `deep_causality_cfd/verification/qtt_ramc_stagline/` and `qtt_park2t_blackout/` (config, gates,
  README, baseline).
- `examples/avionics_examples/src/shared/{constants,world}.rs`; the corridor, weather and
  retropropulsion examples (constants, gates, `output.txt`, CSVs, audit logs).
- Documentation: `deep_causality_cfd/README.md`, `verification/README.md`, the plasma-blackout READMEs,
  `website/cfd/src/data/*` and the pages that quote these numbers, and the preprint notes under
  `papers/counterfactual_fluid_dynamics/`.
- Every headline number quoted from the three examples changes (onset, dwell, miss, wall clock).
