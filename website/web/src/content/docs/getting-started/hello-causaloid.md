---
title: Hello, Causaloid
description: Build, compose, and evaluate a Causaloid in the smallest possible program.
section: getting-started
order: 3
---

This page walks through the smallest program that exercises the library: a Causaloid that wraps a pure predicate, then a second Causaloid composed with the first, then evaluation against a value.

The point is not to be impressive. The point is to leave you with a runnable seed you can mutate.

## What a Causaloid is, in one paragraph

A Causaloid is a self-contained unit of causality. It wraps a function that takes some input, optionally consults a context, and returns a [`PropagatingEffect`](/docs/concepts/effect-propagation-process/). Causaloids compose: a graph of Causaloids is itself a Causaloid. That recursive shape is what lets the library handle both a single rule and a thousand-rule system with one type.

The full concept page is [here](/docs/concepts/causaloid/). For now you only need the shape.

## A first Causaloid

```rust
use deep_causality::{BaseCausaloid, Causaloid, CausalityError, PropagatingEffect};

fn main() -> Result<(), CausalityError> {
    // A Causaloid wrapping a stateless predicate on f64.
    let above_zero: BaseCausaloid<f64> = Causaloid::from_causal_fn(
        1,
        "above_zero",
        "value is greater than zero",
        |x: &f64| -> Result<PropagatingEffect, CausalityError> {
            Ok(PropagatingEffect::Deterministic(*x > 0.0))
        },
    );

    let effect = above_zero.evaluate(&3.5)?;
    println!("effect = {:?}", effect);
    Ok(())
}
```

Three things to notice.

The Causaloid carries an `id`, a short name, and a description. They are not decoration. They show up in the `EffectLog` when a chain of Causaloids fires, which is what makes the chain auditable later.

The wrapped function returns `Result<PropagatingEffect, CausalityError>`. `PropagatingEffect` is a disjoint union of effect shapes; here we use `Deterministic(bool)` because the predicate yields a boolean. The full set is described in the [Effect Propagation Process](/docs/concepts/effect-propagation-process/) page.

`evaluate` runs the Causaloid on a single input. For richer composition you reach for `CausaloidGraph` or one of the collection forms.

## Compose two Causaloids

A trading-style example: a fast/slow moving-average cross plus a volume confirmation. Each rule is its own Causaloid. The conjunction is a graph node whose evaluation depends on both.

```rust
use deep_causality::{BaseCausaloid, Causaloid, CausaloidGraph, CausalityError, PropagatingEffect};

#[derive(Default, Clone, Debug)]
struct Tick {
    fast_ma: f64,
    slow_ma: f64,
    volume: f64,
    median_volume: f64,
}

fn main() -> Result<(), CausalityError> {
    let cross = Causaloid::from_causal_fn(
        1,
        "cross",
        "fast MA above slow MA",
        |t: &Tick| Ok(PropagatingEffect::Deterministic(t.fast_ma > t.slow_ma)),
    );
    let confirm = Causaloid::from_causal_fn(
        2,
        "confirm",
        "volume above 1.4x median",
        |t: &Tick| Ok(PropagatingEffect::Deterministic(t.volume > t.median_volume * 1.4)),
    );

    let mut g = CausaloidGraph::new();
    let a = g.add_root_causaloid(cross);
    let b = g.add_causaloid(confirm);
    g.add_edge(a, b)?;

    let signal: BaseCausaloid<Tick> = Causaloid::from_causal_graph(
        3, "signal", "stage-gated trading signal", g,
    );

    let tick = Tick { fast_ma: 102.5, slow_ma: 100.0, volume: 2_400.0, median_volume: 1_500.0 };
    let effect = signal.evaluate(&tick)?;
    println!("{:?}", effect);
    Ok(())
}
```

The graph has two nodes and one edge. The composed Causaloid evaluates both nodes against the same input and reduces the per-node `PropagatingEffect` values into a single result. The reduction strategy is encoded in the graph; the default is conjunction.

## Reading the effect

A `PropagatingEffect::Deterministic(true)` is the unambiguous case. Other variants carry richer payloads:

- `Numerical(f64)` for a continuous decision.
- `Probabilistic(f64)` when the rule produces a probability.
- `Map`, `RelayTo`, and `ContextualLink` for cross-references that get resolved later.

Pattern-match on the variant you expect, and let the others surface an error if the upstream rule is misbehaving.

## Where to go next

The single-input shape is enough to model many real workflows. When the rules need to see an environment beyond the input value, you move from `Causaloid` to `ContextualCausalFn` and add a [`Context`](/docs/concepts/context/). The [next page](/docs/getting-started/hello-context/) wires that up.
