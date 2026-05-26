# ultragraph-biconnectivity Specification

## Purpose

Provide deterministic, linear-time biconnectivity analysis (articulation points, bridges, and biconnected components) over the undirected view of a `CsmGraph` in the `ultragraph` crate.

## Requirements

### Requirement: Undirected view of CsmGraph

For the purpose of biconnectivity analysis, the system SHALL interpret a `CsmGraph` as undirected. Each stored directed edge `(u, v)` SHALL contribute the undirected edge `{u, v}` to the analysis. The algorithms MUST NOT expose a directional toggle and MUST NOT report different results for the same vertex set when the only difference is edge orientation.

#### Scenario: Directed input is symmetrized

- **WHEN** a `CsmGraph` containing only the directed edge `(0, 1)` is analyzed
- **THEN** the analysis treats `{0, 1}` as a single undirected edge, identical to a graph containing both `(0, 1)` and `(1, 0)`

#### Scenario: Parallel reverse edges are not double-counted

- **WHEN** a `CsmGraph` contains both `(0, 1)` and `(1, 0)`
- **THEN** the analysis treats them as the single undirected edge `{0, 1}` and does not classify it as a multi-edge cycle

### Requirement: articulation_points API

The `CsmGraph` type SHALL expose `fn articulation_points(&self) -> Result<Vec<usize>, GraphError>` returning every vertex whose removal (together with its incident undirected edges) increases the number of connected components of the undirected view. The result MUST contain each articulation point exactly once. Vertex order within the result is unspecified but MUST be deterministic for a given input.

#### Scenario: Linear chain has interior cut vertices

- **WHEN** the graph is the path `0 - 1 - 2 - 3 - 4`
- **THEN** `articulation_points()` returns exactly `{1, 2, 3}`

#### Scenario: Cycle has no articulation points

- **WHEN** the graph is the simple cycle `0 - 1 - 2 - 0`
- **THEN** `articulation_points()` returns an empty vector

#### Scenario: Two cycles joined at a single vertex

- **WHEN** the graph consists of cycle `{0, 1, 2}` and cycle `{2, 3, 4}` sharing vertex `2`
- **THEN** `articulation_points()` returns exactly `{2}`

#### Scenario: Isolated and empty graphs

- **WHEN** the graph has zero vertices, or every vertex is isolated
- **THEN** `articulation_points()` returns an empty vector and does not error

### Requirement: bridges API

The `CsmGraph` type SHALL expose `fn bridges(&self) -> Result<Vec<(usize, usize)>, GraphError>` returning every undirected edge whose removal increases the number of connected components of the undirected view. Each bridge MUST appear exactly once in the result. For each returned tuple `(u, v)`, the implementation MUST emit the endpoints in a canonical order (`u < v`) so that callers can compare results without normalizing.

#### Scenario: Every edge of a tree is a bridge

- **WHEN** the graph is the tree with edges `{(0,1), (1,2), (1,3)}`
- **THEN** `bridges()` returns exactly `{(0,1), (1,2), (1,3)}`, each tuple in `(min, max)` order

#### Scenario: Cycle has no bridges

- **WHEN** the graph is the simple cycle `0 - 1 - 2 - 0`
- **THEN** `bridges()` returns an empty vector

#### Scenario: Two cycles joined by a single edge

- **WHEN** cycle `{0, 1, 2}` and cycle `{3, 4, 5}` are joined by the edge `(2, 3)`
- **THEN** `bridges()` returns exactly `{(2, 3)}`

#### Scenario: Empty and edgeless graphs

- **WHEN** the graph has zero edges
- **THEN** `bridges()` returns an empty vector and does not error

### Requirement: biconnected_components API

The `CsmGraph` type SHALL expose `fn biconnected_components(&self) -> Result<Vec<Vec<usize>>, GraphError>` returning the partition of edges of the undirected view into maximal biconnected (2-vertex-connected) subgraphs, represented by the set of vertices participating in each component. The implementation MUST guarantee:

