---
title: DeepCausality v0.8.2 supports adaptive reasoning
description: This post summarizes the new adaptive reasoning feature of DeepCausality v0.8.2
date: 2025-08-08
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

The DeepCausality project announces the release of DeepCausality 0.8.2. This release includes a series of major updates
to the DeepCausality library, introducing powerful new features that significantly enhance its flexibility, robustness,
and ease of use.

## 🚀 Highlights in 0.8.2

* Adaptive reasoning
* Flexible Collection Reasoning with Aggregate Logic
* Model Assumption Verification
* Unified `PropagatingEffect`
* New code examples

## 💡 Adaptive Reasoning

Previously, causal reasoning flowed strictly along the predefined edges of the graph. Now, with the introduction of the
`PropagatingEffect::RelayTo` variant, a Causaloid can dynamically dispatch the flow of reasoning to any other Causaloid
in the graph. This enables sophisticated, adaptive reasoning patterns where the system can choose its own path through
the graph, conditional on intermediate results.

## ⚡ Collection Reasoning with Aggregate Logic

Reasoning over a causal collection is no longer limited to a simple "all or nothing" mode. DeepCausality 0.8.2
introduces a configurable `AggregateLogic` enum that allows you to specify how results from a group of causaloids should
be combined. You can now reason with aggregation modes like:

* `All`: All causaloids must be active.
* `Any`: At least one causaloid must be active.
* `None`: No causaloids may be active.
* `Some(k)`: A specific number (k) of causaloids must be active (i.e., set a threshold).

This logic is supported across deterministic, probabilistic, and mixed-mode reasoning, giving you fine-grained control
over how collective causes contribute to an effect.

## 📍 Model Assumption Verification

A model's conclusions are only as reliable as its underlying assumptions. To make models safer, more robust, and
transferable, DeepCausality 0.8.2 introduces a formal system for programmatic assumption verification.

The new `Transferable` trait, implemented by the `Model` struct, provides a `verify_assumptions` method. This allows you
to test a model's foundational assumptions against a dataset before putting it into production. Assumption functions (
`EvalFn`) can now fail gracefully, returning a `Result` with the new, dedicated `AssumptionError` type, which pinpoints
exactly which assumption failed.

## 🪐 Unified PropagatingEffect

DeepCausality 0.8.2 unifies the `Evidence` and `PropagatingEffect` types. Previously, `Evidence` was used for inputs and
`PropagatingEffect` for outputs. Now, a single, expanded `PropagatingEffect` enum represents the data that flows into,
through, and out of the causal graph. This creates a simpler, more intuitive, and consistent API. In the same way that
the causaloid unifies cause and effect, the `PropagatingEffect` enum now unifies the input and output types.

## ✨ New Code Examples

As part of the 0.8.2 release, we added a new set of code examples to
the [examples directory](https://github.com/deepcausality-rs/deep_causality/tree/main/examples) on GitHub. These
examples demonstrate how to express a number of existing methodologies with DeepCausality. Specifically, we added
examples for:

* CATE: [Conditional Average Treatment Effects.](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/epp_cate)
* DBN: [Dynamic Bayesian Networks.](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/epp_dbn)
* Granger: [Granger Causality for time series.](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/epp_granger)
* RCM: [Rubin Causal Model.](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/epp_rcm)
* SCM: [Structural Causal Model.](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/epp_scm)

The informed reader may wonder how DeepCausality can possibly express so many different causal methods in a uniform way. A large
part of the generalization DeepCausality achieves is due to its foundation in a single, axiomatic, generalized
definition of causality. The Effect Propagation Process (EPP) defines the foundation of causal reasoning in
DeepCausality, as explained in
the [EPP Paper](https://github.com/deepcausality-rs/deep_causality/blob/main/papers/effect_propagation_process/epp.pdf).

Unlike existing methodologies of computational causality, DeepCausality does not assume the presence of a linear
background spacetime. As a result, it can express externalities such as temporal, spatial, and data dependencies via an
explicit context.

From there, constructing a counterfactual reality comes down to building a new context and applying contextual causal
reasoning linked to the counterfactual context. The Rubin Causal Model example demonstrates this process elegantly by
showing that potential outcomes are really just a function of the context.

## Conclusion

Together, these updates make DeepCausality a more powerful and expressive tool for modeling and reasoning about complex,
dynamic, and context-aware causal systems.

Get Started with DeepCausality 0.8. The Future is Now!

* Explore the [code examples on GitHub](https://github.com/deepcausality-rs/deep_causality/tree/main/examples).
* Join the [community](https://www.deepcausality.com/community/).
* Join the [Discord Server](https://discord.gg/Bxj9P7JXSj).

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and
deterministic context-aware causal reasoning in Rust. Please give us
a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community and drives open source
innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all members of
the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).
