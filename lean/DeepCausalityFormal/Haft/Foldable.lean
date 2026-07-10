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

`fold_map` (the monoidal fold, `Foldable::fold_map`).

Textbook definition: for a monoid `(M, ⊕, e)` and `f : A → M`,
`foldMap f = mconcat ∘ map f`, equivalently the unique monoid homomorphism from the free monoid
on `A` (finite lists, `(List A, ++, [])`) extending `f`
(Haskell `Data.Foldable`; Gibbons & Oliveira, "The Essence of the Iterator Pattern", JFP 2009;
the extension is the universal property of the free monoid, Mac Lane, CWM §VII.3). Its two
characteristic laws are the singleton law `foldMap f [a] = f a` and the homomorphism law
`foldMap f (xs ++ ys) = foldMap f xs ⊕ foldMap f ys` (with `foldMap f [] = e`).

DEVIATION NOTE (`fold_map`): the free monoid on `A` is the canonical `Foldable` carrier, so the
model uses `List A` and reads `pure a` as the singleton `[a]`; a general `Foldable` functor is not
modelled. The monoid is transcribed self-contained as `Mon` (mirroring
`deep_causality_algebra::Monoid`, `empty`/`combine`) to keep the file import-free.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/foldable_tests.rs`.
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

/-- A monoid, transcribed self-contained (mirrors `deep_causality_algebra::Monoid`: `empty`/`combine`
    with the identity and associativity laws). -/
structure Mon (M : Type) where
  e : M
  op : M → M → M
  left_id : ∀ x, op e x = x
  right_id : ∀ x, op x e = x
  assoc : ∀ x y z, op (op x y) z = op x (op y z)

variable {M : Type}

/-- `Foldable::fold_map` over the free-monoid carrier `List A`: fold threading `combine` from
    `empty`, i.e. `fold_map(fa, f) = fold(fa, e, fun acc a => acc.combine(f a))`. -/
def foldMap (m : Mon M) (f : A → M) : List A → M
  | [] => m.e
  | a :: as => m.op (f a) (foldMap m f as)

/-- Singleton law: `fold_map(pure a, f) = f a` (`pure a = [a]`), via the monoid right identity.

    THEOREM_MAP: `haft.foldable.fold_map_pure` -/
theorem fold_map_pure (m : Mon M) (f : A → M) (a : A) :
    foldMap m f [a] = f a := by
  simp [foldMap, m.right_id]

/-- Monoid-homomorphism coherence: `fold_map` respects `empty`/`combine` — the empty structure maps
    to `empty` and concatenation maps to `combine`:
    `fold_map(xs ++ ys, f) = fold_map(xs, f).combine(fold_map(ys, f))`.

    THEOREM_MAP: `haft.foldable.fold_map_monoid_coherence` -/
theorem fold_map_monoid_coherence (m : Mon M) (f : A → M) :
    ∀ xs ys : List A, foldMap m f (xs ++ ys) = m.op (foldMap m f xs) (foldMap m f ys)
  | [], ys => by simp [foldMap, m.left_id]
  | a :: xs, ys => by
    simp [foldMap, List.cons_append, fold_map_monoid_coherence m f xs ys, m.assoc]

end DeepCausalityFormal.Haft.Foldable
