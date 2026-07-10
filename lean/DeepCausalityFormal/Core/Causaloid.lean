/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — Causaloid: the signature functor `F` and the fixpoint `Causaloid ≅ μX.F(X)` (Stage 2 of the
causaloid formalization roadmap).

Textbook definition: an inductive datatype is the **initial algebra** of its signature functor —
the least fixpoint `μX.F(X)` — and by Lambek's lemma the structure map is an isomorphism
(Bird & de Moor, *Algebra of Programming*, 1997, ch. 2; Lambek 1968). Here
`F(X) = Atom(α) + Coll(Bag X, AggLogic) + Graph(Hyper X, Λ-edges)` and the fixpoint theorem is the
`roll`/`unroll` isomorphism. Initiality's *uniqueness* half (the catamorphism theorem) is Stage 5
(`Core/Catamorphism.lean`), not this file.

The inversion (Hardy, *Probability Theories with Dynamic Causal Structure*, arXiv:gr-qc/0509120,
Eq. 2 p. 4): Hardy composes regions with one symmetric product ⊗^Λ and puts the asymmetry in the
theory-specific Λ matrices. DeepCausality factors the same content the other way around — the
**element** stays symmetric (it sees values with intrinsic identity, never a position or a
before/after) and the asymmetry lives in the **wiring**. `eval_factors` states this as an equation:
evaluation = (element-independent wiring) ∘ (wiring-independent element map), and `mapL_perm` shows
the element layer preserves bag equivalence (no order enters through the elements).

