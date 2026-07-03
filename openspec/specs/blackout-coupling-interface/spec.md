# blackout-coupling-interface Specification

## Purpose
TBD - created by archiving change add-plasma-blackout-corridor. Update Purpose after archive.
## Requirements

### Requirement: Extend the existing coupling seam with the navigation channels

`deep_causality_cfd` SHALL extend the existing `.couple` seam (`types/flow/coupling.rs`: `CoupledField<R>`,
`PhysicsStage<D,R>`, `StepContext`, and `BlackoutTrigger`) so it carries the physics→navigation coupling item ④
names, rather than inventing a new interface. It SHALL add to `CoupledField` (or `Ambient`) an **aero-force
(Cartesian vector) channel** the trajectory kick reads and a **control/action channel** the correction writes,
alongside the existing named scalar fields (`speed`, `alpha`, `n_e`, …). The aero-force channel SHALL carry the
**full 3-vector aero acceleration**, not a drag-only kick: drag along the negative velocity direction plus,
when a lift stage is composed, a lift component rotated about the velocity vector by the bank angle. The
control/action channel SHALL be an **actuating** input: the clamped command written by the correction stage is
read back by the lift stage on the next step and steers the ④ vector, in addition to being audited. Consumers
SHALL remain `PhysicsStage` impls composed on the static cons-tuple (no `dyn`); the marcher continues to publish
its per-step projections into the field, and `BlackoutTrigger` continues to derive the blackout flag from peak
`n_e`.

#### Scenario: The extended field carries force and action
- **WHEN** a marcher step publishes into the `CoupledField` and a stage reads it
- **THEN** the aero-force vector and the derived blackout flag are readable by the trajectory stage, and a control
  action is writable by the correction stage, through the same seam that already carries the scalar fields

#### Scenario: Consumers stay PhysicsStage impls
- **WHEN** the trajectory, classifier, and correction consume the coupling
- **THEN** each is a `PhysicsStage` composed via `Coupling::between_steps().then(..)`, with no dynamic dispatch
  and no new seam type

#### Scenario: The clamped control action closes the loop
- **WHEN** the correction stage writes a clamped bank command and the lift stage runs on the next step
- **THEN** the ④ aero vector carries the lift component rotated by that clamped command, and both the truth
  propagation and the navigation predict consume the same steered vector

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

### Requirement: Parallel counterfactual fan-out

The paused coupled march SHALL offer a branch fan-out, `continue_branches(worlds, steps)`, that forks once per
world, alternates each fork into its world, and continues every branch. Branches are data-independent by
construction (the fork shares state through `Arc` and takes one copy-on-write clone at its first write), so with
the `parallel` feature the fan-out SHALL run the branches concurrently on scoped fork-join threads
(`deep_causality_par::scoped_map`, `std::thread::scope`; no Rayon, no thread pool). Reports SHALL come back in
world order, each carrying the same `!!ContextAlternation!!` audit entry a manual fork chain produces, and the
results SHALL be identical to the sequential fan-out. Without the feature the fan-out runs inline; the
`MaybeParallel` bounds are vacuous, so serial builds carry no `Send + Sync` requirements.

#### Scenario: The fan-out matches the manual fork chain
- **WHEN** a paused march fans out over N worlds via `continue_branches`
- **THEN** N reports come back in world order, each with its alternation marker, and every report is
  bit-identical to the one a sequential `fork().alternate_context(world).continue_march(steps)` produces

#### Scenario: Serial builds see no thread-safety bounds
- **WHEN** the crate is built without the `parallel` feature
- **THEN** `continue_branches` compiles and runs inline with no `Send + Sync` obligations on the carrier state,
  the coupling stack, or the reports
