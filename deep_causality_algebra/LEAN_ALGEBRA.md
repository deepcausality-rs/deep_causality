<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Lean Verification Status — `deep_causality_algebra`

Status as of 2026-08-23. This note summarizes the machine-checked formalization of the **algebra
trait tower** — the middle layer of the split numeric stack (`deep_causality_num` ←
`deep_causality_algebra` ← `{num_complex, num_dual, num_rational}`). It is the crate-local view of
the program described in
[`openspec/notes/causal-algebra/Formalization.md`](../openspec/notes/archive/causal-algebra/Formalization.md),
mirroring [`deep_causality_num/LEAN_NUM.md`](../deep_causality_num/LEAN_NUM.md) and
[`deep_causality_core/LEAN_CORE.md`](../deep_causality_core/LEAN_CORE.md).

## Summary

This crate holds no numbers. It holds the *laws* that numbers satisfy, as a tower of traits from
`Magma` up to `Field`, plus the two axes that hang off `CommutativeRing` — `Real` for the analytic
operations and `EuclideanDomain` for exact division. Because the traits are mostly empty markers,
carrying promises the compiler cannot check, the Lean layer is where those promises are actually
discharged.

- **Lean proofs (L1):** 11 files under
  [`lean/DeepCausalityFormal/Algebra/`](../lean/DeepCausalityFormal/Algebra/), one per trait group,
  mirroring the crate's module layout. Every theorem is closed — **zero `sorry`**. The files
  **`import Mathlib`**: each law is stated over Mathlib's canonical algebraic class (`AddMonoid`,
  `Group`, `Ring`, `Field`, `Module`, `StarRing`, `ℤ`) and discharged by the corresponding Mathlib
  lemma. Standalone checking therefore runs in the lake environment, not bare `lean <file>`.
- **Rust witnesses (L2):** one `#[test]` per theorem id under
  [`tests/formalization_lean/`](tests/formalization_lean/), a directory mirroring the Lean tree
  one-to-one (`Ring.lean` ↔ `ring_tests.rs`, `EuclideanDomain.lean` ↔ `domain_euclidean_tests.rs`,
  …). Lean proves ∀ over the carrier; the witness pins the crate's real trait impls to the same
  statement at representative inputs — `f64` for the field, group, ring, module, and scalar laws,
  `i64` for the Euclidean-domain laws, and the boolean/aggregation carriers for the monoid,
  semilattice, and verdict laws.
- **The bridge:** each theorem carries a shared id (e.g. `algebra.euclidean.gcd_nonneg`) recorded
  in [`lean/THEOREM_MAP.md`](../lean/THEOREM_MAP.md) — **40 algebra ids, all proved and
  witnessed**. CI (`.github/workflows/formalization.yml`) runs `lake build`, a guard against
  unproven placeholders, and a consistency gate that fails if any Lean id lacks a tagged Rust file
  or a manifest row. It does not run the witness tests; `cargo llvm-cov --workspace` in
  `rust_coverage.yaml` does.
- **Model fidelity:** the Lean carriers are the canonical structures the traits stand in for. The
  tower's positive claims are proved where the structure holds; equally important, the **negative**
  facts are proved where it does not — `algebra.euclidean.integers_not_field` is what justifies
  withholding `Invertible` from the integers, and so what stops the tower concluding that ℤ is a
  field.

## How to check

```bash
# Lean proofs (from lean/): full Mathlib-backed project build, or a single Algebra file
lake build
lake env lean DeepCausalityFormal/Algebra/EuclideanDomain.lean

# Rust witnesses (one #[test] per theorem id)
cargo test -p deep_causality_algebra --test mod formalization_lean

# Whole workspace (much faster than cargo across all crates)
bazel test //...
```

## Verified correct as documented

