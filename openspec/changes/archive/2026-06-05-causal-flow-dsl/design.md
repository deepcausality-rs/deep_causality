## Context

The causal monad in `deep_causality_core` is one struct,
`CausalEffectPropagationProcess<Value, State, Context, Error, Log>`, with two aliases:
`PropagatingEffect<T>` fixes `State = Context = ()`, and `PropagatingProcess<T, S, C>` fixes
`Error = CausalityError`, `Log = EffectLog`. The HKT layer adds three witness types
(`PropagatingEffectWitness`, `PropagatingProcessWitness`, `CausalEffectPropagationProcessWitness`).

The monad is sound but verbose to drive. Real call sites show the friction:

```rust
let pipeline = PropagatingEffect::pure(())
    .bind(move |_, _, _| stage1(...))
    .bind(move |s1, _, _| match s1.into_value() {
        Some(s) => stage2(s),
        None => error_effect("stage 1 produced no value"),
    });
match pipeline.value.into_value() {
    Some(report) => print(report),
    None => eprintln!("failed: {:?}", pipeline.error),
}
```

The boilerplate is the `pure(())` seed, the `EffectValue<Value>` match inside each `bind`, the manual
error short-circuit, and the terminal `value.into_value()` match. The `deep_causality_discovery` CDL
removed exactly this kind of boilerplate with a fluent builder. A key difference governs the design:
the CDL is a fixed-stage type-state builder (it knows `load → clean → … → finalize`), whereas the core
monad is generic with no fixed stages, so the facade must be a generic fluent wrapper, not a
type-state-per-stage builder.

## Goals / Non-Goals

**Goals:**

- A fluent facade, `CausalFlow`, that constructs and chains both `PropagatingEffect` and
  `PropagatingProcess` through one uniform API.
- Hide the HKT witness types, the verbose constructors (`pure`, `with_state`), the `EffectValue`
  wrapping, and the manual error short-circuit at the call site.
- Lower losslessly to `CausalEffectPropagationProcess`, with conversions in both directions so the
  facade interoperates with existing monad code.
- Additive and non-breaking: no change to the existing monad types or the witness layer.

**Non-Goals:**

- Not a fixed-stage type-state builder (that pattern is domain-specific, as in the CDL).
- Not a replacement for `CausalEffectPropagationProcess`, `bind`, or the `CausalMonad` trait; the
  facade lowers to them.
- Not async, and not a change to `deep_causality_haft`'s HKT machinery.
- Not an attempt to erase the monad's intrinsic `Default + Clone + Debug` bounds; the facade
  centralizes them but cannot remove them.

## Decisions

### D1. A generic fluent facade, not a macro and not type-state stages

`CausalFlow` is a thin newtype wrapper, chosen over a `flow! { … }` macro (less magic, composes with
ordinary control flow, better IDE support) and over a CDL-style type-state builder (inapplicable
because the core flow has no fixed stages).

```rust
pub struct CausalFlow<Value, State = (), Context = ()> {
    inner: CausalEffectPropagationProcess<Value, State, Context, CausalityError, EffectLog>,
}
```

The `State = Context = ()` defaults make `CausalFlow<T>` the stateless form, mirroring
`PropagatingEffect<T>`. `Error`/`Log` are fixed to `CausalityError`/`EffectLog`, so the witness types
never appear in a signature the user reads.

### D2. Location: a `flow` module inside `deep_causality_core`

`CausalFlow` lives next to the monad it lowers to, in `deep_causality_core::flow`, rather than a
separate crate. It is a thin facade with no extra dependencies, so the cost of keeping the core lean
does not justify a new crate. (If the facade later grows domain-specific builders, splitting it out
stays an option.)

### D3. Uniform construction hides the constructors and the witness

```rust
CausalFlow::effect()                 // CausalFlow<()>            (stateless start, unit value)
CausalFlow::value(v)                 // CausalFlow<V>             (stateless start with a value)
CausalFlow::fail(err)                // CausalFlow<V>             (start in the error channel)
CausalFlow::process(state)           // CausalFlow<(), S, ()>     (stateful; hides with_state(pure(())…))
    .context(cfg)                    // CausalFlow<(), S, C>
```

`process(state).context(cfg)` lowers to `with_state(pure(()), state, Some(cfg))`. The user never names
`with_state`, `pure`, or a witness.

### D4. Fluent steps hide `EffectValue` and short-circuit automatically

Steps hand the closure the *unwrapped* `Value` (never `EffectValue`) and propagate an existing error
or absent value without running the closure. The full set (refined by the stress tests below) is:

