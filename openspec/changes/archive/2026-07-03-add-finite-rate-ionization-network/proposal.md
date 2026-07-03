## Why

The corridor's ionization closure is a calibrated surrogate: a Saha equilibrium target times one
forward associative rate, tuned to reproduce the RAM-C II anchor at the 61 km Mach-25 condition.
It has no loss channel, so the blackout exit is not predicted (the surrogate exits at 46 km
against the flight's 25 to 30 km), the carried wake fraction must be frozen, and every off-anchor
number (including the whole weather-dispersion table) is extrapolation of a fit. Replacing the
closure with a small two-way finite-rate network (lever 3 of the chemistry-fidelity gap,
pre-scoped in `openspec/notes/plasma-blackout/finite-rate-cfd-chemistry.md`) turns onset, exit,
and the peak into predictions from published rate data. Calibration becomes validation. The
prerequisite landed with the compressible carrier: the network consumes evolved per-cell
`T_tr`, `n_tot`, and pressure, not reconstructions.

Honest expectation, stated up front: the printed anchor agreement will likely *worsen* from the
calibrated 1.43x into the 2x to 3x band production finite-rate codes (DPLR, LAURA, US3D) occupy,
while becoming meaningful. The gates are re-pinned to an earned ~3x band instead of a granted 5x.

## What Changes

- New finite-rate kernels in `deep_causality_physics` from the Park (1990) rate tables:
  associative ionization `N + O -> NO+ + e-` with its dissociative-recombination reverse,
  thresholded electron-impact ionization of N and O rated at the electron temperature, Park
  equilibrium-constant curve fits so detailed balance holds by construction, and a
  partial-equilibrium neutral atom pool at the controller temperature. Citations in docstrings,
  source PDFs in `deep_causality_physics/papers/`.
- A new `FiniteRateIonizationStage` in `deep_causality_cfd` replacing the corridor's
  `IonizationStage` closure in the same `PhysicsStage` slot: LER-native integration with
  `tau = 1/(k_f[M] + beta * n_e)`, per-cell over the evolved fields, no stiff ODE solver, no
  per-cell Newton solve. `IonizationStage` remains for the QTT surrogate path; nothing existing
  changes contract.
- The sheath-renewal A/B re-run under recombination: the 268x accumulation that made explicit
  renewal mandatory existed because no loss channel did. Whichever mode survives is kept and the
  reasoning recorded, like the first A/B.
- The stagnation-line verification (`qtt_ramc_stagline`) re-measured against the network with no
  Saha calibration target; the flagship and weather examples re-pinned (anchor band toward 3x,
  blackout exit altitude compared against the RAM-C II window, onset recorded as a prediction).
- No marcher change, no new transported fields; the quasi-steady per-cell picture keeps doing
  the transport bookkeeping (Option B territory is explicitly out of scope).

## Capabilities

### New Capabilities
- `finite-rate-ionization-network`: the two-way reaction network — the physics kernels
  (rates, equilibrium constants, detailed balance) and the LER-native coupled-scalar stage that
  evaluates it per cell on the evolved carrier state.

### Modified Capabilities
- `park2t-ionization-kernels`: the pointwise kernel set gains the reverse
  (dissociative-recombination) and electron-impact channels plus equilibrium-constant fits;
  the validation requirement extends to detailed-balance and frozen-limit checks per reaction.
- `park2t-blackout-validation`: the stagnation-line verification's acceptance changes from
  "calibrated closure reproduces the anchor" to "uncalibrated network predicts the anchor
  within the production band"; the LER gates gain the recombination channel.
- `plasma-blackout-flagship`: the coupled validation gate re-pins — the anchor band tightens
  toward 3x on an uncalibrated prediction, and the blackout exit becomes a gated prediction
  compared against the RAM-C II flight window.

## Impact

- `deep_causality_physics`: new rate kernels and quantity plumbing under the existing Park-2T
  module; new reference PDFs in `papers/`; no changes to shipped kernel signatures.
- `deep_causality_cfd`: new stage in `types/flow/blackout.rs` (or sibling module); the corridor
  examples swap the stage in their shared coupling stack; `IonizationStage` untouched.
- `examples/avionics_examples`: the shared `blackout` module's coupling uses the network stage;
  flagship and weather gates re-pinned; `constants.rs` labels updated (the forward-only
  limitation is removed, the sheath-renewal decision re-recorded).
- `deep_causality_cfd/verification/qtt_ramc_stagline`: re-measured, new pinned numbers.
- Runtime: +5 to +15 percent on the flagship (chemistry is a minor slice of the coupled step);
  the wall-clock gates do not move.
