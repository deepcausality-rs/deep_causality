<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# The Algebraic Causaloid

**Status:** design note. Not implemented. Documents the trait-hierarchy gaps that must close *before* the causaloid can be formalized in code, then states the formalization and what it unlocks.

## Why this note exists

Almost every core construct in DeepCausality is already pinned to a named algebraic or categorical structure:

- `PropagatingEffect` is a monad (the causal monad), verified against the three monad laws.
- The effect-propagation process implements `Functor`, `Applicative`, `Monad` over an HKT witness.
- Discovery/pipeline composition is an `Arrow` (a strong category / Freyd category); the causal monad is its Kleisli fragment (see `causal-arrow-generalization.md`).
- The number tower (`Magma → … → Field`, `Module`, `Algebra`, `DivisionAlgebra`) is a faithful abstract-algebra hierarchy (see `deep_causality_num/README_ALGEBRA_TRAITS.md`).

The **causaloid is the exception**. Its three forms — `Singleton`, `Collection`, `Graph` (`CausaloidType`) — are unified at runtime by an enum plus trait dispatch, not by a stated structure. A natural isomorphism was floated as the missing formalism and rejected: the three forms are **not isomorphic** (a `Graph` carries strictly more structure than a `Singleton`; `graph → singleton → graph` cannot round-trip). The relationship is a **coproduct of constructors related by a unique homomorphism out of the whole**, i.e. a **free algebra and its fold** — not a bijection.

That correct structure cannot be written in code yet, because the algebra/category hierarchy is missing several prerequisites. This note documents those gaps first.

---

## Part 1 — Gaps to close first

Each gap lists: what is missing, the evidence, why the causaloid needs it, and the minimal closing move. Gaps are split into the numeric/algebraic layer (`deep_causality_num`) and the categorical layer (`deep_causality_haft`).

### A. Algebraic gaps — `deep_causality_num`

#### A1 — No generic `Monoid`. The monoid traits are numeric-only.

**Evidence.** `src/algebra/monoid.rs`: `AddMonoid: Add<Output = Self> + AddAssign + Zero + Clone` and `MulMonoid: MulMagma + One + Associative`. `Zero`/`One` bake in `Add`/`Mul` respectively. There is no generic monoid over an arbitrary carrier and binary operation anywhere in the crate (verified: no `trait Monoid` exists).

**Why the causaloid needs it.** `Collection` aggregation reduces child results that are **not numbers** — booleans, success probabilities, an `EffectValue`. A monoid over `bool` under `∧`, or over a count, cannot be expressed: `bool` implements neither `Add` nor `Zero`. The entire monoid tower is unreachable for the aggregation carrier.

**Closing move.** Add a carrier-and-operation-generic monoid, e.g. `Monoid { fn empty() -> Self; fn combine(self, other: Self) -> Self; }` (associativity + identity laws), decoupled from `Zero`/`One`. Keep `AddMonoid`/`MulMonoid` as the numeric specializations they already are.

#### A2 — No `CommutativeMonoid`, `Semilattice`, or `Idempotent` marker.

**Evidence.** None of `CommutativeMonoid`, `Semilattice`, `BoundedSemilattice`, `IdempotentMonoid`, or an `Idempotent` marker exist (verified). The only commutativity carriers are `AbelianGroup` (requires inverses) and `CommutativeRing` (requires a full ring) — both far too strong for an aggregation reducer.

**Why the causaloid needs it.** The four `AggregateLogic` cases (`src/types/causal_types/aggregate_logic/mod.rs`) are exactly these structures:

| `AggregateLogic` | Boolean reading | Probability reading (`monadic_collection_utils.rs`) | Algebra |
|---|---|---|---|
| `All` | `∧` over children | `∏ pᵢ` | bounded **∧-semilattice** (idempotent comm. monoid, id = `true`/`1`) |
| `Any` | `∨` over children | `1 − ∏(1 − pᵢ)` | bounded **∨-semilattice** (idempotent comm. monoid, id = `false`/`0`) |
| `None` | `∧` over negations | `∏(1 − pᵢ)` | `Any` post-composed with negation |
| `Some(k)` | at-least-`k` trues | `≥ k` of the events | **count monoid** `(ℕ, +, 0)` followed by a `≥ k` threshold |

