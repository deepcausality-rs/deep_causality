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

### Requirement: Second typed command channel for throttle

`CoupledField` SHALL carry a typed throttle command channel beside the existing bank
`control_action`: `throttle_action: Option<R>` with `throttle_action()` and
`set_throttle_action()` accessors mirroring the bank channel's shape. The channel SHALL default
to `None` so every existing coupling is unaffected, and the bank channel SHALL keep its current
name and semantics unchanged. The two channels make the command bus two-axis by construction,
even though no Tier-A step ever has both axes live (the burn flies on-axis with the bank idle).

#### Scenario: The throttle channel round-trips beside the bank channel

- **WHEN** a stage writes a throttle command with `set_throttle_action` and another stage reads
  `throttle_action()` on the same step, while a bank command sits in `control_action`
- **THEN** both commands are read back independently, and a field that never saw a throttle
  write reports `throttle_action() == None`

#### Scenario: Existing couplings are unaffected

- **WHEN** the corridor's existing coupling stack runs over a field on which no stage writes the
  throttle channel
- **THEN** behavior is identical to before the channel existed, with no stage observing anything
  but `None`

### Requirement: Additive aero-force composition

`CoupledField` SHALL provide an additive force helper, `add_aero_force(delta: [R; 3])`, that
reads the current aero force (treating `None` as zero), adds `delta` component-wise, and writes
the sum back — so a thrust term composes with the lift stage's ④ vector instead of clobbering
it through the overwriting `set_aero_force`. Additive force producers MUST compose after the
④-writing lift stage and before the force consumers (`SuttonGravesLoads`-class loads, the truth
propagator, and the navigation kick), so every consumer sees the same summed vector.

#### Scenario: Thrust adds to lift instead of replacing it

- **WHEN** a lift stage writes `[dx, dy, dz]` via `set_aero_force` and a downstream stage calls
  `add_aero_force([tx, ty, tz])`
- **THEN** `aero_force()` returns `[dx+tx, dy+ty, dz+tz]`, and calling `add_aero_force` on a
  field whose force channel is `None` yields exactly the delta

#### Scenario: All force consumers see one summed vector

- **WHEN** a coupled step composes lift, an additive thrust producer, loads, truth propagation,
  and navigation in the pinned order
- **THEN** the loads stage, the truth kick, and the nav predict all read the identical summed
  force vector for that step

### Requirement: Propulsion state rides the coupled field

The propulsion coupling contract SHALL carry `mass`, `propellant`, and `ignited` as named
scalar fields on `CoupledField` (pinned names: `"mass"`, `"propellant"`, `"ignited"`), and the
commanded throttle SHALL enter a world as the published constant `"commanded_throttle"` through
the existing `publish_constant` mechanism — the same seam as `"commanded_bank"` — so a
counterfactual branch carries its throttle intervention with no new machinery. Before ignition
the carried mass MUST equal the corridor's implied constant-mass bundle (the `CDA_OVER_M`
normalization), so Act-1 force normalization is unchanged by carrying mass as state.

#### Scenario: A world publishes its throttle like its bank

- **WHEN** a world config is built with `.publish_constant("commanded_throttle", v)` and marched
- **THEN** the field carries the scalar `"commanded_throttle" == v` at the top of every coupled
  step, exactly as `"commanded_bank"` behaves today

#### Scenario: Propulsion state survives the pause snapshot

- **WHEN** a coupled march carrying `"mass"`, `"propellant"`, and `"ignited"` scalars is paused
  and its state exported
- **THEN** the snapshot carries all three fields (the scalars vec serializes wholesale), and a
  resumed march reads them back unchanged

### Requirement: Inert A0 propulsion stub satisfies the contract

A stub propulsion `PhysicsStage` (`PropulsionStub`) SHALL satisfy the propulsion coupling
contract behind the seam the M3 production stages (`RetroThrust`, `PlumeObstruction`) will fill,
so downstream consumers build and unit-validate against fixed seams. At commanded throttle ≤ 0
the stub MUST be strictly inert: no force write, no scalar mutation, no log entry — a coupled
step with the stub composed is bit-identical to one without it. At commanded throttle > 0 the
stub SHALL exercise every seam: deplete `"propellant"` via the propellant-mass-flow kernel,
reduce `"mass"` accordingly, set `"ignited"`, add `−T/m·v̂` into the force channel via
`add_aero_force`, and apply the A0 force-channel drag decrement through the existing
Jarvinen–Adams kernels (`srp_thrust_coefficient`, `srp_preserved_drag_fraction`). The A0
correlation is the **drag authority** per the measured de-risk verdict
(`openspec/notes/cfd-plasma-retropulsion/derisk-verdict.md`, AMBER; the 2026-07-17 addendum
measured **both** coupling models on the same harness — the pinned-envelope imprint, which
shields monotonically, and a momentum-carrying jet (`studies/srp_momentum_jet`), which reads as
drag *augmentation* — and attributed the missing Jarvinen–Adams collapse to the harness
(dissipation floor + domain), not the coupling-model class; tensor-train compression was
measured innocent of the accuracy limit). A0 is therefore the **committed** closure, not a
placeholder awaiting a stronger imprint: a marched-layer imprint (the landed `ForcingRegion`
seam), when M3 composes one for state realism, MUST NOT replace the correlation as the force
channel's drag closure — the field carries coupling realism, the correlation carries the drag.
Swapping the stub for the production stages SHALL require no change to any consumer stage.

#### Scenario: Zero throttle is bit-identical inertness

- **WHEN** a corridor-class coupled march runs once with the plain stack and once with
  `PropulsionStub` composed, both at zero commanded throttle
- **THEN** the two runs' reports, final coupled fields, and provenance logs are bit-identical

#### Scenario: Nonzero throttle exercises every seam

- **WHEN** the stub runs one step with `"commanded_throttle" > 0`, a carried mass, and a lift
  stage's force already on the channel
- **THEN** propellant and mass decrease by `ṁ·dt`, `"ignited"` is set, the force channel holds
  lift plus thrust plus the A0 drag decrement, and the depletion is consistent with the
  propellant-mass-flow kernel's value

#### Scenario: The seam swap changes no consumer

- **WHEN** the stub is replaced behind the same contract by the production propulsion stages
- **THEN** no consumer stage (loads, truth, navigation, telemetry, gate) changes code

