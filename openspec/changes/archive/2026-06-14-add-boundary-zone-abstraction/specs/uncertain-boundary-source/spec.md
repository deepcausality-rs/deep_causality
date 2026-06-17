## ADDED Requirements

### Requirement: Cross-domain uncertain boundary source
The solver SHALL provide an `UncertainBoundarySource<R>` that supplies the time-varying value of
any Dirichlet-bearing boundary zone (inflow, moving wall) from a `MaybeUncertain<R>` stream: each
step it presence-gates the sample (`lift_to_uncertain`), collapses a present reading to a scalar
(`expected_value` / `sample`), and updates the last-good value in the monad state. On a dropout
(presence error or a non-finite collapse) it SHALL substitute the last-good (or configured
default) value via `.intervene` and record the dropout in the `EffectLog` at a configurable
verbosity (default: each dropout; lower: onset/recovery transitions). The source SHALL depend only
on `MaybeUncertain<R>` and the monad — not on fluid-dynamics concepts — so it is reusable as a
sensor-fed time-varying parameter in any domain.

#### Scenario: A dropout falls back through a logged intervention
- **WHEN** the stream feeding a boundary zone drops below the presence threshold at a step
- **THEN** the last-good (or default) value is substituted via `.intervene`, the dropout and fallback are recorded in the `EffectLog`, and the march continues without error

#### Scenario: No-dropout stream matches the deterministic control
- **WHEN** a stream with no dropouts feeds a boundary zone and is compared against a deterministic-value run of the same case
- **THEN** the marched fields agree to rounding

#### Scenario: The same source serves a non-inflow zone
- **WHEN** an `UncertainBoundarySource` is attached to a moving-wall zone instead of an inflow zone
- **THEN** the wall's prescribed velocity is sourced from the uncertain stream with the same presence-gate, collapse, and dropout-intervention behaviour
