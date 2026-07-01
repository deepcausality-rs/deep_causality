## ADDED Requirements

### Requirement: Flagship plasma-blackout-corridor example

Stage 5 SHALL provide the flagship example, **written in the Stage-4 Flow DSL** (its central control loop in the
~10–30-line budget), wiring the corridor §4 chain [1]–[7] into one auditable `CausalFlow`:
state/context ingest → regime classifier → coupling layer → tensor-compressed rollout (via the Stage-1 marcher)
+ KS trajectory arc (Stage 2) → counterfactual bank-angle branches → **cybernetic bounded-correction gate**
(`CyberneticLoop`, replacing the corridor §4 [6] Effect Ethos gate for the latency-bound inner loop) → provenance
log. It is a
runnable demonstration verified by its gate (not unit tests), driven over the representative RAM-C trajectory,
and SHALL label every simplification (representative trajectory, synthetic sensors, 1PN clock, optional
finite-rate chemistry) as `[holds under precondition]`, not a hidden gap.

#### Scenario: End-to-end coupled run
- **WHEN** the flagship example is run
- **THEN** it ingests state, classifies the regime, couples the governing models, rolls out the compressed
  flowfield + KS trajectory, branches counterfactual bank angles, applies the cybernetic bounded-correction gate
  (correction inside the verified safety envelope), and emits a full provenance log — all in one `CausalFlow`

### Requirement: Coupled validation gate

Stage 5 SHALL gate the **coupled** behavior end-to-end — real electron density → real blackout window → real INS
drift → reacquisition — which is the milestone that could not run before Stage 1 landed the marcher behind the
interface. Bands SHALL be honest (~2–3× on the electron-density-anchored quantities). The gate SHALL exit nonzero
on any regression.

#### Scenario: Coupled blackout timing drives the navigation outcome
- **WHEN** the RAM-C arc is run with the real marcher behind the coupling interface
- **THEN** the blackout window onset/duration derives from the transported `n_e` (within the ~2–3× band), the INS
  drift and reacquisition follow that window, and the carried clock matches the FS-3 anchor on fix-return

#### Scenario: The four required elements are all present in one process
- **WHEN** the flagship runs
- **THEN** regime change, multiphysics coupling, counterfactual branching, and tensor-network compression are all
  exercised in the single `CausalFlow`, with the provenance log showing the active regime and evidence per step
