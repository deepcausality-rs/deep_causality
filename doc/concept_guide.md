# Concepts

## Origin

The [original design](/doc/swift/Swift_Inference.ipynb) was done in Swift using protocols and
extensions inspired by [differentiable types](https://github.com/tensorflow/swift/blob/main/docs/DifferentiableTypes.md)
For a number of reasons, a re-write from scratch became necessary and eventually Rust was
chosen.

## Terminology

Deep causality's implementation relies heavily on generic protocols and extension traits, which may
need a bit of elaboration.

### Protocols vs Traits

In Swift, protocols can contain multiple abstract members, methods, and types.
Structs and enums can conform to multiple protocols
and the conformance relationship can be established retroactively.
See [Protocol-oriented programming & generics](https://colab.research.google.com/github/tensorflow/swift/blob/main/docs/site/tutorials/protocol_oriented_generics.ipynb)
for more details.

Rust uses traits instead of protocols but there are some minor differences:

1) Swift protocols can contain values, traits cannot. In Rust you can use abstract getters instead to access values in
   default implementation.
2) Swift protocols use extensions to add a default implementation. Rust, however, uses the defining trait or a
   sub-trait.
3) Swift protocols allow overriding default implementations, but this prohibits dynamic dispatching. In Rust, trait
   objects would suit this scenario.

### Extensions

Both, Swift protocols and Rust traits allowing the extension of external libraries (i.e. std lib).
Swift uses
the [extension keyword](https://colab.research.google.com/github/tensorflow/swift/blob/main/docs/site/tutorials/protocol_oriented_generics.ipynb#scrollTo=c_Xmw5cDy_rZ&line=1&uniqifier=1)
to
add additional functionality to an existing type whereas Rust uses
the [Extension Trait](http://xion.io/post/code/rust-extension-traits.html) design pattern.

Extension traits mean that you implement a local trait for an external type. In Rust, however,you cannot implement
an external trait for an external type.

## Concepts

The core concepts implemented in deep causality derive from "Theoretical Impediments to Machine
Learning" ([Perl,2018](https://arxiv.org/abs/1801.04016))

### 1) Encoding Causal Assumptions

Explicit assumptions about the underlying data, pattern, and structures enable transparency and testability.
More profoundly, transfer learning is one of the important advancements in machine learning that allows
relatively easy adoption of existing models without the time consuming process of re-learning the entire model.

However, in practice transfer-learning requires model-fitting, which means re-training the outer layers
of a neuronal net because of the absence of verifiable assumptions that would answer the question of whether the
model is even transferable w.r.t. to the underlying data. That is another way of asking whether the data distribution
of the model trainings data resembles the data distribution of the data of the targeted transfer learning.

Deep causality implements verifiable assumptions as following:

### Assumptions

Traits:

* [Assumable](/src/protocols/assumable/assumable.rs)
* [AssumableReasoning](/src/protocols/assumable/assumable_reasoning.rs)

Extensions:

* [Assumable Array](/src/extensions/assumable/assumable_array.rs)
* [Assumable Map](/src/extensions/assumable/assumable_map.rs)
* [Assumable Vector](/src/extensions/assumable/assumable_vector.rs)

Types:
* [Assumption](/src/types/reasoning_types/assumable/assumption.rs)
* [EvalFn](/src/types/alias_types/mod.rs)

At its core, the assumption type explicitly encodes an assumption in a textual description and an
an eval function that takes a slice of numerical values as an argument and returns a boolean as to which
the assumption holds true on the given data. The implementation of the assumable trait adds functionality
to test the assumption, check if its already been tested, and if its already been valid.

Multiple assumptions are represented in standard collections (array, map, vector) which are extended with
the default implementation of the [AssumableReasoning](/src/protocols/assumable/assumable_reasoning.rs) trait
that adds aggregating functionality i.e. how many of the assumptions are valid, tested etc.

### Observation

Once a set of assumptions has been identified and encoded, the next question is whether some or
all of the assumptions actually exists in the data? Deep causality implements observations as following:

Traits:

* [Observable](/src/protocols/observable/observable.rs)
* [ObservableReasoning](/src/protocols/observable/observable_reasoning.rs)

Extensions:

* [Observable Array](/src/extensions/observable/observation_array.rs)
* [Observable Map](/src/extensions/observable/observation_map.rs)
* [Observable Vector](/src/extensions/observable/observation_vector.rs)

Types:
* [Observation](/src/types/reasoning_types/observable/observation.rs)

An observation defines an observed value i.e. a measured metric and  an observed effect. 
The idea is to hold the observation immutable and invariant after the observation. 
The assumable protocol then adds a default implementation to determine if a target effect 
has been observed relative to the observation in excess of a threshold. Both, the target effect and
target threshold are given as a parameter because it may happen that certain effects may only become 
detectable when adjusting the threshold.

Multiple observations are stored in standard collections (array, map, vector) which are extended with
[ObservableReasoning](/src/protocols/observable/observable_reasoning.rs) to identify the number or percent of
observations vs non-observations.

###  2) Control of confounding

