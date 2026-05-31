---
title: Causal Discovery Algorithms
description: SURD and MRMR, the two algorithms in deep_causality_algorithms that the Causal Discovery Language uses to go from raw observational data to an executable causal model.
sidebar:
  order: 13
  label: Discovery Algorithms
---

The [`deep_causality_algorithms`](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_algorithms) crate ships the two algorithms the [Causal Discovery Language](/concepts/cdl/) wires together to turn raw observational data into a discovery report that informs the construction of a causal model. They are independent and individually useful; they cover complementary halves of the discovery problem: **which variables matter (MRMR)** and **how they interact (SURD)**.

## MRMR: Maximum Relevance, Minimum Redundancy

MRMR is a filter-based feature-selection method from Zhao, Anand, and Wang (Uber, 2019), originally developed for industrial-scale machine-learning pipelines but well suited to causal discovery as a pre-step. Given a target variable and many candidate features, MRMR ranks features by two competing pressures:

- **Maximum relevance**: pick features that are individually informative about the target.
- **Minimum redundancy**: penalize features that mostly repeat what already-selected features tell you.

The result is a compact, informative feature subset that retains most of the signal a richer set carries, at a fraction of the discovery cost. As a filter method, relative to wrappers and embedded methods, MRMR is fast, model-agnostic, and reusable across pipelines.

**When to reach for it:** the data has more candidate features than the discovery step can reasonably search, and you want the discovered model to be both tractable and interpretable. MRMR is the CDL pipeline's standard pre-discovery stage.

**Reference paper:** [arXiv:1908.05376](https://arxiv.org/abs/1908.05376), *Maximum Relevance and Minimum Redundancy Feature Selection Methods for a Marketing Machine Learning Platform*.

## SURD: State-and-interaction-type causality

SURD (Synergistic / Unique / Redundant Decomposition) is the discovery method from Martínez-Sánchez and Lozano-Durán. It quantifies causality as a function of **system state and interaction type**, decomposing the influence one variable has on another into three components:

- **Synergistic**: the joint contribution of multiple sources that only appears when they act together.
- **Unique**: the part each source contributes that no other source can replace.
- **Redundant**: the part that multiple sources carry equivalently.

Traditional methods (Convergent Cross Mapping, PCMCI, Granger causality, conditional independence tests) report **a single average causal strength** across all states of the system. SURD breaks that single number into the three components and tracks them *per state*; two variables that look similar on average but differ in their synergistic content are no longer mistaken for the same relation. The state-conditional view is the natural match for the EPP's dynamic-causality model: causal structure that varies with the state of the system is exactly what the framework is built to represent.

**When to reach for it:** the system is dynamic, the relationships are likely non-linear or state-dependent, and you want to distinguish redundant signals from genuinely additive (synergistic) ones rather than averaging them away.

**Reference paper:** [arXiv:2505.10878](https://arxiv.org/abs/2505.10878), *Observational causality by states and interaction type for scientific discovery*.

## How they compose in the CDL pipeline

The [Causal Discovery Language](/concepts/cdl/) is the typestate-builder pipeline that orchestrates the two: data load → clean → **MRMR feature selection** → **SURD causal discovery** → analysis → finalize. Each stage advances the typestate so misuse is rejected at compile time. The output is a `CdlReport` whose recommendations (for example, "Strong unique influence: Recommended Direct edge in `CausaloidGraph`") tell you which Causaloids to wire and how; you then construct the `CausaloidGraph` from those findings using the rest of the framework.

Either algorithm is usable on its own. MRMR is independently useful as a general feature-selection primitive. SURD is independently useful wherever you want a state-conditional decomposition of causal interactions rather than a scalar score.

## See also

- [Causal Discovery Language](/concepts/cdl/): the typestate pipeline that wires both algorithms.
- [Causaloid](/concepts/causaloid/): the output of the discovery step, the unit the rest of the framework composes over.
- [Uncertainty](/concepts/uncertainty/): the `Uncertain<T>` type for downstream propagation under noise.
- Reference: [`deep_causality_algorithms` crate](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality_algorithms).
