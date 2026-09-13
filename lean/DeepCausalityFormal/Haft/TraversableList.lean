/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Traversable laws for the *sequential* carriers.

`Traversable.lean` proves the identity and naturality laws for `OptionWitness::sequence`, whose
carrier holds at most one element; there both proofs discharge by `cases x <;> rfl`. This file
covers the carriers that hold many: `VecWitness`, `DenseVectorWitness` and `CausalTensorWitness`,
whose `sequence` is a left-to-right accumulator fold. Their proofs are inductions over the
element list and are not instances of the `Option` ones.

Rust source: the `Traversable` impls in
  `deep_causality_unified_math/deep_causality_haft/src/extensions/hkt_vec_ext.rs`
  `deep_causality_unified_math/deep_causality_linear/src/extensions/hkt/dense_vector_witness.rs`
  `deep_causality_unified_math/deep_causality_tensor/src/extensions/ext_hkt.rs`
All three run the same fold, transcribed literally by `seqListAux` below:

    acc := M::pure(Vec::new());
    for m_a in fa { acc = M::apply(M::fmap(acc, |v| move |a| { v.push(a); v }), m_a) }

The three differ only in how the result is rebuilt — a `Vec`, a `DenseVector`, or a
`CausalTensor` re-wearing the input's shape — which is a bijection on the element list and so
does not affect the laws proved here. `seqList` is therefore the shared kernel of all three.

Accepted theory: C. McBride & R. Paterson, *Applicative programming with effects*, JFP 18(1),
2008 §3; law names follow Jaskelioff–Rypacek, *An investigation of the laws of traversals*,
MSFP 2012.

WHY `apply` IS PRESERVED HERE AND NOT IN `Traversable.lean`: `Option`'s `sequence` consumes only
`pure` and `fmap`, so its morphism record needs only those two. The accumulator fold also calls
`apply`, so naturality over it genuinely requires `preserves_apply`. That is a strictly stronger
hypothesis on the morphism, and it is discharged by the same applicative morphisms — it is part
of the standard definition (McBride–Paterson 2008 §6), which `Traversable.lean` omitted only
because its carrier never needed it.

The composition law remains deferred, as for `Option`: it needs lawful-applicative hypotheses for
both `M` and `N`. See THEOREM_MAP's deferred section, entry `haft.traversable.composition`.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witnesses:
  `deep_causality_unified_math/deep_causality_haft/tests/formalization_lean/traversable_list_tests.rs`

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks. Mirror any new import into
`cache_roots` in `//MODULE.bazel`. This file adds none.
-/

namespace DeepCausalityFormal.Haft.TraversableList

variable {A B : Type} {M N : Type → Type}

/-- The operations of an applicative functor, as a record (the Rust `M: Applicative<M> + HKT`
    bound). The sequential fold consumes all three, where `Option`'s `sequence` consumed two. -/
structure ApplicativeOps (M : Type → Type) where
  pure : {A : Type} → A → M A
  fmap : {A B : Type} → (A → B) → M A → M B
  apply : {A B : Type} → M (A → B) → M A → M B

/-- `VecWitness::fmap` — the outer functor, mapping the element list. -/
def listFmap (f : A → B) : List A → List B
  | [] => []
  | a :: as => f a :: listFmap f as

/-- The accumulator fold, transcribed from the Rust impls. `acc` threads through the loop; each
    step lifts "push this element" into `M` and applies it to the element's own effect. -/
def seqListAux (ops : ApplicativeOps M) : List (M A) → M (List A) → M (List A)
  | [], acc => acc
  | ma :: rest, acc =>
      seqListAux ops rest
        (ops.apply (ops.fmap (fun (v : List A) => fun (a : A) => v ++ [a]) acc) ma)

/-- `sequence` for the sequential carriers: start at `pure []` and fold left to right. -/
def seqList (ops : ApplicativeOps M) (xs : List (M A)) : M (List A) :=
  seqListAux ops xs (ops.pure [])

/-- An applicative morphism `φ : M → N` (McBride–Paterson 2008 §6). Unlike the record in
    `Traversable.lean`, this one also requires `preserves_apply`, because the fold calls `apply`. -/
structure ApplicativeMorphism (Mops : ApplicativeOps M) (Nops : ApplicativeOps N) where
  app : {A : Type} → M A → N A
  preserves_pure : ∀ {A : Type} (a : A), app (Mops.pure a) = Nops.pure a
  preserves_fmap : ∀ {A B : Type} (f : A → B) (x : M A),
    app (Mops.fmap f x) = Nops.fmap f (app x)
  preserves_apply : ∀ {A B : Type} (f : M (A → B)) (x : M A),
    app (Mops.apply f x) = Nops.apply (app f) (app x)

