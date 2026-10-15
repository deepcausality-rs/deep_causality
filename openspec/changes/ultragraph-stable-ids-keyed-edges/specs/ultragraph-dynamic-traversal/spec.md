## ADDED Requirements

### Requirement: Traversal works in the Dynamic state
`outbound_edges` and `inbound_edges` SHALL list a live node's successors and predecessors in the Dynamic state, one entry per edge, and SHALL list the same entries, as a multiset, as in the Frozen state after `freeze`; neither SHALL return `GraphError::GraphNotFrozen`.

#### Scenario: Dynamic listings equal frozen listings
- **WHEN** a graph with edges 0 → 1, 0 → 2, 1 → 2, 2 → 0 and 2 → 3 is traversed before and after freezing
- **THEN** for every node the dynamic and frozen outbound lists hold the same targets, and the dynamic and frozen inbound lists hold the same sources

#### Scenario: One entry per edge
- **WHEN** edges `(0, 1, kind_a)` and `(0, 1, kind_b)` exist
- **THEN** `outbound_edges(0)` yields 1 twice and `inbound_edges(1)` yields 0 twice, in the Dynamic and in the Frozen state

### Requirement: The backward edges are the forward edges reversed
In the Dynamic and in the Frozen state, a node `a` SHALL appear in `inbound_edges(b)` exactly as many times as `b` appears in `outbound_edges(a)`, after every mutation.

#### Scenario: Removing an edge updates both directions
- **WHEN** edges 0 → 1 and 2 → 1 exist and `remove_edge(0, 1)` is called in the Dynamic state
- **THEN** `inbound_edges(1)` yields only 2 and `outbound_edges(0)` yields nothing

#### Scenario: Agreement over generated mutation sequences
- **WHEN** random sequences of node and edge additions and removals run against a reference model of edges as `(source, target, key)` triples
- **THEN** after every step, every node's outbound and inbound lists match the model in both states

### Requirement: remove_node removes the edges into the node
`remove_node(t)` SHALL remove every edge out of `t` and every edge into `t`, so no edge in any state points at a tombstone.

#### Scenario: Incoming edges leave with the node
- **WHEN** edges 0 → 2 and 1 → 2 exist and `remove_node(2)` is called
- **THEN** `outbound_edges(0)` and `outbound_edges(1)` yield nothing, `contains_edge(0, 2)` is false, and `number_edges()` is 0

### Requirement: number_edges counts live edges in the Dynamic state
`number_edges` SHALL return the number of edges between live nodes in the Dynamic state, equal to the count the Frozen state reports after `freeze`.

#### Scenario: Counting after a node removal
- **WHEN** edges 0 → 1, 1 → 2 and 2 → 0 exist and node 1 is removed
- **THEN** `number_edges()` is 1 before freezing and 1 after freezing
