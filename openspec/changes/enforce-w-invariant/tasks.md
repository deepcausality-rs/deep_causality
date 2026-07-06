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

- [ ] 5.1 Extend `lean/DeepCausalityFormal/Core/CausalMonad.lean`: add the error channel to the model (`Except`-shaped, transcribing the new `bind`), keep `bind_left_id`, prove `bind_right_id` and `bind_assoc`; typecheck standalone with bare `lean`
- [ ] 5.2 Update `deep_causality_core/tests/kani_proofs.rs`: right-identity-on-errored-input, continuation-does-not-run, log-monotonicity harnesses; note the W-harness discharged by construction; run `cargo kani --tests -p deep_causality_core` (Kani nightly ≤ workspace MSRV — see memory note)
- [ ] 5.3 Add Rust witness tests tagged `THEOREM_MAP: core.causal_monad.right_id` and `core.causal_monad.assoc`; move both ids out of the "blocked" section of `lean/THEOREM_MAP.md` (`lawful` stays blocked on P1); verify the CI consistency gate passes locally
- [ ] 5.4 Update `Formalization.md` work-plan item 1 to done and the P2 references in `openspec/notes/causal-algebra/haft-formalization-deviations.md` (D6 note) and `deep_causality_haft/LEAN_HAFT.md` if they mention P2 as pending

## 6. Documentation and closure

- [ ] 6.1 Update carrier/trait docstrings (five-channel description, W-by-construction, unconditional laws), `deep_causality_core/README.md` / `Notes.md` if they show field construction, and both crates' CHANGELOGs with the migration patterns (BREAKING)
- [ ] 6.2 Final verification: workspace `make build && make test`, all Lean files typecheck, CI theorem-map gate exits 0, examples run; prepare the final commit message
