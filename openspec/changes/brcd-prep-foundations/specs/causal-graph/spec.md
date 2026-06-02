## ADDED Requirements

### Requirement: Structural layer is precision-agnostic

The causal-graph operations SHALL carry no floating-point scalars; they operate over structure (node indices and typed edges) only. They SHALL therefore take no precision type parameter and SHALL compose with any precision `T` used elsewhere in the pipeline.

#### Scenario: Graph operations require no precision parameter
- **WHEN** Meek completion, the validity check, and MEC sizing are used
- **THEN** they operate on graph structure alone and expose no `RealField` type parameter

#### Scenario: Composes with any downstream precision
- **WHEN** a downstream algorithm runs at `f32`, `f64`, or `Float106`
- **THEN** it reuses the same graph operations unchanged

### Requirement: Operations consume the shared mixed-graph type

The causal-graph operations SHALL be implemented over `deep_causality_topology::MixedGraph` — the typed-endpoint graph delivered by the `mixed-graph` capability — rather than defining a separate graph type. They SHALL use its directed-arc projection (`parents`/`children`), undirected neighbors, per-kind edge enumeration, and built-in topological sort / acyclicity checks.

#### Scenario: A CPDAG is represented as a MixedGraph
- **WHEN** a CPDAG is built as a `MixedGraph` from directed arcs and undirected edges
- **THEN** the causal-graph operations read its arcs, undirected edges, and per-node parents directly, with no separate PDAG type

#### Scenario: Acyclicity uses the graph's own projection
- **WHEN** an operation needs the arc-projection topological order or a cycle check
- **THEN** it uses `MixedGraph`'s `topological_sort` / `has_cycle`, not a separate directed-graph store

### Requirement: Meek orientation rules

The system SHALL apply the Meek orientation rules to complete a `MixedGraph` PDAG into its CPDAG, and equivalently to convert a fully directed DAG into its CPDAG. Completion SHALL be idempotent.

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
