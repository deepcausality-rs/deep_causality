## Context

Two algebras sit in the codebase on one foundation and never meet.

The **causal monad** is `CausalEffectPropagationProcess<Value, State, Context, Error, Log>` in
`deep_causality_core`, with `pure` and a state-threading `bind` that short-circuits on error and
accumulates logs. `CausalFlow<Value, State, Context>` wraps it as a fluent facade. Today the facade
composes sequentially only: `and_then`, `try_step`, `map`, and the channel updaters each lower to one
`bind`. The chain is linear and spent as it is built.

The **value-level Arrow algebra** is `deep_causality_haft::Arrow`: a strong category with concrete,
defunctionalized carriers (`Id`, `Lift`, `Compose`, the strength combinators, and an `EndoArrow`
iteration extension) plus an `ArrowBuilder` that hides the nested combinator types. `core` depends on
`haft` but uses none of it.

The two cannot be glued directly, and the reason drives the design:

- `haft::Compose<F, G>` requires `G: Arrow<In = F::Out>` and runs `g.run(f.run(x))`. A causal stage is
  `A → CausalFlow<B>`, so its `Out` is `CausalFlow<B>` while the next stage's `In` is `B`. The two do not
  line up; sequential composition of causal stages must **`bind`**, not apply.
- `haft::EndoArrow` iterates `Arrow<In = S, Out = S>` with `state = self.run(state)`. The causal carrier
  to iterate is the flow itself; an iteration step is a **flow endomorphism** `CausalFlow → CausalFlow`,
  and each step must `bind`, threading the monad's channels.

This is the textbook relationship between a category and its **Kleisli category**: same arrow shapes,
different composition law (`bind` instead of `∘`). The Causal Arrow is the Kleisli arrow of the causal
monad. It carries its own combinator structs, because the pure `haft` ones compose by application; it can
still *implement* the `haft::Arrow` trait, so the interface is shared even though the carriers are not.

The gap shows up concretely in every time-stepped example. They hand-roll the loop:

```rust
let mut process = model::initial_process();
for _ in 0..N_TICKS {
    process = process.bind(model::simulate_step);   // advance one tick
    // ... conditional intervention, read out of process.value, rebind ...
}
```

The loop sits outside the monad. Iteration is unbounded by the type, error short-circuit and log
accumulation are manual, and the conditional correction is a plain `if` around a rebind. The Causal Arrow
turns the `for` into `iterate_n`, the `if` into `branch`, and a stage into a reusable, composable value.

## Goals / Non-Goals

**Goals:**

- A reusable Causal Arrow: a `A → CausalFlow<B, S, C>` value implementing `haft::Arrow`, with Kleisli
  sequential composition that threads value, error, state, context, and logs.
- Composition of whole pipelines: author process one, two, three each as a `CausalFlow` pipeline, then
  wire them with the same fluent `next`.
- Bounded loops on the flow DSL: `iterate_n`, `iterate_until`, `iterate_to_fixpoint`, each iterating a flow
  endomorphism with an explicit step bound and an honest convergence outcome.
- Branches on the flow DSL: `branch`, `branch_with`, `either`, over a small `Either`.
- A showcase: the `corrective_*` intervention family and the `geometric_tcas` / `hypersonic_2t` /
  `magnav` avionics examples rewritten to the new surface, output preserved.
- Additive and non-breaking; no change to the monad types, the witness layer, or the existing
  `CausalFlow` surface.

**Non-Goals (the explicit scope boundary):**

- Not causal discovery (SURD, BRCD, PC/GES/BOSS); no discovery operator is cast onto the arrow.
- Not multi-input / the monoidal product (`split` / `***`, `fanout` / `&&&`, `first`, `second`). Every
  arrow here is single-input. Wiring two structured inputs into one stage is out of scope.
- Not the static structural parameter (a frozen graph or lattice). It exists to serve discovery, which is
  out, so it is out.
- Not the governance or action fragments; not async.

## Decisions

### D1. The Causal Arrow is the Kleisli arrow of the causal monad

A causal arrow is a reusable `A → CausalFlow<B, S, C>`. It implements `haft::Arrow` with `In = A`,
`Out = CausalFlow<B, S, C>`, so it is a first-class `Arrow` value. Its sequential composition is Kleisli:
it binds the first arrow's result into the second. One line keeps the abstraction honest: **pure `haft`
Arrow = the category; Causal Arrow = its Kleisli category over `CausalEffectPropagationProcess`.**

