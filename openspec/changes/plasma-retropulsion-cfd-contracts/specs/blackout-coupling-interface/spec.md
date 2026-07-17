## ADDED Requirements

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
Jarvinen–Adams kernels (`srp_thrust_coefficient`, `srp_preserved_drag_fraction`). Swapping the
stub for the production stages SHALL require no change to any consumer stage.

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
