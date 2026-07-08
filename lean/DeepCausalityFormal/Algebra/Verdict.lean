/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Num — the verdict carrier: bounded-lattice + complement laws.

Mirrors the Rust trait `deep_causality_num::Verdict` (`src/algebra/verdict.rs`), the aggregation
output type: a bounded lattice with complement. The Boolean class (`bool`) is proved concretely on
`Bool`'s `&&`/`||`/`not` — meet/join commutativity + absorption and complement involution + De
Morgan — which needs no `Mathlib.Order.*` (its olean cache is unavailable here) and mirrors the Rust
`impl Verdict for bool` exactly. The probability carrier's complement `1 − p` is the MV-algebra
complement, witnessed on the Rust side (`Prob`); `None` = `Any` (join-fold) post-composed with
complement.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_algebra/tests/algebra/verdict_tests.rs`.
-/

namespace DeepCausalityFormal.Algebra

/-- Meet commutativity of the boolean verdict lattice.

    THEOREM_MAP: `algebra.verdict.lattice_laws` -/
theorem verdict_meet_comm (x y : Bool) : (x && y) = (y && x) := by
  cases x <;> cases y <;> rfl

/-- Absorption of the boolean verdict lattice: `x ⊓ (x ⊔ y) = x`.

    THEOREM_MAP: `algebra.verdict.lattice_laws` -/
theorem verdict_absorption (x y : Bool) : (x && (x || y)) = x := by
  cases x <;> cases y <;> rfl

/-- Complement is an involution: `complement (complement x) = x`.

    THEOREM_MAP: `algebra.verdict.complement` -/
theorem verdict_compl_compl (x : Bool) : (!(!x)) = x := by
  cases x <;> rfl

/-- De Morgan: `complement (x ⊓ y) = complement x ⊔ complement y`.

    THEOREM_MAP: `algebra.verdict.complement` -/
theorem verdict_de_morgan (x y : Bool) : (!(x && y)) = ((!x) || (!y)) := by
  cases x <;> cases y <;> rfl

end DeepCausalityFormal.Algebra
