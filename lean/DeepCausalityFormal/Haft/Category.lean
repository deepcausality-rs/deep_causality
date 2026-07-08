/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Category laws (the function category).

Rust source: `deep_causality_haft/src/category/mod.rs` (trait `Category`, witness `Fun`). `Fun` is
the category of functions: `Hom<B> = B`, `id = fun a => a`, `compose f g = fun a => g (f a)`.

Textbook definition: a category consists of objects, hom-sets `Hom(A, B)`, an associative
composition `∘`, and an identity `id_A ∈ Hom(A, A)` satisfying the left/right unit and associativity
axioms (Mac Lane, *Categories for the Working Mathematician*, §I.1; Awodey, *Category Theory*, §1.1).
It is modelled here as the category **Set/Type of Lean functions** — objects are types, morphisms are
functions `A → B`, composition is function composition, identity is `fun a => a`. This is the
semantic category the value-level Rust `Arrow` runs in (Hughes, "Generalising Monads to Arrows",
*Sci. Comput. Program.* 37, 2000; see `Haft/Arrow.lean`).

DEVIATION NOTE: the model is the concrete (extensional) function category rather than an abstract
category typeclass — there is no first-class `Category` object in Lean here, matching the Rust `Fun`
witness whose morphisms are plain closures. Composition is written in diagrammatic order
(`compose f g = g ∘ f`, "`f` then `g`") to match the Rust `Category::compose`.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/category_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Category

variable {A B C : Type}

/-- `Fun::id` — the identity morphism. -/
def idf : A → A := fun a => a

/-- `Fun::compose` — diagrammatic composition `f` then `g` (`= g ∘ f`). -/
def comp (f : A → B) (g : B → C) : A → C := fun a => g (f a)

/-- The three category axioms of the function category: left identity, right identity, associativity.

    THEOREM_MAP: `haft.category.laws` -/
theorem category_laws (f : A → B) (g : B → C) :
    comp (idf) g = g
    ∧ comp f (idf) = f
    ∧ ∀ {D : Type} (h : C → D), comp (comp f g) h = comp f (comp g h) :=
  ⟨rfl, rfl, fun _ => rfl⟩

end DeepCausalityFormal.Haft.Category
