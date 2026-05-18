---
title: DeepCausality v.0.8 adds support for non-Euclidean context
description: This post summarizes the new feature of DeepCausality v.0.8
date: 2025-07-09
author: Marvin Hansen
draft: false
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

## Overview

The DeepCausality project announces the release of DeepCausality 0.8 that strengthens the core of the framework with
added async concurrency, added non-Euclidean context, added relative temporal index, an unified adjustable trait
implementation, and unified causal reasoning.

## 🚀 Highlights in 0.8

- **Async concurrency** full compatibility with Tokio
- **Added unified causal reasoning** for deterministic and probabilistic reasoning
- **Added non-Euclidean geometry** contexts
- **Added relative temporal index** for simplified handling of time graphs.
- **Unified `Adjustable` trait** implementation across all context types

## ⚡ Added support for Tokio & Async Rust

In DeepCausality 0.8, all Causaloids, Contextoids, and Model types are now able to be `Send` and `Sync`, enabling
concurrency and thus true parallel inference pipelines. See the
new [Tokio code example](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/tokio_example) for details about
how to build concurrent causal inference with DeepCausality.

## 💡 Added Unified Causal Reasoning

Previously, DeepCausality was limited to deterministic reasoning (true/false), which restricted its application in
complex dynamic systems. DeepCausality 0.8 introduces a uniform reasoning framework that supports deterministic,
probabilistic, and advanced non-numerical data-flow modalities. This enables hybrid models that combine multiple
reasoning modes within a single, consistent, and performant framework.

1) **The CausableReasoning Trait:**

Recognizing that not all causal models require the same level of complexity, the updated `CausableReasoning` trait
offers a tiered approach with distinct evaluation strategies. This allows you to choose the most efficient and
straightforward method for your specific use case:

* `evaluate_deterministic_propagation`: Provides a highly optimized path for strict, all-or-nothing causal chains where
  every link must resolve to true.
* `evaluate_probabilistic_propagation`: Designed for reasoning under uncertainty, this efficiently aggregates outcomes
  in purely probabilistic chains, for instance, by multiplying the probabilities of each link.
* `evaluate_mixed_propagation`: The most powerful and flexible strategy, this method implements a **dynamic data-flow
  engine**. It is designed to execute complex, heterogeneous chains with a mix of deterministic checks, probabilistic
  weightings, and data-forwarding via the new `ContextualLink` mechanism.

This tiered design provides both optimized functions for common scenarios and a robust engine for the most complex dynamic models.

2) **The PropagatingEffect:**

The **PropagatingEffect** enum represents the outcome of a causaloid's evaluation. While Deterministic and Probabilistic
are value-based outcomes, the ContextualLink variant is a powerful new directive. It instructs the next Causaloid to
locate the propagation effect at the linked Contextoid. The linked Contextoid itself becomes the `Evidence` for the next
step in the reasoning process. This is particular valuable in combination with the added support for non-Euclidean
context as this allows, for example, to natively reason over movement in spacetime using tangent Bundle spacetime
contextoids or reason whether a series of spacial operations propagated via quaternions fulfill certain criteria thus
allow for advanced constraints solver.

3) **Isomorphic Recursive Evidence**

The `Evidence` enum is a versatile container for data fed into a causal model. Its isomorphic, recursive structure
allows you to represent simple numerical inputs, complex collections, or entire sub-graphs of evidence using a single,
consistent API. This enables the construction of sophisticated, multi-layered causal models where the output of one
sub-graph can be transformed into an evidence graph and serves that input for another causal graph. It hold the
following variants:

* `Deterministic(bool)` - For classic causal inference
* `Numerical(NumericalValue)` - For numerical input i.e. sensor readings
* `Probability(NumericalValue` - When reasoning over potential outcomes
* `ContextualLink(ContextId, ContextoidId)` - For reasoning over complex, non-numerical data like geometric objects
  movement in spacetime.
* `Map(...)` - A hashmap of evidence uses in causal collections or graphs.
* `Graph(...)` - A hypergraph of evidence for usage in a causal graph.
* `None`: For gracefully handling missing or unreliable data.

NumericalValue is a type alias to f64 mainly for simplicity and performance. For use cases where f32 is deemed
sufficient, a simple update of the type aliases allows for a reduced memory footprint which may be advantageous on low
power embedded controllers.  
Combined, the CausableReasoning trait, the PropagatingEffect and the isomorphic recursive Evidence form the backbone of
the new unified causal reasoning in DeepCausality 0.8. The added support for non-Euclidean context takes the reasoning
capabilities one step further.

## 🌎 Added non-Euclidean Context Types

Previously, DeepCausality supported flat-space representations such as EuclideanSpace and EuclideanSpacetime. With 0.8,
DeepCausality gains native support for:

### 📍 Non-Euclidean Space Contexts

- EcefSpace - 3D Cartesian space relative to the Earth's center of mass
- NedSpace - A local tangent plane spatial context using the North-East-Down (NED) reference frame
- WGS84 Geospace - geodetic coordinates space
- QuaternionSpace — rotation-aware 3D orientation tracking

### 🪐 Non-Euclidean Spacetime Contexts

- Minkowski spacetime — special relativistic flat spacetime
- Lorentzian spacetime — general relativistic curved spacetime
- Tangent Bundle spacetime — position and velocity within local tangent spacetime

### 🧭 Advanced Time Representations

- Discrete time - represents discrete, uniformly spaced ticks
- Euclidean time semantically represents the rotated, imaginary-time axis as a real-valued scalar.
- Lorentzian time corresponds to the real-valued coordinate time used in special and general relativity
- Symbolic time models time points that are defined in terms of symbolic relationships.

