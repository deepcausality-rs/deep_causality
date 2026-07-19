## ADDED Requirements

### Requirement: The retropulsion example lands in the family layout with the house file set

The retropulsion example SHALL live at
`examples/avionics_examples/cfd/plasma_blackout/retropulsion/` as the third sibling of the
plasma-blackout family, carrying the family's six-file source skeleton — `main.rs` (the CfdFlow
expression and the exit-code mapping), `model.rs` (worlds, reductions, gate sequences),
`constants.rs` (this example's own tuned knobs only), `utils_print.rs` (every `println!`),
`README.md`, and `output.txt` (a committed capture of a real passing release run) — alongside its
recorded run artifacts. Recorded artifacts are first-class and committed, as the siblings' recorded
CSVs are and as the weather example's gated per-branch `audit/` directory is; a campaign that opts
into the audit sink commits the directory it writes. It MUST be registered by the family's third
`[[example]]` stanza named `plasma_blackout_retropulsion` with the crate-root-relative path
`cfd/plasma_blackout/retropulsion/main.rs`, MUST declare its own per-example `pub type FloatType`
alias in `main.rs` (the per-example alias is deliberate and MUST NOT be consolidated onto the
shared one), and MUST map the merged verdict to the house exit codes: 0 when every gate passes, 1
on a gate regression, 2 on a setup failure. The example folder carries **no tests** — anything
general-purpose belongs in a library crate and is tested there.

#### Scenario: The example runs from the family folder under its own binary name

- **WHEN** `cargo run --release -p avionics_examples --example plasma_blackout_retropulsion` is run
- **THEN** the example executes from `cfd/plasma_blackout/retropulsion/main.rs`, prints its intro
  and per-act blocks through `utils_print`, renders one merged verdict, and exits 0

#### Scenario: A gate regression is distinguishable from a setup failure

- **WHEN** the study returns a verdict whose gates did not all pass, versus when it returns a
  `StudyError`
- **THEN** the first exits 1 with the FAIL lines rendered, and the second exits 2 with the failing
  verb named on stderr

### Requirement: The burn stack is carried from step 0 in one march call

The powered-descent coupling stack SHALL be assembled once as a new `impl PhysicsStage<2,
FloatType>` assembler beside `corridor_coupling` in `examples/avionics_examples/src/shared/world.rs`
and MUST carry the propulsion stages from the first step, inert while the commanded throttle is
zero, so that ignition is a published-command event inside one world and never a coupling-stack
swap. Acts 2 and 3 — the coast, the ignition commit, the burn, and the mid-burn fork — MUST share
one march call, because a coupling stack is fixed per march call and `MarchState` carries the
coupled field but not the marched fluid tensor: a leg boundary at ignition would re-seed the flow
and the fork would then fork a state from which the plume had already been discarded. Leg
boundaries with re-seeds are permitted only at the Act-1/Act-2 blackout exit and at cutoff before
the terminal leg, and each re-seed MUST be logged.

#### Scenario: Ignition happens inside one world

- **WHEN** the commanded throttle rises from zero during the coast-and-burn march call
- **THEN** the same coupling stack that flew the coast carries the burn, the marched field is
  continuous across ignition, and no re-seed entry appears in provenance at the ignition step

#### Scenario: The fork sees a plume-coupled state

- **WHEN** the march is paused mid-burn for the counterfactual fork
- **THEN** the paused field is the one the burn evolved, not a freshly re-seeded field

### Requirement: The propulsion stages compose between the force writer and the force consumers

The stack SHALL compose the burn stages after the ④-writing lift stage (`BankSteeredLift`, which
uses the overwriting `set_aero_force`) and before the first ④ consumer (`SuttonGravesLoads`, the
truth propagator, and the navigation kick), which is the ordering contract stated on
`CoupledField::add_aero_force`. It MUST compose the M3 production pair `RetroThrust` and
`PlumeObstruction`, and MUST NOT compose the M2 `PropulsionStub` alongside them: the stub bundles
thrust, depletion, and the A0 drag decrement in one contract stub, so composing it beside the
production stages applies the thrust term and the propellant/mass depletion **twice**, and
compounds the drag decrement — the A0 decrement is written as `add_aero_force((f − 1)·x)`, which is
a *multiplicative* rescale `x ← f·x` of whatever sits on the axial channel, so applying it twice
yields `f²·D` rather than `f·D`. Nothing in the type system prevents the combination.

#### Scenario: Every force consumer reads one summed vector

- **WHEN** the lift stage, `RetroThrust`, and `PlumeObstruction` have all run in a burn step
- **THEN** `SuttonGravesLoads`, the truth propagator, and the navigation kick each read a single
  aero-force vector carrying lift plus thrust plus the A0 decrement

#### Scenario: The contract stub does not fly beside the production stages

- **WHEN** the powered-descent stack is assembled
- **THEN** `PropulsionStub` is absent from it

### Requirement: Carried mass and propellant are seeded on the field, never published as constants

The burn world SHALL seed the `"mass"` and `"propellant"` scalars exactly once onto the initial
`CoupledField` through `set_scalar`, and MUST NOT supply them through the world's
`publish_constant` seam. The carrier re-publishes every world constant with `set_scalar` at the head
of every step, before the coupling runs, so a published `"mass"` would be silently restored to its
seed value after each step's depletion: the burn would never consume propellant, the propellant
floor could never trip, and the mass-aware thrust normalization would be frozen at its initial
value. Before ignition the seeded mass MUST equal the mass implied by the corridor's `CDA_OVER_M`
bundle, so Act-1 normalization is unchanged.

#### Scenario: Propellant falls monotonically across the burn

- **WHEN** the burn runs for N ignited steps
- **THEN** the `"propellant"` scalar at the end is lower than at ignition by the accumulated
  `ṁ·Δt`, and the `"mass"` scalar has fallen by the same amount

#### Scenario: The propellant floor can trip

- **WHEN** a burn is commanded long enough to drive `"propellant"` to the envelope's floor
- **THEN** the cybernetic gate logs the propellant-floor breach and returns an error, rather than
  burning indefinitely against a restored constant

### Requirement: The throttle command is driven on both seams

The example's throttle guidance SHALL write the commanded throttle to **both** the typed throttle
channel (`set_throttle_action`) and the world-published `"commanded_throttle"` scalar, because the
three consumers read different seams: the propulsion stages prefer the channel and fall back to the
scalar, the cybernetic gate's burn path reads **only** the channel, and the carrier's plume
re-imprint reads **only** the scalar. Driving one seam alone is a silent partial wiring — channel
only yields thrust and envelope clamping but no plume re-imprint, scalar only yields thrust and
re-imprint but no envelope clamping at all.

#### Scenario: All three consumers see the commanded throttle

- **WHEN** the guidance commands a throttle during the burn
- **THEN** the propulsion stages form thrust from it, the cybernetic gate clamps it into the burn
  envelope, and the carrier's plume imprint refreshes on a tolerance-sized move

#### Scenario: The imprint lags ignition by one step

- **WHEN** the first ignited step runs
- **THEN** the plume re-imprint does not fire on that step, because it reads the
  `"plume_max_radius"`/`"plume_penetration"` scalars that `PlumeObstruction` publishes later in the
  same step, and it fires on the following step instead

### Requirement: The opt-in library axes are enabled and the scalars they sense are produced

The powered-descent stack SHALL call `RegimeClassify::with_flight_axes` and
`CyberneticCorrect::with_burn_sensing`, and MUST publish the `"q_inf"` and `"descent_rate"` scalars
from an example-local stage. All four are inert by default in ways that produce a plausible-looking
run rather than an error: without `with_flight_axes` the Mach, thrust-state, and touchdown axes stay
`Unknown` even when `"flight_mach"`, `"ignited"`, and `"flight_altitude"` all ride the field, so the
regime cascade never logs; without `with_burn_sensing` the gate's `thrust_ref` stays 0 and the
dynamic C_T cap never binds; and because `peak` over an absent scalar is 0, a missing
`"descent_rate"` means the descent-rate breach can never fire while a missing `"q_inf"` silently
degrades the dynamic C_T cap to the static ceiling. Nothing in `deep_causality_cfd` produces either
scalar.

#### Scenario: The regime cascade logs under thrust

- **WHEN** the burn crosses a Mach band boundary with the flight axes enabled
- **THEN** a regime transition naming the new Mach regime and the thrust state appears in provenance

#### Scenario: The dynamic thrust-coefficient cap binds

- **WHEN** a commanded throttle would drive `C_T` past the envelope's `max_ct` at the sensed `q∞`
- **THEN** the gate clamps the throttle below the static ceiling and logs the bounding

### Requirement: The plume reference area is named distinctly from the acoustic reference speed

The example SHALL introduce a distinctly named reference-**area** constant for the plume stage's
`s_ref` argument and MUST NOT pass the shared `S_REF` constant to it. The shared `S_REF` is the
reference wave speed of the implicit acoustic envelope, not an area; it is the same scalar type and
a plausible-looking name, so passing it compiles, runs, and silently computes a wrong thrust
coefficient — which then propagates into the preserved-drag fraction, the dynamic C_T cap, and every
counterfactual witness downstream of them.

#### Scenario: The thrust coefficient is formed from an area

- **WHEN** `PlumeObstruction` is constructed for the burn world
- **THEN** its `s_ref` argument is the propulsion reference area in m², documented as such, and is
  not the shared acoustic `S_REF`
