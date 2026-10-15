<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# ultragraph-projection-view Specification

## Purpose
Defines the projection view of `ultragraph`: the `Projection` trait of a node test and an edge test, `project` over the frozen graph, the `ParallelEdges` mode chosen per projection, the simple projection every direct algorithm call runs on, and what a view's reads, traversals and algorithms answer.

## Requirements
### Requirement: Projection is a node test and an edge test
`ultragraph` SHALL declare a trait `Projection<N, W>` with `node(&self, node: &N) -> bool` and `edge(&self, edge: &W) -> bool`, and SHALL implement it for every pair `(F, G)` where `F: Fn(&N) -> bool` and `G: Fn(&W) -> bool`.

#### Scenario: A pair of closures is a projection
- **WHEN** `(|n: &u32| *n > 1, |w: &u64| *w < 10)` is passed where a `Projection<u32, u64>` is expected
- **THEN** it compiles and its tests are the two closures

#### Scenario: A caller's own type is a projection
- **WHEN** an enum with one variant per scope implements `Projection<N, W>`
- **THEN** `project` accepts it with every test statically dispatched

### Requirement: ParallelEdges decides how parallel edges count
`ultragraph` SHALL declare an enum `ParallelEdges` with the variants `Merge`, under which the edges between an ordered pair of nodes count as one connection and a weighted path takes the lightest of them by `W`'s order, and `Distinct`, under which every edge counts as its own link.

Undirected algorithms (articulation points, bridges, biconnected components, undirected betweenness) count the edges between an unordered pair `{u, v}` as one undirected connection under `Merge`, and as `max(f, r)` undirected edges under `Distinct`, where `f` counts the edges `u → v` and `r` the edges `v → u`. Under `Distinct`, bridges, articulation points and biconnected components return today's results; undirected betweenness deduplicates neighbours today, so its results today are those under `Merge`.

#### Scenario: Bridges under each mode
- **WHEN** a frozen graph has edges `(0, 1, kind_a)`, `(0, 1, kind_b)` and `(1, 2, kind_a)`, and `bridges` runs on a projection that passes everything
- **THEN** under `Merge` it returns `[(0, 1), (1, 2)]`, and under `Distinct` it returns `[(1, 2)]`

#### Scenario: Opposite edges count once in both modes
- **WHEN** a frozen graph has edges 0 → 1, 1 → 0 and 1 → 2 and `bridges` runs under each mode
- **THEN** both return `[(0, 1), (1, 2)]`

#### Scenario: Distinct counts the larger direction
- **WHEN** a frozen graph has edges `(0, 1, kind_a)`, `(0, 1, kind_b)`, `(1, 0, kind_a)` and `(1, 2, kind_a)`, and `bridges` runs under `Distinct`
- **THEN** it returns `[(1, 2)]`, because `{0, 1}` counts `max(2, 1) = 2` undirected edges

#### Scenario: Directed betweenness under each mode
- **WHEN** a frozen graph has edges `(0, 1, kind_a)`, `(0, 1, kind_b)`, 0 → 2, 1 → 3 and 2 → 3, and unnormalised directed `betweenness_centrality` runs under each mode
- **THEN** under `Merge` nodes 1 and 2 score 1/2 each, because the two shortest paths from 0 to 3 pass one through each; and under `Distinct` node 1 scores 2/3 and node 2 scores 1/3, because three shortest paths exist and two pass through 1

#### Scenario: Undirected betweenness under each mode
- **WHEN** the graph of the previous scenario runs unnormalised undirected `betweenness_centrality` under each mode
- **THEN** under `Merge` every node scores 1/2, as every node does today, because the pairs `{0, 3}` and `{1, 2}` each have two shortest paths, one through each of their two middle nodes; and under `Distinct` nodes 0 and 1 score 2/3 and nodes 2 and 3 score 1/3, because the two edges between 0 and 1 give each of those pairs three shortest paths, two of which cross the doubled edge

#### Scenario: The weighted path takes the lightest parallel edge in both modes
- **WHEN** a frozen graph has two edges 0 → 1 whose payloads carry different keys and the weights 5 and 2, in a payload type ordered and added by its weight, and `shortest_weighted_path(0, 1)` runs under each mode
- **THEN** both return the path `[0, 1]` with a length of weight 2

