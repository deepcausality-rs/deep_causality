## Context

The Lean formalization proves `Num`, `Haft` (~60 laws), and the `Core` layer of `deep_causality_core`: the arity-5 causal-monad laws (`Core/CausalMonad.lean`, generic over state `S`, context `C`, error `E`, log `Λ`), the Kleisli-arrow laws (`Core/CausalArrow.lean`), the free monad `CausalEffect = Free CausalCommand (Option V)` with `map`/`fold` (`Core/{CausalEffect,CausalCommand}.lean`), and the witness/inherent functor+applicative totality (`Core/Consistency.lean`). The `THEOREM_MAP.md` binds each theorem to a Rust witness; every core Lean file typechecks standalone with bare `lean`.

The **main `deep_causality` crate** sits above this: `Causaloid` (singleton = a Kleisli arrow into `CausalEffect`; collection; graph), the BFS **graph-reasoning engine** (`traits/causable_graph/graph_reasoning/`), the **Context** contextoid hypergraph (backed by `ultragraph`, which now exposes `has_cycle`/`freeze`), and `AggregateLogic`. None of this is in Lean yet.

Two deferred capabilities must be *reachable from* the formalization: Pearl's do-operator and quantum causal models. Both, on analysis, live on the **same** substrate — a reified causal (hyper)graph interpreted by a fold/handler, with the arity-5 state channel kept generic. The design's job is to make that substrate explicit so both are expressible without a bespoke second framework.

Source basis: `openspec/notes/causal-algebra/{Formalization.md, Causaloid-Formalization.md, deepcausality-formalization.md}`, `openspec/notes/quantum/{QCM-on-EPP.md, quantum-epp.md}`, and the three `ctx/papers/` quantum-causal-model papers (Lorenz 2022; Lorenz–Barrett 2001.07774; Barrett–Lorenz–Oreshkov 2002.12157).

## Goals / Non-Goals

**Goals:**
- Formalize, in Lean bound to Rust witnesses, the main-crate reasoning surface: singleton + collection causaloid, the reasoning engine as a `Free::fold` catamorphism, and the context hypergraph with parent-set semantics.
- Make Pearl's do-operator expressible both as graph surgery and as a handler over the `Free` program, with the intervention-commutes-with-encapsulation theorem.
- Provide the full QCM predicate/obligation layer (`CJOp`, `NoInfluence`, `IsMarkov`, `ValidProcess`, `Compatible → Markov`, `traceOut_preserves_commute`) as *stated* Lean signatures, so the deferred QCM implementation is a reconstruction, not a redesign.
- Keep the arity-5 state channel generic so operator-valued state inherits the monad laws for free.
- Preserve house style: bare-`lean` per file, `THEOREM_MAP.md` rows, Rust witnesses, `bazel test //...` green, zero new runtime deps, `unsafe_code = "forbid"`.

**Non-Goals:**
- Implementing quantum causal models in Rust (deferred to a subsequent change).
- Proving the hard Layer-D proposition (`traceOut_preserves_commute`) or the `Markov → Compatible` converse — these are stated as obligations/open hypotheses.
- Re-implementing the Rust graph engine's `∇_G` join in this change unless the witness for the comonoid-join theorem strictly requires it (see Decisions → engine gap).
- Formalizing CSM, generative types, or the model layer (out of scope; separate future work).

## Decisions

**D1 — One reified substrate for engine + do() + QCM.** Model the reasoning program as data (`CausalEffect` `Free` program) and the causal structure as a hypergraph with a parent-set map `Pa`; interpret with a fold/handler. Rationale: the `deepcausality-formalization.md` "program ≠ interpreter" insight — the same program admits the default engine, an intervention handler, and (with operator-valued state) a QCM process-builder. *Alternative rejected:* three separate models (engine, do, QCM) — triples the surface and blurs the static/dynamic boundary.

**D2 — Reconvergence join `∇_G` = copy/discard comonoid (free Markov category / cPROP).** Adopt the established comonoid rather than inventing a join; constrain it by comonoid laws (Fritz 2020; JKZ 2019/2021). Rationale: the same structure grounds both the do-calculus (JKZ surgery) and the QCM factorization (parent sets), and answers the ACT-reviewer "why not a Markov category?" by *extending* it. *Alternatives:* AggregateLogic-style monoid join (simpler, weaker literature leverage); defer join (fastest, but leaves the engine unformalized at branches). Chosen per user decision.

