## ADDED Requirements

### Requirement: Compressible marcher on the completed 3-D body-fitted coordinate

Stage 1 SHALL complete the compressible-marcher's named open remainders so the 3-D marcher runs on a real
shock-aligned coordinate rather than Cartesian capture: a **3-D body-fitted `MetricProvider`** and the
**dynamic marched-rank re-pin** (Res 5 / D9). This builds on the existing `body-fitted-qtt-coordinate` /
`compressible-reacting-qtt-marcher` capabilities and SHALL hold the curved-shock bond dimension at `χ ~ O(10)`
(constant in resolution) rather than the Cartesian-captured `χ ~ √side`.

#### Scenario: Body-fitted 3-D run holds bounded rank
- **WHEN** a realistically-formed 3-D curved shock is marched on the body-fitted coordinate
- **THEN** the bond dimension stays `O(10)` (constant in resolution), not `√side`, and the marched rank is
  re-pinned across the flux-through-front

### Requirement: Marcher implements the blackout coupling interface

The compressible marcher SHALL implement the Stage-0 `blackout-coupling-interface` through `CfdFlow`, emitting
the **real** per-step aero force, heat flux, transported electron density `n_e`, and the derived blackout flag —
replacing the Stage-0 stub producer with no change to any consumer. The `n_e` SHALL come from the transported
`T_tr` / `T_ve` / species fields (the T_ve-controlled ionization already at ~1.1× of RAM-C), not a
recovery-temperature reconstruction.

#### Scenario: Real marcher output drives consumers unchanged
- **WHEN** the marcher adapter replaces the Stage-0 stub behind the interface
- **THEN** the trajectory engine and classifier consume real force / heat / `n_e` / blackout flag with no code
  change, and the blackout flag fires from the transported `n_e` crossing the plasma-frequency threshold

#### Scenario: Optional finite-rate chemistry firms the band
- **WHEN** chemistry lever 3 (finite-rate ionization network) is enabled
- **THEN** peak `n_e` moves from ~1.1× toward the production band, and it is off by default (not a flagship
  blocker)
