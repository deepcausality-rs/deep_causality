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

Deep learning and DeepCausality are complementary methodologies with different strengths that compose well in a single system. Deep learning is a strong choice for perception: object recognition, anomaly detection in raw signals, embeddings of unstructured text. DeepCausality is a strong choice for dynamic reasoning with a verifiable audit trail. A drone might use a deep-learning vision system to recognize the tunnel ahead, and a DeepCausality model to reason about what the loss of GPS implies for navigation and which fallback is permissible under the current ethos. Each method plays to its strengths, and either can feed the other depending on what the system requiements.

DeepCausality is a computational causality framework, and it differs from the classical frameworks in one specific way: it treats causality itself as a dynamic process. The [next page](/docs/overview/the-problem/) explains what that means in practice, and the [page after that](/docs/overview/core-idea/) gives you the single idea on which the library is built.
