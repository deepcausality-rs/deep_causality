/-
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.

Haft — signature-only interfaces: RiemannMap and CyberneticLoop.

Rust sources: `deep_causality_haft/src/riemann_map/mod.rs` (trait `RiemannMap<P: HKT4Unbound>`,
operations `curvature`, `scatter`) and `deep_causality_haft/src/cybernetic_loop/mod.rs`
(trait `CyberneticLoop<P: HKT5Unbound>`, operation `control_step`).

HONESTY NOTE (deviation): these two traits carry **no equational theory** — they are typed
signatures, not algebraic structures. `RiemannMap`'s docstring invokes "a Multilinear Map in a
Tensor Category", but multilinearity is an equation system (additivity and homogeneity in each
argument, e.g. `R(u+u',v)w = R(u,v)w + R(u',v)w`) over types that this trait does not require
to carry any algebra; the actual curvature symmetries (antisymmetry `R(u,v)w = -R(v,u)w`, first
Bianchi identity — e.g. do Carmo, *Riemannian Geometry*, Ch. 4) are likewise unstated and
unstatable at this signature. Recommendation: either state those laws where the concrete
implementations live (`deep_causality_topology` / `deep_causality_physics`, where the types do
carry algebra) or soften the docstring to "a typed interface for rank-4 interactions".

What IS provable at this level is recorded below: the two shapes are definable (they are just
function types — nothing to verify), and `control_step`'s canonical semantics factors as a
Kleisli composite over `Except` — observe, then decide, in the error monad — which is the
"loop = composition of Sense ∘ Decide" claim made by the docstring, made precise.

This file is self-contained (no imports) so it typechecks standalone with bare `lean`.

Rust witness: `deep_causality_haft/tests/algebra/formalization_law_tests.rs`.
-/

namespace DeepCausalityFormal.Haft.Signatures

variable {S B C A E X Y Z : Type}

/-- `RiemannMap::curvature` — shape only: `R(u, v)w → D`. A signature, not a structure. -/
def CurvatureSig (A B C D : Type) : Type := A → B → C → D

/-- `RiemannMap::scatter` — shape only: `(A, B) → (C, D)`. -/
def ScatterSig (A B C D : Type) : Type := A × B → C × D

/-- Kleisli composition in the `Except` monad. -/
def kleisli (f : X → Except E Y) (g : Y → Except E Z) : X → Except E Z :=
  fun x =>
    match f x with
    | .error e => .error e
    | .ok y => g y

/-- `CyberneticLoop::control_step`, canonical pure semantics: observe the sensor input under
    the fixed context, decide an action from the belief under the same context. -/
def controlStep (ctx : C) (observe : S → C → B) (decide : B → C → A) (s : S) : Except E A :=
  .ok (decide (observe s ctx) ctx)

/-- The control step IS Kleisli composition: `control_step = (pure ∘ observe) >=> (pure ∘
    decide)` in the `Except` error monad. The OODA loop's Observe→Decide chaining is monadic
    sequencing, not a new primitive.

    THEOREM_MAP: `haft.cybernetic.kleisli_factorization` -/
theorem control_step_kleisli (ctx : C) (observe : S → C → B) (decide : B → C → A) (s : S) :
    controlStep (E := E) ctx observe decide s
      = kleisli (fun s => .ok (observe s ctx)) (fun b => .ok (decide b ctx)) s := rfl

end DeepCausalityFormal.Haft.Signatures
