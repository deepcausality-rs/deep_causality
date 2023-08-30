[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# How is deep causality different from deep learning?

* Free of
  the [independent and identically distributed data (IID) assumption.](https://towardsdatascience.com/independent-and-identically-distributed-ce250ad1bfa8)
* Deterministic and explainable causal reasoning.
* Reasoning over causal collection, graph, or hyper-graph structure supported.
* Recursive causal data-structures enable concise expression of arbitrary complex causal structures.
* Context aware causality reasoning across data-like, time-like, space-like, spacetime-like entities stored within (
  multiple) context-hyper-graphs

## When to use deep causality and when to use  deep learning?

**Use deep learning (PyTorch, Tensorflow, Keras, MXNEet etc):**

* Explanations are not needed.
* Language models (ChatGPT etc)
* Generative models (Midjourney etc.)
* Object / Image classification ([ImageNet](https://paperswithcode.com/sota/image-classification-on-imagenet) etc.)

Whenever it is expected that the data the model sees in production have the
same or similar distribution and characteristics as the data used during the
model training, deep learning will be very successful. Also, when it is acceptable
that decisions made by the model remain an unexplainable blackbox or an explanation
is fundamentally not needed (image classification), then deep learning is perhaps the best solution
available for the time being. Last but not least, use deep learning whenever the data distribution
used in a model remains largely unaffected whenever context may change.
Otherwise, you would have to retrain a model quite frequently.

**Use deep causality:**

* Explanations are required
* Determinism is required
* Multi-causal, contextual multi-stage reasoning is required
* Complex event processing in real-time is required

Whenever an inferred decision must be deterministic and explainable regardless
of data distribution or characteristics, use deep causality. Furthermore,
when data feed into the model are affected by contextual changes, then deep causality
allows you to model the context and account for those changes before reasoning occurs.
Deep causality is inherently deterministic and therefore suited for applications with
high reliability and robustness standards.

## When to use both, deep causality and deep learning?

In practice, there is a broad and wide space for coexistence because of the
largely complementary properties. For example, data from a sensor array may
use deep learning for object classification and use then a deep causality model
to derive contextualized decisions in real-time. Or in another setting, a deep causality
model monitors multiple IoT sensor arrays in real-time for anomalies and only when an anomaly
has been detected triggers the engagement of a drone with a more sophisticated Ai model
to discern what kind of anomaly may occurs. On a more fundamental level, when multiple causal models
haven been developed, [hyper geometric deep learning ](https://deephypergraph.com/) may help to find
more or novel causality graph representations.
These are just a few examples, but in reality endless possibilities emerge from combining deep learning with deep
causality. 