Reusability is the win the facade lacks. A `CausalFlow` holds a value mid-flight and is spent when
chained. A `CausalArrow` is built once and run on many inputs:

```rust
let stage = causal_arrow(parse).next(validate).next(score).build();
let a = stage.run(input_a);   // &self: reusable
let b = stage.run(input_b);
```

### D2. Location: a `causal_arrow` module in `deep_causality_core`

The arrow lowers to the monad on every combinator, so it lives next to it, in
`deep_causality_core::causal_arrow`, beside `causal_flow`. No new crate; the existing lints, edition, and
coverage policy apply.

### D3. A minimal, defunctionalized carrier set (no product)

No `dyn`, so each combinator is a concrete generic struct, as in `haft`. Because multi-input is out of
scope, the set is small:

| carrier | shape | role |
| --- | --- | --- |
| `CausalLift<A, B, S, C, F>` | `F: Fn(A) -> CausalFlow<B, S, C>` | lift a stage function into an arrow |
| `KleisliCompose<P, Q>` | `P: A → CF<B>`, `Q: B → CF<C>` | `next`: run `P`, `bind` into `Q` |
| `CausalArrowBuilder<S>` | wraps the growing arrow | fluent `.next` (add a sub-process), `.build` / `.run` |

No `Split`, `Fanout`, `First`, or `Second`; those are the product, which is out of scope. The builder
mirrors `haft::ArrowBuilder` for the sequential fragment only, so the user writes a left-to-right chain
and never names `KleisliCompose<…>`.

### D4. Composing whole pipelines: the CausalFlow DSL is the surface, the arrow is the engine

A *pipeline* is a process authored as a function `A → CausalFlow<B>` with the CausalFlow DSL. Pipelines
compose through the DSL, not by naming the arrow type: `next` applies the next pipeline to the flow, so
process one, two, and three wire together exactly as stages do.

```rust
// each process is authored with the DSL and returns a flow
fn sense(raw: Raw)    -> CausalFlow<Reading> { CausalFlow::value(raw).try_step(parse).try_step(calibrate) }
fn decide(r: Reading) -> CausalFlow<Plan>    { CausalFlow::value(r).try_step(estimate).map(plan) }
fn act(p: Plan)       -> CausalFlow<Command> { CausalFlow::value(p).try_step(authorize).map(emit) }

// wire the three pipelines with the high-level DSL
let command = CausalFlow::value(raw)
    .next(sense)
    .next(decide)
    .next(act)
    .finish()?;
```

`next` is the pipeline-composition verb on `CausalFlow`: it applies a whole pipeline `Fn(Value) ->
CausalFlow<U>` and lowers to `bind`, the same lowering as `and_then` (a pipeline is a full monadic step).
Reuse is a named function; `sense` runs on many inputs by being called again.

Underneath, `CausalArrow` is the **engine** (D1, D3). It stays out of the examples and the headline API; it
exists for the one case the DSL surface cannot express, a *named, storable, reusable composite* built
without applying it to an input yet:

```rust
// engine / advanced layer — not used in the showcase:
let controller: CausalArrow<Raw, Command> = causal_arrow(sense).next(decide).next(act).build();
let a = controller.run(raw_a);   // the composite is a value: store it, pass it, reuse it
let b = controller.run(raw_b);
```

`flow.and_then(|v| controller.run(v))` applies such a reified composite back into a flow (the existing
`and_then`, no dedicated method). The rule: author and compose pipelines through the CausalFlow DSL; reach
for the `CausalArrow` value only when a composite must be held as data (stored in a field, returned, or
passed across an API boundary).

### D5. Branches: `branch` / `branch_with`, and `either` (over the `Either` sum)

A branch routes to one of two continuations. The continuations are **flow endomorphisms**
(`CausalFlow<V, S, C> → CausalFlow<V, S, C>`), so they keep threading state, context, and logs; this is
what the stateful `corrective_*` loops need. The predicate peeks the current value before routing:

```rust
.branch(cond:      Fn(&Value) -> bool,
        on_true:   FnOnce(CausalFlow<V, S, C>) -> CausalFlow<V, S, C>,
        on_false:  FnOnce(CausalFlow<V, S, C>) -> CausalFlow<V, S, C>) -> CausalFlow<V, S, C>

.branch_with(cond: Fn(&Value, &State, Option<&Context>) -> bool, on_true, on_false) -> CausalFlow<V, S, C>
```

