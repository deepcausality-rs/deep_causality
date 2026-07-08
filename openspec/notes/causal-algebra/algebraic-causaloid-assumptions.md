<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Algebraic Causaloid — Assumptions & Open Questions

**Companion to** [`algebraic-causaloid.md`](algebraic-causaloid.md). That note states the formalization; this one tracks every hidden or implicit assumption it rests on, so each can be interrogated, decided, and documented before the formalization is treated as settled.

**Purpose.** The goal is to formalize the causaloid in a rigorous abstract algebra and/or category theory, which first requires the *right* abstraction. A partial or full rewrite of the causaloid to reach that formalization is on the table, but every such rewrite must be justified. These assumption tests exist to separate what the current design already justifies from what a rewrite would have to earn.

**Do not promote any claim in `algebraic-causaloid.md` to "settled" while the assumption it depends on is `OPEN`.**

## How to use this tracker

Each entry has a fixed shape:

- **Assumption** — the claim, restated as a falsifiable statement.
- **Depends-on / Affects** — which gap (A1–B5) or Part-N claim breaks if the assumption is false.
- **Confounder / bias** — why the assumption looks true; the cognitive trap that planted it.
- **Status** — `OPEN` | `INVESTIGATING` | `DECIDED` | `REJECTED`.
- **Resolves when** — the concrete question(s) whose answer settles it. Each is answerable from code or by an explicit design decision.
- **Decision** — left blank until resolved; record the ruling + date + who, and link the commit/test/spec that enforces it.

Severity: **L** = load-bearing (falsifies a headline claim if wrong); **C** = caveat (scopes/weakens a claim, does not break it).

### Status board

| # | Title | Sev | Status |
|---|---|---|---|
| 1 | Order-independence of Collection | L | DECIDED |
| 2 | Graph form: which algebra? (join/reconvergence semantics) | L | INVESTIGATING |
| 3 | Reified free Arrow: uniform *and* statically typed | L | OPEN |
| 4 | Generation over a closed, decidable atom set | L | OPEN |
| 5 | Collection output is a verdict, not arbitrary `O` | C | OPEN |
| 6 | A single fixed semantic carrier | C | OPEN (premise discharged) |
| 7 | `PropagatingEffect` laws hold unconditionally | C | DECIDED |
| 8 | "Free Arrow" = "Arrow modulo laws" | C | OPEN (law-side formalized) |
| 9 | Nesting is well-founded (μX, not νX) | C | OPEN |
| 10 | The refactor is behaviour-preserving | C | OPEN |
| 11a | `CausaloidType` is closed at three forms | C | OPEN |
| 11b | Contextual atoms see a stable context per pass | C | OPEN |

---

## Load-bearing assumptions

### 1 — Order-independence of Collection
- **Assumption.** Collection evaluation is a commutative-monoid / multiset fold: child order is semantically irrelevant and parallel evaluation is sound.
- **Depends-on / Affects.** A2, B1; Part 2 "Aggregation is a multiset operation"; Part 3 item 3 (parallelism).
- **Confounder / bias.** *Streetlight + level-confusion.* The boolean verdict (`All`/`Any`) is genuinely symmetric; that easy fact is laundered into "the whole evaluation is order-independent," dropping the carrier. Evaluation is `PropagatingEffect<O>`, which threads an order-sensitive `EffectLog` (and `PS` on the stateful path). Symmetry of the verdict ≠ commutativity of the effect.
- **Status.** DECIDED
- **Resolves when.**
  - Q1. Is the canonical Collection path the stateless `MonadicCausable` one (`State = ()`), or does any production path thread `PS` across siblings? (Code: `collection_reasoning/monadic_collection.rs` vs `stateful_monadic_collection.rs`.) **Answered: both exist; the stateful path threads `PS` across siblings.**
  - Q2. Is `EffectLog` order semantically meaningful, or a reorderable multiset/bag? Decide: order-significant ⇒ Collection is *not* commutative; multiset ⇒ define a canonical commutative log-merge. **Answered: order-significant (append/chronological log).**
  - Q3. Restate the theorem as "order-independent **up to log permutation**, stateless path, siblings sharing no mutable context," and property-test exactly that. **Adopted as the scoped theorem below.**
