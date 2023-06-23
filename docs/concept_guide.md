# Concepts

## Origin

The [original design](/docs/swift/Swift_Inference.ipynb) of deep causality was implemented in Swift using protocols and
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

## Concepts

The core concepts implemented in deep causality derive from "Theoretical Impediments to Machine
Learning" ([Perl,2018](https://arxiv.org/abs/1801.04016))

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

* [Assumable](/src/protocols/assumable/mod.rs)
* [Assumable Reasoning](/src/protocols/assumable/mod.rs)

Extensions:

* [Assumable Array](/src/extensions/assumable/mod.rs)
* [Assumable Map](/src/extensions/assumable/mod.rs)
* [Assumable Vector](/src/extensions/assumable/mod.rs)

Types:

* [Assumption](/src/types/reasoning_types/assumable/assumption.rs)
* [EvalFn](/src/types/alias_types/mod.rs)

The assumption type explicitly encodes an assumption in a textual description and an eval function that takes a slice of
numerical values as an argument and returns a boolean for which the assumption holds on the given data. Implementing the
assumable trait adds functionality to test the assumption, check if it has already been tested, and if it has already
been valid.

Multiple assumptions are represented in standard collections (array, map, vector), which are extended with
the default implementation of the [AssumableReasoning](/src/protocols/assumable/mod.rs) trait
that adds aggregating functionality i.e. how many of the assumptions are valid or tested.

### Observation

Once a set of assumptions has been identified and encoded, the next question is whether some or all assumptions exist in
the data. Deep causality implements observations in the following way:

Traits:

* [Observable](/src/protocols/observable/mod.rs)
* [ObservableReasoning](/src/protocols/observable/mod.rs)

Extensions:

* [Observable Array](/src/extensions/observable/mod.rs)
* [Observable Map](/src/extensions/observable/mod.rs)
* [Observable Vector](/src/extensions/observable/mod.rs)

Types:

* [Observation](/src/types/reasoning_types/observable/observation.rs)

An observation defines an observed value i.e., a measured metric and an observed effect.
The idea is to hold the observation immutable and invariant after the observation.
The assumable protocol then adds a default implementation to determine if a target effect
has been observed relative to the observation in excess of a threshold. Both, the target effect and
target threshold are given as a parameter because it may happen that certain effects may only become
detectable when adjusting the threshold.

Multiple observations are stored in standard collections (array, map, vector) which are extended with
[ObservableReasoning](/src/protocols/observable/mod.rs) to identify the number or percent of
observations vs non-observations.

### 2) Control of confounding

Confounding refers to the presence of unobserved causes of two or more variables. In the presence of
confounding, false conclusions can easily be drawn from data. Existing approaches address confounding
through either graphical modeling using the back-door technique or through the application of the do-calculus.
Both methodologies are effective in de-confounding causality models.

Deep Causality adds the concept of inference to discern whether the stipulated cause actually occurs
in tandem with the expected observation to decide whether one can infer the expected effect from that cause.

Traits:

* [Inferable](/src/protocols/inferable/mod.rs)
* [InferableReasoning](/src/protocols/inferable/mod.rs)

Extensions:

* [Inferable Array](/src/extensions/inferable/mod.rs)
* [Inferable Map](/src/extensions/inferable/mod.rs)
* [Inferable Vector](/src/extensions/inferable/mod.rs)

Types:

* [Inference](/src/types/reasoning_types/inferable/inference.rs)

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

