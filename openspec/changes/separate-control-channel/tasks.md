## 1. Preflight

- [x] 1.1 Verify `Map` is dead — constructed only in `effect_value_tests.rs` (4 sites) + `is_map`/`display`/`partial_eq` arms; no production/example/reasoning use. **Confirmed: remove, do not rename.**
- [ ] 1.2 Full `grep` inventory of `EffectValue::RelayTo` / `EffectValue::Map` across core, `deep_causality`, examples, tests — the authoritative migration checklist
- [ ] 1.3 Confirm `deep_causality_haft::Free` / `FreeWitness` are exported and `alloc`-gated as needed (they are — see the free-monad tests' Bazel `crate_features`)

## 2. Remove the dead `Map` operation

- [ ] 2.1 Remove `Map` from `EffectValue` (`effect_value/mod.rs`), `is_map` (`predicates.rs`), and the `Map` arms in `display.rs` / `partial_eq.rs`
- [ ] 2.2 Delete the 4 `EffectValue::Map` unit tests + the `is_map` test in `effect_value_tests.rs`
- [ ] 2.3 Drop the now-unused `HashMap` / `IdentificationValue` imports in `effect_value/mod.rs`

## 3. Add the control operation functor + wire `haft::Free`

- [ ] 3.1 Create `deep_causality_core/src/types/causal_command/mod.rs`: `CausalCommand<K> { RelayTo(usize, K) }` (single-hole operation functor), `alloc`-gated
- [ ] 3.2 `causal_command/hkt.rs`: `CausalCommandWitness: HKT<Constraint = NoConstraint>` (`Type<K> = CausalCommand<K>`) + `Functor` (`fmap` maps the `RelayTo` hole; total, identity + composition)
- [ ] 3.3 `causal_command/partial_eq.rs` + `display.rs`: derive `PartialEq` when `K: PartialEq` (structural congruence); export `CausalCommand` / `CausalCommandWitness` from `lib.rs`
- [ ] 3.4 Define the control program alias `Free<CausalCommandWitness, EffectValue<V>>` and confirm `Free::{pure, lift, fold}` cover the constructions the reasoning engine needs

## 4. Make EffectValue the clean value functor

- [ ] 4.1 Remove `RelayTo` from `EffectValue<T>` (`effect_value/mod.rs`) — leaves `{ None, Value, ContextualLink }`
- [ ] 4.2 Replace hand-written `partial_eq.rs` with `#[derive(PartialEq)]`; delete the carve-outs
- [ ] 4.3 Make `EffectValue` `fmap` total; fix `into_value` to the honest `Maybe` projection; update `from.rs`
- [ ] 4.4 Remove the arity-5 `fmap` `.expect` panic (`causal_effect_propagation_process/hkt.rs`)

## 5. Widen the carrier outcome to the 3-way sum

- [ ] 5.1 Change the outcome to `Value(EffectValue<V>) | Error(E) | Control(Free<CausalCommandWitness, EffectValue<V>>)` (`causal_effect_propagation_process/mod.rs`)
- [ ] 5.2 Update every constructor to be total over the 3-way outcome; add a control constructor
- [ ] 5.3 Add `control() -> Option<&Free<CausalCommandWitness, EffectValue<Value>>>`; update `effect()` to discriminate `{ None, Value, ContextualLink }` only; update `value()`/`value_cloned()`/`into_value()`/`error()`/predicates (`getters.rs`)
- [ ] 5.4 Update `bind`/`fmap`/`and_then` match arms for the `Control` arm (propagate control programs lawfully, like `None`/`ContextualLink`)

## 6. Rewrite the reasoning handler as `Free::fold`

- [ ] 6.1 `graph_reasoning/mod.rs`: interpret the `Control` program via `Free::fold` — `algebra` resolves `RelayTo(target, sub)` (reset traversal, jump, feed `sub`, combine logs), `pure_case` emits the leaf effect. Preserve missing-target error + stop-on-control
- [ ] 6.2 `graph_reasoning/stateful.rs`: same fold-based interpretation on the stateful path
- [ ] 6.3 `csm/eval.rs` + `causable_stateful.rs`: read the `Control` arm; pass-through/inactive per current semantics
- [ ] 6.4 Migrate the affected reasoning + effect-value test files to construct/match the control program

## 7. Formalize the functor precondition

- [ ] 7.1 Lean: prove `CausalCommandWitness` functor laws (identity + composition on the single hole) — the precondition `haft::Free` needs; cite `haft.free_monad.*` for the resulting monad laws. Bare-`lean` typecheck
- [ ] 7.2 Rust witness + `THEOREM_MAP` row (`core.causal_command.functor_laws`); wire into `DeepCausalityFormal.lean`

## 8. Verify

- [ ] 8.1 `grep` clean: no `EffectValue::RelayTo` / `EffectValue::Map` remain anywhere
- [ ] 8.2 `bazel test //...` and `cargo test -p deep_causality_core -p deep_causality` green
- [ ] 8.3 Behavior preserved: graph-reasoning jump-to-target and missing-target-error tests pass unchanged
- [ ] 8.4 `make format && make fix` clean (fix clippy lints, do not suppress)
- [ ] 8.5 Update the deviation ledger (`core-formalization-plan.md`): D5/D6/D14/D15 → **Fixed**; note the two safety bugs closed and `Map` removed. Prepare a commit message per changed crate; ask before committing
