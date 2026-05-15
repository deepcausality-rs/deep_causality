---
title: Dynamic causality
description: The umbrella idea that the rest of the library sits inside.
section: concepts
order: 1
---

DeepCausality is a framework for **dynamic causality**. The phrase is precise; it is not marketing.

This page explains what the phrase commits to, where the commitment came from, and what it earns you in exchange.

## The axiom

A causal relation is a **monadic functional dependency**. Formally:

```
m₂ = m₁ >>= f
```

Where `m₁` and `m₂` are propagating effects, `f` is a causal function, and `>>=` is monadic bind. The monograph chapter `causality_as_epp.tex` calls this *the* axiom of causality. Every concept in this library is a specialization of that single equation.

Classical causality drops out as a special case. Take the monadic context to be the unit type, restrict the carried function, and you recover Pearl-style structural causal models, dynamic Bayesian networks, Granger causality, Rubin's potential-outcomes framework, and conditional average treatment effects. They are all parametric specializations of the same axiom.

## Why "dynamic"

The dynamics are not about time. They are about whether the structure itself can change.

The monograph identifies three modalities:

| Modality | EPP structure | Context | Epistemology |
| --- | --- | --- | --- |
| Static | fixed | fixed | correspondence (positivism) |
| Dynamic | fixed | mutable | coherence |
| Emergent | mutable | mutable | pragmatic efficacy |

A static model takes a snapshot. A dynamic model lets the context evolve while the rules stay. An emergent model lets the rules themselves evolve. DeepCausality supports all three, and the type system distinguishes them at compile time.

The default position is dynamic: the rules are fixed during a run, the context updates, and the same Causaloid composed against a new Context yields a new propagating effect. That is the day-to-day operating mode for production systems.

## The three primitives

Everything else in the library descends from three things.

[**Causaloid**](/docs/concepts/causaloid/) — a self-contained unit of causality. Wraps a function from input (and optionally context) to a `PropagatingEffect`. Composes recursively: a graph of Causaloids is itself a Causaloid. Borrowed from physicist Lucian Hardy's work on quantum gravity, where the construction first appeared as a way to talk about causal structure without assuming a fixed temporal order.

[**Context**](/docs/concepts/context/) — an explicit hypergraph encoding the environment. Nodes are typed `Contextoid`s carrying data, space, time, spacetime, or symbolic payloads. Edges are arbitrary n-ary relations. Mutating the Context is how the model becomes dynamic; running counterfactuals is how the model interrogates itself.

[**Effect Ethos**](/docs/concepts/effect-ethos/) — a verification layer that sits above effect propagation. Encodes operational rules as `Teloid`s and evaluates every action against them with a defeasible deontic calculus. The conflict-resolution machinery (Lex Posterior, Lex Specialis, Lex Superior) gives you a principled way to reconcile contradictory norms.

## What this earns you

A specific list. Marketing-free.

- **Composition without identity loss.** Two Causaloids combine into a third with the same type. The third behaves like the first two; tooling that worked on one works on the composition.
- **Audit trail by construction.** Every chain accumulates an `EffectLog`. When the system makes a decision, you can ask the system to explain itself, and the explanation is the record of which Causaloids fired in what order with which intermediate effects.
- **Counterfactuals as a first-class operation.** Build a parallel Context, mutate the Contextoids you want to interrogate, evaluate the same Causaloid against it. There is no separate counterfactual engine; the same machinery does both.
- **Reasoning at three modalities** without rewriting the calling code. Promote a static model to dynamic by switching from `Causaloid::from_causal_fn` to `Causaloid::from_contextual_causal_fn`. Promote a dynamic model to emergent by letting the Causaloid graph itself respond to the Context.
- **A formal grounding** in the monograph at `papers/src/EPP/`. The Rust types correspond to formal definitions, not to vibes.

## Where to look next

The three primitive pages walk through the actual types: [Causaloid](/docs/concepts/causaloid/), [Context](/docs/concepts/context/), [Effect Ethos](/docs/concepts/effect-ethos/). The [Effect Propagation Process](/docs/concepts/effect-propagation-process/) page explains the type that ties them together. The [Causal Monad](/docs/concepts/causal-monad/) and [HKT](/docs/concepts/hkt/) pages show how the algebra is implemented in Rust without runtime overhead.

For the formal treatment, start with the [monograph](/docs/monograph/). The shortest path is `Preprint_EPP` for the axiom, then `Preprint_EPP_Formalization` for the math.