```rust
.and_then(|v| -> CausalFlow<U, S, C>)           // full monadic step; effect-returning stages via .into()
.try_step(|v| -> Result<U, CausalityError>)     // common case: Ok -> value, Err -> error channel
.map(|v| -> U)                                  // infallible value transform (fmap)
.guard(|v: &V| -> Result<(), CausalityError>)   // validate; Err short-circuits, Ok passes v through
.recover(|err| -> U)                            // turn the error channel back into a value
.try_step_with(|v, &state, ctx| -> Result<U,_>) // stateful step; threads State/Context automatically
.step_mut(|v, &mut state, ctx| -> Result<U,_>)  // canonical stateful step; mutate state, transform value
.update_value(|v| -> V)                         // evolve the value in place (same-type sibling of map)
.update_state(|state, v: &V| -> S)              // evolve the Markovian state from the value
.update_context(|ctx, v: &V| -> Option<C>)      // evolve the context from the value
.update_value_state_context(|v, s, c| -> (V,S,Option<C>)) // rewrite all three channels at once
.bind(f) / .bind_or_error(f, msg)               // drop-in passthroughs for existing monad-shaped stages
```

`.try_step` wraps `Ok` with `from_value` and `Err` with `from_error`; `.and_then` lowers to `bind`;
`.try_step_with` lowers to `bind` with read access to state and context, re-wrapping the returned value in
the same `State`/`Context`; `.map` lowers to `fmap`; `.bind` / `.bind_or_error` forward to the existing
monad methods so un-migrated stages drop in unchanged. The recurring `Default + Clone + Debug` bounds are
stated once on the facade, not at every call site. The stateful step returns a value rather than a flow so
the facade keeps threading `State`/`Context`; see "Stress tests" below for why.

### D5. Terminals hide the final extraction; escape hatches preserve interop

```rust
.finish() -> Result<Value, CausalityError>   // value or error, no EffectValue match
.run(on_ok: FnOnce(Value), on_err: FnOnce(CausalityError))
.into_effect()  -> PropagatingEffect<Value>            // when State = Context = ()
.into_process() -> PropagatingProcess<Value, State, Context>
```

`From<CausalEffectPropagationProcess<…>> for CausalFlow<…>` and the reverse let existing monad code and
the facade mix freely: a stage written against the raw monad drops into a flow, and a flow drops back
out for code that expects the concrete type.

### D6. Before / after (the `gm_recovery` five-stage pipeline)

Today the chronometric `gm_recovery` example seeds the chain with `pure`, threads each stage through
`bind` (each stage written against the raw bind signature
`Fn(EffectValue<V>, State, Option<Context>) -> PropagatingEffect<U>`), and unwraps the result by hand:

```rust
let result: PropagatingEffect<GmReport<FloatType>> = PropagatingEffect::pure(inputs)
    .bind(stage_load::<FloatType>)
    .bind(stage_align)
    .bind(stage_pair)
    .bind(stage_solve_gm)
    .bind(stage_aggregate);

match result.value {
    EffectValue::Value(report) => print_gm_report(&report),
    _ => {
        eprintln!("Pipeline failed:");
        if let Some(err) = result.error {
            eprintln!("  {:?}", err.0);
        }
        std::process::exit(1);
    }
}
```

With `CausalFlow` the seed drops its type annotation and `pure`, each stage becomes a plain
`Fn(Value) -> CausalFlow<U>` (no `EffectValue` / `State` / `Context` in its signature), and the
terminal match collapses to `run`:

```rust
CausalFlow::value(inputs)
    .step(stage_load::<FloatType>)
    .step(stage_align)
    .step(stage_pair)
    .step(stage_solve_gm)
    .step(stage_aggregate)
    .run(print_gm_report, |err| {
        eprintln!("Pipeline failed:\n  {err:?}");
        std::process::exit(1);
    });
```

The win is twofold: the call site loses the `pure` seed and the nine-line `EffectValue` match, and the
stage functions in the `pipeline` module shed the `EffectValue` / state / context arguments they never
needed, becoming ordinary `Value -> CausalFlow<U>` functions.

## Stress tests (before / after)

Three further pipelines were run through the design to validate it against real shapes the gm_recovery
case does not exercise: a stateful per-step loop, an error-swallowing `bind` chain, and a chain that
already uses `bind_or_error`. Each surfaced a concrete refinement, folded into the combinator set below.

### S1. `event_horizon_probe` — stateful flow with a context, inside a loop

Before (one loop iteration): the state and the black-hole mass are carried by `with_state(pure(()), …)`,
the step reads them through a `bind` closure, and the result is pulled out with an `EffectValue` match.

