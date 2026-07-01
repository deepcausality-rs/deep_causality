## ADDED Requirements

### Requirement: Extend the existing coupling seam with the navigation channels

`deep_causality_cfd` SHALL extend the existing `.couple` seam (`types/flow/coupling.rs`: `CoupledField<R>`,
`PhysicsStage<D,R>`, `StepContext`, and `BlackoutTrigger`) so it carries the physics→navigation coupling item ④
names — rather than inventing a new interface. It SHALL add to `CoupledField` (or `Ambient`) an **aero-force
(Cartesian vector) channel** the trajectory kick reads and a **control/action channel** the correction writes,
alongside the existing named scalar fields (`speed`, `alpha`, `n_e`, …). Consumers SHALL remain `PhysicsStage`
impls composed on the static cons-tuple (no `dyn`); the marcher continues to publish its per-step projections
into the field, and `BlackoutTrigger` continues to derive the blackout flag from peak `n_e`.

#### Scenario: The extended field carries force and action
- **WHEN** a marcher step publishes into the `CoupledField` and a stage reads it
- **THEN** the aero-force vector and the derived blackout flag are readable by the trajectory stage, and a control
  action is writable by the correction stage, through the same seam that already carries the scalar fields

#### Scenario: Consumers stay PhysicsStage impls
- **WHEN** the trajectory, classifier, and correction consume the coupling
- **THEN** each is a `PhysicsStage` composed via `Coupling::between_steps().then(..)`, with no dynamic dispatch
  and no new seam type

### Requirement: Stub coupling stage for contract-first construction

A **stub** `PhysicsStage` (mock aero drag + a scheduled blackout window) SHALL satisfy the extended contract so
downstream stages build and unit-validate before the real marcher lands. Swapping the stub for the real
Stage-1 marcher stage SHALL require no change to any consumer stage.

#### Scenario: Stub satisfies the contract
- **WHEN** the stub stage drives the trajectory and classifier stages
- **THEN** they run end-to-end, and replacing the stub with the real marcher stage changes no consumer code

### Requirement: Classifier-input and provenance schema

The extension SHALL define the regime-classifier input contract (Knudsen number, ionization fraction / `n_e`,
GNSS state) as fields on the coupled state, and the provenance record schema (per step: active regime, selected
governing model, carried clock correction, evidence) emitted through the existing `EffectLog`, so Stage-3
composition fills a defined seam rather than introducing a new one.

#### Scenario: Provenance record is populated per step
- **WHEN** a coupled step is composed
- **THEN** an `EffectLog` provenance entry records the active regime, the selected model, and the carried clock
  correction for that step