`branch` sees the value; `branch_with` also reads state and context, mirroring `try_step` vs
`try_step_with`. On an errored flow both are a no-op that preserves the error and logs. The common arm is
`|flow| flow` (pass through), so the "act only when the monitor trips" shape reads as
`branch(tripped, |f| f.intervene(correction), |f| f)`.

`either` is the typed multi-way form over a sum: for `CausalFlow<Either<L, R>, S, C>`, route `Left` and
`Right` to arms producing `CausalFlow<U, S, C>`. A two-variant `Either<L, R>` is added to `haft` (D7);
`Result<L, R>` is rejected as the carrier because it already means success-or-error, and routing on it
would conflate a branch with a failure.

### D6. Loops: the causal `EndoArrow` as a flow endomorphism

A loop iterates a flow endomorphism `step: Fn(CausalFlow<V, S, C>) -> CausalFlow<V, S, C>`. This is the
causal analogue of `haft::EndoArrow`: there the carrier is `S` and the step is `S → S`; here the carrier
is the flow and the step is `flow → flow`, which threads value, state, context, error, and logs through
each iteration (§10's monoid of endomorphisms on the effect carrier). Three forms:

- `iterate_n(n, step)`: apply the step exactly `n` times; an error mid-way short-circuits and skips the rest.
- `iterate_until(pred, max_steps, step)`: apply until `pred(&value)` holds or `max_steps` is reached; the
  predicate is checked on the current value before each step, matching `EndoArrow::iterate_until`.
- `iterate_to_fixpoint(max_steps, step)` (`Value: PartialEq + Clone`): apply until the value stops changing or
  `max_steps` is reached.

Convergence is reported in the monad's own channel. If `iterate_until` or `iterate_to_fixpoint` exhausts `max_steps`
without meeting its condition, the flow short-circuits with a `CausalityError` (`did not converge within
N steps`) rather than returning a non-converged value as a success. This is the flow-level form of
`EndoArrow` returning `(S, false)`: the bound guarantees termination, and missing the target is an error a
caller can `recover` from. `iterate_n(n)` has no convergence notion; it always performs `n` steps (fewer on
error). Because the step is a flow endomorphism, all three work on stateful flows with no extra
machinery, which is what the simulation examples require.

### D7. `Either` placement: `haft` (decided)

`Either<L, R>` goes in `deep_causality_haft`, alongside the Arrow algebra. The reason is reuse: the same
sum is the natural carrier for a future `ArrowChoice` on pure arrows, and it is likely to be reused in the
Causal Discovery Language (CDL), which also builds on `haft`. `core`'s `either` and the flow DSL import it
from `haft`. The cost is that this change now touches `haft` as well as `core`; the benefit is one `Either`
shared across the flow DSL, the pure Arrow algebra, and the CDL, rather than a core-local duplicate.

### D8. The flow DSL surface (summary)

| method | shape | lowers to |
| --- | --- | --- |
| `iterate_n(n, step)` | `step: Fn(CF<V,S,C>) -> CF<V,S,C>` | `n` binds of the endo step |
| `iterate_until(pred, max, step)` | `pred: Fn(&V) -> bool` | bind until `pred` or bound (error at bound) |
| `iterate_to_fixpoint(max, step)` | `V: PartialEq + Clone` | bind until value stable or bound (error at bound) |
| `branch(cond, on_true, on_false)` | arms are flow endos | one bind selecting a continuation |
| `branch_with(cond, on_true, on_false)` | `cond` reads value+state+context | one bind selecting a continuation |
| `either(left, right)` | `self: CF<Either<L,R>,S,C>` | one bind routing the sum |
| `next(pipeline)` | `pipeline: Fn(V) -> CF<U,S,C>` | one bind applying a whole pipeline (composition verb) |
| `and_then(\|v\| arrow.run(v))` | `arrow: CausalArrow<V, U, S, C>` | apply a reified engine composite (existing combinator, no dedicated method) |

Each keeps the facade contract: the value is handed over unwrapped, an errored flow short-circuits, and
state, context, and logs thread exactly as the existing steps.

## Worked rewrites (the showcase)

Snippets are illustrative of the shape (as in the `causal-flow-dsl` design), not compile-exact; the exact
closure plumbing is settled during implementation, and each rewrite preserves the example's output.

### W1. `corrective_lane_keeping` — loop + conditional intervention

The closed loop today is a `for` over a `let mut process`: advance one tick, read the offset out of
`process.value`, and `intervene` when it crosses the threshold.

```rust
fn run_closed_loop() -> LaneProcess<FloatType> {
    let mut process = model::initial_process();
    for _ in 0..N_TICKS {
        process = process.bind(model::simulate_step);
        let current = match &process.value { EffectValue::Value(v) => *v, _ => continue };
        let cfg = process.context.clone().expect("LaneConfig present");
        if current.abs() > cfg.anomaly_threshold {
            let corrected = model::correction(current, &cfg);
            process.state.correction_count += 1;
            process = process.intervene(corrected);
        }
    }
    process
}
```

After: the `for` becomes `iterate_n`, the `if` becomes `branch_with`, and the corrective intervene is the
true arm. No `let mut`, no manual `EffectValue` match, iteration is bounded by the type, and an errored
tick short-circuits the whole loop.

```rust
fn run_closed_loop() -> LaneProcess<FloatType> {
    CausalFlow::from(model::initial_process())
        .iterate_n(N_TICKS, |tick| {
            tick.bind(model::simulate_step)                  // advance one tick (existing stage, passthrough)
                .branch_with(
                    |v, _st, ctx| v.abs() > ctx.unwrap().anomaly_threshold,
                    |hot|  hot.intervene_correction(),        // record + .intervene(corrected)
                    |cold| cold,                              // nominal: pass through
                )
        })
        .into_process()
}
```

The open loop is the same `iterate_n` without the branch: `CausalFlow::from(initial).iterate_n(N_TICKS, |t|
t.bind(simulate_step)).into_process()`. The before/after makes the open-vs-closed contrast one line: the
presence of the `branch`. The other three `corrective_*` examples (glucose pump, network failover,
decompression stops) share this shape exactly.

### W2. `geometric_tcas` — loop, closed-loop interlock, and an Euler step

Today a 30-step `for` runs the threat assessment, computes a `triggered` flag, drives `intervene_if` on a
`CausalFlow` per tick, and advances the kinematics with an `Euler` endo-arrow. The loop and the per-tick
flow are separate; the flow is rebuilt from scratch every tick.

```rust
for t in 0..30 {
    let report = tcas.assess_threat(&ownship, &intruder);
    let will_intervene = /* ra persisted and descent authority left */;
    ownship.vel = CausalFlow::value(ownship.vel.clone())
        .intervene_if(|_| will_intervene, |vel| descend(vel))
        .finish().expect("velocity flow always carries a value");
    // ... print row ...
    ownship.pos = Euler::new(dt, |_| own_vel).iterate_n(ownship.pos.clone(), 1);
    // ... advance intruder ...
}
```

After: the encounter is one bounded loop over the engagement state, each tick a flow step that assesses,
applies the interlock, and integrates one Euler step. `iterate_until` ends the run when the encounter
resolves, or at the 30-step bound.

```rust
CausalFlow::value(Engagement::initial(ownship, intruder))
    .iterate_until(|e| e.resolved(), 30, |tick| {
        tick.map(assess_and_log)                     // assess threat, print the row
            .branch(|e| e.ra_committed(),            // interlock: auto-pilot takes over
                    |fire| fire.intervene(descend),  //   substitute the descent vector, audited
                    |hold| hold)                     //   otherwise hold course
            .map(|e| e.step_kinematics(dt))          // one Euler integration step
    })
    .run(report_complete, report_abort);
```

### W3. `hypersonic_2t` and `magnav` — simulation loops the linear DSL could not host

These were left out of the earlier flow migration on the grounds that "their structure is iterative, not
a monadic chain." The loop DSL is exactly the missing piece. `hypersonic_2t` is a pure 20-step tracker
loop (predict, observe, derive metrics, carry history); `magnav` is a 30-step navigation loop (one Euler
kinematics step, sensor sim, particle-filter predict/update/resample). Both become `iterate_n` over a state
that bundles what the `for` threaded by hand:

```rust
// hypersonic_2t: state = (tracker, prev_pos, prev_vel)
CausalFlow::value(TrackState::acquire(init))
    .iterate_n(20, |tick| tick.map(|s| s.predict(dt).observe().log_row()))
    .finish()?;

// magnav: state = (true_pos, filter), kinematics is an Euler endo-arrow reused each tick
CausalFlow::value(NavState::start(true_pos, filter))
    .iterate_n(30, |tick| tick.try_step(|s| s.step_truth(&kinematics).sense(&map).filter_update(&map)))
    .finish()?;
```

`magnav`'s measurement update already returns a `Result`, so it rides the flow's error channel through
`try_step` instead of the example's `?` on each call; a filter failure short-circuits the loop with the
error, which the linear `for` could not express.

### W4. Composing three pipelines (through the DSL)

The new requirement, shown on its own. Each process is authored as a pipeline with the CausalFlow DSL, and
the three wire together with `next`. No arrow type appears at the call site:

```rust
fn sense(raw: Raw)    -> CausalFlow<Reading> { CausalFlow::value(raw).try_step(parse).try_step(calibrate) }
fn decide(r: Reading) -> CausalFlow<Plan>    { CausalFlow::value(r).try_step(estimate).map(choose_plan) }
fn act(p: Plan)       -> CausalFlow<Command> { CausalFlow::value(p).try_step(authorize).map(emit) }

let command = CausalFlow::value(raw_frame)
    .next(sense)
    .next(decide)
    .next(act)
    .finish()?;

// the same pipeline reused inside a loop tick, still through the DSL:
CausalFlow::value(state).iterate_n(N, |tick| tick.next(controller_step));
```

Each process is reusable as a named function. The avionics tick in W2 is the same idea at smaller grain
(assess, then interlock, then integrate); W4 shows it at the level of whole processes. The reified
`CausalArrow` engine (D4) sits underneath for the case a composite must be held as data, and is kept out
of the showcase by design.

## Risks / Trade-offs

- **Flow-endomorphism arms read as higher-order.** A `branch` arm or a `iterate_n` step is `Fn(CausalFlow) ->
  CausalFlow`, which is one notch more abstract than a value closure. The mitigation is that the common arm
  is `|f| f`, the common step is `|f| f.and_then(stage)`, and both compose with the existing fluent
  methods, so call sites stay readable (W1–W3).
- **Non-convergence as an error.** Turning a hit step-bound into a `CausalityError` (D6) is opinionated;
  the alternative is a `(value, converged)` pair. The error form keeps the loop inside the monad's one
  short-circuit channel and lets `recover` express a fallback, consistent with the rest of the facade.
  Flagged for review.
- **Two arrow notions in the tree.** `haft::Arrow` (pure) and `CausalArrow` (Kleisli) coexist. The
  mitigation is the explicit D1 correspondence and the shared `Arrow` interface, so the relationship is
  named rather than left to inference.
- **Surface growth on `CausalFlow`.** The loop, branch, and composition methods (`iterate_n`,
  `iterate_until`, `iterate_to_fixpoint`, `branch`, `branch_with`, `either`, `next`). Each maps to one
  bind-threading lowering and reuses the existing short-circuit contract, so the model stays "named sugar
  over the monad." Applying a reified engine composite reuses the existing `and_then`; no `pipe` is added.

## Resolved decisions

- **`CausalArrow` is the engine, the DSL is the surface (D4).** The reified `CausalArrow` value stays as an
  underlying layer for named, storable composites, but every example and the headline API compose pipelines
  through the CausalFlow DSL (`next` / `and_then`). The arrow type does not appear in the showcase.
- **`Either` lives in `haft` (D7).** Placed for reuse: the pure `ArrowChoice` carrier and the CDL, both on
  `haft`, can share it. The change touches `haft` and `core`.
- **Non-convergence short-circuits with a `CausalityError` (D6).** A hit step-bound on `iterate_until` /
  `iterate_to_fixpoint` fails the flow rather than returning a flag, keeping one short-circuit mechanism consistent
  across the flow DSL and the monad. A caller that wants a fallback uses `recover`.
- **Branch arms (and loop steps) are flow endomorphisms (D5).** `branch` / `branch_with` / `either` arms
  are `Fn(CausalFlow<V,S,C>) -> CausalFlow<V,S,C>`, the same shape as an `iterate_n` step. This threads
  state, context, and logs through the arm (the stateful `corrective_*` and avionics examples need it) and
  lets an arm be a composable sub-pipeline. The cost, accepted: a `flow.` prefix in the stateless case.
- **Naming (decided).** The loop family is the `iterate_` set, mirroring `haft::EndoArrow`'s own method
  names: `iterate_n` (fixed count), `iterate_until` (predicate), `iterate_to_fixpoint` (value stabilizes).
  Sequential composition is `next` ("the next sub-process") on both the flow DSL and the engine builder;
  the existing `and_then` stays for an inline single stage and also applies a reified engine composite
  (`and_then(|v| arrow.run(v))`), so no separate `pipe` verb is added. Branches are `branch` /
  `branch_with` / `either`.
