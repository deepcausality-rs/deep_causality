---
title: Hello, Effect Propagation
description: How the Causaloid, the Causal Monad, and the propagating-effect types compose into one chain.
section: getting-started
order: 5
---

The earlier pages introduced three pieces of the library: the Causal Monad (with its `pure` and `bind`), the Causaloid (which wraps a causal function), and the Context (which holds the world a rule reads from). Each one has its own page because each one has its own surface. This page is about what happens when you put them together.

The short version: a Causaloid's evaluation and a Causal Monad's `bind` both return the same type. That single shared return type is what lets one chain mix structural causal reasoning (Causaloids, Collections, Graphs) with sequential causal reasoning (monadic bind) without bridge code.

## The shared return type

Every Causaloid evaluation returns a `PropagatingEffect<T>` (stateless) or a `PropagatingProcess<T, S, C>` (stateful). Every monadic `bind` returns the same. The Causaloid page covered the structural side; the [Causal Monad page](/docs/getting-started/hello-causal-monad/) covered the sequential side. The type that flows between them is the same on both sides.

A practical consequence: you can evaluate a Causaloid, take its `PropagatingEffect`, and `.bind(|...|)` directly onto it. You can run a bind chain, take the result, and feed it into a Causaloid's `evaluate`. Both directions work because the carrier is uniform.

## Two aliases, one underlying type

`PropagatingEffect<T>` and `PropagatingProcess<T, S, C>` are both type aliases over the same 5-arity container, [`CausalEffectPropagationProcess<V, S, C, E, L>`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_core):

```rust
pub type PropagatingEffect<T> =
    CausalEffectPropagationProcess<T, (), (), CausalityError, EffectLog>;

pub type PropagatingProcess<T, S, C> =
    CausalEffectPropagationProcess<T, S, C, CausalityError, EffectLog>;
```

The difference is the second and third type parameters. `PropagatingEffect` fixes state and context to the unit type `()`. `PropagatingProcess` keeps them generic.

## Non-Markovian vs Markovian

The distinction maps directly onto two reasoning styles you will run into in practice:

- **`PropagatingEffect<T>` is non-Markovian**: each step depends only on its input value and the rules it runs. No state is carried across steps, no context is threaded through the chain. Treated as a Higher-Kinded Type, the witness is `HKT3` over `(T, E, L)`. Use this when the steps in your chain do not need to remember anything from earlier steps.
- **`PropagatingProcess<T, S, C>` is Markovian**: each step receives the previous value *plus* the threaded state `S` and context `C`. The state evolves as the chain runs; the context can be read from at any step. Treated as a Higher-Kinded Type, the witness is `HKT5` over `(T, S, C, E, L)`. Use this when the chain needs to accumulate, decay, or condition on what came before.

Both share the underlying container, so the `bind` operator works the same way for both. What differs is what each step has access to.

## Lifting a `PropagatingEffect` into a `PropagatingProcess`

A common pattern: you start with simple non-Markovian rules, then realize one downstream step needs state. The library lets you lift without rewriting the upstream.

```rust
use deep_causality::{PropagatingEffect, PropagatingProcess};

#[derive(Clone, Default)]
struct RiskState { total_risk: f64 }

let stateless: PropagatingEffect<f64> = PropagatingEffect::pure(0.5);

let stateful: PropagatingProcess<f64, RiskState, ()> =
    PropagatingProcess::with_state(stateless, RiskState::default(), None);

let updated = stateful.bind(|val, mut state, _ctx| {
    let v = val.into_value().unwrap_or_default();
    state.total_risk += v;
    let mut next = PropagatingProcess::pure(v * 2.0);
    next.state = state;
    next
});
```

`PropagatingProcess::with_state(effect, initial_state, initial_context)` takes a stateless `PropagatingEffect`, an initial state, and an optional initial context. It returns a `PropagatingProcess` ready for stateful binds. The value, error, and log channels carry across unchanged. The state and context channels are now live.

The reverse direction is also fine: a stateful chain can ignore its state and context entirely when a step does not need them. The type-level distinction is real, but the boundary is one constructor call.

## The Causal Monad stands on its own

You can use the Causal Monad without Causaloids. The starter example does exactly that: [`examples/starter_example`](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/starter_example) walks Pearl's Ladder of Causation as a pure `pure`/`bind`/`intervene` chain with no Causaloid in sight. The monad is enough on its own when the reasoning is sequential and the rules are small enough to live as closures.

You can also use Causaloids without explicit monadic chaining. A single `causaloid.evaluate(&PropagatingEffect::pure(input))` is a complete program for the cases where structural composition (Singleton, Collection, Graph) is what you want.

## How the Causal Monad and Causaloid compose

Real systems need both. Sequential transforms belong in a Causal Monad bind-chain. Parallel aggregation belongs in a Causaloid Collection. Cross-influencing dependencies belong in a Causaloid Graph. Because every shape returns the same propagating-effect carrier, a single pipeline can mix all three. A Causaloid Graph emits a `PropagatingEffect`, which a `.bind` step consumes, which feeds another Causaloid evaluation, which feeds a `.bind`, and so on. State and audit log accumulate across every stage.

The [flight envelope monitor example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/avionics_examples/flight_envelope_monitor) is the canonical demonstration. It runs a Causaloid Collection over five sensor-health checks, a three-step Causal Monad bind-chain for state estimation, and a Causaloid hypergraph of six envelope protections, all threading through one `PropagatingProcess<T, FlightState, AircraftConfig>` with state and audit log carried across every stage.

## What this enables

Most of the advanced examples in the repository depend on this composition. The pattern is the same in each: a Causaloid (Singleton, Collection, or Graph) supplies the structural reasoning, and a Causal Monad bind-chain sequences the rest, both sharing the same propagating-effect carrier. The boundary between "structural" and "sequential" is fluid and you can move that boundary as the problem evolves. Start with a simple non-Markovian causal monad. Add structure later with a Causaloid. Add state later into the Markovian part of the chain, and then combine all parts fluently.

The concept pages on [Causaloid](/docs/concepts/causaloid/), [Causal Monad](/docs/concepts/causal-monad/), and [Effect Propagation Process](/docs/concepts/effect-propagation-process/) go deeper on the algebra. The [examples](/examples/) put the composition to work.
