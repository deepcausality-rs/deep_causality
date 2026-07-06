/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Monad laws.

Rust source: `deep_causality_haft/src/monad/mod.rs` (trait `Monad<F>: Functor<F> + Pure<F>`,
operations `bind`, `join`). Canonical carrier: `Option` via `OptionWitness`, whose `bind`
matches `Some → f(a) / None → None` — transcribed by `optBind`.

Accepted theory: E. Moggi, *Notions of computation and monads*, Inf. & Comp. 93(1), 1991;
P. Wadler, *Monads for functional programming*, 1995. The Kleisli-triple presentation demands
left identity, right identity, and associativity — exactly the three laws the Rust docstring
states. `join = bind id` is the crate's default implementation; it recovers the
multiplication `μ` of the monoid-in-endofunctors presentation (Mac Lane §VI.1).

HIERARCHY DEVIATION (deliberate, documented in the Rust source): the crate uses
`Monad: Functor + Pure` instead of the conventional `Monad: Applicative` (Haskell post-AMP), to
keep strict constrained witnesses implementable. Mathematically harmless — a monad induces its
applicative via `apply f_ab f_a = bind f_ab (fun f => fmap f f_a)` — but the *coherence* of that
induced applicative with any hand-written `apply` must then be a law. It is stated and proved
below (`opt_monad_applicative_coherence`) for the Option carrier; the recommendation is to state
it in the docs for every witness implementing both traits.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/monad_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Monad

variable {A B C : Type}

/-- `OptionWitness::fmap` (see `Functor.lean`). -/
def optFmap (f : A → B) : Option A → Option B
  | some a => some (f a)
  | none => none

/-- `OptionWitness::pure`. -/
def optPure (a : A) : Option A := some a

/-- `OptionWitness::apply` (see `Applicative.lean`). -/
def optApply (fab : Option (A → B)) (fa : Option A) : Option B :=
  match fab with
  | some f => optFmap f fa
  | none => none

/-- `OptionWitness::bind`: `Some a → f a`, `None → None`. -/
def optBind (m : Option A) (f : A → Option B) : Option B :=
  match m with
  | some a => f a
  | none => none

/-- Monad left identity: `bind (pure a) f = f a` (Moggi 1991; Wadler 1995).

    THEOREM_MAP: `haft.monad.laws` -/
theorem opt_bind_left_id (a : A) (f : A → Option B) :
    optBind (optPure a) f = f a := rfl

/-- Monad right identity: `bind m pure = m` (Moggi 1991; Wadler 1995).

    THEOREM_MAP: `haft.monad.laws` -/
theorem opt_bind_right_id (m : Option A) :
    optBind m optPure = m := by
  cases m <;> rfl

/-- Monad associativity: `bind (bind m f) g = bind m (fun a => bind (f a) g)`
    (Moggi 1991; Wadler 1995).

    THEOREM_MAP: `haft.monad.laws` -/
theorem opt_bind_assoc (m : Option A) (f : A → Option B) (g : B → Option C) :
    optBind (optBind m f) g = optBind m (fun a => optBind (f a) g) := by
  cases m <;> rfl

/-- Monad ⇒ Applicative coherence: the hand-written `apply` agrees with the applicative the
    monad induces — `apply f_ab f_a = bind f_ab (fun f => fmap f f_a)`. This is the law the
    `Monad: Functor + Pure` hierarchy (in place of `Monad: Applicative`) owes.

    THEOREM_MAP: `haft.monad.applicative_coherence` -/
theorem opt_monad_applicative_coherence (fab : Option (A → B)) (fa : Option A) :
    optApply fab fa = optBind fab (fun f => optFmap f fa) := by
  cases fab <;> rfl

/-- `join = bind id` (the crate's default `join`) recovers the multiplication `μ : M∘M ⇒ M` of
    the monoid presentation (Mac Lane §VI.1); flattening a doubly-wrapped value. -/
theorem opt_join_flattens (mma : Option (Option A)) :
    optBind mma (fun x => x)
      = match mma with | some ma => ma | none => none := by
  cases mma <;> rfl

end DeepCausalityFormal.Haft.Monad
