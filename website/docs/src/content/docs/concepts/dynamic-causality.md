---
title: Dynamic causality
description: The umbrella idea the rest of the library sits inside, and the philosophical commitment behind it.
sidebar:
  order: 2
---

DeepCausality is a framework for **dynamic causality**. This page explains what the phrase commits to, where the commitment came from, and what it earns you in exchange.

## The axiom

A causal relation is a **monadic functional dependency**. Formally:

```
m₂ = m₁ >>= f
```

Where `m₁` and `m₂` are propagating effects, `f` is a causal function, and `>>=` is monadic bind. The [EPP preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf) calls this *the* axiom of causality. Every concept in this library is a specialization of that single equation.

Pearl SCMs, dynamic Bayesian networks, Granger causality, the Rubin causal model, and conditional average treatment effects are all parametric specializations of the same axiom. The [classical causality examples](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/classical_causality_examples) walk each one through in code.

## What "dynamic" means

Classical computational causality, from Pearl's Structural Causal Models to Granger time-series analysis, assumes a fixed background spacetime and a static causal structure. At the frontiers of science and engineering, that assumption breaks. Regime shifts in financial markets, multi-scale feedback in autonomous vehicles, and the dynamic spacetime of general relativity all share a property: the causal rules themselves can evolve.

The EPP commits to making that evolution first-class, and the commitment is philosophical before it is technical. Chapter 3 of the preprint traces a line from Aristotle's substance metaphysics, through Hume, Kant, and Reichenbach, to Russell's 1912 critique. Russell observed that modern physics describes systems with time-symmetric equations, while classical causality demands temporal asymmetry. He concluded that "the law of causality is a relic of a bygone age."

The EPP's answer is the **Functional View**, drawn from Whitehead's process philosophy. Reality is a process of becoming, not a collection of substances. Causality is not an external relation linking pre-existing things. It is the structure of effect propagation itself. Russell's puzzle dissolves once you notice that two distinct asymmetries had been conflated under one name: the asymmetry of laws (whether the equations are time-reversal invariant) and the asymmetry of propagation (the direction of flow from cause to effect). The monadic axiom separates the two. The function `f` inside `bind` can be a time-symmetric physics equation. The `bind` operator carries propagation asymmetry as a structural property of composition. Classical causality and modern physics stop fighting.

## The four reasoning modalities

Section 4.9 of the EPP defines four modes of causal reasoning. Each is a setting of two knobs: is the graph fixed or constructed at runtime, and is the context fixed or evolving?

- **Static**: the causal graph is fixed and traversed along a pre-defined pathway. The classical case. A medical risk score driven by a fixed clinical protocol.
- **Dynamic**: the graph is fixed, but the pathway through it is selected at runtime. A subgraph or a shortest-path traversal answers a query. The context may itself be dynamic and feed into pathway selection. A trading model that queries a static rule library against a live market feed.
- **Adaptive**: the Causaloid itself dispatches to the next step based on its own internal logic and the current context. A clinical model that hands off to a "normal blood pressure" or "high blood pressure" subgraph depending on the latest reading. The set of possible pathways is closed and known at design time.
- **Emergent**: the graph is constructed and modified at runtime in response to the context, often combined with adaptive reasoning inside the new graph. New Causaloids and new edges are introduced by a generative process. The set of possible pathways is no longer closed.

Static, Dynamic, and Adaptive all remain deterministic. Their state space is bounded, their dispatch rules are known up front, and formal verification is feasible. Emergent reasoning is different.

## Adaptability vs verifiability

Section 4.9.5 of the EPP states the trade-off plainly:

- **Adaptive reasoning** is deterministic dynamics. The system adapts within a pre-defined and pre-validated set of behaviors. It is verifiable.
- **Emergence** is non-deterministic dynamics. The system can construct causal structures that were not foreseen by the designer. It is not statically verifiable.

The non-verifiability of emergent reasoning is not a flaw in the formalism. It is a property of the world the system is coupled to. When a causal graph evolves in response to a sensor stream, the system reads from an open environment. The generative function cannot be proven deterministic in the abstract, because it consumes data from a world that is not itself bounded.

