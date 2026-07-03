## Context

The corridor's chemistry sits at lever 3 of the chemistry-fidelity ladder
(`openspec/notes/plasma-blackout/gap-3/chemistry-fidelity-gap.md`): lever 1 (T_ve-controlled
ionization) shipped and took the stagnation-line error from 12x to ~1.1x; lever 2 (explicit 3-T
electron energy) was prototyped and closed, leaving the durable insight `T_e = T_ve` for
electron-created-in-the-bath conditions. The current `IonizationStage` relaxes a carried
fraction toward a Saha equilibrium target with a single forward associative rate: calibrated,
forward-only, no exit mechanism. Since `2026-07-02-add-compressible-blackout-carrier`, the
corridor consumes evolved per-cell `T_tr`, `n_tot`, and `pressure_atm`, so a rate network has
real transported inputs. The preparation note
(`openspec/notes/plasma-blackout/finite-rate-cfd-chemistry.md`) fixes the scope: Option A only;
Option B (species transport on tensor trains) is out of scope and rides the validation path.

Constraints: no `dyn`, no `unsafe`, no macros in lib code, precision as a parameter
(`CfdScalar`), tests mirror src with Bazel registration, kernels cite their papers with PDFs in
`deep_causality_physics/papers/`, float literals only in tests and examples, and the corridor's
minutes-not-hours budget.

## Goals / Non-Goals

**Goals:**
- Two-way finite-rate ionization: dissociative recombination as a first-class loss channel, so
  blackout exit and the carried wake become predictions.
- Detailed balance by construction: backward rates derived as `k_b = k_f / K_eq` from the Park
  equilibrium-constant fits, never independently fitted, so the network's fixed point *is* the
  thermodynamic equilibrium at every temperature.
- Uncalibrated validation: the stagnation line and the flagship anchor gate are re-measured with
  no Saha calibration target; bands re-pinned to what the network earns (~3x expectation).
- LER-native integration: unconditionally stable closed-form relaxation per cell, no stiff ODE
  integrator, no per-cell Newton solve.
- The sheath-renewal question settled again under a loss channel, with the decision recorded.

**Non-Goals:**
- Option B: species advection, transported `E_ve`, mixture thermodynamics, and the
  decode-react-encode timing study. Explicitly deferred.
- New chemistry beyond the three named channels (no Zeldovich exchange in the atom pool unless
  the stagnation-line comparison demands it; see D3).
- Any marcher, carrier, or coupled-loop change. The stage slot is the whole footprint.
- Touching `IonizationStage`: the calibrated surrogate remains shipped and untouched for the
  QTT path and the archived corridor behavior.

## Decisions

**D1 — The network is three channels, no more.** (1) `N + O <-> NO+ + e-`: Park associative
ionization forward (the rate the current stage already carries), dissociative recombination
backward via detailed balance. This channel dominates at RAM-C speeds (7 to 8 km/s) and its
reverse is the physical exit mechanism. (2) Electron-impact ionization `N + e- -> N+ + 2e-` and
the O analog: thresholded Arrhenius forms rated at the electron temperature; secondary at these
speeds but it shapes the `n_e` buildup slope, hence the onset altitude. (3) A neutral atom pool
(`N`, `O`) closing channel 1's reactant concentrations. Alternative considered: an 11-species
network with charged diatomics; rejected because above ~9 km/s physics is out of scope and every
added channel dilutes the validation story.

**D2 — Temperatures per channel follow the recorded physics.** Heavy-particle channels (forward
associative, atom-pool equilibrium) run at the Park controller `T_a = sqrt(T_tr * T_ve)`, the
shipped lever-1 mechanism. Electron-involving channels (electron impact, dissociative
recombination) run at `T_e = T_ve`, the lever-2 insight kept when the explicit 3-T was closed.
Alternative considered: everything at `T_a`; rejected because DR rates are
electron-temperature physics and rating them at the hotter controller would systematically
under-predict recombination, which is the channel this change exists to add.

