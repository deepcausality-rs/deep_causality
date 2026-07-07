## Why

The Lean formalization (`lean/DeepCausalityFormal/`) proves `Num`, `Haft`, and the `Core` causal-monad / Kleisli-arrow / free-monad laws — all of `deep_causality_core`. The **main `deep_causality` crate** — the Causaloid, the graph-reasoning engine, the Context hypergraph, and Collection — is **not yet formalized**. That engine is the crown jewel and the one genuinely hard-to-verify part of the system. Formalizing it now, while the architecture is fresh from the `CausalEffect` free-monad refactor, is the natural next layer.

Two downstream capabilities must be *reachable* from the formalization even though their implementation is deferred: Pearl's **do-operator** (causal calculus) and **quantum causal models (QCM)** hosted on the arity-5 causal monad. The formalization is therefore architected so both are expressible as operations/predicates over the same reified structure, not bolted on later.

## What Changes

- Formalize the **singleton causaloid** as a context-parameterized Kleisli arrow `I → CausalEffect<O>`, reconciling the stale `Causaloid-Formalization.md` Part II to the current `CausalEffect` model (the input-command-errors behaviour is the F-3 resolution; verify the F-1 `error ⇒ value=None` caveat is now closed by the `Result<CausalEffect, _>` carrier).
- Formalize the **Collection causaloid** as a commutative-monoid fold over a verdict carrier — `AggregateLogic {All, Any, None, Some(k)}` order-invariance.
- Formalize the **graph-reasoning engine as a `Free::fold` catamorphism** over the canonical topological linearization, with reconvergent sharing carried by the keyed valuation (the let-environment), not by subterm duplication — a tree-shaped `Free` cannot carry reconvergence (`bind` clones the continuation per hole). The fan-in itself is **front-loaded into the prerequisite change `comonoid-graph-join`, which lands first**: labeled wire-slot resolution (`Fired`/`Inactive`), per-node join mechanisms over parent-indexed effects, and the `unique_valuation` + `schedule_invariance` theorems. This change composes with those results; the copy law is stated as a law of the **classical interpreter only** (no-cloning compatibility — verified against the `ctx/papers/` QCM sources).
- Formalize the **Context hypergraph** with parent-set (hyperedge) semantics — the shared substrate both do()-surgery and QCM-factorization require.
- Formalize the **do-operator mechanism** two ways: graph **surgery** on the causal hypergraph (JKZ cut-wires / Lorenz–Tull "opening") and an alternate **handler/algebra** over the `RelayTo` `Free` program; prove intervention commutes with encapsulation. The surgery primitive is the one defined on the shared substrate in `comonoid-graph-join` D10 — **wire-key deletion + mechanism pinning** — so single-edge cuts (`sever P₁ → X, keep P₂`) are expressible and the *same* surgery is respected by both the classical evaluator and the deferred QCM assembler. Acyclicity of the surgical result is enforceable via the existing `ultragraph::has_cycle` freeze gate.
- Formalize the **QCM predicate/obligation layer** (`CJOp`, `NoInfluence`/`DirectCause`/`Pa`, `IsMarkov = Factorizes ∧ PairwiseCommute`, `ValidProcess`, `Compatible → Markov`, and the open `traceOut_preserves_commute` obligation). Hard proofs and the Rust implementation are **deferred**; the deliverable is that every QCM predicate is *stated over the crate's structures*, with the arity-5 state channel kept generic so operator-valued state inherits the monad laws for free. Acyclicity is a **relaxable parameter** so cyclic QCMs (quantum switch / indefinite causal order) reuse the same apparatus.
- Every new theorem is bound to a **Rust witness** (`tests/formalization_lean/`) and a **`THEOREM_MAP.md`** row; each Lean file typechecks standalone with bare `lean`; `bazel test //...` stays green.

## Capabilities

### New Capabilities
- `causaloid-formalization`: singleton = context-parameterized Kleisli arrow into `CausalEffect`; collection = commutative-monoid fold over a verdict carrier (order-invariance).
- `graph-reasoning-formalization`: the reasoning engine as a `Free::fold` catamorphism over the canonical linearization with keyed-valuation sharing; fan-in composed from the `comonoid-graph-join` theorems (`unique_valuation`, `schedule_invariance`); the copy/discard laws scoped to the classical interpreter (interpreter-neutral substrate).
- `context-hypergraph-formalization`: the contextoid hypergraph with parent-set (hyperedge) semantics; acyclicity as a freeze-enforceable, relaxable parameter.
- `intervention-do-operator`: Pearl `do(X=x)` as graph surgery + handler over the `Free` program; intervention-commutes-with-encapsulation.
- `quantum-causal-model-support`: the QCM predicate/obligation layer (CJ operators, no-influence, Markov = factorize + pairwise-commute, valid-process, compatibility, partial-trace obligations) stated over the crate structures; implementation deferred.

### Modified Capabilities
<!-- No existing spec's REQUIREMENTS change. `core-formalization`, `control-channel`, `lawful-effect-channel`, and `stateful-causal-arrow` are the proven foundation this change builds ON (dependencies), not requirement changes. The Rust graph-engine join alignment is captured as a gated task in design/tasks, not a spec-level modification here. -->

## Impact

- **Depends on `comonoid-graph-join`** (front-loaded prerequisite): the Rust engine must implement labeled fan-in (wire-slot resolution, per-node join mechanisms) with the `unique_valuation`/`schedule_invariance` theorems proved before this change formalizes the engine. That change also owns the reconvergent-graph test updates and defines the do-surgery attachment point (D10) this change's intervention capability builds on.
- **New Lean modules:** `Core/{Causaloid, Collection, GraphReasoning, ContextGraph, Intervention}.lean`, `Quantum/{CJOp, QCM}.lean`; registered in `DeepCausalityFormal.lean`; new `THEOREM_MAP.md` rows.
- **New Rust witnesses:** `deep_causality/tests/formalization_lean/*` and/or `deep_causality_core/tests/formalization_lean/*` binding each theorem.
- **Docs reconciled (flagged, not silently rewritten):** `openspec/notes/causal-algebra/{Causaloid-Formalization.md, CausalMonadProptest.md}` updated from `EffectValue` to the `CausalEffect` model; `LEAN_CORE.md` / `Formalization.md` work-plan items #2/#11 (graph join) and #12 (RelayTo handler) advanced.
- **Engine fan-in:** handled by the prerequisite `comonoid-graph-join` change (wire-slot resolution + labeled join mechanisms), so this change describes real engine behaviour — no gap. `∇_G` as a standalone join operation is dissolved: fan-in is the labeled parent product consumed by the node's own mechanism.
- **No external dependencies.** Lean uses Mathlib (already a project dependency); the Rust side adds only test files. `unsafe_code = "forbid"` and the zero-macro/zero-dep ethos are untouched.
- **QCM Rust implementation and the hard Layer-D proof are explicitly out of scope** (deferred to a subsequent change).
