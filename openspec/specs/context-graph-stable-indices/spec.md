<!--
SPDX-License-Identifier: MIT
Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
-->

# context-graph-stable-indices Specification

## Purpose
Defines how the context graph of `deep_causality_context` behaves on permanent node positions: `freeze` and `unfreeze` keep every stored index, index entries naming a removed node are dropped at removal, neighbour listings work in both states, and one relation per pair is kept through the store.

## Requirements
### Requirement: Freezing and unfreezing keep every index the context stores
`Context::freeze` and `Context::unfreeze` SHALL convert the base graph without changing any index the context stores, so `get_node_index_by_id`, `get_data_index` and `get_time_index` return the same index before and after either call.

#### Scenario: An index survives a removal and a freeze
- **WHEN** data contextoids with ids 10, 20 and 30 are added at indices 0, 1 and 2, id 20 is removed, and the context is frozen
- **THEN** `get_node_index_by_id(30)` returns `Some(2)`, `get_data_by_id(30)` returns the payload stored with id 30, and `get_node_index_by_id(20)` returns `None`

#### Scenario: Stored data and time indices follow their nodes
- **WHEN** `set_data_index(1, i30, true)` and `set_time_index(3, i10, true)` are set, id 20 is removed, and the context is frozen and unfrozen
- **THEN** `get_data_index(&1, true)` returns `Some(&i30)` and `get_time_index(&3, true)` returns `Some(&i10)`

### Requirement: Index entries naming a removed node are dropped at removal
`Context::remove_node`, and the release of a base-graph node by `Context::apply`, SHALL drop every entry of the current and previous data and time index maps whose value is the removed node's index.

#### Scenario: The entries go before any freeze
- **WHEN** `set_data_index(2, i20, false)` and `set_time_index(4, i20, false)` are set and id 20 is removed
- **THEN** `get_data_index(&2, false)` and `get_time_index(&4, false)` return `None` while the context is still dynamic

#### Scenario: A retraction from the store drops them too
- **WHEN** `set_data_index(2, i20, true)` is set and the context applies `ContextEvent::NodeRetracted(20)`
- **THEN** `get_data_index(&2, true)` returns `None`

### Requirement: Neighbour listings work without freezing
`ContextuableGraph::outbound_edges` and `ContextuableGraph::inbound_edges` SHALL list a node's neighbours in the dynamic and in the frozen context, and SHALL report a `ContextIndexError` naming the index when no node is held there.

#### Scenario: Listings agree across states
- **WHEN** a context with edges between its contextoids lists every node's neighbours before and after freezing
- **THEN** each node's dynamic listing holds the same indices as its frozen listing

#### Scenario: Listing at an absent index is reported
- **WHEN** `outbound_edges(99)` is called on a context with no node at index 99
- **THEN** it returns a `ContextIndexError` naming index 99

### Requirement: The context holds one relation per pair through the store
The context's base graph SHALL key its edges by `RelationKind`, and the context SHALL keep the store contract of one relation per pair: `apply` adds an edge only between a pair that holds none, `snapshot` refuses a graph holding two edges between one pair, and `remove_edge(a, b)` removes every edge from `a` to `b`.

#### Scenario: A repeated relation of one kind is one edge
- **WHEN** `add_edge(a, b, RelationKind::Temporal)` is called twice
- **THEN** `number_of_edges()` is 1 and `snapshot()` succeeds

#### Scenario: Two kinds between one pair are refused by snapshot
- **WHEN** `add_edge(a, b, RelationKind::Datial)` and `add_edge(a, b, RelationKind::Spatial)` are called
- **THEN** `number_of_edges()` is 2 and `snapshot()` returns `ProjectionError::Identity(id_a, "an edge is carried twice")`

#### Scenario: Removing a pair removes both kinds
- **WHEN** the pair of the previous scenario is removed with `remove_edge(a, b)`
- **THEN** `contains_edge(a, b)` is false and `number_of_edges()` is 0

### Requirement: A pair's relation reads the same in every state and context
`ContextuableGraph::get_edge(a, b)` and `extra_ctx_get_edge(a, b)` SHALL return the lowest `RelationKind` among the edges from `a` to `b`, in the dynamic and in the frozen context, and the extra contexts' `extra_ctx_add_edge` and `extra_ctx_remove_edge` SHALL key and remove edges as the base graph does.

The frozen adjacency is sorted by key and the dynamic one is in insertion order, so the first matching edge would differ between the states.

#### Scenario: The relation read does not depend on the state
- **WHEN** `add_edge(a, b, RelationKind::Spatial)` and then `add_edge(a, b, RelationKind::Datial)` are called, and `get_edge(a, b)` is read before and after freezing
- **THEN** both reads return `RelationKind::Datial`

#### Scenario: An extra context keys its edges
- **WHEN** `extra_ctx_add_edge(a, b, RelationKind::Temporal)` is called twice, then `extra_ctx_add_edge(a, b, RelationKind::Spatial)`, then `extra_ctx_remove_edge(a, b)`
- **THEN** `extra_ctx_edge_count()` reads 1 after the first two calls and 2 after the third, and `extra_ctx_contains_edge(a, b)` is false after the removal
