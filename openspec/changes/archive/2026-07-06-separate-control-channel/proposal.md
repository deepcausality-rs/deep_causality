## Why

`EffectValue<T>` conflates *values* (`None`, `Value`, `ContextualLink`) with *control operations*
(`RelayTo` = a computed jump, `Map` = a dispatch table). That conflation is the root cause of five
formalization deviations (`core-formalization-plan.md`): D5 (`PartialEq` is a partial-equivalence
relation — `Map(_)==Map(_)` is always `false`, `RelayTo` equality ignores its payload), D6
(`into_value` conflates "absent" with "dispatch"), D14 (`map` ≠ `bind(pure∘f)` off the value carrier),
and D15 (four disagreeing `fmap`s, one of which **panics** via `.expect` in the arity-5 witness). It
also blocks `core.causal_monad.lawful`: no lawful `Functor`/`LawfulMonad` instance exists over the fused
type. Making the conflation lawful in place (Option A: recursive `fmap` + structural eq on the fused
type) was rejected; the decision was **end the conflation** — and the design then converged all the way
to deleting `EffectValue`: the value functor is standard `Option`, control is a separate operation
functor, and both live inside one free monad.

This is the textbook algebraic-effects / free-monad shape (`Free f a = Pure a | Op (f (Free f a))`;
Plotkin & Power 2003, Swierstra 2008): `Option<V>` (`Maybe`) is the value/`Pure` part, `CausalCommand`
is the operation functor `f`, and the graph-reasoning BFS is the `Free::fold` handler.

## What Changes

> **Final shape (implemented).** After downstream investigation and two `invent` passes the design
> converged past the intermediate `EffectPayload`/3-way draft to the **fully unified free monad**:
> `EffectValue` is *deleted* and the success channel becomes
> `CausalEffect<V> = Free<CausalCommandWitness, Option<V>>`.

- **BREAKING** Delete the **dead** `Map` and `ContextualLink` variants entirely (both verified dead —
  `Map` used only in its own tests; `ContextualLink` never interpreted, incl. the `ActionParameterValue`
  input-side mirror). Neither is renamed.
- **BREAKING** Delete `EffectValue` entirely. Its `{None, Value}` become the `Option<V>` (`Maybe`) value
  leaf of the free monad — the value functor is now standard `Option`, already lawful (haft).
- **BREAKING** Add the control operation functor `CausalCommand<K> = { RelayTo(usize, K) }` (single hole
  `K`) with `CausalCommandWitness: HKT + Functor`, and add
  `CausalEffect<V> = Free<CausalCommandWitness, Option<V>>` — the adaptive-reasoning program: `Pure(None)`
  = absence, `Pure(Some v)` = a value, `Suspend(RelayTo(t,k))` = a command. The graph BFS is `Free::fold`
  (the F-algebra handler); the free-monad laws come from `haft.free_monad.*`.
- **BREAKING** The carrier outcome is `Result<CausalEffect<V>, Error> = Except E (Free CausalCommand
  (Maybe V))` — a transformer stack of three proven monads. W-invariant on value/error; command is a
  *second* left-zero of `bind` (the engine folds it). Add `command_target()`/`into_command()`;
  `effect()` returns `&CausalEffect`.
- Fix D5/D6/D14/D15 **structurally**: total panic-free `fmap` (maps `Option` leaves through `Free`),
  honest `into_value`, congruent structural equality; the arity-5 `.expect` panic is gone.
- Rewrite the adaptive-reasoning handlers (`graph_reasoning/mod.rs`, `graph_reasoning/stateful.rs`) as a
  `Free::fold` (`command_target`/`into_command`); migrate CSM eval + `causable*` to accessor-based
  value/none/command handling — behavior preserved.

## Capabilities

### New Capabilities
- `control-channel`: The removal of the dead `Map`/`ContextualLink`; the single-operation control functor
  `CausalCommand<K> { RelayTo(usize, K) }` + `CausalCommandWitness: HKT + Functor`; the unified
  adaptive-reasoning effect `CausalEffect<V> = haft::Free<CausalCommandWitness, Option<V>>`; the carrier
  outcome `Result<CausalEffect<V>, Error>` with `command_target()`/`into_command()`; and the reasoning
  handler as `Free::fold`. Defines the value/command/error separation as the transformer stack
  `Except ∘ Free ∘ Maybe`.

### Modified Capabilities
- `lawful-effect-channel`: The carrier outcome changes from `Result<EffectValue<Value>, Error>` to
  `Result<CausalEffect<Value>, Error>` (the free-monad success channel), and `EffectValue` is deleted —
  so the "Value-XOR-error…" carrier requirement and the "Accessor surface" requirement change.

## Impact

- **Downstream of nothing; prerequisite of `formalize-deep-causality-core`.** Independent of
  `causal-arrow-state-threading` (either order).
- **BREAKING public API** (`deep_causality_core`): `EffectValue` deleted (its 5-file module); the
  carrier outcome type and accessors; `hkt.rs`. New public `CausalCommand`/`CausalEffect`.
- **Reuses `deep_causality_haft::Free`/`FreeWitness`** (already a dependency); `CausalEffect`/
  `CausalCommand` are `alloc`-gated as the haft `Free` is.
- **Consumers** (`deep_causality`): the graph-reasoning handlers (×2) **rewritten as a `Free::fold`**
  (`command_target`/`into_command`); CSM eval + `causable*` + `monadic_collection*` + tests + benches
  migrated. Causaloids emit `PropagatingEffect::relay_to(..)` instead of `EffectValue::RelayTo(..)`.
  Also removes the dead `ActionParameterValue::ContextualLink` mirror.
- **Formalization:** `CausalMonad.lean` congruence note (value content = `Free CausalCommand (Maybe V)`;
  laws over the `Pure` fragment; `EPP = CausalMonad ⊕ CausalEffect`; P1 resolved). The `CausalCommand`
  functor-law Lean witness rides with `formalize-deep-causality-core`.
- **Blast radius (measured):** 538 `EffectValue` occurrences / 105 files. Mostly mechanical
  (compiler-verified); the delicate core is the carrier + reasoning `fold` + match rewrites.
- **Verified:** core 216 tests (+18 `CausalEffect`) + doctests; `deep_causality` 1097+24; cfd 663; all
  examples + benches; `cargo build --workspace --all-targets` + clippy clean; **`bazel test //...` green**.
