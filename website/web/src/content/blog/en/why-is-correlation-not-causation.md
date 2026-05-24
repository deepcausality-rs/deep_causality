---
title: "Why Is Correlation Not Causation? A Structural Explanation"
description: "Correlation is not causation because two variables can move together for four distinct reasons. Only one is causal. Here is the precise structural explanation."
date: 2026-05-20
author: Marvin Hansen
tags:
  - causality
  - causal-inference
  - correlation
  - causation
  - statistics
  - machine-learning
  - deep-causality
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

**Short answer.** Correlation means two variables move together. Two variables can move together for several reasons. They might share a common cause. They might be linked through an intervening variable. The apparent link might be an artifact of how the data was sampled. The relationship might even reverse direction when the regime changes. None of these is causation.

## What does "correlation is not causation" actually mean?

The phrase is a warning about a specific inferential mistake: concluding that one variable causes another because the two are statistically associated. The warning is correct, but the slogan obscures *why* it is correct. The deeper reason is that correlation and causation are answers to different questions.

- **Correlation** answers: *Given what I observe, how do these two variables co-vary?* This is computed from data.
- **Causation** answers: *If I intervened on one variable, what would happen to the other?* This is a claim about a hypothetical experiment, not about observed data.

The two questions can have different answers, and frequently do. A correlation can exist without causation, and (in unusual cases) causation can exist without observable correlation. The conflation of the two is the central error of correlation-based reasoning.

## The four ways correlation appears without causation

There are exactly four mechanisms by which two variables can be correlated. Only one of them is direct causation. The other three are the reasons the slogan exists.

```mermaid
flowchart LR
  subgraph A["1. Direct causation"]
    A1[X] --> A2[Y]
  end
  subgraph B["2. Reverse causation"]
    B1[X] <-- causes --- B2[Y]
  end
  subgraph C["3. Common cause"]
    C1[Z] --> C2[X]
    C1 --> C3[Y]
  end
  subgraph D["4. Selection bias"]
    D1[X] --> D2[(S)]
    D3[Y] --> D2
  end
```

### 1. Direct causation (X causes Y)

The cleanest case. Pressing the accelerator increases speed. Correlation exists because intervention on X changes Y.

### 2. Reverse causation (Y causes X)

The arrow is the other way. Hospitals correlate with sickness because sick people go to hospitals, not because hospitals cause sickness. Confusing the direction is one of the most common errors in complex causal systems.

### 3. Common cause (Z causes both X and Y)

A third variable causes both observed variables. Ice cream sales correlate with drowning deaths, not because ice cream causes drowning, but because both increase in summer. The summer (Z) is the common cause. This pattern is also called *confounding*.

### 4. Selection bias (the sample is conditioned on something)

The data were collected in a way that induces a correlation that does not exist in the population. Hospital admissions over-represent severe cases of two unrelated conditions, making the two conditions appear correlated within the hospital. The correlation is real in the sample and spurious in the population.

A correlation observed in data is consistent with all four mechanisms. The data alone do not distinguish them. Distinguishing them requires either an intervention (perform an experiment, change X and observe Y) or a structural assumption about the causal graph (encoded as a DAG, with the relevant independencies).

## Why this matters in practice

The cost of confusing correlation with causation is asymmetric. A weak correlation that you treat as causal will produce policy or product decisions that fail when the underlying mechanism does not behave as the correlation suggested.

Examples that have caused real harm:

- **Hormone replacement therapy.** Observational studies showed lower heart-disease rates in women taking HRT. The correlation was driven by selection (women who took HRT were on average healthier in many other ways). Randomized trials later showed HRT slightly increases cardiovascular risk. Treating the observational correlation as causal led to a decade of medical practice in the wrong direction.
- **School class size.** Smaller classes correlate with better outcomes. Some of this is causal; much of it is confounded by parental income, school funding, and selection effects. Reforms based on the raw correlation under-deliver.
- **Algorithmic decisions.** Predictive models in lending, hiring, and criminal justice frequently encode correlations driven by historical confounders. Deployed as if they were causal predictors, they reproduce and amplify the original confounding.

In each case the inferential error is the same: treat a number computed from data (correlation) as if it were a property of mechanism (causation). The two are not the same, and the cost of treating them as the same lands on whoever is most exposed to the resulting decisions.

## How do you tell the difference between correlation and causation?

Three approaches help to identify the difference between correlation and causation:

