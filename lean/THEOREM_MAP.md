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
| `algebra.add_monoid.assoc` | `(a+b)+c = a+(b+c)` for `AddMonoid` | proved | `DeepCausalityFormal/Algebra/Monoid.lean :: add_monoid_assoc` | `deep_causality_algebra/tests/formalization_lean/monoid_tests.rs :: test_add_monoid_assoc` | ✓ | n/a | — |
| `algebra.add_monoid.identity` | `a+0 = a ∧ 0+a = a` for `AddMonoid` | proved | `DeepCausalityFormal/Algebra/Monoid.lean :: add_monoid_identity` | `deep_causality_algebra/tests/formalization_lean/monoid_tests.rs :: test_add_monoid_identity` | ✓ | n/a | — |
| `algebra.monoid.left_id` | `empty().combine(x) = x` for the generic `Monoid` (Mathlib `one_mul`) | proved | `DeepCausalityFormal/Algebra/MonoidGeneric.lean :: monoid_left_id` | `deep_causality_algebra/tests/formalization_lean/monoid_generic_tests.rs :: test_generic_monoid_laws` | ✓ | n/a | — |
| `algebra.monoid.right_id` | `x.combine(empty()) = x` (Mathlib `mul_one`) | proved | `DeepCausalityFormal/Algebra/MonoidGeneric.lean :: monoid_right_id` | `deep_causality_algebra/tests/formalization_lean/monoid_generic_tests.rs :: test_generic_monoid_laws` | ✓ | n/a | — |
| `algebra.monoid.assoc` | `combine` associativity (Mathlib `mul_assoc`) | proved | `DeepCausalityFormal/Algebra/MonoidGeneric.lean :: monoid_assoc` | `deep_causality_algebra/tests/formalization_lean/monoid_generic_tests.rs :: test_generic_monoid_laws` | ✓ | n/a | — |
| `algebra.commutative_monoid.comm` | `x.combine(y) = y.combine(x)` for `CommutativeMonoid` (Mathlib `mul_comm`) | proved | `DeepCausalityFormal/Algebra/CommutativeMonoid.lean :: commutative_monoid_comm` | `deep_causality_algebra/tests/formalization_lean/commutative_monoid_tests.rs :: test_commutative_monoid_comm` | ✓ | n/a | — |
| `algebra.semilattice.idempotent` | `x.combine(x) = x` for the boolean ∧-semilattice (`Conjunction`) | proved | `DeepCausalityFormal/Algebra/CommutativeMonoid.lean :: semilattice_idempotent` | `deep_causality_algebra/tests/formalization_lean/commutative_monoid_tests.rs :: test_semilattice_idempotent` | ✓ | n/a | — |
| `algebra.semilattice.assoc` | associativity of the boolean ∧-semilattice | proved | `DeepCausalityFormal/Algebra/CommutativeMonoid.lean :: semilattice_assoc` | `deep_causality_algebra/tests/formalization_lean/commutative_monoid_tests.rs :: test_semilattice_assoc_and_comm` | ✓ | n/a | — |
| `algebra.semilattice.comm` | commutativity of the boolean ∧-semilattice | proved | `DeepCausalityFormal/Algebra/CommutativeMonoid.lean :: semilattice_comm` | `deep_causality_algebra/tests/formalization_lean/commutative_monoid_tests.rs :: test_semilattice_assoc_and_comm` | ✓ | n/a | — |
| `algebra.verdict.lattice_laws` | boolean verdict lattice: meet commutativity + absorption | proved | `DeepCausalityFormal/Algebra/Verdict.lean :: verdict_meet_comm / verdict_absorption` | `deep_causality_algebra/tests/formalization_lean/verdict_tests.rs :: test_verdict_lattice_laws` | ✓ | n/a | — |
| `algebra.verdict.complement` | complement involution + De Morgan (Boolean); MV-algebra complement `1−p` (Prob) | proved | `DeepCausalityFormal/Algebra/Verdict.lean :: verdict_compl_compl / verdict_de_morgan` | `deep_causality_algebra/tests/formalization_lean/verdict_tests.rs :: test_verdict_complement` | ✓ | n/a | — |
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


### Num / Algebra / Complex / Dual layers (extracted numeric crates)