The EPP responds with an architectural answer: the **Effect Ethos**, an operational guardrail that uses a Defeasible Deontic Inheritance Calculus (DDIC, after Olson, Salas-Damian, and Forbus). The Effect Ethos sits above the Causal State Machine and intercepts every proposed action. It evaluates the action against a graph of Teloids, each one a computable norm. Conflict between norms is resolved by three principles:

- **Lex Posterior**: the more recent norm wins.
- **Lex Specialis**: the more specific norm wins.
- **Lex Superior**: the higher-priority norm wins.

Verifiability is restored at the action layer rather than the reasoning layer. The reasoning graph may evolve in ways no static proof can foresee, yet every action that leaves the system has been checked against an immutable ethos. The result is a discipline for managing emergence via programmable ethics.

## The operational pieces

Four primitives operationalize the axiom:

- [**Causaloid**](/concepts/causaloid/): a self-contained unit of causality. Wraps a function from input (and optionally context) to a `PropagatingEffect`. Composes isomorphic-recursively into Collections and hypergraphs that share the same trait surface.
- [**Context**](/concepts/context/): an explicit hypergraph encoding the environment. Nodes are typed Contextoids carrying data, space, time, spacetime, or symbolic payloads. The Context is what makes a Causaloid evaluation context-relative without committing to a fixed background spacetime.
- [**Propagating Effect**](/concepts/effect-propagation-process/): the shared carrier every Causaloid emits and consumes. Realized as a 5-arity container `CausalEffectPropagationProcess<V, S, C, E, L>` in [`deep_causality_core`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_core), exposed through two aliases: the non-Markovian `PropagatingEffect<T>` with state and context fixed to the unit type, and the Markovian `PropagatingProcess<T, S, C>` that keeps both generic. Lifting from one to the other is one constructor call. The carrier implements the [**Causal Monad**](/concepts/causal-monad/) trait, the `pure`/`bind`/`intervene` algebra that gives structural reasoning (a Causaloid) and sequential reasoning (a bind-chain) a single boundary type, which is why they compose without bridge code. `intervene` implements Pearl's `do()` operator mid-chain, making counterfactual analysis a first-class operation rather than a separate engine.
- [**Effect Ethos**](/concepts/effect-ethos/): the deontic verification layer described above. Required wherever emergent reasoning is in play. Optional otherwise.


## What this earns you

- **Composition without identity loss**: two Causaloids combine into a third with the same type. Tooling that worked on one works on the composition.
- **Audit trail by construction**: every chain accumulates an `EffectLog`. The trail survives errors and is the system's answer to "which rule fired, on what inputs, in what order?"
- **Counterfactuals as a first-class operation**: `intervene` rewrites the value at any point in the chain. The same machinery handles factual and counterfactual evaluation.
- **Four modalities under one type**: a single Causaloid expression can be promoted from static to dynamic to adaptive without rewriting the calling code. Emergence is reachable through the generative process documented in chapter 8 of the preprint.
- **A formal grounding**: the Rust types correspond to definitions in the [EPP preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf) and its companion volumes, not to convention.

## Where to look next

The three primitive pages walk through the actual types: [Causaloid](/concepts/causaloid/), [Context](/concepts/context/), [Effect Ethos](/concepts/effect-ethos/). The [Effect Propagation Process](/concepts/effect-propagation-process/) page covers the 5-arity container. The [Causal Monad](/concepts/causal-monad/) and [HKT](/concepts/hkt/) pages show how the algebra is implemented in Rust without runtime overhead.

For the formal treatment, start with the [EPP preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf). Chapter 3 covers the philosophical move. Chapter 4 defines the axiom and the four reasoning modalities. Chapter 7 covers the Effect Ethos and the non-determinism solution. Chapter 8 covers the ontology of emergence. The [Formalization preprint](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/formalization_effect_propagation_process/epp_formalization.pdf) carries the math.
