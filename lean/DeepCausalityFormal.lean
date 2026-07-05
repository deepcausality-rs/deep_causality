/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Root module of the DeepCausality formalization (Lean 4 + Mathlib).

Layered to mirror the Rust crate tiers:
  * `Num`  — foundational algebraic laws (monoid/group/ring/field), mirroring `deep_causality_num`.
  * `Haft` — HKT / functor / monad laws, mirroring `deep_causality_haft`  (scaling work).
  * `Core` — the Causal Monad `pure`/`bind` laws, mirroring `deep_causality_core`.

Each theorem is bound to a Rust witness via `lean/THEOREM_MAP.md`. See `lean/README.md`.

This walking skeleton proves exactly two exemplar theorems end-to-end (num add-monoid laws,
core bind left-identity). The full program is described in
`openspec/notes/causal-algebra/Formalization.md`.
-/

import DeepCausalityFormal.Num.Monoid
import DeepCausalityFormal.Core.CausalMonad
