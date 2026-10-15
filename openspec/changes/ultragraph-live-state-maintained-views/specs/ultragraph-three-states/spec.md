## ADDED Requirements

### Requirement: The graph is in one of three states
`GraphState` SHALL have the variants `Dynamic`, `Live` and `Frozen`; `is_frozen` SHALL be true in `Frozen` only and `is_live` SHALL be true in `Live` only.

#### Scenario: State queries in each state
- **WHEN** a graph is queried in the Dynamic, the Live and the Frozen state
- **THEN** `is_frozen()` is false, false, true and `is_live()` is false, true, false

### Requirement: Each state allows what it serves
Mutations SHALL be accepted in the Dynamic and the Live state, except `compact_ids`, which the Live state refuses with `GraphError::GraphIsLive`, and refused with `GraphError::GraphIsFrozen` in the Frozen state; traversal SHALL work in all three; every algorithm of `GraphAlgorithms` and `project` SHALL work in the Live and the Frozen state and return `GraphError::GraphNotFrozen` in the Dynamic state.

#### Scenario: Live accepts a write and answers an algorithm
- **WHEN** a Live graph receives `add_edge(2, 3, w)` and `is_reachable(0, 3)` is then called
- **THEN** the write succeeds and the answer reflects the new edge without any transition

#### Scenario: Frozen refuses a write
- **WHEN** `add_node` is called on a Frozen graph
- **THEN** it returns `Err(GraphError::GraphIsFrozen)`

### Requirement: Every transition keeps every position, node and edge
The transitions Dynamic → Live, Frozen → Live, Live → Frozen and Live → Dynamic SHALL keep every position, payload, tombstone, edge and the root, so the graph reads the same before and after each. Every transition, `freeze` and `unfreeze` included, SHALL keep every maintained view under its identifier, reading as it did before.

#### Scenario: A round through every state
- **WHEN** a graph with tombstones and parallel edges of several keys goes Dynamic → Live → Frozen → Live → Dynamic
- **THEN** after each transition `get_node`, `get_edges`, both traversals and `get_root_index` return what they returned before it

#### Scenario: Views survive every transition
- **WHEN** a graph with one maintained view goes Dynamic → Live → Frozen → Dynamic → Frozen → Live
- **THEN** after each transition `view(id)` returns the view, and its results are those it returned before the transition

### Requirement: Live answers as a freshly frozen graph would
After any sequence of mutations in the Live state, every algorithm and traversal SHALL return what it returns on the Frozen graph obtained by applying the same operations to a Dynamic graph and freezing it.

#### Scenario: Agreement over generated mutation sequences
- **WHEN** seeded random sequences of node and edge additions and removals run against a Live graph and against a Dynamic graph that is frozen after every step
- **THEN** after every step every algorithm's result and every adjacency list agree

#### Scenario: A row that outgrows its capacity
- **WHEN** a Live node receives more edges than its row's capacity, one at a time
- **THEN** every edge is present afterwards and every other node's adjacency is unchanged

#### Scenario: A removed node in Live
- **WHEN** a node with incoming and outgoing edges is removed in the Live state
- **THEN** `contains_node` is false, no traversal yields it, `outbound_edges` from it returns `NodeNotFound`, and `number_edges` excludes its edges

### Requirement: freeze_clone returns a frozen copy and leaves the source alone
`freeze_clone` SHALL return a new graph in the Frozen state holding the source's positions, payloads, tombstones, edges and root and no maintained views, and SHALL leave the source's state, positions and maintained views unchanged; called on a Frozen graph it SHALL return a copy of it, also without maintained views.

#### Scenario: The source stays mutable
- **WHEN** `freeze_clone` is called on a Dynamic graph and the source then receives `add_node`
- **THEN** the copy is frozen and unchanged, and the source accepts the write

#### Scenario: The copy carries no maintained view
- **WHEN** a Live graph with two maintained views is `freeze_clone`d
- **THEN** the copy has no maintained view and the source still has both, current

### Requirement: compact_ids is refused in the Live state
`compact_ids` SHALL return `GraphError::GraphIsLive` in the Live state and change nothing, as it returns `GraphError::GraphIsFrozen` in the Frozen state.

#### Scenario: Compacting a Live graph is refused
- **WHEN** `compact_ids` is called on a Live graph with a tombstone
- **THEN** it returns `Err(GraphError::GraphIsLive)`, and every position, edge and maintained view is unchanged

### Requirement: clear empties the graph from every state
`clear` SHALL leave an empty Dynamic graph from the Live state as from the others, with positions restarting at 0 and every maintained view registered and empty.

#### Scenario: Clearing a Live graph
- **WHEN** a Live graph with one maintained view is cleared and a node is added
- **THEN** `is_live()` is false, `add_node` returned 0, and the view holds no node until the new node passes its test
