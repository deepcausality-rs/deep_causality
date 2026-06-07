# causal-arrow Specification

## Purpose
TBD - created by archiving change causal-flow-arrow. Update Purpose after archive.
## Requirements
### Requirement: A reusable Kleisli arrow over the causal monad

The system SHALL provide `CausalArrow`, a reusable value representing a causal stage
`A → CausalFlow<B, State, Context>`. It SHALL implement `deep_causality_haft::Arrow` with `In = A` and
`Out = CausalFlow<B, State, Context>`, and its `run` SHALL take `&self` so one arrow applies to many
inputs. Applying a `CausalArrow` SHALL produce the same value, error, state, context, and logs as the
equivalent `CausalEffectPropagationProcess` pipeline; the arrow adds no behavior beyond lowering to the
monad. The arrow SHALL be single-input; the monoidal product (multi-input) is out of scope.

#### Scenario: An arrow is built once and run many times

- **WHEN** a `CausalArrow` is constructed by lifting a stage function and is then applied to two
  different inputs
- **THEN** each application returns that input's result, and the arrow value is not consumed by either
  application (it is taken by shared reference)

#### Scenario: Application equals the equivalent monad pipeline

- **WHEN** a `CausalArrow` is applied to an input
- **THEN** the resulting flow's value, error, state, context, and logs equal those of the same stage
  written directly with `pure` / `bind`

### Requirement: Sequential composition threads the monad

`CausalArrow` SHALL provide Kleisli sequential composition (`next`): composing `A → CausalFlow<B>` with
`B → CausalFlow<C>` SHALL yield `A → CausalFlow<C>` by binding the first arrow's result into the second.
Composition SHALL thread the error, state, and context channels and accumulate logs, and SHALL
short-circuit the second arrow when the first produces an error. A fluent builder SHALL hide the nested
combinator types so the caller writes a left-to-right chain without naming them.

#### Scenario: Composed arrows bind, not apply

- **WHEN** two causal arrows are composed and the composite is run on an input
- **THEN** the second arrow receives the unwrapped value of the first arrow's result, and the composite
  result carries the logs of both stages in order

#### Scenario: A failing first stage short-circuits the second

- **WHEN** the first arrow of a composition produces an error
- **THEN** the second arrow is not run, and the composite carries the first arrow's error and its logs

### Requirement: Reusable pipeline composites compose as engine values

A `CausalArrow` value SHALL compose with another `CausalArrow` through `next`, yielding one composite
`CausalArrow`. The composite is a reusable `A → CausalFlow<D>` value that can be stored, passed, and run on
many inputs, and it SHALL produce the same result as applying the pipelines in sequence by hand. This engine
form serves the held-as-data case only; the routine surface for composing pipelines is the flow DSL's
`next` (the `causal-flow` capability), which does not name the arrow type.

#### Scenario: Three pipeline values wire into one composite

- **WHEN** three pipeline values `p1: A → CausalFlow<B>`, `p2: B → CausalFlow<C>`, and
  `p3: C → CausalFlow<D>` are composed as `p1.next(p2).next(p3)` and the composite is run on an input of
  type `A`
- **THEN** the result equals running `p1`, then binding into `p2`, then binding into `p3`, with the value,
  error, state, context, and logs of the three pipelines threaded in order

