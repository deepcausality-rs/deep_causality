[//]: # (---)

[//]: # (SPDX-License-Identifier: MIT)

[//]: # (---)

# Context

DeepCausality enables context aware causality reason across data-like, time-like, space-like, and spacetime-like
entities stored within a context-hyper-graphs. Fundamentally, this allows scalable contextualization up to four
dimensions.

## Data Context

The most basic form of contextualization adds more data to a model. For example, when modeling GDP for any country,
commodity prices such as oil play a significant role in addition to several national factors. Conventionally, the GDP
model separates internal from external factors to isolate systematic risk. DeepCausality solves this by adding two
distinct data contexts, one for national data, i.e., population growth & manufacturing output, and a second context for
data from external factors, i.e., standard crude oil price.

In a more realistic context, temporal structures become a significant contributing factor. For example, when modeling
volatile markets, the current price action usually operates within a particular micro and macro temporal structure,
often represented as the daily and weekly pivot points and, in some cases, the annual low/high values. For most days,
current price action operates within the temporal microstructure. Still, if, for any reason, the current price
approaches the boundaries set by the relative macro structure, extreme volatility follows.

DeepCausality solves this by adding multiple micro and macro temporal data contexts that provide the model access to
anchoring points representative of the selected temporal resolution. Therefore, modeling market volatility relative to
its intrinsic context becomes relatively easy. Furthermore, for synthetic instruments such as Future Spreads, the
approach can be extended to adding multiple contexts for each future contract used in the spread.

## Temporal and Spatial Context

In a more advanced context, considering temporal and spatial changes becomes a requirement. There is a clear distinction
between stationary context and non-stationary context. The former requires no further adjustment, whereas the latter
does. In the first scenario, consider a sensor array mounted to a drone swarm. In this case, the data stream from the
sensors is subject to positional changes over time because of the drone movement. Note this is a stationary change since
the drone swarm cannot accelerate or move beyond certain limits, so it remains subject to Newtonian physics. Therefore,
the data stream requires contextualization relative to the tempo-spatial position of the drone swarm to provide
meaningful information to the model.

DeepCausality solves this by adding a temporal master context that structures time as a 4-dimensional hyper-graph where
each node encodes a unit of time (A tempoid) which itself contains a link to a sub-node that encodes spatial
information (A spaceoid), which then links to the actual chunk of data through a data object (A dataoid).
That way, multiple drones can stream multiple data to update a 4D context graph that informs the model
and allows for real-time monitoring of complex tempo-spatial patterns.

## Adjustable Temporal and Spatial Context

For non-stationary tempo-spatial processes subject to non-Newtonian changes, the general problem arises that the entire
4D tempo-spatial context structure becomes subject to external changes. This is the case when modeling control systems
subjected to gravitational forces deviating from the terrestrial standard of 9.807 m/s². When that happens, the entire
4D context requires adjustment, for example, when modeling inert navigation for outer-space travel and certainly when
modeling any moving object nearby a black hole.

DeepCausality solves this by activating an optional adjustment mechanism that allows to adjustment of the value of each
element in a context graph before evaluating the attached model. As a result, after primary measurement updated a
context graph, a secondary process may estimate the relative adjustment for time dilation, which results in one of two
scenarios. One, adjustment for all measurements remains constant. In this case, adjusting all measurements might be more
sensible before updating the context graph.
In the second scenario, there is a gradual adjustment for different measurements. Often, this requires the formulation
of an adjustment matrix. The matrix can already be attached to each element of the context graph but may require
periodic updates depending on the change of gravitational force. In this case, the secondary process calculates the new
adjustment matrix for each element of the context graph and then calls the adjustment of the entire context graph,
ensuring all measurements in the context get adjusted relative to the new adjustment matrix that represents the impact
of gravitational change.

The exact adjustment for temporal spatial data depends on the actual structure of the representative structure.
Theoretically, a tensor would be the preferred structure, allowing for multi-dimensional adjustment representation. In
practice, however, tensors occur a non-trivial overhead leading to a non-trivial performance penalty.
Therefore, DeepCausality brings a custom data structure called a Grid that is indexed with a variable PointIndex. The
difference to a tensor is that a tensor remains parametric over N dimensions, thus requiring a complex object
representation. In contrast, a Grid is limited to low dimensions (1 to 4), allowing representations as a scalar, vector,
or matrix, representing all types represented as a static fixed-size array. Fixed-sized arrays allow for several
compiler optimizations, including the complete removal of runtime array boundary checks, since all structural parameters
are known upfront, providing a significant performance boost over tensors. Performance is critical because context
hyper-graphs may grow large with millions of nodes, and obviously, one wants the fastest possible global adjustment in
those cases.

