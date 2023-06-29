# Causal Structure

As previously outlined, some use cases require gradient adjustment of 4D hyper-graph struc-tures. Still, space-temporal
adjustment implies a less obvious problem:

Can causal structure shift or even invert when the adjustment becomes large enough?

As it turned out, causal inversion can only happen under rare circumstances (spacetime in-version inside a black hole),
but in a more practical sense, temporal shift and, with it, causal shift, will strike long before reaching a black hole.
Specifically, a relatively minor temporal shift already occurs when adjusting time-dilatation when correcting GPS
signals for spacetime curvature.

However, the degree of the temporal shift depends on distance and gravity. Therefore, any larger extra-terrestrial
distance or major change in gravity force induces a significant tem-poral shift. Fundamentally, this makes causal
relation that are relying on a cause-effect no-tion non-stationary because, when the temporal shift becomes large
enough, causes be-come effects, and effects become causes. This non-trivial problem roots in the time sym-metry
assumption and can result in complete system failure.

DeepCausality solves this problem by borrowing and implementing
a [novel causal concept pioneered by Lucien Hardy](https://arxiv.org/abs/gr-qc/0608043) who
works now at the Perimeter Institute:

## The causaloid

The causaloid encodes a causal relation as a causal function that maps input data to an output decision determining
whether, on the input data, the causal relation, encoded as a function, holds.
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
