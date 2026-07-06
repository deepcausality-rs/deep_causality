/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Profunctor laws.

Rust source: `deep_causality_haft/src/functor/profunctor.rs` (trait `Profunctor<P: HKT2Unbound>`,
operations `dimap`, `lmap`, `rmap`). Canonical carrier: the function profunctor `P A B = A → B`
(the crate's `profunctor_tests.rs` uses exactly this via its `FunctionWitness`).

Accepted theory: a profunctor is a functor `P : Cᵒᵖ × D → Set` (J. Bénabou, *Distributors at
work*, 2000; F. Loregian, *(Co)end Calculus*, CUP 2021, §5) — contravariant in the first
argument, covariant in the second. The functor laws specialize to
  1. `dimap id id = id`
  2. `dimap (f ∘ f') (g' ∘ g) = dimap f' g' ∘ dimap f g`   (note the contravariant twist on
     the first argument: pre-processors compose in reversed order)

DEVIATION NOTE: the Rust docstring gives the definition and use-cases but **states no laws**.
Both laws are proved below; the recommendation is to add them to the docstring as was done for
`Functor` and `Bifunctor`.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/profunctor_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Profunctor

variable {A B C D A' B' : Type}

/-- `dimap` on the function carrier: pre-process the input, post-process the output. -/
def dimap (pre : C → A) (post : B → D) (p : A → B) : C → D :=
  fun c => post (p (pre c))

/-- Profunctor identity: `dimap id id = id` (Loregian §5).

    THEOREM_MAP: `haft.profunctor.laws` -/
theorem dimap_id (p : A → B) :
    dimap (fun a => a) (fun b => b) p = p := rfl

/-- Profunctor composition with the contravariant twist:
    `dimap pre' post' (dimap pre post p) = dimap (pre ∘ pre') (post' ∘ post) p`
    (Loregian §5). Pre-processors compose reversed; post-processors compose forward.

    THEOREM_MAP: `haft.profunctor.laws` -/
theorem dimap_comp (pre : C → A) (post : B → D) (pre' : A' → C) (post' : D → B') (p : A → B) :
    dimap pre' post' (dimap pre post p)
      = dimap (fun a' => pre (pre' a')) (fun b => post' (post b)) p := rfl

end DeepCausalityFormal.Haft.Profunctor
