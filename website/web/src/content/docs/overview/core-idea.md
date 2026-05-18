---
title: The core idea
description: One axiom in plain English — causality as a spacetime-agnostic monadic functional dependency.
section: overview
order: 3
---

DeepCausality rethinks causality from a single foundation:

> Causality is a spacetime-agnostic monadic functional dependency.

That phrase is dense. Unpacked:

- **Functional dependency**: an effect is a function of the previous effect. `m₂ = m₁ >>= f`. Think of a process of event propagation, not a "cause" acting on an "effect."
- **Monadic**: the function is composed inside a monad that carries state, context, error, and an audit log alongside the value.
- **Spacetime-agnostic**: time and space are not built into the relation. They are data the causal function reads from a context, the same way it reads any other data.
- **Explicit context**: because spacetime is not built in, anything time-like or space-like has to live in an explicit Context. DeepCausality gives you a hypergraph for that, and a model can read from more than one Context at a time.

A useful intuition is a ripple in a pond. One ripple is an effect. It propagates outward and produces the next ripple. DeepCausality is a framework for defining how those ripples spread, what each one carries, and what happens when the rules for spreading themselves change.

## Classical methods drop out of the same axiom

Pearl's Structural Causal Models, dynamic Bayesian networks, Granger causality, the Rubin causal model, and conditional average treatment effects are all parametric specializations of this one axiom. Setting the monadic context to the unit type and applying three further restrictions on the carried function recovers Pearl's classical definition exactly. The [classical causality examples](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples) walk you through implementing any of the classical methods of computational causality expressed via DeepCausality.

## Where to go from here

Once you have the axiom, the rest of the library is the operational machinery that makes it computable:

- The [Causaloid](/docs/concepts/causaloid/) is the unit that wraps the causal function.
- The [Context](/docs/concepts/context/) is the hypergraph that holds the world the function reads from.
- The [Effect Ethos](/docs/concepts/effect-ethos/) is the safety layer for the cases where the causal rules themselves evolve.
- The [Dynamic causality](/docs/concepts/dynamic-causality/) page is the technical entry point: four reasoning modalities, the philosophical move behind them, and the adaptability-versus-verifiability trade-off.

For the formal treatment, see the [Effect Propagation Process preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf).
