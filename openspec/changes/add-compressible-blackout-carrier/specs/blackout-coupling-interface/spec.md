## MODIFIED Requirements

### Requirement: Extend the existing coupling seam with the navigation channels

`deep_causality_cfd` SHALL extend the existing `.couple` seam (`types/flow/coupling.rs`: `CoupledField<R>`,
`PhysicsStage<D,R>`, `StepContext`, and `BlackoutTrigger`) so it carries the physics→navigation coupling item ④
names — rather than inventing a new interface. It SHALL add to `CoupledField` (or `Ambient`) an **aero-force
(Cartesian vector) channel** the trajectory kick reads and a **control/action channel** the correction writes,
alongside the existing named scalar fields (`speed`, `alpha`, `n_e`, …). The aero-force channel SHALL carry the
**full 3-vector aero acceleration** — drag along the negative velocity direction plus, when a lift stage is
composed, a lift component rotated about the velocity vector by the bank angle — not a drag-only kick. The
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
