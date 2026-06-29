<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

## ADDED Requirements

### Requirement: State-derived recovery temperature

The driving translational temperature `T_tr` SHALL be reconstructed from the flow state the incompressible QTT
solver already produces — `T_tr(x) = T_post − ½|u(x)|²/c_p` — where `T_post` is the post-shock stagnation
temperature obtained from the **configured** flight condition through a **Rankine–Hugoniot normal-shock jump**.
The spatial structure of `T_tr` SHALL come from the computed velocity field; only the scalar flight condition
(`M_∞`, `T_∞`) is config. `T_tr` SHALL NOT be a prescribed field or a hardcoded schedule. The reconstruction
SHALL be documented as a recovery-temperature reconstruction (the Tier-A stand-in), not a true post-shock
thermodynamic path.

#### Scenario: Temperature is emergent, not prescribed
- **WHEN** the freestream velocity is changed and the flowfield re-solved
- **THEN** the reconstructed `T_tr` field changes structurally (it tracks the computed stagnation pattern), not
  merely by a constant rescale

#### Scenario: Rankine–Hugoniot jump sets the magnitude
- **WHEN** the recovery temperature is reconstructed at a flight Mach number of about 25
- **THEN** the peak `T_post` lands in the ~10⁴ K band reported by RAM-C / Park (the mandatory shock jump), not
  the much colder isentropic-recovery value

### Requirement: Ionization and EOS coupling stages on the QTT rollout

`deep_causality_cfd` SHALL provide an `IonizationStage` and an `EosStage` as LER between-step `PhysicsStage`s
that drive the Park-2T kernels over the temperature/species scalar fields of the existing QTT rollout (reusing
`QttImmersed2d::advance_scalar`). `IonizationStage` SHALL relax the carried ionization state toward
`α_eq(ρ, T_tr)` with `τ_ion` and write back `ElectronDensity`. `τ_ion` SHALL be grounded in the dominant
associative-ionization reaction rate (N + O → NO⁺ + e⁻), computed from state — not a free fitted constant. The
equilibrium ionization target SHALL include the ionized species so that it can be nonzero.

#### Scenario: Nonequilibrium lag is present
- **WHEN** the driving temperature is ramped faster than `τ_ion`
- **THEN** the ionization state visibly lags its equilibrium target (`α ≠ α_eq` during the transient), and the
  gap closes as the ramp slows

#### Scenario: Electrons are produced
- **WHEN** the ionization stage runs at a reentry-representative temperature with the ionized-species target
- **THEN** the electron density is strictly positive (the target is not identically zero)

### Requirement: Blackout trigger on the causal seam

`deep_causality_cfd` SHALL provide a `BlackoutTrigger` on the `CausalFlow`/`bind_or_error` seam that maps the
electron density to a plasma frequency (the plasma-frequency kernel), compares it to a **configured** comms
band, and raises a GNSS/comms-denied flag when the plasma frequency exceeds the band. The comparison threshold
is config; the plasma frequency it compares is computed from state.

#### Scenario: Blackout raised above the band
- **WHEN** the computed plasma frequency exceeds the configured comms band
- **THEN** the trigger raises the GNSS/comms-denied flag

#### Scenario: No blackout below the band
- **WHEN** the computed plasma frequency is below the configured comms band
- **THEN** the trigger leaves the link available

### Requirement: Blackout observables

The QTT march run SHALL emit blackout observables — electron density `n_e`, plasma frequency, and blackout dwell
(the time the denied flag is raised) — into the run's report, alongside the existing flow diagnostics.

#### Scenario: Observables emitted
- **WHEN** a march is run with the ionization stage and blackout trigger wired in
- **THEN** the report carries an `n_e` series, a plasma-frequency series, and a blackout-dwell measure

### Requirement: Counterfactual path-dependence

Because the LER ionization state carries memory, two counterfactual branches with different histories SHALL be
able to produce different blackout outcomes from the same nominal endpoint state — the strengthened form of the
dynamic-by-construction test (two histories → two outcomes, not merely two states → two outputs).

#### Scenario: Divergent histories diverge in blackout
- **WHEN** two branches reach a comparable instantaneous state via different temperature histories
- **THEN** their reported electron density / blackout dwell differ, reflecting the carried lag
