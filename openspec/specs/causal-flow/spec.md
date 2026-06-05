# causal-flow Specification

## Purpose
TBD - created by archiving change causal-flow-dsl. Update Purpose after archive.
## Requirements
### Requirement: Uniform construction of effect and process flows without naming witnesses

`CausalFlow` SHALL provide constructors that build a stateless flow (lowering to `PropagatingEffect`)
and a stateful flow (lowering to `PropagatingProcess`) through one uniform type, without the caller
naming any HKT witness type (`PropagatingEffectWitness`, `PropagatingProcessWitness`, …) or calling
`pure` / `with_state` directly. `Error` and `Log` SHALL be fixed to `CausalityError` and `EffectLog`
so they never appear in a user-facing signature.

#### Scenario: Stateless flow seeds an effect

- **WHEN** the caller writes `CausalFlow::effect()` (or `CausalFlow::value(v)`)
- **THEN** it yields a stateless `CausalFlow` whose lowered form equals `PropagatingEffect::pure(())`
  (respectively a value-carrying effect), with no witness type named at the call site

#### Scenario: Stateful flow seeds a process

- **WHEN** the caller writes `CausalFlow::process(state).context(cfg)`
- **THEN** the lowered form equals `with_state(PropagatingEffect::pure(()), state, Some(cfg))`, and the
  caller never writes `with_state` or `pure`

#### Scenario: Error constructor seeds the error channel

- **WHEN** the caller writes `CausalFlow::fail(err)`
- **THEN** the lowered form carries `Some(err)` in the error channel and an empty value

### Requirement: Fluent steps unwrap the value and short-circuit on error

`CausalFlow` steps SHALL hand the caller's closure the unwrapped `Value` (never `EffectValue`), and
SHALL propagate an existing error or absent value without invoking the closure. The step methods SHALL
cover: `and_then` (closure returns a `CausalFlow`), `try_step` (closure returns `Result<U, CausalityError>`),
`map` (infallible value transform), `guard` (validation that short-circuits on `Err`), and for the
stateful carrier `try_step_with` (read-only state and context) and `step_mut` (mutable state alongside the
value transform, for the common case where the state update derives from the value). The facade SHALL
also provide a channel-update family: the confined `update_value` (value only), `update_state` (state
evolved from the value), and `update_context` (context evolved from the value), each writing a single
channel and reading the value as the driver, plus `update_value_state_context`, which rewrites the value,
state, and context together in one closure.

#### Scenario: Step receives the raw value on success

- **WHEN** `step` (or `try_step`) is applied to a flow carrying a value
- **THEN** the closure is invoked with the raw `Value`, and its result becomes the new flow

#### Scenario: Step is skipped on an errored flow

- **WHEN** `step`, `try_step`, `map`, or `guard` is applied to a flow already in the error channel
- **THEN** the closure is not invoked and the existing error and accumulated logs are preserved

#### Scenario: try_step lifts a Result into the flow

- **WHEN** a `try_step` closure returns `Ok(u)`
- **THEN** the flow carries value `u`; **WHEN** it returns `Err(e)`, **THEN** the flow short-circuits
  with `e` in the error channel

#### Scenario: Guard validates without changing the value

- **WHEN** `guard` returns `Ok(())`, **THEN** the value passes through unchanged; **WHEN** it returns
  `Err(e)`, **THEN** the flow short-circuits with `e`

#### Scenario: Stateful step reads state and context and keeps threading them

- **WHEN** `try_step_with` is applied to a stateful flow
- **THEN** the closure receives the unwrapped value, a reference to the current `State`, and an
  `Option<&Context>`, returns `Result<U, CausalityError>`, and the resulting flow carries the same
  `State`/`Context`

#### Scenario: A step mutates state while transforming the value

- **WHEN** `step_mut` is applied to a stateful flow
- **THEN** the closure receives the unwrapped value and a mutable reference to the `State` (plus the
  context), may update the state from the computed value, and returns the new value; the resulting flow
  carries the mutated `State` and the new value

#### Scenario: Channel-update operators evolve one channel or all three

- **WHEN** `update_value`, `update_state`, or `update_context` is applied to a flow carrying a value
- **THEN** the named channel is replaced (the value in place, or the `State`/`Context` evolved from the
  current value) while the other two channels pass through unchanged
