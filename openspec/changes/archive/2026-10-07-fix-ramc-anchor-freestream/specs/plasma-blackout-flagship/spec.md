## MODIFIED Requirements

### Requirement: Coupled validation gate

Stage 5 SHALL gate the **coupled** behavior end-to-end (real electron density → real blackout window → real INS
drift → reacquisition), which is the milestone that could not run before Stage 1 landed the marcher behind the
interface. Blackout **onset and exit SHALL be flow-resolved events** found by the run as the evolved sheath state
crosses the comms cutoff (ordered onset → nonzero dwell → exit), not station switches. With the finite-rate
network in the coupling, the electron density SHALL be an **uncalibrated prediction** compared with the RAM-C II
flight at matched altitude and speed: at the corridor's 71.93 km crossing against the station-1 Ka-band datum
(N_e,pk = 0.63 × 1.287e-8 × f² cm⁻³ with f in Hz; at f = 3.5e10 Hz, N_e,pk ≈ 9.93e12 cm⁻³ ≈
9.93e18 m⁻³; Grantham 1970, p. 18)
within the ±0.70-decade chemistry-spread allowance, as a `[tripwire]` gate, because the allowance is a
chosen width rather than a flight bound. The truth entry velocity SHALL be sized so the vehicle crosses 71.93 km at the
flight's 7.66 km/s. Below the anchor the corridor's light probe and RAM-C II fly different trajectories, so
the corridor SHALL report its 61 km `n_e` and speed beside the flight's lower bound without gating them; the
lower-bound gate belongs to the stagnation-line harness at RAM-C II's own 61 km freestream. The run SHALL report the descent's peak electron
density and its altitude, and SHALL gate that the peak lies inside the flow-resolved denied window. The
**blackout exit altitude SHALL be a gated prediction** reported against the RAM-C II flight window (the flight
stayed dark to roughly 25 to 30 km), with the onset altitude recorded as a prediction in the report. Branch
miss distances SHALL be trajectory-derived, with the committed steered branch measurably diverging from the
zero-bank branch. `[reference]` bounds SHALL come from cited flight data and SHALL NOT be tuned; `[tripwire]`
bounds SHALL be pinned from measurement and labelled as such. The gate SHALL exit nonzero on any regression,
and the run SHALL stay inside the minutes-not-hours wall-clock budget.

#### Scenario: Coupled blackout timing drives the navigation outcome
- **WHEN** the RAM-C descent is run with the compressible marcher and the finite-rate network behind the
  coupling interface
- **THEN** the blackout onset and exit derive from the evolved `n_e` crossing the cutoff (onset before exit,
  nonzero dwell), `n_e` at the 71.93 km crossing is compared with the station-1 Ka-band datum, the
  crossing speed is within 0.5 % of 7.66 km/s, the 61 km `n_e` and speed are reported beside the flight's
  lower bound, the exit altitude is reported against the RAM-C II flight window, and the INS drift and
  reacquisition follow that window

#### Scenario: The descent's peak is visible
- **WHEN** the corridor finishes
- **THEN** its report prints the peak `n_e` over the whole descent and the altitude where it occurred, and a
  gate fails if that peak lies outside the flow-resolved denied window

#### Scenario: The four required elements are all present in one process
- **WHEN** the flagship runs
- **THEN** regime change, multiphysics coupling, counterfactual branching (with trajectory-derived miss
  distances), and tensor-network compression are all exercised in the single `CausalFlow`, with the provenance
  log showing the active regime and evidence per step
