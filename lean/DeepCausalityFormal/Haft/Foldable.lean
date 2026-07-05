/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Foldable.

Rust source: `deep_causality_haft/src/foldable/mod.rs` (trait `Foldable<F: HKT>`, operation
`fold` — a left fold). Canonical carriers: `Option` (modelled here) and `Vec` (Lean `List`
with `List.foldl`).

Accepted theory: folds are the universal property of the initial algebra (catamorphism); for a
trait exposing only `foldl`, the checkable coherences are its interaction with the other
operations. The law proved here is the docstring's "fold identity":
`fold f init (pure x) = f init x`.

DEVIATION NOTE: the Rust docstring's law 1 relates `foldr`, `flip`, and `reverse` — none of
which exist in the trait or the crate. It is a Haskell law quoted out of context; the
recommendation is to drop it or define the missing operations. (Law 2 is real and proved.)

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/algebra/formalization_law_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Foldable

variable {A B : Type}

/-- `OptionWitness::pure`. -/
def optPure (a : A) : Option A := some a

/-- `OptionWitness::fold`: apply once under `Some`, return `init` under `None`. -/
def optFold (fa : Option A) (init : B) (f : B → A → B) : B :=
  match fa with
  | some a => f init a
  | none => init

/-- Fold–pure compatibility: `fold (pure x) init f = f init x` (Rust docstring law 2).

    THEOREM_MAP: `haft.foldable.pure_compat` -/
theorem fold_pure_compat (x : A) (init : B) (f : B → A → B) :
    optFold (optPure x) init f = f init x := rfl

/-- Fold on the empty structure returns the accumulator unchanged — the unit case that makes
    `fold` total on partial containers. -/
theorem fold_none (init : B) (f : B → A → B) :
    optFold none init f = init := rfl

end DeepCausalityFormal.Haft.Foldable
