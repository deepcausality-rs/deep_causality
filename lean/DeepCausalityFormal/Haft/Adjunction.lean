/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Adjunction laws.

Rust source: `deep_causality_haft/src/adjunction/mod.rs` (trait `Adjunction<L, R, Context>`,
operations `unit`, `counit`, `left_adjunct`, `right_adjunct`).

Accepted theory: Mac Lane, *CWM* 2nd ed., §IV.1 — an adjunction `L ⊣ R` is a natural bijection
`Hom(L A, B) ≅ Hom(A, R B)` (the two adjuncts), equivalently unit/counit satisfying the two
triangle identities `Rε ∘ ηR = id_R` and `εL ∘ Lη = id_L`. The Rust docstring states exactly
these — **correct as documented**.

Rust artifact: the `Context` parameter (runtime metric/shape data) has no counterpart in the
mathematical definition; it indexes a *family* of adjunctions, one per context value. The laws
are per-fixed-context, which is what the model proves (the context is a fixed ambient parameter
here — `S` plays that role structurally).

Canonical model: the currying adjunction `(- × S) ⊣ (S → -)` — named in the Rust docstring
itself ("Currying/Uncurrying") and the adjunction underlying the state monad. `L A = A × S`,
`R B = S → B`.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/adjunction_tests.rs`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

namespace DeepCausalityFormal.Haft.Adjunction

variable {S A B : Type}

/-- Left adjoint `L A = A × S`. -/
def L (S A : Type) : Type := A × S

/-- Right adjoint `R B = S → B`. -/
def R (S B : Type) : Type := S → B

/-- `unit : A → R (L A)` — η of the currying adjunction. -/
def unit (a : A) : R S (L S A) := fun s => (a, s)

/-- `counit : L (R B) → B` — ε (evaluation). -/
def counit (lrb : L S (R S B)) : B := lrb.1 lrb.2

/-- `left_adjunct : (L A → B) → (A → R B)` — Mac Lane's φ, `φ(f) = R f ∘ η`. -/
def leftAdjunct (f : L S A → B) (a : A) : R S B := fun s => f (a, s)

/-- `right_adjunct : (A → R B) → (L A → B)` — Mac Lane's φ⁻¹, `φ⁻¹(g) = ε ∘ L g`. -/
def rightAdjunct (g : A → R S B) (la : L S A) : B := g la.1 la.2

/-- Triangle identity on `R`: `R(ε) ∘ η_R = id_R` (Mac Lane §IV.1, eq. (9); Rust docstring
    triangle 1). `R` acts on morphisms by post-composition.

    THEOREM_MAP: `haft.adjunction.triangles` -/
theorem triangle_right (rb : R S B) :
    (fun s => counit ((unit rb) s)) = rb := rfl

/-- Triangle identity on `L`: `ε_L ∘ L(η) = id_L` (Mac Lane §IV.1, eq. (9); Rust docstring
    triangle 2). `L` acts on morphisms on the first component.

    THEOREM_MAP: `haft.adjunction.triangles` -/
theorem triangle_left (la : L S A) :
    counit (unit la.1, la.2) = la := rfl

/-- The adjuncts are mutually inverse — the `Hom(L A, B) ≅ Hom(A, R B)` bijection
    (Mac Lane §IV.1, Theorem 1).

    THEOREM_MAP: `haft.adjunction.adjunct_inverse` -/
theorem adjunct_inverse (f : L S A → B) (g : A → R S B) :
    rightAdjunct (leftAdjunct f) = f ∧ leftAdjunct (rightAdjunct g) = g :=
  ⟨rfl, rfl⟩

/-- Adjunct-via-unit factorization: `left_adjunct f = R(f) ∘ η` — the adjunct is not extra
    data; it is determined by the unit (Mac Lane §IV.1). -/
theorem left_adjunct_via_unit (f : L S A → B) (a : A) :
    leftAdjunct f a = fun s => f ((unit a) s) := rfl

end DeepCausalityFormal.Haft.Adjunction