All four are **symmetric functions** — they depend only on the *multiset* of child results, never the order. That fact is currently implicit; without a commutative/idempotent abstraction it cannot be stated or enforced.

**Closing move.** Add `CommutativeMonoid: Monoid` (commutativity law) and an `Idempotent` marker; optionally `BoundedSemilattice: CommutativeMonoid + Idempotent`. `Some(k)` decomposes as a `CommutativeMonoid` on counts plus a threshold predicate — no idempotence claimed there.

#### A3 — Law markers are bare and not operation-scoped.

**Evidence.** `Associative`, `Commutative`, `Distributive` (`associative.rs`, `commutative.rs`, `distributive.rs`) are empty markers with no operation parameter. This is adequate for the numeric tower, where each type has one canonical `+` and `×`. It cannot express "operation `combine` on carrier `Verdict` is idempotent and commutative" independently of `Add`/`Mul`.

**Why the causaloid needs it.** The aggregation laws attach to the *aggregation* operation, not to `Add`/`Mul` on the carrier. The marker must travel with the monoid operation introduced in A1.

**Closing move.** Let the A1/A2 traits carry their own laws (the law lives on `combine`), rather than retrofitting operation parameters onto the global markers. Smallest addition: an `Idempotent` marker consumed by `BoundedSemilattice`.

#### A4 — No free monoid / free algebra construction.

**Evidence.** No `Free*`, generators-and-relations, or term construction in the crate (verified). The `Algebra<R>` hierarchy is concrete number systems only.

**Why the causaloid needs it.** Lower priority. The free structure the causaloid needs lives at the categorical layer (B3, the free Arrow), not the numeric layer. Recorded for completeness; no numeric-side action required.

### B. Categorical gaps — `deep_causality_haft`

#### B1 — `Foldable` has no monoidal fold.

**Evidence.** `src/foldable/mod.rs`: the only method is `fold<A, B, Func>(fa, init: B, f: FnMut(B, A) -> B) -> B` — a seeded left fold. There is no `fold_map<A, M: Monoid>(fa, f: Fn(A) -> M) -> M` (verified).

**Why the causaloid needs it.** `Collection` *is* a `fold_map` into the aggregation monoid of A1/A2: map each child to its verdict, fold the commutative monoid, decide. A seeded left fold cannot express order-independence or reuse the monoid laws.

**Closing move.** Add `fold_map<A, M: Monoid>` to `Foldable` (a default in terms of `fold` + `Monoid::combine`). Depends on A1.

#### B2 — No F-algebra / catamorphism / fixpoint machinery.

**Evidence.** No `Fix`/`Mu`/`Nu`, no `cata`/`ana`, no F-algebra trait anywhere in `haft` (verified). `Foldable` folds already-concrete containers; there is no recursion scheme over an inductive type.

**Why the causaloid needs it.** `Causaloid = μX. F(X)` for the signature functor `F` below. `evaluate` should be the **catamorphism** — the unique `F`-algebra homomorphism into the semantic domain. Today `evaluate` is *not* a catamorphism: the base `MonadicCausable::evaluate` (`deep_causality/src/types/causal_types/causaloid/causable.rs`) handles only `Singleton` and returns `"… not available in this build; use specialized … APIs"` for `Collection`/`Graph`, whose real logic lives in separate `collection_reasoning`/`graph_reasoning` traits. That split is the symptom of an un-formalized fold.

**Closing move.** A full `Fix`/`cata` framework is not required and would fight the no-`dyn` policy. The minimal move is to express `evaluate` as a single **total structural fold** with one explicit `F`-algebra (three clauses), and state its universal property in prose + property tests. Reify the recursion in data (B3), not in a higher-kinded `Fix`.

