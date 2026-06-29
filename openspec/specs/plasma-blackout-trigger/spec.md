# plasma-blackout-trigger Specification

## Purpose
TBD - created by archiving change add-park2t-blackout-tier-a. Update Purpose after archive.
## Requirements
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
associative-ionization reaction rate (N + O → NO⁺ + e⁻), computed from state — not a free fitted constant — as
the rate-limiting onset timescale. The equilibrium ionization target SHALL include the ionized species so that
it can be nonzero, and SHALL carry electron-impact-produced electrons as well as NO⁺ because RAM-C (~7.6 km/s)
sits in the mixed associative + electron-impact band (Aiken–Carter–Boyd 2025).

#### Scenario: Nonequilibrium lag is present
- **WHEN** the driving temperature is ramped faster than `τ_ion`
- **THEN** the ionization state visibly lags its equilibrium target (`α ≠ α_eq` during the transient), and the
  gap closes as the ramp slows

#### Scenario: Electron-density overshoot signature
- **WHEN** the driving temperature rises then relaxes along a streamline (a compression followed by expansion)
- **THEN** the carried `n_e` transiently overshoots its instantaneous local-equilibrium value before relaxing
  back down — the documented nonequilibrium overshoot (Lin et al. 1962; Aiken–Carter–Boyd 2025) that a
  memoryless equilibrium model cannot produce (asserted qualitatively, given the Tier-A reconstruction)

#### Scenario: Electrons are produced
- **WHEN** the ionization stage runs at a reentry-representative temperature with the ionized-species target
- **THEN** the electron density is strictly positive (the target is not identically zero)

### Requirement: The QTT marcher hosts the between-step coupling seam

The `PhysicsStage` coupling seam SHALL be generalized off the DEC-specific context — a `FlowSnapshot<R>`
read-view trait (`dt`, `step`) over which `PhysicsStage` is generic (static dispatch, no `dyn`) — so the same
stages run under both the DEC `MarchRun` and the QTT `QttMarchRun`. `QttMarchRun` SHALL gain a between-step
coupling host that, each step, publishes the primary-state projections its coupling needs (e.g. a per-cell
`"speed"` field, dequantized from the tensor-train state), transports the reacting scalar fields via
`QttImmersed2d::advance_scalar`, applies the coupling, and reads back `Ambient`. The QTT **solver math (the
spectral-projection / Brinkman `advance`) SHALL NOT change**.

#### Scenario: One stage, two marcher hosts
- **WHEN** an LER stage built against the `FlowSnapshot` seam is composed onto the QTT march
- **THEN** it runs under `QttMarchRun` with no modification (it reads only `dt` and named scalar fields), exactly
  as it would under the DEC `MarchRun`

#### Scenario: Reacting scalars are transported as tensor trains
- **WHEN** the QTT march advances a step with the ionization coupling wired in
- **THEN** the reacting scalar fields are advected by `advance_scalar` (staying tensor trains) and updated
  pointwise by the LER stage, and the solver's spectral projection is unchanged

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

