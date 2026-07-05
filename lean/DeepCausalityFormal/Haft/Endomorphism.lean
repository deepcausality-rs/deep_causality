/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Endomorphism monoid and bounded iteration.

Rust source: `deep_causality_haft/src/morphism/morphism_endo.rs` (trait `Endomorphism<P>`,
combinators `iterate_n`, `iterate_to_fixpoint`, `iterate_until`) and its value-level twin
`deep_causality_haft/src/arrow/arrow_endo.rs` (`EndoArrow<S>` with the same three combinators).

Accepted theory: for any object `T`, the endomorphisms `End(T)` form a monoid under composition
with the identity as unit (Mac Lane, *CWM* 2nd ed., §I.1 — a one-object category IS a monoid).
The Rust docstring claims exactly this ("`End(T)` is a monoid under composition") — verified
below. `iterate_n` is the monoid power `fⁿ`; the power law `f^(m+n) = fⁿ ∘ fᵐ` justifies
folding a list of steps into one (the docstring's stated purpose).

The `iterate_to_fixpoint` / `iterate_until` combinators add explicit step bounds and reached
flags — engineering-honest totalization of possibly-divergent iteration; their mathematical
content is the power law, which is what is proved.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/algebra/formalization_law_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Endomorphism

variable {S : Type}

/-- Composition of endomorphisms (diagrammatic order, matching `Compose::run`). -/
def endoComp (f g : S → S) : S → S := fun x => g (f x)

/-- `iterate_n`: apply `f` exactly `n` times — the `n`-th monoid power. The Rust `for` loop
    computes the same function; head recursion is chosen here for clean structural induction. -/
def iterateN (f : S → S) : Nat → S → S
  | 0, x => x
  | n + 1, x => f (iterateN f n x)

/-- `End(T)` is a monoid: identity is a two-sided unit and composition is associative
    (Mac Lane §I.1).

    THEOREM_MAP: `haft.endo.monoid` -/
theorem endo_monoid (f g h : S → S) :
    endoComp (fun x => x) f = f
      ∧ endoComp f (fun x => x) = f
      ∧ endoComp (endoComp f g) h = endoComp f (endoComp g h) :=
  ⟨rfl, rfl, rfl⟩

/-- Monoid power law: `iterate_n f (m + n) = iterate_n f n ∘ iterate_n f m` — iterating in two
    bouts equals iterating once; the law that licenses folding step lists.

    THEOREM_MAP: `haft.endo.iterate_add` -/
theorem iterate_add (f : S → S) (m n : Nat) (x : S) :
    iterateN f (m + n) x = iterateN f n (iterateN f m x) := by
  induction n with
  | zero => rfl
  | succ n ih => exact congrArg f ih

end DeepCausalityFormal.Haft.Endomorphism
