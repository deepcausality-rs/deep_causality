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
  - `core` (deferred): **Aeneas** extraction — "the code IS the model".

The `THEOREM_MAP:` tag in each Lean file and the matching comment in each Rust witness carry the
same **id**. CI (`.github/workflows/formalization.yml`) fails if an id lacks either side.

## Legend

- **Lean**: `proved` = closed, no `sorry`; `sorry` = stated but unproved; `—` = not yet stated.
- **Kani** / **Test** / **Aeneas**: `✓` present & passing · `partial` · `—` not started · `n/a`.

## Map

| id | statement | Lean | Lean location | Rust witness | Test | Kani | Aeneas |
|---|---|---|---|---|---|---|---|
| `num.add_monoid.assoc` | `(a+b)+c = a+(b+c)` for `AddMonoid` | proved | `DeepCausalityFormal/Num/Monoid.lean :: add_monoid_assoc` | `deep_causality_num/tests/algebra/monoid_tests.rs :: test_add_monoid_associativity` | ✓ | n/a | — |
| `num.add_monoid.identity` | `a+0 = a ∧ 0+a = a` for `AddMonoid` | proved | `DeepCausalityFormal/Num/Monoid.lean :: add_monoid_identity` | `deep_causality_num/tests/algebra/monoid_tests.rs :: test_add_monoid_identity` | ✓ | n/a | — |
| `core.causal_monad.left_id` | `pure a >>= f = f a` | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_left_id` | `deep_causality_core/tests/kani_proofs.rs :: causal_monad_left_identity` | n/a | ✓ | — |
| `core.causal_monad.right_id` | `m >>= pure = m` (unconditional — holds on errored carriers) | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_right_id` | `deep_causality_core/tests/types/causal_monad/causal_monad_tests.rs :: test_right_identity_unconditional` | ✓ | ✓ | — |
| `core.causal_monad.assoc` | `(m >>= f) >>= g = m >>= (λx. f x >>= g)` | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_assoc` | `deep_causality_core/tests/types/causal_monad/causal_monad_tests.rs :: test_associativity_across_erroring_continuation` | ✓ | ✓ | — |
| `core.causal_monad.left_zero` | `raise e >>= f = raise e` (error short-circuit) | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_raise_left_zero` | `deep_causality_core/tests/kani_proofs.rs :: causal_monad_short_circuit` | n/a | ✓ | — |
| `core.causal_monad.lawful` | `LawfulMonad`-with-effect: left/right identity + associativity co-hold on one carrier (P1 resolved) | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: causal_monad_lawful` | `deep_causality_core/tests/formalization_lean/causal_monad_tests.rs :: test_causal_monad_lawful` | ✓ | — | — |
| `core.causal_arrow.category_laws` | Kleisli category laws (left/right identity, associativity) threading state/context over arbitrary `S`, `C` | proved | `DeepCausalityFormal/Core/CausalArrow.lean :: kcomp_left_id / kcomp_right_id / kcomp_assoc` | `deep_causality_core/tests/types/causal_arrow/causal_arrow_tests.rs :: arrow_threads_accumulated_state` | ✓ | ✓ | — |
| `core.causal_arrow.left_zero` | errored stage short-circuits composition; state preserved, downstream not run | proved | `DeepCausalityFormal/Core/CausalArrow.lean :: kcomp_left_zero` | `deep_causality_core/tests/types/causal_arrow/causal_arrow_tests.rs :: arrow_error_short_circuit_preserves_state` | ✓ | — | — |
| `core.effect_log.left_id` | `append empty x = x` (free monoid / Writer output) | proved | `DeepCausalityFormal/Core/EffectLog.lean :: append_left_id` | `deep_causality_core/tests/formalization_lean/effect_log_tests.rs :: test_effect_log_left_identity` | ✓ | — | — |
| `core.effect_log.right_id` | `append x empty = x` | proved | `DeepCausalityFormal/Core/EffectLog.lean :: append_right_id` | `deep_causality_core/tests/formalization_lean/effect_log_tests.rs :: test_effect_log_right_identity` | ✓ | — | — |
| `core.effect_log.assoc` | `append (append x y) z = append x (append y z)` | proved | `DeepCausalityFormal/Core/EffectLog.lean :: append_assoc` | `deep_causality_core/tests/formalization_lean/effect_log_tests.rs :: test_effect_log_associativity` | ✓ | — | — |
| `core.effect_log.monotone` | incoming log is a prefix of the combined log (append-only) | proved | `DeepCausalityFormal/Core/EffectLog.lean :: append_monotone` | `deep_causality_core/tests/formalization_lean/effect_log_tests.rs :: test_effect_log_monotone_prefix` | ✓ | — | — |
| `core.causal_effect.into_value` | `into_value` is the honest `Maybe` projection (`Pure(Some v)↦Some v`, `Pure(None)`/command`↦None`); value functor = `Option` (`haft.functor.laws`) | proved | `DeepCausalityFormal/Core/CausalEffect.lean :: into_value_value / into_value_none / into_value_command` | `deep_causality_core/tests/formalization_lean/causal_effect_tests.rs :: test_causal_effect_into_value` | ✓ | — | — |
| `core.causal_command.functor_laws` | single-hole `CausalCommand` functor laws (`fmap id = id`, `fmap (g∘f) = fmap g ∘ fmap f`); free monad over it = `haft.free_monad.*` | proved | `DeepCausalityFormal/Core/CausalCommand.lean :: cmap_id / cmap_comp` | `deep_causality_core/tests/formalization_lean/causal_command_tests.rs :: test_causal_command_functor_laws` | ✓ | — | — |
| `core.witness.agree` | every law-bearing HKT surface computes the SAME total success-channel functor/applicative: `fmap` = inherent `fmap` on every carrier (`Some`/`None`/`Err`/**command**, command preserved not collapsed, no panic); `apply` total — value-less/command operand ↦ `None`, never `InternalLogicError` (D15 fully retired across all three witnesses) | proved | `DeepCausalityFormal/Core/Consistency.lean :: witness_agree / fmap_preserves_command / apply_none_yields_none / apply_command_yields_none` | `deep_causality_core/tests/formalization_lean/consistency_tests.rs :: test_witness_agrees_with_inherent_fmap / test_apply_none_operand_yields_none / test_apply_command_operand_yields_none` | ✓ | — | — |
| `core.alternatable.set_get` | lens set-get on the log-erasing projection (value/state/context) | proved | `DeepCausalityFormal/Core/Alternatable.lean :: set_get_value / set_get_state / set_get_context` | `deep_causality_core/tests/formalization_lean/alternatable_tests.rs :: test_alternatable_set_get` | ✓ | — | — |
| `core.alternatable.set_set_proj` | lens set-set idempotence up-to-log (`proj`); full carrier grows the log (D9) | proved | `DeepCausalityFormal/Core/Alternatable.lean :: set_set_value_proj / set_set_state_proj / set_set_context_proj / set_set_grows_log` | `deep_causality_core/tests/formalization_lean/alternatable_tests.rs :: test_alternatable_set_set_up_to_log` | ✓ | — | — |
| `core.alternatable.channel_independence` | each setter touches only its own channel | proved | `DeepCausalityFormal/Core/Alternatable.lean :: value_preserves_state_ctx / state_preserves_value_ctx / context_preserves_value_state` | `deep_causality_core/tests/formalization_lean/alternatable_tests.rs :: test_alternatable_channel_independence` | ✓ | — | — |
| `core.alternatable.error_noop` | every setter (and `clear_context`) is a no-op on an errored carrier | proved | `DeepCausalityFormal/Core/Alternatable.lean :: value_error_noop / state_error_noop / context_error_noop / clear_context_error_noop` | `deep_causality_core/tests/formalization_lean/alternatable_tests.rs :: test_alternatable_error_noop` | ✓ | — | — |
| `core.causal_flow.flow_iso` | `CausalFlow ≅ Process` (newtype wrap/unwrap, `rfl`) | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: flow_iso` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_iso` | ✓ | — | — |
| `core.causal_flow.map_id` | `map id = id` (facade functor identity) | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: map_id` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_map_id` | ✓ | — | — |
| `core.causal_flow.map_comp` | `map (g∘f) = map g ∘ map f` | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: map_comp` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_map_comp` | ✓ | — | — |
| `core.causal_flow.map_eq_andThen` | `map f = and_then (pure∘f)` — holds on the `None` effect as well as a value (D14) | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: map_eq_andThen` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_map_eq_and_then` | ✓ | — | — |
| `core.causal_flow.recover` | `MonadError.catch`: no-op on success, raise↦handler value | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: recover_catch` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_recover` | ✓ | — | — |
| `core.causal_flow.iterate` | bounded search terminates; budget exhaustion injects `MaxStepsExceeded`, state/context/log preserved | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: iterate_contract` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_iterate` | ✓ | — | — |
| `core.causal_flow.finish` | terminal value-observation drops state/context/log (depends only on the outcome) | proved | `DeepCausalityFormal/Core/CausalFlow.lean :: finish_drops_state_ctx_log` | `deep_causality_core/tests/formalization_lean/causal_flow_tests.rs :: test_causal_flow_finish` | ✓ | — | — |
| `core.io.csv_roundtrip` | `parse (render header rows) = header :: rows` under the no-`','`/no-`'\n'` precondition (cites `haft.io.laws`) | proved | `DeepCausalityFormal/Core/Csv.lean :: csv_roundtrip` | `deep_causality_core/tests/formalization_lean/csv_tests.rs :: test_csv_roundtrip` | ✓ | — | — |
| `core.graph_join.unique_valuation` | the acyclic labeled system `σ(n)=f_n(σ|Pa(n))` has exactly one solution (well-founded induction on rank; no algebraic hypothesis on mechanisms) | proved | `DeepCausalityFormal/Core/GraphJoin.lean :: unique_valuation` | `deep_causality/tests/formalization_lean/graph_join_tests.rs :: test_core_graph_join_unique_valuation` | ✓ | — | — |
| `core.graph_join.schedule_invariance` | two schedules each producing a solution agree — the topological linearization does not change the result (command-free) | proved | `DeepCausalityFormal/Core/GraphJoin.lean :: schedule_invariance` | `deep_causality/tests/formalization_lean/graph_join_tests.rs :: test_core_graph_join_schedule_invariance` | ✓ | — | — |
| `core.graph_join.union_comm` | disjoint-key parent-map union is commutative (fan-in order-independence, by construction) | proved | `DeepCausalityFormal/Core/GraphJoin.lean :: unionMap_comm` | `deep_causality/tests/formalization_lean/graph_join_tests.rs :: test_core_graph_join_union_comm` | ✓ | — | — |
| `core.graph_join.classical_copy` | fan-out delivers the same value `σ(n)` to every child — the copy law of the *classical* interpreter (scoped; quantum replaces copy with commuting access) | proved | `DeepCausalityFormal/Core/GraphJoin.lean :: classical_copy` | `deep_causality/tests/formalization_lean/graph_join_tests.rs :: test_core_graph_join_classical_copy` | ✓ | — | — |
| `core.graph_join.linear_surgery_locality` | cutting parent `p`'s wire drops exactly `weights[p]·v_p` from the `LinearJoin` result (kernel-level shadow of opening a mechanism) | proved | `DeepCausalityFormal/Core/GraphJoin.lean :: linear_surgery_locality` | `deep_causality/tests/formalization_lean/graph_join_tests.rs :: test_core_graph_join_linear_surgery_locality` | ✓ | — | — |

### Haft layer (`deep_causality_haft`)

All Lean files under `DeepCausalityFormal/Haft/`; the Rust witnesses live in
`deep_causality_haft/tests/formalization_lean/`, which mirrors the Lean tree one-to-one
(`Haft/Functor.lean` ↔ `functor_tests.rs`, `Haft/EffectSystem.lean` ↔
`effect_system_tests.rs`, …; `Haft/Hkt.lean` is a definitional bridge with no theorems and
hence no test file). One `#[test]` per id, name pattern `test_<id>`. Citations per file;
deviations recorded in `../openspec/notes/causal-algebra/haft-formalization-deviations.md`.

