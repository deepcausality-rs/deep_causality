---
title: Towards a Fundamental Understanding of Computational Causality
date: 2025-06-04
author: Marvin Hansen
---

[//]: # (SPDX-License-Identifier: CC-BY-4.0)

![DeepCausality logo](/img/logo-color.png)


Conventional approaches to computational causality impose familiar structures such as linear time, Euclidean space, and discrete cause-effect sequences onto the systems we seek to model.

However, complex systems with dynamic feedback loops may not adhere to those familiarities and instead emit emergent adaptive behavior that conventional methods of computational causality struggle to grasp. These methods, when applied to complex systems, squeeze the intricateness of reality into the constraints of the modeling approach.

DeepCausality emerges from a different premise: that the modeling paradigms must strive to match the structural richness and dynamic nature of reality itself, even if this path entails a departure from familiar concepts.
The motivation to depart from familiar concepts stems from practical challenges of applying causal inference to non-linear temporal structures such as temporal graphs or dynamic feedback loops. The root of the problem is that conventional causality pre-supposes a time-asymmetric order, which directly conflicts with non-linear temporal structures that lack a fixed temporal order.
To reconcile causality with non-linear temporal structures, DeepCausality implements the [Effect Propagation Process](https://www.deepcausality.com/docs/concepts/effect-propagation-process/), a spacetime-agnostic definition of post-quantum causality, enabling uniform causal inference over non-linear temporal structures and non-Euclidean spaces.

Central to DeepCausality's approach is the Causaloid, an idea borrowed from pioneering work by Lucien Hardy[^1] at the Perimeter Institute of Theoretical Physics. The causaloid is a unit where the traditional separation of cause and effect is  folded into a singular, encapsulated, testable causal function that is spacetime-agnostic.

The Causaloid signifies an evaluable state: the precise conditions under which a specific causal linkage holds true for a given observation within the prevailing context.

The causaloid can be a single unit, a collection (Map, Array, or Vector), or a hypergraph. In a CausaloidGraph, Causaloids can themselves be entire networks (subgraphs) of finer-grained causal relationships, allowing for a hierarchical decomposition of complex phenomena into a modular architecture. This architecture permits the tracing of influence not only between atomic events, but through nested layers of causal understanding, reflecting the deeply composed nature of many real-world systems. The CausaloidGraph can be analyzed in various ways, for example, by reasoning over the entire graph, a specific subgraph, or reasoning over all Causaloids between designated start and stop Causaloids. In all cases, the causal inference occurs relative to the context attached to the CausaloidGraph.

Context in DeepCausality is designed to manage multi-dimensional hypergraphs, capable of representing data, time, space, and spacetime, seamlessly integrating both Euclidean geometries and non-Euclidean relational structures—such as the connectivity of a social network or the dependencies within a conceptual framework.

Contextualized causal reasoning is implemented by implicitly passing a reference to the causal function. This mechanism allows for the contextualization of certain Causaloids while leaving others context-free. The diagram below shows a scenario in which specific causaloids use data from a context while other causaloids remain context-free.

![Causaloid diagram](/img/docs/causaloid.png)


DeepCausality elevates space and time from implicit, fixed assumptions to explicit, dynamic, and flexible contextual elements. Furthermore, DeepCausality can handle multiple contexts per model or share a specific context between models. The latter case requires a designated global update mechanism to ensure conflict-free updates.

DeepCausality uniquely enables dynamic context modifications via its Adjustable mechanism. For example, correcting contextual data for sensor drift comes down to implementing the Adjustable trait that corrects the drift in the context before it propagates into the model. In a more advanced setting, adjusting satellite signals for gravitationally induced time-dilation  follows the same path by implementing the Adjustable trait to correct distortions of contextual data. This is of particular value when dealing with high-precision satellite navigation, for example, when working with autonomous drones.

This conceptual architecture, featuring a unified causal mechanism structured recursively, operating within a generalized and dynamic context, is born out of the recognition that computer systems become increasingly more embedded into our complex dynamic world. When we attempt to model systems where physical, informational, and relational influences intertwine, where feedback loops are abundant, and where the rules of interaction may shift, the traditional imperative to simplify reality to fit the model becomes insufficient.

> DeepCausality inverts the prevalent paradigm: Instead of squeezing the intricacy of reality into the constraints of the modeling approach, DeepCausality removes modeling  constraints and provides a modeling toolkit sufficiently expressive to capture the inherent intricacy of reality.

The apparent 'complexity' of the DeepCausality framework, therefore, is less an imposition and more a reflection of the sophisticated causal tapestries it is designed to engage with.

Ensuring this deep causal reasoning translates into coherent system behavior is the role of the Causal State Machine (CSM). Supervising one or more CausaloidGraphs and their interplay with associated Context(s), the CSM identifies salient patterns of activated causal configurations. From these 'causal states,' it deterministically initiates predefined actions, providing an operational bridge from nuanced understanding of reality to effective interaction with reality.
The CSM facilitates a dynamic interplay: interventions it initiates can, in turn, update the Context, thereby closing the gap between model-based action and evolving real-world observations. This creates a pathway for continuous learning and adaptation grounded in causal understanding

DeepCausality charts a path towards greater fidelity in computational causality. It proposes that the CausaloidGraph, with its recursive depth, offers a more robust modular representation of mechanistic linkage. The application of the 'effect propagation process' through generalized context hypergraphs results in a more versatile medium for these  continuous causal interactions. Lastly, the Causal State Machine, by supervising this interplay, ensures the entire causal understanding remains operationally effective, supervised, and explainable.

This perspective, while demanding a departure from simpler paradigms, holds the promise of a more profound and verifiable grasp of the complex causal dynamics that shape our world, empowering intelligent systems to bridge the gap between profound causal understanding and effective, reasoned interaction with the intricate dynamics of the surrounding world.

## About

DeepCausality is a dynamic-causality framework that enables fast and deterministic context-aware causal reasoning in Rust. The DeepCausality project is hosted at the Linux Foundation for Artificial Intelligence and Data (LF AI & Data). Learn more about DeepCausality on GitHub and join the DeepCausality-Announce Mailing List.

The LF AI & Data Foundation supports an open artificial intelligence (AI) and data community, and drives open source innovation in the AI and data domains by enabling collaboration and the creation of new opportunities for all the members of the community. For more information, please visit lfaidata.foundation.



[^1]: Hardy, Lucien. "Probability theories with dynamic causal structure: a new framework for quantum gravity." arXiv preprint gr-qc/0509120 (2005).