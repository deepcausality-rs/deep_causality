---
title: Linear
description: Rank-nullity over 𝔽₂ and the three facts the Betti-number computation rests on, machine-checked in Lean against Mathlib and bound to Rust witnesses on the bit-packed matrix.
sidebar:
  order: 7
---

Four laws for the 𝔽₂ layer of [`deep_causality_linear`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_linear). Proved in [`lean/DeepCausalityFormal/Linear/RankNullity.lean`](https://github.com/deepcausality-rs/deep_causality/blob/main/lean/DeepCausalityFormal/Linear/RankNullity.lean) and checked by witness tests in `deep_causality_linear/tests/formalization_lean/rank_nullity_tests.rs`.

## Why this layer, and why 𝔽₂

One statement is load-bearing and invisible, which is why it was chosen.

`ChainComplex::betti_number_over` in `deep_causality_topology` computes

```
β_k = (n_k − rank ∂_k) − rank ∂_{k+1}
```

from two ranks that this crate's elimination produces and one cell count. It never builds a kernel, never builds a quotient, and never checks that the first bracket is a nullity. It substitutes `n_k − rank ∂_k` for `dim ker ∂_k` and moves on. That substitution is rank-nullity, it is the step that makes the whole homology path correct, and nothing in either crate's source states it.

𝔽₂ is the case formalized because it is the one where the coefficient ring being a field is a *choice the caller makes* rather than a given. Rank-nullity holds over ℚ for the same reason, and the characteristic-zero path is covered by tests.

| id | statement | Lean proof | Rust witness | Test |
|---|---|---|---|---|
| `linear.gf2.rank_nullity` | `rank ∂ + dim ker ∂ = n` over 𝔽₂ | `RankNullity.lean :: gf2_rank_nullity` | `rank_nullity_tests.rs :: test_gf2_rank_nullity` | ✓ |
| `linear.gf2.nullity_is_count_minus_rank` | `dim ker ∂ = n − rank ∂`; the substitution `betti_number_over` performs, computing a nullity it never materialises | `RankNullity.lean :: gf2_nullity_is_count_minus_rank` | `rank_nullity_tests.rs :: test_gf2_nullity_is_count_minus_rank` | ✓ |
| `linear.gf2.rank_le_cell_count` | `rank ∂ ≤ n`; the `saturating_sub` floor in `betti_number_over` is never reached at that step | `RankNullity.lean :: gf2_rank_le_cell_count` | `rank_nullity_tests.rs :: test_gf2_rank_le_cell_count` | ✓ |
| `linear.gf2.betti_from_ranks` | `dim H_k = (n_k − rank ∂_k) − rank ∂_{k+1}` over 𝔽₂; what `ChainComplex::betti_number_over` computes is the dimension of mod-2 homology | `RankNullity.lean :: gf2_betti_from_ranks` | `rank_nullity_tests.rs :: test_gf2_betti_from_ranks` | ✓ |

Every theorem is closed with no `sorry`, and `#print axioms` reports only `propext`, `Classical.choice`, and `Quot.sound` for each. The file is Mathlib-backed, so it is checked by `lake build` rather than standalone.

## The witnesses compute both sides independently

Lean proves the statement for all of `Matrix (Fin m) (Fin n) (ZMod 2)`. Each Rust witness pins `PackedGf2<u64>` to the same statement across nine matrices: square, wide, tall, zero, full-rank, and the even-weight dependency whose rank differs between ℚ and 𝔽₂.

The two sides are computed by **different routines**. The rank comes from elimination (`rank_gf2`); the nullity comes from counting the columns of `kernel_basis_gf2`. The identity is therefore a claim about two independent computations agreeing rather than an algebraic rearrangement of one.

## Model fidelity

The Lean carrier is `ZMod 2`, which Mathlib makes a `Field` exactly when 2 is prime. The Rust carrier is `deep_causality_num::Gf2`, which implements `IntegralDomain` directly and reaches `Field` through the tower's blanket implementation, sound for the same reason. Mathlib's `Matrix.rank` is `finrank` of the range of `mulVecLin`, which is the number `rank_gf2` reaches by counting pivots.

## Scope

Three edges, stated rather than glossed.

**The characteristic-zero path is not formalized.** `HomologyField::Rational` runs fraction-free elimination over ℤ. Rank-nullity holds there for the same reason, since rank is a fraction-field notion, and the Rust tests cover it.

**Termination and complexity of the eliminations are out of scope.** Both are bounded and mechanical, and the tests cover them.

**The dense decompositions are out of scope.** QR, the Hermitian eigenvalue sweep, and the one-sided Jacobi SVD are floating-point iterations whose properties are approximate and quantified over a tolerance. That is not what this layer is for; they are checked against reference values in the test suite instead.

## Related reading

- [`LEAN_LINEAR.md`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality_linear/LEAN_LINEAR.md): the crate-local view of this layer.
- [Uniform Math](/concepts/uniform-math/): where `deep_causality_linear` sits in the math stack.
- [Topology](/formalization/topology/): the curvature laws proved at the concrete `CurvatureTensor`.
