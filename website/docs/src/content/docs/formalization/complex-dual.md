---
title: Complex & Dual
description: Complex, quaternion, and dual-number laws (field, division ring, conjugation, norm, Leibniz product rule), machine-checked in Lean and bound to Rust law-tests.
sidebar:
  order: 5
---

Fifteen laws for the number types layered on the algebra tower: `ℂ` as a field with involutive conjugation and multiplicative norm, `ℍ` as a division ring with a non-commutativity witness, and the dual numbers `R[ε]` with `ε² = 0` and the forward-mode Leibniz product rule. Proved in [`lean/DeepCausalityFormal/`](https://github.com/deepcausality-rs/deep_causality/tree/main/lean/DeepCausalityFormal) (the `Complex/` and `Dual/` subdirectories) and checked by law-tests in `deep_causality_num_complex/tests/formalization_lean/` (complex, quaternion) and `deep_causality_num_dual/tests/formalization_lean/` (dual).

Every row is `proved` in Lean. The **Lean proof** cells are relative to `lean/DeepCausalityFormal/`; the **Rust witness** cells give the test file and name inside the witness directories above.

| id | statement | Lean proof | Rust witness | Test |
|---|---|---|---|---|
| `complex.field.mul_inv` | `z ≠ 0 → z * z⁻¹ = 1` (ℂ is a field) | `Complex/Complex.lean :: complex_field_mul_inv` | `complex_tests.rs :: test_complex_field_mul_inv` | ✓ |
| `complex.conj.involutive` | `conj (conj z) = z` | `Complex/Complex.lean :: complex_conj_involutive` | `complex_tests.rs :: test_complex_conj_involutive` | ✓ |
| `complex.conj.mul` | `conj (z*w) = conj z * conj w` | `Complex/Complex.lean :: complex_conj_mul` | `complex_tests.rs :: test_complex_conj_mul` | ✓ |
| `complex.norm_sq.mul` | `normSq (z*w) = normSq z * normSq w` | `Complex/Complex.lean :: complex_normSq_mul` | `complex_tests.rs :: test_complex_norm_sqr_mul` | ✓ |
| `complex.norm.mul` | `‖z*w‖ = ‖z‖*‖w‖` | `Complex/Complex.lean :: complex_norm_mul` | `complex_tests.rs :: test_complex_norm_mul` | ✓ |
| `quaternion.division_ring.mul_inv` | `q ≠ 0 → q * q⁻¹ = 1` (ℍ is a division ring) | `Complex/Quaternion.lean :: quaternion_mul_inv` | `quaternion_tests.rs :: test_quaternion_division_ring_mul_inv` | ✓ |
| `quaternion.norm_sq.mul` | `normSq (q*p) = normSq q * normSq p` | `Complex/Quaternion.lean :: quaternion_normSq_mul` | `quaternion_tests.rs :: test_quaternion_norm_sqr_mul` | ✓ |
| `quaternion.conj.mul` | `star (q*p) = star p * star q` | `Complex/Quaternion.lean :: quaternion_conj_mul` | `quaternion_tests.rs :: test_quaternion_conj_mul` | ✓ |
| `quaternion.noncomm` | `∃ q p, q*p ≠ p*q` (ℍ is non-commutative) | `Complex/Quaternion.lean :: quaternion_noncomm` | `quaternion_tests.rs :: test_quaternion_noncomm` | ✓ |
| `dual.comm_ring.mul_comm` | `a*b = b*a` for `R[ε]` (commutative ring) | `Dual/Dual.lean :: dual_comm_ring_mul_comm` | `dual_tests.rs :: test_mul_comm` | ✓ |
| `dual.eps_sq_zero` | `ε * ε = 0` | `Dual/Dual.lean :: dual_eps_sq_zero` | `dual_tests.rs :: test_eps_sq_zero` | ✓ |
| `dual.real_projection.add` | `fst (a+b) = fst a + fst b` (the value is additive) | `Dual/Dual.lean :: dual_fst_is_ring_hom_add` | `dual_tests.rs :: test_real_projection_add` | ✓ |
| `dual.real_projection.mul` | `fst (a*b) = fst a * fst b` (the value multiplies) | `Dual/Dual.lean :: dual_fst_mul` | `dual_tests.rs :: test_real_projection_mul` | ✓ |
| `dual.leibniz.product_rule` | `snd (a*b) = fst a * snd b + snd a * fst b` (forward-mode AD product rule) | `Dual/Dual.lean :: dual_leibniz` | `dual_tests.rs :: test_leibniz_product_rule` | ✓ |
| `dual.not_field.zero_divisor` | `ε ≠ 0 ∧ ε*ε = 0` (a nonzero zero-divisor; `R[ε]` is not a field) | `Dual/Dual.lean :: dual_not_field_zero_divisor` | `dual_tests.rs :: test_not_field_zero_divisor` | ✓ |
