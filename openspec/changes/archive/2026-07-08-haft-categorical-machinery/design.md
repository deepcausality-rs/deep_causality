## Context

`deep_causality_haft` already carries the algebraic substrate the causaloid formalization sits on: `Functor`/`Applicative`/`Monad`/`CoMonad`/`Pure`, `Free<F, A>` with `fold` (a working catamorphism, `free_monad.rs`), the value-level `Arrow` with concrete defunctionalized combinators (`compose/split/fanout/first/second/id/lift`, no `dyn`), `Traversable`/`Foldable`/`Adjunction`/`Bifunctor`/`Profunctor`/`MonoidalMerge`, and the arity-1..6 HKT witness machinery. The Lean layer proves the monad, arrow, free-monad, and Kleisli-category laws.

What is missing is exactly the machinery to (a) fold `Collection` into a monoid, (b) name the Kleisli semantic target as a code type, (c) reify the free Arrow so the graph is *interpreted*, (d) interpret it one-way into Kleisli, and (e) express the reconvergence copy/merge as a symmetric-monoidal category. These are gaps B1/B4/B3/B5 (`algebraic-causaloid.md`) plus the PROP the merge needs (`algebraic-causaloid-assumptions.md` #2 Q2).

## Goals / Non-Goals

**Goals:**
- Add `Foldable::fold_map`, a named `Category`+`Kleisli`, a reified free Arrow (`ArrowTerm`), a one-way interpreter (`NaturalTransformation`/`ArrowInterpreter`), and a symmetric-monoidal PROP with copy (Δ)/merge (∇) generators.
- Test every law (Rust law-tests, Bazel-registered) and **prove every law in Lean** (bare-`lean`, THEOREM_MAP-bound, Rust witness) — full formalization.
- Preserve the crate's constraints: `unsafe_code = "forbid"`, no `dyn`, macro-free `/src`, no-std, no external deps.

**Non-Goals:**
- Re-implementing `evaluate` in `deep_causality` (that is `formalize-main-crate`'s B2 refactor consuming this machinery).
- Deciding the `∇` merge's *use* in the graph engine (that is the deferred symmetric-monoidal extension change; this change provides the H5 substrate it will use).
- The generic `Monoid` itself (delivered by `num-generic-monoid-tower`; a hard dependency of H1).

## Decisions

**D1 — `fold_map` as a default over `fold` + `Monoid::combine` (H1).** `fn fold_map<A, M: Monoid>(fa: F::Type<A>, f: impl Fn(A) -> M) -> M { fold(fa, M::empty(), |acc, a| acc.combine(f(a))) }`. Depends on the `num-generic-monoid-tower` `Monoid`. The law to prove: `fold_map(pure a, f) = f a` and monoid-homomorphism coherence (`fold_map` respects `empty`/`combine`). Rationale: this is the direct expression of `Collection` as a monoidal fold; order-independence then reuses `CommutativeMonoid` laws.

**D2 — `Category` + `Kleisli<M>` (H2).** `pub trait Category { type Obj; fn id(...) -> ...; fn compose(f, g) -> ...; }` packaging id + associative composition, and `Kleisli<M: Monad>` implementing it with `compose = bind`. The category *laws* are already proved in Lean (`core.causal_arrow.category_laws`); this adds the *code type* so the interpreter (D4) has a typed codomain, and re-states the laws as `haft`-level law-tests over the abstract `Category`. Both `Arrow` and `Kleisli` should satisfy `Category` (the note's B4 closing move). Rationale: without a named codomain there is nothing to state the interpretation functor against.

**D3 — Reified free Arrow `ArrowTerm` (H3), typed-by-construction / erased-core (assumption #3).** Reify the existing combinators as data: `ArrowTerm<In, Out>` (or a typed builder that lowers to an erased core AST). Rust has no GADTs, so a *uniform storable* term and *static `In`/`Out`* are mutually exclusive; the decision (per assumption #3 Q1/Q2) is **typed construction, erased storage** — a typed builder API guarantees well-typed-by-construction, lowering into an untyped core term for storage/interpretation/rewriting. No `dyn`: the core is an enum of the defunctionalized combinators already present. Alternative recorded: adopt `CausaloidGraph` directly as the term language (deferred to `formalize-main-crate`, which owns the graph).

**D4 — One-way interpreter (H4).** `NaturalTransformation<F, G> { fn transform<T>(F::Type<T>) -> G::Type<T> }` (naturality: commutes with `fmap`) and/or `ArrowInterpreter<A, M: Monad>` mapping `ArrowTerm` (D3) into `Kleisli<M>` (D2). Laws proved: `preserves id`, `preserves compose` (functoriality). Rationale: `evaluate` is one-way and non-invertible (you cannot recover the graph from a composed Kleisli arrow), so the bidirectional `NaturalIso` is the wrong tool; this is the `evaluate`-as-unique-interpretation-functor substrate (assumption #6).

**D5 — Symmetric-monoidal PROP with copy Δ / merge ∇ (H5).** A symmetric-monoidal / free-Markov structure over the effect monad with explicit **copy comonoid** (`Δ: A → A ⊗ A`, discard `ε: A → I`) and **merge** (`∇: A ⊗ A → A`) generators, plus the symmetry (swap) and coherence laws. `CoMonad` and `MonoidalMerge` are partial building blocks (a comonad is not a comonoid object; `MonoidalMerge` is the lax-monoidal tensor structure map, not Δ/∇), so this is a new structure built on them. Laws proved: comonoid (coassociativity, counit), monoid (associativity, unit) for the merge, and symmetry/naturality. Rationale: this is the categorical home the reconvergence merge (`∇`, assumption #2 Q2) needs; the deferred extension change consumes it. Scope here is the *substrate + laws*, not its wiring into the graph engine.

**D6 — Lean formalization, one law one theorem, house style.** Each law is proved in `DeepCausalityFormal/Haft/*.lean` (bare-`lean`, self-contained, transcribe minimal structures as the existing Haft files do), bound to a `THEOREM_MAP.md` id under the Haft layer with a Rust witness in `deep_causality_haft/tests/`. Reuse the proven `haft.free_monad.*`/`haft.arrow.*`/`core.causal_arrow.*` where the new laws reduce to them (e.g. `Kleisli` category laws cite the causal-arrow proof shape; `ArrowInterpreter` functoriality cites the arrow laws).

## Risks / Trade-offs

- **[GADT-lessness — H3]** a uniform `ArrowTerm` and compile-time wiring safety cannot coexist in Rust. → Decide typed-builder-over-erased-core (D3); reword the "type system rejects every nonsensical graph" claim to "well-typed by construction at build time, executed from an erased core" (assumption #3 Q3).
- **[Monomorphisation blow-up]** a fully type-indexed non-uniform arrow term explodes compile time at depth. → The erased core avoids it; prototype `Compose<First<…>>` at depth ~5 to confirm (assumption #3 Q2).
- **[Δ/∇ over the effect monad interacts with the log/state channels]** copy duplicates the log; merge must combine channels. → Scope H5 to the categorical substrate + laws over an abstract carrier; the channel-combine policy is the deferred extension's concern (it will pick the value monoid from `num` and a log/state merge).
- **[H5 breadth]** a full Markov category is large. → Deliver the minimal generators (Δ, ε, ∇, swap) + their laws, transcribed self-contained; do not pull heavy category-theory Mathlib.

## Migration Plan

Additive; depends on `num-generic-monoid-tower` landing first (for `fold_map`). Steps: (1) `Foldable::fold_map` + law + Lean + witness; (2) `Category` + `Kleisli` + laws + Lean + witnesses; (3) `ArrowTerm` (typed builder + erased core) + interpretation + Lean + witnesses; (4) `NaturalTransformation`/`ArrowInterpreter` + functoriality + Lean + witnesses; (5) symmetric-monoidal PROP (Δ/∇/swap) + laws + Lean + witnesses; (6) `bazel test //deep_causality_haft/...` green, bare-`lean` on every new `Haft/*.lean` exit 0. Rollback = remove the new files; the existing haft surface is unaffected.

## Open Questions

- H3: typed-builder-over-erased-core vs adopting `CausaloidGraph` as the term language? — lean to the typed builder here; the graph-as-term option is `formalize-main-crate`'s.
- H4: a general `NaturalTransformation<F,G>` vs a specialized `ArrowInterpreter<A, Kleisli<M>>`? — provide the specialized interpreter for the causaloid path; the general NT if it is cheap and reusable.
- H5: how much of the Markov-category structure to build now vs. with the deferred merge extension? — minimal generators + laws now; the wiring later.
