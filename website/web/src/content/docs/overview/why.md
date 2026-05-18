---
title: Why DeepCausality
description: What computational causality is, what it earns you, and where DeepCausality fits in.
section: overview
order: 1
---

Computational causality is not deep learning. Deep learning excels at pattern matching: object detection, fraud detection, next-word prediction in large language models. Those systems are correlation engines. They have no foundational concept of space, time, context, or causality, which is why a confident LLM can also be a confidently wrong one.

Computational causality is the alternative when correlation is not enough. It gives you three things deep learning cannot:

- **Deterministic reasoning**: same input, same output. The system gives the same answer on Tuesday that it gave on Monday.
- **Probabilistic reasoning**: explicit odds, explicit confidence. The system tells you not just *what* it concluded but *how sure* it is.
- **Full explainability**: a logical line of reasoning. You can ask the system "why" and get a structured answer.

These properties matter in regulated and high-stakes domains: medicine, finance, robotics, avionics, industrial control. In those domains, a regulator, an operator, or a courtroom is going to ask why a decision was made, and here, an audit trail of deterministic reasoning becomes critical.

Deep learning and DeepCausality are complementary methodologies with different strengths that compose well in a single system. Deep learning is a strong choice for perception: object recognition, anomaly detection in raw signals, embeddings of unstructured text. DeepCausality is a strong choice for dynamic reasoning with a verifiable audit trail. A drone might use a deep-learning vision system to recognize the tunnel ahead, and a DeepCausality model to reason about what the loss of GPS implies for navigation and which fallback is permissible under the current ethos. Each method plays to its strengths, and either can feed the other depending on what the system requires.

## What makes DeepCausality unique

Most causality frameworks pick one paradigm. Pearl's Structural Causal Models pick a graph. Granger causality and the Rubin causal model pick a sequence. State-space models and control theory pick a process. Each is sound on its own ground, and each pays for the other paradigms through escape hatches, glue layers, or external orchestration. The price shows up when a real system needs the structural reasoning, the sequential reasoning, and the stateful threading in one inference path. Then you spend more time gluing models together than reasoning about the problem.

DeepCausality collapses those paradigms into one carrier: the propagating effect.

The library has two reasoning primitives that emit the same propagating-effect type:

- The **Causaloid** handles structural reasoning and composes isomorphic-recursively. A Singleton Causaloid, a collection of Causaloids, and a graph of Causaloids all nest into each other. A collection of causaloids nest into a singleton causaloid, which then become a node into a causaloid graph. The graph itself might be a node into a larger causaloid graph.

- The **Causal Monad** handles sequential reasoning. `pure` lifts a value into a chain; `bind` chains the next step; `intervene` rewrites a value mid-chain for counterfactual analysis. The chain accumulates an audit log automatically and short-circuits cleanly on error.

Because both primitives return the same propagating-effect type, you can take a Causaloid's verdict and `.bind` directly onto it. You can run a Causal Monad bind chain and feed its result into a Causaloid. The boundary between "structural reasoning" and "sequential reasoning" moves as the problem evolves. Instead of picking one, two, or more frameworks and wiring them together, you just choose oneframework and pick the modality relative to the problem you're solving.

The same carrier covers two reasoning regimes. The non-Markovian `PropagatingEffect<T>` is the simpler case where each step depends only on its inputs and rules. The Markovian `PropagatingProcess<T, S, C>` carries state and context through every step. Both are aliases of the same underlying 5-arity type, so promoting a non-Markovian chain into a Markovian one is one constructor call rather than a rewrite.

The avionics [flight envelope monitor](https://github.com/deepcausality-rs/deep_causality/tree/main/examples/avionics_examples/flight_envelope_monitor) runs a Causaloid Collection over five sensor-health checks, a three-step Causal Monad bind-chain for state estimation, and a Causaloid hypergraph of six envelope protections, all threading through one `PropagatingProcess<T, FlightState, AircraftConfig>` with state and audit log carried across every stage. The same PropagatingEffect supports the physics, medicine, and distributed-systems examples in the repository.

The Causaloid alone gives you structure. The Causal Monad alone gives you sequencing. Neither would deliver the multi-domain composition the examples show. The fact that both emit the same type, that both can be lifted between Markovian and non-Markovian forms, and that they nest freely, is the move that makes DeepCausality unique.

### From reasoning to action

Reasoning composition is only half of the story. DeepCausality enables reasoning based action with two further primitives:

- The **Causal State Machine (CSM)** is the bridge from inference to the outside world. It reads the propagating effect produced by the reasoning layer, evaluates which of its registered causal states have become active, and proposes the action linked to each active state. The state space is inferred at runtime rather than enumerated at design time, which is how the CSM avoids the limitation of a classical finite state machine.

- The **Effect Ethos** is a programmable safety guardrail above the CSM. Every action the CSM would otherwise fire is intercepted by the Ethos and evaluated against an immutable graph of computable norms. The Ethos returns a verdict: Obligatory, Impermissible, or Optional with an associated cost. Based on the verdict, the proposed CSM action may execute or get stopped out. When it gets stopped out, the reason from the effect eaters is locked together with the outcome, so that in a subsequent audit one sees why the was stopped, what its line of reasoning was, what the last proposed action was and why it was stopped.

The combination matters most in the dynamic and emergent regimes. When the underlying reasoning is fully deterministic, the Ethos is unnecessary and the CSM can fire directly. When the causal structure itself evolves at runtime, static verification of the reasoning is no longer feasible, and the Ethos becomes the layer that restores verifiability at the action boundary. Whether the inference behind a proposed action was statically provable or emerged at runtime, the action only leaves the system if the Ethos says it is permissible under the its encoded ethos.

## Where to go from here

DeepCausality treats causality itself as a dynamic process. The [next page](/docs/overview/the-problem/) explains what that means in practice, and the [page after that](/docs/overview/core-idea/) gives you the single idea on which the library is built.
