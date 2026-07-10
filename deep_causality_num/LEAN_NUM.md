<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Lean Verification Status — `deep_causality_num`

Status as of 2026-07-10. This note summarizes the machine-checked formalization of the **num-core**
crate — the base of the split numeric tower (`deep_causality_num` ← `deep_causality_algebra` ←
`{num_complex, num_dual}`). It is the crate-local view of the program described in
[`openspec/notes/causal-algebra/Formalization.md`](../openspec/notes/causal-algebra/Formalization.md),
mirroring [`deep_causality_core/LEAN_CORE.md`](../deep_causality_core/LEAN_CORE.md) and
[`deep_causality_haft/LEAN_HAFT.md`](../deep_causality_haft/LEAN_HAFT.md).

## Summary

The load-bearing numeric contracts of this crate — the identity markers, the integer ring, the
primitive-cast conversions, and the `Float106` double-double type — are formalized in Lean 4 and
linked back to the Rust implementation by a per-theorem witness test:

- **Lean proofs (L1):** 4 files under
  [`lean/DeepCausalityFormal/Num/`](../lean/DeepCausalityFormal/Num/) — `Identity.lean`,
  `Integer.lean`, `Cast.lean`, `Float106.lean`, one per numeric surface, mirroring the crate's
  module layout. Every theorem is closed — **zero `sorry`**. Unlike the self-contained Core/Haft
  layers, these files **`import Mathlib`** (v4.15.0): the laws are near-inherited from Mathlib's
  `Monoid`/`AddMonoid` classes and its `Int`/`Rat`/`Real` lemmas, and each proof is a one-line term
  discharged by the corresponding Mathlib lemma. Standalone single-file checking therefore runs in
  the lake environment (Mathlib on the search path), not bare `lean <file>`.
- **Rust witnesses (L2):** one `#[test]` per theorem id under
  [`tests/formalization_lean/`](tests/formalization_lean/), a directory that mirrors the Lean tree
  (`Num/Identity.lean` ↔ `identity_tests.rs`, `Num/Cast.lean` ↔ `cast_tests.rs`, …). Lean proves ∀;
  the witness pins the actual Rust implementation to the same statement at representative inputs
  (`i64`/`usize`/`f64` for the integer, identity, and cast laws; `Float106` limbs compared within a
  tight epsilon for the model laws).
- **The bridge:** each theorem carries a shared id (e.g. `num.integer.euclidean`) recorded in
  [`lean/THEOREM_MAP.md`](../lean/THEOREM_MAP.md) — **10 num ids, all proved and witnessed**. CI
  (`.github/workflows/formalization.yml`) runs `lake build`, the witness tests, and a consistency
  gate that fails if any Lean id lacks a Rust witness or a manifest row.
- **Model fidelity:** the Lean carriers are the canonical mathematical structures the crate's traits
  stand in for — the additive/multiplicative identity monoids for the `Zero`/`One` markers, the
  commutative ring with Euclidean division over Mathlib `ℤ` for `Integer`, the `ℕ ↔ ℤ` round-trip
  and the injective `ℤ ↪ ℚ` characteristic-zero cast for `FromPrimitive`/`ToPrimitive`/`NumCast`,
  and the ordered-field laws of `ℝ` as the real-number model of the `Float106` double-double.

## How to check

```bash
# Lean proofs (from lean/): full Mathlib-backed project build, or a single Num file in the lake env
lake build
lake env lean DeepCausalityFormal/Num/Float106.lean

# Rust witnesses (one #[test] per theorem id)
cargo test -p deep_causality_num --test mod formalization_lean

# Whole workspace (much faster than cargo across all crates)
bazel test //...
```

## Verified correct as documented

| Mechanism (id) | Reference | Status |
|---|---|---|
| Identity markers `num.zero.identity`, `num.one.identity` — two-sided additive/multiplicative identity (`0+a = a ∧ a+0 = a`, `1*a = a ∧ a*1 = a`) for the `Zero`/`One` markers | Mathlib `AddMonoid`/`Monoid` (`zero_add`/`add_zero`, `one_mul`/`mul_one`) | proved & witnessed |
| Integer ring `num.integer.mul_comm`, `num.integer.distrib`, `num.integer.euclidean` — multiplication commutes, left distributivity, and the Euclidean reconstruction `b*(a/b) + a%b = a` over `ℤ` for the `Integer` trait | Mathlib `Int` (`mul_comm`, `mul_add`, `ediv_add_emod`) | proved & witnessed |
| Cast laws `num.cast.nat_int_roundtrip`, `num.cast.int_injective` — `((n:ℤ)).toNat = n` and the injective `ℤ ↪ ℚ` widening for `FromPrimitive`/`ToPrimitive`/`NumCast` | Mathlib `Int.toNat_natCast`, `Int.cast_injective` | proved & witnessed |
| `Float106` model `num.float106.model.add_comm`, `num.float106.model.mul_comm`, `num.float106.model.distrib` — the double-double's arithmetic models the ordered-field laws of `ℝ` (each stated over `ℝ`, the value the two limbs stand for) | Mathlib `Real` (`add_comm`, `mul_comm`, `mul_add`); Dekker 1971 / Knuth (double-double, bit-exact part `[open]`) | proved & witnessed |

## Outstanding issues

1. **`Float106` bit-exact error bounds are out of L1 scope.** The Lean layer pins only the
   ALGEBRAIC MODEL — the sense in which the double-double stands for a real number and obeys the
   ordered-field laws over `ℝ`. The bit-exact double-double behaviour — the Dekker/Knuth two-sum and
   two-product error-free transformations and their limb-level error bounds — is `[open]` and is NOT
   proved in Lean; it is covered by the crate's regular `tests/float_double/` numeric tests only.
2. **Laws are proved per canonical carrier, not per generic instance.** The Lean carriers are the
   canonical structures (`ℤ`, `ℕ`, `ℚ`, `ℝ`, the identity monoids); the Rust witnesses check
   representative concrete types (`i64`, `usize`, `f64`, `Float106`). Extending the model to every
   integer/float width the traits are implemented for is mechanical scaling work — Lean proves ∀
   over the carrier, the witnesses pin the shipped instances at representative inputs.
3. **Mathlib is a build dependency of this layer.** These four files are not self-contained (they
   `import Mathlib`), so — unlike the Core/Haft notes — they cannot be checked with bare
   `lean <file>`; use `lake build` or `lake env lean <file>`. Bumping the Lean toolchain requires
   matching the Mathlib `rev` pinned in `lean/lakefile.toml`.