**D3 — The atom pool ships lagged, with the dissociation-rate clock, from day one.** Atom
fractions relax toward their dissociation equilibrium at `T_a` (Park equilibrium-constant fits)
with `tau_pool = 1/(k_d[M])` from the Park dissociation rates, through the same LER kernel: one
more `(target, tau)` pair, no new machinery. A *partial-equilibrium* pool was considered and
rejected by the change's own founding logic: dissociation is the slowest post-shock relaxation
process, so over one residence time the atom fractions sit far below their `T_a` equilibrium,
and an equilibrium pool would over-predict `[N][O]` (and channel 1's production with it),
reintroducing through the back door the equilibrium optimism lever 1 removed for electrons.
Partial equilibrium remains as the limit the lagged pool relaxes toward, and as the correct
behavior wherever the dissociation clock beats the residence clock. Zeldovich exchange stays
out unless the stagnation-line comparison demands it.

**D4 — Detailed balance by construction, from one data source.** All forward rates and all
equilibrium constants come from Park (1990); backward rates are `k_f / K_eq` inside the kernels,
never separate fits. This makes equilibrium recovery a mathematical identity rather than a tuned
coincidence, and it is the property the per-reaction unit tests pin (approach equilibrium from
above and below, land on the same fixed point). One precision the two-temperature rating (D2)
forces: the identity closes only where all temperatures coincide, so the detailed-balance tests
pin the **thermal-equilibrium limit** (`T_tr = T_ve`), and at genuine two-temperature states the
network's fixed point *deliberately* departs from any single-temperature equilibrium. A test
that forced single-temperature behavior at a two-temperature state would be wrong physics
passing a wrong test. Kernel placement: alongside the existing Park-2T
kernels in `deep_causality_physics`, same quantity newtypes, citations in docstrings, PDF in
`papers/`.

**D5 — LER-native coupled-scalar integration.** The network's per-cell update is the LER pattern
generalized: the electron density relaxes toward the network's fixed point `n_e*` (where
production balances loss) with `tau = 1/(k_f[M] + beta * n_e)`, the exact extension the
`IonizationStage` docstring has documented as deferred work since the first corridor build. The
fixed point of channel 1 plus 2 is a quadratic in `n_e` (production linear in reactants, loss
quadratic through `beta * n_e * n_NO+` with quasi-neutrality `n_NO+ ~ n_e`), solved in closed
form; no iteration. Alternative considered: sub-cycled explicit integration of the raw ODE;
rejected because the LER kernel exists precisely to make stiff relaxation unconditionally stable
in one closed-form step, and the whole crate's chemistry rides that contract
(`lagging-equilibrium-relaxation` spec).

**D6 — One new stage, the old one untouched.** `FiniteRateIonizationStage` lands as a sibling of
`IonizationStage` with the same field vocabulary (reads the controller and electron temperatures
and the evolved per-cell density; carries its scalars; writes `"alpha"` and `"n_e"`), including
the optional sheath-renewal mode so the A/B is a one-line toggle. The corridor examples swap the
stage in their shared coupling; the QTT surrogate path and the archived behavior keep
`IonizationStage`. Rollback is swapping back.

**D7 — Validation is uncalibrated, sequenced, and diagnostic: stagnation line first, channel
by channel, corridor second.** The stagnation line is measured first with channel 1 plus the
lagged atom pool alone, then with electron impact enabled, so a band miss is attributable to a
channel rather than to the network as a whole. The
`qtt_ramc_stagline` verification re-measures peak `n_e` against the RAM-C II anchor with the
network and no Saha target; only after that number is pinned do the flagship and weather
examples re-pin (anchor band toward 3x, exit altitude gated against the flight's 25 to 30 km
window, onset recorded as a prediction). Pinning order matters: the corridor's gates must
inherit a measured stagnation-line result, not the other way around.

## Risks / Trade-offs

- [The uncalibrated network misses the ~3x band] → The band is pinned from measurement, not
  hoped; D7's channel-by-channel ordering makes the miss attributable (pool clock, electron
  impact, or rate data); if it traces to the pool, Zeldovich exchange is the designed next step;
  if it traces to rate data, the honest outcome is a wider earned band with the discrepancy
  documented, which still beats a tuned 1.43x.
- [Electron-impact avalanche makes the fixed point stiff or runaway near peak heating] → The
  channel is thresholded (Arrhenius activation ~ionization energy at `T_e = T_ve`, which stays
  well below `T_tr` in the sheath), the LER step is unconditionally stable by contract, and the
  quadratic loss term caps the fixed point. Unit tests pin the frozen limit and the capped
  approach.
- [The sheath-renewal A/B becomes ambiguous (both modes plausible under recombination)] → Same
  policy as the first A/B: keep the mode that matches the stagnation-line closure, record the
  measured numbers for both, label the survivor in `constants.rs`.
- [The weather table's pinned INS numbers shift with the new window] → Expected and accepted;
  the table's *mechanism* gates (drift factor, statistical resolution) are ratio- and
  sigma-based and should survive; the absolute constants get re-pinned from the new
  measurement.
- [Blackout exit moves toward flight but the corridor's 850-step horizons no longer bracket it]
  → The exit moving from 46 km toward 25 to 30 km lengthens the dwell; horizons and budgets are
  example constants and get re-pinned with the gates; the minutes budget has 10x headroom
  against the +5 to +15 percent chemistry cost.

## Migration Plan

Additive throughout. New kernels beside the shipped Park-2T set; a new stage beside the shipped
`IonizationStage`; the examples swap one stage in the shared coupling and re-pin their gate
constants from measurement. The archived corridor changes remain reproducible (they name
`IonizationStage`, which does not move). Rollback at any point is reverting the stage swap in
`avionics_examples::blackout::world` and restoring the previous gate constants.

## Open Questions

- Does explicit sheath renewal survive the introduction of a real loss channel? (Settled
  empirically by the A/B in the tasks; both outcomes are acceptable, the record is mandatory.)
- Does the lagged atom pool with the dissociation-rate clock hold the production band, or does
  the stagnation-line comparison demand Zeldovich exchange in the pool? (Settled by the
  uncalibrated stagline measurement, channel by channel per D7's diagnostic ordering.)
