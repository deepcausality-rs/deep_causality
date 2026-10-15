<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# ultragraph-keyed-edges Specification

## Purpose
Defines edge identity in `ultragraph`: an edge is its source, its target and the key its payload returns through `EdgeKey`, a trait declared in the dependency-free crate `ultragraph_traits`; keyed and unkeyed add, remove and contains; the unit and primitive numeric payloads with `Key = ()`; the frozen adjacency sorted by target then key.

## Requirements
### Requirement: ultragraph_traits holds EdgeKey and depends on nothing
The workspace SHALL contain a crate `ultragraph_traits` that declares the trait `EdgeKey`, with an associated type `Key: Copy + Ord` and a method `edge_key(&self) -> Self::Key`, declares no dependency of any kind, builds without `std`, and is re-exported by `ultragraph` at its root.

#### Scenario: The manifest declares nothing
- **WHEN** the `[dependencies]` table of `ultragraph_traits/Cargo.toml` is read
- **THEN** it is absent or empty, and the crate opts into the workspace lints

#### Scenario: One trait under two paths
- **WHEN** a type implements `ultragraph_traits::EdgeKey`
- **THEN** it satisfies the bound `ultragraph::EdgeKey` with no further impl

#### Scenario: The crate builds without std
- **WHEN** `ultragraph_traits` is built for a target without `std`
- **THEN** the build succeeds

### Requirement: Every graph payload implements EdgeKey
`UltraGraphContainer`, `DynamicGraph` and `CsmGraph` SHALL bound their edge payload as `W: Clone + Default + EdgeKey`.

#### Scenario: The weight and unit payloads compile
- **WHEN** graphs are built with `W` set to `()`, `u64`, `i64` and `f64`
- **THEN** they compile and accept `add_edge`

### Requirement: The unit and primitive numeric types take an empty key
`ultragraph_traits` SHALL implement `EdgeKey` with `Key = ()` for `()`, `bool`, `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `f32` and `f64`.

A payload of one of these types is a weight or nothing, so a graph carrying it holds at most one edge per ordered pair.

#### Scenario: A second weight updates the first
- **WHEN** `add_edge(0, 1, 10_u64)` and then `add_edge(0, 1, 15_u64)` are called on a dynamic graph
- **THEN** `number_edges()` is 1 and `get_edges(0)` returns `[(1, &15)]`

#### Scenario: A unit payload collapses a repeated edge
- **WHEN** `add_edge(0, 1, ())` is called twice
- **THEN** `number_edges()` is 1

### Requirement: An edge is identified by its source, its target and its key
A graph SHALL hold at most one edge per source, target and key, and `add_edge(a, b, w)` SHALL insert the edge `(a, b, w.edge_key())` or, when that edge exists, replace its payload with `w`, leaving every edge between `a` and `b` with another key in place.

#### Scenario: Edges with different keys coexist
- **WHEN** a payload type whose key is a two-variant kind is used, and `add_edge(0, 1, kind_a)` and `add_edge(0, 1, kind_b)` are called
- **THEN** `number_edges()` is 2 and `get_edges(0)` returns both payloads

#### Scenario: An edge with the same key is replaced
- **WHEN** a payload struct carries a kind as its key and a weight as data, and `add_edge(0, 1, Edge { kind: kind_a, weight: 3 })` is followed by `add_edge(0, 1, Edge { kind: kind_a, weight: 7 })`
- **THEN** `number_edges()` is 1 and the payload read back is `Edge { kind: kind_a, weight: 7 }`

#### Scenario: An edge to or from an absent node is refused
- **WHEN** `add_edge` names a position never issued or a tombstone as either end
- **THEN** it returns `Err(GraphError::EdgeCreationError { source, target })` with the two positions, and no edge is added

### Requirement: remove_edge removes every key, remove_edge_keyed removes one
`remove_edge(a, b)` SHALL remove every edge from `a` to `b` whatever its key, and `remove_edge_keyed(a, b, k)` SHALL remove the edge from `a` to `b` with key `k` alone; each SHALL return `GraphError::NodeNotFound` when `a` holds no live node and `GraphError::EdgeNotFoundError` when no edge it would remove exists.

#### Scenario: Removing a pair removes both keys
- **WHEN** edges `(0, 1, kind_a)` and `(0, 1, kind_b)` exist and `remove_edge(0, 1)` is called
- **THEN** `contains_edge(0, 1)` is false and `number_edges()` is 0

#### Scenario: Removing one key leaves the other
- **WHEN** edges `(0, 1, kind_a)` and `(0, 1, kind_b)` exist and `remove_edge_keyed(0, 1, kind_a)` is called
- **THEN** `contains_edge_keyed(0, 1, kind_a)` is false, `contains_edge_keyed(0, 1, kind_b)` is true, and `number_edges()` is 1

#### Scenario: Removing an absent edge is reported
- **WHEN** `remove_edge(0, 1)` is called with no edge from 0 to 1, and `remove_edge_keyed(0, 1, kind_b)` is called with only `(0, 1, kind_a)` present
- **THEN** each returns `Err(GraphError::EdgeNotFoundError { source: 0, target: 1 })`

#### Scenario: Removing from an absent source is reported
- **WHEN** `remove_edge(5, 1)` is called on a graph whose position 5 is a tombstone
- **THEN** it returns `Err(GraphError::NodeNotFound(5))`

### Requirement: contains_edge tests any key, contains_edge_keyed tests one
`contains_edge(a, b)` SHALL be true when an edge from `a` to `b` exists with any key and both ends are live, and `contains_edge_keyed(a, b, k)` SHALL be true when the edge with key `k` exists and both ends are live, in the Dynamic and in the Frozen state.

#### Scenario: Keyed and unkeyed tests agree with the edges present
- **WHEN** only `(0, 1, kind_a)` exists, before and after freezing
- **THEN** `contains_edge(0, 1)` is true, `contains_edge_keyed(0, 1, kind_a)` is true, `contains_edge_keyed(0, 1, kind_b)` is false, and `contains_edge(1, 0)` is false

### Requirement: The frozen adjacency is sorted by target, then key
In the Frozen state, the outgoing and the incoming edges of every node SHALL be ordered by the other end's position, then by key, and `contains_edge` SHALL return the same answer on either side of each search threshold.

#### Scenario: Edges inserted out of order read back sorted
- **WHEN** edges `(0, 3, kind_b)`, `(0, 1, kind_b)`, `(0, 3, kind_a)` and `(0, 1, kind_a)` are added in that order and the graph is frozen
- **THEN** `get_edges(0)` returns the targets and keys in the order `(1, kind_a)`, `(1, kind_b)`, `(3, kind_a)`, `(3, kind_b)`

#### Scenario: Lookups across the search threshold
- **WHEN** a hub has 63, 64 and 65 outgoing edges in three graphs, each frozen
- **THEN** `contains_edge` is true for every present target and false for an absent one in all three

#### Scenario: Sorting across the radix threshold
- **WHEN** a hub has 127, 128 and 129 outgoing edges added in descending target order, in three graphs, each frozen
- **THEN** `get_edges` of the hub returns ascending targets in all three
