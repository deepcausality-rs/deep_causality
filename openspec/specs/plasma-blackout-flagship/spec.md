# plasma-blackout-flagship Specification

## Purpose
TBD - created by archiving change add-plasma-blackout-corridor. Update Purpose after archive.
## Requirements

### Requirement: Flagship plasma-blackout-corridor example

Stage 5 SHALL provide the flagship example, **written in the Stage-4 Flow DSL** (its central control loop in the
~10–30-line budget), wiring the corridor §4 chain [1]–[7] into one auditable `CausalFlow`:
state/context ingest → regime classifier → coupling layer → tensor-compressed rollout (via the **compressible
two-temperature carrier**) + KS trajectory arc (Stage 2) → counterfactual bank-angle branches with
**bank-steered lift** → **cybernetic bounded-correction gate** (`CyberneticLoop`, replacing the corridor §4 [6]
Effect Ethos gate for the latency-bound inner loop) → provenance log. The corridor SHALL fly **one continuous
descent** over a scheduled freestream driven by the truth trajectory (no station switches), with post-shock
quantities (`T_tr`, `T_ve`, `n_tot`) read from the evolved marched state rather than reconstructed, and station
constants reduced to freestream plus geometry. It is a runnable demonstration verified by its gate (not unit
tests), driven over the representative RAM-C trajectory, and SHALL label every simplification (representative
trajectory, synthetic sensors, 1PN clock, compressed-time mapping, point-mass 3-DOF lift) as
`[holds under precondition]`, not a hidden gap.

#### Scenario: End-to-end coupled run
- **WHEN** the flagship example is run
- **THEN** it ingests state, classifies the regime, couples the governing models, rolls out the compressed
  compressible flowfield + KS trajectory through one continuous descent, branches counterfactual bank angles
  whose steered trajectories diverge, applies the cybernetic bounded-correction gate (correction inside the
  verified safety envelope, actuating the lift), and emits a full provenance log, all in one `CausalFlow`

#### Scenario: The descent is continuous, not station-switched
- **WHEN** the flagship descends through the flight envelope
- **THEN** the freestream follows the truth trajectory through the atmosphere schedule in a single coupled run,
  and no per-station post-shock constants are supplied

### Requirement: Coupled validation gate

Stage 5 SHALL gate the **coupled** behavior end-to-end (real electron density → real blackout window → real INS
drift → reacquisition), which is the milestone that could not run before Stage 1 landed the marcher behind the
interface. Blackout **onset and exit SHALL be flow-resolved events** found by the run as the evolved sheath state
crosses the comms cutoff (ordered onset → nonzero dwell → exit), not station switches. The peak electron density
SHALL hold the RAM-C II anchor band as the descent sweeps the 61 km condition. Branch miss distances SHALL be
trajectory-derived, with the committed steered branch measurably diverging from the zero-bank branch. Bands
SHALL be honest (~2–3× on the electron-density-anchored quantities; a 5× band on the anchor itself). The gate
SHALL exit nonzero on any regression, and the run SHALL stay inside the minutes-not-hours wall-clock budget.

#### Scenario: Coupled blackout timing drives the navigation outcome
- **WHEN** the RAM-C descent is run with the compressible marcher behind the coupling interface
- **THEN** the blackout onset and exit derive from the evolved `n_e` crossing the cutoff (onset before exit,
  nonzero dwell), the peak `n_e` holds the anchor band at the 61 km passage, the INS drift and reacquisition
  follow that window, and the carried clock matches the FS-3 anchor on fix-return

#### Scenario: The four required elements are all present in one process
- **WHEN** the flagship runs
- **THEN** regime change, multiphysics coupling, counterfactual branching (with trajectory-derived miss
  distances), and tensor-network compression are all exercised in the single `CausalFlow`, with the provenance
  log showing the active regime and evidence per step
