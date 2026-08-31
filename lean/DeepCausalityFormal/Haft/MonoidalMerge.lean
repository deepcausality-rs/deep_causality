/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — MonoidalMerge (semigroupal merge).

Rust source: `deep_causality_unified_math/deep_causality_haft/src/monad/monoidal_merge.rs` (trait
`MonoidalMerge<P: HKT3Unbound>`, operation `merge`).

Naming history: the trait was previously `Promonad` — a misnomer, since in the accepted
literature a promonad is a monad in the bicategory of profunctors / an identity-on-objects
functor (Loregian, *(Co)end Calculus*, CUP 2021, §5.2; Jacobs, Heunen & Hasuo, *Categorical
semantics for arrows*, JFP 19, 2009). Deviation D3 of the formalization audit; resolved by
renaming (proposal P-1). Restricted to the diagonal `D A := P⟨A,A,A⟩`, `merge` is the binary
lifting `liftA2` of the structure map `φ : D A ⊗ D B → D (A ⊗ B)` (McBride–Paterson 2008 §7;
related to Day convolution). The former `fuse : A → B → P⟨A,B,C⟩` operation — whose free `C`
was structurally undetermined (every workspace implementation either panicked or discarded
its inputs) — was removed in the same change.

SCOPE: this is a *semigroupal* structure, not a lax monoidal one. A lax monoidal functor is a
triple `(F, φ, η)`; `MonoidalMerge` carries `φ` alone, with no unit `η : I → D I` and none
derivable from `merge`. So the unitality coherence conditions are not statable against this
trait, and no theorem below asserts them; only naturality is in scope. The trait name predates
this distinction and is kept for continuity. See
`openspec/notes/hkt_gat/monoidal-applicative.md` §4.1.

What is lawful about `merge` is proved here on the diagonal Option carrier: `merge` is a
*binatural* transformation — it commutes with `fmap` in both arguments.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_unified_math/deep_causality_haft/tests/formalization_lean/monoidal_merge_tests.rs`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

namespace DeepCausalityFormal.Haft.MonoidalMerge

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

    THEOREM_MAP: `haft.monoidal_merge.merge_naturality` -/
theorem merge_naturality (p : A → A') (q : B → B') (h : A' → B' → C)
    (pa : Option A) (pb : Option B) :
    merge (optFmap p pa) (optFmap q pb) h = merge pa pb (fun x y => h (p x) (q y)) := by
  cases pa <;> cases pb <;> rfl

end DeepCausalityFormal.Haft.MonoidalMerge
