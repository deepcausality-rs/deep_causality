/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — Morphism (witness-level identity + application).

Rust source: `deep_causality_haft/src/morphism/morphism_base.rs` (trait `Morphism<P>`,
operations `identity`, `apply`; canonical carrier `FnMorphism` with `Type<A, B> = fn(A) -> B`).

The trait deliberately has NO composition: under the repo's `unsafe_code = "forbid"` / no-`dyn`
policy, composing two capturing closures has no nameable carrier type, and `Box<dyn Fn>` is
forbidden (documented in the Rust source — an honest, Rust-necessitated restriction, not a
mathematical claim). Total composition is realized at the value level by the `Arrow` algebra
(`Arrow.lean`). What `Morphism` owes mathematically is only that `identity` is the identity
under `apply` — the unit law of the (external) category whose hom-carrier it names.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/algebra/formalization_law_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Morphism

variable {A : Type}

/-- `FnMorphism::identity` — the identity arrow `A → A`. -/
def identity : A → A := fun a => a

/-- `FnMorphism::apply` — arrow application. -/
def apply (arrow : A → A) (input : A) : A := arrow input

/-- Identity law: `apply identity a = a`.

    THEOREM_MAP: `haft.morphism.identity` -/
theorem apply_identity (a : A) : apply identity a = a := rfl

end DeepCausalityFormal.Haft.Morphism
