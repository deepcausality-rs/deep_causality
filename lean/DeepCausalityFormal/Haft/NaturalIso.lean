/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Natural isomorphism laws.

Rust source: `deep_causality_haft/src/iso/natural_iso.rs` (trait `NaturalIso<F, G>`, operations
`to_target`, `to_source`) and its arity-2..5 siblings (`natural_iso_2.rs` … `natural_iso_5.rs`,
which lift the same two laws to more parameters; `NaturalIso5` targets the arity-5
propagating-effect carrier).

Accepted theory: Mac Lane, *CWM* 2nd ed., §I.4 — a natural isomorphism is a natural
transformation whose every component is invertible. The Rust module docstring states precisely
the two families of laws (component-wise round-trip; naturality squares commuting with `fmap`)
— **correct as documented**.

Canonical model: `Option A ≅ Unit ⊕ A` — the standard presentation of `Option` as a sum,
between two genuinely different type constructors, so the naturality square carries content.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/algebra/formalization_law_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.NaturalIso

variable {A B : Type}

/-- `F::fmap` — Option. -/
def optFmap (f : A → B) : Option A → Option B
  | some a => some (f a)
  | none => none

/-- `G::fmap` — the sum functor `Unit ⊕ -`. -/
def sumFmap (f : A → B) : Sum Unit A → Sum Unit B
  | .inl u => .inl u
  | .inr a => .inr (f a)

/-- `to_target : Option A → Unit ⊕ A`. -/
def toTarget : Option A → Sum Unit A
  | none => .inl ()
  | some a => .inr a

/-- `to_source : Unit ⊕ A → Option A`. -/
def toSource : Sum Unit A → Option A
  | .inl _ => none
  | .inr a => some a

/-- Round-trip laws (Rust docstring law 1): both composites are the identity, for every
    component `T` (Mac Lane §I.4 — each component is an isomorphism).

    THEOREM_MAP: `haft.natural_iso.laws` -/
theorem round_trip (x : Option A) (y : Sum Unit A) :
    toSource (toTarget x) = x ∧ toTarget (toSource y) = y := by
  constructor
  · cases x <;> rfl
  · cases y with
    | inl u => cases u; rfl
    | inr a => rfl

/-- Naturality (Rust docstring law 2): the square `to_target ∘ fmap_F h = fmap_G h ∘ to_target`
    commutes (Mac Lane §I.4).

    THEOREM_MAP: `haft.natural_iso.laws` -/
theorem naturality (h : A → B) (x : Option A) :
    toTarget (optFmap h x) = sumFmap h (toTarget x) := by
  cases x <;> rfl

end DeepCausalityFormal.Haft.NaturalIso
