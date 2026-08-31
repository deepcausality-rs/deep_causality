/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Applicative functor laws.

Rust source: `deep_causality_unified_math/deep_causality_haft/src/applicative/mod.rs` (trait `Applicative<F>: Functor<F> +
Pure<F>`, operation `apply`). Canonical carrier: `Option` via `OptionWitness`, whose `apply` is
`f_ab.and_then(|f| f_a.map(f))` — transcribed by `optApply`.

Accepted theory: C. McBride & R. Paterson, *Applicative programming with effects*, JFP 18(1),
2008 — an applicative functor satisfies FOUR laws: Identity, Composition, Homomorphism,
Interchange, plus the functor-compatibility `fmap f x = pure f <*> x`.

All five are proved below for the Option carrier, and the Rust docstring lists all five: the four
laws numbered in order plus functor-compatibility. An earlier revision of this header reported the
Composition law and functor-compatibility as missing from the docstring; that was deviation D1,
recorded as RESOLVED (docs) in the deviations note, and the header claim is retired with it.

The `apply` derived from the lax monoidal structure map is proved to agree with `optApply` below
in `LaxMonoidal.lean`, under `haft.lax_monoidal.apply_agreement`. That is the obligation a witness
takes on by implementing both `Applicative` and `MonoidalApplicative`; nothing in this file
changes as a result.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_unified_math/deep_causality_haft/tests/formalization_lean/applicative_tests.rs`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
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
    (McBride–Paterson 2008). Law 2 in the Rust docstring's numbering.

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
