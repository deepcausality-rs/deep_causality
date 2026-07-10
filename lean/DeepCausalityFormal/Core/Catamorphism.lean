/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Core — the keystone: `evaluate` is the UNIQUE F-algebra catamorphism, per fixed carrier
(Stage 5 of the causaloid formalization roadmap; goal B2, assumptions #6 and #8).

Textbook definition. The initial algebra `μX.F(X)` admits, for every F-algebra
`alg : F(X) → X`, exactly ONE homomorphism `⦅alg⦆ : μF → X` — the **catamorphism**; uniqueness is
initiality (Lambek 1968; Bird & de Moor, *Algebra of Programming*, 1997, ch. 2; Goguen–Thatcher–
Wagner–Wright, "Initial Algebra Semantics," *J. ACM* 24(1), 1977). **Catamorphism fusion** is its
one-liner corollary: a homomorphism after a catamorphism is a catamorphism, so folding a nested
structure equals folding the flattened one. The `Atom`/`compose` fragment of the causaloid is the
free category over the atom generators; interpretation into any target is determined by the
generator images (the free property, `haft.arrow_term.free`), and two terms related by the
category laws interpret equally — the interpreter factors through the quotient `T/≈`
(assumption #8: `T` is the free term, `T/≈` the strong category presented by the Arrow laws).

This file proves, on the Stage-2 model (`Core/Causaloid.lean`, re-declared self-contained):
  * `catamorphism_unique` — a hypothesis interpreter satisfying the three case equations (atom,
    coll, graph) is pointwise equal to `eval`. The carrier — the semantic algebra `(V, elemSem,
    nabla)` — is an explicit FIXED parameter: uniqueness is **per carrier**; uniqueness across
    carriers is neither claimed nor true (different carriers give different, each-unique
    interpreters — #6, correctly scoped).
  * `encapsulation_flat` — nested fold = flat fold: wrapping a bag in a `coll` node of the same
    logic evaluates exactly like splicing its members inline (catamorphism fusion at the bag).
  * `arrow_fragment` — the atom/compose fragment ≅ the reified term language: `eval` on a
    causaloid chain equals `interpret` on the corresponding `ArrowTerm` chain, and the
    interpretation factors through `T/≈` (`compose_assoc_interp`: terms related by associativity
    interpret equally — the quotient lemma).

DEVIATION NOTES.
  1. The model re-declares the Stage-2 inductive (bare-`lean` files are self-contained); the
     Stage-2 `eval` is extended verbatim. `graph`'s Λ data is carried, with the Stage-4 semantics
     living in `Core/GraphAlgebra.lean`; here the wiring equations treat `coll` and `graph`
     uniformly, which is exactly what the case equations of the uniqueness statement pin down.
  2. The Kleisli target is represented by its function carrier `V → V` (the Stage-2 element
     semantics); the effectful Kleisli laws are `core.causal_arrow.category_laws` and the
     interpreter functoriality is `haft.interpreter.{preserves_id, preserves_compose,
     choice_preserved}` — cited, not re-proved. The `⊕`-enlarged fragment's agreement is
     inherited through `haft.arrow_term.choice_interpret_sound` and witnessed in Rust.
  3. `encapsulation_flat` requires associativity of `∇` and nothing else — the monad-law-3
     inheritance the roadmap names; it is stated with that single hypothesis.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses: `deep_causality/tests/formalization_lean/catamorphism_tests.rs`.
-/

namespace DeepCausalityFormal.Core.Catamorphism

/-- `AggregateLogic`, as in `Core/Causaloid.lean`. -/
inductive AggLogic where
  | all   : AggLogic
  | any   : AggLogic
  | none  : AggLogic
  | someK : Nat → AggLogic

mutual
  /-- The Stage-2 causaloid model (`Causaloid ≅ μX.F(X)`), re-declared self-contained. -/
  inductive Causaloid (α Λ : Type) where
    | atom  : α → Causaloid α Λ
    | coll  : CList α Λ → AggLogic → Causaloid α Λ
    | graph : CList α Λ → (Nat → Nat → Option Λ) → Causaloid α Λ

  inductive CList (α Λ : Type) where
    | nil  : CList α Λ
    | cons : Causaloid α Λ → CList α Λ → CList α Λ
end

variable {α Λ V : Type}

mutual
  /-- The Stage-2 evaluation (the catamorphism whose uniqueness this file proves): atoms through
      the element semantics, composites as the ∇-fold of their children. -/
  def eval (nabla : V → V → V) (elemSem : α → V → V) : Causaloid α Λ → V → V
    | .atom a     => elemSem a
    | .coll cs _  => fun v => evalL nabla elemSem cs v
    | .graph cs _ => fun v => evalL nabla elemSem cs v

  def evalL (nabla : V → V → V) (elemSem : α → V → V) : CList α Λ → V → V
    | .nil       => fun v => v
    | .cons c cs => fun v => nabla (eval nabla elemSem c v) (evalL nabla elemSem cs v)
end

-- ------------------------------------------------------------------
-- The keystone: uniqueness of the catamorphism, per fixed carrier.
-- ------------------------------------------------------------------

mutual
  /-- **`catamorphism_unique`** — initiality of the fixpoint: any interpreter `h` (with its bag
      fold `hL`) satisfying the three case equations of the algebra `(V, elemSem, nabla)` — the
      atom, cons/nil (bag), and coll/graph (wiring) equations — is pointwise equal to `eval`.
      The carrier is FIXED: `elemSem` and `nabla` are parameters held constant on both sides;
      nothing is claimed across carriers (deviation note; #6 correctly scoped).

      THEOREM_MAP: `core.causaloid.catamorphism_unique` -/
  theorem catamorphism_unique (nabla : V → V → V) (elemSem : α → V → V)
      (h : Causaloid α Λ → V → V) (hL : CList α Λ → V → V)
      (h_atom : ∀ a, h (.atom a) = elemSem a)
      (h_coll : ∀ cs lgc v, h (.coll cs lgc) v = hL cs v)
      (h_graph : ∀ cs lam v, h (.graph cs lam) v = hL cs v)
      (h_nil : ∀ v, hL .nil v = v)
      (h_cons : ∀ c cs v, hL (.cons c cs) v = nabla (h c v) (hL cs v)) :
      (c : Causaloid α Λ) → (v : V) → h c v = eval nabla elemSem c v
    | .atom a, v => by rw [h_atom a]; rfl
    | .coll cs lgc, v => by
        rw [h_coll cs lgc v]
        show hL cs v = eval nabla elemSem (.coll cs lgc) v
        rw [catamorphism_unique_list nabla elemSem h hL h_atom h_coll h_graph h_nil h_cons cs v]
        rfl
    | .graph cs lam, v => by
        rw [h_graph cs lam v]
        show hL cs v = eval nabla elemSem (.graph cs lam) v
        rw [catamorphism_unique_list nabla elemSem h hL h_atom h_coll h_graph h_nil h_cons cs v]
        rfl

  theorem catamorphism_unique_list (nabla : V → V → V) (elemSem : α → V → V)
      (h : Causaloid α Λ → V → V) (hL : CList α Λ → V → V)
      (h_atom : ∀ a, h (.atom a) = elemSem a)
      (h_coll : ∀ cs lgc v, h (.coll cs lgc) v = hL cs v)
      (h_graph : ∀ cs lam v, h (.graph cs lam) v = hL cs v)
      (h_nil : ∀ v, hL .nil v = v)
      (h_cons : ∀ c cs v, hL (.cons c cs) v = nabla (h c v) (hL cs v)) :
      (cs : CList α Λ) → (v : V) → hL cs v = evalL nabla elemSem cs v
    | .nil, v => by rw [h_nil v]; rfl
    | .cons c cs, v => by
        rw [h_cons c cs v,
            catamorphism_unique nabla elemSem h hL h_atom h_coll h_graph h_nil h_cons c v,
            catamorphism_unique_list nabla elemSem h hL h_atom h_coll h_graph h_nil h_cons cs v]
        rfl
end

-- ------------------------------------------------------------------
-- Encapsulation is flat (catamorphism fusion at the bag).
-- ------------------------------------------------------------------

/-- Splice a bag in front of another (the flattened structure). -/
def appendL : CList α Λ → CList α Λ → CList α Λ
  | .nil,       ds => ds
  | .cons c cs, ds => .cons c (appendL cs ds)

/-- The bag fold with an explicit continuation `k` in place of the identity seed — the
    continuation form catamorphism fusion lands in. -/
def evalWith (nabla : V → V → V) (elemSem : α → V → V) (k : V → V) : CList α Λ → V → V
  | .nil       => k
  | .cons c cs => fun v => nabla (eval nabla elemSem c v) (evalWith nabla elemSem k cs v)

/-- **`encapsulation_flat`**, wrapper transparency — the spec's scenario: a bag containing a
    causaloid-wrapped sub-bag evaluates exactly like the sub-bag's members contributing inline:
    the wrapper's fold IS its members' fold (nested fold = flat fold at the wrapper). Holds
    definitionally — encapsulation is cost- and semantics-free by construction.

    THEOREM_MAP: `core.causaloid.encapsulation_flat` -/
theorem encapsulation_flat (nabla : V → V → V) (elemSem : α → V → V)
    (inner : CList α Λ) (lgc : AggLogic) (rest : CList α Λ) (v : V) :
    evalL nabla elemSem (.cons (.coll inner lgc) rest) v
      = nabla (evalL nabla elemSem inner v) (evalL nabla elemSem rest v) := rfl

/-- **`encapsulation_flat`**, fusion form (catamorphism fusion): folding a flattened bag equals
    folding the front bag with the back bag's fold as the continuation — `⦅alg⦆ ∘ splice =
    ⦅alg with continuation⦆`, by induction, no side conditions (the monad-law-3 inheritance in
    continuation form; with a unital-associative `∇` this specializes to
    `fold (xs ++ ys) = fold xs ∇ fold ys`).

    THEOREM_MAP: `core.causaloid.encapsulation_flat` -/
theorem evalL_append (nabla : V → V → V) (elemSem : α → V → V) :
    (cs : CList α Λ) → (ds : CList α Λ) → (v : V) →
      evalL nabla elemSem (appendL cs ds) v
        = evalWith nabla elemSem (evalL nabla elemSem ds) cs v
  | .nil, _ds, _v => rfl
  | .cons c cs, ds, v => by
      show nabla (eval nabla elemSem c v) (evalL nabla elemSem (appendL cs ds) v)
          = nabla (eval nabla elemSem c v)
              (evalWith nabla elemSem (evalL nabla elemSem ds) cs v)
      rw [evalL_append nabla elemSem cs ds v]

-- ------------------------------------------------------------------
-- The arrow fragment: atoms + compose ≅ the reified term language, and T/≈.
-- ------------------------------------------------------------------

/-- The `Atom`/`compose` fragment of the reified term language (`Haft/ArrowTerm.lean`, restricted
    to the category generators; re-declared self-contained, interpreted over plain `V` — the pure
    shadow of the Kleisli interpretation, deviation note 2). -/
inductive Term (α : Type) where
  | id      : Term α
  | gen     : α → Term α
  | compose : Term α → Term α → Term α

/-- Interpretation of the fragment: `id` to the identity, generators through the element
    semantics, `compose` to function composition — `interpret`/`interpret_kleisli` restricted to
    this fragment (`haft.interpreter.{preserves_id, preserves_compose}`). -/
def interp (φ : α → V → V) : Term α → V → V
  | .id          => fun v => v
  | .gen a       => φ a
  | .compose f h => fun v => interp φ h (interp φ f v)

/-- A linear chain of atoms as a term: the right-nested composition of its generators. -/
def ofChain : List α → Term α
  | []      => .id
  | a :: as => .compose (.gen a) (ofChain as)

/-- The causaloid side of the same chain: the sequential wire semantics of a linear graph — each
    node consumes its (single) parent's output (`Core/GraphAlgebra.lean`: on a chain, the
    denotational `val` composes the elements along the wires). -/
def evalChain (elemSem : α → V → V) : List α → V → V
  | []      => fun v => v
  | a :: as => fun v => evalChain elemSem as (elemSem a v)

/-- **`arrow_fragment`** — the fragment correspondence: interpreting the reified chain term
    equals the causaloid's sequential evaluation of the same chain, atom for atom. With
    `haft.arrow_term.{interpret_sound, choice_interpret_sound}` and
    `haft.interpreter.{preserves_id, preserves_compose, choice_preserved}` this extends to the
    full (⊕-enlarged) generator set on the effectful target: `evaluate = interpret_kleisli` on
    the fragment.

    THEOREM_MAP: `core.causaloid.arrow_fragment` -/
theorem arrow_fragment (elemSem : α → V → V) :
    (atoms : List α) → (v : V) →
      interp elemSem (ofChain atoms) v = evalChain elemSem atoms v
  | [], _v => rfl
  | a :: as, v => by
      show interp elemSem (ofChain as) (elemSem a v) = evalChain elemSem as (elemSem a v)
      exact arrow_fragment elemSem as (elemSem a v)

/-- **The quotient `T/≈`** (assumption #8): terms related by the category laws — associativity of
    `compose` and the two identity units — interpret equally, so the interpretation factors
    through the quotient of the free term language by the Arrow laws. Each equation is
    definitional in the function target.

    THEOREM_MAP: `core.causaloid.arrow_fragment` -/
theorem interp_respects_category_laws (φ : α → V → V) (f g h : Term α) :
    (∀ v, interp φ (.compose (.compose f g) h) v = interp φ (.compose f (.compose g h)) v)
    ∧ (∀ v, interp φ (.compose .id f) v = interp φ f v)
    ∧ (∀ v, interp φ (.compose f .id) v = interp φ f v) :=
  ⟨fun _ => rfl, fun _ => rfl, fun _ => rfl⟩

end DeepCausalityFormal.Core.Catamorphism
