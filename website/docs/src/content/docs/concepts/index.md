---
title: Concepts
description: Reference pages for every primitive in DeepCausality — Causaloid, Context, the Effect Propagation Process (the carrier that implements the CausalMonad trait), Causal State Machine, Effect Ethos, and the type-level machinery underneath.
sidebar:
  order: 0
---

Reference pages for every primitive the library exposes. Start with **Dynamic causality** for the framing, then **Causaloid** and **Context** for the two structural units, then the **Effect Propagation Process** for what flows between them. The **Causal Monad** page covers the `pure`/`bind` algebra that carrier implements. The rest can be read on demand.

## Foundations

- **[The Axiom](/concepts/axiom/)** — the single axiom the framework rests on, `m₂ = m₁ >>= f`, and the properties it unlocks.
- **[Dynamic causality](/concepts/dynamic-causality/)** — the umbrella idea the rest of the library sits inside, and the philosophical commitment behind it.
- **[Causaloid](/concepts/causaloid/)** — a self-contained unit of causality that composes into larger units of itself.
- **[Context](/concepts/context/)** — the explicit hypergraph that a dynamic causal rule reasons against.
- **[Effect Ethos](/concepts/effect-ethos/)** — the deontic guardrail that intercepts every action the Causal State Machine proposes before it executes.

## Propagation

- **[Effect Propagation Process](/concepts/effect-propagation-process/)** — the carrier effect: the struct that carries a value, a state, a context, an error, and a log through a chain of Causaloids.
- **[Causal Monad](/concepts/causal-monad/)** — the pure/bind algebra the carrier implements, which makes effect propagation composable, auditable, and short-circuiting on error. A trait, not a separate type.
- **[Causal Flow](/concepts/causal-flow/)** — the fluent high-level DSL over the causal monad. Pipelines, loops, branches, and interventions as verbs, lowering to `pure` and `bind`.
- **[Higher-Kinded Types](/concepts/hkt/)** — how DeepCausality encodes the type constructors that Rust does not natively support.

## Surfaces and tooling

- **[Causal Discovery Language](/concepts/cdl/)** — a typestate-builder DSL for going from raw observational data to an executable causal model.
- **[Causal State Machine](/concepts/csm/)** — a registry of state-action pairs whose transitions are driven by causal evaluation rather than fixed thresholds.
- **[Uncertainty](/concepts/uncertainty/)** — a first-order type for uncertain values, plus a companion type for probabilistic presence.
- **[Uniform Math](/concepts/uniform-math/)** — one Functor/Monad/CoMonad surface across tensors, multivectors, manifolds, sparse matrices, and effect propagation.

## Reference

- **[Glossary](/concepts/glossary/)** — canonical terminology for DeepCausality. The names land here once; everything else references this page.
