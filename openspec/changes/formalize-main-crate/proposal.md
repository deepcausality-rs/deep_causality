<!--
Scope reduced 2026-07-10 (user ruling) and reconciled against the landed changes
`haft-categorical-machinery` + `causaloid-formalization-stages-2-5`: the do-operator and QCM
capabilities are removed — `deep_causality_do_calculus` and `deep_causality_quantum` will be
spec'd as separate changes (see `causaloid-formalization-roadmap.md` §6–§7). This change now owns
only the main-crate formalization RESIDUE: doc reconciliation, the sorry-guard, the F-3
command-input theorem, collection permutation-invariance, relay-round composition, and the
context hypergraph. See tasks.md for the delivered-by markers.
-->

## Why

The Lean formalization (`lean/DeepCausalityFormal/`) proves `Num`, `Haft`, and the `Core` causal-monad / Kleisli-arrow / free-monad laws — all of `deep_causality_core`. The **main `deep_causality` crate** — the Causaloid, the graph-reasoning engine, the Context hypergraph, and Collection — is **not yet formalized**. That engine is the crown jewel and the one genuinely hard-to-verify part of the system. Formalizing it now, while the architecture is fresh from the `CausalEffect` free-monad refactor, is the natural next layer.

Two downstream capabilities must be *reachable* from the formalization even though their implementation is deferred: Pearl's **do-operator** (causal calculus) and **quantum causal models (QCM)** hosted on the arity-5 causal monad. The formalization is therefore architected so both are expressible as operations/predicates over the same reified structure, not bolted on later.

## What Changes

- Formalize the **singleton causaloid** as a context-parameterized Kleisli arrow `I → CausalEffect<O>`, reconciling the stale `Causaloid-Formalization.md` Part II to the current `CausalEffect` model (the input-command-errors behaviour is the F-3 resolution; verify the F-1 `error ⇒ value=None` caveat is now closed by the `Result<CausalEffect, _>` carrier).
- Formalize the **Collection causaloid** as a commutative-monoid fold over a verdict carrier — `AggregateLogic {All, Any, None, Some(k)}` order-invariance.
- Formalize the **graph-reasoning engine as a `Free::fold` catamorphism** over the canonical topological linearization, with reconvergent sharing carried by the keyed valuation (the let-environment), not by subterm duplication — a tree-shaped `Free` cannot carry reconvergence (`bind` clones the continuation per hole). The fan-in itself is **front-loaded into the prerequisite change `comonoid-graph-join`, which lands first**: labeled wire-slot resolution (`Fired`/`Inactive`), per-node join mechanisms over parent-indexed effects, and the `unique_valuation` + `schedule_invariance` theorems. This change composes with those results; the copy law is stated as a law of the **classical interpreter only** (no-cloning compatibility — verified against the `ctx/papers/` QCM sources).
- Formalize the **Context hypergraph** with parent-set (hyperedge) semantics — the shared substrate both do()-surgery and QCM-factorization require.
- **REMOVED FROM SCOPE (2026-07-10):** the do-operator mechanism and the QCM predicate/obligation layer — `deep_causality_do_calculus` and `deep_causality_quantum` are spec'd as separate changes.
- Every new theorem is bound to a **Rust witness** (`tests/formalization_lean/`) and a **`THEOREM_MAP.md`** row; each Lean file typechecks standalone with bare `lean`; `bazel test //...` stays green.

## Capabilities

### New Capabilities
- `causaloid-formalization`: singleton = context-parameterized Kleisli arrow into `CausalEffect`; collection = commutative-monoid fold over a verdict carrier (order-invariance).
- `graph-reasoning-formalization`: the reasoning engine as a `Free::fold` catamorphism over the canonical linearization with keyed-valuation sharing; fan-in composed from the `comonoid-graph-join` theorems (`unique_valuation`, `schedule_invariance`); the copy/discard laws scoped to the classical interpreter (interpreter-neutral substrate).
- `context-hypergraph-formalization`: the contextoid hypergraph with parent-set (hyperedge) semantics; acyclicity as a freeze-enforceable, relaxable parameter.

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
