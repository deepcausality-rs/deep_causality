# compressible-reacting-qtt-marcher Specification

## Purpose
TBD - created by archiving change add-cfd-compressible-qtt-marcher. Update Purpose after archive.
## Requirements
### Requirement: Compressible marcher implements the Marcher trait

`deep_causality_cfd` SHALL provide a compressible QTT marcher implementing the existing `Marcher` trait (a
`CausalTensorTrain`-valued conservative state), assembled from `compressible-qtt-flux` +
`qtt-imex-time-integration` + the body-fitted coordinate + (where used) `qtt-shock-fitting`, so it slots into
the `CfdFlow` DSL like `QttIncompressible2d`/`QttImmersed2d`.

#### Scenario: Drop-in marcher
- **WHEN** the compressible marcher is driven through the `CfdFlow` march pipeline
- **THEN** it advances the conservative state and emits a `Report` of the same shape as the incompressible
  marchers

### Requirement: Tier-A reacting/ionization sources reused unchanged

The marcher SHALL carry the **Tier-A** Park-2T reacting/ionization **LER `PhysicsStage`s** and the
`BlackoutTrigger` **unchanged**, advancing the species and `T_tr`/`T_ve` energies on the compressible rollout
via TT-cross. `T_tr`/`T_ve` SHALL be **real transported states** (retiring the Tier-A recovery-temperature
reconstruction).

#### Scenario: Tier-A stages run on the compressible marcher
- **WHEN** the `IonizationStage`/`EosStage`/`BlackoutTrigger` from `add-park2t-blackout-tier-a` are composed
  onto the compressible marcher
- **THEN** they type-check and run with no modification, reading the transported `T_tr`/`T_ve` instead of the
  reconstruction

#### Scenario: Transported temperature replaces the reconstruction
- **WHEN** the post-shock temperature is read by the ionization stage
- **THEN** it is the marched conservative-energy-derived `T_tr`, not the Rankine–Hugoniot + ½|u|² recovery
  reconstruction Tier-A used

### Requirement: Staged geometry — stagnation line, 2-D, 3-D forebody

The marcher SHALL be exercised at increasing geometry fidelity: a quasi-1-D **stagnation line** (the RAM-C
slice), a **2-D** blunt-body bow shock, and a **3-D forebody** configuration, each in the body-fitted
coordinate so the **forebody** rank stays bounded. The 3-D wake is out of scope (it needs turbulence, a
non-goal, and is downstream of the blackout sheath).

#### Scenario: Bounded forebody rank at each geometry stage
- **WHEN** the marcher is run at the 1-D, 2-D, and 3-D-forebody stages in the body-fitted coordinate
- **THEN** the forebody bond dimension stays bounded (consistent with the measured `χ ~ O(10)` fitted result),
  not the Cartesian `χ ~ √side`

