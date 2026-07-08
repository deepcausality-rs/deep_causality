/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Num — Float106 algebraic-model laws.

`Float106` (Rust `deep_causality_num::Float106`, `src/float_106/`) is a double-double type: it
represents one real number as an unevaluated sum of two `f64` limbs (a high part and a low
correction term), extending the working precision to roughly 106 bits. This file formalizes only
the ALGEBRAIC-MODEL laws — the sense in which `Float106`'s arithmetic models the ordered-field laws
of `ℝ`. Each law is therefore stated directly over `ℝ`, the value the double-double stands for.

The bit-exact double-double behaviour — the Dekker/Knuth two-sum and two-product error-free
transformations and their limb-level error bounds — is `[open]`: it is out of L1 scope and is NOT
proved here. What is pinned here is the model, not the floating-point representation.

Rust witness: `deep_causality_num/tests/float_double/`.
-/

import Mathlib.Data.Real.Basic

namespace DeepCausalityFormal.Num

/-- Model additive commutativity: `a + b = b + a` over `ℝ`.

    THEOREM_MAP: `num.float106.model.add_comm` -/
theorem float106_model_add_comm (a b : ℝ) : a + b = b + a :=
  add_comm a b

/-- Model multiplicative commutativity: `a * b = b * a` over `ℝ`.

    THEOREM_MAP: `num.float106.model.mul_comm` -/
theorem float106_model_mul_comm (a b : ℝ) : a * b = b * a :=
  mul_comm a b

/-- Model left distributivity: `a * (b + c) = a * b + a * c` over `ℝ`.

    THEOREM_MAP: `num.float106.model.distrib` -/
theorem float106_model_distrib (a b c : ℝ) : a * (b + c) = a * b + a * c :=
  mul_add a b c

end DeepCausalityFormal.Num
