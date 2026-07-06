<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# THEOREM_MAP — Lean ↔ Rust traceability

This is **the bridge**. There is no tool that converts a Lean proof into a Rust test
(`openspec/notes/causal-algebra/Formalization.md` §3). Instead, each **property statement** is
transcribed once per layer and linked here:

- **Lean** *proves* the statement (deductive, unbounded, higher-order).
- **Rust witness** *checks* the same statement independently:
  - `num` / `haft`: a law-test and/or the trait contract itself (the house style).
  - `core`: a **Kani** harness (bounded model checking — first-order, fixed continuations).
  - `core` (deferred): **Aeneas** extraction — "the code IS the model".

The `THEOREM_MAP:` tag in each Lean file and the matching comment in each Rust witness carry the
same **id**. CI (`.github/workflows/formalization.yml`) fails if an id lacks either side.

## Legend

- **Lean**: `proved` = closed, no `sorry`; `sorry` = stated but unproved; `—` = not yet stated.
- **Kani** / **Test** / **Aeneas**: `✓` present & passing · `partial` · `—` not started · `n/a`.

## Map

| id | statement | Lean | Lean location | Rust witness | Test | Kani | Aeneas |
|---|---|---|---|---|---|---|---|
| `num.add_monoid.assoc` | `(a+b)+c = a+(b+c)` for `AddMonoid` | proved | `DeepCausalityFormal/Num/Monoid.lean :: add_monoid_assoc` | `deep_causality_num/tests/algebra/monoid_tests.rs :: test_add_monoid_associativity` | ✓ | n/a | — |
| `num.add_monoid.identity` | `a+0 = a ∧ 0+a = a` for `AddMonoid` | proved | `DeepCausalityFormal/Num/Monoid.lean :: add_monoid_identity` | `deep_causality_num/tests/algebra/monoid_tests.rs :: test_add_monoid_identity` | ✓ | n/a | — |
| `core.causal_monad.left_id` | `pure a >>= f = f a` | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_left_id` | `deep_causality_core/tests/kani_proofs.rs :: causal_monad_left_identity` | n/a | ✓ | — |
| `core.causal_monad.right_id` | `m >>= pure = m` (unconditional — holds on errored carriers) | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_right_id` | `deep_causality_core/tests/types/causal_monad/causal_monad_tests.rs :: test_right_identity_unconditional` | ✓ | ✓ | — |
| `core.causal_monad.assoc` | `(m >>= f) >>= g = m >>= (λx. f x >>= g)` | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_assoc` | `deep_causality_core/tests/types/causal_monad/causal_monad_tests.rs :: test_associativity_across_erroring_continuation` | ✓ | ✓ | — |
| `core.causal_monad.left_zero` | `raise e >>= f = raise e` (error short-circuit) | proved | `DeepCausalityFormal/Core/CausalMonad.lean :: bind_raise_left_zero` | `deep_causality_core/tests/kani_proofs.rs :: causal_monad_short_circuit` | n/a | ✓ | — |

### Haft layer (`deep_causality_haft`)

All Lean files under `DeepCausalityFormal/Haft/`; the Rust witnesses live in
`deep_causality_haft/tests/formalization_lean/`, which mirrors the Lean tree one-to-one
(`Haft/Functor.lean` ↔ `functor_tests.rs`, `Haft/EffectSystem.lean` ↔
`effect_system_tests.rs`, …; `Haft/Hkt.lean` is a definitional bridge with no theorems and
hence no test file). One `#[test]` per id, name pattern `test_<id>`. Citations per file;
deviations recorded in `../openspec/notes/causal-algebra/haft-formalization-deviations.md`.

