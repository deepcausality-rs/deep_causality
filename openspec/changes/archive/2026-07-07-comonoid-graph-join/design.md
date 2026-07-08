## Context

`evaluate_subgraph_from_cause` was a BFS that marked a child visited when the *first* parent enqueued it, passing that one parent's effect; a reconvergence node consumed an arbitrary single parent and silently dropped the rest (`algebraic-causaloid-assumptions.md` #2). This change fixes the *sequencing* principledly and stops the silent drop, while **deferring** the reconvergence *merge* to the layer where it belongs.

The design travelled through two wrong iterations before landing here, both recorded so the reasoning is not lost:

1. **User-declared commutative monoid at fan-in.** Rejected: it erases parent identity, cannot express asymmetric mechanisms, and demands a commutativity axiom the value type cannot carry.
2. **Per-causaloid labeled join (`join_fn`/`ParentEffects`/`LinearJoin`) keyed by parent node index.** Rejected: it puts the merge *inside* the single-input causaloid and keys it by *graph node index* — a spacetime/sequencing position — leaking graph structure into the user-facing causal function. Under the spacetime-agnostic property, parent/child/before-after is exactly what is undefined at that layer.

The formalization notes name the correct home. The causaloid is a **single-input Kleisli arrow** `I → CausalEffect<O>` (`Core/CausalArrow.lean`); the reconvergence merge `∇` is a **symmetric-monoidal generator** (copy/discard comonoid Δ + merge ∇) over the effect monad — a dataflow / PROP / free-Markov category (`algebraic-causaloid-assumptions.md` #2 Q2) — an **extension** of the causaloid that is not expressible under the current definition, and whose asymmetry lives on the *connections* (edges, Hardy's Λ), not the element. Assumption #2 is load-bearing and OPEN; this change does not close it.

## Goals / Non-Goals

**Goals:**
- Replace first-parent-wins with principled wire-slot topological sequencing over the frozen acyclic graph; keep linear/tree graphs bit-identical.
- Turn the silent multi-parent drop into a **loud error** that points at the deferred merge extension.
- Add nothing to the user-facing API; remove the leaky join surface entirely.
- Keep `RelayTo` semantics (sequential round composition) and add an acyclicity gate.

**Non-Goals (deferred to the symmetric-monoidal merge extension):**
- Defining the reconvergence merge `∇` (the PROP with Δ/∇ over the effect monad; assumption #2 Q1/Q2).
- Weighted/asymmetric influence (Hardy's Λ) as edge/connection data.
- The Lean formalization of `∇` and any weighted-edge kernel.

## Decisions

**D1 — Wire-slot topological evaluation.** Per node, one slot per in-wire resolving to fired/inactive; a reachability pre-pass from the start marks non-descendant wires inactive (so they are never counted); ready nodes are processed in ascending node index. This is the principled replacement for BFS arrival-order and makes single-input/linear/tree evaluation deterministic and bit-identical.

**D2 — Single fired parent = identity.** A reconvergence reached by exactly one fired parent is not a merge; the node consumes that effect unchanged. This is what keeps `RelayTo`-conditional and mid-start graphs working unchanged.

**D3 — Multi-parent reconvergence = loud error, merge deferred.** Two or more fired parents → a descriptive `CausalityError` naming the node and its fired parents and pointing at the pending `∇` extension. Rationale: assumption #2 is open and load-bearing; any guessed merge (monoid, labeled join, or Collection reuse) closes it in the wrong layer. A loud failure is honest and reversible; the previous silent first-parent-wins was neither.

**D4 — No user-facing merge surface.** The `Causaloid` stays a single-input arrow. `join_fn`/`context_join_fn`, `JoinFn`/`ContextualJoinFn`, `ParentEffects`, and `LinearJoin`/`join_kernels` are removed, along with the `deep_causality_num` dependency added only for the kernel. The merge is introduced later as the symmetric-monoidal extension, where per-connection data rides the edges.

**D5 — RelayTo = sequential round composition; acyclicity gate.** A command ends the round and restarts at the target with the sub-program; the classical run requires a frozen acyclic graph (`ultragraph::has_cycle`).

## Risks / Trade-offs

- **[Reconvergence now errors instead of (wrongly) evaluating]** → This is intended: the previous behaviour was a silent correctness bug (assumption #2). The scan showed no existing test asserted a multi-fired *result*, so no regression; new tests pin the loud error and the single-fired identity.
- **[The change no longer implements a "join" despite its id]** → The id `comonoid-graph-join` is retained for continuity; the join (∇) is explicitly deferred. The follow-on extension change will carry the merge.

## Migration Plan

Land before `formalize-main-crate`. Steps: (1) remove the leaky join surface (types, aliases, causaloid fields/ctors, kernel, num dep, Lean GraphJoin + witnesses); (2) wire-slot engine in `graph_reasoning/{mod,stateful}.rs` with loud reconvergence error; (3) golden linear/tree + new reconvergence tests; (4) rescope `formalize-main-crate` graph-reasoning spec; (5) record the ruling in `algebraic-causaloid-assumptions.md` #2. Rollback = revert the engine change; the removed surface was introduced by this change only.

## Open Questions (all belong to the deferred extension, not this change)

- The reconvergence merge `∇` semantics and its categorical target (PROP with Δ/∇ over the effect monad) — assumption #2 Q1/Q2.
- Where per-connection asymmetry (weights / Hardy's Λ) lives — expected: edges, decided with the do-operator/QCM layer.
