# Deep Dive: The Effect Propagation Process

In the effect propagation process, cause, and effect are folded into one single entity, the causaloid, that takes
a propagating effect as input and returns another propagating effect as its output. A causaloid models causal relations
as a functional dependency of the previous propagating effect on the current propagating effect via a causal function.
The key difference from conventional classical causality, which models a causal relationship as a temporal order,
comes down to two properties of the causal function. One, the functional dependency is
independent of temporal order and therefore can handle non-Euclidean representation and relativistic effects. Second,
the causal function is unconstrained and therefore can be deterministic, probabilistic, a support vector machine or even
a non-deterministic method such as a neural net. As long as the computed effect can be expressed as a propagating
effect, the causal function is valid and can be stored in a Causaloid.

A propagating effect can be a deterministic (causal), a probabilistic value, a probabilistic distribution, or an
arbitrarily complex type stored as a contextual reference. DeepCausality provides reasoning for deterministic and
probabilistic modalities, whereas reasoning over arbitrarily complex types requires custom implementation. To streamline
data sharing, those complex types are stored and loaded from a context attached to the causal model.

DeepCausality supports multiple contexts that can store complex spatio-temporal data as well as data distributions to
account for uncertainty. The adjustable mechanism enables dynamic data updates, for example, from real-time data streams
or sensor data. A hypergraph represents each context and thus enables flexible data relations, i.e., a point in
spacetime may link to multiple sensor readings and a reference data distribution for each sensor to detect data
anomalies. The context, therefore, supports sophisticated reasoning across advanced causal structures. Out of the box,
DeepCausality supports multi-modal causal reasoning across singleton, causal collection, and causal hyper-graph
structures. For causal collections, multiple modes of aggregation are supported, whereas the causal hypergraph
implements the effect propagation process in which the reasoning engine traverses the graph, applies the previous
propagating effect to the current causaloid, and then takes that propagating effect and applies it to the next causaloid
until the graph traversal ends. To support flexible reasoning over geometric causal structures, DeepCausality supports
common path algorithms, i.e., shortest path, start from a node, and path between two nodes.

Once a final conclusion has been reached, the causal state machine enables the explicit linking between the conclusion
and a specific action to be taken as a result. However, because dynamic reasoning over a dynamic context may not always
result in a predictable outcome, DeepCausality has developed the EffectEthos, a programmable ethos, to encode contextual
operational rules the causal state machine can check to ensure that a proposed action is safe and within the pre-defined
rules. The effect ethos can access the same context as the causal model that has led to the insight that triggered a
proposed action and can therefore retrieve relevant and timely data to decide whether the action should be taken. One
key aspect of the effect ethos is its ability to resolve conflicting rules via an internal algorithm that gives
precedent to a rule with higher priority, or a higher authority, to ensure the final rule set is correctly applied.
Furthermore, a tagging system enables efficient re-use and selection of applicable rules.

DeepCausality applies state-of-the-art performance optimization, such as its custom compact sparse representation (CSR)
hypergraph implementation that delivers sub-second traversal time on graphs with ten million nodes or more. Furthermore,
static dispatching in all critical hot paths ensures significant performance even on moderate hardware and thus is
suitable for real-time applications without additional acceleration hardware.

In terms of applications, DeepCausality enables a number of advanced use cases, such as real-time sensor fusion,
real-time contextual risk monitoring, and contextual interaction i.e. in robotics safeguarded by its effect ethos.