- **Decision.** DECIDED 2026-06-26. The unqualified claim is **REJECTED**; the following scoped claim is accepted.

  **Scoped theorem.** Collection evaluation is order-independent in the **aggregated value only**, on the **stateless** `MonadicCausableCollection` path, in the **all-success** case, and **up to log permutation**. It is order-sensitive (a) in the `EffectLog` channel in all cases, (b) under the first-error short-circuit (which error surfaces, and the partial logs at failure, depend on order), and (c) in the **value** channel on the `StatefulMonadicCausableCollection` path (state is threaded across siblings, so each item evaluates against a different state).

  **Evidence.**
  - *Value symmetric:* `aggregate_effects` is a symmetric function for every `AggregateLogic` and carrier — bool `.all`/`.any`/count, f64 `product`/inclusion-exclusion/count, `UncertainBool` commutative `&`/`|` reduce (`utils/monadic_collection_utils.rs:50-87`). The stateless fold evaluates every item against the *same* `incoming_effect` (`collection_reasoning/monadic_collection.rs:76`), so items are independent.
  - *Log order-sensitive:* logs append in iteration order (`monadic_collection.rs` bind log-merge; `stateful_monadic_collection.rs:116`). `EffectLog` is an append/chronological log, not a reorderable multiset.
  - *Error short-circuit:* the first item error in iteration order halts the fold (`monadic_collection.rs:79`; `stateful_monadic_collection.rs:118-128`).
  - *Stateful threads state across siblings:* `acc_state`/`acc_context` are propagated item-to-item (`stateful_monadic_collection.rs:99,113,131`).

  **Consequence.** "Parallel evaluation is sound" holds only for the value, on the stateless error-free path, with logs treated as a bag. A2/B1, Part 2 "Aggregation is a multiset operation," and Part 3 item 3 (parallelism) must carry this scope, not the unqualified claim.

  **Determinism ≠ commutativity (recorded to prevent conflation).** `HashMap`-backed collections iterate in non-deterministic order, so logs, the surfaced error, and stateful results are non-reproducible across runs. A deterministic ordering (e.g. a `Sortable` accessor keyed on `Identifiable::id()`) would fix *reproducibility* but does **not** make order semantically irrelevant. Sorting buys determinism, not commutativity; do not present one as the other.

  **Formalization status (2026-07-07).** The ruling is DECIDED but **not yet formalized in Lean** — there is no `Core/Collection.lean`, because it is blocked on the generic-monoid gaps: the num crate still exposes only `AddMonoid`/`MulMonoid` (numeric, baked into `Add`/`Mul` + `Zero`/`One`); there is no carrier-and-operation-generic `Monoid`/`CommutativeMonoid`/`Idempotent`/`BoundedSemilattice` (gaps **A1/A2/A3 remain OPEN**), and `Foldable` still has only a seeded `fold`, no `fold_map` (gap **B1 OPEN**). When A1/A2/B1 land, the scoped theorem should be formalized as a commutative-monoid (`fold_map`) order-invariance proof over the verdict carrier, mirroring the `haft`/`core` law style.

