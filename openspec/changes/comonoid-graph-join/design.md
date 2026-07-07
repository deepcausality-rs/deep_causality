## Context

`evaluate_subgraph_from_cause` (and its stateful twin) is a BFS that marks a child visited when the *first* parent enqueues it (`graph_reasoning/mod.rs:199-203`), passing that one parent's `result_effect`. A reconvergence node therefore consumes an arbitrary single parent (BFS arrival order) and ignores the rest. The suite already contains such a graph (`causality_graph_reasoning_tests.rs` `logic_graph`: node `idx2` has parents `idx1` and `idx4`). This is the undecided `∇_G` join (`Formalization.md` #2/#11), and it is foundational to the do-operator and QCM formalizations, so it is front-loaded here as a paired Lean-model + Rust-implementation unit. The graph backend (`ultragraph`) now exposes `has_cycle`/`find_cycle`/`freeze`, giving both an acyclicity gate and a topological order.

## Goals / Non-Goals

**Goals:**
- Replace first-parent-wins with a principled fan-in join over the frozen acyclic graph, keeping the `Causaloid` input API unchanged (engine combines parents → one effect).
- Prove, in Lean bound to a Rust witness, that the whole-graph fold is invariant under the choice of topological linearization (reconvergence is deterministic).
- Leave linear/tree graphs bit-identical; change only reconvergent graphs.

**Non-Goals:**
- The cyclic-graph join (indefinite causal order) — deferred to the QCM change; acyclicity is a required precondition here.
- Any `Causaloid` signature change (product-input route was explicitly rejected).
- The rest of the main-crate formalization (`formalize-main-crate`) — this change is its prerequisite.

## Decisions

**D1 — Engine combines → one value (chosen route).** At a fan-in the engine reduces parent effects to a single `PropagatingEffect` via a declared associative-commutative join-combine, then evaluates the node normally. Rationale: keeps the causaloid API stable (smallest blast radius) while still satisfying the comonoid/Markov laws; the fan-in monoid is the copy/discard comonoid's partner. *Alternative rejected:* node `causal_fn` takes a product `⊗(parents)` — categorically purest but a breaking signature change across every graph example.

**D2 — Topological (Kahn) evaluation over the frozen DAG.** A node is scheduled when its in-degree of processed parents equals its total in-degree; parent effects are buffered until then. Rationale: correctness requires all parents before the join; `ultragraph` already supplies acyclicity + topological order. *Alternative rejected:* keep BFS with a re-visit/merge pass — more state, easy to get the join order wrong.

**D3 — Channel-combine rules.** value: the declared commutative-associative monoid (reuses the collection order-invariance result); log: deterministic canonical-order (node-index) concatenation (the log monoid is free/non-commutative, so a canonical order makes the result deterministic); error: short-circuit / left-zero (first error by canonical parent order); state/context: declared combine (identity/trivial for the stateless engine `S = C = ()`). Rationale: each channel keeps its established algebra; only fan-in ordering is newly pinned.

**D4 — Where the join-combine is declared.** Settle in task 1 after the blast-radius scan: a graph-level default policy vs a per-call argument vs reuse of `AggregateLogic`. Leaning to a graph-level policy defaulting to a sensible commutative monoid, so existing single-parent graphs need no change. Recorded as an open question until the scan.

**D5 — Acyclic-only, freeze-gated.** The join requires a topological order, so the graph must be frozen and acyclic; `has_cycle` rejects cyclic graphs. The same code path is later relaxed (in the QCM change) for cyclic models — noted, not built here.

**D6 — Lean: `Core/GraphJoin.lean`.** Model the join as a commutative monoid and prove topological-order invariance by reduction to commutative-monoid fold order-invariance + the comonoid copy law. Self-contained, bare-`lean`, bound to a Rust witness that runs the real engine under two topological orders.

## Risks / Trade-offs

- **[Breaking change to reconvergent graphs]** results change from first-parent-wins to joined. → Blast radius is only multi-parent graphs; task 1 enumerates them; update the diamond tests to the join-derived expectation (not to whatever the code now emits — AGENTS.md: fix the API, then the test asserts the correct value).
- **[Choosing the wrong value-combine default]** a bad default silently changes semantics. → Make the default a well-known commutative monoid, document it, and let it be overridable; the order-invariance theorem guarantees only determinism, not that the chosen monoid is domain-correct.
- **[Stateful join underspecified]** state/context combine at fan-in is nontrivial for `S ≠ ()`. → Scope the proven order-invariance to value/log/error; for the stateful engine, either require a declared state-combine or document that multi-parent stateful joins are deferred, whichever the scan shows is actually used.
- **[Topological eval regressions]** rewriting the loop risks behavioural drift on linear graphs. → Golden-test linear/tree graphs for bit-identical output before/after; only diamonds may change.

## Migration Plan

Land before `formalize-main-crate`. Steps: (1) blast-radius scan; (2) topological engine + join in `graph_reasoning/{mod,stateful}.rs`; (3) update reconvergent tests; (4) `Core/GraphJoin.lean` + witness + `THEOREM_MAP`; (5) `bazel test //...` green. Rollback = revert the engine change; linear graphs are unaffected so the blast surface is contained.

## Open Questions

- Where is the join-combine declared (graph-level default vs per-call vs `AggregateLogic` reuse)? — settle in task 1.
- Does any current stateful multi-parent graph exist, forcing a declared state-combine now vs deferring it? — settle in the scan.
- Default value-combine monoid for the common boolean-verdict case (AND-like `All`)? — propose in task 1, confirm before implementing.