#### B3 — `Arrow` is value-level only; there is no free (syntactic) Arrow.

**Evidence.** `src/arrow/mod.rs`: `Arrow { type In; type Out; fn run(&self, In) -> Out; … }` with concrete defunctionalized combinators `Id`, `Lift`, `Compose`, `First`, `Second`, `Split`, `Fanout`. Every combinator returns a new concrete type. There is no arrow-term AST and no functor interpreting arrow terms into another `Arrow` (e.g. into Kleisli) (verified).

**Why the causaloid needs it.** The `Graph` form is a morphism in the free (traced) symmetric monoidal category generated by singleton causal functions — i.e. a **reified arrow term**, the DAG itself. To *interpret* that DAG (rather than eagerly `run` it) the wiring must exist as data. The codebase already chose defunctionalization (concrete combinator types over `dyn`); a free Arrow is just the **enum reification of those same combinators**, which fits the `unsafe_code = "forbid"` / no-`dyn` policy exactly. Note `Morphism` (`morphism/morphism_base.rs`) deliberately omits composition because composing closures yields an unnameable type — reifying composition as a data constructor is the policy-consistent escape.

**Closing move.** Introduce an `ArrowTerm<In, Out>` enum (the free Arrow over a generator set), or treat the existing `CausaloidGraph` as that term language directly. No new unsafe, no `dyn`.

#### B4 — No explicit Kleisli category and no `Category` trait.

**Evidence.** Kleisli appears only in doc-comments (`io/mod.rs`: "a nullary Kleisli arrow `() ⇝ A`"). There is no `Kleisli<M, A, B>` type and no `Category` trait packaging objects + hom + `id` + `compose` (verified). `Monad::bind` exists (`monad/mod.rs`), but the Kleisli *category* it generates is not named.

**This is not a missing monad — it is a missing packaging.** The monad is present and is the causal monad; mathematically the Kleisli category *is* that monad presented as a category (same data, inter-derivable). The gap is that `Monad<F>::bind` operates on *wrapped values* `F::Type<A>` and is not itself an `Arrow`: it has no `In`/`Out`/`run`/`compose` at the morphism level, so a Kleisli arrow cannot be stored as a value or composed as a category morphism, and `Monad<F>` cannot serve as the typed codomain of the interpreter (B5). Note that `evaluate`'s shape `F<I> → F<O>` is exactly the Kleisli *extension* operator `k*`, composed by ordinary function composition — Kleisli-shaped already, merely unreified.

**Why the causaloid needs it.** The interpretation target is `Kleisli(PropagatingEffect)` — its objects are effect-typed values and its morphisms are `PropagatingEffect<I> → PropagatingEffect<O>`, which is precisely the `MonadicCausable::evaluate` signature. Without a named Kleisli category there is no codomain to state the interpretation functor against.

**Closing move.** Add a `Kleisli<M: Monad>` newtype implementing `Arrow` (its `compose` is `bind`), and optionally a small `Category` trait (`id`, `compose`) that both `Arrow` and `Kleisli` satisfy. This also retires the informal Kleisli language in `io/mod.rs`.

#### B5 — No one-directional natural transformation / interpreter.

**Evidence.** Only `NaturalIso<F, G>` (`iso/natural_iso.rs`), which is bidirectional (`to_target` + `to_source`). There is no one-way `NaturalTransformation<F, G>` and no `Interpreter`/`Compiler` abstraction (verified).

**Why the causaloid needs it.** `evaluate` is a one-way, structure-preserving map from the free Arrow (syntax) into Kleisli (semantics). It is **not** invertible — you cannot recover the syntactic graph from a composed Kleisli arrow — so `NaturalIso` is the wrong tool (this is the same reason the iso framing failed for the three forms). What is needed is a one-directional natural transformation / interpreter.

