/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — "Promonad" (monoidal merge).

Rust source: `deep_causality_haft/src/monad/promonad.rs` (trait `Promonad<P: HKT3Unbound>`,
operations `merge`, `fuse`).

DEVIATION NOTE (naming): in the accepted literature a **promonad** is a monad in the bicategory
`Prof` of profunctors — equivalently an identity-on-objects functor out of the base category
(F. Loregian, *(Co)end Calculus*, CUP 2021, §5.2; B. Jacobs et al. on arrows-as-promonads,
*Categorical semantics for arrows*, JFP 19(3–4), 2009). The Rust trait is **not** that object.
Its `merge : P⟨A,A,A⟩ → P⟨B,B,B⟩ → (A → B → C) → P⟨C,C,C⟩`, restricted to the diagonal
`D A := P⟨A,A,A⟩`, is the binary lifting `liftA2` of a lax monoidal functor
`D A ⊗ D B → D (A ⊗ B)` (McBride–Paterson 2008 §7 relate `liftA2` and monoidal functors; the
docstring's own reference to Day convolution points the same way). Recommendation: rename the
trait (e.g. `MonoidalMerge`) or re-document it as a lax-monoidal merge, and reconsider
`fuse : A → B → P⟨A,B,C⟩`, whose free `C` is structurally undetermined (an implementor must
manufacture a `P⟨A,B,C⟩` for *every* `C` from an `A` and a `B` alone — only phantom-like
carriers can do this).

What IS lawful about `merge` is proved here on the diagonal Option carrier: `merge` is a
*binatural* transformation — it commutes with `fmap` in both arguments.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/algebra/formalization_law_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Promonad

variable {A B C A' B' : Type}

/-- `OptionWitness::fmap` (see `Functor.lean`). -/
def optFmap (f : A → B) : Option A → Option B
  | some a => some (f a)
  | none => none

/-- `merge` on the diagonal Option carrier: combine two contexts with `f` (= `liftA2 f`). -/
def merge (pa : Option A) (pb : Option B) (f : A → B → C) : Option C :=
  match pa, pb with
  | some a, some b => some (f a b)
  | _, _ => none

/-- Binaturality of `merge`: mapping the inputs first equals merging with the composed
    combiner — `merge (fmap p a) (fmap q b) h = merge a b (fun x y => h (p x) (q y))`.
    This is the naturality square of the lax-monoidal structure map
    (McBride–Paterson 2008 §7).

    THEOREM_MAP: `haft.promonad.merge_naturality` -/
theorem merge_naturality (p : A → A') (q : B → B') (h : A' → B' → C)
    (pa : Option A) (pb : Option B) :
    merge (optFmap p pa) (optFmap q pb) h = merge pa pb (fun x y => h (p x) (q y)) := by
  cases pa <;> cases pb <;> rfl

end DeepCausalityFormal.Haft.Promonad