| id | statement | Lean | Lean location | Test | Kani | Aeneas |
|---|---|---|---|---|---|---|
| `haft.functor.laws` | `fmap id = id`; `fmap (g∘f) = fmap g ∘ fmap f` | proved | `Haft/Functor.lean` | ✓ | n/a | — |
| `haft.pure.naturality` | `fmap f ∘ pure = pure ∘ f` | proved | `Haft/Pure.lean` | ✓ | n/a | — |
| `haft.applicative.laws` | McBride–Paterson identity, homomorphism, interchange, **composition** | proved | `Haft/Applicative.lean` | ✓ | n/a | — |
| `haft.applicative.functor_compat` | `fmap f x = pure f <*> x` | proved | `Haft/Applicative.lean` | ✓ | n/a | — |
| `haft.monad.laws` | left/right identity, associativity | proved | `Haft/Monad.lean` | ✓ | n/a | — |
| `haft.monad.applicative_coherence` | `apply = bind (fmap ·)` | proved | `Haft/Monad.lean` | ✓ | n/a | — |
| `haft.comonad.laws` | Uustalu–Vene coKleisli laws (Env carrier) | proved | `Haft/Comonad.lean` | ✓ | n/a | — |
| `haft.bifunctor.laws` | `bimap id id = id`; composition; first/second decomposition | proved | `Haft/Bifunctor.lean` | ✓ | n/a | — |
| `haft.profunctor.laws` | `dimap id id = id`; contravariant-twist composition | proved | `Haft/Profunctor.lean` | ✓ | n/a | — |
| `haft.parametric_monad.laws` | Atkey indexed monad laws (IxState carrier) | proved | `Haft/ParametricMonad.lean` | ✓ | n/a | — |
| `haft.monoidal_merge.merge_naturality` | `merge` is binatural (lax-monoidal structure map; trait renamed from `Promonad`, D3/P-1) | proved | `Haft/MonoidalMerge.lean` | ✓ | n/a | — |
| `haft.free_monad.left_id` | `bind (pure a) k = k a` (free monad on a functor) | proved | `Haft/FreeMonad.lean` | ✓ | n/a | — |
| `haft.free_monad.right_id` | `bind m pure = m` | proved | `Haft/FreeMonad.lean` | ✓ | n/a | — |
| `haft.free_monad.assoc` | `bind (bind m f) g = bind m (λx. bind (f x) g)` | proved | `Haft/FreeMonad.lean` | ✓ | n/a | — |
| `haft.free_monad.lift_bind` | `bind (lift op) k` runs `k` under the operation node | proved | `Haft/FreeMonad.lean` | ✓ | n/a | — |
| `haft.free_monad.map_id` | `map id = id` (functor identity via right id) | proved | `Haft/FreeMonad.lean` | ✓ | n/a | — |
| `haft.arrow.category_laws` | `id>>>f = f`; `f>>>id = f`; `>>>` associative | proved | `Haft/Arrow.lean` | ✓ | n/a | — |
| `haft.arrow.arr_functor` | `arr id = id`; `arr (g∘f) = arr f >>> arr g` | proved | `Haft/Arrow.lean` | ✓ | n/a | — |
| `haft.arrow.strength_laws` | Hughes' five `first` laws | proved | `Haft/Arrow.lean` | ✓ | n/a | — |
| `haft.arrow.derived_combinators` | `second`/`***`/`&&&` from `first` + `arr` | proved | `Haft/Arrow.lean` | ✓ | n/a | — |
| `haft.endo.monoid` | `End(T)` monoid (unit + associativity) | proved | `Haft/Endomorphism.lean` | ✓ | n/a | — |
| `haft.endo.iterate_add` | `f^(m+n) = f^n ∘ f^m` | proved | `Haft/Endomorphism.lean` | ✓ | n/a | — |
| `haft.morphism.identity` | `apply identity a = a` | proved | `Haft/Morphism.lean` | ✓ | n/a | — |
| `haft.adjunction.triangles` | triangle identities (currying adjunction) | proved | `Haft/Adjunction.lean` | ✓ | n/a | — |
| `haft.adjunction.adjunct_inverse` | adjuncts are the Hom-bijection | proved | `Haft/Adjunction.lean` | ✓ | n/a | — |
| `haft.foldable.pure_compat` | `fold (pure x) init f = f init x` | proved | `Haft/Foldable.lean` | ✓ | n/a | — |
| `haft.traversable.identity` | `sequence` at Identity applicative = id | proved | `Haft/Traversable.lean` | ✓ | n/a | — |
| `haft.traversable.naturality` | applicative morphisms commute with `sequence` | proved | `Haft/Traversable.lean` | ✓ | n/a | — |
| `haft.natural_iso.laws` | round-trip + naturality (`Option ≅ Unit ⊕ ·`) | proved | `Haft/NaturalIso.lean` | ✓ | n/a | — |
| `haft.either.coproduct_universal` | `[f,g]` exists and is unique | proved | `Haft/Either.lean` | ✓ | n/a | — |
| `haft.effect3.monad_laws` | monad laws + raise-left-zero (sum carrier) | proved | `Haft/EffectSystem.lean` | ✓ | n/a | — |
| `haft.io.monad_laws` | monad laws on the `run` denotation | proved | `Haft/Io.lean` | ✓ | n/a | — |
| `haft.cybernetic.kleisli_factorization` | `control_step` = Kleisli composite | proved | `Haft/Signatures.lean` | ✓ | n/a | — |

