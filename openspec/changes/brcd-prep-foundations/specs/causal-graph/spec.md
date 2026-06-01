## ADDED Requirements

### Requirement: Structural layer is precision-agnostic

The causal-graph layer SHALL carry no floating-point scalars; it represents structure over node indices and edges only. It SHALL therefore take no precision type parameter and SHALL compose with any precision `T` used elsewhere in the pipeline.

#### Scenario: Graph operations require no precision parameter
- **WHEN** the PDAG/CPDAG type, Meek completion, the validity check, and MEC sizing are used
- **THEN** they operate on graph structure alone and expose no `RealField` type parameter

#### Scenario: Composes with any downstream precision
- **WHEN** a downstream algorithm runs at `f32`, `f64`, or `Float106`
- **THEN** it reuses the same graph layer unchanged

### Requirement: PDAG/CPDAG representation

The system SHALL provide a partially directed acyclic graph type over a fixed node set that holds both directed arcs and undirected edges, and SHALL expose the parents of a node, the neighbors of a node, the set of arcs, and the set of undirected edges. The type SHALL distinguish a directed arc from an undirected edge in adjacency queries.

#### Scenario: Construct from arcs and undirected edges
- **WHEN** a PDAG is built from a node set, a list of arcs, and a list of undirected edges
- **THEN** the arcs, undirected edges, and per-node parents are reported consistently with the construction

#### Scenario: Adjacency distinguishes arc from undirected edge
- **WHEN** two nodes are connected by a directed arc
- **THEN** the query reports them adjacent and reports the edge as directed, not undirected

### Requirement: Directed-graph operations via ultragraph

The system SHALL provide topological order and acyclicity checks over the directed-arc projection of a PDAG, reusing `ultragraph` for storage and traversal. A parents/predecessors accessor SHALL be available on the directed projection.

#### Scenario: Acyclic arc set yields a topological order
- **WHEN** the arc projection is acyclic
- **THEN** a valid topological order over the nodes is returned

#### Scenario: Cyclic arc set is reported as cyclic
- **WHEN** the arc projection contains a cycle
- **THEN** the acyclicity check reports a cycle and no topological order is produced

### Requirement: Meek orientation rules

The system SHALL apply the Meek orientation rules to complete a PDAG into its CPDAG, and equivalently to convert a fully directed DAG into its CPDAG. Completion SHALL be idempotent.

#### Scenario: Known PDAG completes to the expected CPDAG
- **WHEN** a PDAG with a known completion is supplied
- **THEN** Meek completion returns the expected CPDAG

#### Scenario: Arcs-only DAG is unchanged by completion
- **WHEN** the input is fully directed and acyclic with no undirected edges
- **THEN** completion leaves the arc set unchanged

#### Scenario: Completion is idempotent
- **WHEN** completion is applied twice
- **THEN** the second application produces the same graph as the first

### Requirement: Unshielded-collider validity check

The system SHALL determine whether a completed PDAG introduces a new unshielded collider `a → t ← b` (with `a` and `b` non-adjacent) at a target node, relative to a supplied baseline parent set for that node.

#### Scenario: New unshielded collider is flagged
- **WHEN** an orientation creates `a → t ← b` with `a` and `b` non-adjacent and not both present in the baseline parents of `t`
- **THEN** the check reports a new unshielded collider

#### Scenario: Pre-existing collider is not flagged as new
- **WHEN** the only colliders at `t` are already present in the baseline parent set
- **THEN** the check reports no new unshielded collider

### Requirement: Markov-equivalence-class size, trivial arcs-only case

The system SHALL return the size of the Markov equivalence class of a graph and a representative DAG from that class. For a fully directed acyclic input the size SHALL be 1 and the representative SHALL equal the input graph. The API SHALL be shaped so that a later uniform sampler over non-trivial classes can be added without changing the signature.

#### Scenario: Fully directed input has class size one
- **WHEN** a fully directed acyclic graph is supplied
- **THEN** the equivalence-class size is 1 and the representative DAG equals the input

#### Scenario: Extension point is documented for the full sampler
- **WHEN** a non-trivial equivalence class is supplied in a future change
- **THEN** the existing API admits a uniform sampler without a breaking signature change
