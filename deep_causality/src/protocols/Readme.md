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

The adjutable protocol only provides a dummy default implementation to make its functionality optional. 
To  use the adjustment function, the default implementation must be overwritten with a custom implementatio n. 
Note, this should rarely be necessary because there are roughly two scenarios when that becomes necessary. 
One, there is a systematic inference in the data collection that somehow
cannot be handled by other means hence require the adjustment of some or all of the context nodes. Two, when
you model is subject to contextual changes such as spacetime curvature in theoretical physics.
Both scenarios, while rare, can be handled with a custom implementation of adjustable protocol. 
