## 1. Preflight

- [ ] 1.1 Verify `PropagatingEffect<T>: PartialEq` holds (determines whether `CausalCommand` eq can be `#[derive]`d or must be hand-written congruent)
- [ ] 1.2 Full `grep` inventory of `EffectValue::RelayTo` / `EffectValue::Map` across `deep_causality_core`, `deep_causality`, examples, and tests — the authoritative migration checklist

## 2. Add the control functor

- [ ] 2.1 Create `deep_causality_core/src/types/causal_command/mod.rs`: `CausalCommand<T> = { RelayTo(usize, Box<PropagatingEffect<T>>), Dispatch(HashMap<IdentificationValue, Box<PropagatingEffect<T>>>) }` (`Dispatch` = renamed `Map`, `#[cfg(feature = "std")]` on the `HashMap`)
- [ ] 2.2 `causal_command/hkt.rs`: total `Functor` `fmap` (identity + composition, both variants, no panic)
- [ ] 2.3 `causal_command/partial_eq.rs`: congruent equality — `RelayTo` compares target + payload recursively, `Dispatch` compares the map structurally (derive if 1.1 allows)
- [ ] 2.4 `causal_command/display.rs` + `predicates.rs` (`is_relay_to` / `is_dispatch`); export `CausalCommand` from `lib.rs`

## 3. Make EffectValue the clean value functor

- [ ] 3.1 Remove `RelayTo` and `Map` from `EffectValue<T>` (`effect_value/mod.rs`) — leaves `{ None, Value, ContextualLink }`
- [ ] 3.2 Replace hand-written `partial_eq.rs` with `#[derive(PartialEq)]`; delete the `Map`/`RelayTo` carve-outs
- [ ] 3.3 Make `EffectValue` `fmap` total; fix `into_value` to the honest `Maybe` projection; update `display.rs`, `predicates.rs`, `from.rs`
- [ ] 3.4 Remove the arity-5 `fmap` `.expect` panic (`causal_effect_propagation_process/hkt.rs`) — now total over the clean functor

## 4. Widen the carrier outcome

- [ ] 4.1 Change the outcome field to the 3-way sum `Value(EffectValue<V>) | Error(E) | Control(CausalCommand<V>)` (`causal_effect_propagation_process/mod.rs`)
- [ ] 4.2 Update every constructor to be total over the 3-way outcome; add a control constructor
- [ ] 4.3 Add `control() -> Option<&CausalCommand<Value>>`; update `effect()` to discriminate `{ None, Value, ContextualLink }` only; update `value()`/`value_cloned()`/`into_value()`/`error()`/predicates (`getters.rs`)
- [ ] 4.4 Update `bind`/`fmap`/`and_then` match arms for the `Control` arm (propagate control carriers lawfully, like `None`/`ContextualLink`)

## 5. Migrate consumers (deep_causality)

- [ ] 5.1 `graph_reasoning/mod.rs`: read `control()` / `CausalCommand::RelayTo` instead of `EffectValue::RelayTo`
- [ ] 5.2 `graph_reasoning/stateful.rs`: same migration on the stateful reasoning path
- [ ] 5.3 `csm/eval.rs`: treat `Control` carriers as before (inactive/dispatch per current semantics)
- [ ] 5.4 `causable_stateful.rs`: pass-through of control carriers (was `RelayTo`/`Map` pass-through)
- [ ] 5.5 Migrate the ~7 affected test files to construct/match the `Control` arm

## 6. Verify

- [ ] 6.1 `grep` clean: no `EffectValue::RelayTo` / `EffectValue::Map` remain anywhere (src, examples, tests)
- [ ] 6.2 `bazel test //...` and `cargo test -p deep_causality_core -p deep_causality` green
- [ ] 6.3 Behavior preserved: graph-reasoning jump-to-target and missing-target-error tests pass unchanged
- [ ] 6.4 `make format && make fix` clean (fix clippy lints, do not suppress)
- [ ] 6.5 Update the deviation ledger (`core-formalization-plan.md`): D5/D6/D14/D15 → **Fixed**; note the two safety bugs closed. Prepare a commit message per changed crate; ask before committing
