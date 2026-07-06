/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Bifunctor laws.

Rust source: `deep_causality_haft/src/functor/bifunctor.rs` (trait `Bifunctor<F: HKT2Unbound>`,
operations `bimap`, `first`, `second`). Canonical carrier: `Result<A, B>` via
`ResultUnboundWitness`; modelled by the two-constructor sum (Lean `Sum`), which also models the
crate's own `Either<L, R>`.

Accepted theory: a bifunctor is a functor from the product category `C × D → E`
(Mac Lane, *CWM* 2nd ed., §II.3), determined by
  1. `bimap id id = id`
  2. `bimap (f' ∘ f) (g' ∘ g) = bimap f' g' ∘ bimap f g`
The Rust docstring states exactly these — **correct as documented**. The crate's `first`/`second`
are the default specializations `bimap f id` / `bimap id g`; the commuting decomposition
`bimap f g = first f ∘ second g` is proved as well (it is what "independently and simultaneously"
in the docstring means formally).

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/bifunctor_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Bifunctor

variable {A B C D A' B' : Type}

/-- `bimap` on the sum carrier (models `Result` / `Either`): map each side. -/
def bimap (f : A → C) (g : B → D) : Sum A B → Sum C D
  | .inl a => .inl (f a)
  | .inr b => .inr (g b)

/-- Bifunctor identity: `bimap id id = id` (Mac Lane §II.3; Rust docstring law 1).

    THEOREM_MAP: `haft.bifunctor.laws` -/
theorem bimap_id (x : Sum A B) :
    bimap (fun a => a) (fun b => b) x = x := by
  cases x <;> rfl

/-- Bifunctor composition: `bimap (f' ∘ f) (g' ∘ g) = bimap f' g' ∘ bimap f g`
    (Mac Lane §II.3; Rust docstring law 2).

    THEOREM_MAP: `haft.bifunctor.laws` -/
theorem bimap_comp (f : A → C) (f' : C → A') (g : B → D) (g' : D → B') (x : Sum A B) :
    bimap (fun a => f' (f a)) (fun b => g' (g b)) x = bimap f' g' (bimap f g x) := by
  cases x <;> rfl

/-- Decomposition: `bimap f g = first f ∘ second g` where `first f = bimap f id` and
    `second g = bimap id g` (the crate's default implementations). The two partial maps
    commute and jointly recover `bimap`.

    THEOREM_MAP: `haft.bifunctor.laws` -/
theorem bimap_first_second (f : A → C) (g : B → D) (x : Sum A B) :
    bimap f g x = bimap f (fun b => b) (bimap (fun a => a) g x) := by
  cases x <;> rfl

end DeepCausalityFormal.Haft.Bifunctor