### Requirement: project returns a view of the frozen graph
`project(projection, mode)` SHALL return, in the Frozen state, a view that borrows the graph and runs every algorithm of `GraphAlgorithms` and both traversals over the nodes and edges the projection passes, and SHALL return `GraphError::GraphNotFrozen` in the Dynamic state.

#### Scenario: Projecting a dynamic graph is refused
- **WHEN** `project` is called on a graph that is not frozen
- **THEN** it returns `Err(GraphError::GraphNotFrozen)`

### Requirement: A projection passes nodes and edges by their tests
In a projection, a node SHALL pass when it is live and the node test holds for its payload, and an edge SHALL pass when the edge test holds for its payload and both of its ends pass; results SHALL name positions as `add_node` issued them.

#### Scenario: An edge between scopes fails with its end
- **WHEN** nodes 0 and 1 carry zone A, node 2 carries zone B, edges 0 → 1 and 1 → 2 exist, and the node test passes zone A
- **THEN** on the projection `outbound_edges(1)` yields nothing and `is_reachable(0, 2)` is false, while on the graph itself `is_reachable(0, 2)` is true

#### Scenario: An edge test filters by payload
- **WHEN** edges 0 → 1 with `valid_until = None` and 1 → 2 with `valid_until = Some(5)` exist, and the edge test passes open edges
- **THEN** on the projection `shortest_path(0, 2)` is `None` and `shortest_path(0, 1)` is `[0, 1]`

#### Scenario: Positions are not renumbered
- **WHEN** the node test fails node 0 of a chain 0 → 1 → 2 → 3
- **THEN** `topological_sort` on the projection returns `[1, 2, 3]`

### Requirement: A view's reads and traversals list its passing elements
On a projection view, the node reads of `GraphView` (`contains_node`, `get_node`, `get_all_nodes`, `number_nodes`, `get_last_index`, `is_empty`, and the three root reads, which return the graph's root only when it passes) SHALL answer for the passing nodes only, and `contains_edge`, `contains_edge_keyed`, `get_edges`, `number_edges`, `outbound_edges` and `inbound_edges` SHALL answer for, and list one entry per, passing edge under either mode; `is_frozen` SHALL be true.

The mode decides how algorithms count parallel edges, not what reads list. A representative of several parallel edges would need an order on `W`, which only `shortest_weighted_path` requires.

#### Scenario: Reads on a view under Merge
- **WHEN** nodes 0 and 1 carry zone A, node 2 carries zone B, edges `(0, 1, kind_a)`, `(0, 1, kind_b)` and 1 → 2 exist, and a view passes zone A under `Merge`
- **THEN** `number_nodes()` is 2, `contains_node(2)` is false, `get_edges(0)` lists both edges to 1, `outbound_edges(0)` yields 1 twice, `number_edges()` is 2, and `outbound_edges(1)` yields nothing

### Requirement: Algorithms on the graph run on the simple projection
Every algorithm, read and traversal called on a frozen graph directly SHALL return what it returns on the simple projection: a view whose node and edge tests pass everything, under `ParallelEdges::Merge`.

#### Scenario: Direct calls equal the simple projection
- **WHEN** random frozen graphs with parallel edges of several keys are generated and every algorithm, read and traversal runs directly and on the simple projection
- **THEN** each pair of results is equal

### Requirement: A projection's results equal those of the graph it selects
Every algorithm on a projection under a mode SHALL return the result it returns, under the same mode, on a projection that passes everything of a graph built from the nodes and edges the first projection passes, at the same positions.

#### Scenario: Agreement over generated graphs and projections
- **WHEN** random frozen graphs, random node and edge tests and both modes are generated, and each projection's results are compared, under its mode, with those of a graph built from its passing elements by an independent filter
- **THEN** every result is equal

#### Scenario: Results that do not depend on the mode
- **WHEN** reachability, cycle detection, topological sort, strongly connected components, unweighted shortest path, articulation points and biconnected components run on graphs with parallel edges under each mode
- **THEN** each returns the same result under `Merge` and `Distinct`
