## Why

The Lean formalization (`lean/DeepCausalityFormal/`) proves `Num`, `Haft`, and the `Core` causal-monad / Kleisli-arrow / free-monad laws — all of `deep_causality_core`. The **main `deep_causality` crate** — the Causaloid, the graph-reasoning engine, the Context hypergraph, and Collection — is **not yet formalized**. That engine is the crown jewel and the one genuinely hard-to-verify part of the system. Formalizing it now, while the architecture is fresh from the `CausalEffect` free-monad refactor, is the natural next layer.

Two downstream capabilities must be *reachable* from the formalization even though their implementation is deferred: Pearl's **do-operator** (causal calculus) and **quantum causal models (QCM)** hosted on the arity-5 causal monad. The formalization is therefore architected so both are expressible as operations/predicates over the same reified structure, not bolted on later.

## What Changes

- Formalize the **singleton causaloid** as a context-parameterized Kleisli arrow `I → CausalEffect<O>`, reconciling the stale `Causaloid-Formalization.md` Part II to the current `CausalEffect` model (the input-command-errors behaviour is the F-3 resolution; verify the F-1 `error ⇒ value=None` caveat is now closed by the `Result<CausalEffect, _>` carrier).
- Formalize the **Collection causaloid** as a commutative-monoid fold over a verdict carrier — `AggregateLogic {All, Any, None, Some(k)}` order-invariance.
- Formalize the **graph-reasoning engine as a `Free::fold` catamorphism**. The multi-node reconvergence join `∇_G` (copy/discard comonoid) is **front-loaded into a separate prerequisite change, `comonoid-graph-join`, which lands first**; this change therefore formalizes the engine over a *comonoid-correct* Rust engine (no "engine gap") rather than the old `last_propagated_effect` linear-chain behaviour.
- Formalize the **Context hypergraph** with parent-set (hyperedge) semantics — the shared substrate both do()-surgery and QCM-factorization require.
- Formalize the **do-operator mechanism** two ways: graph **surgery** on the causal hypergraph (JKZ cut-wires / Lorenz–Tull "opening") and an alternate **handler/algebra** over the `RelayTo` `Free` program; prove intervention commutes with encapsulation. Acyclicity of the surgical result is enforceable via the existing `ultragraph::has_cycle` freeze gate.
- Formalize the **QCM predicate/obligation layer** (`CJOp`, `NoInfluence`/`DirectCause`/`Pa`, `IsMarkov = Factorizes ∧ PairwiseCommute`, `ValidProcess`, `Compatible → Markov`, and the open `traceOut_preserves_commute` obligation). Hard proofs and the Rust implementation are **deferred**; the deliverable is that every QCM predicate is *stated over the crate's structures*, with the arity-5 state channel kept generic so operator-valued state inherits the monad laws for free. Acyclicity is a **relaxable parameter** so cyclic QCMs (quantum switch / indefinite causal order) reuse the same apparatus.
- Every new theorem is bound to a **Rust witness** (`tests/formalization_lean/`) and a **`THEOREM_MAP.md`** row; each Lean file typechecks standalone with bare `lean`; `bazel test //...` stays green.

## Capabilities

### New Capabilities
- `causaloid-formalization`: singleton = context-parameterized Kleisli arrow into `CausalEffect`; collection = commutative-monoid fold over a verdict carrier (order-invariance).
- `graph-reasoning-formalization`: the reasoning engine as a `Free::fold` catamorphism; the reconvergence join `∇_G` as a copy/discard comonoid (free Markov category).
- `context-hypergraph-formalization`: the contextoid hypergraph with parent-set (hyperedge) semantics; acyclicity as a freeze-enforceable, relaxable parameter.
- `intervention-do-operator`: Pearl `do(X=x)` as graph surgery + handler over the `Free` program; intervention-commutes-with-encapsulation.
- `quantum-causal-model-support`: the QCM predicate/obligation layer (CJ operators, no-influence, Markov = factorize + pairwise-commute, valid-process, compatibility, partial-trace obligations) stated over the crate structures; implementation deferred.

### Modified Capabilities
<!-- No existing spec's REQUIREMENTS change. `core-formalization`, `control-channel`, `lawful-effect-channel`, and `stateful-causal-arrow` are the proven foundation this change builds ON (dependencies), not requirement changes. The Rust graph-engine join alignment is captured as a gated task in design/tasks, not a spec-level modification here. -->

## Impact

- **Depends on `comonoid-graph-join`** (front-loaded prerequisite): the Rust engine's `∇_G` join must be comonoid-correct before this change formalizes the engine. That change also owns the reconvergent-graph test updates.
- **New Lean modules:** `Core/{Causaloid, Collection, GraphReasoning, ContextGraph, Intervention}.lean`, `Quantum/{CJOp, QCM}.lean`; registered in `DeepCausalityFormal.lean`; new `THEOREM_MAP.md` rows.
- **New Rust witnesses:** `deep_causality/tests/formalization_lean/*` and/or `deep_causality_core/tests/formalization_lean/*` binding each theorem.
- **Docs reconciled (flagged, not silently rewritten):** `openspec/notes/causal-algebra/{Causaloid-Formalization.md, CausalMonadProptest.md}` updated from `EffectValue` to the `CausalEffect` model; `LEAN_CORE.md` / `Formalization.md` work-plan items #2/#11 (graph join) and #12 (RelayTo handler) advanced.
- **Engine `∇_G` join:** handled by the prerequisite `comonoid-graph-join` change (topological eval + fan-in join), so this change describes a real comonoid join — no gap.
- **No external dependencies.** Lean uses Mathlib (already a project dependency); the Rust side adds only test files. `unsafe_code = "forbid"` and the zero-macro/zero-dep ethos are untouched.
- **QCM Rust implementation and the hard Layer-D proof are explicitly out of scope** (deferred to a subsequent change).