- **WHEN** `update_value_state_context` is applied to a flow carrying a value
- **THEN** the closure owns the value, `State`, and `Option<Context>` and returns the next triple, which
  becomes the new flow

### Requirement: Terminals extract the result and interoperate with the underlying monad

`CausalFlow` SHALL provide terminals that yield the final `Value` or `CausalityError` without the
caller matching `EffectValue`, and SHALL convert losslessly to and from
`CausalEffectPropagationProcess` so the facade interoperates with existing monad code.

#### Scenario: finish yields a Result

- **WHEN** `finish` is called on a successful flow, **THEN** it returns `Ok(value)`; **WHEN** called on
  a failed flow, **THEN** it returns `Err(error)`

#### Scenario: run dispatches to the success or error handler

- **WHEN** `run(on_ok, on_err)` is called, **THEN** exactly one of `on_ok(value)` or `on_err(error)` is
  invoked according to the flow's outcome

#### Scenario: Conversions round-trip losslessly

- **WHEN** a `CausalEffectPropagationProcess` is converted into a `CausalFlow` and back via
  `into_effect` / `into_process` (or `From`/`Into`)
- **THEN** the value, error, state, context, and logs are unchanged

### Requirement: Drop-in interoperability with existing stages and explicit error recovery

`CausalFlow` SHALL provide passthrough combinators `bind` and `bind_or_error` that accept stage functions
written against the existing monad signatures unchanged, and SHALL provide a `recover` combinator that
converts the error channel back into a value. `From<PropagatingEffect<…>>` / `From<PropagatingProcess<…>>`
SHALL let an effect- or process-returning stage adapt into a flow with `.into()`.

#### Scenario: Existing bind_or_error stages drop in unchanged

- **WHEN** a chain wraps an effect with `CausalFlow::from(effect)` and applies `bind_or_error(stage, msg)`,
  where `stage` is written against the existing `(Value, State, Option<Context>) -> …` signature
- **THEN** the stage runs unchanged and the outcome matches the equivalent `effect.bind_or_error(stage, msg)`

#### Scenario: An effect-returning stage adapts via From

- **WHEN** a stage returns a `PropagatingEffect<U>` and is used inside `and_then` as `stage(v).into()`
- **THEN** the flow continues with that effect's value, error, and logs

#### Scenario: recover turns an error into a value

- **WHEN** `recover(f)` is applied to a flow in the error channel
- **THEN** `f` is invoked with the error and its return value becomes the flow's value, clearing the error;
  **WHEN** applied to a successful flow, **THEN** the value is unchanged and `f` is not invoked

### Requirement: Closed-loop intervention is a first-class flow step

`CausalFlow` SHALL expose the `Intervenable` (Pearl Layer 2) closed-loop override as a step: `intervene`
SHALL substitute a new value into the flow, and `intervene_if` SHALL do so only when a predicate over the
current value holds. Both SHALL lower to the underlying `Intervenable::intervene` and preserve the
intervention's audit entry in the log.

#### Scenario: intervene substitutes the value and records the override

- **WHEN** `intervene(new_value)` is applied to a successful flow
- **THEN** the flow carries `new_value`, and the log records the value substitution (the intervention
  marker), matching the underlying `Intervenable::intervene`

#### Scenario: intervene_if fires only on the trigger condition

- **WHEN** `intervene_if(cond, f)` is applied and `cond(&value)` is true
- **THEN** the value is replaced by `f(value)` and the override is logged; **WHEN** `cond` is false,
  **THEN** the value passes through unchanged and no override is recorded

### Requirement: The facade is behavior-preserving sugar over the monad

A `CausalFlow` pipeline SHALL produce the same final value, error, state, context, and logs as the
equivalent hand-written `CausalEffectPropagationProcess` pipeline. The facade SHALL add no behavior of
its own beyond lowering to existing constructors and combinators.

#### Scenario: Flow chain equals the equivalent bind chain

- **WHEN** a multi-step `CausalFlow` chain is run and the same steps are written directly with
  `pure` / `bind` / `bind_or_error`
- **THEN** the two produce equal value, error, and accumulated logs

#### Scenario: Error short-circuit and log accumulation match the monad

- **WHEN** a step in the middle of a flow fails
- **THEN** subsequent steps are skipped and the logs accumulated up to the failure match those of the
  equivalent `bind` chain