| Mechanism (id) | Reference | Status |
|---|---|---|
| Monoid `algebra.add_monoid.assoc`, `algebra.add_monoid.identity`, `algebra.monoid.assoc`, `algebra.monoid.left_id`, `algebra.monoid.right_id` — associativity and two-sided identity for `AddMonoid` and the generic `Monoid` | Mathlib `AddMonoid`/`Monoid` | proved & witnessed |
| Commutative monoid & semilattice `algebra.commutative_monoid.comm`, `algebra.semilattice.{idempotent,assoc,comm}` — commutativity, and the idempotent ∧-semilattice behind `Conjunction`/`Disjunction` | Mathlib `CommMonoid`, `SemilatticeInf` | proved & witnessed |
| Group `algebra.group.mul_inv`, `algebra.add_group.neg_cancel`, `algebra.abelian_group.add_comm` — inverses and commutativity for `AddGroup`/`AbelianGroup` | Mathlib `Group`/`AddGroup`/`AddCommGroup` | proved & witnessed |
| Ring `algebra.ring.{mul_assoc,left_distrib,right_distrib}`, `algebra.commutative_ring.mul_comm` — the `Ring`/`CommutativeRing` laws behind the `Associative`/`Distributive`/`Commutative` markers | Mathlib `Ring`/`CommRing` | proved & witnessed |
| Field `algebra.field.{mul_inv_cancel,inv_mul_cancel}`, `algebra.real_field.mul_pos` — two-sided inverses away from zero, and the ordered-field sign rule | Mathlib `mul_inv_cancel₀`, `inv_mul_cancel₀`, `mul_pos` | proved & witnessed |
| Module & algebra `algebra.module.{one_smul,mul_smul,add_smul,smul_add}`, `algebra.algebra.{mul_smul_comm,smul_mul_assoc}` — the scalar-action laws | Mathlib `Module`, `Algebra` | proved & witnessed |
| Conjugation & norm `algebra.conjugate.{star_star,star_add,star_mul}`, `algebra.normed.{norm_mul,norm_nonneg}`, `algebra.division_algebra.mul_inv` — the involution and norm laws behind `ConjugateScalar`/`Normed`/`DivisionAlgebra` | Mathlib `StarRing`, `NormedField` | proved & witnessed |
| Verdict `algebra.verdict.lattice_laws`, `algebra.verdict.complement` — the boolean verdict lattice and its complement, plus the MV-algebra complement `1−p` for `Prob` | Mathlib `BooleanAlgebra` | proved & witnessed |
| Euclidean domain `algebra.euclidean.{remainder_nonneg,remainder_lt_divisor}` — `rem_euclid` is non-negative, and strictly below a positive divisor. The second is the **termination argument** for the Euclidean algorithm: φ is ℕ-valued and strictly decreases, so the recursion is well founded | Mathlib `Int.emod_nonneg`, `Int.emod_lt_of_pos` | proved & witnessed |
| Euclidean gcd `algebra.euclidean.{gcd_dvd_left,gcd_dvd_right,gcd_nonneg,gcd_zero_right}` — the gcd divides both arguments, is non-negative whatever their signs, and bottoms out at `gcd(a,0) = \|a\|` | Mathlib `Int.gcd_dvd_left`, `Int.gcd_dvd_right`, `Int.gcd_zero_right` | proved & witnessed |
| `algebra.euclidean.integers_not_field` — `¬∃ x : ℤ, 2x = 1`. The negative fact that keeps ℤ at `CommutativeRing`: integer `/` truncates rather than inverts, so the `Invertible` marker is withheld | Mathlib `Int`, discharged by `omega` | proved & witnessed |

## Outstanding issues

1. **`gcd_nonneg` found a real defect, and the trait changed to satisfy it.** Writing the witness
   for `algebra.euclidean.gcd_nonneg` showed that the Euclidean algorithm as first implemented
   returned `gcd(-7, 0) = -7` and `gcd(-24, -12) = -12`: the loop exits holding whichever seed ran
   last, so the result inherited that seed's sign. A gcd is only defined **up to associates**, so
   the fix was to make the choice of representative explicit — `EuclideanDomain::normalize`, the
   canonical associate (absolute value over ℤ, monic over `F[x]`). This is the intended value of
   the L1/L2 bridge: the Lean statement is a specification, and the witness is what discovers that
   the implementation does not meet it.
2. **Laws are proved per canonical carrier, not per generic instance.** The Lean carriers are
   Mathlib's classes and `ℤ`; the Rust witnesses check representative concrete types (`f64`, `i64`,
   the boolean carriers). Extending the model to every type the traits are implemented for is
   mechanical scaling work — Lean proves ∀ over the carrier, the witnesses pin the shipped
   instances at representative inputs.
3. **The remainder bound is stated for a positive divisor.** `algebra.euclidean.remainder_lt_divisor`
   is `0 < b → a % b < b` rather than the general `b ≠ 0 → a % b < |b|`. This is the case the
   algorithm actually runs in — every divisor after the first step is a previous remainder, and
   `rem_euclid` is non-negative — so it is the bound termination needs. The general form requires a
   Mathlib module outside the current tree-shaken olean set (see `cache_roots` in `//MODULE.bazel`).
4. **ℕ has no algebraic slot yet.** The unsigned integers satisfy the `Commutative`/`Associative`/
   `Distributive` markers but stop before `AbelianGroup`, correctly — ℕ has no additive inverses.
   A `CommutativeSemiring` level and the matching Lean file are not yet written, so ℕ's membership
   is currently recorded only by the three markers and by
   `deep_causality_algebra/tests/algebra/integer_tower_tests.rs`.
