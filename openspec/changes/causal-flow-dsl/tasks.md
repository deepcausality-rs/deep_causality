## 1. The `CausalFlow` type and module

- [x] 1.1 Add a `flow` module to `deep_causality_core` and define `CausalFlow<Value, State = (), Context = ()>` as a newtype wrapping `CausalEffectPropagationProcess<Value, State, Context, CausalityError, EffectLog>`. Re-export `CausalFlow` from the crate root.
- [x] 1.2 Implement `From<CausalEffectPropagationProcess<Value, State, Context, CausalityError, EffectLog>> for CausalFlow<…>` and the inverse, so the facade and the raw monad convert losslessly.

## 2. Constructors (hide witnesses + `pure` / `with_state`)

- [x] 2.1 `CausalFlow::effect() -> CausalFlow<()>` and `CausalFlow::value(v) -> CausalFlow<V>`, lowering to `PropagatingEffect::pure(())` / a value effect.
- [x] 2.2 `CausalFlow::fail(err) -> CausalFlow<V>`, lowering to `from_error`.
- [x] 2.3 `CausalFlow::process(state) -> CausalFlow<(), S, ()>` and `.context(cfg) -> CausalFlow<(), S, C>`, lowering to `with_state(pure(()), state, Some(cfg))`. No witness type appears in any public signature.

## 3. Fluent steps (hide `EffectValue` + auto short-circuit)

- [x] 3.1 `.and_then(|Value| -> CausalFlow<U, S, C>)`: unwrap the value, short-circuit on absent value or existing error without invoking the closure, otherwise call and lower to `bind`. Accepts effect-returning stages via `From`/`.into()`.
- [x] 3.2 `.try_step(|Value| -> Result<U, CausalityError>)`: `Ok` lifts to a value flow, `Err` to the error channel.
- [x] 3.3 `.map(|Value| U)`: infallible value transform lowering to `fmap`; preserves logs/state/error channel.
- [x] 3.4 `.guard(|&Value| -> Result<(), CausalityError>)`: `Ok` passes the value through, `Err` short-circuits.
- [x] 3.5 `.recover(|CausalityError| -> Value)`: on a failed flow, convert the error into a value and clear the error channel; on a successful flow, leave it unchanged.
- [x] 3.6 Stateful steps: `.try_step_with(|Value, &State, Option<&Context>| -> Result<U, _>)` (read-only state), `.step_mut(|Value, &mut State, Option<&Context>| -> Result<U, _>)` (the canonical case: mutate state while transforming the value), and `.update_state(|State, &Value| -> State)`.
- [x] 3.7 Closed-loop intervention: `.intervene(new_value)` and `.intervene_if(|&Value| -> bool, |Value| -> U)`, lowering to `Intervenable::intervene` and preserving the audit log entry.
- [x] 3.8 Drop-in passthroughs `.bind(f)` and `.bind_or_error(f, msg)` that forward to the existing monad methods so un-migrated stage functions work unchanged.

## 4. Terminals and interop

- [x] 4.1 `.finish() -> Result<Value, CausalityError>` (value or error, no `EffectValue` match).
- [x] 4.2 `.run(on_ok: FnOnce(Value), on_err: FnOnce(CausalityError))`.
- [x] 4.3 `.into_effect() -> PropagatingEffect<Value>` (for `State = Context = ()`) and `.into_process() -> PropagatingProcess<Value, State, Context>`.

## 5. Tests (core crate; 100% coverage on new code)

- [x] 5.1 Constructor tests: `effect`/`value`/`fail`/`process().context()` lower to the expected `CausalEffectPropagationProcess` (value, state, context, error, logs).
- [x] 5.2 Step tests: `and_then`/`try_step`/`map`/`guard` pass the raw value on success, skip the closure and preserve error+logs on a failed flow; `try_step` `Err` short-circuits; `guard` `Err` short-circuits; `recover` clears the error to a value (and is a no-op on success); `try_step_with`/`step_mut`/`update_state` thread and mutate state and context; `intervene`/`intervene_if` substitute the value and log the override (and `intervene_if` is a no-op when the predicate is false); `bind`/`bind_or_error` passthroughs match the underlying monad and `From`/`.into()` adapts an effect-returning stage.
- [x] 5.3 Terminal + round-trip tests: `finish`/`run` dispatch correctly; `From`/`into_effect`/`into_process` round-trip losslessly.
- [x] 5.4 Parity test: a `CausalFlow` chain and the equivalent `pure`/`bind`/`bind_or_error` chain produce equal value, error, and accumulated logs (behavior-preserving sugar), including mid-chain error short-circuit.
- [x] 5.5 Error-path coverage: every `Err` / short-circuit branch in the new module is exercised.

## 6. Demonstration on examples (the stress-test set)

- [x] 6.1 `chronometric_examples/gm_recovery`: seed `value(inputs)`, `.try_step(stage_*)`, `.run(print_gm_report, …)`; adapt the `pipeline` stage signatures from the raw bind shape to `Value -> Result<U, CausalityError>`. Output unchanged. (Headline before/after.)
- [ ] 6.2 `physics_examples/event_horizon_probe`: replace `with_state(pure(()), state, Some(ctx))` + `bind` + `EffectValue` match with `process(state).context(mass).try_step_with(…).finish()` inside the loop. Output unchanged.
- [ ] 6.3 `physics_examples/grmhd`: replace the `into_value().unwrap_or_default()` stages with `value(seed).and_then(|s| model::fn(s).into())` and the `is_err()` terminal with `.run(…)`; make the prior default-on-error explicit with `.recover` where intended. Output unchanged.
- [ ] 6.4 `physics_examples/multi_physics_pipeline`: drop-in migration via `CausalFlow::from(klein_gordon(…))` + the `bind_or_error` passthrough (no stage changes) + `.run(…)`. Output unchanged.
- [ ] 6.5 `avionics_examples/flight_envelope_monitor`: migrate the 5-stage `PropagatingProcess<_, FlightState, AircraftConfig>` chain to `process(state).context(cfg)` + `.step_mut(…)` (state updates derived from the value) + a `.finish()`/`.run()` terminal. Exercises the stateful path. Output unchanged.
- [ ] 6.6 `avionics_examples/geometric_tcas` (flagship): make it precision-generic and lean on `deep_causality_calculus` — replace `pos += vel·dt` with an `Euler`/`Rk4` trajectory step, and express each time step as `CausalFlow::value(engagement).try_step(assess_cpa).intervene_if(ra_fired, evade).and_then(propagate_rk4)`. Combines the causal monad, the Arrow calculus, the `Intervenable` interlock, and precision-as-a-parameter. Output behaviour preserved.

Note: `hypersonic_2t` (imperative tracker loop) and `magnav` (particle-filter loop) are intentionally **not** migrated; their structure is iterative, not a monadic chain, so `CausalFlow` does not apply.

## 7. Verification

- [ ] 7.1 `cargo build` / `cargo test` for `deep_causality_core` and every touched example; `make format && make fix` (0 clippy warnings, no `#[allow(...)]`).
- [ ] 7.2 Doc comments on `CausalFlow` and its methods, each noting the `CausalEffectPropagationProcess` operation it lowers to; a module-level example mirroring the `gm_recovery` before/after.
- [ ] 7.3 Commit message prepared; owner commits. (Edition 2024; `unsafe_code = "forbid"`; additive, non-breaking.)
