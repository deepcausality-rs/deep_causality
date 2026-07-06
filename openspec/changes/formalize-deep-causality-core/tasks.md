## 1. Prerequisites (separate changes — must land before groups 4+)

- [ ] 1.1 Confirm `separate-control-channel` (control Option B) is merged: `RelayTo`/`Map` moved out of `EffectValue` into `CausalCommand`; the arity-5 `fmap` panic and non-reflexive `Map` equality gone
- [ ] 1.2 Confirm `causal-arrow-state-threading` (arrow Option B) is merged: the arrow stage threads `(value, state, context)`; `and_then` preserves `None` lawfully
- [ ] 1.3 Verify the corrected code is green (`bazel test //...`, `cargo test -p deep_causality_core`) before writing any B-dependent proof

## 2. Witness mirror scaffolding

- [ ] 2.1 Create `deep_causality_core/tests/formalization_lean/mod.rs` with the module preamble (mirroring the haft `formalization_lean/mod.rs` docstring + cfg-gated module list)
- [ ] 2.2 Register the mirror in `deep_causality_core/tests/BUILD.bazel` (new `rust_test_suite` or existing suite entry; list `crate_features` explicitly — Bazel does not resolve Cargo feature transitivity)
- [ ] 2.3 Wire the module into the crate's test tree so `cargo test -p deep_causality_core` discovers it

## 3. Prerequisite-independent slice (no dependency on Option B)

- [ ] 3.1 `EffectLog.lean`: remove the "staged" qualifier from the four `THEOREM_MAP` tags; verify it typechecks with bare `lean`
- [ ] 3.2 Add `effect_log_tests.rs` witnesses for `core.effect_log.{left_id,right_id,assoc,monotone}` (one `#[test]` per id) against the real `EffectLog`/`LogAppend`
- [ ] 3.3 Add the four `core.effect_log.*` rows to `lean/THEOREM_MAP.md` (drop "staged"); register the file in `lean/DeepCausalityFormal.lean`
- [ ] 3.4 `CausalMonad.lean`: reframe the docstring to cite `haft.monad.laws` as the base (delta-only framing); confirm the 5 existing ids still typecheck and are witnessed. Leave `lawful` for group 5.5

## 4. Clean value functor + control channel (post `separate-control-channel`)

- [ ] 4.1 `EffectValue.lean` (new): total `fmap_id`/`fmap_comp` over `{None, Value, ContextualLink}`; `into_from_roundtrip`; `maybe_section` (`≅ Option`). Bare-`lean` check
- [ ] 4.2 `effect_value_tests.rs`: witnesses for `core.effect_value.{fmap_id,fmap_comp,into_from_roundtrip,maybe_section}`; `THEOREM_MAP` rows; add to `DeepCausalityFormal.lean`
- [ ] 4.3 `CausalCommand.lean` (new): `CausalCommand` functor laws + the free monad over it, citing `haft.free_monad.*`; equality by `fold`-canonicalization; prove over a representative functor (positivity). Bare-`lean` check
- [ ] 4.4 `causal_command_tests.rs`: witnesses for `core.causal_command.*` via `fold`-canonicalization; `THEOREM_MAP` rows; add to `DeepCausalityFormal.lean`
- [ ] 4.5 `Consistency.lean` (new): `core.witness.agree` — witness `fmap` = inherent `fmap` on every carrier, no reachable panic. Bare-`lean` check
- [ ] 4.6 `consistency_tests.rs`: witness for `core.witness.agree`; `THEOREM_MAP` row; add to `DeepCausalityFormal.lean`

## 5. Causal monad lawful + causal arrow (post both prerequisites)

- [ ] 5.1 `CausalMonad.lean`: add `core.causal_monad.lawful` (now unblocked); bare-`lean` check
- [ ] 5.2 Witness `core.causal_monad.lawful`; flip its `THEOREM_MAP` row from "blocked on P1" to `proved`
- [ ] 5.3 `CausalArrow.lean` (new): Kleisli `category_laws` + `left_zero` threading state/context; unconditional right identity (no `None → Err`, no `S,C`-erasure caveat). Bare-`lean` check
- [ ] 5.4 `causal_arrow_tests.rs`: witnesses for `core.causal_arrow.{category_laws,left_zero}`, including the `f >>> arr id = f` case for a `None`-emitting stage; `THEOREM_MAP` rows; add to `DeepCausalityFormal.lean`
- [ ] 5.5 Add a Kani harness for the arrow category laws (bounded, beyond point-witness) per design open question; record in the `THEOREM_MAP` `Kani` column

## 6. Lens family + flow facade + IO codec

- [ ] 6.1 `Alternatable.lean` (new): `set_get`, `set_set_proj`, `channel_independence`, `error_noop` under the `proj` eraser; the up-to-log negative lemma (D9); `clear_context` `None`-set + no-op-on-error. Docstring points the do-operator forward to the hypergraph layer (D8). Bare-`lean` check
- [ ] 6.2 `alternatable_tests.rs`: witnesses for `core.alternatable.*`; `THEOREM_MAP` rows; add to `DeepCausalityFormal.lean`
- [ ] 6.3 `CausalFlow.lean` (new): `flow_iso`, `map_id`/`map_comp`, `map_eq_andThen` on the full effect value (D14 fixed); `recover` (catch law), iterate combinators (`MaxStepsExceeded` contract), `finish` (terminal projection, log-drop note) as documented extensions. Bare-`lean` check
- [ ] 6.4 `causal_flow_tests.rs`: witnesses for `core.causal_flow.*`; `THEOREM_MAP` rows; add to `DeepCausalityFormal.lean`
- [ ] 6.5 `Csv.lean` (new): `core.io.csv_roundtrip` under the comma/newline-free hypothesis, citing `haft.io.laws`. Bare-`lean` check
- [ ] 6.6 `csv_tests.rs`: witness for `core.io.csv_roundtrip`; `THEOREM_MAP` row; add to `DeepCausalityFormal.lean`

## 7. Documentation + finalization

- [ ] 7.1 Finalize the deviation ledger: graduate `core-formalization-plan.md` → `core-formalization-deviations.md` (mirror `haft-formalization-deviations.md`); give every D1–D17 a terminal disposition; settle the two soft-flagged items (D10, D16)
- [ ] 7.2 Write `deep_causality_core/LEAN_CORE.md` mirroring `LEAN_HAFT.md`: summary, `how to check` commands, per-mechanism status table with references, pointer to the audit
- [ ] 7.3 Verify counts match: every `core.*` id in `THEOREM_MAP.md` has a closed Lean theorem and a passing witness; `LEAN_CORE.md` counts agree

## 8. Full verification gate

- [ ] 8.1 `lake build` (all Core Lean files typecheck as a project) and bare `lean <file>` per Core file (standalone)
- [ ] 8.2 `cargo test -p deep_causality_core` and `bazel test //...` green (witnesses + Kani harnesses)
- [ ] 8.3 Run the CI consistency gate logic locally: no `core.*` id missing a Lean side, a witness, or a `THEOREM_MAP` row
- [ ] 8.4 `make format && make fix` clean (fix clippy lints, do not suppress); prepare a commit message per changed crate/area and ask before committing
