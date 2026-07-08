## Why

The full formalization of the causaloid requires it to be written as **`(free Arrow) —interpreter→ (named Kleisli category)`**, with `Collection` a monoidal fold and the graph's fan-out/fan-in a symmetric-monoidal category. `deep_causality_haft` supplies most of the substrate — `Functor`/`Applicative`/`Monad`/`CoMonad`, the `Free<F, A>` monad *with* `fold` (a catamorphism), the value-level `Arrow` combinators, `Traversable`/`Adjunction`/`Bifunctor`/`Profunctor`/`MonoidalMerge`, and the Lean proofs of the monad/arrow/free-monad/Kleisli-category laws — but five categorical pieces the causaloid formalization needs are missing (gaps B1/B3/B4/B5 in `algebraic-causaloid.md`, plus the symmetric-monoidal PROP the reconvergence merge `∇` needs, `algebraic-causaloid-assumptions.md` #2 Q2):

- `Foldable` has only a seeded `fold`, no `fold_map` (B1) — so `Collection = fold_map into a monoid` is unwritable.
- There is no named `Category`/`Kleisli` **type** (B4) — the Kleisli category *laws* are Lean-proved (`core.causal_arrow.category_laws`) but there is no code type to be the interpreter's typed codomain.
- The `Arrow` is value-level only; there is no reified free Arrow / `ArrowTerm` (B3) — so the graph cannot be *interpreted* (rather than eagerly `run`), and there is no free object for the universal property.
- There is only a bidirectional `NaturalIso`; no one-way `NaturalTransformation` / interpreter (B5) — but `evaluate` is a one-way, non-invertible map from syntax to semantics.
- There is `CoMonad` and `MonoidalMerge` (the lax-monoidal tensor structure map), but no symmetric-monoidal / PROP category with **copy (comonoid Δ)** and **merge (∇)** generators over the effect monad — the structure the reconvergence merge is.

This change adds those five pieces, each tested and fully formalized in Lean. It depends on the generic-monoid tower delivered by `num-generic-monoid-tower` (for `fold_map`).

## What Changes

- **`Foldable::fold_map` (H1).** `fn fold_map<A, M: Monoid>(fa, f: Fn(A) -> M) -> M`, a default in terms of `fold` + `Monoid::combine` (the `Monoid` from `num-generic-monoid-tower`). Proves `Collection` is a monoidal fold and lets order-independence reuse the monoid laws.
- **`Category` + `Kleisli<M>` (H2).** A `Category` trait (`id`, `compose`) and a `Kleisli<M: Monad>` newtype implementing it (`compose = bind`), giving the interpretation a typed codomain and turning "Kleisli is a category" into a structural law-test. Retires the informal Kleisli language in `io/mod.rs`.
- **Reified free Arrow (H3).** An `ArrowTerm<In, Out>` — the free Arrow over the generator set (or the formal adoption of the graph as the term language) — reifying the existing `compose/split/fanout/first/second/id/lift` combinators as data, with no new `unsafe`/`dyn`. This is the syntax the graph interprets, and the free object the universal property needs. (Per assumption #3: uniform storable term + static `In`/`Out` are mutually exclusive without GADTs — this change decides "typed-by-construction, erased core.")
- **One-way interpreter (H4).** A `NaturalTransformation<F, G>` and/or `ArrowInterpreter<A: Arrow, M: Monad>` mapping the free Arrow (H3) into `Kleisli` (H2), with functoriality (`preserves id`, `preserves compose`) as tested + proved laws. This is what makes `evaluate` a *unique interpretation functor*.
- **Symmetric-monoidal PROP with Δ/∇ (H5).** A symmetric-monoidal / free-Markov structure over the effect monad with explicit **copy/discard comonoid Δ** (fan-out) and **merge ∇** (fan-in) generators — the categorical home the reconvergence merge (assumption #2 Q2) needs, and the attachment point for the deferred do-surgery / QCM factorization.
- **Tests + Lean.** Every law (fold_map monoid coherence, category laws, arrow-term interpretation, interpreter functoriality, comonoid/monoid + symmetry laws) is exercised by a Rust law-test (Bazel-registered) and **proved in Lean** under `DeepCausalityFormal/Haft/`, THEOREM_MAP-bound with a Rust witness, bare-`lean` typecheck.

## Capabilities

### New Capabilities
- `haft-foldable-foldmap`: `Foldable::fold_map` into a generic `Monoid`, with the fold/monoid coherence law.
- `haft-category-kleisli`: a named `Category` trait and a `Kleisli<M: Monad>` category (compose = bind), with the category laws as a code-level structure (the Lean laws already exist).
- `haft-free-arrow`: a reified free Arrow (`ArrowTerm`) over the generator set, typed-by-construction with an erased core.
- `haft-arrow-interpreter`: a one-way `NaturalTransformation` / `ArrowInterpreter` mapping the free Arrow into `Kleisli`, with functoriality laws.
- `haft-symmetric-monoidal-prop`: a symmetric-monoidal / PROP category over the effect monad with copy (Δ) and merge (∇) generators — the reconvergence-merge substrate.

### Modified Capabilities
<!-- Additive new traits/types alongside the existing haft surface; no existing requirement changes. -->

## Impact

- **New `haft` surface:** `Foldable::fold_map`; `src/category/` (`Category`, `Kleisli`); `src/arrow/arrow_term.rs` (free Arrow) + interpreter (`src/arrow/interpreter.rs` or `src/natural_transformation/`); `src/monoidal/` (symmetric-monoidal PROP with Δ/∇). Exported from `lib.rs`.
- **New Lean:** `DeepCausalityFormal/Haft/{FoldMap,Category,Kleisli,ArrowTerm,Interpreter,SymmetricMonoidal}.lean` (or extend existing `Haft/{Foldable,Arrow}.lean`), registered in `DeepCausalityFormal.lean`; new `THEOREM_MAP.md` rows under the Haft layer.
- **New Rust witnesses/tests:** `deep_causality_haft/tests/**` law-tests, registered in `tests/BUILD.bazel`.
- **Depends on `num-generic-monoid-tower`** (H1 needs the generic `Monoid`). **Unblocks** `formalize-main-crate` (causaloid = free-Arrow-interpreted-into-Kleisli; `evaluate` as a catamorphism) and the deferred symmetric-monoidal merge (∇) extension that decides assumption #2.
- **No external dependencies;** `unsafe_code = "forbid"`, no `dyn`, macro-free `/src`, no-std compatible — consistent with the crate's constraints. The free Arrow is the enum reification of the combinators already present, not new unsafe.
