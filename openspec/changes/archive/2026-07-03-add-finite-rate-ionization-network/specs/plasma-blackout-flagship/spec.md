## MODIFIED Requirements

### Requirement: Coupled validation gate

Stage 5 SHALL gate the **coupled** behavior end-to-end (real electron density → real blackout window → real INS
drift → reacquisition), which is the milestone that could not run before Stage 1 landed the marcher behind the
interface. Blackout **onset and exit SHALL be flow-resolved events** found by the run as the evolved sheath state
crosses the comms cutoff (ordered onset → nonzero dwell → exit), not station switches. With the finite-rate
network in the coupling, the peak electron density at the 61 km passage SHALL be an **uncalibrated prediction**
holding a band pinned from the stagnation-line measurement (expectation ~3x, replacing the granted 5x band of
the calibrated surrogate), and the **blackout exit altitude SHALL be a gated prediction** compared against the
RAM-C II flight window (the flight stayed dark to roughly 25 to 30 km), with the onset altitude recorded as a
prediction in the report. Branch miss distances SHALL be trajectory-derived, with the committed steered branch
measurably diverging from the zero-bank branch. Bands SHALL be honest and pinned from measurement, not tuned.
The gate SHALL exit nonzero on any regression, and the run SHALL stay inside the minutes-not-hours wall-clock
budget.

#### Scenario: Coupled blackout timing drives the navigation outcome
- **WHEN** the RAM-C descent is run with the compressible marcher and the finite-rate network behind the
  coupling interface
- **THEN** the blackout onset and exit derive from the evolved `n_e` crossing the cutoff (onset before exit,
  nonzero dwell), the uncalibrated peak `n_e` holds its pinned band at the 61 km passage, the exit altitude is
  gated against the RAM-C II flight window, and the INS drift and reacquisition follow that window

#### Scenario: The four required elements are all present in one process
- **WHEN** the flagship runs
- **THEN** regime change, multiphysics coupling, counterfactual branching (with trajectory-derived miss
  distances), and tensor-network compression are all exercised in the single `CausalFlow`, with the provenance
  log showing the active regime and evidence per step
