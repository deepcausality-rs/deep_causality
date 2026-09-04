> **Final shape (implemented):** the design converged on `CausalEffect<V> = Free<CausalCommandWitness, Option<V>>`
> (design.md D0), deleting `EffectValue` entirely rather than the intermediate `EffectPayload`/`Dispatch`
> draft the tasks below were first written against. `Map` and `ContextualLink` were verified dead and
> deleted (both mirrors). The carrier outcome is `Result<CausalEffect<V>, Error> = Except E (Free CausalCommand (Maybe V))`.

## 1. Preflight

- [x] 1.1 `Map` verified dead (tests only) — removed, not renamed.
- [x] 1.2 `ContextualLink` verified dead repo-wide (both `EffectValue` and the `ActionParameterValue` mirror) — removed.
- [x] 1.3 `deep_causality_haft::{Free, FreeWitness}` reused; `alloc`-gated as needed.
- [x] 1.4 Blast radius measured (538 `EffectValue` occurrences / 105 files) + congruence with `CausalMonad.lean` checked before proceeding.

## 2. Control operation functor + free-monad effect

- [x] 2.1 `CausalCommand<K> { RelayTo(usize, K) }` (single-hole operation functor) + `CausalCommandWitness: HKT<NoConstraint> + Functor`.
- [x] 2.2 `CausalEffect<V>` newtype = `Free<CausalCommandWitness, Option<V>>`: constructors (`value`/`none`/`from_option`/`relay_to`), accessors (`as_value`/`into_value`/`is_none`/`is_value`/`is_command`/`command_target`/`into_command`), total `map`, `fold` handler, manual `Debug`/`Clone`/`PartialEq` (congruent) + `Default`.

## 3. Delete EffectValue; make the value channel `Option`

- [x] 3.1 Delete the `effect_value` module (type + 5 files); `Option<V>` is the `Free` leaf.
- [x] 3.2 D5/D6/D14/D15 dissolved: total `fmap` (maps `Option` leaves through `Free`), honest `into_value`, congruent structural eq, and the arity-5 `.expect` **panic removed**.

## 4. Carrier outcome = `Result<CausalEffect<V>, Error>`

- [x] 4.1 Rewire `mod.rs` (outcome field, `new`/`into_parts`/`into_value`, `bind` continuation takes `CausalEffect`, `fmap`, constructors + `relay_to`, `from_effect`/`from_effect_with_log`).
- [x] 4.2 `getters.rs` (`value`/`value_cloned`/`effect` → `&CausalEffect`/`command_target`), `hkt.rs` (panic-free total fmap/apply), `alternatable_value.rs`; predicates/display/explain unchanged (Result-based).
- [x] 4.3 `causal_flow` facade (`steps`/`branch`/`iterate`/`construction`/`mod` `ok_leaf`): match value/none/command via accessors.
- [x] 4.4 Core green: 198 tests + 18 new `CausalEffect` tests + doctests; clippy clean; W-invariant preserved.

## 5. Reasoning engine as `Free::fold`

- [x] 5.1 `graph_reasoning/mod.rs` + `stateful.rs`: the `RelayTo` BFS branches rewritten as the `Free::fold` handler (`command_target()` + `into_command()`), threading the relaying node's state/context/logs; missing-target error + stop-on-command preserved.
- [x] 5.2 `causable.rs` / `causable_stateful.rs` / `causable_utils.rs` / `monadic_collection*` / `csm`: read value/none/command via accessors; command outputs pass through for the engine to fold.
- [x] 5.3 `deep_causality` green (1097 + 24 tests), clippy clean — adaptive-reasoning behavior preserved.

## 6. Downstream + verify

- [x] 6.1 `deep_causality_cfd` migrated (663 tests) — used only `{Value, None}`, mechanical.
- [x] 6.2 All examples migrated; `deep_causality` benches migrated (`from_effect_value` → `from_value`).
- [x] 6.3 `grep` clean: no `EffectValue`/`from_effect_value` in code anywhere (doc comments updated too).
- [x] 6.4 `cargo build --workspace --all-targets` clean + formatted; **`bazel test //...` green repo-wide**.

## 7. Formalization

- [x] 7.1 `CausalMonad.lean` congruence note: value content is `Free CausalCommand (Maybe V)`; laws over the `Pure` fragment; `EPP = CausalMonad ⊕ CausalEffect`; P1 resolved. Typechecks (bare `lean`).
- [ ] 7.2 `CausalCommand` functor-law Lean witness + `THEOREM_MAP` rows — **deferred to `formalize-deep-causality-core`** (owns core formalization; free-monad laws already `haft.free_monad.*`).

## 8. Close-out

- [ ] 8.1 Reconcile proposal.md / specs to the `CausalEffect` final shape (still describe the superseded `EffectPayload`/`Dispatch` draft), then archive. Prepare per-crate commit messages; ask before committing.
