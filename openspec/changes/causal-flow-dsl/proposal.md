## Why

The core causal monad (`CausalEffectPropagationProcess<Value, State, Context, Error, Log>`) is expressive but its construction surface is verbose and leaks the HKT layer: callers write `PropagatingEffect::pure(())` with its `Default + Clone + Debug` bounds, lift a stateful process with `with_state(pure(()), state, Some(ctx))`, match `EffectValue<Value>` inside every `bind` closure, and unwrap the final `value.into_value()` by hand. The `deep_causality_discovery` CDL proved that a fluent builder can hide all of this (witness types, constructors, `EffectValue`, error short-circuit) behind a readable chain; this change brings the same fluency to the general monad so every `PropagatingEffect` / `PropagatingProcess` pipeline reads as a clean flow.

## What Changes

- Add **`CausalFlow`**, a fluent facade over `CausalEffectPropagationProcess`, in `deep_causality_core`. It is a generic facade (not a fixed-stage type-state builder like CDL, because the core monad has no fixed stages).
- **Uniform entry** for both carriers, hiding the constructors and the witness: `CausalFlow::effect()` (stateless → `PropagatingEffect`) and `CausalFlow::process(state)` / `.context(cfg)` (stateful → `PropagatingProcess`, hiding `with_state(pure(()), …)`). Plus `CausalFlow::value(v)` and `CausalFlow::fail(err)`.
- **Fluent steps** that hand the closure the *unwrapped* value and short-circuit automatically on `None`/error, so no `EffectValue` matching appears in user code: `.step(|v| -> CausalFlow<…>)`, `.try_step(|v| -> Result<U, CausalityError>)`, `.map(|v| u)`, `.guard(|v| Result<(), _>)`; and for the stateful carrier `.step_with(|v, &state, &ctx| …)` and `.update_state(|state, &v| …)`.
- **Terminals** that hide the final `EffectValue` extraction: `.finish() -> Result<T, CausalityError>`, `.run(on_ok, on_err)`, and escape hatches `.into_effect()` / `.into_process()` that return the underlying type for interop with existing code.
- The change is **additive and non-breaking**: the existing `CausalEffectPropagationProcess` / `PropagatingEffect` / `PropagatingProcess` API is unchanged, and `CausalFlow` is a thin wrapper that lowers to it. A subset of the monadic examples are migrated to demonstrate the before/after.
- No new external dependency; no change to the witness/HKT layer in `deep_causality_haft`.

## Capabilities

### New Capabilities

- `causal-flow`: a fluent builder facade over the causal-effect propagation monad that constructs and chains both `PropagatingEffect` and `PropagatingProcess` while hiding the HKT witness types, the verbose constructors, the `EffectValue` wrapping, and the manual error short-circuit.

### Modified Capabilities

<!-- None. The change is additive; it does not alter the requirements of the existing causal-monad behavior, only adds a facade that lowers to it. -->

## Impact

- **Code**: new `flow` module in `deep_causality_core` (the `CausalFlow` type, its fluent methods, and `From`/`Into` conversions to and from `CausalEffectPropagationProcess`). No edits to the existing monad types beyond what conversions require.
- **APIs**: purely additive. `CausalFlow` is opt-in; existing `pure` / `bind` / `with_state` call sites keep working.
- **Examples**: migrate the chronometric `gm_recovery` example to the flow facade as the headline before/after (its five-stage chain plus a verbose terminal `match` make the improvement vivid), and optionally a second monadic example, as living documentation.
- **Dependencies**: none added. Edition 2024; `unsafe_code = "forbid"`; the crate's existing lint and coverage policy applies to the new module.
