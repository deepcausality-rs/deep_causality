/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — the HKT witness pattern: what it denotes.

Rust source: `deep_causality_haft/src/hkt/mod.rs` (traits `HKT`, `HKT2..5`, `HKT2..6Unbound`,
`Satisfies`, `NoConstraint`, `Placeholder`).

This file is the definitional bridge between the Rust encoding and the mathematics — it
contains `def`s and `example`s (checked by the compiler), not theorems, because an encoding has
nothing to prove; it has a *denotation*:

  Rust                                  | Mathematics / Lean
  --------------------------------------|--------------------------------------------
  witness type `W: HKT`                 | a type constructor `F : Type → Type` (native)
  GAT projection `W::Type<T>`           | application `F T`
  `W: HKT2Unbound` (`Type<A, B>`)       | `F : Type → Type → Type`
  fixed-arity `HKT3<F1, F2>`            | PARTIAL APPLICATION of a multi-arg constructor
  `Satisfies<C>` bound                  | a typeclass/`Prop` constraint on the argument
  trait impl on the witness             | an operations record / instance for `F`

The witness pattern is *defunctionalization at the type level* (J. Reynolds, *Definitional
Interpreters for Higher-Order Programming Languages*, ACM '72): Rust generics cannot abstract
over type constructors (no kind `* → *` parameters), so each constructor is named by a
zero-sized proxy and re-projected through a GAT. In Lean, constructors are first-class and the
entire apparatus disappears — which is the precise sense in which the pattern is
"Rust-necessitated, mathematically transparent". Every other file in this directory exploits
this: it models `W::Type<T>` directly as `F T` and the trait impl as defs on `F`.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.
-/

namespace DeepCausalityFormal.Haft.Hkt

/-- `HKT2` (fix the SECOND slot of a binary constructor, leaving one hole): partial
    application. E.g. `ResultWitness<E>: HKT2<E>` with `Type<T> = Result<T, E>`. -/
def fixSnd (F : Type → Type → Type) (E : Type) : Type → Type := fun T => F T E

/-- `HKT3<F1, F2>` (fix two slots of a ternary constructor): iterated partial application,
    the shape `Effect3` bridges (see `EffectSystem.lean`). -/
def fixTwo (F : Type → Type → Type → Type) (F1 F2 : Type) : Type → Type :=
  fun T => F T F1 F2

/-- The projection is literal: `ResultWitness<String>::Type<Nat>` denotes `Except String Nat`
    (Rust `Result<T, E>` ≙ Lean `Except E T`; argument order is presentation only). -/
example : fixSnd (fun T E => Except E T) String Nat = Except String Nat := rfl

/-- Partial application commutes with projection — fixing channels then applying the hole is
    applying the full constructor. This is the entire semantic content of the `Effect3/4/5`
    bridge traits. -/
example (F : Type → Type → Type → Type) (F1 F2 T : Type) :
    fixTwo F F1 F2 T = F T F1 F2 := rfl

/-- `NoConstraint`: the trivially-true constraint — every type satisfies it. -/
def NoConstraint (_ : Type) : Prop := True

/-- Every type `Satisfies<NoConstraint>` — the blanket impl `impl<T> Satisfies<NoConstraint>
    for T {}`, as a one-line proof. -/
example (T : Type) : NoConstraint T := trivial

end DeepCausalityFormal.Haft.Hkt
