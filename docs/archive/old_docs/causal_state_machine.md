[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Causal State Machine

The causal state machine encodes causal entities as types with the actual constraint represented as a causal function.
One or more causal entities form a collection that is encapsulated in an abstraction called a causaloid. The causaloid
represents the state, which then gets passed into a the state machine for evaluation and subsequent action.
Unlike a finite state machine, the causal state machine may not know all its causal entities
upfront which allows for greater flexibility:

1) The causaloid may gets generated
2) The causaloid may changes in structure
3) The causaloid may evolves over time

The state machine, therefore, only evaluates the causal state with its structure
at a point in time. Therefore, revision requires to encapsulate the causaloid in structure that
carries meta-data that inform about structural changes.

* [Source](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality/src/csm)
* [Example](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality/examples/csm)
* [Test](https://github.com/deepcausality-rs/deep_causality/tree/main/deep_causality/tests)
