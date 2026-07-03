# blackout-composition Specification

## Purpose
TBD - created by archiving change add-plasma-blackout-corridor. Update Purpose after archive.
## Requirements

### Requirement: Regime classifier

Stage 3 SHALL provide the regime classifier that reads the Stage-0 classifier-input contract (Knudsen number,
electron density / ionization fraction, GNSS state) and selects the governing models (airflow closure, gas
model, timing mode) — the corridor §4 [2]/[3] coupling, generalizing the `grmhd/select_metric` pattern. The
threshold(s) it compares against SHALL be config; the indicators SHALL be computed from state.

#### Scenario: Neutral→plasma and GNSS→denied transitions fire from state
- **WHEN** the transported `n_e` crosses the plasma-frequency threshold (blackout flag) and Knudsen crosses the
  continuum/slip threshold
- **THEN** the classifier selects the ionized gas model + INS-only timing, and logs the regime change

### Requirement: Counterfactual bank-angle branches

Stage 3 SHALL spawn counterfactual bank-angle branches via `continue_with`, each a coupled rollout to the landing
ellipse returning (peak heat flux, integrated thermal load, miss distance, blackout dwell), each propagating
predict-only through the blackout window.

#### Scenario: Branches yield distinct outcomes
- **WHEN** N bank-angle profiles are branched through the blackout window
- **THEN** each returns its own (heat, thermal load, miss distance, dwell) tuple, and two histories yield two
  distinct clock/position outcomes (path-dependence)

### Requirement: Cybernetic bounded-correction gate and provenance

Stage 3 SHALL apply the corrective safety gate as a **cybernetic loop**
(`deep_causality_haft::CyberneticLoop::control_step`), not the Effect Ethos layer — a deterministic,
self-contained, real-time-capable feedback step that compiles to low-overhead machine code (the Effect monad is
unsuitable for the latency-bound guidance inner loop). The loop's five components map to the corridor: **Sensor
(S)** = the sensed coupled state (heat flux, g-load, miss distance) from the counterfactual rollouts; **Belief
(B)** = the estimated trajectory / thermal-margin state (`observe_fn: S × &C → B`); **Context (C)** = the
**verified safety envelope** (thermal-corridor bounds, g-load limit, crewed physiological / ROE limits);
**Action (A)** = the bank-angle correction (`decide_fn: B × &C → A`); **Entropy (E)** = an unrecoverable
envelope breach (no safe action). It SHALL emit the Stage-0 provenance schema to the `EffectLog` for every step
and decision.

The **bounded-correction invariant** SHALL hold: every returned Action `A` lies within the safety envelope `C`
by construction — a correction that would exit the envelope is clamped to the envelope boundary, and if no safe
action exists the step returns `E` rather than an out-of-envelope command. The correction is therefore always
inside a verified safety envelope, deterministically.

#### Scenario: Correction stays inside the verified safety envelope
- **WHEN** the decided bank-angle correction would drive the trajectory outside the thermal-corridor / g-load
  envelope
- **THEN** the returned Action is clamped to the envelope boundary (never an out-of-envelope command), and the
  clamp is recorded in the `EffectLog`

#### Scenario: Unrecoverable breach returns entropy, not an unsafe action
- **WHEN** no bank-angle action keeps the trajectory inside the safety envelope
- **THEN** the control step returns the Entropy `E` (an unrecoverable-breach signal), and the `EffectLog` records
  the breach and the evidence each candidate rested on — no unsafe Action is ever emitted

#### Scenario: Deterministic real-time step
- **WHEN** the control step is invoked on identical Sensor + Context inputs
- **THEN** it produces the identical Action with no Effect-monad allocation on the hot path (a low-overhead,
  real-time-suitable feedback step)
