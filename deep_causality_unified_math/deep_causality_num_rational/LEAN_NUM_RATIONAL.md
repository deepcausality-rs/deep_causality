<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Lean Verification Status — `deep_causality_num_rational`

Status as of 2026-08-23. This note summarizes the machine-checked formalization of ℚ — the field of
fractions of an integral domain, and the exact-arithmetic member of the numeric tower. It is the
crate-local view of the program described in
[`openspec/notes/causal-algebra/Formalization.md`](../openspec/notes/archive/causal-algebra/Formalization.md),
mirroring [`deep_causality_unified_math/deep_causality_num_dual/LEAN_NUM_DUAL.md`](../deep_causality_num_dual/LEAN_NUM_DUAL.md)
and [`deep_causality_unified_math/deep_causality_num_complex/LEAN_NUM_COMPLEX.md`](../deep_causality_num_complex/LEAN_NUM_COMPLEX.md).

## Summary

`Rational<T>` is built over `T: EuclideanDomain`, because reducing a fraction to lowest terms needs
a gcd and a Euclidean domain is what supplies one. Three groups of facts are formalized: the field
laws, the canonical-form invariants, and density.

- **Lean proofs (L1):** one file,
  [`lean/DeepCausalityFormal/Rational/Rational.lean`](../lean/DeepCausalityFormal/Rational/Rational.lean),
  carrying **8 theorems**. Every theorem is closed — **zero `sorry`**. The file is **Mathlib-backed**:
  the laws are stated on Mathlib's `ℚ` (`import Mathlib.Data.Rat.Cast.Defs`,
  `Mathlib.Algebra.Field.Basic`, `Mathlib.Tactic.Linarith`), so it is checked as part of
  `lake build` — or `bazel test //lean:Rational` — not standalone with bare `lean`.
- **Rust witnesses (L2):** one `#[test]` per theorem id under
  [`tests/formalization_lean/rational_tests.rs`](tests/formalization_lean/rational_tests.rs). Lean
  proves ∀ over Mathlib's ℚ; the witness pins the crate's `Rational<i64>` to the same statement.
  Because ℚ is exact, the witnesses need no epsilon tolerance — unlike the `Complex<f64>` and
  `Dual<f64>` witnesses, these are exact equalities, and several sweep a range of inputs rather
  than checking a single representative.
- **The bridge:** each theorem carries a shared id (e.g. `rational.canonical.coprime`) recorded in
  [`lean/THEOREM_MAP.md`](../lean/THEOREM_MAP.md) — **8 rational ids, all proved and witnessed**.
  CI (`.github/workflows/formalization.yml`) runs `lake build`, a guard against unproven
  placeholders, and a consistency gate that fails if any Lean id lacks a tagged Rust file or a
  manifest row. It does not run the witness tests; `cargo llvm-cov --workspace` in
  `rust_coverage.yaml` does.
- **Model fidelity:** the Lean carrier is Mathlib's `Rat`, which maintains **exactly the
  normalisation the Rust type does** — `Rat.den_pos` and `Rat.reduced` are the model of the crate's
  invariants 1 and 2 (positive denominator, coprime components). That correspondence is unusually
  tight: the two implementations agree not just on the values but on the representation, so the
  canonical-form theorems transfer directly rather than through a quotient. The crate carries a
  fourth invariant with **no** counterpart in the model — the numerator is never `T::MIN` — which
  is a fixed-width artifact rather than a fact about ℚ: Mathlib's `Rat` is unbounded and has
  nothing to exclude. It is what makes the `rational.abelian_group.add_neg` witness hold at every
  representable input rather than at all but one, since `-T::MIN` does not fit.

## How to check

```bash
# Lean proofs (from lean/): full project build, or just this namespace
lake build
lake env lean DeepCausalityFormal/Rational/Rational.lean
bazel test //lean:Rational

# Rust witnesses (one #[test] per theorem id)
cargo test -p deep_causality_num_rational --test mod formalization_lean

# Whole workspace (much faster than cargo across all crates)
bazel test //...
```

## Verified correct as documented

| Mechanism (id) | Reference | Status |
|---|---|---|
| `rational.field.mul_inv` — `q ≠ 0 → q · q⁻¹ = 1`; ℚ is a field. The axiom ℤ cannot satisfy, and what the `Invertible` marker records | Mathlib `mul_inv_cancel₀` | proved & witnessed |
| `rational.field.mul_comm` — `a·b = b·a`; the `Commutative` marker | Mathlib `mul_comm` | proved & witnessed |
| `rational.field.mul_assoc` — `(a·b)·c = a·(b·c)`; the `Associative` marker | Mathlib `mul_assoc` | proved & witnessed |
| `rational.field.distrib` — `a·(b+c) = a·b + a·c`; the `Distributive` marker | Mathlib `mul_add` | proved & witnessed |
| `rational.abelian_group.add_neg` — `a + (−a) = 0`; the `AbelianGroup` marker, and the `Neg` impl that negates the numerator while leaving the denominator positive | Mathlib `add_neg_cancel` | proved & witnessed |
| `rational.canonical.den_pos` — `0 < q.den`; invariant 1. A sign never survives in the denominator, which is what lets `Ord` cross-multiply without tracking a sign flip | Mathlib `Rat.den_pos` | proved & witnessed |
| `rational.canonical.coprime` — `gcd(num, den) = 1`; invariant 2. Uniqueness of the representation is what makes equality structural rather than a cross-multiplication | Mathlib `Rat.reduced` | proved & witnessed |
| `rational.order.dense` — `a < b → a < (a+b)/2 < b`; ℚ has no successor function, the order property separating it from ℤ | Mathlib, discharged by `linarith` | proved & witnessed |

## Outstanding issues

1. **That ℚ is not analytically closed is not proved in Lean.** ℚ implements `Field` but
   deliberately not [`Real`](deep_causality_algebra::Real), because there is no rational `sqrt(2)`,
   `exp(1)`, or `ln(2)`. The irrationality of `√2` — the oldest theorem about this gap, and the one
   that would justify the omission — is `[open]` here: it needs a Mathlib module outside the
   tree-shaken olean set (see `cache_roots` in `//MODULE.bazel`). The omission is currently enforced
   by the Rust type system instead, where `Real for Rational<T>` is simply not implemented and a
   `Real` bound on it fails to compile.
2. **Overflow is out of L1 scope.** Lean's ℚ is unbounded; the crate's `Rational<T>` is backed by a
   fixed-width `T`, so `a/b + c/d` can overflow even after the implementation's cross-cancellation
   and integer-part split. The formalized laws are the algebraic ones and hold of the mathematical
   values; the fixed-width behaviour is covered by
   `tests/rational/rational_number/arithmetic_tests.rs` and `construction_tests.rs` only, where the
   edge cases are cross-checked against a wider integer width rather than proved. Construction,
   comparison, and negation *are* total — see the `Overflow` section on `Rational` for which
   operations are which — but that totality is tested, not machine-checked. This is the same
   boundary as `Float106`, whose real-field model is proved while its bit-exact error bounds are
   `[open]`.
3. **Laws are proved on Mathlib's `Rat`, not on `Rational<T>` itself.** L1 states the laws over
   Mathlib's ℚ; the bridge to the crate's `Rational<i64>` is the L2 witness tests. Unlike the float
   -backed crates the witnesses are exact rather than epsilon-tolerant, but they remain checks at
   representative inputs, not a Lean-level identification of the two carriers.
