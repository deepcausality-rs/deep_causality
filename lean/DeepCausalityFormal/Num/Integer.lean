/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Num — integer ring and Euclidean-division laws.

Mirrors the Rust trait `deep_causality_num::Integer` (`src/integer/`), which fixes the commutative
ring structure over `ℤ` together with Euclidean division. The commutativity and distributivity laws
are the ring laws; the Euclidean law mirrors the Rust `div_euclid`/`rem_euclid` pair, pinning the
identity that reconstructs a dividend from its quotient and remainder.

Rust witness: `deep_causality_num/tests/integer/`.
-/

import Mathlib.Algebra.Order.Group.Int

namespace DeepCausalityFormal.Num

/-- Commutativity of integer multiplication: `a * b = b * a`.

    THEOREM_MAP: `num.integer.mul_comm` -/
theorem integer_mul_comm (a b : ℤ) : a * b = b * a :=
  Int.mul_comm a b

/-- Left distributivity over `ℤ`: `a * (b + c) = a * b + a * c`.

    THEOREM_MAP: `num.integer.distrib` -/
theorem integer_left_distrib (a b c : ℤ) : a * (b + c) = a * b + a * c :=
  Int.mul_add a b c

/-- Euclidean division: `b * (a / b) + a % b = a`, mirroring the Rust
    `div_euclid`/`rem_euclid` reconstruction of a dividend.

    THEOREM_MAP: `num.integer.euclidean` -/
theorem integer_euclidean (a b : ℤ) : b * (a / b) + a % b = a :=
  Int.mul_ediv_add_emod a b

end DeepCausalityFormal.Num