### Topology layer (`deep_causality_topology`)

Opened by proposal P-3 of the haft deviations note: the `RiemannMap` trait is a bare
signature; the curvature laws live at the concrete `CurvatureTensor`
(`deep_causality_topology/src/types/curvature_tensor/`). Lean files under
`DeepCausalityFormal/Topology/`; Rust witnesses in
`deep_causality_topology/tests/types/curvature_tensor/curvature_tensor_law_tests.rs`.
Reference: do Carmo, *Riemannian Geometry*, Ch. 4.

| id | statement | Lean | Lean location | Test | Kani | Aeneas |
|---|---|---|---|---|---|---|
| `topology.curvature.antisymmetry` | `R(u,v)w = −R(v,u)w` | proved | `Topology/RiemannCurvature.lean` | ✓ | n/a | — |
| `topology.curvature.bianchi_first` | `R(u,v)w + R(v,w)u + R(w,u)v = 0` (needs `g` symmetric) | proved | `Topology/RiemannCurvature.lean` | ✓ | n/a | — |
| `topology.curvature.linearity` | additivity + homogeneity in the transported slot | proved | `Topology/RiemannCurvature.lean` | ✓ | n/a | — |

## Not yet on the map (blocked / scaling — see Formalization.md work plan)

| id (planned) | statement | blocked on |
|---|---|---|
| `haft.traversable.composition` | `sequence` at a composite applicative `M ∘ N` | needs lawful-applicative hypotheses for `M`, `N` (scaling) |
| `haft.effect_unbound.laws` | indexed-monad laws for `MonadEffect3/4/5Unbound` | same shape as `haft.parametric_monad.laws`; a dedicated carrier model is scaling work |
