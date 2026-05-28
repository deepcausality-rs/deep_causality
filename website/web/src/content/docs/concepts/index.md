---
title: Concepts
description: Reference pages for every primitive in DeepCausality — Causaloid, Context, Causal Monad, Effect Propagation Process, Causal State Machine, Effect Ethos, and the type-level machinery underneath.
section: concepts
order: 0
---

Reference pages for every primitive the library exposes. Start with **Dynamic causality** for the framing, then **Causaloid** and **Context** for the two structural units, then the **Effect Propagation Process** for what flows between them. The **Causal Monad** page covers the `pure`/`bind` algebra that carrier implements. The rest can be read on demand.

## Foundations

- **[Dynamic causality](/docs/concepts/dynamic-causality/)** — the umbrella idea the rest of the library sits inside, and the philosophical commitment behind it.
- **[Causaloid](/docs/concepts/causaloid/)** — a self-contained unit of causality that composes into larger units of itself.
- **[Context](/docs/concepts/context/)** — the explicit hypergraph that a dynamic causal rule reasons against.
- **[Effect Ethos](/docs/concepts/effect-ethos/)** — the deontic guardrail that intercepts every action the Causal State Machine proposes before it executes.

## Propagation

- **[Effect Propagation Process](/docs/concepts/effect-propagation-process/)** — the carrier effect: the struct that carries a value, a state, a context, an error, and a log through a chain of Causaloids.
- **[Causal Monad](/docs/concepts/causal-monad/)** — the pure/bind algebra the carrier implements, which makes effect propagation composable, auditable, and short-circuiting on error. A trait, not a separate type.
- **[Higher-Kinded Types](/docs/concepts/hkt/)** — how DeepCausality encodes the type constructors that Rust does not natively support.

## Surfaces and tooling

- **[Causal Discovery Language](/docs/concepts/cdl/)** — a typestate-builder DSL for going from raw observational data to an executable causal model.
- **[Causal State Machine](/docs/concepts/csm/)** — a registry of state-action pairs whose transitions are driven by causal evaluation rather than fixed thresholds.
- **[Uncertainty](/docs/concepts/uncertainty/)** — a first-order type for uncertain values, plus a companion type for probabilistic presence.
- **[Uniform Maths](/docs/concepts/uniform-math/)** — one Functor/Monad/CoMonad surface across tensors, multivectors, manifolds, sparse matrices, and effect propagation.

## Reference

- **[Glossary](/docs/concepts/glossary/)** — canonical terminology for DeepCausality. The names land here once; everything else references this page.