| id | statement | Lean | Lean location | Rust witness | Test | Kani | Aeneas |
|---|---|---|---|---|---|---|---|
| `num.one.identity` | `1*a = a ∧ a*1 = a` (the `One` identity) | proved | `DeepCausalityFormal/Num/Identity.lean :: one_is_identity` | `deep_causality_num/tests/formalization_lean/identity_tests.rs :: test_one_identity` | ✓ | n/a | — |
| `num.zero.identity` | `0+a = a ∧ a+0 = a` (the `Zero` identity) | proved | `DeepCausalityFormal/Num/Identity.lean :: zero_is_identity` | `deep_causality_num/tests/formalization_lean/identity_tests.rs :: test_zero_identity` | ✓ | n/a | — |
| `num.integer.mul_comm` | `a*b = b*a` over `ℤ` | proved | `DeepCausalityFormal/Num/Integer.lean :: integer_mul_comm` | `deep_causality_num/tests/formalization_lean/integer_tests.rs :: test_integer_mul_comm` | ✓ | n/a | — |
| `num.integer.distrib` | `a*(b+c) = a*b + a*c` over `ℤ` | proved | `DeepCausalityFormal/Num/Integer.lean :: integer_left_distrib` | `deep_causality_num/tests/formalization_lean/integer_tests.rs :: test_integer_distrib` | ✓ | n/a | — |
| `num.integer.euclidean` | `b*(a/b) + a%b = a` over `ℤ` (Euclidean division) | proved | `DeepCausalityFormal/Num/Integer.lean :: integer_euclidean` | `deep_causality_num/tests/formalization_lean/integer_tests.rs :: test_integer_euclidean` | ✓ | n/a | — |
| `num.cast.nat_int_roundtrip` | `((n:ℤ)).toNat = n` (ℕ↔ℤ cast round-trip) | proved | `DeepCausalityFormal/Num/Cast.lean :: cast_nat_int_roundtrip` | `deep_causality_num/tests/formalization_lean/cast_tests.rs :: test_cast_nat_int_roundtrip` | ✓ | n/a | — |
| `num.cast.int_injective` | `ℤ → ℚ` cast is injective | proved | `DeepCausalityFormal/Num/Cast.lean :: cast_int_injective` | `deep_causality_num/tests/formalization_lean/cast_tests.rs :: test_cast_int_injective` | ✓ | n/a | — |
| `num.float106.model.add_comm` | `a+b = b+a` (real-field model of the `Float106` double-double; bit-exact bounds are [open]) | proved | `DeepCausalityFormal/Num/Float106.lean :: float106_model_add_comm` | `deep_causality_num/tests/formalization_lean/float106_tests.rs :: test_float106_add_comm` | ✓ | n/a | — |
| `num.float106.model.mul_comm` | `a*b = b*a` (real-field model of `Float106`) | proved | `DeepCausalityFormal/Num/Float106.lean :: float106_model_mul_comm` | `deep_causality_num/tests/formalization_lean/float106_tests.rs :: test_float106_mul_comm` | ✓ | n/a | — |
| `num.float106.model.distrib` | `a*(b+c) = a*b + a*c` (real-field model of `Float106`) | proved | `DeepCausalityFormal/Num/Float106.lean :: float106_model_distrib` | `deep_causality_num/tests/formalization_lean/float106_tests.rs :: test_float106_distrib` | ✓ | n/a | — |
| `algebra.group.mul_inv` | `a * a⁻¹ = 1` for `Group` | proved | `DeepCausalityFormal/Algebra/Group.lean :: group_mul_inv` | `deep_causality_algebra/tests/formalization_lean/group_tests.rs :: test_group_mul_inv` | ✓ | n/a | — |
| `algebra.add_group.neg_cancel` | `-a + a = 0` for `AddGroup` | proved | `DeepCausalityFormal/Algebra/Group.lean :: add_group_neg_cancel` | `deep_causality_algebra/tests/formalization_lean/group_tests.rs :: test_add_group_neg_cancel` | ✓ | n/a | — |
| `algebra.abelian_group.add_comm` | `a + b = b + a` for `AbelianGroup` | proved | `DeepCausalityFormal/Algebra/Group.lean :: abelian_group_add_comm` | `deep_causality_algebra/tests/formalization_lean/group_tests.rs :: test_abelian_group_add_comm` | ✓ | n/a | — |
| `algebra.ring.left_distrib` | `a*(b+c) = a*b + a*c` for `Ring` | proved | `DeepCausalityFormal/Algebra/Ring.lean :: ring_left_distrib` | `deep_causality_algebra/tests/formalization_lean/ring_tests.rs :: test_ring_left_distrib` | ✓ | n/a | — |
| `algebra.ring.right_distrib` | `(a+b)*c = a*c + b*c` for `Ring` | proved | `DeepCausalityFormal/Algebra/Ring.lean :: ring_right_distrib` | `deep_causality_algebra/tests/formalization_lean/ring_tests.rs :: test_ring_right_distrib` | ✓ | n/a | — |
| `algebra.ring.mul_assoc` | `(a*b)*c = a*(b*c)` for `AssociativeRing` | proved | `DeepCausalityFormal/Algebra/Ring.lean :: ring_mul_assoc` | `deep_causality_algebra/tests/formalization_lean/ring_tests.rs :: test_ring_mul_assoc` | ✓ | n/a | — |
| `algebra.commutative_ring.mul_comm` | `a*b = b*a` for `CommutativeRing` | proved | `DeepCausalityFormal/Algebra/Ring.lean :: commutative_ring_mul_comm` | `deep_causality_algebra/tests/formalization_lean/ring_tests.rs :: test_commutative_ring_mul_comm` | ✓ | n/a | — |
| `algebra.field.mul_inv_cancel` | `a ≠ 0 → a * a⁻¹ = 1` for `Field` | proved | `DeepCausalityFormal/Algebra/Field.lean :: field_mul_inv_cancel` | `deep_causality_algebra/tests/formalization_lean/field_tests.rs :: test_field_mul_inv_cancel` | ✓ | n/a | — |
| `algebra.field.inv_mul_cancel` | `a ≠ 0 → a⁻¹ * a = 1` for `Field` | proved | `DeepCausalityFormal/Algebra/Field.lean :: field_inv_mul_cancel` | `deep_causality_algebra/tests/formalization_lean/field_tests.rs :: test_field_inv_mul_cancel` | ✓ | n/a | — |
| `algebra.real_field.mul_pos` | `0<a → 0<b → 0<a*b` for the ordered `RealField` | proved | `DeepCausalityFormal/Algebra/Field.lean :: real_field_mul_pos` | `deep_causality_algebra/tests/formalization_lean/field_tests.rs :: test_real_field_mul_pos` | ✓ | n/a | — |
| `algebra.module.smul_add` | `r•(x+y) = r•x + r•y` for `Module` | proved | `DeepCausalityFormal/Algebra/Module.lean :: module_smul_add` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_module_smul_add` | ✓ | n/a | — |
| `algebra.module.add_smul` | `(r+s)•x = r•x + s•x` for `Module` | proved | `DeepCausalityFormal/Algebra/Module.lean :: module_add_smul` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_module_add_smul` | ✓ | n/a | — |
| `algebra.module.one_smul` | `1•x = x` for `Module` | proved | `DeepCausalityFormal/Algebra/Module.lean :: module_one_smul` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_module_one_smul` | ✓ | n/a | — |
| `algebra.module.mul_smul` | `(r*s)•x = r•(s•x)` for `Module` | proved | `DeepCausalityFormal/Algebra/Module.lean :: module_mul_smul` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_module_mul_smul` | ✓ | n/a | — |
| `algebra.algebra.smul_mul_assoc` | `r•(a*b) = (r•a)*b` for `Algebra` over a ring | proved | `DeepCausalityFormal/Algebra/Module.lean :: algebra_smul_mul_assoc` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_algebra_smul_mul_assoc` | ✓ | n/a | — |
| `algebra.algebra.mul_smul_comm` | `r•(a*b) = a*(r•b)` for `Algebra` over a ring | proved | `DeepCausalityFormal/Algebra/Module.lean :: algebra_mul_smul_comm` | `deep_causality_algebra/tests/formalization_lean/module_tests.rs :: test_algebra_mul_smul_comm` | ✓ | n/a | — |
| `algebra.division_algebra.mul_inv` | `a ≠ 0 → a * a⁻¹ = 1` for `DivisionAlgebra` | proved | `DeepCausalityFormal/Algebra/DivisionAlgebra.lean :: division_algebra_mul_inv` | `deep_causality_algebra/tests/formalization_lean/division_algebra_tests.rs :: test_division_algebra_mul_inv` | ✓ | n/a | — |
| `algebra.conjugate.star_star` | `star (star a) = a` for `ConjugateScalar` | proved | `DeepCausalityFormal/Algebra/Scalar.lean :: conjugate_star_star` | `deep_causality_algebra/tests/formalization_lean/scalar_tests.rs :: test_conjugate_star_star` | ✓ | n/a | — |
| `algebra.conjugate.star_mul` | `star (a*b) = star b * star a` for `ConjugateScalar` | proved | `DeepCausalityFormal/Algebra/Scalar.lean :: conjugate_star_mul` | `deep_causality_algebra/tests/formalization_lean/scalar_tests.rs :: test_conjugate_star_mul` | ✓ | n/a | — |
| `algebra.conjugate.star_add` | `star (a+b) = star a + star b` for `ConjugateScalar` | proved | `DeepCausalityFormal/Algebra/Scalar.lean :: conjugate_star_add` | `deep_causality_algebra/tests/formalization_lean/scalar_tests.rs :: test_conjugate_star_add` | ✓ | n/a | — |
| `algebra.normed.norm_mul` | `‖a*b‖ = ‖a‖*‖b‖` for `Normed` | proved | `DeepCausalityFormal/Algebra/Scalar.lean :: normed_norm_mul` | `deep_causality_algebra/tests/formalization_lean/scalar_tests.rs :: test_normed_norm_mul` | ✓ | n/a | — |
| `algebra.normed.norm_nonneg` | `0 ≤ ‖a‖` for `Normed` | proved | `DeepCausalityFormal/Algebra/Scalar.lean :: normed_norm_nonneg` | `deep_causality_algebra/tests/formalization_lean/scalar_tests.rs :: test_normed_norm_nonneg` | ✓ | n/a | — |
| `complex.field.mul_inv` | `z ≠ 0 → z * z⁻¹ = 1` (ℂ is a field) | proved | `DeepCausalityFormal/Complex/Complex.lean :: complex_field_mul_inv` | `deep_causality_num_complex/tests/formalization_lean/complex_tests.rs :: test_complex_field_mul_inv` | ✓ | n/a | — |
| `complex.conj.involutive` | `conj (conj z) = z` | proved | `DeepCausalityFormal/Complex/Complex.lean :: complex_conj_involutive` | `deep_causality_num_complex/tests/formalization_lean/complex_tests.rs :: test_complex_conj_involutive` | ✓ | n/a | — |
| `complex.conj.mul` | `conj (z*w) = conj z * conj w` | proved | `DeepCausalityFormal/Complex/Complex.lean :: complex_conj_mul` | `deep_causality_num_complex/tests/formalization_lean/complex_tests.rs :: test_complex_conj_mul` | ✓ | n/a | — |
| `complex.norm_sq.mul` | `normSq (z*w) = normSq z * normSq w` | proved | `DeepCausalityFormal/Complex/Complex.lean :: complex_normSq_mul` | `deep_causality_num_complex/tests/formalization_lean/complex_tests.rs :: test_complex_norm_sqr_mul` | ✓ | n/a | — |
| `complex.norm.mul` | `‖z*w‖ = ‖z‖*‖w‖` | proved | `DeepCausalityFormal/Complex/Complex.lean :: complex_norm_mul` | `deep_causality_num_complex/tests/formalization_lean/complex_tests.rs :: test_complex_norm_mul` | ✓ | n/a | — |
| `quaternion.division_ring.mul_inv` | `q ≠ 0 → q * q⁻¹ = 1` (ℍ is a division ring) | proved | `DeepCausalityFormal/Complex/Quaternion.lean :: quaternion_mul_inv` | `deep_causality_num_complex/tests/formalization_lean/quaternion_tests.rs :: test_quaternion_division_ring_mul_inv` | ✓ | n/a | — |
| `quaternion.norm_sq.mul` | `normSq (q*p) = normSq q * normSq p` | proved | `DeepCausalityFormal/Complex/Quaternion.lean :: quaternion_normSq_mul` | `deep_causality_num_complex/tests/formalization_lean/quaternion_tests.rs :: test_quaternion_norm_sqr_mul` | ✓ | n/a | — |
| `quaternion.conj.mul` | `star (q*p) = star p * star q` | proved | `DeepCausalityFormal/Complex/Quaternion.lean :: quaternion_conj_mul` | `deep_causality_num_complex/tests/formalization_lean/quaternion_tests.rs :: test_quaternion_conj_mul` | ✓ | n/a | — |
| `quaternion.noncomm` | `∃ q p, q*p ≠ p*q` (ℍ is non-commutative) | proved | `DeepCausalityFormal/Complex/Quaternion.lean :: quaternion_noncomm` | `deep_causality_num_complex/tests/formalization_lean/quaternion_tests.rs :: test_quaternion_noncomm` | ✓ | n/a | — |
| `dual.comm_ring.mul_comm` | `a*b = b*a` for `R[ε]` (commutative ring) | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_comm_ring_mul_comm` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_mul_comm` | ✓ | n/a | — |
| `dual.eps_sq_zero` | `ε * ε = 0` | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_eps_sq_zero` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_eps_sq_zero` | ✓ | n/a | — |
| `dual.real_projection.add` | `fst (a+b) = fst a + fst b` (the value is additive) | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_fst_is_ring_hom_add` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_real_projection_add` | ✓ | n/a | — |
| `dual.real_projection.mul` | `fst (a*b) = fst a * fst b` (the value multiplies) | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_fst_mul` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_real_projection_mul` | ✓ | n/a | — |
| `dual.leibniz.product_rule` | `snd (a*b) = fst a * snd b + snd a * fst b` (forward-mode AD product rule) | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_leibniz` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_leibniz_product_rule` | ✓ | n/a | — |
| `dual.not_field.zero_divisor` | `ε ≠ 0 ∧ ε*ε = 0` (a nonzero zero-divisor; `R[ε]` is not a field) | proved | `DeepCausalityFormal/Dual/Dual.lean :: dual_not_field_zero_divisor` | `deep_causality_num_dual/tests/formalization_lean/dual_tests.rs :: test_not_field_zero_divisor` | ✓ | n/a | — |

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
| `haft.category.laws` | function category `Fun`: left/right identity + associativity of `compose` | proved | `Haft/Category.lean` | ✓ | n/a | — |
| `haft.kleisli.category_laws` | Kleisli category (`id=pure`, `compose=bind`): left/right identity + associativity, reducing to the monad laws | proved | `Haft/Kleisli.lean` | ✓ | n/a | — |
| `haft.arrow.arr_functor` | `arr id = id`; `arr (g∘f) = arr f >>> arr g` | proved | `Haft/Arrow.lean` | ✓ | n/a | — |
| `haft.arrow.strength_laws` | Hughes' five `first` laws | proved | `Haft/Arrow.lean` | ✓ | n/a | — |
| `haft.arrow.derived_combinators` | `second`/`***`/`&&&` from `first` + `arr` | proved | `Haft/Arrow.lean` | ✓ | n/a | — |
| `haft.arrow_term.interpret_sound` | reified free arrow: `interpret` is a homomorphism — commutes with `compose`/`first`/`second`/`split`/`fanout` (interpreting a term = composing its combinators) | proved | `Haft/ArrowTerm.lean` | ✓ | n/a | — |
| `haft.arrow_term.free` | free arrow universal property: interpretation is determined by the generators (agree on generators ⇒ agree on every term) | proved | `Haft/ArrowTerm.lean` | ✓ | n/a | — |
| `haft.interpreter.preserves_id` | interpreter `ArrowTerm → Kleisli<M>` is functorial: `id ↦` target identity (`pure`) | proved | `Haft/Interpreter.lean` | ✓ | n/a | — |
| `haft.interpreter.preserves_compose` | interpreter is functorial: `compose f g ↦` target composition (`bind`) | proved | `Haft/Interpreter.lean` | ✓ | n/a | — |
| `haft.interpreter.naturality` | `Option ⇒ List` component (`OptionToVec`) commutes with `map` (naturality square) | proved | `Haft/Interpreter.lean` | ✓ | n/a | — |
| `haft.endo.monoid` | `End(T)` monoid (unit + associativity) | proved | `Haft/Endomorphism.lean` | ✓ | n/a | — |
| `haft.endo.iterate_add` | `f^(m+n) = f^n ∘ f^m` | proved | `Haft/Endomorphism.lean` | ✓ | n/a | — |
| `haft.morphism.identity` | `apply identity a = a` | proved | `Haft/Morphism.lean` | ✓ | n/a | — |
| `haft.adjunction.triangles` | triangle identities (currying adjunction) | proved | `Haft/Adjunction.lean` | ✓ | n/a | — |
| `haft.adjunction.adjunct_inverse` | adjuncts are the Hom-bijection | proved | `Haft/Adjunction.lean` | ✓ | n/a | — |
| `haft.foldable.pure_compat` | `fold (pure x) init f = f init x` | proved | `Haft/Foldable.lean` | ✓ | n/a | — |
| `haft.foldable.fold_map_pure` | `fold_map(pure a, f) = f a` (singleton law) | proved | `Haft/Foldable.lean` | ✓ | n/a | — |
| `haft.foldable.fold_map_monoid_coherence` | `fold_map(xs ++ ys, f) = fold_map(xs,f).combine(fold_map(ys,f))` (monoid homomorphism) | proved | `Haft/Foldable.lean` | ✓ | n/a | — |
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
