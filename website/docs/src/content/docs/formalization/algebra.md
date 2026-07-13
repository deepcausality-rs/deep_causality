---
title: Algebra
description: Abstract-algebra trait-tower laws (group, ring, field, module, algebra, conjugation, norm), machine-checked in Lean and bound to Rust law-tests.
sidebar:
  order: 2
---

Thirty-three laws for the abstract-algebra trait tower: monoid and commutative-monoid, group and abelian-group, ring and commutative-ring, field and real-field, module and algebra over a ring, division algebra, conjugation (`star`), norm, semilattice, and the verdict lattice. Proved in [`lean/DeepCausalityFormal/Algebra/`](https://github.com/deepcausality-rs/deep_causality/tree/main/lean/DeepCausalityFormal/Algebra) and checked by law-tests in `deep_causality_algebra/tests/formalization_lean/`. These are the laws the [Uniform Math](/concepts/uniform-math/) surface relies on.

Every row is `proved` in Lean. The **Lean proof** and **Rust witness** cells give the file and theorem/test name, relative to the directories above.

| id | statement | Lean proof | Rust witness | Test |
|---|---|---|---|---|
| `algebra.add_monoid.assoc` | `(a+b)+c = a+(b+c)` for `AddMonoid` | `Monoid.lean :: add_monoid_assoc` | `monoid_tests.rs :: test_add_monoid_assoc` | ✓ |
| `algebra.add_monoid.identity` | `a+0 = a ∧ 0+a = a` for `AddMonoid` | `Monoid.lean :: add_monoid_identity` | `monoid_tests.rs :: test_add_monoid_identity` | ✓ |
| `algebra.monoid.left_id` | `empty().combine(x) = x` for the generic `Monoid` (Mathlib `one_mul`) | `MonoidGeneric.lean :: monoid_left_id` | `monoid_generic_tests.rs :: test_generic_monoid_laws` | ✓ |
| `algebra.monoid.right_id` | `x.combine(empty()) = x` (Mathlib `mul_one`) | `MonoidGeneric.lean :: monoid_right_id` | `monoid_generic_tests.rs :: test_generic_monoid_laws` | ✓ |
| `algebra.monoid.assoc` | `combine` associativity (Mathlib `mul_assoc`) | `MonoidGeneric.lean :: monoid_assoc` | `monoid_generic_tests.rs :: test_generic_monoid_laws` | ✓ |
| `algebra.commutative_monoid.comm` | `x.combine(y) = y.combine(x)` for `CommutativeMonoid` (Mathlib `mul_comm`) | `CommutativeMonoid.lean :: commutative_monoid_comm` | `commutative_monoid_tests.rs :: test_commutative_monoid_comm` | ✓ |
| `algebra.group.mul_inv` | `a * a⁻¹ = 1` for `Group` | `Group.lean :: group_mul_inv` | `group_tests.rs :: test_group_mul_inv` | ✓ |
| `algebra.add_group.neg_cancel` | `-a + a = 0` for `AddGroup` | `Group.lean :: add_group_neg_cancel` | `group_tests.rs :: test_add_group_neg_cancel` | ✓ |
| `algebra.abelian_group.add_comm` | `a + b = b + a` for `AbelianGroup` | `Group.lean :: abelian_group_add_comm` | `group_tests.rs :: test_abelian_group_add_comm` | ✓ |
| `algebra.ring.left_distrib` | `a*(b+c) = a*b + a*c` for `Ring` | `Ring.lean :: ring_left_distrib` | `ring_tests.rs :: test_ring_left_distrib` | ✓ |
| `algebra.ring.right_distrib` | `(a+b)*c = a*c + b*c` for `Ring` | `Ring.lean :: ring_right_distrib` | `ring_tests.rs :: test_ring_right_distrib` | ✓ |
| `algebra.ring.mul_assoc` | `(a*b)*c = a*(b*c)` for `AssociativeRing` | `Ring.lean :: ring_mul_assoc` | `ring_tests.rs :: test_ring_mul_assoc` | ✓ |
| `algebra.commutative_ring.mul_comm` | `a*b = b*a` for `CommutativeRing` | `Ring.lean :: commutative_ring_mul_comm` | `ring_tests.rs :: test_commutative_ring_mul_comm` | ✓ |
| `algebra.field.mul_inv_cancel` | `a ≠ 0 → a * a⁻¹ = 1` for `Field` | `Field.lean :: field_mul_inv_cancel` | `field_tests.rs :: test_field_mul_inv_cancel` | ✓ |
| `algebra.field.inv_mul_cancel` | `a ≠ 0 → a⁻¹ * a = 1` for `Field` | `Field.lean :: field_inv_mul_cancel` | `field_tests.rs :: test_field_inv_mul_cancel` | ✓ |
| `algebra.real_field.mul_pos` | `0<a → 0<b → 0<a*b` for the ordered `RealField` | `Field.lean :: real_field_mul_pos` | `field_tests.rs :: test_real_field_mul_pos` | ✓ |
| `algebra.module.smul_add` | `r•(x+y) = r•x + r•y` for `Module` | `Module.lean :: module_smul_add` | `module_tests.rs :: test_module_smul_add` | ✓ |
| `algebra.module.add_smul` | `(r+s)•x = r•x + s•x` for `Module` | `Module.lean :: module_add_smul` | `module_tests.rs :: test_module_add_smul` | ✓ |
| `algebra.module.one_smul` | `1•x = x` for `Module` | `Module.lean :: module_one_smul` | `module_tests.rs :: test_module_one_smul` | ✓ |
| `algebra.module.mul_smul` | `(r*s)•x = r•(s•x)` for `Module` | `Module.lean :: module_mul_smul` | `module_tests.rs :: test_module_mul_smul` | ✓ |
| `algebra.algebra.smul_mul_assoc` | `r•(a*b) = (r•a)*b` for `Algebra` over a ring | `Module.lean :: algebra_smul_mul_assoc` | `module_tests.rs :: test_algebra_smul_mul_assoc` | ✓ |
| `algebra.algebra.mul_smul_comm` | `r•(a*b) = a*(r•b)` for `Algebra` over a ring | `Module.lean :: algebra_mul_smul_comm` | `module_tests.rs :: test_algebra_mul_smul_comm` | ✓ |
| `algebra.division_algebra.mul_inv` | `a ≠ 0 → a * a⁻¹ = 1` for `DivisionAlgebra` | `DivisionAlgebra.lean :: division_algebra_mul_inv` | `division_algebra_tests.rs :: test_division_algebra_mul_inv` | ✓ |
| `algebra.conjugate.star_star` | `star (star a) = a` for `ConjugateScalar` | `Scalar.lean :: conjugate_star_star` | `scalar_tests.rs :: test_conjugate_star_star` | ✓ |
| `algebra.conjugate.star_mul` | `star (a*b) = star b * star a` for `ConjugateScalar` | `Scalar.lean :: conjugate_star_mul` | `scalar_tests.rs :: test_conjugate_star_mul` | ✓ |
| `algebra.conjugate.star_add` | `star (a+b) = star a + star b` for `ConjugateScalar` | `Scalar.lean :: conjugate_star_add` | `scalar_tests.rs :: test_conjugate_star_add` | ✓ |
| `algebra.normed.norm_mul` | `‖a*b‖ = ‖a‖*‖b‖` for `Normed` | `Scalar.lean :: normed_norm_mul` | `scalar_tests.rs :: test_normed_norm_mul` | ✓ |
| `algebra.normed.norm_nonneg` | `0 ≤ ‖a‖` for `Normed` | `Scalar.lean :: normed_norm_nonneg` | `scalar_tests.rs :: test_normed_norm_nonneg` | ✓ |
| `algebra.semilattice.idempotent` | `x.combine(x) = x` for the boolean ∧-semilattice (`Conjunction`) | `CommutativeMonoid.lean :: semilattice_idempotent` | `commutative_monoid_tests.rs :: test_semilattice_idempotent` | ✓ |
| `algebra.semilattice.assoc` | associativity of the boolean ∧-semilattice | `CommutativeMonoid.lean :: semilattice_assoc` | `commutative_monoid_tests.rs :: test_semilattice_assoc_and_comm` | ✓ |
| `algebra.semilattice.comm` | commutativity of the boolean ∧-semilattice | `CommutativeMonoid.lean :: semilattice_comm` | `commutative_monoid_tests.rs :: test_semilattice_assoc_and_comm` | ✓ |
| `algebra.verdict.lattice_laws` | boolean verdict lattice: meet commutativity + absorption | `Verdict.lean :: verdict_meet_comm / verdict_absorption` | `verdict_tests.rs :: test_verdict_lattice_laws` | ✓ |
| `algebra.verdict.complement` | complement involution + De Morgan (Boolean); MV-algebra complement `1−p` (Prob) | `Verdict.lean :: verdict_compl_compl / verdict_de_morgan` | `verdict_tests.rs :: test_verdict_complement` | ✓ |
