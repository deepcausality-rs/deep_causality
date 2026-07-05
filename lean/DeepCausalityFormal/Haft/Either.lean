/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Either as the binary coproduct.

Rust source: `deep_causality_haft/src/either/mod.rs` (`enum Either<L, R> { Left(L), Right(R) }`
— the carrier for choice in the arrow algebra, deliberately distinct from `Result`).

Accepted theory: Mac Lane, *CWM* 2nd ed., §III.3 — the coproduct `L + R` with injections
`Left`/`Right` satisfies the universal property: for every pair `f : L → C`, `g : R → C` there
is a UNIQUE `[f, g] : L + R → C` with `[f, g] ∘ Left = f` and `[f, g] ∘ Right = g`. This is
what makes `Either` "the" sum: any two-armed router through it is uniquely determined by its
arms. The Rust enum is this coproduct; `match` is the mediating morphism.

The inductive below transcribes the Rust enum verbatim (rather than reusing Lean's `Sum`) so
the correspondence is one-to-one.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/algebra/formalization_law_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.EitherCoproduct

/-- Transcription of the Rust `Either<L, R>`. -/
inductive Either (L R : Type) where
  | left (l : L)
  | right (r : R)

variable {L R C : Type}

/-- The mediating morphism `[f, g]` — a Rust `match` on the two variants. -/
def either (f : L → C) (g : R → C) : Either L R → C
  | .left l => f l
  | .right r => g r

/-- Coproduct universal property (Mac Lane §III.3): `[f, g]` commutes with both injections,
    and it is the UNIQUE such morphism.

    THEOREM_MAP: `haft.either.coproduct_universal` -/
theorem coproduct_universal (f : L → C) (g : R → C) :
    (∀ l, either f g (.left l) = f l)
      ∧ (∀ r, either f g (.right r) = g r)
      ∧ ∀ h : Either L R → C,
          (∀ l, h (.left l) = f l) → (∀ r, h (.right r) = g r) → h = either f g := by
  refine ⟨fun _ => rfl, fun _ => rfl, fun h hl hr => ?_⟩
  funext x
  cases x with
  | left l => exact (hl l).trans rfl
  | right r => exact (hr r).trans rfl

end DeepCausalityFormal.Haft.EitherCoproduct