### 2 — Graph form: which algebra, and is reconvergence defined?
- **Assumption.** The `CausaloidGraph` form has a compositional algebra: it is acyclic, reconvergent nodes (joins) combine their incoming effects under a defined operator, and the whole interprets into the effect category as a single morphism (the formalization's "Graph form" / Arrow closure).
- **Depends-on / Affects.** B3, B4; Part 2 "Graph form"; Part 3 item 4 (Arrow closure); #11a (is `RelayTo` a further control construct `F` must name?).
- **Confounder / bias.** *Elegance bias.* The categorical reading (an Arrow / monoidal interpretation in topological order) was assumed because the examples are chains and trees, where it happens to hold. The graph form's actual semantics — in particular what a *join* means — was never decided; the notation hides it.
- **Status.** INVESTIGATING
- **Findings (from code, 2026-06-26).** The original "Arrow / topological-fold" reading is **rejected**: the engine does not interpret the graph compositionally and has no join semantics.
  - *No join.* `evaluate_subgraph_from_cause` is a hand-written BFS that broadcasts each node's whole output (clone) to its **unvisited** children (`traits/causable_graph/graph_reasoning/mod.rs:177-183`). At any reconvergent node the first parent in BFS order supplies the input; every other parent's effect is **silently dropped**. The acyclic diamond `A→{B,C}→D` evaluates `D` on `B`'s output only; `C`'s contribution is lost. So even the simplest join — two causes determining one effect, the basic reason to have a graph — is mis-evaluated.
  - *Result is "last node," not a sink combination.* The return value is the last node popped in BFS order (`graph_reasoning/mod.rs:116,189`), not a structured aggregation; it depends on node-index / edge-insertion order.
  - *Acyclicity is not enforced by default.* `add_edge` performs no cycle check (`types/causal_types/causaloid_graph/causable_graph.rs:112-117`) and `freeze()` accepts cyclic graphs; "DAG" is documented intent only (`causaloid/mod.rs:53`). BFS tolerates a cycle via the `visited` guard but assigns it no meaning. **Mitigated (opt-in):** `CausableGraph::freeze_dag()` now enforces acyclicity at the freeze boundary — see Q3.
  - *`RelayTo` is dynamic control flow outside any static diagram.* A causaloid may return `EffectValue::RelayTo(target, inner)`, which clears `visited`/queue and jumps (`graph_reasoning/mod.rs:143-166`) — a computed `goto`, not composition. It can **fail to terminate** (each relay does `visited.fill(false)` with no relay bound), and a static cycle check cannot see relay loops because the target is decided at runtime.
  - *Graph bypasses `evaluate`.* A `Graph` causaloid is not run through the base `evaluate` (it returns "use specialized graph APIs"); the graph is reached only by calling the reasoning methods explicitly (the split-brain of #10).
- **Resolves when (the live question, reframed).**
  - Q1. **Decide join/reconvergence semantics first.** When node `D` has parents `B` and `C`, does `D` see a *merge* of their effects, and under what operator (the collection `AggregateLogic`? a user-supplied join? value-only vs log/state merge)? No algebra for the graph is definable until this is decided. Today it is decided by accident (BFS first-parent-wins).
  - Q2. **Identify the right abstraction.** Arbitrary causal graphs with copy and merge are not the free arrow generated by `compose`/`split`/`fanout`; the natural target is a dataflow / PROP / symmetric-monoidal category with explicit copy (Δ) and merge (∇) generators, over the effect monad. Confirm or replace this target. **[SUBSTRATE LANDED]** The symmetric-monoidal PROP generators — copy comonoid `Δ`/`ε`, merge monoid `∇`/`η`, symmetry `σ` — are implemented and law-checked in `deep_causality_haft::SymMonoidal` (`src/monoidal/mod.rs`; laws in `lean/DeepCausalityFormal/Haft/SymmetricMonoidal.lean`, `haft.monoidal.{comonoid_laws, merge_monoid_laws, symmetry}`). This is the **substrate the deferred reconvergence-merge (∇) extension consumes**: `∇ = Monoid::combine` is the join operator Q1 must pick, applied at a reconvergent node. The **graph wiring** that threads `Δ`/`∇` through a `CausaloidGraph` (branch/reconverge) is **out of scope here** and remains the deferred extension; only the algebraic generators + laws are provided.
  - Q3. **Decide `RelayTo`'s status** (a further control construct `F` must name, or out of scope) and add a relay-termination bound. **[DONE — structural hygiene]** The opt-in `freeze_dag()` that enforces acyclicity at the freeze boundary is **implemented** (`CausableGraph::freeze_dag`, additive/non-breaking; see [`freeze-dag-optin.md`](../archive/freeze-dag-optin.md)). This closes the "acyclicity is not enforced" finding *as an opt-in* — the default `freeze()` still accepts cyclic graphs. It is orthogonal to Q1/Q2 and to the rest of Q3: a DAG can still reconverge, and a static cycle check cannot see `RelayTo` loops, so **`RelayTo` status + a relay-termination bound remain open.**
- **Justification gate (purpose of this entry).** Making the graph form a rigorous algebra requires deciding Q1 and then a real rewrite (topological fold with the chosen merge generator; `RelayTo` semantics). That rewrite is **justified only if** a formal graph algebra is a goal worth the blast radius (it overlaps #10). It is independently justified on correctness grounds regardless of the formalization, because the current silent join-drop is a latent bug. Record the go/no-go decision here.
- **Decision.** _(pending Q1 — join semantics — and the go/no-go on a graph-form rewrite)_

### 3 — Reified free Arrow: uniform *and* statically typed
- **Assumption.** The free Arrow can be a single storable/generatable `ArrowTerm` type **and** have its `In`/`Out` wiring rejected by the type system at compile time.
- **Depends-on / Affects.** B3; Part 3 item 6 (syntactic rewriting) and item 8 ("type system rejects every nonsensical graph").
- **Confounder / bias.** *Borrowing from GADT languages.* The categorical literature assumes Haskell/Agda GADTs, which make this free. Rust has no GADTs, so a uniform term type and static type-indexing are mutually exclusive; the note never confronts it.
- **Status.** OPEN
- **Resolves when.**
  - Q1. Which property is load-bearing for *generation*: one storable term type, or compile-time wiring safety? (Generation needs the uniform type ⇒ wiring safety becomes a runtime check.)
  - Q2. Can a typed builder (static `In`/`Out`) lower into an untyped core AST (runtime tags) — typed construction, erased storage? Prototype `Compose<First<…>>` at depth ~5 and measure monomorphisation/compile cost to confirm the non-uniform path is untenable at scale.
  - Q3. Reword Part 3.8 to "well-typed **by construction at build time**, executed from an erased core," not "the type system rejects every nonsensical graph."
- **Decision.** _(blank)_

### 4 — Generation over a closed, decidable atom set
- **Assumption.** Well-formed causaloids are exactly the terms a finite signature builds, and the search can quotient by semantic equivalence.
- **Depends-on / Affects.** Part 3 item 8 (algorithmic generation — the headline payoff).
- **Confounder / bias.** *Notation laundering.* `Atom(CausalFn<I,O>)` reads as a finite/symbolic generator set, but `CausalFn = fn(I) -> PropagatingEffect<O>` is opaque and infinite; function equality is undecidable, so "quotient by equivalence" is uncomputable at the atom level.
- **Status.** OPEN
- **Resolves when.**
  - Q1. Will the learner compose from a **registered atom library** (finite, named, declared properties) or synthesise atom bodies? Only the former makes the claim true.
  - Q2. Is equivalence enforced only at the **combinator** layer (Arrow/monoid laws over a fixed atom alphabet), explicitly not at atom-body level? State that boundary.
  - Q3. What metadata must an atom declare to participate soundly (purity, idempotence, type signature)?
- **Decision.** _(blank)_

---

## Caveat assumptions

### 5 — Collection output is a verdict, not arbitrary `O`
- **Assumption.** `Coll(List<X>, AggregateLogic)` is well-typed for any child output type `O`.
- **Depends-on / Affects.** A2, B1; Part 2 signature functor `F`.
- **Confounder / bias.** *Functor notation hides a constraint.* Aggregation reduces `Vec<EffectValue<O>>` to bool/prob (`monadic_collection_utils.rs`); `None` needs a **complement**. So `O` must coerce to a verdict and the carrier is at least a bounded lattice / MV-algebra, not a bare monoid.
- **Status.** OPEN
- **Resolves when.**
  - Q1. What exact trait must `O` satisfy for `All/Any/None/Some` (e.g. `Into<bool>` / a `Verdict` trait / `[0,1]`)? Name it; make Collection require it instead of "any `O`."
  - Q2. Is the carrier a Boolean algebra, a probability MV-algebra, or both behind one enum? Pin the class that supports complement.
- **Decision.** _(blank)_

### 6 — A single fixed semantic carrier
- **Assumption.** `evaluate` is *the* unique interpretation functor (absolute uniqueness).
- **Depends-on / Affects.** B5; Part 2 "core property"; Part 3 item 2 (uniqueness theorem).
- **Confounder / bias.** *Absolute-uniqueness overreach.* The A2 table already gives two carriers (boolean, probabilistic) with two code paths — two functors. Uniqueness is relative to a fixed carrier.
- **Status.** OPEN — but the **premise is now discharged.** The uniqueness-of-interpreter argument needs the codomain (the Kleisli category of `PropagatingEffect`) to be a *lawful* category; that lawfulness is now machine-checked (`core.causal_arrow.category_laws` + `core.causal_monad.lawful`, see #7). What remains OPEN is uniqueness itself: there is no interpreter formalized (B5 not landed — no `Core/{Causaloid,GraphReasoning}.lean`, no Rust `Category`/`Kleisli`/`NaturalTransformation`), so "unique interpretation functor" is unproven, and the two-carrier point still stands (uniqueness is per fixed carrier).
- **Resolves when.**
  - Q1. Are deterministic and probabilistic evaluation two interpreters over one syntax, or one parameterised interpreter? Name them.
  - Q2. Restate uniqueness as "unique **per fixed semantic algebra**," with carrier choice an explicit premise.
- **Decision.** _(open — the lawful-Kleisli premise is discharged (#7); the interpreter and its uniqueness are not yet formalized)_

### 7 — `PropagatingEffect` laws hold unconditionally
- **Assumption.** The Kleisli target is a lawful category/monad, so the universal property applies cleanly.
- **Depends-on / Affects.** B4, B5; Part 2 uniqueness; Part 3 item 5 (law-based tests).
- **Confounder / bias.** *Inherited optimism.* `CausalMonadProptest` showed right-identity holds only conditionally (`error.is_some() ⇒ value == None`). The uniqueness argument silently assumes unconditional laws.
- **Status.** DECIDED 2026-07-07 — **resolved by formalization.** The historical conditional was an artifact of the deleted `EffectValue` carrier; the current `outcome: Result<CausalEffect, E>` carrier makes the laws unconditional, and this is now machine-checked in Lean.
- **Resolves when.**
  - Q1. Do the monad laws hold unconditionally or under the documented side condition? **Answered: unconditionally.** `Core/CausalMonad.lean` proves `core.causal_monad.right_id` = "`m >>= pure = m`, unconditional — holds on errored carriers", together with `left_id`, `assoc`, `left_zero`, and `core.causal_monad.lawful` (left/right identity + associativity co-hold on one carrier; the old "P1" conflict resolved). Witnessed at `deep_causality_core/tests/…/causal_monad_tests.rs` (Test ✓) and Kani (`kani_proofs.rs`, ✓). The Kleisli-category version threading state/context over arbitrary `S, C` is also proved (`core.causal_arrow.category_laws`, `core.causal_arrow.left_zero`).
  - Q2. Is the side condition an invariant, or gone? **Answered: gone.** The `Result<CausalEffect, E>` carrier removes the `error ⇒ value = None` coupling entirely (value and error are one channel), so there is no residual precondition to maintain — the F-1 caveat is closed by the carrier, not by an invariant.
- **Decision.** DECIDED 2026-07-07. The Kleisli target of the interpretation (the premise B4/B5 and #6 invoke) is a **lawful monad/category, unconditionally**, machine-checked. — Lean: `DeepCausalityFormal/Core/{CausalMonad,CausalArrow}.lean`; witnesses: `deep_causality_core/tests/{formalization_lean/causal_monad_tests.rs, types/causal_arrow/causal_arrow_tests.rs, kani_proofs.rs}`; map: `lean/THEOREM_MAP.md` (`core.causal_monad.*`, `core.causal_arrow.*`).

### 8 — "Free Arrow" = "Arrow modulo laws"
- **Assumption.** The same object serves both the universal property (uniqueness) and the rewrite/normal-form system.
- **Depends-on / Affects.** B3; Part 2 syntax; Part 3 items 6 & 8.
- **Confounder / bias.** *Syntax/quotient conflation.* Uniqueness-of-interpretation needs the **free** object (no relations); normal forms need the **quotient by Arrow laws**. Different algebras; the note names both "free Arrow."
- **Status.** OPEN — the **law-side is formalized**, the free-vs-quotient distinction is not. The Arrow laws (`haft.arrow.{category_laws, arr_functor, strength_laws, derived_combinators}`) and the free-monad laws (`haft.free_monad.{left_id, right_id, assoc, lift_bind, map_id}`) are machine-checked in Lean, so "semantics respects the Arrow laws / the fold laws" (the `T/≈` soundness side) has its underlying equations proved. What remains OPEN: there is no reified free Arrow at all (B3 not landed — the arrow module is concrete combinators, no `ArrowTerm`), so the free term algebra `T` vs its quotient `T/≈` cannot yet be separated in code, and the interpreter that must factor through `T/≈` does not exist (B5).
- **Resolves when.**
  - Q1. Separate the free term algebra `T` (universal property) from `T/≈` under the Arrow laws (normalization). Confirm the interpreter factors through `T/≈` (semantics respects the laws) — the soundness obligation for rewriting, stated as a tested law.
- **Decision.** _(open — Arrow/free-monad equations proved in Lean; the reified free Arrow and the factoring interpreter are not built)_

### 9 — Nesting is well-founded (μX, not νX)
- **Assumption.** `Causaloid ≅ μX.F(X)` is a finite, acyclic inductive term.
- **Depends-on / Affects.** B2; Part 2 "Signature functor and carrier."
- **Confounder / bias.** *ADT-as-initial-algebra reflex.* True for plain ADTs, but `Arc<…Self…>` admits shared/cyclic nesting → potentially the greatest fixpoint (νX, coinductive).
- **Status.** OPEN
- **Resolves when.**
  - Q1. Can a `Causaloid` transitively contain itself via `Arc`? Any construction guard? Decide: forbid (validate acyclic *nesting*, distinct from graph-edge acyclicity in #2) or embrace coinduction with a non-termination story.
  - Q2. Is `Arc` sharing semantic, or pure memoization? If only memoization, document "tree semantics, DAG representation."
- **Decision.** _(blank)_

### 10 — The refactor is behaviour-preserving
- **Assumption.** Folding `collection_reasoning`/`graph_reasoning` into one catamorphism only removes the split-brain, with no behaviour change.
- **Depends-on / Affects.** B2; Part 3 item 1 (one total `evaluate`); the Part-1 scope/risk claim.
- **Confounder / bias.** *Author optimism toward a clean rewrite.* Assumes the existing specialised evaluators do nothing extra (mid-graph context writes, short-circuit, caching, bespoke error/log policy).
- **Status.** OPEN
- **Resolves when.**
  - Q1. Diff existing collection/graph evaluation semantics against the proposed three-clause algebra: any short-circuit, ordering, context-write, or error-merge behaviour not captured by the fold?
  - Q2. Build a characterization-test corpus from current behaviour *before* refactoring; require the fold to pass it. Any divergence is a deliberate, documented change.
- **Decision.** _(blank)_

### 11a — `CausaloidType` is closed at three forms
- **Assumption.** The signature functor has exactly three constructors (`Singleton`/`Collection`/`Graph`); the taxonomy is complete.
- **Depends-on / Affects.** B2; Part 2 `F`; Part 3 item 2 (uniqueness re-derivation on change).
- **Confounder / bias.** *Present-state-as-complete.* `F` has three arms because today's enum does.
- **Status.** OPEN — with concrete evidence that `F` is **not** closed at three forms. The reconvergence-merge investigation (#2) establishes that multi-parent fan-in needs a **symmetric-monoidal generator** (copy Δ + merge ∇) that the current `Atom + Coll + Graph` signature does not provide — a genuine extension of the causaloid, not expressible under the single-input definition (`Core/CausalArrow.lean` stage is `A → …`, single input). So a fourth arm / a PROP generator layer is already anticipated, and any uniqueness re-derivation must be designed for that extension.
- **Resolves when.**
  - Q1. Is a 4th form plausible (probabilistic mixture, temporal/recursive, contextual switch)? Does adding one reopen the uniqueness proof, or is `F` designed for extension? **Partial: the reconvergence merge (∇) is a confirmed extension point (see #2); `F` must be designed for extension.**
- **Decision.** _(open — `F` must be extensible; the ∇ merge is the first confirmed new generator (#2))_

### 11b — Contextual atoms see a stable context per pass
- **Assumption.** Context is read-only and stable for the duration of one reasoning pass, so atoms behave as pure functions of their input.
- **Depends-on / Affects.** Part 2 atoms-as-generators; Part 3 items 3, 6, 8 (parallelism, rewriting, generation all need purity).
- **Confounder / bias.** *Effect shape hides context.* `evaluate`'s `Effect → Effect` shape conceals that `ContextualCausalFn` reads external `C`; atoms aren't pure functions of `I`. This sits awkwardly against the project's "dynamic context" framing.
- **Status.** OPEN
- **Resolves when.**
  - Q1. Is context guaranteed immutable for one reasoning pass? Can two siblings observe different context snapshots?
  - Q2. If context can change mid-pass, weaken the purity-dependent claims (rewriting / parallelism / generation) accordingly.
- **Decision.** _(blank)_

---

## Cross-cutting biases to guard against

- **Elegance bias.** The categorical story is attractive, so its preconditions get assumed rather than checked. Every "by the universal property…" must name the precondition it invokes.
- **Notation laundering.** Writing `F(X) = Atom + Coll + Graph` makes typing, purity, and effect side-conditions invisible on the page. Expand the functor with its real constraints before trusting it.
- **Author-as-reviewer.** This tracker was authored by the same party that wrote the formalization; assumptions 1, 2, 3, 8, 10 are exactly where motivated reasoning is most likely. Prefer an independent check on those.

## Formalization status snapshot (2026-07-07)

What the Lean formalization (`lean/DeepCausalityFormal/`, `lean/THEOREM_MAP.md`) has already proved, and which gap/assumption each discharges. All ids below are `proved` with a Rust witness.

**Proved (bears on this tracker):**
- **Causal monad, unconditional.** `core.causal_monad.{left_id, right_id, assoc, left_zero, lawful}` — `right_id` unconditional on errored carriers; all laws co-hold on the one `Result<CausalEffect, E>` carrier. → **discharges #7** (DECIDED) and the lawful-codomain **premise of #6**.
- **Kleisli category of the causal monad.** `core.causal_arrow.{category_laws, left_zero}` — identity/associativity/left-zero threading state+context over arbitrary `S, C`. → the semantic target of B4 is a proved lawful category, even though the *code* `Category`/`Kleisli` type does not exist.
- **Value channel.** `core.causal_effect.into_value` — value functor = `Option` (`haft.functor.laws`); honest `Maybe` projection. → grounds the `Atom`/value channel of `F`.
- **Arrow + free-monad laws (haft).** `haft.arrow.{category_laws, arr_functor, strength_laws, derived_combinators}`, `haft.free_monad.{left_id, right_id, assoc, lift_bind, map_id}`, `haft.foldable.pure_compat`, `haft.traversable.{identity, naturality, composition}`, `haft.monad.{laws, applicative_coherence}`. → the **law-side** of B3/#8 (semantics respects Arrow/fold laws) is proved.

**Not formalized / not landed in code (still OPEN):**
- **A1/A2/A3** — no carrier-and-operation-generic `Monoid`/`CommutativeMonoid`/`Idempotent`/`BoundedSemilattice` in `deep_causality_num` (only `AddMonoid`/`MulMonoid`; `Associative`/`Commutative` are bare markers). Blocks #1's Lean proof and the Collection algebra.
- **B1** — `Foldable` has only seeded `fold`, no `fold_map`.
- **B2** — no `evaluate`-as-catamorphism; no `Core/{Causaloid, Collection, GraphReasoning}.lean`.
- **B3** — no reified free Arrow / `ArrowTerm`; the arrow module is concrete combinators (`compose/split/fanout/first/second/id/lift`).
- **B4** — no Rust `Category`/`Kleisli` type (only doc-comment mentions); the *laws* are proved (above) but the *packaging* is absent.
- **B5** — no one-way `NaturalTransformation`/interpreter (only bidirectional `NaturalIso`).
- **#2 core (∇ merge)** — the reconvergence merge / symmetric-monoidal PROP (Δ/∇ over the effect monad) is not defined or formalized; the engine fails loudly at multi-parent fan-in (Path A, 2026-07-07).

**Net:** the *causal-monad / Kleisli-category / arrow-law substrate* the causaloid formalization sits on is proved and unconditional; the *causaloid-specific* structures (generic monoid tower, `fold_map`, reified free Arrow, named Kleisli/Category, one-way interpreter, the ∇ merge) are all still to build. The tracker's load-bearing gaps for the causaloid (#2 ∇, #3 free Arrow, #4 generation) remain OPEN; the caveats whose premise was the substrate's lawfulness (#7 fully, #6/#8 partly) are discharged.

## Resolution log

_(Append one line per state change: `YYYY-MM-DD #N OPEN→DECIDED — <ruling> — <commit/test/spec>`.)_

2026-06-26 #1 OPEN→DECIDED — unqualified order-independence rejected; scoped to value-only / stateless / all-success / up-to-log-permutation; stateful path and first-error short-circuit are order-sensitive; determinism ≠ commutativity — code: `utils/monadic_collection_utils.rs`, `traits/causable_collection/collection_reasoning/{monadic_collection,stateful_monadic_collection}.rs`

2026-06-26 #2 OPEN→INVESTIGATING — Arrow / topological-fold reading rejected (engine is BFS-broadcast with no join; reconvergent nodes silently drop all but the first parent; `RelayTo` is dynamic and may not terminate; acyclicity unenforced); reframed to "decide join/reconvergence semantics, then pick a copy/merge dataflow abstraction"; graph-form rewrite gated on justification — code: `traits/causable_graph/graph_reasoning/mod.rs`, `types/causal_types/causaloid_graph/causable_graph.rs`

2026-06-26 #2 Q3 partial — opt-in `CausableGraph::freeze_dag()` implemented (enforces acyclicity at the freeze boundary; additive/non-breaking; default `freeze()` unchanged). Closes the acyclicity finding as opt-in. STILL OPEN: join/reconvergence semantics (Q1), the categorical abstraction (Q2), `RelayTo` status + relay-termination bound (Q3). — code: `traits/causable_graph/graph/mod.rs`, tests `tests/types/causal_types/causaloid_graph/causality_graph_freeze_tests.rs`; note: `freeze-dag-optin.md`

2026-07-07 #2 partial — the `comonoid-graph-join` change replaces first-parent-wins BFS with principled **wire-slot topological sequencing** (reachability pre-pass + ascending-node-index canonical schedule + acyclicity gate; `RelayTo` as sequential round composition), so the silent-parent-drop bug is fixed for single-input/linear/tree graphs (bit-identical) and a **multi-parent reconvergence now fails LOUDLY** rather than silently selecting one parent. This is Path A: it does NOT decide the merge. **STILL OPEN (the load-bearing core of #2): Q1 the reconvergence merge `∇` semantics and Q2 its categorical target — the free symmetric-monoidal / PROP category with copy (Δ) and merge (∇) generators over the effect monad, an extension of the single-input causaloid.** An earlier iteration that put a per-causaloid `join_fn` keyed by parent *node index* was rejected and removed: it leaked graph sequencing/position into the single-input causal function (the spacetime-agnostic property makes parent/child/before-after undefined at that layer). `∇` + per-connection asymmetry (Hardy's Λ on edges) are deferred to a dedicated symmetric-monoidal extension change. — code: `traits/causable_graph/graph_reasoning/{mod,stateful}.rs`; specs: `openspec/changes/comonoid-graph-join/`

2026-07-07 #7 OPEN→DECIDED — resolved by formalization: the causal-monad laws are machine-checked **unconditional** on the `Result<CausalEffect, E>` carrier (`core.causal_monad.{right_id (unconditional), left_id, assoc, left_zero, lawful}`), and the Kleisli-category version threading state/context is proved (`core.causal_arrow.{category_laws, left_zero}`). The historical `CausalMonadProptest` conditional (`error ⇒ value = None`) was an `EffectValue`-carrier artifact, closed by the new carrier. — Lean: `DeepCausalityFormal/Core/{CausalMonad,CausalArrow}.lean`; witnesses: `deep_causality_core/tests/{formalization_lean/causal_monad_tests.rs, types/causal_arrow/causal_arrow_tests.rs, kani_proofs.rs}`; map: `lean/THEOREM_MAP.md`

2026-07-07 formalization-state audit — verified against `lean/`, `deep_causality_num/src/algebra/`, `deep_causality_haft/src/`. Proved substrate: causal-monad laws (unconditional), Kleisli-category laws, `Option` value functor, and the haft Arrow + free-monad laws. Confirmed NOT landed (gaps remain OPEN): A1/A2/A3 generic `Monoid`/`CommutativeMonoid`/`Idempotent` (num still `AddMonoid`/`MulMonoid` only), B1 `Foldable::fold_map`, B2 `evaluate`-as-catamorphism (no `Core/Collection.lean` or `Core/Causaloid.lean`), B3 reified free Arrow (`ArrowTerm`), B4 Rust `Category`/`Kleisli` type (laws proved, type absent), B5 one-way interpreter/`NaturalTransformation`. Discharged: #7 fully; #6 and #8 premises partially. Added the "Formalization status snapshot" section above.
