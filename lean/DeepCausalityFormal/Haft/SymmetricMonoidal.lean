/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — the symmetric-monoidal PROP: copy comonoid `Δ`/`ε`, merge monoid `∇`/`η`, symmetry `σ`.

Rust source: `deep_causality_haft/src/monoidal/mod.rs` (`SymMonoidal::{copy, discard, swap, merge, unit}`).

Textbook definition. A **symmetric monoidal category** is a monoidal category `(C, ⊗, I)` with a
natural braiding `σ_{A,B} : A ⊗ B → B ⊗ A` satisfying `σ_{B,A} ∘ σ_{A,B} = id` and the hexagon
coherences (Mac Lane, *Categories for the Working Mathematician* 2nd ed., §VII.1 and §XI.1). A
**PROP** (product-and-permutation category) is a symmetric strict monoidal category whose objects are
the natural numbers under `+` (Mac Lane, "Categorical Algebra," *Bull. AMS* 71, 1965; Lack,
"Composing PROPs," *Theory Appl. Categ.* 13, 2004). In a **cartesian** monoidal category every object
carries a *unique* cocommutative comonoid — the diagonal `Δ : A → A ⊗ A` (copy) and the terminal map
`ε : A → I` (discard) — and every monoid object `(A, ∇, η)` interacts with it as a **bimonoid /
bialgebra**: `Δ` is a monoid homomorphism (T. Fox, "Coalgebras and Cartesian Categories,"
*Comm. Algebra* 4(7), 1976; for the string-diagram / spider view, Coecke & Kissinger, *Picturing
Quantum Processes*, CUP 2017, Ch. 8).

This file proves the three law-bundles the Rust generators rest on:
  * `comonoid_laws`  — coassociativity, counit, cocommutativity of the diagonal `(Δ, ε)`.
  * `merge_monoid_laws` — associativity and (left/right) unit of the merge `(∇, η)`, which are the
    monoid laws (transcribed as a `Mon` structure).
  * `symmetry` — `σ ∘ σ = id`.

DEVIATION NOTES.
  1. The monoidal product is the concrete cartesian product `α × β` with unit `Unit`; `Δ` is the
     literal diagonal `fun a => (a, a)` (Fox), so the comonoid laws hold definitionally (`rfl`). The
     associator/unitors are the definitional tuple reassociations of a strict cartesian skeleton and
     are elided as identities — matching the Rust generators, which are plain tuple functions.
  2. The merge monoid is transcribed self-contained as a `Mon` structure carrying `unit`/`merge` and
     the monoid laws as fields (rather than importing Mathlib), mirroring the Rust `Monoid`
     (`empty`/`combine`); this matches `Haft/Kleisli.lean`'s self-contained `Monad'`.
  3. Copy–merge coherence (the bialgebra law `Δ(x ∇ y) = Δx ∇ Δy` and, over a commutative monoid,
     `∇ ∘ σ = ∇`) is exercised in the Rust witness (`test_monoidal_copy_merge_bialgebra`) and not
     re-proved here; the three named bundles above are the Lean scope.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/formalization_lean/monoidal_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.SymmetricMonoidal

variable {α β : Type}

/-- Copy `Δ`: the diagonal `A → A ⊗ A`. -/
def copy (a : α) : α × α := (a, a)

/-- Discard `ε`: the counit `A → I` (the terminal map). -/
def discard (_a : α) : Unit := ()

/-- Swap `σ`: the symmetry `A ⊗ B → B ⊗ A`. -/
def swap (p : α × β) : β × α := (p.2, p.1)

/-- The copy comonoid laws — coassociativity (both flattenings give `(a, a, a)`), counit (each
    copied half is the original), and cocommutativity (`swap ∘ copy = copy`). All definitional for
    the diagonal.

    THEOREM_MAP: `haft.monoidal.comonoid_laws` -/
theorem comonoid_laws (a : α) :
    -- coassociativity up to the associator: (Δ ⊗ id) ∘ Δ and (id ⊗ Δ) ∘ Δ, flattened
    ((copy (copy a).1).1, (copy (copy a).1).2, (copy a).2)
        = ((copy a).1, (copy (copy a).2).1, (copy (copy a).2).2)
    -- counit: both halves equal the original (discard the other ⇒ identity)
    ∧ (copy a).1 = a ∧ (copy a).2 = a
    -- cocommutativity
    ∧ swap (copy a) = copy a := by
  refine ⟨?_, ?_, ?_, ?_⟩ <;> rfl

/-- A monoid transcribed self-contained: `unit`, `merge`, and the monoid laws (mirrors Rust
    `Monoid` `empty`/`combine`). -/
structure Mon (M : Type) where
  unit : M
  merge : M → M → M
  assoc : ∀ x y z, merge (merge x y) z = merge x (merge y z)
  left_unit : ∀ x, merge unit x = x
  right_unit : ∀ x, merge x unit = x

/-- The merge monoid laws — associativity and (left/right) unit of `(∇, η)` — read straight off the
    monoid structure.

    THEOREM_MAP: `haft.monoidal.merge_monoid_laws` -/
theorem merge_monoid_laws {M : Type} (mo : Mon M) (x y z : M) :
    mo.merge (mo.merge x y) z = mo.merge x (mo.merge y z)
    ∧ mo.merge mo.unit x = x
    ∧ mo.merge x mo.unit = x :=
  ⟨mo.assoc x y z, mo.left_unit x, mo.right_unit x⟩

/-- The symmetry law — the braiding `σ` is its own inverse (`σ ∘ σ = id`).

    THEOREM_MAP: `haft.monoidal.symmetry` -/
theorem symmetry (p : α × β) : swap (swap p) = p := by
  cases p
  rfl

end DeepCausalityFormal.Haft.SymmetricMonoidal
