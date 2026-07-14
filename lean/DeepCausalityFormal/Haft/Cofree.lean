/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — the Cofree Comonad: `Cofree f a`, the cofree comonad on a functor `f` and the categorical
dual of the free monad `Free f` (`FreeMonad.lean`).

Rust source: `deep_causality_haft/src/monad/cofree_comonad.rs` (`Cofree<F, A>`, `CofreeWitness<F>`).

The cofree comonad is `Cofree f a = a :< f (Cofree f a)` — a label paired with an `f`-structure of
sub-trees — with
  * `extract (a :< _)   = a`                                (the counit ε; dual of `Free.pure`)
  * `extend  k w        = k w :< fmap (extend k) (tail w)`  (cobind; dual of `Free.bind`)
  * `unfold  c x        = let (a, fx) = c x in a :< fmap (unfold c) fx`  (the anamorphism; dual of
    `Free.fold`)
and the THREE comonad (coKleisli) laws hold for EVERY functor `f`, using only `f`'s functor laws
(Uustalu & Vene, *Comonadic Notions of Computation*, ENTCS 203(5), 2008; Ghani, Uustalu & Vene,
*Build, Augment, Destroy, Unfold*, APLAS 2004). The three laws are exactly those of `Comonad.lean`.

Positivity & finiteness. Lean's inductive checker rejects `a :< f (Cofree f a)` for a *variable*
functor `f`, and — unlike the free monad — the cofree comonad is INFINITE over a functor with no
empty shape (it is coinductive). We therefore prove the laws over a REPRESENTATIVE functor that
*admits an empty shape*, so the tree is a finite inductive: `f a = Option a` (0-or-1 hole), written
in its two-constructor unfolded form
  `leaf a  = a :< none`      (an empty `f`-structure of children)
  `cons a c = a :< some c`   (one child)
— a non-empty chain. The proofs use only that functor's laws, so they discharge the general result,
exactly as `FreeMonad.lean` proves `Free` over `f a = E × a`. The multi-hole case (`f a = List a`,
i.e. the Rust `VecWitness`) has the identical induction over `List.map`'s functor laws; the Rust
witnesses use `VecWitness`.

Deviation notes:
  * Rust `extract`/`extend` take/return owned values with `A: Clone` on `extract`; the borrowing and
    cloning are memory-management encodings with no mathematical content (the Lean model consumes
    values), matching the `Comonad.lean` note.
  * `unfold` is coinductive in general; here it is witnessed at a representative depth-decreasing
    coalgebra on `Nat` (the seed strictly decreases), which is exactly the finite anamorphism the
    Rust `test_cofree_unfold_*` witnesses check.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality_haft/tests/formalization_lean/cofree_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Cofree

variable {A B C : Type}

/-- `Cofree Option a` in two-constructor form (the cofree comonad on the `Option` functor):
    `leaf a = a :< none`, `cons a c = a :< some c` — a non-empty chain of labels. -/
inductive Cofree (A : Type) where
  | leaf : A → Cofree A
  | cons : A → Cofree A → Cofree A

/-- `extract`: the counit ε — the label at the root. -/
def extract : Cofree A → A
  | .leaf a   => a
  | .cons a _ => a

/-- `extend`: cobind — refocus every node on the observation `k` of its whole sub-tree
    (`extend k w = k w :< fmap (extend k) (tail w)`). -/
def extend (k : Cofree A → B) : Cofree A → Cofree B
  | .leaf a   => .leaf (k (.leaf a))
  | .cons a c => .cons (k (.cons a c)) (extend k c)

/-- `map`: the functor action (`fmap f = extend (f ∘ extract)`; written directly here). -/
def map (f : A → B) : Cofree A → Cofree B
  | .leaf a   => .leaf (f a)
  | .cons a c => .cons (f a) (map f c)

-- ------------------------------------------------------------------
-- The three comonad laws (Uustalu–Vene 2008; the statements of `Comonad.lean`). They hold for the
-- cofree comonad over ANY functor; here over `Option`, using only that functor's laws.
-- ------------------------------------------------------------------

/-- Comonad right identity: `extract ∘ extend k = k` — the root of `extend k w` is `k w`, so this is
    definitional (`rfl` in each constructor case).

    THEOREM_MAP: `haft.cofree.comonad_laws` -/
theorem extract_extend (k : Cofree A → B) (w : Cofree A) :
    extract (extend k w) = k w := by
  cases w with
  | leaf a   => rfl
  | cons a c => rfl

/-- Comonad left identity: `extend extract = id` — by induction on the chain; the `cons` case needs
    the induction hypothesis on the child (the functor identity law of `Option`).

    THEOREM_MAP: `haft.cofree.comonad_laws` -/
theorem extend_extract (w : Cofree A) : extend extract w = w := by
  induction w with
  | leaf a => rfl
  | cons a c ih => simp [extend, extract, ih]

/-- Comonad associativity: `extend g ∘ extend f = extend (fun w' => g (extend f w'))` — by induction;
    the `cons` case reduces to the same statement at the child (the functor composition law of
    `Option`).

    THEOREM_MAP: `haft.cofree.comonad_laws` -/
theorem extend_assoc (f : Cofree A → B) (g : Cofree B → C) (w : Cofree A) :
    extend g (extend f w) = extend (fun w' => g (extend f w')) w := by
  induction w with
  | leaf a => rfl
  | cons a c ih => simp [extend, ih]

/-- Functor identity, as a corollary of left identity via `map f = extend (f ∘ extract)`; here the
    direct `map id = id`. -/
theorem map_id (w : Cofree A) : map (fun a => a) w = w := by
  induction w with
  | leaf a => rfl
  | cons a c ih => simp [map, ih]

-- ------------------------------------------------------------------
-- The anamorphism `unfold` (dual of `Free`'s `fold`), witnessed at a representative terminating
-- coalgebra on `Nat`: `unfold 0 = 0 :< none`, `unfold (n+1) = (n+1) :< some (unfold n)`.
-- ------------------------------------------------------------------

/-- A terminating anamorphism: the depth-decreasing coalgebra `n ↦ (n, if n = 0 then none else some
    (n-1))`. Finite because the `Nat` seed strictly decreases. -/
def unfold : Nat → Cofree Nat
  | 0     => .leaf 0
  | n + 1 => .cons (n + 1) (unfold n)

/-- `unfold` computation rule (anamorphism, dual of `Free.fold`): the root label is the coalgebra's
    head — `extract (unfold n) = n`.

    THEOREM_MAP: `haft.cofree.unfold` -/
theorem extract_unfold (n : Nat) : extract (unfold n) = n := by
  cases n with
  | zero   => rfl
  | succ m => rfl

/-- `unfold` unrolling: one step exposes the coalgebra's head and the recursive unfold of the
    decremented seed — `unfold (n+1) = (n+1) :< some (unfold n)`.

    THEOREM_MAP: `haft.cofree.unfold` -/
theorem unfold_succ (n : Nat) : unfold (n + 1) = Cofree.cons (n + 1) (unfold n) := rfl

end DeepCausalityFormal.Haft.Cofree
