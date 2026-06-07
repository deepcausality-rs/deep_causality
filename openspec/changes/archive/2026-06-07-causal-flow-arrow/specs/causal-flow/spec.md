## ADDED Requirements

### Requirement: The flow DSL expresses bounded loops

`CausalFlow` SHALL provide bounded iteration that repeats a flow endomorphism
`step: CausalFlow<Value, State, Context> → CausalFlow<Value, State, Context>`, the causal analogue of
`EndoArrow`. `iterate_n(n, step)` SHALL apply the step exactly `n` times. `iterate_until(pred, max_steps,
step)` SHALL apply the step until `pred(&value)` holds or `max_steps` is reached, checking the predicate
before each step. `iterate_to_fixpoint(max_steps, step)` SHALL apply the step until the value stops changing or
`max_steps` is reached. Each SHALL thread value, state, context, and logs through every iteration and
short-circuit on an error. When `iterate_until` or `iterate_to_fixpoint` reaches the step bound without meeting its
condition, the flow SHALL short-circuit with a `CausalityError` reporting non-convergence.

#### Scenario: iterate_n applies the step a fixed number of times

- **WHEN** `iterate_n(n, step)` is applied to a flow carrying a value and the step never errors
- **THEN** the flow carries the value after `n` applications, with the logs of all applications
  accumulated

#### Scenario: iterate_until stops on the predicate or fails at the bound

- **WHEN** `iterate_until(pred, max_steps, step)` runs and `pred` becomes true within the bound
- **THEN** the flow carries the value at which `pred` first held; **WHEN** the bound is reached first,
  **THEN** the flow short-circuits with a non-convergence `CausalityError`

#### Scenario: iterate_to_fixpoint stops when the value stops changing

- **WHEN** `iterate_to_fixpoint(max_steps, step)` runs and a step returns a value equal to its input within the bound
- **THEN** the flow carries that fixpoint value; **WHEN** the bound is reached first, **THEN** the flow
  short-circuits with a non-convergence `CausalityError`

#### Scenario: A failing step short-circuits the loop

- **WHEN** any iteration's step produces an error
- **THEN** the loop stops, the remaining iterations are skipped, and the error and accumulated logs are
  preserved

### Requirement: The flow DSL expresses branches

`CausalFlow` SHALL provide conditional routing whose continuations are flow endomorphisms
(`CausalFlow<Value, State, Context> → CausalFlow<Value, State, Context>`), so a branch keeps threading
state, context, and logs. `branch(cond, on_true, on_false)` SHALL select a continuation by a predicate
over the current value. `branch_with(cond, on_true, on_false)` SHALL select by a predicate that also reads
the state and context. `either(left, right)` SHALL route a flow whose value is `Either<L, R>` to a left or
right continuation. An errored flow SHALL short-circuit without running any continuation.

#### Scenario: branch selects a continuation by predicate

- **WHEN** `branch(cond, on_true, on_false)` is applied to a flow carrying a value and `cond(&value)` is
  true
- **THEN** `on_true` runs on the flow and `on_false` does not; **WHEN** `cond(&value)` is false, **THEN**
  `on_false` runs and `on_true` does not

#### Scenario: branch_with reads state and context in the predicate

- **WHEN** `branch_with(cond, on_true, on_false)` is applied and `cond` inspects the value, state, and
  context
- **THEN** the selected continuation runs on the flow, threading the same state and context

#### Scenario: either routes an Either value to its arm

- **WHEN** `either(left, right)` is applied to a flow carrying `Either::Left(l)`
- **THEN** the left continuation runs; **WHEN** the value is `Either::Right(r)`, **THEN** the right
  continuation runs

#### Scenario: Branching is skipped on an errored flow

- **WHEN** `branch`, `branch_with`, or `either` is applied to a flow already in the error channel
- **THEN** no continuation runs and the existing error and accumulated logs are preserved

### Requirement: Whole pipelines compose through the flow DSL

`CausalFlow` SHALL compose whole pipelines through the high-level DSL, without the caller naming the
`CausalArrow` engine type. A pipeline is a function `Value → CausalFlow<U, State, Context>`; `next` SHALL
apply such a pipeline to the flow, threading value, error, state, context, and logs, and SHALL short-circuit
an errored flow without running the pipeline. Composing pipelines with `next` SHALL be the surface used by
examples and documentation; the reified `CausalArrow` value SHALL remain available for held-as-data
composites but SHALL NOT be required to compose pipelines.

#### Scenario: Three pipelines wire together through the DSL

- **WHEN** three pipelines `p1: A → CausalFlow<B>`, `p2: B → CausalFlow<C>`, and `p3: C → CausalFlow<D>` are
  composed as `CausalFlow::value(a).next(p1).next(p2).next(p3)`
- **THEN** the result equals applying the pipelines in sequence, with value, error, state, context, and logs
  threaded in order, and no `CausalArrow` type named at the call site

#### Scenario: next is skipped on an errored flow

- **WHEN** `next(pipeline)` is applied to a flow already in the error channel
- **THEN** the pipeline is not run and the existing error and accumulated logs are preserved

### Requirement: A reified pipeline value applies through `and_then`

A reified `CausalArrow` pipeline value SHALL be applied to a flow through the existing `and_then`
combinator, as `and_then(|v| arrow.run(v))`; no dedicated method is added (for consistency with the rest of
the DSL). The result SHALL thread value, error, state, context, and logs, and SHALL short-circuit an errored
flow without running the arrow. This is the bridge for the case a composite was built and held as a
`CausalArrow` value (the engine); routine composition uses `next`.

#### Scenario: and_then applies a reified arrow

- **WHEN** `and_then(|v| arrow.run(v))` is applied to a flow carrying a value
- **THEN** the flow continues with `arrow.run(value)`'s value, error, state, context, and logs; **WHEN**
  applied to an errored flow, **THEN** the arrow is not run and the error and logs are preserved