/-- The Identity applicative — `pure = id`, `fmap` and `apply` = application. -/
def idOps : ApplicativeOps (fun A => A) where
  pure := fun a => a
  fmap := fun f a => f a
  apply := fun f a => f a

/-- The fold at the Identity applicative appends the remaining elements to the accumulator.
    Generalising over `acc` is what makes the induction go through: the accumulator is not
    invariant across a step, so the statement has to quantify over it. -/
theorem seq_identity_aux (xs : List A) :
    ∀ acc : List A, seqListAux idOps xs acc = acc ++ xs := by
  induction xs with
  | nil => intro acc; simp [seqListAux]
  | cons a as ih =>
      intro acc
      show seqListAux idOps as (acc ++ [a]) = acc ++ a :: as
      rw [ih (acc ++ [a])]
      simp

/-- Traversable identity law for the sequential carriers: `sequence` at the Identity applicative
    is the identity (Jaskelioff–Rypacek 2012, law I).

    This is the law the Rust docstring states vacuously; see the DEVIATION NOTE in
    `Traversable.lean`. Here it also pins element order: a fold that reversed the list, or
    dropped an element, would fail it.

    THEOREM_MAP: `haft.traversable.list.identity` -/
theorem seq_identity (xs : List A) : seqList idOps xs = xs := by
  show seqListAux idOps xs [] = xs
  rw [seq_identity_aux xs []]
  simp

/-- Naturality generalised over the accumulator, for the same reason as `seq_identity_aux`. -/
theorem seq_naturality_aux {Mops : ApplicativeOps M} {Nops : ApplicativeOps N}
    (φ : ApplicativeMorphism Mops Nops) (xs : List (M A)) :
    ∀ acc : M (List A),
      φ.app (seqListAux Mops xs acc) = seqListAux Nops (listFmap φ.app xs) (φ.app acc) := by
  induction xs with
  | nil => intro acc; rfl
  | cons ma rest ih =>
      intro acc
      show φ.app (seqListAux Mops rest (Mops.apply (Mops.fmap _ acc) ma))
         = seqListAux Nops (listFmap φ.app rest)
             (Nops.apply (Nops.fmap _ (φ.app acc)) (φ.app ma))
      rw [ih]
      rw [φ.preserves_apply, φ.preserves_fmap]

/-- Traversable naturality for the sequential carriers: every applicative morphism `φ` commutes
    with `sequence` — `φ (sequence_M xs) = sequence_N (fmap φ xs)` (Jaskelioff–Rypacek 2012,
    law N).

    THEOREM_MAP: `haft.traversable.list.naturality` -/
theorem seq_naturality {Mops : ApplicativeOps M} {Nops : ApplicativeOps N}
    (φ : ApplicativeMorphism Mops Nops) (xs : List (M A)) :
    φ.app (seqList Mops xs) = seqList Nops (listFmap φ.app xs) := by
  show φ.app (seqListAux Mops xs (Mops.pure []))
     = seqListAux Nops (listFmap φ.app xs) (Nops.pure [])
  rw [seq_naturality_aux φ xs (Mops.pure []), φ.preserves_pure]

/-- Element count is preserved, generalised over the accumulator. -/
theorem seq_length_aux (xs : List A) :
    ∀ acc : List A, (seqListAux idOps xs acc).length = acc.length + xs.length := by
  induction xs with
  | nil => intro acc; simp [seqListAux]
  | cons a as ih =>
      intro acc
      show (seqListAux idOps as (acc ++ [a])).length = acc.length + (a :: as).length
      rw [ih (acc ++ [a])]
      simp
      omega

/-- `sequence` preserves the element count.

    This is the formal content of the shape claim `CausalTensorWitness::sequence` makes. A tensor
    carries a shape independent of its length, and `sequence` re-wears the input's shape on the
    output; that is sound exactly because the element count is unchanged, which is what this
    theorem states. `bind` has no counterpart, because its continuation may return any number of
    elements — the asymmetry recorded on the Rust impl.

    THEOREM_MAP: `haft.traversable.list.length_preserved` -/
theorem seq_length (xs : List A) : (seqList idOps xs).length = xs.length := by
  show (seqListAux idOps xs []).length = xs.length
  rw [seq_length_aux xs []]
  simp

end DeepCausalityFormal.Haft.TraversableList
