/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — the one-way interpreter `ArrowTerm → Kleisli<M>` as a functor, plus naturality.

Rust source: `deep_causality_haft/src/arrow/interpreter.rs` (`ArrowCore::interpret_kleisli`) and
`deep_causality_haft/src/natural_transformation/mod.rs` (`NaturalTransformation`, `OptionToVec`).

Textbook definition. The free arrow over a generator set is an initial object: for any target
strong category (arrow), an interpretation of the generators extends to a **unique functor** from
the free arrow to the target, preserving identities and composition (John Hughes, "Generalising
Monads to Arrows," *Sci. Comput. Program.* 37(1–3), 2000; Steve Awodey, *Category Theory* 2nd ed.,
§1.4 functors and §5 free/initial constructions). The Rust interpreter's target is the **Kleisli
category** of an effect monad `M` (Mac Lane, *CWM*, §VI.5; Moggi, "Notions of Computation and
Monads," *Inf. and Comput.* 93(1), 1991), where `id = pure` and `compose = bind`. A
**natural transformation** `η : F ⇒ G` between functors is a family of components commuting with
mapping — the naturality square `η ∘ fmap f = fmap f ∘ η` (Mac Lane, *CWM*, §I.4).

This file proves the three facts the interpreter rests on:
  * `preserves_id` — the interpreter sends `id` to the target identity (`pure`, in Kleisli).
  * `preserves_compose` — it sends `compose f g` to the target composition (`bind`, in Kleisli).
  * `naturality` — the `Option ⇒ List` component (`OptionToVec` in Rust) commutes with `map`.

The interpreter is modelled as the unique fold into an **arrow algebra** `ArrowAlg` — a target
carrier equipped with the seven strong-category operations. `preserves_id`/`preserves_compose` are
the two functor equations, definitional for the fold; specialising the carrier to the Kleisli arrows
of `M` (`aid := pure`, `acompose := bind`) is exactly the Rust `interpret_kleisli`.

DEVIATION NOTES.
  1. The Kleisli target is abstracted to a generic `ArrowAlg` carrier rather than a concrete monad,
     so `preserves_id`/`preserves_compose` hold for *every* target (the free arrow's universal
     property) and specialise to Kleisli by choosing `aid = pure`, `acompose = bind` — the choice
     the Rust `interpret_kleisli` makes. The Kleisli category laws themselves are in `Haft/Kleisli.lean`.
  2. Naturality is proved for the concrete component `optionToList` (`OptionToVec`), over Lean's
     `Option.map` / `List.map`, mirroring the Rust `fmap` witnesses; it is the naturality square a
     monad morphism relating two Kleisli interpreters must satisfy.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/interpreter_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Interpreter

/-- The erased free-arrow core (re-declared self-contained; mirrors `Haft/ArrowTerm.lean`) —
    including the choice generators `left`/`right`/`choice`/`fanin` of the `⊕` fragment
    (Stage 2b). -/
inductive ArrowCore (G : Type) : Type where
  | id
  | gen (g : G)
  | compose (f h : ArrowCore G)
  | first (f : ArrowCore G)
  | second (f : ArrowCore G)
  | split (f h : ArrowCore G)
  | fanout (f h : ArrowCore G)
  | left (f : ArrowCore G)
  | right (f : ArrowCore G)
  | choice (f h : ArrowCore G)
  | fanin (f h : ArrowCore G)

/-- A target **arrow algebra**: a carrier `T` with the strong-category operations plus the four
    choice operations of the `⊕` fragment. The Kleisli category of a monad is one such algebra
    (`aid = pure`, `acompose = bind`; the choice operations route the effect to the taken
    branch — the Rust `interpret_kleisli` arms). -/
structure ArrowAlg (G T : Type) where
  gen : G → T
  aid : T
  acompose : T → T → T
  afirst : T → T
  asecond : T → T
  asplit : T → T → T
  afanout : T → T → T
  aleft : T → T
  aright : T → T
  achoice : T → T → T
  afanin : T → T → T

/-- The interpreter: the unique arrow-homomorphism (fold) from the free arrow into a target algebra.
    Mirrors `ArrowCore::interpret_kleisli` with the target specialised to Kleisli arrows. -/
def interp {G T : Type} (alg : ArrowAlg G T) : ArrowCore G → T
  | .id          => alg.aid
  | .gen g       => alg.gen g
  | .compose f h => alg.acompose (interp alg f) (interp alg h)
  | .first f     => alg.afirst (interp alg f)
  | .second f    => alg.asecond (interp alg f)
  | .split f h   => alg.asplit (interp alg f) (interp alg h)
  | .fanout f h  => alg.afanout (interp alg f) (interp alg h)
  | .left f      => alg.aleft (interp alg f)
  | .right f     => alg.aright (interp alg f)
  | .choice f h  => alg.achoice (interp alg f) (interp alg h)
  | .fanin f h   => alg.afanin (interp alg f) (interp alg h)

variable {G T : Type}

/-- Functoriality (identity): the interpreter sends `id` to the target identity — `pure`, in the
    Kleisli target.

    THEOREM_MAP: `haft.interpreter.preserves_id` -/
theorem preserves_id (alg : ArrowAlg G T) : interp alg .id = alg.aid := rfl

/-- Functoriality (composition): the interpreter sends `compose f g` to the target composition —
    `bind`, in the Kleisli target.

    THEOREM_MAP: `haft.interpreter.preserves_compose` -/
theorem preserves_compose (alg : ArrowAlg G T) (f h : ArrowCore G) :
    interp alg (.compose f h) = alg.acompose (interp alg f) (interp alg h) := rfl

/-- Functoriality extends to the choice generators: the interpreter sends `left`/`right`/
    `choice`/`fanin` to the target algebra's choice operations — in the Kleisli target, the arms
    that route the effect to the taken branch (`ArrowCore::interpret_kleisli`). Each equation is
    definitional for the fold, extending `preserves_id`/`preserves_compose` to the `⊕`-enlarged
    generator set.

    THEOREM_MAP: `haft.interpreter.choice_preserved` -/
theorem choice_preserved (alg : ArrowAlg G T) (f h : ArrowCore G) :
    interp alg (.left f) = alg.aleft (interp alg f)
    ∧ interp alg (.right f) = alg.aright (interp alg f)
    ∧ interp alg (.choice f h) = alg.achoice (interp alg f) (interp alg h)
    ∧ interp alg (.fanin f h) = alg.afanin (interp alg f) (interp alg h) :=
  ⟨rfl, rfl, rfl, rfl⟩

/-- The `Option ⇒ List` component of the `OptionToVec` natural transformation. -/
def optionToList {α : Type} : Option α → List α
  | none => []
  | some a => [a]

/-- Naturality of `optionToList`: it commutes with `map` (`transform ∘ map f = map f ∘ transform`).
    Proved by cases on the option.

    THEOREM_MAP: `haft.interpreter.naturality` -/
theorem naturality {α β : Type} (f : α → β) (o : Option α) :
    optionToList (o.map f) = (optionToList o).map f := by
  cases o with
  | none => rfl
  | some a => rfl

end DeepCausalityFormal.Haft.Interpreter