```rust
let next_state_effect = CausalEffectPropagationProcess::with_state(
    CausalEffectPropagationProcess::pure(()),
    current_state.clone(),
    Some(black_hole_mass),
)
.bind(|_, state, ctx: Option<Mass<FloatType>>| {
    let bh_mass = ctx.unwrap();
    /* … compute the next ProbeState … */
    CausalEffectPropagationProcess::pure(ProbeState { /* … */ })
});
if let EffectValue::Value(s) = next_state_effect.value() {
    current_state = s.clone();
}
```

After:

```rust
let next = CausalFlow::process(current_state.clone())
    .context(black_hole_mass)
    .try_step_with(|_unit, state, ctx| {
        let bh_mass = *ctx.expect("context holds the black-hole mass");
        /* … compute the next ProbeState … */
        Ok(ProbeState { /* … */ })
    })
    .finish();
if let Ok(s) = next {
    current_state = s;
}
```

**Reveals:** a stateful step must hand back a *value* (`Result<U, _>`), not a `CausalFlow<U, S, C>`. If
it returned a flow, the closure would have to rebuild one with the right `State`/`Context` type
parameters, which is exactly the friction the facade exists to remove. So `try_step_with` (value-in,
`Result`-out, framework threads `S`/`C`) is the primary stateful combinator; the flow-returning form is
the advanced escape hatch. The `with_state(pure(()), …)` seed and the `EffectValue` match are clean wins.

### S2. `grmhd` — a `bind` chain that swallows errors with `unwrap_or_default`

Before: every stage closure unwraps the `EffectValue` with `into_value().unwrap_or_default()` and the
terminal re-checks `is_err()` then unwraps again.

```rust
let result = PropagatingEffect::pure(GrmhdState::new(&config))
    .bind(|state, _, _| { println!("[Step 1] …"); model::calculate_curvature(state.into_value().unwrap_or_default()) })
    .bind(|state, _, _| { println!("[Step 2] …"); model::select_metric(state.into_value().unwrap_or_default()) })
    /* … 3 more … */;
if result.is_err() { eprintln!("Simulation failed: {:?}", result.error); return; }
let final_state = result.value.into_value().unwrap_or_default();
```

After (existing `model::*` stages return `PropagatingEffect`, so `.into()` adapts them):

```rust
let result = CausalFlow::value(GrmhdState::new(&config))
    .and_then(|s| { println!("[Step 1] …"); model::calculate_curvature(s).into() })
    .and_then(|s| { println!("[Step 2] …"); model::select_metric(s).into() });
    /* … 3 more … */
result.run(print_conclusion, |err| eprintln!("Simulation failed: {err:?}"));
```

**Reveals:** (1) the per-stage `into_value().unwrap_or_default()` disappears because `and_then` passes the
raw value; (2) effect-returning stage functions adapt with `From<PropagatingEffect>` via `.into()`, so no
stage rewrite is forced; (3) a *semantic* difference worth naming: `CausalFlow` short-circuits on error
where the original silently defaulted. That is more correct, but if a stage genuinely wants a fallback it
should say so with an explicit `.recover(|_err| GrmhdState::default())`. So the design adds `and_then`
(flow-returning step) and a `recover` combinator.

### S3. `multi_physics_pipeline` — already on `bind_or_error`

Before: this chain is already clean; its named stages have the `(value, (), Option<()>) -> Effect` shape.

```rust
let result = klein_gordon(&phi_manifold, mass)
    .bind_or_error(stage_field_to_partons, "Field → Partons failed")
    .bind_or_error(stage_lund_fragmentation, "Lund fragmentation failed")
    /* … 2 more … */;
print_summary(&result);
```

After (drop-in: one `From`, zero stage changes, via a `bind_or_error` passthrough on the facade):

```rust
CausalFlow::from(klein_gordon(&phi_manifold, mass))
    .bind_or_error(stage_field_to_partons, "Field → Partons failed")
    .bind_or_error(stage_lund_fragmentation, "Lund fragmentation failed")
    /* … 2 more … */
    .run(print_summary_ok, |err| eprintln!("pipeline failed: {err:?}"));
```

**Reveals:** for code already using `bind_or_error` plus named stages, the honest win is small (mostly the
terminal). To make migration zero-friction rather than a rewrite, the facade should expose `.bind(f)` and
`.bind_or_error(f, msg)` passthroughs that accept the *existing* monad-shaped stage functions unchanged. A
later, optional cleanup lets those stages shed their unused `(_, _)` arguments and move to `.try_step`.

### S4. `flight_envelope_monitor` — stateful process whose state update depends on the value

This `PropagatingProcess<_, FlightState, AircraftConfig>` chain threads a Markovian `FlightState`
(estimate, covariance, cumulative `risk`) and a read-only `AircraftConfig`. Every stage reads the value,
mutates the state, and transforms the value in one closure, and the state update *depends on* the value:

