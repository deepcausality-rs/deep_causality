## ADDED Requirements

### Requirement: The container takes a projection type with a default that maintains nothing
`UltraGraphContainer` SHALL take a third type parameter `P: Projection<N, W>` for its maintained views, with a default type that maintains no view, and the aliases `UltraGraph<T>` and `UltraGraphWeighted<T, W>` SHALL keep that default.

#### Scenario: Existing code compiles unchanged
- **WHEN** a graph is named as `UltraGraphWeighted<T, W>` and built with `UltraGraphWeighted::with_capacity`
- **THEN** it compiles without naming a projection type

#### Scenario: Several scopes under one type
- **WHEN** a caller's enum with variants for a zone and for validity at a time implements `Projection<N, W>` and is the container's projection type
- **THEN** views of both variants can be maintained on one graph

### Requirement: maintain registers a view the graph keeps current
`maintain(projection, mode)` SHALL, in every state, build a view of the passing nodes and edges under `mode`, register it with the graph, and return an identifier. `view(id)` SHALL return that view in every state, and `None` for an identifier the graph did not issue. Every algorithm, read and traversal SHALL run on the view in every state, its reads and traversals listing one entry per passing edge as a projection view's do.

#### Scenario: A view answers for its scope
- **WHEN** a view is maintained for zone A on a graph whose edge 1 → 2 crosses into zone B
- **THEN** on the view `is_reachable(0, 2)` is false while on the graph it is true

#### Scenario: A view answers while the graph is dynamic
- **WHEN** a view is maintained on a Dynamic graph and `is_reachable(0, 1)` is called on the view and on the graph
- **THEN** the view answers without a transition, and the graph returns `Err(GraphError::GraphNotFrozen)`

### Requirement: Every maintained view is current after every mutation
After every mutation in the Dynamic or the Live state, each maintained view SHALL equal the projection of the current graph under its predicates and mode, with no call other than the mutation itself.

#### Scenario: An edge added in scope appears in the view
- **WHEN** `add_edge(0, 3, w)` with both ends in zone A follows the view's registration
- **THEN** the view's `outbound_edges(0)` yields 3 immediately

#### Scenario: A node leaving the scope leaves the view with its edges
- **WHEN** `update_node(1, payload_in_zone_b)` is called on a node of the zone-A view
- **THEN** the view's `contains_node(1)` is false and no traversal of the view yields 1

#### Scenario: Agreement over generated mutation sequences
- **WHEN** seeded random mutation sequences run with four maintained views of different tests and modes
- **THEN** after every step each view's results equal those of a projection built afresh from the current graph

### Requirement: compact_ids carries every maintained view through the renumbering
In the Dynamic state, `compact_ids` SHALL rebuild every maintained view from its projection and mode at the new positions, under its identifier.

#### Scenario: A view after compaction
- **WHEN** position 1 of a Dynamic graph with a maintained view is removed and `compact_ids` is called
- **THEN** `view(id)` returns a view whose results equal those of a view newly maintained on the compacted graph with the same projection and mode

### Requirement: A maintained view's tests read payloads and registration parameters only
A maintained view SHALL be re-tested only for the nodes and edges a mutation touches, so its tests SHALL depend only on the payloads and on parameters fixed when the view is registered.

#### Scenario: A view of open relations follows a closing write
- **WHEN** a view passes edges whose `valid_until` is unset, and `add_edge(a, b, w_closed)` replaces the payload of edge `(a, b, k)` with one whose `valid_until` is set
- **THEN** the view no longer holds the edge

### Requirement: freeze keeps the views and freeze_clone leaves them behind
`freeze` in place SHALL keep every maintained view readable in the Frozen state; `freeze_clone` SHALL return a copy without maintained views (see `ultragraph-three-states`).

#### Scenario: Views read after freezing in place
- **WHEN** a graph with a maintained view is frozen with `freeze`
- **THEN** `view(id)` returns the view, and its results are those it returned before the freeze
