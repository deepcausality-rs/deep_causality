/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Applicative functor laws.

Rust source: `deep_causality_haft/src/applicative/mod.rs` (trait `Applicative<F>: Functor<F> +
Pure<F>`, operation `apply`). Canonical carrier: `Option` via `OptionWitness`, whose `apply` is
`f_ab.and_then(|f| f_a.map(f))` — transcribed by `optApply`.

Accepted theory: C. McBride & R. Paterson, *Applicative programming with effects*, JFP 18(1),
2008 — an applicative functor satisfies FOUR laws: Identity, Composition, Homomorphism,
Interchange, plus the functor-compatibility `fmap f x = pure f <*> x`.

DEVIATION NOTE: the Rust docstring lists only three laws (Identity, Homomorphism, Interchange).
The **Composition law is missing**, and functor-compatibility is unstated. All five are proved
below for the Option carrier; the recommendation is to complete the docstring's law list.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/applicative_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Applicative

variable {A B C : Type}

/-- `OptionWitness::fmap` (see `Functor.lean`). -/
def optFmap (f : A → B) : Option A → Option B
  | some a => some (f a)
  | none => none

/-- `OptionWitness::pure`. -/
def optPure (a : A) : Option A := some a

/-- `OptionWitness::apply`: `f_ab.and_then(|f| f_a.map(f))`. -/
def optApply (fab : Option (A → B)) (fa : Option A) : Option B :=
  match fab with
  | some f => optFmap f fa
  | none => none

/-- Applicative Identity: `pure id <*> v = v` (McBride–Paterson 2008).

    THEOREM_MAP: `haft.applicative.laws` -/
theorem opt_apply_identity (v : Option A) :
    optApply (optPure (fun a => a)) v = v := by
  cases v <;> rfl

/-- Applicative Homomorphism: `pure f <*> pure x = pure (f x)` (McBride–Paterson 2008).

    THEOREM_MAP: `haft.applicative.laws` -/
theorem opt_apply_homomorphism (f : A → B) (x : A) :
    optApply (optPure f) (optPure x) = optPure (f x) := rfl

/-- Applicative Interchange: `u <*> pure y = pure (fun f => f y) <*> u` (McBride–Paterson 2008).

    THEOREM_MAP: `haft.applicative.laws` -/
theorem opt_apply_interchange (u : Option (A → B)) (y : A) :
    optApply u (optPure y) = optApply (optPure (fun f => f y)) u := by
  cases u <;> rfl

/-- Applicative Composition: `pure (∘) <*> u <*> v <*> w = u <*> (v <*> w)`
    (McBride–Paterson 2008). **This law is absent from the Rust docstring** — the deviation this
    file reports.

    THEOREM_MAP: `haft.applicative.laws` -/
theorem opt_apply_composition (u : Option (B → C)) (v : Option (A → B)) (w : Option A) :
    optApply (optApply (optApply (optPure (fun (f : B → C) (g : A → B) (a : A) => f (g a))) u) v) w
      = optApply u (optApply v w) := by
  cases u <;> cases v <;> cases w <;> rfl

/-- Functor compatibility: `fmap f x = pure f <*> x` (McBride–Paterson 2008). Ensures `apply`
    and `fmap` present one functor, not two.

    THEOREM_MAP: `haft.applicative.functor_compat` -/
theorem opt_apply_fmap_compat (f : A → B) (x : Option A) :
    optFmap f x = optApply (optPure f) x := by
  cases x <;> rfl

end DeepCausalityFormal.Haft.Applicative
