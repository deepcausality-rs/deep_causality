## Why

The graph-reasoning engine visits each node once: the first parent to reach a node (in BFS order) enqueues it and marks it visited, so a node with **multiple parents** (a reconvergence / diamond) silently uses whichever single parent BFS happened to reach first. This is a latent **correctness bug**, not merely a missing feature — and it blocks every principled use of shared-parent structure: Pearl's do-calculus and the QCM factorization `σ = ∏_i ρ_{A_i | Pa(A_i)}` are both defined over parent *sets*. A diamond already exists in the test suite (`causality_graph_reasoning_tests.rs`: node `idx2` has parents `idx1` and `idx4`), so the behaviour is exercised today, just not principled.

The reconvergence join `∇_G` is the gating open problem (`Formalization.md` #2/#11). It is also **foundational**: the intervention (`do()`) and QCM formalizations both build on it. Front-loading it — as a paired Lean model + Rust implementation — gives the rest of the formalization a comonoid-correct substrate and removes the "engine gap" risk from `formalize-main-crate`.

## What Changes

- **BREAKING (reconvergent graphs only):** replace first-parent-wins with a principled **fan-in join**. A node fires only when **all** its parents have produced effects (topological / Kahn evaluation over the frozen acyclic graph); the engine gathers the parent effects and reduces them, via a declared **associative-commutative join-combine**, to the **one** effect the node consumes. Linear/tree graphs (single parent per node) are unaffected — a join of one is identity.
- **Causaloid API unchanged.** The engine combines parents into one effect *before* the node; a node's `causal_fn` still receives a single input (the chosen "engine-combines → one value" route), so no `Causaloid` signature change.
- **Channel-combine rules at fan-in:** value = the declared commutative-associative monoid (the comonoid's monoidal partner; reuses the collection order-invariance result); log = deterministic canonical-order (by node index) concatenation; error = short-circuit (first error by canonical parent order, a left-zero); state/context = declared combine (trivial for the stateless engine where `S = C = ()`).
- **Comonoid framing.** Fan-out stays copy (the engine already passes a node's result to each child); fan-in is the declared commutative monoid. The headline theorem: the whole-graph fold is **invariant under the choice of topological linearization** (reconvergence is deterministic) — reducing to commutative-monoid fold order-invariance + the comonoid copy law.
- **Lean model** (`Core/GraphJoin.lean`) proving the join-monoid laws + topological-order invariance, bound to a Rust witness (`tests/formalization_lean/`) and a `THEOREM_MAP.md` row; bare-`lean` typecheck.
- **Scope: acyclic (frozen DAG) only.** The cyclic-graph join (indefinite causal order) is deferred to the QCM change; this change keeps acyclicity a required precondition, enforced via `ultragraph::has_cycle`/`freeze`.
- Audit and update reconvergent tests (e.g. the `logic_graph` diamond) to the joined semantics.

## Capabilities

### New Capabilities
- `comonoid-graph-join`: principled fan-in join for the reasoning engine — topological evaluation over the frozen DAG, a declared associative-commutative join-combine reducing parents to one effect, and the Lean-proved order-invariance (choice of topological linearization does not change the result).

### Modified Capabilities
<!-- No existing capability spec covers graph-reasoning behaviour (core-formalization is the monad layer). The behaviour change is captured under the new capability; the follow-on `formalize-main-crate` change is re-scoped in ITS artifacts to depend on this one. -->

## Impact

- **Rust engine:** `deep_causality/src/traits/causable_graph/graph_reasoning/{mod.rs,stateful.rs}` — topological evaluation + fan-in join; the join-combine policy surface (where the combine is declared: graph-level default vs per-call).
- **New Lean:** `Core/GraphJoin.lean`; registered in `DeepCausalityFormal.lean`; `THEOREM_MAP.md` + `LEAN_CORE.md` rows (advances `Formalization.md` #2/#11).
- **Tests (blast radius = reconvergent graphs only):** update diamond graphs (`causality_graph_reasoning_tests.rs` `logic_graph`, and any other multi-parent graph) to the joined result; linear/tree graphs unchanged. New witness for the order-invariance theorem.
- **Downstream:** `formalize-main-crate` is re-scoped to depend on this change (its graph-reasoning spec drops the "engine gap / gated follow-up" and describes real code).
- **No new dependencies;** `unsafe_code = "forbid"` and the zero-macro/zero-dep ethos untouched. Cyclic-graph join and QCM are out of scope.
