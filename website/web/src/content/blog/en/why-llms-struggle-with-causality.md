---
title: "Why Do LLMs Struggle With Causality?"
description: "Large language models are next-token predictors over text distributions. Causality requires intervention and counterfactuals. The two are structurally different."
date: 2026-05-26
author: Marvin Hansen
tags:
  - llm
  - causality
  - causal-inference
  - counterfactuals
  - pearl-hierarchy
  - machine-learning
  - deep-causality
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

**Short answer.** Large language models are next-token predictors trained on text corpora. Their internal representation is a joint distribution over token sequences, which is statistical. Causality is a property of *mechanism*: which variable depends on which, under intervention. These are different mathematical objects. An LLM can produce text that reads like causal reasoning because such text appears in its training data, but it cannot perform causal inference on a novel structure.

Ask a large language model whether the rooster causes the sunrise, and it will tell you no. Ask it to reason about a five-step causal chain in a domain it has not been trained on, and it will fail in ways that look subtle until you check the work.

The first kind of question is a pattern recall. The second is a structural test. LLMs do the first well because the answer appears many times in their training data. They do the second badly because causality is not a pattern. It is a structural property of how interventions propagate, and pattern matching is exactly the wrong tool for it.

## What does causal reasoning actually require?

Judea Pearl's hierarchy is the cleanest way to frame this. Three rungs, in order of increasing difficulty:

1. **Association.** *What is, given what is observed?* P(Y | X). This is statistics. It is what every supervised learning system does.
2. **Intervention.** *What would happen if I did this?* P(Y | do(X)). This is what controlled experiments measure. It requires a model of the world, not a distribution over observations.
3. **Counterfactual.** *What would have happened if things had been different, given that this is what actually happened?* P(Y_x | X', Y'). This is what humans use to assign blame, plan, and learn from individual experiences.

An LLM trained on next-token prediction operates at rung one. The training objective is association, optimized over a very large corpus. The model learns which tokens co-occur with which other tokens, and at scale, this looks like reasoning because the corpus contains a lot of human reasoning that the model can complete. But auto-completing a sentence that contains causal language is not the same as performing causal inference. The model is matching the surface form of an answer, not deriving the answer itself.

## Where does the limitation surface?

Three places where the limitation surfaces predictably.

**Counterfactuals in novel scenarios.** Ask an LLM: "Suppose Marie Curie had been born in Brazil instead of Poland. Would she still have discovered radium?" The model will produce a fluent response. The response will be a plausible essay about access to research institutions, cultural context, and historical contingency. It will not be a counterfactual computation. The model is generating text that reads like counterfactual reasoning because such text exists in the corpus, not because the model is reasoning counterfactually.

**Chains with hidden confounders.** Give an LLM a five-variable causal scenario with a hidden common cause, and ask it to infer the structure. The model will pattern-match on surface cues (which variable was mentioned first, which words suggest causation) rather than perform the structural inference. Pearl's original ladder paper has examples of this; recent benchmarks like CausalBench and CLadder have systematic ones. Performance degrades sharply as the structure deviates from canonical textbook examples.

**Regime changes.** This is the most consequential failure. The causal structure of a system changes when the system crosses a regime boundary. Low interest rates cause stock prices to rise during normal markets. During a crisis, fear causes all assets to fall regardless of interest rates, and the prior relationship inverts. An LLM trained only on data from normal markets will be spectacularly wrong when the regime changes into a bear market, where stock prices trend downwards most of the time. The structural reason is detailed in [Why Correlation Breaks Under Regime Change](/blog/why-correlation-breaks-under-regime-change/).

## Why this is structural, not a tuning problem

The fundamental issue is that an LLM represents the world as a joint distribution over token sequences. Causality requires a representation of *mechanism*: which variable depends on which, in what direction, under which intervention. These are different objects.

A joint distribution can be projected from many different causal structures. Statistically, "the rooster crowed and then the sun rose" looks identical to "the sun rose and the rooster crowed because the morning light woke it." Both produce the same joint distribution over the two events. Only an interventional or counterfactual experiment distinguishes them. Crow at midnight; observe whether the sun rises.

An LLM cannot run that experiment. It can only sample from the distribution it was trained on, which contains both interpretations weighted by their frequency in the corpus, and produce text consistent with that distribution. The model has no representation that distinguishes the two structures because the distinguishing experiment is not in the training data.

A larger training corpus contains more of both interpretations, in similar relative frequencies. The distinguishing experiment is rare in any corpus, because in the real world we mostly do not run controlled experiments on roosters. So the model trained on a bigger corpus is not closer to causality; it is closer to the population distribution of human writing about causality, which is itself rung-one. For the parallel argument about scientific reasoning, see [Why LLMs Can't Do Physics](/blog/why-llms-cant-do-physics/).

## What does an actually causal system look like?

A causal computational system carries three things:

**Explicit dependency.** Each computation declares its inputs and its outputs. The dependency graph is the model of mechanism. In the DeepCausality framework, this is expressed as a chain of Causaloids and causal monads; each takes a propagating effect and returns one, and the chain itself is the causal structure.

**Intervention as a first-class operation.** You can replace a value at any node and propagate the consequences. This is Pearl's do-operator, encoded directly. In DeepCausality, the `intervene()` method on a propagating-effect chain forces a value at a node and re-propagates the rest, preserving the audit log so the comparison against the factual run is exact.

**Counterfactual comparison.** Because intervention is supported, counterfactual queries reduce to two runs of the same chain: one factual, one with the intervention applied. The causal effect is the difference.

None of these properties can be retrofit onto a network that was trained on next-token prediction, because the training objective does not require any of them. The model is not failing to do causal reasoning; it was never built to do causal reasoning. It was built to predict tokens, and it does that well. Asking an LLM to do causal analysis is a bit like asking a submarine to fly across a continent. Complaining that the submarine cannot fly misses the point. It is the wrong tool for the job. The converse is equally true: using a causal substrate for pattern matching or next-token prediction is also the wrong tool.

## The hybrid architecture that works

There is a useful way to combine the two. An LLM is good at unstructured extraction: reading text, summarizing, generating hypotheses, mapping natural language to schemas. A causal chain is good at structured propagation: applying mechanism, intervening, comparing factual against counterfactual.

The architecture is to let the LLM sit inside one node of a causal chain, as a bounded oracle. The LLM extracts a value from a text input; the value is wrapped in a propagating effect; the effect is handed to the next node, which is deterministic. If the LLM hallucinates, the next node rejects the value, and the chain fails loudly with an audit trail rather than silently with a confident wrong answer.

## Closing thoughts

LLMs struggle with causality because causality is not a property of token distributions. It is a property of mechanism, intervention, and counterfactual comparison. None of these are accessible from the inside of a next-token predictor.

The fix is a substrate that supports causal operations natively, with the LLM contained inside it as a useful but bounded component. The substrate refuses to propagate values that violate the chain's structure, and that refusal is what makes the system causal.

Further reading: [Why Is Correlation Not Causation?](/blog/why-is-correlation-not-causation/) · [Why Is Distribution Shift a Problem in AI?](/blog/why-is-distribution-shift-a-problem-in-ai/) · [Why LLMs Can't Do Physics](/blog/why-llms-cant-do-physics/)

## About DeepCausality

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast, deterministic, context-aware causal reasoning in Rust. The project is hosted at the Linux Foundation for AI & Data. Please give us a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).
