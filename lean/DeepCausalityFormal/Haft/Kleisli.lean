/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Kleisli category laws.

Rust source: `deep_causality_haft/src/category/mod.rs` (witness `Kleisli<M>`). The Kleisli category of
a monad `M`: `Hom<B> = M::Type<B>`, `id = pure`, `compose f g = fun a => bind (f a) g`.

Textbook definition: for a monad `(M, η, μ)` on a category, its **Kleisli category** `Kl(M)` has the
same objects, hom-sets `Hom_Kl(A, B) = Hom(A, M B)`, composition the Kleisli arrow
`f >=> g = μ ∘ M g ∘ f` (equivalently `fun a => bind (f a) g`), and identity the unit `η = pure`
(Mac Lane, *Categories for the Working Mathematician*, §VI.5; Moggi, "Notions of Computation and
Monads", *Inf. and Comput.* 93(1), 1991). The Kleisli category laws (left identity, right identity,
associativity) reduce exactly to the three monad laws — this is what the proof below shows.

DEVIATION NOTE: the monad is transcribed self-contained as a `Monad'` structure carrying `pure`,
`bind`, and the three monad laws as fields (rather than importing Mathlib's `Monad`/`LawfulMonad`),
so the file typechecks standalone; this mirrors `Haft/Monad.lean` and matches the Rust `Kleisli<M>`
whose `id`/`compose` are `pure`/`bind`. Composition is diagrammatic (`compose f g`, "`f` then `g`").

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/kleisli_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Kleisli

variable {M : Type → Type} {A B C D : Type}

/-- A monad transcribed self-contained: `pure`, `bind`, and the three monad laws. -/
structure Monad' (M : Type → Type) where
  pure : ∀ {X}, X → M X
  bind : ∀ {X Y}, M X → (X → M Y) → M Y
  left_id : ∀ {X Y} (a : X) (g : X → M Y), bind (pure a) g = g a
  right_id : ∀ {X} (m : M X), bind m pure = m
  assoc : ∀ {X Y Z} (m : M X) (g : X → M Y) (h : Y → M Z),
      bind (bind m g) h = bind m (fun x => bind (g x) h)

/-- `Kleisli::id` — the identity Kleisli arrow, `pure`. -/
def kid (mo : Monad' M) : A → M A := mo.pure

/-- `Kleisli::compose` — the Kleisli arrow `f >=> g` (diagrammatic order). -/
def kcomp (mo : Monad' M) (f : A → M B) (g : B → M C) : A → M C :=
  fun a => mo.bind (f a) g

/-- The Kleisli category laws — left identity, right identity, associativity — each reducing to the
    corresponding monad law.

    THEOREM_MAP: `haft.kleisli.category_laws` -/
theorem category_laws (mo : Monad' M) (f : A → M B) (g : B → M C) :
    kcomp mo (kid mo) g = g
    ∧ kcomp mo f (kid mo) = f
    ∧ ∀ (h : C → M D), kcomp mo (kcomp mo f g) h = kcomp mo f (kcomp mo g h) := by
  refine ⟨?_, ?_, ?_⟩
  · funext a; exact mo.left_id a g
  · funext a; exact mo.right_id (f a)
  · intro h; funext a; exact mo.assoc (f a) g h

end DeepCausalityFormal.Haft.Kleisli
