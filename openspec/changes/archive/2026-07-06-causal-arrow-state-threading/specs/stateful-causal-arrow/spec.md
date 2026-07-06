# stateful-causal-arrow Specification

## ADDED Requirements

### Requirement: There is exactly one Kleisli bind, and it threads state
The flow's Kleisli bind SHALL be a single operation `and_then` whose continuation receives
`(value, state, context)` and returns the next flow, whose state/context are threaded **forward**
(`s0 → s1 → s2`) exactly as the causal monad's `bind` threads them. The stateless case SHALL be a
specialization of this one bind — with `State = Context = ()` there is nothing to thread — not a
separate operation. There SHALL NOT be a parallel value-only bind with divergent state semantics
(the former state-discarding `and_then` is removed).

#### Scenario: State threads across a composed chain
- **WHEN** two stages that read and update the accumulated state are sequenced with `and_then` from
  initial state `s0`
- **THEN** the first stage sees `s0`, the second sees the first's output state `s1`, and the result
  carries `s2` (state threaded, not discarded)

#### Scenario: Error short-circuits and preserves state
- **WHEN** the first stage returns an errored flow with state `s1`
- **THEN** the second stage is not invoked and the result carries the error with state `s1` and the
  accumulated logs (left zero)

### Requirement: `next` is the value-only sugar over the one bind
The DSL SHALL provide `next(pipeline)` where `pipeline: Fn(Value) -> CausalFlow<U>`, defined as exactly
`and_then(|v, _, _| pipeline(v))` — the everyday value-only step for stateless pipelines and the
migration form of the former value-only `and_then`. It SHALL NOT be a second bind law (it only
pre-ignores the state/context inputs), so for stateless flows `next` and `and_then` coincide, and
existing `.next(stage)` call sites (closures and named `Fn(Value)` stage functions) SHALL keep
compiling unchanged.

#### Scenario: next is a drop-in value-only step
- **WHEN** a stateless pipeline is written with `.next(|v| ...)` or `.next(named_stage_fn)`
- **THEN** it behaves identically to `.and_then(|v, _, _| ...)`, and pre-existing `.next(...)` sites
  compile without change

### Requirement: The reified arrow stage receives and threads state
The reified causal-arrow stage SHALL have type `(A, S, Option<C>) -> CausalFlow<B, S, C>` — one stage
type, receiving the incoming `(value, state, context)`; the stateless case is the specialization
`|a, _, _| ...`. There SHALL NOT be separate value-only and stateful stage types. `KleisliCompose`
SHALL thread `P`'s evolved `(value, state, context)` into `Q` via `and_then`, and the `CausalArrow`
marker and builder (`causal_arrow`, `.next`) SHALL use this stage type. A `run_value(a)` convenience
SHALL apply an `S = C = ()` arrow to a bare value (`run((a, (), None))`).

#### Scenario: A reusable arrow threads accumulated state
- **WHEN** a two-stage arrow whose stages add the incoming value to the state is run from state `0`
- **THEN** the state threads across both stages and the final state reflects both updates

#### Scenario: Stateless arrow ergonomics
- **WHEN** a stateless arrow is built with `causal_arrow(|x, _, _| ...)` and applied via `run_value(x)`
- **THEN** it produces the same result as the equivalent monad pipeline

### Requirement: The arrow laws hold over arbitrary state and are machine-checked
Over arbitrary `S` and `C`, the arrow SHALL satisfy left identity `η >=> f = f`, right identity
`f >=> η = f`, associativity `(f >=> g) >=> h = f >=> (g >=> h)`, and error left-zero, threading state
on both sides of each equation with no `S,C`-erasure caveat. These SHALL be proved in
`lean/DeepCausalityFormal/Core/CausalArrow.lean` (stage `A → S → Option C → Process B S C E Λ`,
reducing to the monad theorems) and witnessed in `deep_causality_core` under ids
`core.causal_arrow.category_laws` and `core.causal_arrow.left_zero`.

#### Scenario: Lean file typechecks and reduces to the monad laws
- **WHEN** `lean lean/DeepCausalityFormal/Core/CausalArrow.lean` is run
- **THEN** it typechecks with zero `sorry`, and the arrow theorems close by reduction to
  `bind_left_id`/`bind_right_id`/`bind_assoc`/`bind_raise_left_zero`

#### Scenario: Witnesses pin the Rust arrow
- **WHEN** the arrow witness tests run
- **THEN** a reusable stateful pipeline threads accumulated state; identity, associativity, and
  error-short-circuit-preserves-state each pass, carrying the `core.causal_arrow.*` id