**What Can You Do With The New Context Types?**

**Tangent Spacetime**
Combines spacetime position and velocity to model movement in a relativistic manifold. Enables local inertial modeling,
frame drift detection, and velocity-aware corrections in high-altitude avionics navigation.

**Lorentzian Spacetime**
Models gravitational effects, time dilation, or geometric distortion commonly adjusted for in Global Satellite
Navigation Systems (GNSS) such as GPS, Galileo, or GLONASS.

**Quaternion Space**
Quaternions efficiently model 3D orientation in space and are used for robotics, computer vision, and inertial
navigation systems.

DeepCausality 0.8 introduces the unique capability of flexible integration of heterogeneous geometries. For example, a
Euclidean context represents data in a Euclidean representation whereas additional contexts for QuaternionSpace or
LorentzianSpacetime represent non-Euclidean data. DeepCausality supports multiple contexts since 0.6 and, because of its
generic design, a QuaternionSpace context will be statically verified by the compiler to hold only QuaternionSpace data
thus preventing dangerous unit or geometry mismatches. This enables the fusion of different sensor data i.e. position
data, sensor data, and drift detection data with each data set residing in its respective geometric context.

In scenarios where data of different space, time, or spacetime representations have to reside within the same context,
DeepCausality offers three convenience types for uniform yet type-safe access to heterogeneous geometry data:

* SpaceKind
* TimeKind
* SpaceTimeKind

Each of those convenience types implements all relevant traits over an abstract algebraic data structure that gives
uniform access to the underlying types. For example, a contextoid of type SpaceKind gives uniform access to Euclidean,
Ecef, Ned, WGS84, and Quaternion space all from within the same context. Likewise, a contextoid of type SpaceTimeKind
gives uniform access to all spacetime types from within the same context.

All context and all convenience types implement the various traits of the extensive context trait systems, which allows
advanced users fine-grained customization. If you need a different spacetime distance metric, you can overwrite the
Metric trait. If you need a custom geometry, you can implement the relevant traits and plug your custom geometry types
into DeepCausality.

## ✈️ Added Relative Temporal Index to Context

A temporal hypergraph, by design, holds all past and present temporal values simultaneously within its structure. This
co-existence of multiple temporal points simplifies non-trivial temporal arithmetic over hetero-scaled time units, yet
it imposes a vexing problem: How do you know if a time value in a node of the graph is current or past?

The problem is non-trivial because, as time progresses, the context continuously generates the non-Euclidean temporal
hypergraph representation with the implication that, at one lookup, the value of a temporal node is current, but at the
next one, it might be past; however, the exact temporal distance at which a “current” value becomes “past” depends on
the node’s time scale.

DeepCausality 0.8 solves the problem by adding a relative temporal index to the context that allows to get or set the
current or previous time index relative to its time scale. The new CurrentTimeIndex and PreviousTimeIndex are optional,
but when imported provide a default implementation that allows to set scale depending time indexes.

For example, setting the index for the current day becomes as simple as calling the corresponding
_set_current_day_index_ method within the index. Once the index of the current day has been retrieved using the
_get_current_day_index_, the graph node holding the current day data can be retrieved from the context using the
existing _get_node_ methods from the context. Advanced users can overwrite the TimeIndexable, CurrentTimeIndex, and
PreviousTimeIndex to customize time indexing to specific requirements.

## 🗺️ Unified Adjustable Trait

Modeling dynamic systems requires contexts that can evolve. DeepCausality 0.8 introduces a simplified and more powerful
way to handle dynamic context data. All AdjustableXYZ types are gone for good. Now, you work with primary context types,
like EuclideanSpace, and when you need dynamic updates or adjustments then import the Adjustable trait and implemented
update() and adjust() methods become available on the original type. All Adjustable implementations for all context
types include rigorous validation and overflow protection, ensuring that your dynamic context modifications are safe,
robust, and reliable.

## ⚙️ Miscellaneous

- 95% test coverage across the entire codebase.
- The is_active, number_active, and percent_active API methods are gone for good as these became untenable in the new
  unified reasoning design.
- DeepCausality 0.8 updated to UltraGraph 0.8, which boost performance on a range of 12 benchmarks on average by 300x.
  See the [UltraGraph announcement for details.](https://www.deepcausality.com/blog/announcement-ultragraph-0-8)
- Groundwork for emergent dynamic causality has been done via an initial implementation of a generic generative process
  that dynamically constructs causal graphs or context at runtime. This starts the implementation of dynamic causal
  emergence as outlined in the Effect Propagation Process article. However, due to the novelty of dynamic causal
  emergence in relativistic contexts, this feature is early access and neither documented nor officially supported at
  this stage.

## Conclusion

DeepCausality 0.8 offers uniform deterministic and probabilistic causal reasoning across Euclidean and non-Euclidean
context
that is end to end explainable to support advanced use cases for dynamic causal modeling.

Get Started with DeepCausality 0.8. The Future is Now!

* Explore the [code examples on GitHub](https://github.com/deepcausality-rs/deep_causality/tree/main/examples).
* Join the [community](https://www.deepcausality.com/community/).
* Join the [Discord Server](https://discord.gg/Bxj9P7JXSj)

## About

[DeepCausality](https://www.deepcausality.com/) is a dynamic-causality framework that enables fast and
deterministic context-aware causal reasoning in Rust. Please give us
a [star on GitHub.](https://github.com/deepcausality-rs/deep_causality)

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community, and drives open source
innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all the
members of the community. For more information, please visit [lfaidata.foundation](https://lfaidata.foundation).