```rust
fn health_fold(value: EffectValue<f64>, mut state: FlightState, ctx: …) -> FlightProcess<…> {
    let health = value.into_value()…;
    state.risk += (1.0 - health) * RISK_HEALTH_WEIGHT;   // update derived from the value
    /* … wrap the new value + the new state … */
}
```

`try_step_with` (read-only state) plus `update_state` cannot express this without recomputing `health` in
the update closure; the two phases are one. **Reveals** the canonical stateful combinator
`step_mut(|value, &mut State, Option<&Context>| -> Result<U, _>)`: one closure, mutable state, value in and
out. It matches the example's existing `mut state` stages exactly and lowers straight to `bind`.

### S5. `geometric_tcas` — closed-loop intervention, and kinematics that belong in the calculus

TCAS is a time loop whose monad use is shallow: one `bind` per step for the threat decision, the
`Intervenable::intervene` safety interlock fired mid-loop, and a hand-rolled trajectory update.

```rust
ownship.pos = add_vec(&ownship.pos, &scale_vec(&ownship.vel, dt));   // manual Euler step
…
if triggered { vel_effect.intervene(Some(evasion_vel)) }            // closed-loop override
```

**(F2 — intervention.)** `intervene` is a first-class monad facet (Pearl's Layer 2), so `CausalFlow`
should expose it: `.intervene(new_value)` and the sugar `.intervene_if(cond, |v| new_value)`, lowering to
`Intervenable::intervene` and keeping the audit log. **(Calculus.)** The `pos += vel·dt` is exactly an
`Euler` step, and an accelerating threat wants `Rk4`; the closing rate is `derivative` of `range(t)` and
the CPA is where it crosses zero (or `iterate_until(range < threshold)`); the bivector CPA `|P ∧ V| / |V|`
is already clean. Simplified, TCAS becomes the flagship example, weaving the causal monad, the Arrow
calculus (`Rk4` propagation), the `Intervenable` interlock, and precision-as-a-parameter on one
safety-critical problem:

```rust
CausalFlow::value(engagement)
    .try_step(assess_cpa)
    .intervene_if(ra_fired, |_| evasion_vel)
    .and_then(|s| propagate_rk4(s, dt))
```

### Refined combinator set (result of the stress tests)

| Combinator | Closure | Notes |
| --- | --- | --- |
| `and_then(f)` | `Value -> CausalFlow<U, S, C>` | full monadic step; takes effect-returning stages via `From`/`.into()` (S2) |
| `try_step(f)` | `Value -> Result<U, CausalityError>` | the common stateless case |
| `map(f)` | `Value -> U` | infallible value transform |
| `guard(f)` | `&Value -> Result<(), CausalityError>` | validate; `Err` short-circuits |
| `recover(f)` | `CausalityError -> U` | turn the error channel back into a value (explicit fallback, S2) |
| `try_step_with(f)` | `(Value, &State, Option<&Context>) -> Result<U, CausalityError>` | stateful step, read-only state (S1) |
| `step_mut(f)` | `(Value, &mut State, Option<&Context>) -> Result<U, CausalityError>` | canonical stateful step; mutate state while transforming the value (S4) |
| `update_value(f)` | `Value -> Value` | evolve the value in place; same-type sibling of `map` |
| `update_state(f)` | `(State, &Value) -> State` | evolve the Markovian state from the value |
| `update_context(f)` | `(Option<Context>, &Value) -> Option<Context>` | evolve the context from the value |
| `update_value_state_context(f)` | `(Value, State, Option<Context>) -> (Value, State, Option<Context>)` | rewrite all three channels at once |
| `intervene(v)` / `intervene_if(c, f)` | `Value` / `(&Value) -> bool`, `(Value) -> U` | closed-loop value override (Pearl Layer 2); lowers to `Intervenable` (S5) |
| `bind(f)` / `bind_or_error(f, msg)` | the existing monad signatures | drop-in passthroughs for un-migrated stage fns (S3) |

## Risks / Trade-offs

- **Two ways to build the same monad.** The facade and the raw constructors coexist. Mitigation: the
  conversions are lossless and documented as one being sugar for the other; examples show the facade
  as the recommended default while keeping raw usage valid.
- **Bounds cannot be fully erased.** `Value`/`State`/`Context` still need `Default + Clone + Debug`
  where the underlying `pure`/`bind` require them; the facade centralizes the bounds but a user type
  that violates them still fails to compile (with a clearer single site).
- **Surface growth.** `CausalFlow` adds methods to learn. Mitigation: the method set is small and each
  maps one-to-one to an existing monad operation, so the mental model is "named sugar," not a new
  semantics.
- **`step` vs `try_step` choice.** Offering both risks mild redundancy. Mitigation: document `try_step`
  as the common case and `step` as the escape to full monadic power (custom logs, state threading).
