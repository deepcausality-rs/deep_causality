---
title: Num
description: Machine-checked identity, integer ring, cast, and Float106 real-field laws, each bound to a Rust law-test in deep_causality_num.
sidebar:
  order: 1
---

Ten laws for the numeric foundation, proved in [`lean/DeepCausalityFormal/Num/`](https://github.com/deepcausality-rs/deep_causality/tree/main/lean/DeepCausalityFormal/Num) and checked by law-tests in `deep_causality_num/tests/formalization_lean/`. These types underpin the [Unified Math](/concepts/uniform-math/) surface.

Every row is `proved` in Lean. The **Lean proof** and **Rust witness** cells give the file and theorem/test name, relative to the directories above.

| id | statement | Lean proof | Rust witness | Test |
|---|---|---|---|---|
| `num.one.identity` | `1*a = a ∧ a*1 = a` (the `One` identity) | `Identity.lean :: one_is_identity` | `identity_tests.rs :: test_one_identity` | ✓ |
| `num.zero.identity` | `0+a = a ∧ a+0 = a` (the `Zero` identity) | `Identity.lean :: zero_is_identity` | `identity_tests.rs :: test_zero_identity` | ✓ |
| `num.integer.mul_comm` | `a*b = b*a` over `ℤ` | `Integer.lean :: integer_mul_comm` | `integer_tests.rs :: test_integer_mul_comm` | ✓ |
| `num.integer.distrib` | `a*(b+c) = a*b + a*c` over `ℤ` | `Integer.lean :: integer_left_distrib` | `integer_tests.rs :: test_integer_distrib` | ✓ |
| `num.integer.euclidean` | `b*(a/b) + a%b = a` over `ℤ` (Euclidean division) | `Integer.lean :: integer_euclidean` | `integer_tests.rs :: test_integer_euclidean` | ✓ |
| `num.cast.nat_int_roundtrip` | `((n:ℤ)).toNat = n` (ℕ↔ℤ cast round-trip) | `Cast.lean :: cast_nat_int_roundtrip` | `cast_tests.rs :: test_cast_nat_int_roundtrip` | ✓ |
| `num.cast.int_injective` | `ℤ → ℚ` cast is injective | `Cast.lean :: cast_int_injective` | `cast_tests.rs :: test_cast_int_injective` | ✓ |
| `num.float106.model.add_comm` | `a+b = b+a` (real-field model of the `Float106` double-double; bit-exact bounds are `[open]`) | `Float106.lean :: float106_model_add_comm` | `float106_tests.rs :: test_float106_add_comm` | ✓ |
| `num.float106.model.mul_comm` | `a*b = b*a` (real-field model of `Float106`) | `Float106.lean :: float106_model_mul_comm` | `float106_tests.rs :: test_float106_mul_comm` | ✓ |
| `num.float106.model.distrib` | `a*(b+c) = a*b + a*c` (real-field model of `Float106`) | `Float106.lean :: float106_model_distrib` | `float106_tests.rs :: test_float106_distrib` | ✓ |
