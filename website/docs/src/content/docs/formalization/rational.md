---
title: Rational
description: Field laws, canonical-form invariants, and density for ℚ, machine-checked in Lean against Mathlib's Rat and bound to Rust witnesses.
sidebar:
  order: 6
---

Eight laws for **ℚ**, the field of fractions. Proved in [`lean/DeepCausalityFormal/Rational/`](https://github.com/deepcausality-rs/deep_causality/tree/main/lean/DeepCausalityFormal/Rational) and checked by witness tests in `deep_causality_num_rational/tests/formalization_lean/`.

`Rational<T>` is built over `T: EuclideanDomain`, because reducing a fraction to lowest terms needs the `gcd` that a [Euclidean domain](/formalization/algebra/) supplies. Three groups of facts are pinned: the field laws, the canonical-form invariants, and density.

The carrier correspondence here is unusually tight. Mathlib's `Rat` maintains **exactly the normalisation the Rust type does** — a positive denominator and coprime components — so `Rat.den_pos` and `Rat.reduced` model the Rust invariants directly rather than through a quotient. And because ℚ is exact, the witnesses need no epsilon tolerance: unlike the `Complex<f64>` and `Dual<f64>` witnesses, these are exact equalities, and several sweep a range of inputs rather than checking a single representative.

Every row is `proved` in Lean. The **Lean proof** and **Rust witness** cells give the file and theorem/test name, relative to the directories above.

| id | statement | Lean proof | Rust witness | Test |
|---|---|---|---|---|
| `rational.field.mul_inv` | `q ≠ 0 → q · q⁻¹ = 1`; ℚ is a field — the axiom ℤ cannot satisfy | `Rational.lean :: rational_mul_inv` | `rational_tests.rs :: test_mul_inv` | ✓ |
| `rational.field.mul_comm` | `a·b = b·a` over ℚ | `Rational.lean :: rational_mul_comm` | `rational_tests.rs :: test_mul_comm` | ✓ |
| `rational.field.mul_assoc` | `(a·b)·c = a·(b·c)` over ℚ | `Rational.lean :: rational_mul_assoc` | `rational_tests.rs :: test_mul_assoc` | ✓ |
| `rational.field.distrib` | `a·(b+c) = a·b + a·c` over ℚ | `Rational.lean :: rational_distrib` | `rational_tests.rs :: test_distrib` | ✓ |
| `rational.abelian_group.add_neg` | `a + (−a) = 0`; negation moves the sign to the numerator, leaving the denominator positive | `Rational.lean :: rational_add_neg` | `rational_tests.rs :: test_add_neg` | ✓ |
| `rational.canonical.den_pos` | `0 < q.den`; invariant 1 — a sign never survives in the denominator, which is what lets `Ord` cross-multiply | `Rational.lean :: rational_den_pos` | `rational_tests.rs :: test_denominator_is_positive` | ✓ |
| `rational.canonical.coprime` | `gcd(num, den) = 1`; invariant 2 — uniqueness of the representation, which makes equality structural | `Rational.lean :: rational_coprime` | `rational_tests.rs :: test_numerator_and_denominator_are_coprime` | ✓ |
| `rational.order.dense` | `a < b → a < (a+b)/2 < b`; ℚ has no successor function, the order property separating it from ℤ | `Rational.lean :: rational_dense` | `rational_tests.rs :: test_density` | ✓ |

## Scope

Two edges, stated rather than glossed.

**That ℚ is not analytically closed is not proved here.** `Rational<T>` implements `Field` but deliberately not `Real`, because there is no rational `sqrt(2)`, `exp(1)`, or `ln(2)`. The irrationality of `√2` — the oldest theorem about this gap, and the one that would justify the omission — is `[open]`: it needs a Mathlib module outside the tree-shaken olean set the build fetches. The omission is enforced by the Rust type system instead, where a `Real` bound on `Rational<T>` simply fails to compile.

**Overflow is out of scope.** Lean's ℚ is unbounded; `Rational<T>` is backed by a fixed-width `T`, so `a/b + c/d` can overflow even after the implementation's cross-cancellation. The formalized laws are the algebraic ones and hold of the mathematical values. This is the same boundary as `Float106`, whose real-field model is proved while its bit-exact error bounds remain `[open]`.
