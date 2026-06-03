## ADDED Requirements

### Requirement: Typed-endpoint mixed-graph type

The system SHALL provide a `MixedGraph` type in `deep_causality_topology` whose every edge carries a mark at each of its two endpoints, drawn from `Tail`, `Arrow`, and `Circle`. The pair of endpoint marks SHALL name the edge, covering every edge kind used by DAGs, CPDAGs, MAGs, and PAGs: `(Tail, Arrow)` directed `u → v`, `(Tail, Tail)` undirected `u — v`, `(Arrow, Arrow)` bidirected `u ↔ v`, `(Circle, Arrow)` partially directed `u ∘→ v`, `(Circle, Circle)` nondirected `u ∘—∘ v`, and `(Circle, Tail)` partially undirected `u ∘— v`; the absence of an edge is the empty state. The type SHALL be generic over a node payload and SHALL be usable structurally (over `usize` node indices) when the payload carries no scalar. Edges SHALL carry no weight.

#### Scenario: Every endpoint-mark combination is representable
- **WHEN** edges with endpoint marks `(Tail, Arrow)`, `(Tail, Tail)`, `(Arrow, Arrow)`, `(Circle, Arrow)`, `(Circle, Circle)`, and `(Circle, Tail)` are added
- **THEN** each edge reports its exact endpoint marks and is classified as the corresponding kind (directed, undirected, bidirected, partially directed, nondirected, partially undirected)

#### Scenario: Marks are readable in either node order
- **WHEN** an edge between `u` and `v` is queried as `(u, v)` and as `(v, u)`
- **THEN** the endpoint marks reported are consistent with a single edge, swapped to match the query order

### Requirement: Endpoint invariant

The graph SHALL maintain, for every unordered node pair, exactly one state: no edge, a directed arc in one direction, a directed arc in the other direction, or an undirected edge. The two endpoint marks of any pair SHALL always be mutually consistent (both set for an edge, both empty for no edge). No operation SHALL leave a pair in a half-set or contradictory state.

#### Scenario: A pair holds at most one edge
- **WHEN** an edge is added between a pair that already has an edge
- **THEN** the operation either replaces the prior edge as specified or returns an error, and never records two simultaneous edges for the pair

#### Scenario: Marks stay symmetric after mutation
- **WHEN** any edge is added, removed, or reoriented
- **THEN** the endpoint marks read from both directions of the pair remain consistent with a single well-defined edge state

### Requirement: Construction and node/edge population

The system SHALL allow constructing a `MixedGraph` with a fixed number of nodes and adding directed arcs and undirected edges by node index. Operations referencing a node index outside the graph SHALL return an error rather than panic.

#### Scenario: Build a small CPDAG
- **WHEN** a graph of three nodes is created and an arc `0 → 1` and an undirected edge `1 — 2` are added
- **THEN** the graph reports three nodes, one arc, and one undirected edge

#### Scenario: Out-of-range endpoint is rejected
- **WHEN** an edge is added referencing a node index that does not exist
- **THEN** the operation returns an error and the graph is unchanged

### Requirement: Structural queries

The system SHALL answer, for any node: its parents (sources of incoming arcs), its children (targets of outgoing arcs), and its undirected neighbors. The system SHALL report whether two nodes are adjacent (by an arc in either direction or an undirected edge), and SHALL enumerate all directed arcs and all undirected edges.

#### Scenario: Parents and neighbors are reported separately
- **WHEN** a node has an incoming arc from `a` and an undirected edge to `b`
- **THEN** `a` appears among its parents and not its undirected neighbors, and `b` appears among its undirected neighbors and not its parents

#### Scenario: Adjacency spans both edge kinds
- **WHEN** two nodes are joined by either a directed arc or an undirected edge
- **THEN** the adjacency query reports them as adjacent

### Requirement: Edge orientation

The system SHALL allow orienting an existing undirected edge `u — v` into a directed arc (`u → v` or `v → u`) as a single operation that preserves the endpoint invariant. Orienting an edge that is not undirected SHALL return an error.

#### Scenario: Orient an undirected edge
- **WHEN** an undirected edge `u — v` is oriented to `u → v`
- **THEN** the pair reports the arc `u → v`, `u` is a parent of `v`, and the pair is no longer an undirected neighbor relation

#### Scenario: Orienting a non-undirected edge is rejected
- **WHEN** orientation is requested on a pair that has a directed arc or no edge
- **THEN** the operation returns an error and the graph is unchanged

### Requirement: Arc-projection acyclicity and topological order

The system SHALL compute a topological order over the directed-arc projection of the graph (ignoring undirected edges) and SHALL report whether that projection contains a directed cycle. When the arc projection is acyclic, the topological order SHALL list every node such that each arc points from an earlier to a later position; when it contains a cycle, the system SHALL report the absence of a topological order rather than returning an invalid one.

#### Scenario: Acyclic arc projection yields a valid order
- **WHEN** the arc projection is acyclic
- **THEN** a topological order is returned in which every arc `u → v` places `u` before `v`

#### Scenario: Cyclic arc projection is detected
- **WHEN** the arc projection contains a directed cycle
- **THEN** the cycle is reported and no topological order is returned

#### Scenario: Undirected edges do not affect acyclicity
- **WHEN** undirected edges are added to an otherwise acyclic arc projection
- **THEN** the projection is still reported acyclic and a topological order is still returned

### Requirement: Higher-kinded-type and comonad integration

The system SHALL integrate `MixedGraph` with the crate's higher-kinded-type scaffolding to full parity with `Graph`/`Hypergraph`: an HKT witness, a `MixedGraphTopology` trait, and a comonadic interface (`extract`, `extend`, `duplicate`) over a node-payload focus (cursor). `extract` SHALL return the focused node's payload; `extend` SHALL produce a new graph whose payload at each node is a neighborhood-aware function evaluated with that node as the focus.

#### Scenario: Witness and topology trait are available
- **WHEN** a consumer uses `MixedGraph` through the crate's HKT witness and topology trait
- **THEN** the type composes with the same higher-kinded-type machinery used by `Graph` and `Hypergraph`

#### Scenario: Comonadic extract returns the focused payload
- **WHEN** the cursor is set to a node and `extract` is called
- **THEN** the focused node's payload is returned

#### Scenario: Comonadic extend respects the comonad laws
- **WHEN** `extend`/`duplicate` is applied
- **THEN** the result satisfies the comonad laws (left identity, right identity, associativity) as exercised by the test suite

### Requirement: PAG endpoint orientation

The system SHALL support orienting and reorienting any endpoint of an existing edge to any mark (`Tail`, `Arrow`, `Circle`), so that the full DAG/CPDAG/MAG/PAG edge calculus is expressible and mutable. Every such operation SHALL preserve the endpoint invariant. Setting an endpoint of a non-existent edge SHALL return an error.

#### Scenario: A circle endpoint can be oriented to an arrowhead
- **WHEN** a partially directed edge `u ∘→ v` has its `u` endpoint set from `Circle` to `Tail`
- **THEN** the edge becomes the directed arc `u → v`

#### Scenario: An edge can be made bidirected
- **WHEN** both endpoints of an edge are set to `Arrow`
- **THEN** the edge is reported as bidirected `u ↔ v`