| id | statement | Lean | Lean location | Test | Kani | Aeneas |
|---|---|---|---|---|---|---|
| `haft.functor.laws` | `fmap id = id`; `fmap (g∘f) = fmap g ∘ fmap f` | proved | `Haft/Functor.lean` | ✓ | n/a | — |
| `haft.pure.naturality` | `fmap f ∘ pure = pure ∘ f` | proved | `Haft/Pure.lean` | ✓ | n/a | — |
| `haft.applicative.laws` | McBride–Paterson identity, homomorphism, interchange, **composition** | proved | `Haft/Applicative.lean` | ✓ | n/a | — |
| `haft.applicative.functor_compat` | `fmap f x = pure f <*> x` | proved | `Haft/Applicative.lean` | ✓ | n/a | — |
| `haft.monad.laws` | left/right identity, associativity | proved | `Haft/Monad.lean` | ✓ | n/a | — |
| `haft.monad.applicative_coherence` | `apply = bind (fmap ·)` | proved | `Haft/Monad.lean` | ✓ | n/a | — |
| `haft.comonad.laws` | Uustalu–Vene coKleisli laws (Env carrier) | proved | `Haft/Comonad.lean` | ✓ | n/a | — |
| `haft.bifunctor.laws` | `bimap id id = id`; composition; first/second decomposition | proved | `Haft/Bifunctor.lean` | ✓ | n/a | — |
| `haft.profunctor.laws` | `dimap id id = id`; contravariant-twist composition | proved | `Haft/Profunctor.lean` | ✓ | n/a | — |
| `haft.parametric_monad.laws` | Atkey indexed monad laws (IxState carrier) | proved | `Haft/ParametricMonad.lean` | ✓ | n/a | — |
| `haft.monoidal_merge.merge_naturality` | `merge` is binatural (lax-monoidal structure map; trait renamed from `Promonad`, D3/P-1) | proved | `Haft/MonoidalMerge.lean` | ✓ | n/a | — |
| `haft.arrow.category_laws` | `id>>>f = f`; `f>>>id = f`; `>>>` associative | proved | `Haft/Arrow.lean` | ✓ | n/a | — |
| `haft.arrow.arr_functor` | `arr id = id`; `arr (g∘f) = arr f >>> arr g` | proved | `Haft/Arrow.lean` | ✓ | n/a | — |
| `haft.arrow.strength_laws` | Hughes' five `first` laws | proved | `Haft/Arrow.lean` | ✓ | n/a | — |
| `haft.arrow.derived_combinators` | `second`/`***`/`&&&` from `first` + `arr` | proved | `Haft/Arrow.lean` | ✓ | n/a | — |
| `haft.endo.monoid` | `End(T)` monoid (unit + associativity) | proved | `Haft/Endomorphism.lean` | ✓ | n/a | — |
| `haft.endo.iterate_add` | `f^(m+n) = f^n ∘ f^m` | proved | `Haft/Endomorphism.lean` | ✓ | n/a | — |
| `haft.morphism.identity` | `apply identity a = a` | proved | `Haft/Morphism.lean` | ✓ | n/a | — |
| `haft.adjunction.triangles` | triangle identities (currying adjunction) | proved | `Haft/Adjunction.lean` | ✓ | n/a | — |
| `haft.adjunction.adjunct_inverse` | adjuncts are the Hom-bijection | proved | `Haft/Adjunction.lean` | ✓ | n/a | — |
| `haft.foldable.pure_compat` | `fold (pure x) init f = f init x` | proved | `Haft/Foldable.lean` | ✓ | n/a | — |
| `haft.traversable.identity` | `sequence` at Identity applicative = id | proved | `Haft/Traversable.lean` | ✓ | n/a | — |
| `haft.traversable.naturality` | applicative morphisms commute with `sequence` | proved | `Haft/Traversable.lean` | ✓ | n/a | — |
| `haft.natural_iso.laws` | round-trip + naturality (`Option ≅ Unit ⊕ ·`) | proved | `Haft/NaturalIso.lean` | ✓ | n/a | — |
| `haft.either.coproduct_universal` | `[f,g]` exists and is unique | proved | `Haft/Either.lean` | ✓ | n/a | — |
| `haft.effect3.monad_laws` | monad laws + raise-left-zero (sum carrier) | proved | `Haft/EffectSystem.lean` | ✓ | n/a | — |
| `haft.io.monad_laws` | monad laws on the `run` denotation | proved | `Haft/Io.lean` | ✓ | n/a | — |
| `haft.cybernetic.kleisli_factorization` | `control_step` = Kleisli composite | proved | `Haft/Signatures.lean` | ✓ | n/a | — |

### Topology layer (`deep_causality_topology`)

Opened by proposal P-3 of the haft deviations note: the `RiemannMap` trait is a bare
signature; the curvature laws live at the concrete `CurvatureTensor`
(`deep_causality_topology/src/types/curvature_tensor/`). Lean files under
`DeepCausalityFormal/Topology/`; Rust witnesses in
`deep_causality_topology/tests/types/curvature_tensor/curvature_tensor_law_tests.rs`.
Reference: do Carmo, *Riemannian Geometry*, Ch. 4.

| id | statement | Lean | Lean location | Test | Kani | Aeneas |
|---|---|---|---|---|---|---|
| `topology.curvature.antisymmetry` | `R(u,v)w = −R(v,u)w` | proved | `Topology/RiemannCurvature.lean` | ✓ | n/a | — |
| `topology.curvature.bianchi_first` | `R(u,v)w + R(v,w)u + R(w,u)v = 0` (needs `g` symmetric) | proved | `Topology/RiemannCurvature.lean` | ✓ | n/a | — |
| `topology.curvature.linearity` | additivity + homogeneity in the transported slot | proved | `Topology/RiemannCurvature.lean` | ✓ | n/a | — |

## Not yet on the map (blocked / scaling — see Formalization.md work plan)

| id (planned) | statement | blocked on |
|---|---|---|
| `core.causal_monad.lawful` | `LawfulMonad` instance | P1 (remove `RelayTo`/`Map`) — P2 (W-invariant) landed via `enforce-w-invariant`; `right_id`/`assoc`/`left_zero` are now proved above |
| `haft.traversable.composition` | `sequence` at a composite applicative `M ∘ N` | needs lawful-applicative hypotheses for `M`, `N` (scaling) |
| `haft.effect_unbound.laws` | indexed-monad laws for `MonadEffect3/4/5Unbound` | same shape as `haft.parametric_monad.laws`; a dedicated carrier model is scaling work |
