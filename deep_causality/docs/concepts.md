[//]: # (---)
[//]: # (SPDX-License-Identifier: MIT)
[//]: # (---)

# Concepts

## Origin

The [original design](/deep_causality/docs/swift/Swift_Inference.ipynb) of deep causality was implemented in Swift using protocols and
extensions inspired
by [differentiable types](https://github.com/tensorflow/swift/blob/main/docs/DifferentiableTypes.md).
A rewrite became necessary for several reasons, and Rust was chosen.

## Terminology

Deep causality's implementation uses heavily generic protocols and extension traits, briefly elaborated below.

### Protocols vs. Traits

In Swift, protocols can contain multiple abstract members, methods, and types.
Structs and enums can conform to multiple protocols and the conformance relationship can be established retroactively.
See [Protocol-oriented Programming & Generics](https://colab.research.google.com/github/tensorflow/swift/blob/main/docs/site/tutorials/protocol_oriented_generics.ipynb)
for more details.

Rust uses traits instead of protocols but there are some minor differences:

1) Swift protocols can contain values, traits cannot. In Rust you can use abstract getters instead to access values in
   default implementation.
2) Swift protocols use extensions to add a default implementation. Rust, however, uses the defining trait or a
   sub-trait.
3) Swift protocols allow overriding default implementations, but this prohibits dynamic dispatching. In Rust, trait
   objects would suit this scenario.

### Extensions

Both Swift protocols and Rust traits allow the extension of external libraries (i.e., std lib).

Swift uses
the [extension keyword](https://colab.research.google.com/github/tensorflow/swift/blob/main/docs/site/tutorials/protocol_oriented_generics.ipynb#scrollTo=c_Xmw5cDy_rZ&line=1&uniqifier=1)
to add additional functionality to an existing type whereas Rust uses
the [Extension Trait](http://xion.io/post/code/rust-extension-traits.html) design pattern.

Extension traits mean that you implement a local trait for an external type. In Rust, however,you cannot implement
an external trait for an external type.

## Core Concepts

The core concepts implemented in deep causality derive from "Theoretical Impediments to Machine
Learning" ([Pearl,2018](https://arxiv.org/abs/1801.04016))

### 1) Encoding Causal Assumptions

Explicit assumptions about the underlying data, patterns, and structures enable transparency and testability.
More profoundly, transfer learning is one of the critical advancements in machine learning that allows
relatively easy adoption of existing models without the time-consuming process of re-learning the entire model.

In practice transfer-learning requires model-fitting, which means re-training the outer layers
of a neuronal net because of the absence of verifiable assumptions that would answer whether the
model is even transferable w.r.t. to the underlying data. That is another way of asking whether the data distribution
of the model training data resembles the data distribution of the data of the targeted transfer learning.
Deep causality implements verifiable assumptions in the following way:

### Assumptions

Traits:

* [Assumable](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/protocols/adjustable/mod.rs)
* [Assumable Reasoning](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/protocols/adjustable/mod.rs)

Extensions:

* [Assumable Array](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/extensions/assumable/mod.rs)
* [Assumable Map](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/extensions/assumable/mod.rs)
* [Assumable Vector](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/extensions/assumable/mod.rs)

Types:

* [Assumption](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/types/reasoning_types/assumable/assumption.rs)
* [EvalFn](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/types/reasoning_types/assumable/mod.rs)

The assumption type explicitly encodes an assumption in a textual description and an eval function that takes a slice of
numerical values as an argument and returns a boolean for which the assumption holds on the given data. Implementing the
assumable trait adds functionality to test the assumption, check if it has already been tested, and if it has already
been valid.

Multiple assumptions are represented in standard collections (array, map, vector), which are extended with
the default implementation of the [AssumableReasoning](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/protocols/assumable/mod.rs) trait
that adds aggregating functionality i.e. how many of the assumptions are valid or tested.

### Observation

Once a set of assumptions has been identified and encoded, the next question is whether some or all assumptions exist in
the data. Deep causality implements observations in the following way:

Traits:

* [Observable](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/protocols/observable/mod.rs)
* [ObservableReasoning](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/protocols/observable/mod.rs)

Extensions:

* [Observable Array](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/extensions/observable/mod.rs)
* [Observable Map](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/extensions/observable/mod.rs)
* [Observable Vector](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/extensions/observable/mod.rs)

Types:

* [Observation](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/types/reasoning_types/observable/observation.rs)

An observation defines an observed value i.e., a measured metric and an observed effect.
The idea is to hold the observation immutable and invariant after the observation.
The assumable protocol then adds a default implementation to determine if a target effect
has been observed relative to the observation in excess of a threshold. Both, the target effect and
target threshold are given as a parameter because it may happen that certain effects may only become
detectable when adjusting the threshold.

Multiple observations are stored in standard collections (array, map, vector) which are extended with
[ObservableReasoning](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/protocols/observable/mod.rs) to identify the number or percent of
observations vs non-observations.

### 2) Control of confounding

Confounding refers to the presence of unobserved causes of two or more variables. In the presence of
confounding, false conclusions can easily be drawn from data. Existing approaches address confounding
through either graphical modeling using the back-door technique or through the application of the do-calculus.
Both methodologies are effective in de-confounding causality models.

Deep Causality adds the concept of inference to discern whether the stipulated cause actually occurs
in tandem with the expected observation to decide whether one can infer the expected effect from that cause.

Traits:

* [Inferable](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/protocols/inferable/mod.rs)
* [InferableReasoning](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/protocols/inferable/mod.rs)

Extensions:

* [Inferable Array](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/extensions/inferable/mod.rs)
* [Inferable Map](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/extensions/inferable/mod.rs)
* [Inferable Vector](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/extensions/inferable/mod.rs)

Types:

* [Inference](https://github.com/deepcausality-rs/deep_causality/blob/main/deep_causality/src/types/reasoning_types/inferable/inference.rs)

Deep Causality addresses confounding via a third approach called conjoint delta. For a single observation, the conjoint
delta refers to the difference between an expected and observed effect normalized to one. If the expected effect is
present, the conjoint delta of a singular observation is zero because
the stipulated cause occurs together with the expected effect. Conversely, if the cause is present, but the expected
effect is missing, the conjoint delta is one because the cause does not explain the observed effect.

For a collection of observations, the conjoint delta refers to the difference (delta) between all observations and those
where the effect is not observed together with the cause, again normalized to one. For example, suppose a set of ten
observations contains eight observations where the expected effect occurs. In that case, the conjoint delta is .2 or
20%, meaning in 20% of the observed data, an unknown or unobserved cause must cause the observed effect.

In practice, however, one might try to minimize the conjoint delta to reign in confounding. Still, in many applications,
eliminating confounding might not be feasible. Instead, it's sensible to establish a certain threshold at which
reasoning and inference can proceed with the understanding that the conjoint delta captures everything not explained by
the causal model.

### 3) Counterfactuals

Counterfactual analysis deals with behavior of specific individuals, identified by a distinct set of characteristics.
Counterfactuals analysis falls broadly into two categories; the process of determining analytically if the probability
of a factual sentence is estimable from experimental or observational studies, or combination thereof. And, second,
counterfactual questions concerning “causes of effects” as opposed to effects of causes. Lets suppose Joe died
in the water during a swimming exercise, with his death as factual effect, a counterfactual question would be,
would Joe be alive if he would have not taken part of the swimming exercise? In other words,
was the swimming exercise a cause of Joe’s death?

For the first category of counterfactuals, Deep Causality provides the inferable protocol as an alternative to the
established methods to determine if a a factual sentence is estimable from experimental or observational data.

For the second category, however, Deep Causality does not provide a solution as this topic is still subject
to ongoing research.

### 4) Mediation Analysis and the Assessment of Direct and Indirect Effects

Mediation analysis concerns the mechanisms that transmit changes from a cause to its effects.
The identification of such intermediate mechanism is essential for generating explanations.
Typical queries answerable by this analysis are: What fraction of the effect of X on Y is mediated by variable Z.

Deep Causality offers a novel mechanism of mediation analysis called conjoint delta as explained in the previous section
about confounding. The difference between one and the conjoint delta quantifies the exact direct contribution of a cause
to an effect. That means, for a multi-causality, it is decidable how much each cause contributes to the observed effect.
Furthermore, in a multi layer causality, sectional conjoint-delta refers to the relative indirect impact of a cause
on the final effect.

### 5) External Validity and Sample Selection Bias

The validity of every experimental study is challenged by disparities between the experimental and implementation
setups. A machine trained in one environment cannot be expected to perform well when environmental conditions change,
unless the changes are localized and identified. This problem, and its various manifestations are well recognized.

Deep Causality addresses this problem with the encoding of explicit assumption, as elaborated in section 1.

A few implications follow from causality models that are conditional on explicit assumptions:

1) Applicability of a model to a new dataset only requires testing the assumptions required by the model.
2) The relative impact of a change of context can be assessed relative to how the change affects the models assumptions.
3) Transfer learning, from one domain to another, becomes easier in the sense transfer is fundamentally possible
   whenever the assumptions hold true.

In terms of selection bias, the causal model usually is is free of a bias unless explicitly stated in the assumptions.

Currently, Deep Causality cannot verify whether a causal function depends on unstated assumptions but rather relies
on the model designer to make any assumption explicit. This is a tradeoff that follows from the decision to
encode the causal relation as a generic function, which cannot easily be verified w.r.t to implicit assumptions.
This is one of the areas where more work is needed.

### 6) Missing Data

Problems of missing data plague every science project. Respondents do not answer every question,
sensors fade, and patients often drop from a clinical study. The rich literature provides multiple
techniques of dealing with missing data, often applying techniques such as substituting missing values
either with an average value or a certain default value. These techniques assume an unknown causal process
of data generation and therefore an indiscriminate handling of missing data is applied. This is a direct result
of the model blind paradigm prevalent in all statistic based machine learning methods.

Deep Causality, similar to related work, allows the modeling of the data generation process as
a separate causal model with the implication that, within limits, causal relationships can be recovered from incomplete
data.

### 7) Causal Discovery

Existing work from other research groups can detect and enumerate the testable implications of a given causal model.
This opens the possibility of inferring, with mild assumptions, the set of models that are compatible with the data.

Deep Causality, however, does not provide such a mechanism by default. However, what can be done with Deep Causality
is to run all assumptions of all models against a set of data to determine which models are applicable to the data.
While this accomplishes something similar, this is a rather experimental approach without formalization and
requires that all models have explicit and testable assumptions. 
