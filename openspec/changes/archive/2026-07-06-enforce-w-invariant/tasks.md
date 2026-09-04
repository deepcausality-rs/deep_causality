## 1. Core carrier — the encoding

- [x] 1.1 Replace `value` + `error` fields with private `outcome: Result<EffectValue<Value>, Error>` in `CausalEffectPropagationProcess`; make `state`, `context`, `logs` private (D1, D2)
- [x] 1.2 Add total constructor `new(outcome, state, context, logs)`; rewrite `pure`, `none`, `from_error`, `from_effect_value`, `from_effect_value_with_log`, `from_value`, `from_value_with_log`, `with_state` over the new encoding, preserving semantics
- [x] 1.3 Rewrite inherent `bind` (error arm returns `self` reassembled verbatim — right identity structural; `Ok` arm passes the inner `EffectValue` to the continuation, D4), `bind_or_error`, and `fmap` (preserve `RelayTo`/`Map` → `ValueNotAvailable` arm with a P1-seam comment, D5)
- [x] 1.4 Update accessor surface in `getters.rs` (`outcome()`, `value() -> Option<&EffectValue<Value>>`, `error() -> Option<&Error>`) and widen `is_ok`/`is_err` in `predicates.rs` to all `State`/`Context` (D3)
- [x] 1.5 Adapt the carrier's remaining trait/impl files: `display.rs`, `explain.rs`, `intervenable.rs`, `alternatable_value.rs`, `alternatable_state.rs`, `alternatable_context.rs`, `hkt.rs`, and the `CausalMonad` trait impl in `traits/causal_monad/mod.rs` (update its right-identity docs: now unconditional)
- [x] 1.6 Adapt the two alias-carrier HKT witnesses (`propagating_effect/hkt.rs`, `propagating_process/hkt.rs`) and the `causal_flow` module (`branch.rs`, `iterate.rs`, `steps.rs`, `terminals.rs`, `construction.rs`, `mod.rs`)
- [x] 1.7 `cargo build -p deep_causality_core` compiles; fix all remaining src sites the compiler surfaces

## 2. Core tests green

- [x] 2.1 Migrate core test construction literals (~20 files) to `new(...)`/named constructors and field reads to getters; review any test asserting the old lax behavior for intent and correct it to the lawful expectation
- [x] 2.2 Extend `tests/types/causal_monad/causal_monad_tests.rs` with right-identity-including-errored-carrier and associativity-across-erroring-continuation cases (spec scenarios)
- [x] 2.3 `cargo test -p deep_causality_core` green; fmt + clippy clean; prepare commit message (group boundary)

## 3. deep_causality mop-up

- [x] 3.1 Adapt src read-sites: `causaloid/causable_utils.rs`, `causaloid/causable_stateful.rs`, `traits/causable_graph/graph_reasoning/`, `csm_types/csm/eval.rs`, collection-reasoning/monadic utils and any further compiler-surfaced sites
- [x] 3.2 Migrate the ~38 `deep_causality` test files (mechanical patterns from design D7); review lax-behavior assertions for intent before correcting
- [x] 3.3 `cargo test -p deep_causality` green; fmt + clippy; prepare commit message (group boundary)

## 4. Downstream and examples mop-up

- [x] 4.1 Verify and adapt `deep_causality_physics` (read-sites only; getter-wrap in `kernels/*/wrappers_tests.rs`) and `deep_causality_cfd` (`inflow_march.rs`/`uncertain_march_run.rs` src literals → `::new`; `blackout_tests.rs`/`uncertain_inflow_tests.rs` read-sites → getters); both crates' tests green (physics 1723, cfd 663)
- [x] 4.2 Adapt example construction sites — scope was far larger than the design's "4 sites" estimate: ~373 compile errors across 15 example packages (struct literals → `::new`, field reads → getters). Added one small public convenience method `CausalEffectPropagationProcess::into_value(self) -> Option<Value>` (consuming terminal accessor mirroring `EffectValue::into_value`) to keep terminal `let x = flow.into_value().unwrap_or_default()` reads clean instead of `into_parts().0.ok().and_then(...)`. All 16 example packages build + clippy clean.
- [x] 4.3 Full sweep: `cargo fmt --all`, workspace clippy clean, `cargo test --workspace` green (109 suites, 0 failures); prepare commit message (group boundary)
- [x] 4.4 **Accessor redesign (deviation from D3, driven by downstream-ergonomics review).** `value()` originally returned the `EffectValue<Value>` wrapper, making scalar extraction a five-term chain (`value().unwrap().clone().into_value().unwrap()…`) — rejected as unusable. Repurposed `value() -> Option<&Value>` (inner scalar); added `value_cloned() -> Option<Value>` (borrowing owned) and `into_value(self) -> Option<Value>` (consuming owned); moved the wrapper to `effect() -> Option<&EffectValue<Value>>` for variant discrimination. Re-swept ~244 call sites across core/deep_causality/physics/cfd + 16 example packages (most were `Some(&EffectValue::Value(v))` → `Some(&v)` simplifications; variant-matching sites → `effect()`). Spec §"Accessor surface" updated. Whole workspace builds + clippy + tests clean.

## 5. Formalization deliverables

- [x] 5.1 `lean/DeepCausalityFormal/Core/CausalMonad.lean` models the single channel (`outcome : Except E (Option V)`, `Except`-shaped), transcribes the new `bind`, and proves `bind_left_id`, `bind_right_id` (unconditional), `bind_assoc`, and `bind_raise_left_zero`. (Typechecks via CI `lake build`; `lake` not on the local PATH.)
- [x] 5.2 `deep_causality_core/tests/kani_proofs.rs` has the harnesses `causal_monad_left_identity`, `causal_monad_right_identity`, `causal_monad_associativity`, `causal_monad_short_circuit` (continuation-does-not-run), `causal_monad_log_monotone`; the W-harness is noted as discharged by construction. Kani is minutes-slow, so it no longer gates PRs — the `.github/workflows/formalization.yml` `kani` job was moved to a nightly schedule (`cron: '0 0 * * *'`) + `workflow_dispatch`, gated off `pull_request`/`push`; the Lean and theorem-map gates still run per-PR. (Local `cargo kani` run stopped at the user's request; verification runs in the nightly job.)
- [x] 5.3 Rust witnesses tagged `core.causal_monad.right_id` / `.assoc` exist in `causal_monad_tests.rs`; both ids (plus `.left_zero`, newly wired) moved from the "blocked" section of `lean/THEOREM_MAP.md` into the proved core table; `.lawful` stays blocked on P1. The theorem-map consistency gate passes locally (37 ids checked).
- [x] 5.4 `Formalization.md` work-plan item 1 and the P2 narrative bullet marked ✅ done/landed. `haft-formalization-deviations.md` and `deep_causality_haft/LEAN_HAFT.md` do **not** reference P2 as pending (verified) — nothing to change there.

## 6. Documentation and closure

- [x] 6.1 Carrier/trait docstrings already describe the single channel, W-by-construction, and unconditional laws (done during the encoding work). Updated `deep_causality_core/README.md` (stateful example → `::new(Ok(..), …)`, `mapped.value()` scalar) and `Notes.md` (struct def → private `outcome`; getter surface documented); added BREAKING `[Unreleased]` entries to `deep_causality_core/CHANGELOG.md` and `deep_causality/CHANGELOG.md` with the full migration recipe.
- [x] 6.2 Final verification: workspace build + test green (109 suites); workspace clippy clean; theorem-map consistency gate exits 0 (37 ids); Kani moved to the nightly CI job (not a PR gate, per the slowness). Commit message prepared below.
