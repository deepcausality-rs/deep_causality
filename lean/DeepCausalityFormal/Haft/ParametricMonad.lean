/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Parametric (indexed) monad laws.

Rust source: `deep_causality_haft/src/monad/parametric_monad.rs` (trait
`ParametricMonad<M: HKT3Unbound>`, operations `pure : A → M S S A` and
`ibind : M S1 S2 A → (A → M S2 S3 B) → M S1 S3 B`). The same shape underlies
`MonadEffect3Unbound/4/5` (`src/effect_system/monad_effect_unbound.rs`), whose `ibind` threads a
state-type transition `S1 → S2 → S3` around fixed effect channels.

Accepted theory: R. Atkey, *Parameterised notions of computation*, JFP 19(3–4), 2009 — the
parameterised monad. The indexed laws mirror the ordinary Kleisli-triple laws with composable
index transitions `(S1→S2) ∘ (S2→S3) = (S1→S3)`, exactly as the Rust docstring's
"Mathematical Definition" section describes — **correct as documented**.

Canonical model: the indexed state monad `IxState S1 S2 A = S1 → A × S2` (Atkey §2's motivating
example). The crate's `parametric_monad_tests.rs` uses a phantom-indexed value carrier — a
degenerate special case; the model here keeps real state so the index discipline does work.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/algebra/formalization_law_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.ParametricMonad

variable {S1 S2 S3 S4 S : Type} {A B C : Type}

/-- Indexed state: consume an `S1`-typed state, produce a value and an `S2`-typed state. -/
def IxState (S1 S2 A : Type) : Type := S1 → A × S2

/-- `pure`: value in, state type unchanged (`S → S`), mirroring the Rust signature. -/
def ipure (a : A) : IxState S S A := fun s => (a, s)

/-- `ibind`: run the `S1 → S2` step, feed its value to `f`, run the `S2 → S3` step. -/
def ibind (m : IxState S1 S2 A) (f : A → IxState S2 S3 B) : IxState S1 S3 B :=
  fun s1 => f (m s1).1 (m s1).2

/-- Indexed left identity: `ibind (ipure a) f = f a` (Atkey 2009, eq. for `η`).

    THEOREM_MAP: `haft.parametric_monad.laws` -/
theorem ibind_left_id (a : A) (f : A → IxState S1 S2 B) :
    ibind (ipure a) f = f a := rfl

/-- Indexed right identity: `ibind m ipure = m` (Atkey 2009). Proof is `rfl` via product eta.

    THEOREM_MAP: `haft.parametric_monad.laws` -/
theorem ibind_right_id (m : IxState S1 S2 A) :
    ibind m ipure = m := rfl

/-- Indexed associativity across the transition chain `S1 → S2 → S3 → S4` (Atkey 2009).

    THEOREM_MAP: `haft.parametric_monad.laws` -/
theorem ibind_assoc (m : IxState S1 S2 A) (f : A → IxState S2 S3 B) (g : B → IxState S3 S4 C) :
    ibind (ibind m f) g = ibind m (fun a => ibind (f a) g) := rfl

end DeepCausalityFormal.Haft.ParametricMonad