**Randomized controlled experiment.** Assign treatment at random, observe outcomes. Randomization breaks confounding by construction: there is no common cause of treatment assignment and outcome because the assignment was random. This is the gold standard. It is also expensive, sometimes unethical, and often impossible.

**Causal graph plus the do-calculus.** Specify the causal structure you believe holds (which variables cause which), and use Pearl's do-calculus to determine whether the causal effect of interest is identifiable from observational data. If it is, compute it. If it is not, the data alone cannot answer the causal question, regardless of how much data you have.

**Structural model of the mechanism.** Build a system that encodes the mechanism directly: which variable depends on which, through what process, under what conditions. The model is not derived from data; it is derived from prior knowledge of the system. Data can then be used to estimate parameters within the model.

The third approach is what causal-substrate systems do. In the DeepCausality framework, a causal chain is a structural model of mechanism. Each node encodes a specific process; the chain encodes how processes compose and draw causal conclusions.

## How do you discover causal structure from data?

It can happen that there is no complete prior knowledge of causal structure. When sufficient data are available, the DeepCausality project provides a [discovery crate](/blog/announcement-causal-discovery/) that contains state-of-the-art algorithms for recovering causal structure among variables. This directly supports the design of data pipelines that reason about the causal impact of one set of variables on another set of variables that represent the state of a system.

In the event that there are no data or clearly insufficient data available, it is necessary to organize data collection first. In that case, careful hypothesis testing becomes necessary to ensure that the actual causal relationship is captured in the data. Counterfactual experiments help to identify whether a set of variables should be recorded. For example, if variable A is suspected to cause X, a counterfactual experiment tests whether X fails to occur when A does not occur. Whenever that test fails repeatedly (X occurs even when A does not), A and X are merely correlated and A is not a cause of X.

## Why machine learning makes this confusion easier to make

A trained machine-learning model is a function from inputs to outputs that performs well on the training distribution. Internally, the model is a correlation engine. It encodes which combinations of inputs co-occur with which outputs. The performance metric (accuracy on a test set) measures fit to a distribution regardless of reality.

For example, if an LLM proclaims "If you increase the marketing spend by Y, sales will increase by X," it sounds like a causal claim. The model has produced a number consistent with the joint distribution observed in its training data, in which marketing spend and sales were jointly distributed for a host of reasons (some causal, some confounded). The user does not know where the majority of that training data came from, so there is no guarantee that any of those correlations between marketing spend and actual sales transfer to the user's specific business. Quarterly and annual filings with the SEC, for example, contain marketing and sales numbers and percentage changes that hold for a typical publicly listed U.S. enterprise.

SEC filings are publicly available and generally of high quality, so they were almost certainly included in the training data of large language models. The same correlation between marketing spend and sales conversion that works for a typical U.S. enterprise may not apply to a small business in a rural area outside the United States.

This is the structural reason that machine-learning systems fail under regime change and distribution shift. The impact of marketing spend on sales may hold on average within one industry and fail entirely in a different industry with a different sales mechanism. When the regime moves (a different industry, or a different company in a different country), the correlations break, and the model has no representation of mechanism to know that it has failed.

For more details on the underlying technical reasons, please see [Why Correlation Breaks Under Regime Change](/blog/why-correlation-breaks-under-regime-change/) and [Why Is Distribution Shift a Problem in AI?](/blog/why-is-distribution-shift-a-problem-in-ai/).

## The architectural answer

A system that does not confuse correlation with causation has three properties:

1. **An explicit causal structure.** Variables and their dependencies are declared, not learned from co-occurrence statistics.
2. **A first-class intervention operator.** The system can answer "what would happen if I forced X to value v" without re-fitting the model.
3. **Counterfactual support.** Factual and counterfactual runs can be compared exactly, because intervention preserves everything except the intervened value.

The DeepCausality framework implements these as a propagating-effect monad with a `do()`-style intervention operator. The substrate is structural; correlation-based components, including LLMs, can be embedded as bounded oracles inside the structure. The longer technical treatment is in [Why LLMs Can't Do Physics](/blog/why-llms-cant-do-physics/) and the companion piece [Why LLMs Struggle With Causality](/blog/why-llms-struggle-with-causality/).

## About DeepCausality

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast, deterministic, context-aware causal reasoning in Rust. The project is hosted at the Linux Foundation for AI & Data. Please give us a [star on GitHub](https://github.com/deepcausality-rs/deep_causality).
