/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Arrow laws (Hughes' strong category).

Rust source: `deep_causality_haft/src/arrow/` — trait `Arrow` with combinators
`Id` (`id.rs`), `Lift` (`lift.rs`, = Hughes' `arr`), `Compose` (`compose.rs`, `>>>`),
`First`/`Second` (`first.rs`/`second.rs`), `Split` (`split.rs`, `***`),
`Fanout` (`fanout.rs`, `&&&`).

Accepted theory: J. Hughes, *Generalising Monads to Arrows*, Sci. Comput. Program. 37(1–3),
2000 (laws as tabulated in R. Paterson, *A New Notation for Arrows*, ICFP 2001, Fig. 1):
category laws, `arr` functoriality, and the five `first`(strength) laws. The categorical home
is a premonoidal/Freyd category (Power–Robinson, MSCS 7(5), 1997).

Denotational model — and why it is faithful: every Rust combinator is a concrete struct whose
`run` is a *pure function*, fixed by its `impl` (e.g. `Compose::run = g.run(f.run(input))`,
`First::run = (f.run(a), c)`). The denotation `⟦arrow⟧ = run` therefore maps the combinator
algebra into the category of Lean functions, where each combinator below is the literal
transcription of the corresponding `run` body. Arrow equality is extensional equality of `run`
(the crate's arrows carry no other observable state). Under this denotation ALL of Hughes'
laws hold — proved below — so the Rust `Arrow` is a strong category as claimed by its docs.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/arrow_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Arrow

variable {A B C D E X Y : Type}

/-- `Id::run` — the identity arrow. -/
def aId : A → A := fun a => a

/-- `Lift::run` — Hughes' `arr`: a pure function as an arrow. -/
def arr (f : A → B) : A → B := f

/-- `Compose::run` — `f >>> g`. -/
def comp (f : A → B) (g : B → C) : A → C := fun a => g (f a)

/-- `First::run` — strength on the first component. -/
def first (f : A → B) : A × C → B × C := fun p => (f p.1, p.2)

/-- `Second::run` — strength on the second component. -/
def second (f : A → B) : C × A → C × B := fun p => (p.1, f p.2)

/-- `Split::run` — the monoidal product `***`. -/
def split (f : A → B) (g : C → D) : A × C → B × D := fun p => (f p.1, g p.2)

/-- `Fanout::run` — `&&&` (input duplicated to both arrows).

    Modelling note: the Rust `Fanout::run` feeds the *cloned* input to `f` and the *original* to
    `g` (`(self.0.run(input.clone()), self.1.run(input))`, requiring `F::In: Clone`). This proof
    duplicates the same Lean value `a` to both arrows, so the `&&&` laws below hold under the
    standard **lawful-`Clone`** assumption `input.clone() = input` — i.e. `Clone` is a pure copy
    with no observable effect. For a type whose `Clone` is non-lawful, the Rust `&&&` may diverge
    from what is proved here; such types are outside the modelled fragment. -/
def fanout (f : A → B) (g : A → C) : A → B × C := fun a => (f a, g a)

-- ------------------------------------------------------------------
-- Category laws (Hughes 2000; Mac Lane §I.1)
-- ------------------------------------------------------------------

/-- Left identity: `id >>> f = f`.

    THEOREM_MAP: `haft.arrow.category_laws` -/
theorem comp_id_left (f : A → B) : comp aId f = f := rfl

/-- Right identity: `f >>> id = f`.

    THEOREM_MAP: `haft.arrow.category_laws` -/
theorem comp_id_right (f : A → B) : comp f aId = f := rfl

/-- Associativity: `(f >>> g) >>> h = f >>> (g >>> h)`.

    THEOREM_MAP: `haft.arrow.category_laws` -/
theorem comp_assoc (f : A → B) (g : B → C) (h : C → D) :
    comp (comp f g) h = comp f (comp g h) := rfl

-- ------------------------------------------------------------------
-- arr functoriality (Hughes 2000)
-- ------------------------------------------------------------------

/-- `arr id = id`.

    THEOREM_MAP: `haft.arrow.arr_functor` -/
theorem arr_id : arr (fun a : A => a) = aId := rfl

/-- `arr (g ∘ f) = arr f >>> arr g`.

    THEOREM_MAP: `haft.arrow.arr_functor` -/
theorem arr_comp (f : A → B) (g : B → C) :
    arr (fun a => g (f a)) = comp (arr f) (arr g) := rfl

-- ------------------------------------------------------------------
-- Strength laws (Hughes 2000, the five `first` laws)
-- ------------------------------------------------------------------

/-- `first (arr f) = arr (f × id)`.

    THEOREM_MAP: `haft.arrow.strength_laws` -/
theorem first_arr (f : A → B) :
    first (C := C) (arr f) = arr (fun p : A × C => (f p.1, p.2)) := rfl

/-- `first (f >>> g) = first f >>> first g` — strength is functorial.

    THEOREM_MAP: `haft.arrow.strength_laws` -/
theorem first_comp (f : A → B) (g : B → C) :
    first (C := D) (comp f g) = comp (first f) (first g) := rfl

/-- Exchange: `first f >>> arr (id × g) = arr (id × g) >>> first f` — the strength commutes
    with pure post-processing of the passthrough component.

    THEOREM_MAP: `haft.arrow.strength_laws` -/
theorem first_exchange (f : A → B) (g : C → D) :
    comp (first f) (arr (fun p : B × C => (p.1, g p.2)))
      = comp (arr (fun p : A × C => (p.1, g p.2))) (first f) := rfl

/-- Unit: `first f >>> arr fst = arr fst >>> f` — projecting after equals projecting before.

    THEOREM_MAP: `haft.arrow.strength_laws` -/
theorem first_unit (f : A → B) :
    comp (first (C := C) f) (arr Prod.fst) = comp (arr Prod.fst) f := rfl

/-- Association: `first (first f) >>> arr assoc = arr assoc >>> first f` — strength is
    coherent with the reassociation `((A × C) × D) ≅ (A × (C × D))`.

    THEOREM_MAP: `haft.arrow.strength_laws` -/
theorem first_assoc (f : A → B) :
    comp (first (C := D) (first (C := C) f))
        (arr (fun p : (B × C) × D => (p.1.1, (p.1.2, p.2))))
      = comp (arr (fun p : (A × C) × D => (p.1.1, (p.1.2, p.2))))
          (first f) := rfl

-- ------------------------------------------------------------------
-- Derived combinators (Hughes 2000 §4: second/***/&&& are definable from first)
-- ------------------------------------------------------------------

/-- `second f = arr swap >>> first f >>> arr swap`.

    THEOREM_MAP: `haft.arrow.derived_combinators` -/
theorem second_derived (f : A → B) :
    second (C := C) f
      = comp (arr (fun p : C × A => (p.2, p.1)))
          (comp (first f) (arr (fun p : B × C => (p.2, p.1)))) := rfl

/-- `f *** g = first f >>> second g`.

    THEOREM_MAP: `haft.arrow.derived_combinators` -/
theorem split_derived (f : A → B) (g : C → D) :
    split f g = comp (first f) (second g) := rfl

/-- `f &&& g = arr dup >>> (f *** g)`.

    THEOREM_MAP: `haft.arrow.derived_combinators` -/
theorem fanout_derived (f : A → B) (g : A → C) :
    fanout f g = comp (arr (fun a => (a, a))) (split f g) := rfl

end DeepCausalityFormal.Haft.Arrow
