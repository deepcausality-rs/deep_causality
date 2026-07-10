<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# Proposal: `freeze_dag()` — opt-in DAG enforcement for the causal graph

**Status:** implemented. Additive, non-breaking.
**Relates to** [`algebraic-causaloid-assumptions.md`](causal-algebra/algebraic-causaloid-assumptions.md) assumption #2 (Q3, structural hygiene).

**Implemented as** a default method `freeze_dag()` on the `CausableGraph<T>` trait (`deep_causality/src/traits/causable_graph/graph/mod.rs`), using the confirmed "freeze → check → unwind on cycle" ordering: `ultragraph`'s `has_cycle()` is available only on the static (frozen) state, and the static check (Kahn's algorithm) is total (`Ok(true)`/`Ok(false)`), so the only reachable error is "cycle." Tests: `deep_causality/tests/types/causal_types/causaloid_graph/causality_graph_freeze_tests.rs`.

## Problem

`CausableGraph::add_edge` performs no cycle check (`types/causal_types/causaloid_graph/causable_graph.rs:112-117`), and `freeze()` transitions the two-phase graph from its dynamic form to the static (CSR) reasoning form **unconditionally** — it accepts cyclic structures. "DAG" is only a documented intent (`causaloid/mod.rs:53`), never enforced. A cyclic graph therefore builds and freezes silently; reasoning (gated on `is_frozen()`) then traverses it with no defined meaning (BFS tolerates the cycle via its `visited` guard but assigns it none).

## Proposal

Add a new method `freeze_dag()` to `CausableGraph`, alongside the existing `freeze()`:

- `freeze()` — unchanged. Freezes unconditionally; allows non-DAG (cyclic) structures.
- `freeze_dag()` — checks acyclicity and freezes **only if the graph is a DAG**; otherwise returns an error and leaves the graph unfrozen (or unwinds the freeze — see implementation note).

Sketch (exact form to be confirmed against the `ultragraph` cycle API):

```rust
/// Freeze the graph for reasoning, enforcing acyclicity.
/// Returns an error and does not present a frozen DAG if the graph contains a cycle.
fn freeze_dag(&mut self) -> Result<(), CausalityGraphError>;
```

## Why this is safe to add

- **Opt-in.** Callers that need the DAG guarantee call `freeze_dag()`. Every existing `freeze()` call site is untouched and behaves exactly as before.
- **Additive, non-breaking.** A new trait method (with a default body in terms of the graph's existing cycle detection and `freeze()`), no change to any existing signature or behavior. Nothing that compiles today stops compiling.
- **Single checkpoint.** Reasoning already requires `is_frozen()`, so the freeze boundary is the natural and only place to enforce a structural invariant once, rather than paying a cycle check on every `add_edge`.

## What it resolves — and what it does not

**Resolves (part of assumption #2, Q3):** the *structural acyclicity guarantee*. After `freeze_dag()` returns `Ok`, the graph is provably a DAG, which is the precondition for any topological interpretation. This closes the "acyclicity is not enforced" finding as an opt-in.

**Does not resolve (still open in #2):**
- **Join / reconvergence semantics (Q1).** A DAG can still contain reconvergent nodes (the diamond `A→{B,C}→D`). `freeze_dag()` does not define what `D` sees when `B` and `C` both feed it; the current BFS still silently drops all but the first parent. Acyclicity ≠ a defined join.
- **The right abstraction (Q2).** Unchanged.
- **`RelayTo` non-termination.** `RelayTo` jumps are decided at runtime from causaloid output; a *static* cycle check cannot see a relay loop. `freeze_dag()` gives no termination guarantee for relay cycles; a separate relay bound is still needed.

So `freeze_dag()` is necessary structural hygiene and a clean opt-in win, but it is **one** of several pieces; it does not by itself make the graph form a rigorous algebra.

## Implementation note (to confirm before coding)

`ultragraph`'s `has_cycle` / `find_cycle` / `topological_sort` are exposed on the **static** representation (`GraphState::Static` in `ultragraph/src/types/ultra_graph/graph_algo.rs`). If cycle detection requires the static form, `freeze_dag()` cannot literally "check before freeze"; the viable shape is **freeze → check → on cycle, unwind/`unfreeze()` and return `Err`**. Confirm the available state for `has_cycle` and pick whichever ordering the API supports; the external contract (`Ok` ⇒ frozen DAG, `Err` ⇒ not presented as a frozen DAG) stays the same either way.

## Test points

- Acyclic graph: `freeze_dag()` returns `Ok`, `is_frozen()` is `true`, reasoning proceeds.
- Cyclic graph: `freeze_dag()` returns `Err`, the graph is not left in a frozen-and-cyclic state, and the error names the cycle.
- `freeze()` on a cyclic graph: unchanged (still succeeds) — proving the new method is purely additive.
