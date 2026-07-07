## Why

The graph-reasoning engine visited each node once: the first parent to reach a node (in BFS order) enqueued it and marked it visited (`graph_reasoning/mod.rs`), so a node with **multiple parents** (a reconvergence / diamond) silently consumed whichever single parent BFS happened to reach first and dropped the rest. This is a latent **correctness bug** (`algebraic-causaloid-assumptions.md` #2: "the first parent in BFS order supplies the input; every other parent's effect is silently dropped"), and it also made evaluation depend on node-index / edge-insertion order.

An earlier iteration of this change tried to *define* the reconvergence combine — first as a user-declared commutative monoid, then as a per-causaloid labeled join keyed by parent node index. Both were wrong, for a reason the formalization notes make precise: the **reconvergence merge `∇` is a symmetric-monoidal generator over the effect monad** (copy/discard comonoid Δ + merge ∇ — a dataflow / PROP / free-Markov category, `algebraic-causaloid-assumptions.md` #2 Q2), and it is an **extension of the single-input causaloid** (`I → CausalEffect<O>`, the Kleisli arrow of `CausalArrow.lean`), not a function the causaloid carries. Deciding it in the wrong layer leaked graph structure (node indices, parent/child ordering) into the user-facing API — precisely the spacetime-agnostic property that says parent/child/before-after is *undefined* at the causal-function layer.

**Assumption #2 (the reconvergence merge semantics) is load-bearing and OPEN.** Rather than close it in the wrong place, this change delivers the parts that are correct and congruent with the single-input causaloid **now**, and **defers the `∇` merge** to a dedicated symmetric-monoidal extension that will decide assumption #2 with the right structure.

## What Changes

- **Principled sequencing (the correct, congruent part).** Replace first-parent-wins BFS with **wire-slot topological evaluation** over the frozen acyclic graph: a reachability pre-pass from the start node, an ascending-node-index canonical schedule, and firing a node once all its in-wires are resolved. Linear/tree graphs (in-degree ≤ 1) are **bit-identical** to the previous engine; only the sequencing is now principled instead of BFS-arrival-order.
- **Acyclicity gate.** The classical evaluator requires a topological order, so a cyclic frozen graph is rejected (`ultragraph::has_cycle`).
- **`RelayTo` as sequential round composition.** A command ends the current round and starts a fresh one at the target with the sub-program (single-level relay, state/log threaded), preserving today's follow-the-relay-exclusively semantics.
- **Multi-parent reconvergence fails LOUDLY (∇ deferred).** When two or more parents fire into a node, the engine returns a descriptive `CausalityError` naming the node and its fired parents and pointing at the pending symmetric-monoidal merge extension — turning the previous *silent* first-parent-wins bug into an honest failure, without committing to a merge semantics in the wrong layer. A reconvergence reached by a **single** fired parent is the identity and evaluates normally.
- **No new user-facing surface.** The `Causaloid` API is unchanged. There is no `join_fn`, no `ParentEffects`, no join kernel — those are removed. The reconvergence merge is not expressible under the current single-input causaloid, by design, until the extension lands.

## Deferred to the symmetric-monoidal merge extension (a separate change)

- The reconvergence merge `∇` itself: the free symmetric-monoidal / PROP category with copy (Δ) and merge (∇) generators over the effect monad, interpreted into `Kleisli(PropagatingEffect)` (assumption #2 Q1/Q2). This is where multi-parent fan-in becomes definable, and where per-connection asymmetry (weighted influence, Hardy's Λ) lives — on the edges, not the causaloid.
- The associated Lean formalization (the `∇`/PROP theorems) and any weighted-edge (Λ) kernel.

## Capabilities

### New Capabilities
- `comonoid-graph-join`: principled wire-slot topological sequencing for the reasoning engine (reachability + canonical schedule + acyclicity gate + `RelayTo` round composition), with multi-parent reconvergence failing loudly pending the deferred symmetric-monoidal merge (∇) extension.

### Modified Capabilities
<!-- No existing capability spec covers graph-reasoning behaviour. The behaviour change is captured under the new capability; the follow-on `formalize-main-crate` change is re-scoped in ITS artifacts. -->

## Impact

- **Rust engine:** `deep_causality/src/traits/causable_graph/graph_reasoning/{mod.rs,stateful.rs}` — wire-slot evaluation, reachability pre-pass, canonical schedule, relay rounds, acyclicity gate, loud reconvergence error.
- **Removed:** the `join_fn`/`context_join_fn` causaloid fields + `new_join*` constructors + getters; the `JoinFn`/`ContextualJoinFn` aliases; the `ParentEffects` type; the `LinearJoin` kernel / `join_kernels` module; the `deep_causality_num` dependency (added only for the kernel); `Core/GraphJoin.lean` + its witnesses/THEOREM_MAP rows.
- **Tests (blast radius = reconvergent graphs only):** linear/tree graphs golden-identical; new tests assert the loud multi-fired error and the single-fired identity. No existing test asserted a multi-fired *result* (scan: every pre-existing reconvergent graph is single-fired at runtime), so nothing regresses.
- **Downstream:** `formalize-main-crate` is re-scoped — its graph-reasoning spec drops the reconvergence-join theorems and records the `∇` merge as deferred to the symmetric-monoidal extension. `algebraic-causaloid-assumptions.md` #2 records this change's ruling (sequencing fixed + loud error; merge semantics still OPEN).
- **No new dependencies;** `unsafe_code = "forbid"` and the zero-macro/zero-dep ethos untouched.
