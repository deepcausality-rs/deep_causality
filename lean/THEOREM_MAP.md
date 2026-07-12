<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# THEOREM_MAP — Lean ↔ Rust traceability

This is **the bridge**. There is no tool that converts a Lean proof into a Rust test
(`openspec/notes/causal-algebra/Formalization.md` §3). Instead, each **property statement** is
transcribed once per layer and linked here:

- **Lean** *proves* the statement (deductive, unbounded, higher-order).
- **Rust witness** *checks* the same statement independently:
  - `num` / `haft`: a law-test and/or the trait contract itself (the house style).
  - `core`: a **Kani** harness (bounded model checking — first-order, fixed continuations).

The `THEOREM_MAP:` tag in each Lean file and the matching comment in each Rust witness carry the
same **id**. CI (`.github/workflows/formalization.yml`) fails if an id lacks either side.

## Legend

- **Lean**: `proved` = closed, no `sorry`; `sorry` = stated but unproved; `—` = not yet stated.
- **Kani** / **Test**: `✓` present & passing · `partial` · `—` not started · `n/a`.

## Map

| id | statement | Lean | Lean location | Rust witness | Test | Kani |
|---|---|---|---|---|---|---|
| `algebra.add_monoid.assoc` | `(a+b)+c = a+(b+c)` for `AddMonoid` | proved | `DeepCausalityFormal/Algebra/Monoid.lean :: add_monoid_assoc` | `deep_causality_algebra/tests/formalization_lean/monoid_tests.rs :: test_add_monoid_assoc` | ✓ | n/a |
| `algebra.add_monoid.identity` | `a+0 = a ∧ 0+a = a` for `AddMonoid` | proved | `DeepCausalityFormal/Algebra/Monoid.lean :: add_monoid_identity` | `deep_causality_algebra/tests/formalization_lean/monoid_tests.rs :: test_add_monoid_identity` | ✓ | n/a |
| `algebra.monoid.left_id` | `empty().combine(x) = x` for the generic `Monoid` (Mathlib `one_mul`) | proved | `DeepCausalityFormal/Algebra/MonoidGeneric.lean :: monoid_left_id` | `deep_causality_algebra/tests/formalization_lean/monoid_generic_tests.rs :: test_generic_monoid_laws` | ✓ | n/a |
| `algebra.monoid.right_id` | `x.combine(empty()) = x` (Mathlib `mul_one`) | proved | `DeepCausalityFormal/Algebra/MonoidGeneric.lean :: monoid_right_id` | `deep_causality_algebra/tests/formalization_lean/monoid_generic_tests.rs :: test_generic_monoid_laws` | ✓ | n/a |
| `algebra.monoid.assoc` | `combine` associativity (Mathlib `mul_assoc`) | proved | `DeepCausalityFormal/Algebra/MonoidGeneric.lean :: monoid_assoc` | `deep_causality_algebra/tests/formalization_lean/monoid_generic_tests.rs :: test_generic_monoid_laws` | ✓ | n/a |
| `algebra.commutative_monoid.comm` | `x.combine(y) = y.combine(x)` for `CommutativeMonoid` (Mathlib `mul_comm`) | proved | `DeepCausalityFormal/Algebra/CommutativeMonoid.lean :: commutative_monoid_comm` | `deep_causality_algebra/tests/formalization_lean/commutative_monoid_tests.rs :: test_commutative_monoid_comm` | ✓ | n/a |
| `algebra.semilattice.idempotent` | `x.combine(x) = x` for the boolean ∧-semilattice (`Conjunction`) | proved | `DeepCausalityFormal/Algebra/CommutativeMonoid.lean :: semilattice_idempotent` | `deep_causality_algebra/tests/formalization_lean/commutative_monoid_tests.rs :: test_semilattice_idempotent` | ✓ | n/a |
| `algebra.semilattice.assoc` | associativity of the boolean ∧-semilattice | proved | `DeepCausalityFormal/Algebra/CommutativeMonoid.lean :: semilattice_assoc` | `deep_causality_algebra/tests/formalization_lean/commutative_monoid_tests.rs :: test_semilattice_assoc_and_comm` | ✓ | n/a |
| `algebra.semilattice.comm` | commutativity of the boolean ∧-semilattice | proved | `DeepCausalityFormal/Algebra/CommutativeMonoid.lean :: semilattice_comm` | `deep_causality_algebra/tests/formalization_lean/commutative_monoid_tests.rs :: test_semilattice_assoc_and_comm` | ✓ | n/a |
| `algebra.verdict.lattice_laws` | boolean verdict lattice: meet commutativity + absorption | proved | `DeepCausalityFormal/Algebra/Verdict.lean :: verdict_meet_comm / verdict_absorption` | `deep_causality_algebra/tests/formalization_lean/verdict_tests.rs :: test_verdict_lattice_laws` | ✓ | n/a |
| `algebra.verdict.complement` | complement involution + De Morgan (Boolean); MV-algebra complement `1−p` (Prob) | proved | `DeepCausalityFormal/Algebra/Verdict.lean :: verdict_compl_compl / verdict_de_morgan` | `deep_causality_algebra/tests/formalization_lean/verdict_tests.rs :: test_verdict_complement` | ✓ | n/a |
| `core.causal_monad.left_id` | `pure a >>= f = f a` | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_left_id` | `deep_causality_core/tests/kani_proofs.rs :: causal_monad_left_identity` | n/a | ✓ |
| `core.causal_monad.right_id` | `m >>= pure = m` (unconditional — holds on errored carriers) | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_right_id` | `deep_causality_core/tests/types/causal_monad/causal_monad_tests.rs :: test_right_identity_unconditional` | ✓ | ✓ |
| `core.causal_monad.assoc` | `(m >>= f) >>= g = m >>= (λx. f x >>= g)` | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_assoc` | `deep_causality_core/tests/types/causal_monad/causal_monad_tests.rs :: test_associativity_across_erroring_continuation` | ✓ | ✓ |
| `core.causal_monad.left_zero` | `raise e >>= f = raise e` (error short-circuit) | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_raise_left_zero` | `deep_causality_core/tests/kani_proofs.rs :: causal_monad_short_circuit` | n/a | ✓ |
| `core.causal_monad.lawful` | `LawfulMonad`-with-effect: left/right identity + associativity co-hold on one carrier (P1 resolved) | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: causal_monad_lawful` | `deep_causality_core/tests/formalization_lean/causal_monad_tests.rs :: test_causal_monad_lawful` | ✓ | — |
| `core.causal_arrow.category_laws` | Kleisli category laws (left/right identity, associativity) threading state/context over arbitrary `S`, `C` | proved | `DeepCausalityFormal/Core/CausalArrow.lean :: kcomp_left_id / kcomp_right_id / kcomp_assoc` | `deep_causality_core/tests/types/causal_arrow/causal_arrow_tests.rs :: arrow_threads_accumulated_state` | ✓ | ✓ |
| `core.causal_arrow.left_zero` | errored stage short-circuits composition; state preserved, downstream not run | proved | `DeepCausalityFormal/Core/CausalArrow.lean :: kcomp_left_zero` | `deep_causality_core/tests/types/causal_arrow/causal_arrow_tests.rs :: arrow_error_short_circuit_preserves_state` | ✓ | — |
| `core.effect_log.left_id` | `append empty x = x` (free monoid / Writer output) | proved | `DeepCausalityFormal/Core/EffectLog.lean :: append_left_id` | `deep_causality_core/tests/formalization_lean/effect_log_tests.rs :: test_effect_log_left_identity` | ✓ | — |
| `core.effect_log.right_id` | `append x empty = x` | proved | `DeepCausalityFormal/Core/EffectLog.lean :: append_right_id` | `deep_causality_core/tests/formalization_lean/effect_log_tests.rs :: test_effect_log_right_identity` | ✓ | — |
| `core.effect_log.assoc` | `append (append x y) z = append x (append y z)` | proved | `DeepCausalityFormal/Core/EffectLog.lean :: append_assoc` | `deep_causality_core/tests/formalization_lean/effect_log_tests.rs :: test_effect_log_associativity` | ✓ | — |
| `core.effect_log.monotone` | incoming log is a prefix of the combined log (append-only) | proved | `DeepCausalityFormal/Core/EffectLog.lean :: append_monotone` | `deep_causality_core/tests/formalization_lean/effect_log_tests.rs :: test_effect_log_monotone_prefix` | ✓ | — |
| `core.causal_effect.into_value` | `into_value` is the honest `Maybe` projection (`Pure(Some v)↦Some v`, `Pure(None)`/command`↦None`); value functor = `Option` (`haft.functor.laws`) | proved | `DeepCausalityFormal/Core/CausalEffect.lean :: into_value_value / into_value_none / into_value_command` | `deep_causality_core/tests/formalization_lean/causal_effect_tests.rs :: test_causal_effect_into_value` | ✓ | — |
| `core.causal_effect.transformer_stack` | the composite outcome `Except E (Free CausalCommand (Maybe V))` is a lawful monad: left/right identity, associativity, `Err` global left zero, `None` local zero, relay threading with error hoisting (Rust: `CausalEffect::try_and_then`/`and_then`) | proved | `DeepCausalityFormal/Core/CausalEffect.lean :: obind_left_id / obind_right_id / obind_assoc / obind_err_zero / obind_none_zero` | `deep_causality_core/tests/formalization_lean/causal_effect_tests.rs :: test_transformer_stack_monad_laws / test_transformer_stack_zeros_and_relay_threading` | ✓ | — |
| `core.causal_effect.fold_universal` | `CausalEffect::fold` satisfies the two handler equations and is the UNIQUE such interpreter (initiality of the free monad on `CausalCommand`) | proved | `DeepCausalityFormal/Core/CausalEffect.lean :: fold_pure / fold_relay / fold_unique` | `deep_causality_core/tests/formalization_lean/causal_effect_tests.rs :: test_fold_universal` | ✓ | — |
| `core.causal_effect.relay_termination` | the fuel-bounded relay handler is total: a value answers, answers are fuel-monotone, and a self-relay cycle exhausts (reports) instead of looping — the engine bound is `MAX_RELAY_ROUNDS` (closes tracker #2 Q3's termination item) | proved | `DeepCausalityFormal/Core/CausalEffect.lean :: run_pure / run_fuel_monotone / run_self_relay_none` | `deep_causality_core/tests/formalization_lean/causal_effect_tests.rs :: test_relay_termination_fuel_bound` (engine: `deep_causality/tests/…/causality_graph_reasoning_sub_tests.rs :: test_evaluate_subgraph_cuts_a_relay_cycle_with_the_fuel_bound`, stateful twin) | ✓ | — |
| `core.causal_effect.relay_round_composition` | multi-round adaptive evaluation is the sequential (Kleisli) composition of its rounds (graph-reasoning-formalization, task 3.6): the round step iterates additively (`rounds_add` — round `m` seeds round `m+1`), a reached answer is stable under further rounds (`run_monotone_add`), and the fuel-bounded run splits at any round boundary (`run_rounds_compose`; `run_relay_peel` the two-round step); the fuel bound composes with no new termination argument, inheriting `core.causal_effect.relay_termination` (engine `'rounds` loop, `MAX_RELAY_ROUNDS`) | proved | `DeepCausalityFormal/Core/CausalEffect.lean :: rounds_add / run_rounds_compose / run_monotone_add / run_relay_peel` | `deep_causality/tests/formalization_lean/relay_round_composition_tests.rs :: test_relay_round_composition / test_relay_round_fuel_bound_composes` | ✓ | — |
| `core.causaloid.fixpoint` | `Causaloid ≅ μX.F(X)` with `F(X) = Atom + Coll(Bag X, AggLogic) + Graph(Hyper X, Λ-edges)`: the `roll`/`unroll` Lambek isomorphism, the three summands ↔ the three sealed `CausaloidType` forms (#11a), well-founded (μ, not ν — `size` total; closes tracker #9) | proved | `DeepCausalityFormal/Core/Causaloid.lean :: roll_unroll / unroll_roll / size_pos` | `deep_causality/tests/formalization_lean/causaloid_tests.rs :: test_fixpoint_three_forms_and_roll_unroll / test_undecorated_graph_evaluates_identically` | ✓ | — |
| `core.causaloid.inversion` | the Hardy inversion, formal: `eval = wiring ∘ element-map`, the element map is pointwise and bag-symmetric (`mapL_perm`) — no ordering asymmetry enters through the element; Λ-edges are identity-keyed connection data (Rust: `LambdaEdges`, enumeration-order-free) | proved | `DeepCausalityFormal/Core/Causaloid.lean :: eval_factors / mapL_perm` | `deep_causality/tests/formalization_lean/causaloid_tests.rs :: test_inversion_element_is_symmetric / test_lambda_edges_identity_keyed_and_order_free` | ✓ | — |
| `core.verdict.closure` | `All`/`Any`/`None`/`Some(k)` are closed operations in the Verdict algebra (`None` = `Any` ∘ `complement`; `Some(k)` = Count + boundary decision) ⇒ `Coll : Causaloid → Causaloid` (closes tracker #5); Rust: `Aggregatable: Verdict` carrier bound on `evaluate_collection` | proved | `DeepCausalityFormal/Core/VerdictClosure.lean :: closure_fold_step / none_is_any_complement / someK_decides / coll_closure` | `deep_causality/tests/formalization_lean/verdict_closure_tests.rs :: test_verdict_closure_aggregation_modes` | ✓ | — |
| `core.verdict.carriers` | the named carriers behind the one trait: `bool` Boolean (distributive, proved) and `Prob`/`f64` MV on `[0,1]` (min/max/1−p, excluded middle fails — Rust-witnessed), lifted pointwise to `UncertainBool`/`UncertainF64`; orthomodular projection lattice planned (quantum), general effects excluded (partial meet/join) | proved | `DeepCausalityFormal/Core/VerdictClosure.lean :: bool_carrier_characterization / bool_distributive` | `deep_causality/tests/formalization_lean/verdict_closure_tests.rs :: test_verdict_carriers` | ✓ | — |
| `core.verdict.perm_invariance` | collection aggregation is a BAG operation — the #1 scoped order-invariance theorem: for every `AggregateLogic` mode the aggregate VALUE is invariant under permutation of the member bag (`All`/`Any` commutative-associative meet-/join-folds, `None` from `Any`, `Some(k)` from the permutation-invariant firing count), lifted to the `Coll` node (`coll_perm`); scope = value channel, stateless, all-success path (the #1 ruling — the log channel and stateful path are excluded by statement) | proved | `DeepCausalityFormal/Core/VerdictClosure.lean :: aggregate_perm / coll_perm` | `deep_causality/tests/formalization_lean/verdict_closure_tests.rs :: test_verdict_perm_invariance` | ✓ | — |
| `core.causaloid.graph_fold_order_invariant` | the topological fold with `∇ ∘ (Λ₁ ⊗ Λ₂)` at reconvergent joins is invariant under every schedule consistent with the causal order: the ∇-fuse is a bag (`fuse_perm`), every consistent schedule computes the schedule-free denotation (`exec_computes_val`), two consistent schedules agree (`schedule_invariant`); preconditions checked at freeze (`freeze_verified`: acyclicity + single-writer + level hook) — closes tracker #2 Q1, engine `∇ = Verdict::join` (Rust behavior change: the loud-fail diamond is now the defined merge, corpus-gated) | proved | `DeepCausalityFormal/Core/GraphAlgebra.lean :: fuse_perm / exec_computes_val / schedule_invariant` | `deep_causality/tests/formalization_lean/graph_algebra_tests.rs :: test_graph_fold_order_invariant / test_two_writer_diamond_rejected_at_freeze` (corpus: `deep_causality/tests/traits/causable_graph/graph_reasoning/characterization_corpus_tests.rs`) | ✓ | — |
| `core.causaloid.catamorphism_unique` | initiality of the fixpoint, **per fixed carrier**: any interpreter satisfying the three case equations (atom/coll/graph, with the bag equations) of the semantic algebra `(V, elemSem, ∇)` is pointwise equal to `eval`; uniqueness across carriers is neither claimed nor true (#6 correctly scoped; goal B2) | proved | `DeepCausalityFormal/Core/Catamorphism.lean :: catamorphism_unique / catamorphism_unique_list` | `deep_causality/tests/formalization_lean/catamorphism_tests.rs :: test_catamorphism_unique` | ✓ | — |
| `core.causaloid.encapsulation_flat` | nested fold = flat fold (catamorphism fusion): a wrapped bag contributes exactly its members' fold (wrapper transparency, definitional) and folding a flattened bag equals folding with the back bag as continuation — wrapping a subgraph in a causaloid does not change the semantics | proved | `DeepCausalityFormal/Core/Catamorphism.lean :: encapsulation_flat / evalL_append` | `deep_causality/tests/formalization_lean/catamorphism_tests.rs :: test_encapsulation_flat` | ✓ | — |
| `core.causaloid.arrow_fragment` | the `Atom`/`compose` fragment ≅ the reified `ArrowTerm` language: interpreting the chain term = the causaloid's sequential wire evaluation, extended to the ⊕-enlarged set via `haft.arrow_term.choice_interpret_sound`; the interpretation factors through `T/≈` (terms related by the category laws interpret equally — closes tracker #8) | proved | `DeepCausalityFormal/Core/Catamorphism.lean :: arrow_fragment / interp_respects_category_laws` | `deep_causality/tests/formalization_lean/catamorphism_tests.rs :: test_arrow_fragment` | ✓ | — |
| `core.causaloid.command_input` | F-3: a command (`RelayTo`) on a singleton's INPUT channel yields a specific, named error — never a silent `None`, never a dropped signal, and distinct from the absence-of-evidence error; the input dispatch is total (`command_yields_cmd_err` / `command_never_ok` / `command_err_distinct_from_absent`); matches the real `Causaloid::evaluate` and `evaluate_stateful` singleton paths | proved | `DeepCausalityFormal/Core/CommandInput.lean :: command_yields_cmd_err / command_never_ok / command_err_distinct_from_absent` | `deep_causality/tests/formalization_lean/command_input_tests.rs :: test_command_input_yields_command_error / test_command_input_yields_command_error_stateful` | ✓ | — |
| `core.context_graph.threading_bind` | the Context hypergraph's parent-set (hyperedge) semantics keyed by identity (`Pa`, the `fired[child][parent]` / `LambdaEdges` `(source,target)` wire surface): hyperedge threading of a node's parents IS the causal monad `bind` (`thread_is_bind`), nested threading = flat (`thread_append`/`evalParents_split`), and encapsulation = flat is bind ASSOCIATIVITY inherited from `core.causal_monad.assoc` (`encapsulation_flat`) — the graph-side `core.causaloid.encapsulation_flat` | proved | `DeepCausalityFormal/Core/ContextGraph.lean :: thread_is_bind / thread_append / evalParents_split / encapsulation_flat` | `deep_causality/tests/formalization_lean/context_graph_tests.rs :: test_context_parent_set_keyed_by_identity / test_context_encapsulation_is_bind_assoc` | ✓ | — |
| `core.context_graph.acyclicity_separable` | acyclicity is a SEPARABLE, freeze-enforceable parameter over the same parent-set graph: `Acyclic Pa` = a rank certificate (`ultragraph::has_cycle` / `freeze_dag`); a self-parent (cycle) has no certificate and is rejected at freeze (`self_parent_not_acyclic`); the threading/encapsulation apparatus never consults acyclicity, so the cyclic case reuses the same definitions (`apparatus_acyclicity_agnostic`) — enabling the deferred cyclic case (quantum switch / indefinite causal order) | proved | `DeepCausalityFormal/Core/ContextGraph.lean :: Acyclic / self_parent_not_acyclic / acyclic_iff_rank / apparatus_acyclicity_agnostic` | `deep_causality/tests/formalization_lean/context_graph_tests.rs :: test_context_acyclicity_freeze_gate` | ✓ | — |
| `core.causal_command.functor_laws` | single-hole `CausalCommand` functor laws (`fmap id = id`, `fmap (g∘f) = fmap g ∘ fmap f`); free monad over it = `haft.free_monad.*` | proved | `DeepCausalityFormal/Core/CausalCommand.lean :: cmap_id / cmap_comp` | `deep_causality_core/tests/formalization_lean/causal_command_tests.rs :: test_causal_command_functor_laws` | ✓ | — |
| `core.witness.agree` | every law-bearing HKT surface computes the SAME total success-channel functor/applicative: `fmap` = inherent `fmap` on every carrier (`Some`/`None`/`Err`/**command**, command preserved not collapsed, no panic); `apply` total — value-less/command operand ↦ `None`, never `InternalLogicError` (D15 fully retired across all three witnesses) | proved | `DeepCausalityFormal/Core/Consistency.lean :: witness_agree / fmap_preserves_command / apply_none_yields_none / apply_command_yields_none` | `deep_causality_core/tests/formalization_lean/consistency_tests.rs :: test_witness_agrees_with_inherent_fmap / test_apply_none_operand_yields_none / test_apply_command_operand_yields_none` | ✓ | — |
| `core.alternatable.set_get` | lens set-get on the log-erasing projection (value/state/context) | proved | `DeepCausalityFormal/Core/Alternatable.lean :: set_get_value / set_get_state / set_get_context` | `deep_causality_core/tests/formalization_lean/alternatable_tests.rs :: test_alternatable_set_get` | ✓ | — |
| `core.alternatable.set_set_proj` | lens set-set idempotence up-to-log (`proj`); full carrier grows the log (D9) | proved | `DeepCausalityFormal/Core/Alternatable.lean :: set_set_value_proj / set_set_state_proj / set_set_context_proj / set_set_grows_log` | `deep_causality_core/tests/formalization_lean/alternatable_tests.rs :: test_alternatable_set_set_up_to_log` | ✓ | — |
| `core.alternatable.channel_independence` | each setter touches only its own channel | proved | `DeepCausalityFormal/Core/Alternatable.lean :: value_preserves_state_ctx / state_preserves_value_ctx / context_preserves_value_state` | `deep_causality_core/tests/formalization_lean/alternatable_tests.rs :: test_alternatable_channel_independence` | ✓ | — |
| `core.alternatable.error_noop` | every setter (and `clear_context`) is a no-op on an errored carrier | proved | `DeepCausalityFormal/Core/Alternatable.lean :: value_error_noop / state_error_noop / context_error_noop / clear_context_error_noop` | `deep_causality_core/tests/formalization_lean/alternatable_tests.rs :: test_alternatable_error_noop` | ✓ | — |
| `core.causal_flow.flow_iso` | `CausalFlow ≅ Process` (newtype wrap/unwrap, `rfl`) | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: flow_iso` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_iso` | ✓ | — |
| `core.causal_flow.map_id` | `map id = id` (facade functor identity) | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: map_id` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_map_id` | ✓ | — |
| `core.causal_flow.map_comp` | `map (g∘f) = map g ∘ map f` | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: map_comp` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_map_comp` | ✓ | — |
| `core.causal_flow.map_eq_andThen` | `map f = and_then (pure∘f)` — holds on the `None` effect as well as a value (D14) | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: map_eq_andThen` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_map_eq_and_then` | ✓ | — |
| `core.causal_flow.recover` | `MonadError.catch`: no-op on success, raise↦handler value | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: recover_catch` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_recover` | ✓ | — |
| `core.causal_flow.iterate` | bounded search terminates; budget exhaustion injects `MaxStepsExceeded`, state/context/log preserved | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: iterate_contract` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_iterate` | ✓ | — |
| `core.causal_flow.finish` | terminal value-observation drops state/context/log (depends only on the outcome) | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: finish_drops_state_ctx_log` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_finish` | ✓ | — |
| `core.io.csv_roundtrip` | `parse (render header rows) = header :: rows` under the no-`','`/no-`'\n'` precondition (cites `haft.io.laws`) | proved | `DeepCausalityFormal/Core/Csv.lean :: csv_roundtrip` | `deep_causality_core/tests/formalization_lean/csv_tests.rs :: test_csv_roundtrip` | ✓ | — |


### Num / Algebra / Complex / Dual layers (extracted numeric crates)

| id | statement | Lean | Lean location | Rust witness | Test | Kani |
|---|---|---|---|---|---|---|
| `num.one.identity` | `1*a = a ∧ a*1 = a` (the `One` identity) | proved | `DeepCausalityFormal/Num/Identity.lean :: one_is_identity` | `deep_causality_num/tests/formalization_lean/identity_tests.rs :: test_one_identity` | ✓ | n/a |
| `num.zero.identity` | `0+a = a ∧ a+0 = a` (the `Zero` identity) | proved | `DeepCausalityFormal/Num/Identity.lean :: zero_is_identity` | `deep_causality_num/tests/formalization_lean/identity_tests.rs :: test_zero_identity` | ✓ | n/a |
| `num.integer.mul_comm` | `a*b = b*a` over `ℤ` | proved | `DeepCausalityFormal/Num/Integer.lean :: integer_mul_comm` | `deep_causality_num/tests/formalization_lean/integer_tests.rs :: test_integer_mul_comm` | ✓ | n/a |
| `num.integer.distrib` | `a*(b+c) = a*b + a*c` over `ℤ` | proved | `DeepCausalityFormal/Num/Integer.lean :: integer_left_distrib` | `deep_causality_num/tests/formalization_lean/integer_tests.rs :: test_integer_distrib` | ✓ | n/a |
| `num.integer.euclidean` | `b*(a/b) + a%b = a` over `ℤ` (Euclidean division) | proved | `DeepCausalityFormal/Num/Integer.lean :: integer_euclidean` | `deep_causality_num/tests/formalization_lean/integer_tests.rs :: test_integer_euclidean` | ✓ | n/a |
| `num.cast.nat_int_roundtrip` | `((n:ℤ)).toNat = n` (ℕ↔ℤ cast round-trip) | proved | `DeepCausalityFormal/Num/Cast.lean :: cast_nat_int_roundtrip` | `deep_causality_num/tests/formalization_lean/cast_tests.rs :: test_cast_nat_int_roundtrip` | ✓ | n/a |
| `num.cast.int_injective` | `ℤ → ℚ` cast is injective | proved | `DeepCausalityFormal/Num/Cast.lean :: cast_int_injective` | `deep_causality_num/tests/formalization_lean/cast_tests.rs :: test_cast_int_injective` | ✓ | n/a |
| `num.float106.model.add_comm` | `a+b = b+a` (real-field model of the `Float106` double-double; bit-exact bounds are [open]) | proved | `DeepCausalityFormal/Num/Float106.lean :: float106_model_add_comm` | `deep_causality_num/tests/formalization_lean/float106_tests.rs :: test_float106_add_comm` | ✓ | n/a |
| `num.float106.model.mul_comm` | `a*b = b*a` (real-field model of `Float106`) | proved | `DeepCausalityFormal/Num/Float106.lean :: float106_model_mul_comm` | `deep_causality_num/tests/formalization_lean/float106_tests.rs :: test_float106_mul_comm` | ✓ | n/a |
| `num.float106.model.distrib` | `a*(b+c) = a*b + a*c` (real-field model of `Float106`) | proved | `DeepCausalityFormal/Num/Float106.lean :: float106_model_distrib` | `deep_causality_num/tests/formalization_lean/float106_tests.rs :: test_float106_distrib` | ✓ | n/a |
| `algebra.group.mul_inv` | `a * a⁻¹ = 1` for `Group` | proved | `DeepCausalityFormal/Algebra/Group.lean :: group_mul_inv` | `deep_causality_algebra/tests/formalization_lean/group_tests.rs :: test_group_mul_inv` | ✓ | n/a |
| `algebra.add_group.neg_cancel` | `-a + a = 0` for `AddGroup` | proved | `DeepCausalityFormal/Algebra/Group.lean :: add_group_neg_cancel` | `deep_causality_algebra/tests/formalization_lean/group_tests.rs :: test_add_group_neg_cancel` | ✓ | n/a |
| `algebra.abelian_group.add_comm` | `a + b = b + a` for `AbelianGroup` | proved | `DeepCausalityFormal/Algebra/Group.lean :: abelian_group_add_comm` | `deep_causality_algebra/tests/formalization_lean/group_tests.rs :: test_abelian_group_add_comm` | ✓ | n/a |
| `algebra.ring.left_distrib` | `a*(b+c) = a*b + a*c` for `Ring` | proved | `DeepCausalityFormal/Algebra/Ring.lean :: ring_left_distrib` | `deep_causality_algebra/tests/formalization_lean/ring_tests.rs :: test_ring_left_distrib` | ✓ | n/a |
| `algebra.ring.right_distrib` | `(a+b)*c = a*c + b*c` for `Ring` | proved | `DeepCausalityFormal/Algebra/Ring.lean :: ring_right_distrib` | `deep_causality_algebra/tests/formalization_lean/ring_tests.rs :: test_ring_right_distrib` | ✓ | n/a |
| `algebra.ring.mul_assoc` | `(a*b)*c = a*(b*c)` for `AssociativeRing` | proved | `DeepCausalityFormal/Algebra/Ring.lean :: ring_mul_assoc` | `deep_causality_algebra/tests/formalization_lean/ring_tests.rs :: test_ring_mul_assoc` | ✓ | n/a |
| `algebra.commutative_ring.mul_comm` | `a*b = b*a` for `CommutativeRing` | proved | `DeepCausalityFormal/Algebra/Ring.lean :: commutative_ring_mul_comm` | `deep_causality_algebra/tests/formalization_lean/ring_tests.rs :: test_commutative_ring_mul_comm` | ✓ | n/a |
| `algebra.field.mul_inv_cancel` | `a ≠ 0 → a * a⁻¹ = 1` for `Field` | proved | `DeepCausalityFormal/Algebra/Field.lean :: field_mul_inv_cancel` | `deep_causality_algebra/tests/formalization_lean/field_tests.rs :: test_field_mul_inv_cancel` | ✓ | n/a |
| `algebra.field.inv_mul_cancel` | `a ≠ 0 → a⁻¹ * a = 1` for `Field` | proved | `DeepCausalityFormal/Algebra/Field.lean :: field_inv_mul_cancel` | `deep_causality_algebra/tests/formalization_lean/field_tests.rs :: test_field_inv_mul_cancel` | ✓ | n/a |
| `algebra.real_field.mul_pos` | `0<a → 0<b → 0<a*b` for the ordered `RealField` | proved | `DeepCausalityFormal/Algebra/Field.lean :: real_field_mul_pos` | `deep_causality_algebra/tests/formalization_lean/field_tests.rs :: test_real_field_mul_pos` | ✓ | n/a |
| `algebra.module.smul_add` | `r•(x+y) = r•x + r•y` for `Module` | proved | `DeepCausalityFormal/Algebra/Module.lean :: module_smul_add` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_module_smul_add` | ✓ | n/a |
| `algebra.module.add_smul` | `(r+s)•x = r•x + s•x` for `Module` | proved | `DeepCausalityFormal/Algebra/Module.lean :: module_add_smul` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_module_add_smul` | ✓ | n/a |
| `algebra.module.one_smul` | `1•x = x` for `Module` | proved | `DeepCausalityFormal/Algebra/Module.lean :: module_one_smul` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_module_one_smul` | ✓ | n/a |
| `algebra.module.mul_smul` | `(r*s)•x = r•(s•x)` for `Module` | proved | `DeepCausalityFormal/Algebra/Module.lean :: module_mul_smul` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_module_mul_smul` | ✓ | n/a |
| `algebra.algebra.smul_mul_assoc` | `r•(a*b) = (r•a)*b` for `Algebra` over a ring | proved | `DeepCausalityFormal/Algebra/Module.lean :: algebra_smul_mul_assoc` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_algebra_smul_mul_assoc` | ✓ | n/a |
| `algebra.algebra.mul_smul_comm` | `r•(a*b) = a*(r•b)` for `Algebra` over a ring | proved | `DeepCausalityFormal/Algebra/Module.lean :: algebra_mul_smul_comm` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_algebra_mul_smul_comm` | ✓ | n/a |
| `algebra.division_algebra.mul_inv` | `a ≠ 0 → a * a⁻¹ = 1` for `DivisionAlgebra` | proved | `DeepCausalityFormal/Algebra/DivisionAlgebra.lean :: division_algebra_mul_inv` | `deep_causality_algebra/tests/formalization_lean/division_algebra_tests.rs :: test_division_algebra_mul_inv` | ✓ | n/a |
| `algebra.conjugate.star_star` | `star (star a) = a` for `ConjugateScalar` | proved | `DeepCausalityFormal/Algebra/Scalar.lean :: conjugate_star_star` | `deep_causality_algebra/tests/formalization_lean/scalar_tests.rs :: test_conjugate_star_star` | ✓ | n/a |
| `algebra.conjugate.star_mul` | `star (a*b) = star b * star a` for `ConjugateScalar` | proved | `DeepCausalityFormal/Algebra/Scalar.lean :: conjugate_star_mul` | `deep_causality_algebra/tests/formalization_lean/scalar_tests.rs :: test_conjugate_star_mul` | ✓ | n/a |
| `algebra.conjugate.star_add` | `star (a+b) = star a + star b` for `ConjugateScalar` | proved | `DeepCausalityFormal/Algebra/Scalar.lean :: conjugate_star_add` | `deep_causality_algebra/tests/formalization_lean/scalar_tests.rs :: test_conjugate_star_add` | ✓ | n/a |
| `algebra.normed.norm_mul` | `‖a*b‖ = ‖a‖*‖b‖` for `Normed` | proved | `DeepCausalityFormal/Algebra/Scalar.lean :: normed_norm_mul` | `deep_causality_algebra/tests/formalization_lean/scalar_tests.rs :: test_normed_norm_mul` | ✓ | n/a |
| `algebra.normed.norm_nonneg` | `0 ≤ ‖a‖` for `Normed` | proved | `DeepCausalityFormal/Algebra/Scalar.lean :: normed_norm_nonneg` | `deep_causality_algebra/tests/formalization_lean/scalar_tests.rs :: test_normed_norm_nonneg` | ✓ | n/a |
| `complex.field.mul_inv` | `z ≠ 0 → z * z⁻¹ = 1` (ℂ is a field) | proved | `DeepCausalityFormal/Complex/Complex.lean :: complex_field_mul_inv` | `deep_causality_num_complex/tests/formalization_lean/complex_tests.rs :: test_complex_field_mul_inv` | ✓ | n/a |
| `complex.conj.involutive` | `conj (conj z) = z` | proved | `DeepCausalityFormal/Complex/Complex.lean :: complex_conj_involutive` | `deep_causality_num_complex/tests/formalization_lean/complex_tests.rs :: test_complex_conj_involutive` | ✓ | n/a |
| `complex.conj.mul` | `conj (z*w) = conj z * conj w` | proved | `DeepCausalityFormal/Complex/Complex.lean :: complex_conj_mul` | `deep_causality_num_complex/tests/formalization_lean/complex_tests.rs :: test_complex_conj_mul` | ✓ | n/a |
| `complex.norm_sq.mul` | `normSq (z*w) = normSq z * normSq w` | proved | `DeepCausalityFormal/Complex/Complex.lean :: complex_normSq_mul` | `deep_causality_num_complex/tests/formalization_lean/complex_tests.rs :: test_complex_norm_sqr_mul` | ✓ | n/a |
| `complex.norm.mul` | `‖z*w‖ = ‖z‖*‖w‖` | proved | `DeepCausalityFormal/Complex/Complex.lean :: complex_norm_mul` | `deep_causality_num_complex/tests/formalization_lean/complex_tests.rs :: test_complex_norm_mul` | ✓ | n/a |
| `quaternion.division_ring.mul_inv` | `q ≠ 0 → q * q⁻¹ = 1` (ℍ is a division ring) | proved | `DeepCausalityFormal/Complex/Quaternion.lean :: quaternion_mul_inv` | `deep_causality_num_complex/tests/formalization_lean/quaternion_tests.rs :: test_quaternion_division_ring_mul_inv` | ✓ | n/a |
| `quaternion.norm_sq.mul` | `normSq (q*p) = normSq q * normSq p` | proved | `DeepCausalityFormal/Complex/Quaternion.lean :: quaternion_normSq_mul` | `deep_causality_num_complex/tests/formalization_lean/quaternion_tests.rs :: test_quaternion_norm_sqr_mul` | ✓ | n/a |
| `quaternion.conj.mul` | `star (q*p) = star p * star q` | proved | `DeepCausalityFormal/Complex/Quaternion.lean :: quaternion_conj_mul` | `deep_causality_num_complex/tests/formalization_lean/quaternion_tests.rs :: test_quaternion_conj_mul` | ✓ | n/a |
| `quaternion.noncomm` | `∃ q p, q*p ≠ p*q` (ℍ is non-commutative) | proved | `DeepCausalityFormal/Complex/Quaternion.lean :: quaternion_noncomm` | `deep_causality_num_complex/tests/formalization_lean/quaternion_tests.rs :: test_quaternion_noncomm` | ✓ | n/a |
| `dual.comm_ring.mul_comm` | `a*b = b*a` for `R[ε]` (commutative ring) | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_comm_ring_mul_comm` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_mul_comm` | ✓ | n/a |
| `dual.eps_sq_zero` | `ε * ε = 0` | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_eps_sq_zero` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_eps_sq_zero` | ✓ | n/a |
| `dual.real_projection.add` | `fst (a+b) = fst a + fst b` (the value is additive) | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_fst_is_ring_hom_add` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_real_projection_add` | ✓ | n/a |
| `dual.real_projection.mul` | `fst (a*b) = fst a * fst b` (the value multiplies) | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_fst_mul` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_real_projection_mul` | ✓ | n/a |
| `dual.leibniz.product_rule` | `snd (a*b) = fst a * snd b + snd a * fst b` (forward-mode AD product rule) | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_leibniz` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_leibniz_product_rule` | ✓ | n/a |
| `dual.not_field.zero_divisor` | `ε ≠ 0 ∧ ε*ε = 0` (a nonzero zero-divisor; `R[ε]` is not a field) | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_not_field_zero_divisor` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_not_field_zero_divisor` | ✓ | n/a |

### Haft layer (`deep_causality_haft`)

All Lean files under `DeepCausalityFormal/Haft/`; the Rust witnesses live in
`deep_causality_haft/tests/formalization_lean/`, which mirrors the Lean tree one-to-one
(`Haft/Functor.lean` ↔ `functor_tests.rs`, `Haft/EffectSystem.lean` ↔
`effect_system_tests.rs`, …; `Haft/Hkt.lean` is a definitional bridge with no theorems and
hence no test file). One `#[test]` per id, name pattern `test_<id>`. Citations per file;
deviations recorded in `../openspec/notes/causal-algebra/haft-formalization-deviations.md`.

| id | statement | Lean | Lean location | Test | Kani |
|---|---|---|---|---|---|
| `haft.functor.laws` | `fmap id = id`; `fmap (g∘f) = fmap g ∘ fmap f` | proved | `Haft/Functor.lean` | ✓ | n/a |
| `haft.pure.naturality` | `fmap f ∘ pure = pure ∘ f` | proved | `Haft/Pure.lean` | ✓ | n/a |
| `haft.applicative.laws` | McBride–Paterson identity, homomorphism, interchange, **composition** | proved | `Haft/Applicative.lean` | ✓ | n/a |
| `haft.applicative.functor_compat` | `fmap f x = pure f <*> x` | proved | `Haft/Applicative.lean` | ✓ | n/a |
| `haft.monad.laws` | left/right identity, associativity | proved | `Haft/Monad.lean` | ✓ | n/a |
| `haft.monad.applicative_coherence` | `apply = bind (fmap ·)` | proved | `Haft/Monad.lean` | ✓ | n/a |
| `haft.comonad.laws` | Uustalu–Vene coKleisli laws (Env carrier) | proved | `Haft/Comonad.lean` | ✓ | n/a |
| `haft.bifunctor.laws` | `bimap id id = id`; composition; first/second decomposition | proved | `Haft/Bifunctor.lean` | ✓ | n/a |
| `haft.profunctor.laws` | `dimap id id = id`; contravariant-twist composition | proved | `Haft/Profunctor.lean` | ✓ | n/a |
| `haft.parametric_monad.laws` | Atkey indexed monad laws (IxState carrier) | proved | `Haft/ParametricMonad.lean` | ✓ | n/a |
| `haft.monoidal_merge.merge_naturality` | `merge` is binatural (lax-monoidal structure map; trait renamed from `Promonad`, D3/P-1) | proved | `Haft/MonoidalMerge.lean` | ✓ | n/a |
| `haft.free_monad.left_id` | `bind (pure a) k = k a` (free monad on a functor) | proved | `Haft/FreeMonad.lean` | ✓ | n/a |
| `haft.free_monad.right_id` | `bind m pure = m` | proved | `Haft/FreeMonad.lean` | ✓ | n/a |
| `haft.free_monad.assoc` | `bind (bind m f) g = bind m (λx. bind (f x) g)` | proved | `Haft/FreeMonad.lean` | ✓ | n/a |
| `haft.free_monad.lift_bind` | `bind (lift op) k` runs `k` under the operation node | proved | `Haft/FreeMonad.lean` | ✓ | n/a |
| `haft.free_monad.map_id` | `map id = id` (functor identity via right id) | proved | `Haft/FreeMonad.lean` | ✓ | n/a |
| `haft.arrow.category_laws` | `id>>>f = f`; `f>>>id = f`; `>>>` associative | proved | `Haft/Arrow.lean` | ✓ | n/a |
| `haft.category.laws` | function category `Fun`: left/right identity + associativity of `compose` | proved | `Haft/Category.lean` | ✓ | n/a |
| `haft.kleisli.category_laws` | Kleisli category (`id=pure`, `compose=bind`): left/right identity + associativity, reducing to the monad laws | proved | `Haft/Kleisli.lean` | ✓ | n/a |
| `haft.arrow.arr_functor` | `arr id = id`; `arr (g∘f) = arr f >>> arr g` | proved | `Haft/Arrow.lean` | ✓ | n/a |
| `haft.arrow.strength_laws` | Hughes' five `first` laws | proved | `Haft/Arrow.lean` | ✓ | n/a |
| `haft.arrow.derived_combinators` | `second`/`***`/`&&&` from `first` + `arr` | proved | `Haft/Arrow.lean` | ✓ | n/a |
| `haft.arrow_term.interpret_sound` | reified free arrow: `interpret` is a homomorphism — commutes with `compose`/`first`/`second`/`split`/`fanout` (interpreting a term = composing its combinators) | proved | `Haft/ArrowTerm.lean` | ✓ | n/a |
| `haft.arrow_term.free` | free arrow universal property: interpretation is determined by the generators (agree on generators ⇒ agree on every term) | proved | `Haft/ArrowTerm.lean` | ✓ | n/a |
| `haft.arrow_choice.laws` | the ArrowChoice fragment `⊕` over `Either`: `left (arr f) = arr (f ⊕ id)`, functoriality/exchange/unit laws, `fanin` as the coproduct elimination (computation + uniqueness), and the used `⊗`-over-`⊕` distributivity (`distl` iso + naturality; full rig coherence deferred) | proved | `Haft/ArrowChoice.lean` | ✓ | n/a |
| `haft.arrow_term.choice_interpret_sound` | interpreting the choice generators (`left`/`right`/`choice`/`fanin`) agrees with the eager ArrowChoice combinators — routing on the sum node, `fanin` unwrapping (extends `interpret_sound` to the `⊕`-enlarged set) | proved | `Haft/ArrowTerm.lean` | ✓ | n/a |
| `haft.arrow_term.choice_free` | the free/universal property extends to the `⊕`-enlarged generator set: agree on generators ⇒ agree on every choice term | proved | `Haft/ArrowTerm.lean` | ✓ | n/a |
| `haft.interpreter.preserves_id` | interpreter `ArrowTerm → Kleisli<M>` is functorial: `id ↦` target identity (`pure`) | proved | `Haft/Interpreter.lean` | ✓ | n/a |
| `haft.interpreter.preserves_compose` | interpreter is functorial: `compose f g ↦` target composition (`bind`) | proved | `Haft/Interpreter.lean` | ✓ | n/a |
| `haft.interpreter.choice_preserved` | `interpret_kleisli` preserves the choice generators: `left`/`right`/`choice`/`fanin` map to the Kleisli choice arms, the effect runs only on the taken branch (extends `preserves_id`/`preserves_compose`) | proved | `Haft/Interpreter.lean` | ✓ | n/a |
| `haft.interpreter.naturality` | `Option ⇒ List` component (`OptionToVec`) commutes with `map` (naturality square) | proved | `Haft/Interpreter.lean` | ✓ | n/a |
| `haft.monoidal.comonoid_laws` | copy comonoid `(Δ, ε)`: coassociativity, counit, cocommutativity of the diagonal | proved | `Haft/SymmetricMonoidal.lean` | ✓ | n/a |
| `haft.monoidal.merge_monoid_laws` | merge monoid `(∇, η)`: associativity + left/right unit (the monoid laws) | proved | `Haft/SymmetricMonoidal.lean` | ✓ | n/a |
| `haft.monoidal.symmetry` | symmetry `σ` is its own inverse (`σ ∘ σ = id`) | proved | `Haft/SymmetricMonoidal.lean` | ✓ | n/a |
| `haft.endo.monoid` | `End(T)` monoid (unit + associativity) | proved | `Haft/Endomorphism.lean` | ✓ | n/a |
| `haft.endo.iterate_add` | `f^(m+n) = f^n ∘ f^m` | proved | `Haft/Endomorphism.lean` | ✓ | n/a |
| `haft.morphism.identity` | `apply identity a = a` | proved | `Haft/Morphism.lean` | ✓ | n/a |
| `haft.adjunction.triangles` | triangle identities (currying adjunction) | proved | `Haft/Adjunction.lean` | ✓ | n/a |
| `haft.adjunction.adjunct_inverse` | adjuncts are the Hom-bijection | proved | `Haft/Adjunction.lean` | ✓ | n/a |
| `haft.foldable.pure_compat` | `fold (pure x) init f = f init x` | proved | `Haft/Foldable.lean` | ✓ | n/a |
| `haft.foldable.fold_map_pure` | `fold_map(pure a, f) = f a` (singleton law) | proved | `Haft/Foldable.lean` | ✓ | n/a |
| `haft.foldable.fold_map_monoid_coherence` | `fold_map(xs ++ ys, f) = fold_map(xs,f).combine(fold_map(ys,f))` (monoid homomorphism) | proved | `Haft/Foldable.lean` | ✓ | n/a |
| `haft.traversable.identity` | `sequence` at Identity applicative = id | proved | `Haft/Traversable.lean` | ✓ | n/a |
| `haft.traversable.naturality` | applicative morphisms commute with `sequence` | proved | `Haft/Traversable.lean` | ✓ | n/a |
| `haft.natural_iso.laws` | round-trip + naturality (`Option ≅ Unit ⊕ ·`) | proved | `Haft/NaturalIso.lean` | ✓ | n/a |
| `haft.either.coproduct_universal` | `[f,g]` exists and is unique | proved | `Haft/Either.lean` | ✓ | n/a |
| `haft.effect3.monad_laws` | monad laws + raise-left-zero (sum carrier) | proved | `Haft/EffectSystem.lean` | ✓ | n/a |
| `haft.io.monad_laws` | monad laws on the `run` denotation | proved | `Haft/Io.lean` | ✓ | n/a |
| `haft.cybernetic.kleisli_factorization` | `control_step` = Kleisli composite | proved | `Haft/Signatures.lean` | ✓ | n/a |

### Topology layer (`deep_causality_topology`)

Opened by proposal P-3 of the haft deviations note: the `RiemannMap` trait is a bare
signature; the curvature laws live at the concrete `CurvatureTensor`
(`deep_causality_topology/src/types/curvature_tensor/`). Lean files under
`DeepCausalityFormal/Topology/`; Rust witnesses in
`deep_causality_topology/tests/types/curvature_tensor/curvature_tensor_law_tests.rs`.
Reference: do Carmo, *Riemannian Geometry*, Ch. 4.

| id | statement | Lean | Lean location | Test | Kani |
|---|---|---|---|---|---|
| `topology.curvature.antisymmetry` | `R(u,v)w = −R(v,u)w` | proved | `Topology/RiemannCurvature.lean` | ✓ | n/a |
| `topology.curvature.bianchi_first` | `R(u,v)w + R(v,w)u + R(w,u)v = 0` (needs `g` symmetric) | proved | `Topology/RiemannCurvature.lean` | ✓ | n/a |
| `topology.curvature.linearity` | additivity + homogeneity in the transported slot | proved | `Topology/RiemannCurvature.lean` | ✓ | n/a |

## Not yet on the map (blocked / scaling — see Formalization.md work plan)

| id (planned) | statement | blocked on |
|---|---|---|
| `haft.traversable.composition` | `sequence` at a composite applicative `M ∘ N` | needs lawful-applicative hypotheses for `M`, `N` (scaling) |
| `haft.effect_unbound.laws` | indexed-monad laws for `MonadEffect3/4/5Unbound` | same shape as `haft.parametric_monad.laws`; a dedicated carrier model is scaling work |

## Quantum — the partial-trace / Choi foundation and the B1 witness

The `add-quantum-crate` change (`openspec/changes/add-quantum-crate`). The pinned Mathlib v4.15.0 has
no partial trace and no Choi–Jamiołkowski layer, so both are built from first principles on the
pair-indexed matrix model in `DeepCausalityFormal/Quantum/`. The headline is the B1 result: the
unconditional `partial_trace_preservation` is **false** (`partial_trace_nonpreservation`, a witnessed
counterexample), while the *conditional* boundary version holds (`partial_trace_preservation_boundary`).
Rust witnesses in `deep_causality_quantum/tests/kernels/{operator_linalg_tests,channel_tests}.rs`.

| id | statement | Lean | Lean location | Rust witness | Test |
|---|---|---|---|---|---|
| `quantum.partial_trace.add` | `Tr_B(M+N) = Tr_B M + Tr_B N` | proved | `Quantum/PartialTrace.lean :: partialTraceRight_add` | `operator_linalg_tests.rs :: test_partial_trace_linearity` | ✓ |
| `quantum.partial_trace.smul` | `Tr_B(c•M) = c•Tr_B M` | proved | `Quantum/PartialTrace.lean :: partialTraceRight_smul` | `operator_linalg_tests.rs :: test_partial_trace_linearity` | ✓ |
| `quantum.partial_trace.kronecker` | `Tr_B(X⊗Y) = Tr(Y)•X` | proved | `Quantum/PartialTrace.lean :: partialTraceRight_kron` | `operator_linalg_tests.rs :: test_partial_trace_product_identity` | ✓ |
| `quantum.partial_trace.bimodule` | `Tr_B((Z⊗1)·M) = Z·Tr_B M` | proved | `Quantum/PartialTrace.lean :: partialTraceRight_bimodule` | `operator_linalg_tests.rs :: test_partial_trace_bimodule_law` | ✓ |
| `quantum.partial_trace.bimodule_right` | `Tr_B(M·(Z⊗1)) = Tr_B M·Z` | proved | `Quantum/PartialTrace.lean :: partialTraceRight_bimodule_right` | `operator_linalg_tests.rs :: test_partial_trace_bimodule_law` | ✓ |
| `quantum.partial_trace_preservation_boundary` | boundary op commutes ⇒ its A-part commutes with `Tr_B M` (Q-PTP) | proved | `Quantum/PartialTrace.lean :: partial_trace_preservation_boundary` | `operator_linalg_tests.rs :: test_partial_trace_preservation_boundary_case` | ✓ |
| `quantum.partial_trace_nonpreservation` | `[X,Y]=0` but `[Tr_B X, Tr_B Y] ≠ 0` (B1 counterexample) | proved | `Quantum/PartialTraceCounterexample.lean :: partial_trace_nonpreservation` | `operator_linalg_tests.rs :: test_partial_trace_nonpreservation_counterexample` | ✓ |
| `quantum.partial_trace_nonpreservation.value` | `[Tr_B X, Tr_B Y] = [[0,4],[−4,0]]` (`= +4i·σy`) | proved | `Quantum/PartialTraceCounterexample.lean :: partial_trace_nonpreservation_value` | `operator_linalg_tests.rs :: test_partial_trace_nonpreservation_counterexample` | ✓ |
| `quantum.choi.apply_add` | `applyChoi J` is additive in the state | proved | `Quantum/Choi.lean :: applyChoi_add` | `channel_tests.rs :: test_apply_kraus_and_apply_choi_agree` | ✓ |
| `quantum.choi.apply_smul` | `applyChoi J (c•A) = c•applyChoi J A` | proved | `Quantum/Choi.lean :: applyChoi_smul` | `channel_tests.rs :: test_apply_kraus_and_apply_choi_agree` | ✓ |

The **CJ reconstruction isomorphism** `applyChoi (choiOf E) = E` and the QCM theorems
(`quantum.no_influence`, `quantum.markov_commutativity`, `quantum.unitary_factorization`,
`quantum.classical_embedding`, `quantum.cyclic_support`, `quantum.verdict.orthomodular`) are stated
as deferred targets in `openspec/changes/add-quantum-crate`; the crate carries their numerical /
property-test witnesses today. The `/Quantum/` tree is exempt from the CI `sorry` gate while this
foundation is extended.
