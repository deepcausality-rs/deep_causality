## ADDED Requirements

### Requirement: A position names one node for the node's whole life
UltraGraph SHALL keep the position `add_node` returned for a node unchanged until that node is removed, across any sequence of mutations, `freeze` and `unfreeze`.

Only `compact_ids` and `clear` change positions, and both are calls the caller makes on purpose.

#### Scenario: Freezing after a removal moves no node
- **WHEN** nodes A, B, C and D are added at positions 0, 1, 2 and 3, B is removed, and the graph is frozen
- **THEN** `get_node(2)` is C, `get_node(3)` is D, and `contains_node(1)` is false

#### Scenario: Unfreezing keeps every position
- **WHEN** the frozen graph of the previous scenario is unfrozen
- **THEN** `get_node(0)` is A, `get_node(2)` is C, `get_node(3)` is D, and `contains_node(1)` is false

#### Scenario: Round trips keep trailing tombstones
- **WHEN** nodes are added at positions 0 to 4, positions 3 and 4 are removed, and the graph is frozen, unfrozen and frozen again
- **THEN** positions 0 to 2 hold their nodes, positions 3 and 4 read as absent, and the next `add_node` after unfreezing returns 5

#### Scenario: The root keeps its position
- **WHEN** a root is added at position 4 after nodes at 0 to 3, position 1 is removed, and the graph is frozen
- **THEN** `get_root_index()` returns `Some(4)` and `get_root_node()` returns the root's payload

#### Scenario: A removed root reads as no root
- **WHEN** the root is removed and the graph is frozen
- **THEN** `contains_root_node()` is false and `get_root_index()` returns `None`

### Requirement: No position is issued twice between two calls of clear
`add_node` SHALL never return a position it has returned before, unless `clear` has wiped the graph since.

#### Scenario: Removing the last node does not free its position
- **WHEN** nodes are added at positions 0 to 3, position 3 is removed, and a node is added
- **THEN** `add_node` returns 4

#### Scenario: A graph with no live node keeps its node table through freeze
- **WHEN** nodes are added at positions 0 to 2, all three are removed, the graph is frozen and unfrozen, and a node is added
- **THEN** `add_node` returns 3

### Requirement: A tombstone reads as absent in every state
In the Dynamic and the Frozen state, a removed node's position SHALL read as absent: `contains_node` is false, `get_node` returns `None`, and the position appears in no result of `get_all_nodes`, `get_edges`, the traversals or the algorithms.

#### Scenario: A frozen tombstone is absent
- **WHEN** position 1 of a four-node graph is removed and the graph is frozen
- **THEN** `contains_node(1)` is false, `get_node(1)` is `None`, and `get_all_nodes()` returns the three remaining payloads

#### Scenario: Algorithm results name no tombstone
- **WHEN** a frozen graph with a tombstone at position 1 is passed to `topological_sort`, `strongly_connected_components`, `articulation_points`, `bridges`, `biconnected_components`, `betweenness_centrality` and `shortest_path`
- **THEN** no returned position, component member or score entry is 1

#### Scenario: is_empty counts live nodes
- **WHEN** every node of a three-node graph is removed, in the Dynamic and in the Frozen state
- **THEN** `is_empty()` is true although the node table holds three slots

### Requirement: Traversal started at a tombstone returns NodeNotFound
`outbound_edges` and `inbound_edges` SHALL return `GraphError::NodeNotFound` with the requested position when the position holds a tombstone, in the Dynamic and in the Frozen state, as they do for a position never issued.

#### Scenario: Traversal from a tombstone fails in both states
- **WHEN** position 1 of a four-node graph is removed, and `outbound_edges(1)` and `inbound_edges(1)` are called before and after freezing
- **THEN** every call returns `Err(GraphError::NodeNotFound(1))`

### Requirement: Counts and the highest live position read live nodes
`number_nodes` SHALL return the number of live nodes and `get_last_index` SHALL return the highest live position, in the Dynamic and in the Frozen state.

#### Scenario: Counting after a removal
- **WHEN** nodes are added at positions 0 to 3 and position 1 is removed, in the Dynamic and in the Frozen state
- **THEN** `number_nodes()` returns 3 and `get_last_index()` returns `Some(3)`

#### Scenario: The highest live position skips trailing tombstones
- **WHEN** nodes are added at positions 0 to 3 and position 3 is removed, in the Dynamic and in the Frozen state
- **THEN** `get_last_index()` returns `Some(2)`

### Requirement: compact_ids reclaims tombstones and returns the mapping
`compact_ids` SHALL, in the Dynamic state, remove every tombstone, renumber the live nodes in their order, remap every edge and the root, and return for each old position its new position or `None`; in the Frozen state it SHALL return `GraphError::GraphIsFrozen` and change nothing.

#### Scenario: Compacting a graph with one tombstone
- **WHEN** nodes A, B and C are at positions 0, 1 and 2 with edges A → C and C → A, B is removed, and `compact_ids` is called
- **THEN** it returns `[Some(0), None, Some(1)]`, `get_node(1)` is C, `contains_edge(0, 1)` and `contains_edge(1, 0)` are true, and the next `add_node` returns 2

#### Scenario: Compacting a graph without tombstones changes nothing
- **WHEN** `compact_ids` is called on a graph with live nodes at positions 0 to 3 and no tombstone
- **THEN** it returns `[Some(0), Some(1), Some(2), Some(3)]` and every node and edge keeps its position

#### Scenario: Compacting a frozen graph is refused
- **WHEN** `compact_ids` is called on a frozen graph
- **THEN** it returns `Err(GraphError::GraphIsFrozen)` and the graph is unchanged

### Requirement: clear wipes the graph and restarts positions at 0
`clear` SHALL drop every node, edge, tombstone and the root, leave the graph empty in the Dynamic state from any state, and make the next `add_node` return 0.

#### Scenario: Clearing a frozen graph
- **WHEN** a frozen graph with nodes at positions 0 to 4 is cleared and a node is added
- **THEN** `is_frozen()` is false and `add_node` returns 0

### Requirement: Every algorithm agrees with the compacted graph
Every algorithm of `GraphAlgorithms` SHALL return, on a frozen graph with tombstones, the result it returns on the same graph after `compact_ids`, mapped back to the original positions through the returned table. An algorithm sizes its arrays by slots and counts live nodes wherever the node count enters a result, as betweenness normalisation does.

#### Scenario: Normalised betweenness counts live nodes
- **WHEN** nodes are added at positions 0 to 4, position 1 is removed, edges 0 → 2, 2 → 3 and 3 → 4 form a path, the graph is frozen, and normalised directed `betweenness_centrality` runs
- **THEN** nodes 2 and 3 score 1/3 each, two shortest paths through each divided by (4 − 1)(4 − 2) = 6 for the four live nodes, and no entry names position 1

#### Scenario: Agreement over generated graphs
- **WHEN** random graphs are generated with random removals, each is frozen once as is and once after `compact_ids`, and every algorithm runs on both
- **THEN** each result on the graph with tombstones equals the result on the compacted graph mapped back through the table
