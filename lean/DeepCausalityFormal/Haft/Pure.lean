/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Pure (pointed functor) naturality.

Rust source: `deep_causality_haft/src/pure/mod.rs` (trait `Pure<F: HKT>`, operation `pure`).
The standalone `Pure` trait is a Rust-motivated split (documented in the source: it lets `Monad`
avoid `Applicative`'s closure constraint); mathematically it is a *pointed functor* — a functor
`F` equipped with a natural transformation `η : Id ⇒ F` (Mac Lane, *CWM* 2nd ed., §I.4 for
natural transformations).

DEVIATION NOTE: the Rust docstring calls `pure` "the natural transformation η: Id → F" but never
states the naturality square `fmap f ∘ pure = pure ∘ f` as a law. Without it, "natural" is an
unearned adjective. It is stated and proved here for the Option carrier; the recommendation is to
add it to the trait's law list.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/pure_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Pure

variable {A B : Type}

/-- `OptionWitness::fmap` (see `Functor.lean`). -/
def optFmap (f : A → B) : Option A → Option B
  | some a => some (f a)
  | none => none

/-- `OptionWitness::pure`: `Some(value)`. -/
def optPure (a : A) : Option A := some a

/-- Naturality of `pure` — the square `fmap f ∘ η = η ∘ f` commutes (Mac Lane §I.4).

    THEOREM_MAP: `haft.pure.naturality` -/
theorem opt_pure_naturality (f : A → B) (a : A) :
    optFmap f (optPure a) = optPure (f a) := rfl

end DeepCausalityFormal.Haft.Pure
