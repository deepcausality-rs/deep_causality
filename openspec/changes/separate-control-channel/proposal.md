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

- **BREAKING** Remove `RelayTo` and `Map` from `EffectValue<T>`, leaving the pure value functor
  `EffectValue<T> = { None, Value(T), ContextualLink(id, id) }`. This makes `EffectValue` a lawful
  pointed functor unconditionally: total `fmap`, a derivable **congruent** `PartialEq`
  (`#[derive]`-able), and an honest `into_value`.
- **BREAKING** Add the control operation functor
  `CausalCommand<T> = { RelayTo(usize, Box<PropagatingEffect<T>>), Dispatch(HashMap<IdentificationValue, Box<PropagatingEffect<T>>>) }`
  (`Map` renamed to `Dispatch`), with a `Functor` instance and a **structural** congruent equality
  (compare `RelayTo` payload and the `Dispatch` map recursively) replacing the broken PER.
- **BREAKING** Widen the carrier outcome to a 3-way sum: `Value(EffectValue<V>) | Error(E) |
  Control(CausalCommand<V>)` — keeping the W-invariant on the value/error arms and giving control its
  own arm (no re-widening of the value carrier). Add a `control() -> Option<&CausalCommand<Value>>`
  accessor; `effect()` now discriminates only `{None, Value, ContextualLink}`.
- Fix the two must-fix bugs that fall out for free: the arity-5 `fmap` **panic** disappears (uniform
  total `fmap` over the clean functor), and the non-reflexive `Map` equality is gone.
- Migrate the adaptive-reasoning consumers to the control arm: the graph-reasoning handlers
  (`graph_reasoning/mod.rs`, `graph_reasoning/stateful.rs`), the CSM evaluator (`csm/eval.rs`), and the
  stateful causaloid path (`causable_stateful.rs`) read `Control(CausalCommand::RelayTo/Dispatch)`
  instead of `EffectValue::RelayTo/Map`.

## Capabilities

### New Capabilities
- `control-channel`: The separated control operation functor `CausalCommand` and its role as the
  operation functor of the adaptive-reasoning free monad — its `Functor` instance, its structural
  congruent equality, and the carrier's `Control` arm and `control()` accessor. Defines the value/
  control separation and the handler seam.

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
- **Consumers** (`deep_causality`): ~4 src sites (graph reasoning ×2, CSM eval, stateful causaloid) +
  ~7 test files move to the `Control` arm. Causaloids emit a control carrier instead of
  `EffectValue::RelayTo(..)`.
- **Formalization unblocked:** `core.causal_monad.lawful`, plus clean `EffectValue.lean`,
  `CausalCommand.lean`, and `Consistency.lean` (no panic, functors agree) in the downstream change.
- **Risk:** medium — a breaking change across a crate boundary. Mitigated by the small, enumerated
  consumer set and full `bazel test //...` coverage.
