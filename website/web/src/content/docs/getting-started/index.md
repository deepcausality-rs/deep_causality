---
title: Getting started
description: Install DeepCausality, then walk through the smallest programs that exercise the Causaloid, the Causal Monad, the Context, and end-to-end effect propagation.
section: getting-started
order: 0
---

Five short pages. Install first, then four "hello, X" programs that each isolate one moving part. By the end you have written, evaluated, and composed every moving part the library is built on.

- **[Install](/docs/getting-started/install/)** — add DeepCausality to a Rust project.
- **[Hello, Causal Monad](/docs/getting-started/hello-causal-monad/)** — the smallest program that exercises `pure` and `bind`. Walk a value through a three-step chain and look at what flowed.
- **[Hello, Causaloid](/docs/getting-started/hello-causaloid/)** — build, evaluate, and compose Causaloids in the smallest possible program.
- **[Hello, Context](/docs/getting-started/hello-context/)** — build a Context hypergraph and let a Causaloid read from it.
- **[Hello, Effect Propagation](/docs/getting-started/hello-effect-propagation/)** — how a Causaloid's structural reasoning and a bind-chain compose over one shared carrier effect.

For the conceptual model behind the code, see [Concepts](/docs/concepts/).
