---
title: Hello, Context
description: Add a Context hypergraph and let a Causaloid read from it.
section: getting-started
order: 4
---

The Causaloid in the [previous page](/docs/getting-started/hello-causaloid/) took an input and returned an effect. That covers a surprising amount of practical work, and it stops covering it the moment your rule needs to know something about the world beyond its input.

Context is the explicit place that world lives.

## What a Context is, in one paragraph

A `Context` is a hypergraph whose nodes are `Contextoid`s, which are typed atoms of context information. A Contextoid carries one of five payload kinds: data, space, time, spacetime, or symbolic. A Context can be queried by id, walked along edges, and mutated in place. Mutating it is the *dynamic* in dynamic causality: the same Causaloid composed against a new Context produces a new propagating effect.

Full concept page is [here](/docs/concepts/context/).

## A first contextual Causaloid

We will rewrite the trading example so the volume threshold is not hard-coded. Instead it lives in the Context as a tunable.

```rust
use deep_causality::{
    BaseCausaloid, BaseContext, Causaloid, CausalityError, Context, Contextoid,
    ContextoidType, Data, PropagatingEffect,
};
use std::sync::Arc;

#[derive(Default, Clone, Debug)]
struct Tick { fast_ma: f64, slow_ma: f64, volume: f64 }

fn main() -> Result<(), CausalityError> {
    // 1. Build a Context with one data Contextoid holding our threshold.
    let mut ctx: BaseContext = Context::with_capacity(1, "trading", 8);
    let threshold = Contextoid::new(
        10,
        ContextoidType::Datoid(Data::new(10, "volume_threshold".into(), 1_500.0)),
    );
    let _node_idx = ctx.add_node(threshold);

    let ctx = Arc::new(ctx);

    // 2. A Causaloid that reads the threshold out of the Context.
    let signal: BaseCausaloid<Tick> = Causaloid::from_contextual_causal_fn(
        100,
        "volume_above_threshold",
        "volume exceeds the threshold from context",
        Arc::clone(&ctx),
        |t: &Tick, c: &Arc<BaseContext>| -> Result<PropagatingEffect, CausalityError> {
            let n = c.get_node(10).ok_or(CausalityError("missing threshold".into()))?;
            let cap = match n.vertex_type() {
                ContextoidType::Datoid(d) => d.data(),
                _ => return Err(CausalityError("wrong contextoid type".into())),
            };
            Ok(PropagatingEffect::Deterministic(t.volume > cap))
        },
    );

    let tick = Tick { fast_ma: 0.0, slow_ma: 0.0, volume: 2_400.0 };
    let effect = signal.evaluate(&tick)?;
    println!("{:?}", effect);
    Ok(())
}
```

Three things to notice.

The Causaloid is built with `from_contextual_causal_fn`, which captures the `Arc<Context>` once and threads it into every invocation. The closure receives both the input and the Context.

Contextoids are typed at the value level via the `ContextoidType` enum. `Datoid` carries a `Data` payload; the other variants — `Spaceoid`, `Tempoid`, `SpaceTempoid`, `Symboid` — carry the other four kinds. You query by id and pattern-match on the variant.

The `BaseContext` and `BaseCausaloid` aliases pin the type parameters to their default monomorphizations. You can lift the aliases when you need a different combination of space, time, and symbolic types.

## Mutating the Context

This is what makes the model dynamic. Replace the threshold and the same Causaloid produces a different effect.

```rust
let new_threshold = Contextoid::new(
    10,
    ContextoidType::Datoid(Data::new(10, "volume_threshold".into(), 3_000.0)),
);
ctx_arc.update_node(10, new_threshold)?;

let effect_after = signal.evaluate(&tick)?;
```

The Causaloid never moved. The rule never moved. The world moved. The library treats that as a first-class case rather than as a fragile assumption.

## When to add a Context

A rule of thumb: if the rule needs to see a value that changes over a run, or a value that someone might want to tune without rebuilding, that value belongs in the Context. If the value is a once-and-done compile-time constant, it stays in the closure.

The Context's real power shows up in the [Effect Propagation Process](/docs/concepts/effect-propagation-process/) page, where it threads through a chain of Causaloids and accumulates structured state along the way.

## What's next

You now have all three core moving parts: a Causaloid, a Context, and a propagating effect. The concept pages explain what they actually are. The [examples](/examples/) take six different fields and show what the same three primitives let you build.
