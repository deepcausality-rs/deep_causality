---
title: Hello, Causaloid
description: Build, evaluate, and compose Causaloids in the smallest possible program.
sidebar:
  order: 4
---

This page walks through the smallest program that exercises [`Causaloid`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality): a single Causaloid that wraps a predicate, then a two-node graph that composes two Causaloids, then evaluation.

## What a Causaloid is

A Causaloid is a self-contained unit of causality. It carries an identifier, a human-readable description, and a causal function from an input value to a [`PropagatingEffect`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_core). Causaloids compose isomorphic-recursively into Collections and hypergraphs that share the same trait surface, which is what the [Causaloid concept page](/concepts/causaloid/) covers in full.

For this example you only need a simple causaloid.

## A first Causaloid

```rust
use deep_causality::{BaseCausaloid, Causaloid, MonadicCausable, PropagatingEffect};

fn above_zero(x: f64) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(x > 0.0)
}

fn main() {
    let causaloid: BaseCausaloid<f64, bool> =
        Causaloid::new(1, above_zero, "value is greater than zero");

    let effect = causaloid.evaluate(&PropagatingEffect::pure(3.5_f64));
    println!("effect = {:?}", effect.value);
}
```

Three things to notice.

The causal function has signature `fn(I) -> PropagatingEffect<O>`. It takes a plain value and returns a `PropagatingEffect`. There is no `Result` wrapping: errors are conveyed through the `PropagatingEffect` itself with `PropagatingEffect::from_error(...)`, and the chain short-circuits automatically.

`Causaloid::new(id, causal_fn, description)` takes an integer id, the causal function, and a description string. The id and description show up in the [`EffectLog`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_core) when the Causaloid fires, which is what makes a chain auditable later.

`evaluate` takes a reference to an incoming `PropagatingEffect`, not a bare input. To pass a plain value, lift it with `PropagatingEffect::pure(...)` first. The return is another `PropagatingEffect`, which you read through `effect.value.into_value()` when you need the inner type back.

## Compose two Causaloids in a graph

A trading-style example: a fast/slow moving-average cross plus a volume confirmation. Each rule is its own Causaloid. A two-node [`CausaloidGraph`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality) composes them.

```rust
use deep_causality::{
    CausableGraph, Causaloid, CausaloidGraph, MonadicCausableGraphReasoning, PropagatingEffect,
};

fn cross(active: bool) -> PropagatingEffect<bool> {
    // Stand-in: the upstream effect carries whether the cross fired.
    PropagatingEffect::pure(active)
}

fn confirm(active: bool) -> PropagatingEffect<bool> {
    // Stand-in: the upstream effect carries whether volume confirmed.
    PropagatingEffect::pure(active)
}

fn main() {
    let c1 = Causaloid::<bool, bool, (), ()>::new(1, cross, "fast MA above slow MA");
    let c2 = Causaloid::<bool, bool, (), ()>::new(2, confirm, "volume above 1.4x median");

    let mut graph: CausaloidGraph<Causaloid<bool, bool, (), ()>> = CausaloidGraph::new(1);
    let root = graph.add_root_causaloid(c1).unwrap();
    let next = graph.add_causaloid(c2).unwrap();
    graph.add_edge(root, next).unwrap();

    // The graph must be frozen before it can be reasoned over.
    graph.freeze();

    let effect = graph.evaluate_single_cause(root, &PropagatingEffect::pure(true));
    println!("{:?}", effect.value);
}
```

Three things changed compared to a singleton:

- The graph is built imperatively. `add_root_causaloid` returns the index of the root, `add_causaloid` adds further nodes, and `add_edge(from, to)` wires them up.
- The graph must be **frozen** before evaluation. Internally, freezing switches the underlying [`ultragraph`](https://github.com/deepcausality-rs/deep_causality/tree/main/ultragraph) backend from its dynamic build phase to its CSR query phase, which is what gives DeepCausality sub-second traversal on graphs of ten million nodes or more.
- Evaluation uses graph methods on the [`MonadicCausableGraphReasoning`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality) trait, not the singleton `evaluate`. `evaluate_single_cause(idx, &effect)` runs one node. `evaluate_subgraph_from_cause`, `evaluate_shortest_path`, and similar methods drive the dynamic and adaptive reasoning modalities documented on the [Dynamic causality page](/concepts/dynamic-causality/).

## Reading the effect

A `PropagatingEffect`'s `value` field is an [`EffectValue<T>`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_core) enum:

- `Value(T)`: the everyday case. A concrete output of type `T`.
- `None`: the explicit absence of a value.
- `ContextualLink(id1, id2)`: a deferred reference into the Context.
- `RelayTo(idx, effect)`: a dispatch command that routes the effect to a different node in the graph. This is what powers adaptive reasoning.
- `Map(parts)`: a labeled bundle of sub-effects for fan-out.

You typically pattern-match on the variant you expect:

```rust
match effect.value {
    deep_causality_core::EffectValue::Value(v) => println!("got {v}"),
    deep_causality_core::EffectValue::None => println!("no effect"),
    other => println!("unexpected: {other:?}"),
}
```

For the common case where you just want the inner value, `effect.value.into_value()` returns an `Option<T>`.

## What's next

The single-input shape is enough to model many real workflows. When rules need to read an environment beyond their input, you move to a context-aware Causaloid via [`Causaloid::new_with_context`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality), which threads a [`Context`](/concepts/context/) into every evaluation. The [next page](/getting-started/hello-context/) sets that up.

For a complete, runnable end-to-end example that walks Pearl's Ladder of Causation through `pure`, `bind`, and `intervene`, see [`examples/starter_example`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/starter_example).