**Closing move.** Add `NaturalTransformation<F, G> { fn transform<T>(F::Type<T>) -> G::Type<T> }` (the naturality law: commutes with `fmap`), and/or an `ArrowInterpreter<A: Arrow, M: Monad>` that maps an `ArrowTerm` into `Kleisli<M>`.

### Dependency ordering of the gaps

```
A1 generic Monoid ──► A2 CommutativeMonoid/Semilattice/Idempotent ──► B1 Foldable::fold_map ──► Collection form
B4 Kleisli + Category ──┐
B3 free Arrow (ArrowTerm / reified CausaloidGraph) ──┼──► B5 interpreter (NT / ArrowInterpreter) ──► Graph form + evaluate
B2 evaluate-as-total-fold ──────────────────────────┘
A3 Idempotent marker rides along with A2.   A4 free monoid: not required.
```

Minimal critical path: **A1 → A2 → B1** (Collection) and **B4 → B3 → B5** (Graph + the unifying interpreter), with **B2** as the refactor that consumes them.

---

## Part 2 — The algebraic causaloid

Once the gaps close, the formalization is direct.

### Signature functor and carrier

```
F(X) = Atom( CausalFn<I,O> )                 // Singleton — a generator
     + Coll( List<X>, AggregateLogic )       // Collection — n-ary combination
     + Graph( CausaloidGraph<X> )            // Graph — typed wiring
```

`Causaloid ≅ μX. F(X)`, the least fixpoint. The recursive Rust enum (`causal_coll: Arc<Vec<Self>>`, `causal_graph: Arc<CausaloidGraph<Self>>`) **is** this initial algebra already — an algebraic data type is the initial algebra of its polynomial functor. The structure is present; only its name and its fold are missing.

### Syntax vs semantics

- **Syntax (free Arrow).** The causaloids are the morphisms of the **free identity-on-objects Arrow (Freyd category) generated by the singleton causal functions**. `Singleton` is a generator; `Graph` is composition/wiring in this free category (the `Arrow` combinators of B3); `Collection` is a commutative-monoidal fold *within a single hom-set* (B1/A2).
- **Semantics.** `evaluate` is the **unique interpretation functor** from that free Arrow into `Kleisli(PropagatingEffect)` (B4), i.e. the catamorphism whose `F`-algebra is:
  - `Atom(f)` ↦ run `f` as a Kleisli arrow `PropagatingEffect<I> → PropagatingEffect<O>`;
  - `Coll(children, logic)` ↦ fan-out, map each child, fold the `AggregateLogic` commutative monoid, decide;
  - `Graph(dag)` ↦ interpret the wiring via `Compose`/`Split`/`Fanout` into one composite Kleisli arrow.

### The core property

A causaloid folds cause and effect into one entity: a morphism `PropagatingEffect<I> → PropagatingEffect<O>` in the Kleisli category of the causal monad. The three forms are three **constructors** of such morphisms, interpreted by one fold. The universal property of the free Arrow gives the load-bearing result:

> Fix the interpretation on the generators (atoms) and on the combinators (the aggregation monoid and Arrow composition). Then the interpretation of **every** causaloid — singleton, collection, graph, and all nestings — is **uniquely determined**.

### Aggregation is a multiset operation (theorem, once A2 lands)

Because `All`/`Any`/`None`/`Some(k)` are symmetric functions, `Collection`'s `Vec<Self>` is **semantically a multiset**: child order carries no meaning. With the commutative-monoid abstraction this is a stated, testable law rather than an implicit assumption — and it licenses parallel evaluation directly.

---

## Part 3 — What the new formalism enables that was not possible before

1. **One total `evaluate`.** The fold collapses the three dispatch paths into a single catamorphism and deletes the `"not available in this build"` branches in `causable.rs`. The `collection_reasoning`/`graph_reasoning` traits become *algebra clauses*, not separate entry points. The split-brain disappears.

