---
title: Getting started
description: Install DeepCausality, then walk through the smallest programs that exercise the Causal Flow DSL, the Causal Monad underneath it, the Causaloid, the Context, and end-to-end effect propagation.
sidebar:
  order: 0
---

Six short pages. Install first, then five "hello, X" programs that each isolate one moving part. By the end you have written, evaluated, and composed every moving part the library is built on.

- **[Install](/getting-started/install/)** — add DeepCausality to a Rust project.
- **[Hello, Causal Flow](/getting-started/hello-causal-flow/)** — the high-level DSL that reads causal reasoning as a pipeline. The clearest place to start.
- **[Hello, Causal Monad](/getting-started/hello-causal-monad/)** — the `pure` and `bind` engine the flow is built on. Walk a value through a three-step chain and look at what flowed.
- **[Hello, Causaloid](/getting-started/hello-causaloid/)** — build, evaluate, and compose Causaloids in the smallest possible program.
- **[Hello, Context](/getting-started/hello-context/)** — build a Context hypergraph and let a Causaloid read from it.
- **[Hello, Effect Propagation](/getting-started/hello-effect-propagation/)** — how a Causaloid's structural reasoning and a bind-chain compose over one shared carrier effect.

For the conceptual model behind the code, see [Concepts](/concepts/).
