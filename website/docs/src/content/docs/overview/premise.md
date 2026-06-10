---
title: Premise
description: One axiom — Causality as a spacetime-agnostic monadic process.
sidebar:
  order: 2
---

## A different premise

DeepCausality starts from a different premise than classical causality.

Classical causality is rooted in a static space-time assumption that traces back to Seneca's definition of causality, formulated approximately two thousand years ago. In the meantime, contemporary science has advanced in foundational fields such as quantum physics, general relativity, and quantum physics, where the fixed space-time assumption simply does not hold any longer. Causal rules can evolve. Spacetime itself can curve. The background against which "cause precedes effect" becomes dynamic.

DeepCausality responds by rooting itself in [**Whitehead's process philosophy**](https://plato.stanford.edu/entries/process-philosophy/), which shifts the Aristotelian assumption of a static snapshot in time toward a dynamic *process of becoming*. The project then adapted the essence of process philosophy into a **spacetime-agnostic dynamic causal process**. The theoretical foundation deserves its own book one day; the [Effect Propagation Process preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf) is the long form of the underlying premise.

## The axiom

That premise is captured in a single foundational axiom, condensed into the following working definition:

> Dynamic Causality is the spacetime-agnostic monadic process in which one propagating effect is obtained from another by applying a causal function within the monad:
>
> `m₂ = m₁ >>= f`

That is a dense phrase. Let's unpack it:

- **Monadic process**: a propagating effect is a type alias over an arity-5 monad that carries state, context, error, and an audit log alongside the value. The monad laws (left identity, right identity, associativity) guarantee that the carrier's bookkeeping is threaded through every step automatically, which is what gives the chain its end-to-end explainability.

- **Functional dependency**: each propagating effect is obtained from the previous one by applying a causal function `f` within the monad: `m₂ = m₁ >>= f`. The function consumes one propagating effect and emits the next. Chained, those steps form a process of effect propagation. Therefore, the key mechanism of dynamic causality is the effect propagation process.

- **Spacetime-agnostic**: time and space are not built into the relation. They are data the causal function reads from a context, the same way it reads any other input it needs to compute its result.

- **Explicit context**: because spacetime is not built in, anything time-like or space-like has to live in an explicit Context. This makes the embedded causal function independent of any specific spacetime, so you can encode Euclidean space, Minkowski or Lorentzian spacetime, and anything in between.

A useful intuition is a ripple in a pond. One ripple is an effect. It propagates outward and produces the next ripple. DeepCausality is a framework for defining how those ripples spread, what each one carries, and what happens when the rules for spreading themselves change.

For more details on the axiom and the properties it unlocks, see [The Axiom](/concepts/axiom/).

## Where to go from here

Once you have the axiom, the rest of the library is the operational machinery that makes it computable:

- The [Causaloid](/concepts/causaloid/) is the unit that wraps the causal function.
- The [Context](/concepts/context/) is the hypergraph that holds the world the function reads from.
- The [Effect Ethos](/concepts/effect-ethos/) is the safety layer for the cases where the causal rules themselves evolve.
- The [Dynamic causality](/concepts/dynamic-causality/) page is the technical entry point: four reasoning modalities, the philosophical move behind them, and the adaptability-versus-verifiability trade-off.

For the formal treatment, see the [Effect Propagation Process preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf).
