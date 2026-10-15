## ADDED Requirements

### Requirement: materialize copies a projection into a frozen graph of its own
`materialize` on a projection view SHALL return an owned graph in the Frozen state that holds the nodes and edges the projection passes, at the source's positions, with every failing node reading as absent, and whose `ParallelEdges` mode is the view's, so its algorithms, reads and traversals answer as the view's do.

#### Scenario: Positions survive materialisation
- **WHEN** a view passes nodes 1, 2 and 3 of a chain 0 → 1 → 2 → 3 and is materialised
- **THEN** the copy's `topological_sort` returns `[1, 2, 3]`, `contains_node(0)` is false, and `get_node(2)` is the source's node 2

#### Scenario: The copy answers as the view does
- **WHEN** random frozen graphs, random tests and both modes are generated, and each view is materialised
- **THEN** every algorithm, read and traversal returns the same result on the view and on its materialised copy

#### Scenario: A Distinct copy keeps its parallel edges
- **WHEN** a view under `Distinct` passes the edges `(0, 1, kind_a)`, `(0, 1, kind_b)` and `(1, 2, kind_a)` and is materialised
- **THEN** `bridges()` on the copy returns `[(1, 2)]`, as on the view, and `number_edges()` is 3

### Requirement: A materialised copy does not follow the source
A materialised copy SHALL be independent of its source: later mutations of the source SHALL NOT change it, and dropping the source SHALL NOT invalidate it.

#### Scenario: The source changes after materialisation
- **WHEN** the source is unfrozen after materialisation and receives `add_edge(1, 3, w)`
- **THEN** the copy's `contains_edge(1, 3)` is still false