- Every undirected edge of the graph appears in exactly one returned component.
- Every articulation point appears in two or more returned components; non-articulation vertices appear in exactly one component.
- A bridge `{u, v}` is reported as a component containing exactly `{u, v}`.
- Vertices within each returned component are sorted ascending, and components are ordered deterministically for a given input.
- Isolated vertices (degree 0) are NOT included in any returned component.

#### Scenario: Cycle is one biconnected component

- **WHEN** the graph is the simple cycle `0 - 1 - 2 - 0`
- **THEN** `biconnected_components()` returns exactly one component with vertex set `{0, 1, 2}`

#### Scenario: Bow-tie graph splits at the cut vertex

- **WHEN** cycle `{0, 1, 2}` and cycle `{2, 3, 4}` share vertex `2`
- **THEN** `biconnected_components()` returns exactly two components: `{0, 1, 2}` and `{2, 3, 4}`; vertex `2` (the articulation point) appears in both

#### Scenario: Bridge between two cycles

- **WHEN** cycle `{0, 1, 2}` and cycle `{3, 4, 5}` are joined by edge `(2, 3)`
- **THEN** `biconnected_components()` returns three components: `{0, 1, 2}`, `{2, 3}`, `{3, 4, 5}`

#### Scenario: Tree edges are reported as bridge components

- **WHEN** the graph is the tree with edges `{(0,1), (1,2), (1,3)}`
- **THEN** `biconnected_components()` returns three two-vertex components: `{0, 1}`, `{1, 2}`, `{1, 3}`

#### Scenario: Empty and isolated-only graphs

- **WHEN** the graph has zero vertices, or every vertex is isolated
- **THEN** `biconnected_components()` returns an empty vector and does not error

### Requirement: Complexity guarantee

Each of the three APIs MUST run in O(V + E) time and O(V) auxiliary space on the undirected view, where V is the vertex count and E is the count of distinct undirected edges after symmetrization. The implementation MUST NOT allocate per-edge auxiliary structures of size greater than O(V + E).

#### Scenario: Linear scaling on large sparse graphs

- **WHEN** any of the three APIs is invoked on a sparse graph with V = 10,000 and E ≈ 30,000
- **THEN** the implementation completes in a single DFS pass with work proportional to V + E (no nested per-vertex sweep)

### Requirement: Cross-API consistency

The three APIs MUST agree on the same input. Specifically:

- A vertex appears in `articulation_points()` if and only if it appears in two or more components of `biconnected_components()`.
- An undirected edge `{u, v}` appears in `bridges()` if and only if `biconnected_components()` contains a component whose vertex set is exactly `{u, v}`.

#### Scenario: Articulation points derivable from biconnected components

- **WHEN** `articulation_points()` and `biconnected_components()` are called on the same `CsmGraph`
- **THEN** the set of articulation points equals the set of vertices that appear in two or more biconnected components

#### Scenario: Bridges derivable from biconnected components

- **WHEN** `bridges()` and `biconnected_components()` are called on the same `CsmGraph`
- **THEN** the multiset of bridges equals the multiset of biconnected components whose vertex set has cardinality exactly two

### Requirement: Error and boundary behavior

The three APIs SHALL return `Ok(...)` for every well-formed `CsmGraph`, including the empty graph, single-vertex graphs, fully disconnected graphs, and graphs with self-loops. Self-loops `(v, v)` MUST be ignored by all three algorithms (they cannot affect biconnectivity). The APIs MUST return `Err(GraphError::...)` only for the same precondition violations that existing structural algorithms (`find_cycle`, `strongly_connected_components`) on `CsmGraph` already surface — no new error variants are introduced.

#### Scenario: Self-loops are ignored

- **WHEN** the graph contains a self-loop `(0, 0)` plus the cycle `{0, 1, 2}`
- **THEN** the three APIs return results identical to the same graph without the self-loop

#### Scenario: Disconnected graph is analyzed per component

- **WHEN** the graph is the disjoint union of cycle `{0, 1, 2}` and edge `{3, 4}`
- **THEN** `articulation_points()` returns `{}`, `bridges()` returns `{(3, 4)}`, and `biconnected_components()` returns `{{0, 1, 2}, {3, 4}}`
