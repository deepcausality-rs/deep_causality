[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Causal Structure

DeepCausality uses the causaloid as its central structure, an idea
borrowed from a [novel causal concept](https://arxiv.org/abs/gr-qc/0608043) pioneered
by [Lucien Hardy](https://perimeterinstitute.ca/people/lucien-hardy) at
the [Perimeter Institute](https://perimeterinstitute.ca/)
of [theoretical physics](https://perimeterinstitute.ca/why-theoretical-physics).

## The causaloid

The causaloid encodes a causal relation as a causal function that maps input data
to an output decision determining whether, on the input data, the causal relation, encoded as a function, holds.
Therefore, the causaloid folds cause and effect into one single entity. Causaloids are structurally invariant and,
therefore, temporarily stationary. In the event of a temporal shift, only the order of causal evaluation may shift
following the temporary shift. Still, the causal structure remains intact because there is no conceptual distinction
between cause and effect anymore. Furthermore, algebraic types applied to causaloids enable recursive causal structures.

## Recursive causal data-structures

DeepCausality scales from simple to advanced use cases with complex contextual requirements. To take the idea of
scalable complexity one step further, DeepCausality also offers scalable causal data structure. Fundamentally, a causal
data structure augments an existing data structure with abilities to conduct causal reasoning. Not everybody needs a 4D
hypergraph to solve common problems. Therefore, DeepCausality provides a set of causal data structures that scale causal
reasoning from simple to complex structures:

* Causal Array
* Causal Vector
* Causal HashMap
* Causaloid Graph

The causaloid, however, can be a singleton, a collection, or a graph. The causaloid-graph, however, is a hypergraph with
each node being a causaloid. This recursive structure means a sub-graph can be encapsulated as a causaloid which then
becomes a node of a graph. A HashMap of causes can be encapsulated as a causaloid and embedded into the same graph.
Then, the entire causaloid-graph can be analyzed in a variety of ways, for example:

* Reason over the entire graph
* Reason only over a specific causaloid
* Reason over all causaloid between a start and stop causaloid.
* Reason over the shortest path between two causaloid.
