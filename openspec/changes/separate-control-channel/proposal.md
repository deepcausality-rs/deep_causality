## Why

`EffectValue<T>` conflates *values* (`None`, `Value`, `ContextualLink`) with *control operations*
(`RelayTo` = a computed jump, `Map` = a dispatch table). That conflation is the root cause of five
formalization deviations (`core-formalization-plan.md`): D5 (`PartialEq` is a partial-equivalence
relation — `Map(_)==Map(_)` is always `false`, `RelayTo` equality ignores its payload), D6
(`into_value` conflates "absent" with "dispatch"), D14 (`map` ≠ `bind(pure∘f)` off the value carrier),
and D15 (four disagreeing `fmap`s, one of which **panics** via `.expect` in the arity-5 witness). It
also blocks `core.causal_monad.lawful`: no lawful `Functor`/`LawfulMonad` instance exists over the fused
type. Making the conflation lawful in place (Option A: recursive `fmap` + structural eq on the fused
type) was rejected; the author's decision (`§2A`) is **Option B — end the conflation**: values stay in
`EffectValue`, control operations move to a separate operation functor consumed by a handler.

This is the textbook algebraic-effects / free-monad shape (`Free f a = Pure a | Op (f (Free f a))`;
Plotkin & Power 2003, Swierstra 2008): `EffectValue` is the value/`Pure` part, `CausalCommand` is the
operation functor `f`, and the existing graph-reasoning BFS is the handler that interprets the jump.

## What Changes

- **BREAKING** Remove the **dead** `Map` variant of `EffectValue` entirely (verified: constructed only
  in its own unit tests) — not renamed, deleted with `is_map` and its tests.
- **BREAKING** Remove `RelayTo` from `EffectValue<T>`, leaving the pure value functor
  `EffectValue<T> = { None, Value(T), ContextualLink(id, id) }` — a lawful pointed functor
  unconditionally: total `fmap`, a **derived** congruent `PartialEq`, and an honest `into_value`.
- **BREAKING** Add the control operation functor `CausalCommand<K> = { RelayTo(usize, K) }` (single
  hole `K`) with `CausalCommandWitness: HKT + Functor`, and make the adaptive-reasoning program a
  **`deep_causality_haft::Free<CausalCommandWitness, EffectValue<V>>`** — reusing the haft free monad:
  `EffectValue` is the `Pure` part, `CausalCommand` is the operation functor `f`, and the graph BFS is
  `Free::fold` (the F-algebra handler). The free-monad laws come from `haft.free_monad.*`; this change
  proves only that `CausalCommandWitness` is a lawful functor.
- **BREAKING** Widen the carrier outcome to a 3-way sum: `Value(EffectValue<V>) | Error(E) |
  Control(Free<CausalCommandWitness, EffectValue<V>>)` — W-invariant on the value/error arms, control
  its own arm. Add `control() -> Option<&Free<CausalCommandWitness, EffectValue<Value>>>`; `effect()`
  discriminates only `{None, Value, ContextualLink}`.
- Fix the two must-fix bugs that fall out for free: the arity-5 `fmap` **panic** disappears, and the
  non-reflexive `Map` equality is gone.
- Rewrite the adaptive-reasoning handlers (`graph_reasoning/mod.rs`, `graph_reasoning/stateful.rs`) as
  a `Free::fold` over the control program; migrate the CSM evaluator (`csm/eval.rs`) and stateful
  causaloid path (`causable_stateful.rs`) to the `Control` arm — behavior preserved.

## Capabilities

### New Capabilities
- `control-channel`: The removal of the dead `Map`; the separated single-operation control functor
  `CausalCommand<K> { RelayTo(usize, K) }` and its `CausalCommandWitness: HKT + Functor`; the
  adaptive-reasoning program as `haft::Free<CausalCommandWitness, EffectValue<V>>`; the carrier's
  `Control` arm + `control()` accessor; and the reasoning handler as `Free::fold`. Defines the
  value/control separation and the free-monad handler seam.

### Modified Capabilities
- `lawful-effect-channel`: The carrier outcome changes from `Result<EffectValue<Value>, Error>` to the
  3-way `Value | Error | Control` sum, and `EffectValue` loses its two control variants — so the
  "Value-XOR-error…" carrier requirement and the "Accessor surface" requirement change.

## Impact

- **Downstream of nothing; prerequisite of `formalize-deep-causality-core`.** Independent of
  `causal-arrow-state-threading` (either order).
- **BREAKING public API** (`deep_causality_core`): `EffectValue` variants removed; the carrier outcome
  type; `predicates.rs` (`is_relay_to`/`is_map`), `partial_eq.rs`, `from.rs`, `display.rs`, `hkt.rs`,
  and the process getters. `deep_causality_core` version bump.
- **New dependency use:** `deep_causality_core` now uses `deep_causality_haft::Free`/`FreeWitness`
  (already a dependency); the control arm and `CausalCommand` are `alloc`-gated exactly as the haft
  `Free` is.
- **Consumers** (`deep_causality`): the graph-reasoning handlers (×2) are **rewritten as a `Free::fold`**
  (larger than a match-arm swap); CSM eval + stateful causaloid + ~7 test files move to the `Control`
  arm. Causaloids emit a control program instead of `EffectValue::RelayTo(..)`.
- **Formalization:** proves `CausalCommandWitness` functor laws here (citing `haft.free_monad.*`);
  unblocks `core.causal_monad.lawful` plus clean `EffectValue.lean` / `Consistency.lean` downstream.
- **Risk:** medium-high — a breaking change across a crate boundary **and** a reasoning-engine refactor
  to a fold. Mitigated by the single-hole operation functor (simplest case), the fold algebra
  transcribing the current BFS, and full `bazel test //...` + graph-reasoning regression coverage.
