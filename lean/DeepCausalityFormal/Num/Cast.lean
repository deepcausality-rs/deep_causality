/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Num — primitive-cast round-trip and injectivity laws.

Mirrors the Rust conversions `deep_causality_num::{FromPrimitive, ToPrimitive}` (`src/cast/`). The
round-trip law pins that a natural number cast up to `ℤ` and back via `toNat` is recovered exactly;
the injectivity law pins that the integer cast into a characteristic-zero field loses no information,
so distinct integers cannot alias — the correctness guarantee behind the Rust widening conversions.

Rust witness: `deep_causality_num/tests/casts/`.
-/

import Mathlib.Data.Int.Cast.Lemmas
import Mathlib.Data.Rat.Cast.Defs

namespace DeepCausalityFormal.Num

/-- Round-trip `ℕ → ℤ → ℕ`: `((n : ℤ)).toNat = n`.

    THEOREM_MAP: `num.cast.nat_int_roundtrip` -/
theorem cast_nat_int_roundtrip (n : ℕ) : ((n : ℤ)).toNat = n :=
  Int.toNat_natCast n

/-- Injectivity of the integer cast into `ℚ`: `(m : ℚ) = (n : ℚ) → m = n`.

    THEOREM_MAP: `num.cast.int_injective` -/
theorem cast_int_injective (m n : ℤ) (h : (m : ℚ) = (n : ℚ)) : m = n :=
  Int.cast_injective h

end DeepCausalityFormal.Num
