/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Traversable laws.

Rust source: `deep_causality_haft/src/traversable/mod.rs` (trait
`Traversable<F>: Functor<F> + Foldable<F>`, operation `sequence`). Canonical carrier: `Option`
(`OptionWitness::sequence`: `Some m_a → M::fmap(m_a, Some)`, `None → M::pure(None)`) —
transcribed literally by `seqOption` below, over an *abstract* applicative `M` given as an
operations record, so the theorems quantify over every applicative the Rust code could see.

Accepted theory: C. McBride & R. Paterson, *Applicative programming with effects*, JFP 18(1),
2008 §3 (traversals); law names follow the standard formulation (e.g. Jaskelioff–Rypacek,
*An investigation of the laws of traversals*, MSFP 2012):
  1. Naturality: for every applicative morphism `φ : M → N`,
     `φ ∘ sequence_M = sequence_N ∘ fmap φ`.
  2. Identity: `sequence` at the Identity applicative is the identity.
  3. Composition: `sequence` at a composite applicative `M ∘ N` is the composite of sequences.

DEVIATION NOTE: the Rust docstring's "Identity" law — `t.sequence == t.map(id).sequence` — is
vacuous (`map id = id` makes both sides syntactically equal by the Functor law; it constrains
nothing). The accepted identity law, proved below, runs `sequence` at the **Identity
applicative**. Recommendation: fix the docstring. The composition law (3) is stated in the docs
and holds for this carrier, but is deferred here (it needs lawful-applicative hypotheses for
both `M` and `N`; tracked in THEOREM_MAP's deferred section).

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/traversable_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Traversable

variable {A B : Type} {M N : Type → Type}

/-- `OptionWitness::fmap` (see `Functor.lean`). -/
def optFmap (f : A → B) : Option A → Option B
  | some a => some (f a)
  | none => none

/-- The operations of an applicative functor, as a record (the Rust `M: Applicative<M> + HKT`
    bound). Only `pure` and `fmap` are consumed by `Option`'s `sequence`. -/
structure ApplicativeOps (M : Type → Type) where
  pure : {A : Type} → A → M A
  fmap : {A B : Type} → (A → B) → M A → M B

/-- `OptionWitness::sequence`, transcribed: flip `Option (M A)` into `M (Option A)`. -/
def seqOption (ops : ApplicativeOps M) : Option (M A) → M (Option A)
  | some ma => ops.fmap some ma
  | none => ops.pure none

/-- An applicative morphism `φ : M → N` — a natural transformation preserving the applicative
    structure (McBride–Paterson 2008 §6, "applicative morphisms"; only the `pure`/`fmap`
    preservation is consumed here). -/
structure ApplicativeMorphism (Mops : ApplicativeOps M) (Nops : ApplicativeOps N) where
  app : {A : Type} → M A → N A
  preserves_pure : ∀ {A : Type} (a : A), app (Mops.pure a) = Nops.pure a
  preserves_fmap : ∀ {A B : Type} (f : A → B) (x : M A),
    app (Mops.fmap f x) = Nops.fmap f (app x)

/-- The Identity applicative — `pure = id`, `fmap = application`. -/
def idOps : ApplicativeOps (fun A => A) where
  pure := fun a => a
  fmap := fun f a => f a

/-- Traversable identity law (the accepted form): `sequence` at the Identity applicative is
    the identity (Jaskelioff–Rypacek 2012, law I). This replaces the vacuous docstring version.

    THEOREM_MAP: `haft.traversable.identity` -/
theorem seq_identity (x : Option A) : seqOption idOps x = x := by
  cases x <;> rfl

/-- Traversable naturality: every applicative morphism `φ` commutes with `sequence` —
    `φ (sequence_M x) = sequence_N (fmap φ x)` (Jaskelioff–Rypacek 2012, law N; the Rust
    docstring's law 1 is the special case where `φ` is post-composition).

    THEOREM_MAP: `haft.traversable.naturality` -/
theorem seq_naturality {Mops : ApplicativeOps M} {Nops : ApplicativeOps N}
    (φ : ApplicativeMorphism Mops Nops) (x : Option (M A)) :
    φ.app (seqOption Mops x) = seqOption Nops (optFmap φ.app x) := by
  cases x with
  | some ma => exact φ.preserves_fmap some ma
  | none => exact φ.preserves_pure none

end DeepCausalityFormal.Haft.Traversable
