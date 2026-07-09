Hardy's causaloid composition (the causaloid product ⊗^Λ, Eq. 2 on p.4 of [`deep_causality/papers/causaloid.pdf`](../../../deep_causality/papers/causaloid.pdf), arXiv:gr-qc/0509120) is symmetric; causal asymmetry lives entirely in the Λ matrices (the connection data between a region-pair), not in either region and not in an ordering imposed by the product.

The causaloid implementation in this project is fundamentally an inversion of Hardy's causaloid because it puts the asymmetry in the outer encapsulation of the causal monad and the symmetry into the inner representation as a causal function.

The two-layer separation, applied to the fan-in:

Outer layer — sequencing / spacetime asymmetry = the monad. "Before/after" and directionality live in the bind (m₂ = m₁ >>= f) and, in the graph, in the edges and the topological evaluation. This is the engine's business; it legitimately uses positional bookkeeping (node indices) internally.

Inner layer — the causal function's own symmetry/anti-symmetry/asymmetry. Whether f(a,b) treats its arguments equally is a property of f, over the values it receives, identified by intrinsic identity — never by their spacetime position in the graph.

Both frameworks agree on the thing that matters here: the causal element never carries the ordering asymmetry. Hardy keeps the element symmetric and pushes asymmetry onto the connection (Λ); DC keeps the element symmetric and pushes asymmetry onto the composition (the monad). Neither puts "before / after" inside the element.


## The structure

A Causaloid is one of three shapes, recorded in `CausaloidType`:

- **Singleton**: a single causal function. The atomic case.
- **Collection**: a native Rust collection of Causaloids evaluated together under an `AggregateLogic` (conjunction, disjunction, threshold). The "many rules, one decision" case. Slices, `VecDeque`, `HashMap`, and `BTreeMap` all pick up the [`MonadicCausableCollection`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/extensions/causable/mod.rs) blanket impl, so any of them works.
- **Graph**: a proper `CausaloidGraph<Self>` hypergraph (backed by [`ultragraph`](https://github.com/deepcausality-rs/deep_causality/tree/main/ultragraph)). The "rules with structure" case. Order and reachability of evaluation matter, and a single hyperedge can connect more than two Causaloids at once.

The three shapes are *isomorphic-recursive*. A Singleton, a Collection, and a Graph are distinct concrete structures, yet each one implements the same [`Causable`](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/types/causal_types/causaloid/causable.rs) and `MonadicCausable` trait surface. As far as the rest of the library is concerned, each one *is* a Causaloid. That uniformity is what makes them composable into each other. A Causaloid wrapping a Graph can be a node in another Graph, an entry in a Collection, or the operand of a `bind` step. The structure nests to arbitrary depth without the calling code changing shape.

This is the central representational move. Classical causality frameworks force you to pick a structure up front. A Pearl SCM is a graph. A Granger model is a set. A Bayesian network is a graph with a specific edge semantics. Changing the structure means rewriting the model. DeepCausality lets you choose your structure for any specific problem, combine different structures for complex cases, and encapsulate sub-modules into single Causaloids to make larger models manageable and composable.

## Construction

```rust
use deep_causality::{
    AggregateLogic, BaseCausaloid, BaseContext, Causaloid, CausaloidGraph, PropagatingEffect,
};
use std::sync::Arc;

// 1. Stateless, no context. The default for simple rules.
let above_zero: BaseCausaloid<f64> = Causaloid::from_causal_fn(
    1,
    "above_zero",
    "value is greater than zero",
    |x: &f64| Ok(PropagatingEffect::Deterministic(*x > 0.0)),
);

// 2. Contextual. The Causaloid captures an Arc<Context>; the closure
//    receives both the input and the captured context on every call.
let with_ctx: BaseCausaloid<Tick> = Causaloid::from_contextual_causal_fn(
    2, "name", "description", ctx,
    |t: &Tick, c: &Arc<BaseContext>| Ok(PropagatingEffect::Deterministic(true)),
);

// 3. Collection. Aggregates a set of Causaloids under one rule.
let any_of: BaseCausaloid<f64> = Causaloid::from_causal_collection(
    3, "any_of", "any predicate fires",
    vec![above_zero.clone(), other_rule.clone()],
    AggregateLogic::Or,
    None,
);

// 4. Graph. Edges encode dependencies between rules.
let mut g = CausaloidGraph::new(4);
let root = g.add_root_causaloid(stage_one);
let next = g.add_causaloid(stage_two);
g.add_edge(root, next)?;
let pipeline: BaseCausaloid<Tick> = Causaloid::from_causal_graph(
    4, "pipeline", "two-stage signal", g,
);
```

Each constructor returns the same `Causaloid` type. They differ only in which of the four optional fields are populated. The discriminant is `causal_type`.

## References

- Lucien Hardy, *Probability Theories with Dynamic Causal Structure: A New Framework for Quantum
  Gravity*, arXiv:gr-qc/0509120 (2005) — [`deep_causality/papers/causaloid.pdf`](../../../deep_causality/papers/causaloid.pdf).
  Anchors used above: the causaloid product ⊗^Λ (Eq. 2, p. 4) composes regions by **union** —
  symmetric, all elementary regions on an equal footing; the theory-specific **Λ matrices** are the
  data that "break the symmetry between elementary regions" (p. 4; in QT, adjacent-pair Λ ≠
  non-adjacent-pair Λ); §2 (p. 3) states the motivating unification — spacelike regions compose by
  `Â ⊗ B̂`, timelike by the sequential product `B̂Â`, and ⊗^Λ unifies the two so the causal structure
  need not be specified in advance; probabilities are `|v|/|u|` when well-defined (Eq. 3, p. 4); the
  framework has no fundamental state-evolving-in-time (p. 5).
- The formalization program that operationalizes the inversion described here:
  [`causaloid-formalization-roadmap.md`](causaloid-formalization-roadmap.md).
- The four-level stack this structure carries: [`../quantum/full-stack.md`](../quantum/full-stack.md).