**D3 — do() = surgery AND handler.** Static graph surgery (cut `Pa(X)`, pin output) for the causal hypergraph; an alternate `Free::fold` algebra for dynamic `RelayTo` interventions. Prove `do` functorial w.r.t. encapsulation. Rationale: covers both the static-structure and dynamic-reasoning axes; matches JKZ (surgery) and ChiRho (handler) prior art. Chosen per user decision.

**D4 — QCM = full predicate layer + obligation slots, generic state.** Because `Core/CausalMonad.lean` is generic in `S`, operator-valued state needs no new monad proof; the QCM content is a predicate family (`CJOp`, `NoInfluence`, `Factorizes`, `PairwiseCommute`, `ValidProcess`, `Compatible`) plus two obligation slots (`traceOut_preserves_commute`, `Compatible → Markov`). `n = 2` commutativity is a derived lemma; `n ≥ 3` is an axiom (footnote-11 fact). `ValidProcess` is a separate predicate (independent of factorize+commute in the cyclic case). Chosen per user decision.

**D5 — Acyclicity is a relaxable, freeze-enforceable parameter.** The graph carries acyclicity as a separable constraint. In the acyclic regime it maps to the existing `ultragraph::has_cycle` freeze gate (and constrains do()-surgery to remain acyclic); relaxing it admits cyclic QCMs (quantum switch / indefinite causal order) with the identical factorization/commutativity apparatus. This directly uses the recently-added implementation DAG check. Chosen per user decision.

**D6 — Reconcile stale notes, do not silently rewrite.** `Causaloid-Formalization.md` and `CausalMonadProptest.md` predate the `CausalEffect` refactor (they say `EffectValue`/`ContextualLink`/`Map`). Update them to the current model with each change flagged; verify the F-1 (`error ⇒ value=None`) caveat is closed by the `Result<CausalEffect,_>` carrier.

**D7 — Layering / dependency order in Lean.** `Core/Causaloid` and `Core/Collection` (reduce to `CausalArrow`/`CausalMonad`), then `Core/GraphReasoning` (reduce to `FreeMonad` + the comonoid), then `Core/ContextGraph`, then `Core/Intervention`, then `Quantum/{CJOp,QCM}`. Each file self-contained (transcribes what it needs, house style), registered in `DeepCausalityFormal.lean`.

## Risks / Trade-offs

- **[Engine gap — resolved by prerequisite]** The comonoid `∇_G` join is delivered by the front-loaded `comonoid-graph-join` change (topological eval + fan-in join), which lands first. → This change depends on it, so it formalizes a real comonoid-correct engine; no witness divergence to manage. If `comonoid-graph-join` slips, this change's graph-reasoning group (§3) is blocked, not its causaloid/context/intervention/QCM groups.
- **[Deferred proofs masquerading as complete]** `sorry`/`axiom` in `Quantum/*` could look proved. → Every deferred item is a named `Hypothesis`/`axiom`/obligation with a `THEOREM_MAP` status of `obligation`/`open`, and CI greps for stray `sorry` outside the whitelisted obligation slots.
- **[Mathlib surface for hypergraph/comonoid/operators]** Comonoid + partial-trace/operator algebra may need nontrivial Mathlib. → Keep each file self-contained and transcribe minimal structures (as the existing core files do); prefer stating obligations over pulling heavy Mathlib where a bare-`lean` transcription suffices; only `Quantum/*` may import Mathlib operator/`Matrix` API.
- **[Scope creep from QCM]** The QCM layer is large. → Its Rust implementation and hard proofs are Non-Goals; the deliverable is *stated, typechecking* signatures + one operator-valued-state witness, nothing more.
- **[Stale-note reconciliation churn]** → Limit edits to the two named notes + `LEAN_CORE.md`/`Formalization.md` work-plan rows; flag each, no unrelated rewrites (AGENTS.md surgical-diff rule).

## Migration Plan

Additive: new Lean files + new Rust witness test files + doc updates. No public Rust API change in this change (the engine-join alignment, if undertaken, is the one behavioral item and is separately gated). Rollback = remove the new files and doc edits; the existing `bazel test //...` and bare-`lean` suites are unaffected until the new witnesses are registered.

## Open Questions

- **Engine-join implementation:** resolved — front-loaded into the prerequisite `comonoid-graph-join` change; this change depends on it.
- **Where do Rust witnesses live** for main-crate theorems — `deep_causality/tests/formalization_lean/` (new) vs `deep_causality_core/tests/formalization_lean/`? (Default: main-crate tests for main-crate theorems; core tests for the operator-valued-state monad witness.)
- **CJOp concrete carrier for the operator-valued-state witness** — reuse `deep_causality_tensor`/`multivector`, or a minimal local matrix for the test only? (Default: minimal local matrix in the witness; full carrier is deferred-implementation concern.)
