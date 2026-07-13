---
title: Haft
description: Higher-kinded functional laws (functor, applicative, monad, arrow, free monad, monoidal, traversable), machine-checked in Lean and bound to Rust law-tests.
sidebar:
  order: 3
---

Forty-nine laws for the Higher-Order Abstract Functional Traits: functor, applicative, monad, comonad, bifunctor, profunctor, arrow, free monad, category, Kleisli, symmetric-monoidal, foldable, traversable, adjunction, and the effect system. These are the type-level laws behind [Higher-Kinded Types](/concepts/hkt/), proved in [`lean/DeepCausalityFormal/Haft/`](https://github.com/deepcausality-rs/deep_causality/tree/main/lean/DeepCausalityFormal/Haft).

Every row is `proved` in Lean. This layer has no per-row Rust-witness column: the witness tree `deep_causality_haft/tests/formalization_lean/` mirrors the Lean tree one-to-one (`Haft/Functor.lean` maps to `functor_tests.rs`), with one law-test per id carrying the id as a `THEOREM_MAP:` annotation.

| id | statement | Lean proof | Test |
|---|---|---|---|
| `haft.functor.laws` | `fmap id = id`; `fmap (g∘f) = fmap g ∘ fmap f` | `Haft/Functor.lean` | ✓ |
| `haft.pure.naturality` | `fmap f ∘ pure = pure ∘ f` | `Haft/Pure.lean` | ✓ |
| `haft.applicative.laws` | McBride–Paterson identity, homomorphism, interchange, **composition** | `Haft/Applicative.lean` | ✓ |
| `haft.applicative.functor_compat` | `fmap f x = pure f <*> x` | `Haft/Applicative.lean` | ✓ |
| `haft.monad.laws` | left/right identity, associativity | `Haft/Monad.lean` | ✓ |
| `haft.monad.applicative_coherence` | `apply = bind (fmap ·)` | `Haft/Monad.lean` | ✓ |
| `haft.comonad.laws` | Uustalu–Vene coKleisli laws (Env carrier) | `Haft/Comonad.lean` | ✓ |
| `haft.bifunctor.laws` | `bimap id id = id`; composition; first/second decomposition | `Haft/Bifunctor.lean` | ✓ |
| `haft.profunctor.laws` | `dimap id id = id`; contravariant-twist composition | `Haft/Profunctor.lean` | ✓ |
| `haft.parametric_monad.laws` | Atkey indexed monad laws (IxState carrier) | `Haft/ParametricMonad.lean` | ✓ |
| `haft.monoidal_merge.merge_naturality` | `merge` is binatural (lax-monoidal structure map; trait renamed from `Promonad`, D3/P-1) | `Haft/MonoidalMerge.lean` | ✓ |
| `haft.free_monad.left_id` | `bind (pure a) k = k a` (free monad on a functor) | `Haft/FreeMonad.lean` | ✓ |
| `haft.free_monad.right_id` | `bind m pure = m` | `Haft/FreeMonad.lean` | ✓ |
| `haft.free_monad.assoc` | `bind (bind m f) g = bind m (λx. bind (f x) g)` | `Haft/FreeMonad.lean` | ✓ |
| `haft.free_monad.lift_bind` | `bind (lift op) k` runs `k` under the operation node | `Haft/FreeMonad.lean` | ✓ |
| `haft.free_monad.map_id` | `map id = id` (functor identity via right id) | `Haft/FreeMonad.lean` | ✓ |
| `haft.arrow.category_laws` | `id>>>f = f`; `f>>>id = f`; `>>>` associative | `Haft/Arrow.lean` | ✓ |
| `haft.category.laws` | function category `Fun`: left/right identity + associativity of `compose` | `Haft/Category.lean` | ✓ |
| `haft.kleisli.category_laws` | Kleisli category (`id=pure`, `compose=bind`): left/right identity + associativity, reducing to the monad laws | `Haft/Kleisli.lean` | ✓ |
| `haft.arrow.arr_functor` | `arr id = id`; `arr (g∘f) = arr f >>> arr g` | `Haft/Arrow.lean` | ✓ |
| `haft.arrow.strength_laws` | Hughes' five `first` laws | `Haft/Arrow.lean` | ✓ |
| `haft.arrow.derived_combinators` | `second`/`***`/`&&&` from `first` + `arr` | `Haft/Arrow.lean` | ✓ |
| `haft.arrow_term.interpret_sound` | reified free arrow: `interpret` is a homomorphism — commutes with `compose`/`first`/`second`/`split`/`fanout` (interpreting a term = composing its combinators) | `Haft/ArrowTerm.lean` | ✓ |
| `haft.arrow_term.free` | free arrow universal property: interpretation is determined by the generators (agree on generators ⇒ agree on every term) | `Haft/ArrowTerm.lean` | ✓ |
| `haft.arrow_choice.laws` | the ArrowChoice fragment `⊕` over `Either`: `left (arr f) = arr (f ⊕ id)`, functoriality/exchange/unit laws, `fanin` as the coproduct elimination (computation + uniqueness), and the used `⊗`-over-`⊕` distributivity (`distl` iso + naturality; full rig coherence deferred) | `Haft/ArrowChoice.lean` | ✓ |
| `haft.arrow_term.choice_interpret_sound` | interpreting the choice generators (`left`/`right`/`choice`/`fanin`) agrees with the eager ArrowChoice combinators — routing on the sum node, `fanin` unwrapping (extends `interpret_sound` to the `⊕`-enlarged set) | `Haft/ArrowTerm.lean` | ✓ |
| `haft.arrow_term.choice_free` | the free/universal property extends to the `⊕`-enlarged generator set: agree on generators ⇒ agree on every choice term | `Haft/ArrowTerm.lean` | ✓ |
| `haft.interpreter.preserves_id` | interpreter `ArrowTerm → Kleisli<M>` is functorial: `id ↦` target identity (`pure`) | `Haft/Interpreter.lean` | ✓ |
| `haft.interpreter.preserves_compose` | interpreter is functorial: `compose f g ↦` target composition (`bind`) | `Haft/Interpreter.lean` | ✓ |
| `haft.interpreter.choice_preserved` | `interpret_kleisli` preserves the choice generators: `left`/`right`/`choice`/`fanin` map to the Kleisli choice arms, the effect runs only on the taken branch (extends `preserves_id`/`preserves_compose`) | `Haft/Interpreter.lean` | ✓ |
| `haft.interpreter.naturality` | `Option ⇒ List` component (`OptionToVec`) commutes with `map` (naturality square) | `Haft/Interpreter.lean` | ✓ |
| `haft.monoidal.comonoid_laws` | copy comonoid `(Δ, ε)`: coassociativity, counit, cocommutativity of the diagonal | `Haft/SymmetricMonoidal.lean` | ✓ |
| `haft.monoidal.merge_monoid_laws` | merge monoid `(∇, η)`: associativity + left/right unit (the monoid laws) | `Haft/SymmetricMonoidal.lean` | ✓ |
| `haft.monoidal.symmetry` | symmetry `σ` is its own inverse (`σ ∘ σ = id`) | `Haft/SymmetricMonoidal.lean` | ✓ |
| `haft.endo.monoid` | `End(T)` monoid (unit + associativity) | `Haft/Endomorphism.lean` | ✓ |
| `haft.endo.iterate_add` | `f^(m+n) = f^n ∘ f^m` | `Haft/Endomorphism.lean` | ✓ |
| `haft.morphism.identity` | `apply identity a = a` | `Haft/Morphism.lean` | ✓ |
| `haft.adjunction.triangles` | triangle identities (currying adjunction) | `Haft/Adjunction.lean` | ✓ |
| `haft.adjunction.adjunct_inverse` | adjuncts are the Hom-bijection | `Haft/Adjunction.lean` | ✓ |
| `haft.foldable.pure_compat` | `fold (pure x) init f = f init x` | `Haft/Foldable.lean` | ✓ |
| `haft.foldable.fold_map_pure` | `fold_map(pure a, f) = f a` (singleton law) | `Haft/Foldable.lean` | ✓ |
| `haft.foldable.fold_map_monoid_coherence` | `fold_map(xs ++ ys, f) = fold_map(xs,f).combine(fold_map(ys,f))` (monoid homomorphism) | `Haft/Foldable.lean` | ✓ |
| `haft.traversable.identity` | `sequence` at Identity applicative = id | `Haft/Traversable.lean` | ✓ |
| `haft.traversable.naturality` | applicative morphisms commute with `sequence` | `Haft/Traversable.lean` | ✓ |
| `haft.natural_iso.laws` | round-trip + naturality (`Option ≅ Unit ⊕ ·`) | `Haft/NaturalIso.lean` | ✓ |
| `haft.either.coproduct_universal` | `[f,g]` exists and is unique | `Haft/Either.lean` | ✓ |
| `haft.effect3.monad_laws` | monad laws + raise-left-zero (sum carrier) | `Haft/EffectSystem.lean` | ✓ |
| `haft.io.monad_laws` | monad laws on the `run` denotation | `Haft/Io.lean` | ✓ |
| `haft.cybernetic.kleisli_factorization` | `control_step` = Kleisli composite | `Haft/Signatures.lean` | ✓ |