2. **A uniqueness theorem for the causal chain.** Freeness turns correctness of the whole engine into correctness on generators + combinators: any alternative reasoning engine that agrees on atoms, aggregation, and composition is *forced* to agree on every chain. This is a genuine result the system does not currently have.

3. **Order-independence and parallelism as a theorem.** The multiset property (Part 2) is provable from the commutative-monoid laws, not asserted. Parallel collection evaluation becomes sound by construction.

4. **Closure of the Causal Arrow programme.** `Graph = free Arrow interpreted into Kleisli` folds the last un-formalized core construct into the Arrow generalization (`causal-arrow-generalization.md`). The causaloid stops being a fourth ad-hoc structure and becomes a corollary of the Arrow already in `haft`.

5. **A law-based test backbone.** Functor/interpreter laws (composition preserved), monoid laws (aggregation), idempotence/commutativity (semilattice), and catamorphism fusion become the property-test suite — replacing scattered behavioural tests with structural ones.

6. **Optimization on syntax before semantics.** A reified free Arrow (B3) is rewritable *before* interpretation: fusion of adjacent atoms, dead-branch elimination on `Any`/`All`, constant-folding of pure sub-graphs. None of this is possible while the graph is an eager `run` with no syntactic layer.

7. **Reuse across the workspace.** The prerequisites are not causaloid-specific. A generic `Monoid`/`CommutativeMonoid`/`Semilattice` (A1/A2), `Foldable::fold_map` (B1), a named `Kleisli`/`Category` (B4), and a one-way `NaturalTransformation`/interpreter (B5) are reusable by the discovery DSL, the BRCD port, and the CFD `Flow` `.couple` seam — each of which already composes effect-typed stages by hand.

8. **Algorithmic generation of causaloids (the largest payoff).** Once `Causaloid` is the free Arrow generated by atomic causal functions plus a commutative-monoid fold, "well-shaped" stops being a judgement call and becomes a structural guarantee. The set of well-formed causaloids is *exactly* the set of terms a finite signature can build, which hands a learning or synthesis algorithm four things it does not have today:

   - **A typed grammar.** Generators (atoms) and combinators (`compose`, `split`, `fanout`, `aggregate`) are the *only* construction paths. Anything built through the constructors is well-formed by construction — no separate validation pass, and no ill-formed causaloid is representable.
   - **Type-checked wiring.** Arrow composition matches `In`/`Out`; a term is **well-typed by construction at build time** — the typed builder only composes when the objects line up — and then **executed from an erased core** (the storable `ArrowCore`), so mistyped wiring never becomes a term in the first place. The search space is the *type-correct* terms, not all syntactic terms.
   - **Free, unique semantics per candidate.** The interpretation functor (`evaluate`) gives every generated term a runnable meaning, so a learner can *score* any candidate by executing it; the universal property makes that meaning **unique**, so there is no ambiguity in "what did the algorithm just learn."
   - **A normal form / rewrite system.** The Arrow laws and aggregation-monoid laws are equations, so the search can be quotiented by semantic equivalence (never explore two provably-equal terms) and a learned causaloid can be normalised into a canonical, minimal shape. These equational laws *are* the "deterministic construction rules."

   This reframes "learn a causal model" as **program synthesis over a typed free algebra**: the learner emits terms in a typed grammar and scores them through the catamorphism, instead of generate-then-validate. It gives `deep_causality_discovery` (CDL), the BRCD port, and SURD a **typed target language** to emit into — they currently produce raw graph structure with no guaranteed executability or normal form.

   **Boundary (kept honest).** The formalism guarantees generated causaloids are well-formed, type-correct, executable, and normalisable. It does **not** guarantee they are *causally true* — fit-to-reality is the learner's scoring job. The algebra supplies the grammar and the semantics; the learning algorithm supplies the search and the empirical fit. The win is that the learner's *entire output space is valid by construction and every candidate is scoreable*, which is what turns causal learning into a well-posed search.

