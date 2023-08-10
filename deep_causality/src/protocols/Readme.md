# Protocols

A protocol is a trait that contains an optional default implementation.
The term protocol is borrowed from the Swift programming language as
it clearly conveys the meaning of defining the behavior of a type. 

The core protocols with  a default implementation are:
* AssumableReasoning
* CausableReasoning
* InferableReasoning
* ObservableReasoning

The default implementation of a protocol is re-used for each type extension of that protocol.
For example, the protocols above have been implemented as type extension for five standard collections
meaning the default implementation is the only code to maintain while its inserted into each of the five type extensions.


The causable graph protocol has been seperaeted into three different protocols:

1) Causable_graph_type: Describes the type (struct). Most interface
2) Causable_graph_explaining: Default implementation for causal explanation 
3) Causable_graph_reasoning: Default implementation for causal reasoning

The causal graph explaining and reasoning protocols only require one 
method implemented for each to make all default implementations work correctly. 
