/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — lax monoidal structure on an endofunctor (φ, η).

Rust source: `deep_causality_unified_math/deep_causality_haft/src/lax_monoidal/mod.rs` (traits
`Semigroupal<F: HKT>` carrying `zip_with`/`zip`, and `LaxMonoidal<F: HKT>: Semigroupal<F>` adding
`unit`). Canonical carrier: `Option<T>` via `OptionWitness`, matching the carrier used by
`Functor.lean`, `Applicative.lean` and `Monad.lean`.

Accepted theory: a lax monoidal functor is a triple (F, φ, η) with φ : F A ⊗ F B → F (A ⊗ B) and
η : I → F I, subject to associativity and the two unit coherence conditions (S. Mac Lane,
*Categories for the Working Mathematician*, 2nd ed., §XI.2). An applicative functor is exactly a
lax monoidal functor for the cartesian product, i.e. a monoid object under Day convolution
(C. McBride & R. Paterson, *Applicative programming with effects*, JFP 18(1), 2008, §7).

SPLIT: the Rust side separates φ from η across two traits, because every context-carrying witness
in the workspace has a lawful φ and no lawful η — `unit()` takes no argument and would have to
invent a complex, a grade or a lattice. The law ids mirror that split: `naturality` and `assoc` are
stated against `Semigroupal` and a φ-only witness owes exactly those two; `unit_laws` is stated
against `LaxMonoidal` and is owed only by witnesses that have η.

NAMING: the `haft.monoidal.*` prefix belongs to `SymmetricMonoidal.lean`, which formalises the
*cartesian* PROP at the level of values (copy Δ, discard ε, merge ∇, swap σ). This file is one
level up, on endofunctors, and deliberately non-cartesian: it exists to avoid requiring Δ, which
in Rust is `Clone`. Reusing that prefix would also break the `theorem-map` CI job, whose
`grep -Fl "$id" lean/THEOREM_MAP.md` would match the wrong row and hide a missing entry.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_unified_math/deep_causality_haft/tests/formalization_lean/lax_monoidal_tests.rs`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

namespace DeepCausalityFormal.Haft.LaxMonoidal

variable {A B C A' B' : Type}

/-- `OptionWitness::fmap` (see `Functor.lean`). -/
def optFmap (f : A → B) : Option A → Option B
  | some a => some (f a)
  | none => none

/-- `Semigroupal::zip_with` for `OptionWitness`: the primitive. Both present, or nothing. -/
def optZipWith (f : A → B → C) : Option A → Option B → Option C
  | some a, some b => some (f a b)
  | _, _ => none

/-- `Semigroupal::zip`, the provided method: `zip_with(fa, fb, |a, b| (a, b))`. -/
def optZip (fa : Option A) (fb : Option B) : Option (A × B) :=
  optZipWith (fun a b => (a, b)) fa fb

/-- `LaxMonoidal::unit`: η : I → F I. -/
def optUnit : Option Unit := some ()

/-- `MonoidalApplicative::apply`, the provided method derived from `zip_with`. -/
def optApply (ff : Option (A → B)) (fa : Option A) : Option B :=
  optZipWith (fun f a => f a) ff fa

/-- Naturality of φ: mapping each side before pairing equals pairing then mapping the pair.
    Stated first because it is what makes φ a natural transformation rather than an arbitrary
    binary function, and it is the law a shape-dependent shortcut violates first.

    Owed by every `Semigroupal` witness.

    THEOREM_MAP: `haft.lax_monoidal.naturality` -/
theorem opt_zip_naturality (f : A → A') (g : B → B') (fa : Option A) (fb : Option B) :
    optZip (optFmap f fa) (optFmap g fb)
      = optFmap (fun p : A × B => (f p.1, g p.2)) (optZip fa fb) := by
  cases fa <;> cases fb <;> rfl

/-- Associativity of φ, modulo the associator `((A, B), C) ≅ (A, (B, C))`.

    Owed by every `Semigroupal` witness; this is the promise recorded by the `Convolutional`
    marker on the Rust side.

    THEOREM_MAP: `haft.lax_monoidal.assoc` -/
theorem opt_zip_assoc (fa : Option A) (fb : Option B) (fc : Option C) :
    optFmap (fun p : (A × B) × C => (p.1.1, (p.1.2, p.2))) (optZip (optZip fa fb) fc)
      = optZip fa (optZip fb fc) := by
  cases fa <;> cases fb <;> cases fc <;> rfl

/-- Left and right unit coherence, modulo the unitors `((), A) ≅ A` and `(A, ()) ≅ A`.

    Owed only by `LaxMonoidal` witnesses. A φ-only witness cannot state these, which is the whole
    reason the Rust traits are split.

    THEOREM_MAP: `haft.lax_monoidal.unit_laws` -/
theorem opt_zip_left_unit (fa : Option A) :
    optFmap (fun p : Unit × A => p.2) (optZip optUnit fa) = fa := by
  cases fa <;> rfl

/-- Right unit; the other half of `haft.lax_monoidal.unit_laws`.

    THEOREM_MAP: `haft.lax_monoidal.unit_laws` -/
theorem opt_zip_right_unit (fa : Option A) :
    optFmap (fun p : A × Unit => p.1) (optZip fa optUnit) = fa := by
  cases fa <;> rfl

/-- `zip` is genuinely derived from `zip_with`, not an independent operation. This is what licenses
    `zip` being a provided method on the Rust trait.

    THEOREM_MAP: `haft.lax_monoidal.assoc` -/
theorem opt_zip_from_zip_with (fa : Option A) (fb : Option B) :
    optZip fa fb = optZipWith (fun a b => (a, b)) fa fb := rfl

/-- Agreement: the `apply` derived from `zip_with` equals the hand-written `Applicative::apply`
    transcribed in `Applicative.lean` as `optApply`. This is the obligation a witness incurs by
    holding both `Applicative` and `MonoidalApplicative`, and it is what licenses `apply` being a
    provided method.

    It is a statement about the *pair* of traits rather than about either one, which is why it
    carries its own id rather than joining `haft.applicative.laws`.

    THEOREM_MAP: `haft.lax_monoidal.apply_agreement` -/
theorem opt_apply_from_zip_with (fab : Option (A → B)) (fa : Option A) :
    optApply fab fa
      = (match fab with
         | some f => optFmap f fa
         | none => none) := by
  cases fab <;> cases fa <;> rfl

end DeepCausalityFormal.Haft.LaxMonoidal