---

## Concrete vs witness-generic: decided by rigor, not by ergonomics

The HKT **witness types** in `deep_causality_haft` exist only because Rust has no native higher-kinded types: `F<_>` is encoded as a zero-sized marker carrying a GAT `type Type<T>`. They are a type-system workaround, not a domain concept, and the DSL keeps them out of the user's sight (`deep_causality_cfd` mentions `Witness` zero times; the user-facing seam is `CausalFn<I,O> = fn(I) -> PropagatingEffect<O>`). That shielding lowers the *cost* of the witness boilerplate — it is **not** a reason to avoid it. The goal of this work is strict well-definedness, so the encoding of each gap is chosen by what it does for rigor:

- **Use the witness/HKT-generic encoding wherever genericity makes a categorical law canonical or type-enforced.** `Foldable::fold_map` (B1), `Kleisli<M>` + a `Category` trait (B4), and the one-way `NaturalTransformation`/interpreter functor (B5) belong here. A named `Category` with `Kleisli<M: Monad>: Category` makes "Kleisli is a category" a structural fact with law tests and gives the interpreter a *typed codomain*; an interpreter trait makes functoriality (`preserves id`, `preserves compose`) an asserted, tested law rather than a paragraph. This is precisely where the witnesses buy rigor, and the DSL hides the cost.
- **Keep concrete only where genericity is indirection, not definition.** The recursion scheme (B2) is the one such place. A generic `Fix`/`cata`/F-algebra tower in Rust fights the type system and buys *reuse*, not rigor, for this single fold. The concrete structural `evaluate` already **is** the catamorphism; its rigor comes from stating the universal property and property-testing the fusion law, which the concrete fold supports fully. Concrete here is a reasoned choice, not an omission.
- **A1/A2 are not a witness question at all** — they are ordinary algebraic traits on concrete carriers (`bool`, counts, probabilities). They are the highest rigor-per-effort items: the multiset/order-independence theorem becomes a consequence of stated laws.

## Implementation phases (for a later change proposal)

- **Phase 0a — `deep_causality_num`:** generic `Monoid`; `CommutativeMonoid`; `Idempotent` marker; `BoundedSemilattice`. (A1–A3)
- **Phase 0b — `deep_causality_haft`:** `Foldable::fold_map`; `Kleisli<M>` + minimal `Category`; one-way `NaturalTransformation` / `ArrowInterpreter`; reify the free Arrow (or adopt `CausaloidGraph` as the term language). (B1, B3, B4, B5)
- **Phase 1 — `deep_causality`:** re-express `Causaloid::evaluate` as the single interpretation functor / total fold; wire `Graph` to the `Arrow`; wire `Collection` to the aggregation monoid; encode the laws as property tests. (B2)

**Scope and risk.** Phase 0 is additive (new traits, no breakage). Phase 1 is a real partial rewrite of `evaluate` and the collection/graph reasoning traits, with a moderate blast radius into their tests — which is the intended cost: the rewrite is justified because it adds a uniqueness theorem, a commutativity theorem, and the Arrow unification, while removing an existing split-brain. Per `AGENTS.md`, an API change drives its tests, not the reverse.

## Related notes

- [`algebraic-causaloid-assumptions.md`](algebraic-causaloid-assumptions.md) — **the hidden/implicit assumptions this note rests on**, tracked as OPEN/DECIDED. No claim here is "settled" while its assumption is still OPEN.
- `openspec/notes/causaloid/` — this directory.
- `causal-arrow-generalization.md` — the Arrow ⊋ causal-monad thesis that Part 2 closes.
- `deep_causality_num/README_ALGEBRA_TRAITS.md` — the existing numeric tower the A-gaps extend.
- `deep_causality_num/README_ISOMORPHISM.md` — Tier-1/2/3 iso machinery; explains why `NaturalIso` is the *wrong* tool here (B5) and a one-way interpreter is the right one.
