/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Functor laws.

Rust source: `deep_causality_unified_math/deep_causality_haft/src/functor/functor_base.rs` (trait `Functor<F: HKT>`, operation
`fmap`). Canonical carrier: `Option<T>` via `OptionWitness`
(`deep_causality_unified_math/deep_causality_haft/src/extensions/hkt_option_ext.rs`), whose `fmap` is `m_a.map(f)` — modelled
here by `optFmap`, transcribing the same match.

Accepted theory: a functor `F : C → C` preserves identities and composition
(S. Mac Lane, *Categories for the Working Mathematician*, 2nd ed., §I.3). The two laws stated in
the Rust docstring are exactly these; both are proved below for the Option carrier.

Purity caveat: the Rust signature admits `FnMut` (stateful) closures; the laws are stated and
provable only for pure functions. The Lean model is pure by construction.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_unified_math/deep_causality_haft/tests/formalization_lean/functor_tests.rs`.

Imports: keep to the exact minimum. Every Mathlib import pulls its whole transitive closure into
the build, so import the narrowest module that still type-checks -- `Mathlib.Analysis.Quaternion`
once reached 8,639 of Mathlib's 9,450 modules to supply four algebraic laws. Mirror any new import
into `cache_roots` in `//MODULE.bazel`: that list tree-shakes the Mathlib olean download to those
roots plus their closure, so a module absent from it is never fetched and the build fails on it.
-/

namespace DeepCausalityFormal.Haft.Functor

variable {A B C : Type}

/-- `OptionWitness::fmap`: apply `f` under `Some`, preserve `None`. -/
def optFmap (f : A → B) : Option A → Option B
  | some a => some (f a)
  | none => none

/-- Functor identity law: `fmap id = id` (Mac Lane §I.3).

    THEOREM_MAP: `haft.functor.laws` -/
theorem opt_fmap_id (x : Option A) : optFmap (fun a => a) x = x := by
  cases x <;> rfl

/-- Functor composition law: `fmap (g ∘ f) = fmap g ∘ fmap f` (Mac Lane §I.3).

    THEOREM_MAP: `haft.functor.laws` -/
theorem opt_fmap_comp (f : A → B) (g : B → C) (x : Option A) :
    optFmap (fun a => g (f a)) x = optFmap g (optFmap f x) := by
  cases x <;> rfl

end DeepCausalityFormal.Haft.Functor
