## ADDED Requirements

### Requirement: The envelope's sensed flight scalars have library producers

`deep_causality_cfd` SHALL publish the `"q_inf"` and `"descent_rate"` scalars from a library
`PhysicsStage`, because the powered-descent gate reads both and **no library, example, or study code
produces either** — the only writers in the workspace are the gate's own unit tests, which is why
both axes demonstrably work under test while having no path to work in flight. The gate reads each
through a peak reduction folded from zero, so an absent producer reads as **0**, which is inside the
envelope for the descent-rate bound and collapses the dynamic thrust-coefficient ceiling to the
static one. The failure is therefore **fail-open on first wiring**: the moment a world attaches burn
axes without publishing these scalars, two axes report as enforcing while being unable to fire, and
nothing in the run says so. Both field names MUST be configurable, matching the gate's own sensing
configuration.

#### Scenario: A world attaching burn axes gets working axes

- **WHEN** a world attaches burn axes and composes this stage
- **THEN** the descent-rate bound and the dynamic thrust-coefficient cap sense published values
  rather than the zero an absent producer would yield

#### Scenario: The axes work in flight, not only under test

- **WHEN** the descent-rate breach and the dynamic cap are exercised in a marching world
- **THEN** they behave as the gate's unit tests already demonstrate they can, because the scalars
  they sense now have a production producer

### Requirement: Dynamic pressure is formed from the freestream with a configured molecular mass

The stage SHALL form `q∞ = ½·ρ∞·V²` with `ρ∞ = n∞·m̄` from the carrier-published freestream number
density and flight speed, taking the mean molecular mass `m̄` as a **constructor parameter**. The
carrier holds the number density in m⁻³ and the speed in m/s but carries no molecular mass, and the
physics crate carries no air mean molecular mass either; rather than push a species constant into
the carrier or the physics crate, the stage takes it by construction, following the existing
freestream stage that already performs the same `n·m̄` conversion.

#### Scenario: The species constant is supplied, not assumed

- **WHEN** the stage is constructed for a world
- **THEN** its mean molecular mass is supplied by that world, and no default air composition is
  assumed inside the stage

### Requirement: Descent rate is positive downward

The stage SHALL publish `"descent_rate"` as `ḣ = −(r·v)/|r|` from the truth state, so that a
descending vehicle yields a **positive** value. The sign convention is load-bearing rather than
cosmetic: the gate tests `descent_rate > max_descent_rate` after a maximum reduction, so an
ascent-positive convention would make the bound unreachable in exactly the regime it exists to
protect. The stage MUST be inert when the truth state is absent or shorter than the six cells the
position and velocity vectors occupy, publishing nothing rather than a value derived from a partial
state.

#### Scenario: A descending vehicle reports a positive rate

- **WHEN** the truth state carries a velocity with a component toward the planet
- **THEN** the published descent rate is positive and grows with that component

#### Scenario: An absent truth state publishes nothing

- **WHEN** the truth state is missing or has fewer than six cells
- **THEN** no descent rate is published, rather than a value formed from a partial state