Rust source: `deep_causality/src/types/causal_types/causaloid/mod.rs` (the `Causaloid` struct —
the sealed three-form surface, assumption #11a), `causal_type.rs` (`CausaloidType`),
`causaloid_graph/lambda_edges.rs` (the identity-keyed Λ slots).

Deviations:
 * D-fix-1: Rust enforces finiteness by construction (values are built bottom-up and `Arc`-shared;
   the sealed surface admits no other builder), the model by the inductive datatype itself — μX,
   not νX. `size` (a total structural measure) is the explicit well-foundedness witness.
 * D-fix-2: the `Coll` bag is a `CList` (a list inlined mutually with `Causaloid`, so all recursion
   is structural in bare Lean) taken up to the `Perm` relation — Bag = CList/Perm.
 * D-fix-3: Λ-edges are a keyed function `Nat → Nat → Option Λ` — intrinsically
   enumeration-order-free (a function has no entry order). The Rust realization is the
   identity-keyed `LambdaEdges` map; its order-freeness is witnessed in Rust.
 * D-fix-4: Stage-2 evaluation gives `graph` the undecorated wiring — the Λ data is carried but not
   yet applied, exactly matching the Rust engine at this stage. The decorated join `∇ ∘ (Λ ⊗ Λ)`
   is Stage 4 (`Core/GraphAlgebra.lean`).

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality/tests/formalization_lean/causaloid_tests.rs`.
-/

namespace DeepCausalityFormal.Core.Causaloid

/-- `AggregateLogic` (Rust `deep_causality/src/types/causal_types/aggregate_logic/`):
    `All`/`Any`/`None`/`Some(k)`. Carried by the `coll` summand; its closure over the Verdict
    algebra is Stage 3 (`Core/VerdictClosure.lean`). -/
inductive AggLogic where
  | all   : AggLogic
  | any   : AggLogic
  | none  : AggLogic
  | someK : Nat → AggLogic

mutual
  /-- The causaloid — the least fixpoint `μX.F(X)`. The three constructors correspond one-to-one
      to the sealed `CausaloidType` forms: `atom` ↔ `Singleton`, `coll` ↔ `Collection`,
      `graph` ↔ `Graph` (assumption #11a: exactly these three, enforced by the sealed trait). -/
  inductive Causaloid (α Λ : Type) where
    | atom  : α → Causaloid α Λ
    | coll  : CList α Λ → AggLogic → Causaloid α Λ
    | graph : CList α Λ → (Nat → Nat → Option Λ) → Causaloid α Λ

  /-- The children bag, inlined as a mutual list so every recursion below is structural.
      A Bag is a `CList` up to `Perm` (D-fix-2). -/
  inductive CList (α Λ : Type) where
    | nil  : CList α Λ
    | cons : Causaloid α Λ → CList α Λ → CList α Λ
end

variable {α β Λ V : Type}

/-- The signature functor `F(X) = Atom(α) + Coll(Bag X, AggLogic) + Graph(Hyper X, Λ-edges)`,
    with `X` the recursion variable. Λ-edges are keyed by intrinsic edge identity
    (`Nat → Nat → Option Λ`), never by enumeration order (D-fix-3). -/
inductive F (α Λ X : Type) where
  | atom  : α → F α Λ X
  | coll  : List X → AggLogic → F α Λ X
  | graph : List X → (Nat → Nat → Option Λ) → F α Λ X

/-- `CList → List` (one unroll step of the children). -/
def toList : CList α Λ → List (Causaloid α Λ)
  | .nil       => []
  | .cons c cs => c :: toList cs

/-- `List → CList` (one roll step of the children). -/
def ofList : List (Causaloid α Λ) → CList α Λ
  | []      => .nil
  | c :: cs => .cons c (ofList cs)

theorem to_of_list (l : List (Causaloid α Λ)) : toList (ofList l) = l := by
  induction l with
  | nil => rfl
  | cons c cs ih => simp [ofList, toList, ih]

theorem of_to_list : (cs : CList α Λ) → ofList (toList cs) = cs
  | .nil       => rfl
  | .cons c cs => by simp [toList, ofList, of_to_list cs]

/-- The structure map (the algebra) `roll : F(Causaloid) → Causaloid`. -/
def roll : F α Λ (Causaloid α Λ) → Causaloid α Λ
  | .atom a       => .atom a
  | .coll xs g    => .coll (ofList xs) g
  | .graph xs lam => .graph (ofList xs) lam

/-- One unfolding step `unroll : Causaloid → F(Causaloid)`. -/
def unroll : Causaloid α Λ → F α Λ (Causaloid α Λ)
  | .atom a        => .atom a
  | .coll cs g     => .coll (toList cs) g
  | .graph cs lam  => .graph (toList cs) lam

/-- The fixpoint, one direction: `roll ∘ unroll = id`. With `unroll_roll` this is the Lambek
    isomorphism `Causaloid ≅ F(Causaloid)` — the causaloid IS the fixpoint of its signature
    functor, and it is the least (μ) fixpoint because the carrier is an inductive type
    (well-foundedness witness: `size`).

    THEOREM_MAP: `core.causaloid.fixpoint` -/
theorem roll_unroll (c : Causaloid α Λ) : roll (unroll c) = c := by
  cases c <;> simp [roll, unroll, of_to_list]

/-- The fixpoint, other direction: `unroll ∘ roll = id`.

    THEOREM_MAP: `core.causaloid.fixpoint` -/
theorem unroll_roll (x : F α Λ (Causaloid α Λ)) : unroll (roll x) = x := by
  cases x <;> simp [roll, unroll, to_of_list]

mutual
  /-- A total structural size — its very definition (accepted by structural recursion) is the
      well-foundedness witness: every causaloid tree is finite (μX, not νX; D-fix-1). -/
  def size : Causaloid α Λ → Nat
    | .atom _     => 1
    | .coll cs _  => sizeList cs + 1
    | .graph cs _ => sizeList cs + 1

  def sizeList : CList α Λ → Nat
    | .nil       => 0
    | .cons c cs => size c + sizeList cs
end

/-- Every causaloid has positive, finite size — the explicit μ-witness.

    THEOREM_MAP: `core.causaloid.fixpoint` -/
theorem size_pos (c : Causaloid α Λ) : 0 < size c := by
  cases c with
  | atom a       => exact Nat.succ_pos 0
  | coll cs g    => exact Nat.succ_pos (sizeList cs)
  | graph cs lam => exact Nat.succ_pos (sizeList cs)

-- ------------------------------------------------------------------
-- Bag = CList up to permutation (D-fix-2).
-- ------------------------------------------------------------------

/-- Permutation of a children bag (the standard swap-generated presentation). -/
inductive Perm : CList α Λ → CList α Λ → Prop where
  | nil   : Perm .nil .nil
  | cons  (c : Causaloid α Λ) {cs cs' : CList α Λ} :
      Perm cs cs' → Perm (.cons c cs) (.cons c cs')
  | swap  (a b : Causaloid α Λ) (cs : CList α Λ) :
      Perm (.cons a (.cons b cs)) (.cons b (.cons a cs))
  | trans {cs₁ cs₂ cs₃ : CList α Λ} :
      Perm cs₁ cs₂ → Perm cs₂ cs₃ → Perm cs₁ cs₃

theorem Perm.refl : (cs : CList α Λ) → Perm cs cs
  | .nil       => .nil
  | .cons c cs => .cons c (Perm.refl cs)

/-- Size is a bag invariant: permutation preserves it. -/
theorem sizeList_perm {cs cs' : CList α Λ} (h : Perm cs cs') : sizeList cs = sizeList cs' := by
  induction h with
  | nil => rfl
  | cons c _ ih => simp [sizeList, ih]
  | swap a b cs => simp [sizeList]; omega
  | trans _ _ ih₁ ih₂ => exact ih₁.trans ih₂

-- ------------------------------------------------------------------
-- The element layer and the Hardy inversion.
-- ------------------------------------------------------------------

mutual
  /-- The element layer: map the atoms' local data, preserving all structure — the functorial
      action of the fixpoint on the element payload. This map is pointwise: it takes no order,
      position, or neighbour argument. -/
  def mapC (f : α → β) : Causaloid α Λ → Causaloid β Λ
    | .atom a       => .atom (f a)
    | .coll cs g    => .coll (mapL f cs) g
    | .graph cs lam => .graph (mapL f cs) lam

  def mapL (f : α → β) : CList α Λ → CList β Λ
    | .nil       => .nil
    | .cons c cs => .cons (mapC f c) (mapL f cs)
end

/-- The element layer is symmetric: it preserves bag equivalence, so no ordering asymmetry can
    enter through the elements — order can only live in the wiring. (One half of the inversion;
    the other half is the factorization `eval_factors`.)

    THEOREM_MAP: `core.causaloid.inversion` -/
theorem mapL_perm (f : α → β) {cs cs' : CList α Λ} (h : Perm cs cs') :
    Perm (mapL f cs) (mapL f cs') := by
  induction h with
  | nil => exact .nil
  | cons c _ ih => exact .cons (mapC f c) ih
  | swap a b cs => exact .swap (mapC f a) (mapC f b) (mapL f cs)
  | trans _ _ ih₁ ih₂ => exact .trans ih₁ ih₂

mutual
  /-- Stage-2 evaluation: atoms through the element semantics `elemSem`; `coll` and `graph` as the
      ∇-fold of their children (`graph`'s Λ data is carried, not yet applied — D-fix-4; the
      decorated join is Stage 4). -/
  def eval (nabla : V → V → V) (elemSem : α → V → V) : Causaloid α Λ → V → V
    | .atom a     => elemSem a
    | .coll cs _  => fun v => evalL nabla elemSem cs v
    | .graph cs _ => fun v => evalL nabla elemSem cs v

  def evalL (nabla : V → V → V) (elemSem : α → V → V) : CList α Λ → V → V
    | .nil       => fun v => v
    | .cons c cs => fun v => nabla (eval nabla elemSem c v) (evalL nabla elemSem cs v)
end

mutual
  /-- The wiring layer: the same composition shape, defined over causaloids whose atoms already
      carry their semantic map `V → V`. It never inspects an element beyond applying it — it is
      element-independent. -/
  def wiring (nabla : V → V → V) : Causaloid (V → V) Λ → V → V
    | .atom g     => g
    | .coll cs _  => fun v => wiringL nabla cs v
    | .graph cs _ => fun v => wiringL nabla cs v

  def wiringL (nabla : V → V → V) : CList (V → V) Λ → V → V
    | .nil       => fun v => v
    | .cons c cs => fun v => nabla (wiring nabla c v) (wiringL nabla cs v)
end

mutual
  /-- **The Hardy inversion, formal**: evaluation factors as
      (element-independent wiring) ∘ (wiring-independent element map). The element map `mapC` is
      pointwise and symmetric (`mapL_perm`); every asymmetry of `eval` therefore lives in the
      wiring layer. Hardy: one symmetric product, asymmetric Λ. DeepCausality: symmetric elements,
      asymmetric composition. Same content, opposite factorization.

      THEOREM_MAP: `core.causaloid.inversion` -/
  theorem eval_factors (nabla : V → V → V) (elemSem : α → V → V) :
      (c : Causaloid α Λ) → (v : V) →
        eval nabla elemSem c v = wiring nabla (mapC elemSem c) v
    | .atom a, _v => rfl
    | .coll cs g, v => by
        simp only [eval, wiring, mapC]
        exact evalL_factors nabla elemSem cs v
    | .graph cs lam, v => by
        simp only [eval, wiring, mapC]
        exact evalL_factors nabla elemSem cs v

  theorem evalL_factors (nabla : V → V → V) (elemSem : α → V → V) :
      (cs : CList α Λ) → (v : V) →
        evalL nabla elemSem cs v = wiringL nabla (mapL elemSem cs) v
    | .nil, _v => rfl
    | .cons c cs, v => by
        simp only [evalL, wiringL, mapL]
        rw [eval_factors nabla elemSem c v, evalL_factors nabla elemSem cs v]
end

end DeepCausalityFormal.Core.Causaloid
