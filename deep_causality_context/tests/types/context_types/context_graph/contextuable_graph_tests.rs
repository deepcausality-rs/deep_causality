/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    BaseContext, BaseContextoid, Context, Contextoid, ContextoidType, ContextuableGraph, Data,
    DataIndexable, Datable, RelationKind, Root, TimeIndexable,
};
use deep_causality_core::Identifiable;

fn get_context() -> BaseContext {
    let id = 1;
    let name = "base context";
    Context::with_capacity(id, name, 10)
}

#[test]
fn test_update_node_err() {
    let mut context = get_context();
    let id = 1;
    let contextoid = Contextoid::new(id, ContextoidType::Root(Root::new(id)));
    let res = context.update_node(id, contextoid);
    assert!(res.is_err());
}

#[test]
fn test_add_edge_err() {
    let mut context = get_context();
    let res = context.add_edge(1, 2, RelationKind::Datial);
    assert!(res.is_err());
}

#[test]
fn test_remove_edge_err() {
    let mut context = get_context();
    let res = context.remove_edge(1, 2);
    assert!(res.is_err());
}

#[test]
fn test_remove_node_err() {
    let mut context = get_context();
    let id = 999;
    let res = context.remove_node(id);
    assert!(res.is_err());
}

#[test]
fn test_update_node_changes_id_mapping() {
    // Add a node, then update it with a node carrying a *different* ID. This
    // exercises the `new_node_id != node_id` branch that rewrites the
    // id-to-index map.
    let mut context = get_context();
    let old_id = 1;
    let new_id = 2;

    let idx = context
        .add_node(Contextoid::new(
            old_id,
            ContextoidType::Root(Root::new(old_id)),
        ))
        .expect("add node");

    let res = context.update_node(
        old_id,
        Contextoid::new(new_id, ContextoidType::Root(Root::new(new_id))),
    );
    assert!(res.is_ok());

    // The node now lives under the new id; updating the old id must fail.
    let stale = context.update_node(
        old_id,
        Contextoid::new(old_id, ContextoidType::Root(Root::new(old_id))),
    );
    assert!(stale.is_err());

    // Updating via the new id still resolves to the same index.
    let ok = context.update_node(
        new_id,
        Contextoid::new(new_id, ContextoidType::Root(Root::new(new_id))),
    );
    assert!(ok.is_ok());
    assert!(context.contains_node(idx));
}

#[test]
fn test_add_edge_err_second_index_missing() {
    // First index present, second index missing: exercises the `index b`
    // not-found guard specifically.
    let mut context = get_context();
    let a = context
        .add_node(Contextoid::new(1, ContextoidType::Root(Root::new(1))))
        .expect("add node a");

    let res = context.add_edge(a, 999, RelationKind::Datial);
    assert!(res.is_err());
    assert!(format!("{:?}", res.unwrap_err()).contains("index b"));
}

#[test]
fn test_remove_edge_err_second_index_missing() {
    // First index present, second index missing: exercises the `index b`
    // not-found guard in remove_edge.
    let mut context = get_context();
    let a = context
        .add_node(Contextoid::new(1, ContextoidType::Root(Root::new(1))))
        .expect("add node a");

    let res = context.remove_edge(a, 999);
    assert!(res.is_err());
    assert!(format!("{:?}", res.unwrap_err()).contains("index b"));
}

#[test]
fn test_edge_relation_survives_storage() {
    let mut context = get_context();

    let a = context
        .add_node(Contextoid::new(1, ContextoidType::Root(Root::new(1))))
        .expect("failed to add node a");
    let b = context
        .add_node(Contextoid::new(2, ContextoidType::Root(Root::new(2))))
        .expect("failed to add node b");
    let c = context
        .add_node(Contextoid::new(3, ContextoidType::Root(Root::new(3))))
        .expect("failed to add node c");

    // Two edges out of the same node, carrying different relations. Endpoints of the same node
    // type do not determine the relation, so only storage can tell these two apart.
    context
        .add_edge(a, b, RelationKind::SpaceTemporal)
        .expect("failed to add edge a -> b");
    context
        .add_edge(a, c, RelationKind::Temporal)
        .expect("failed to add edge a -> c");

    assert_eq!(context.get_edge(a, b), Some(&RelationKind::SpaceTemporal));
    assert_eq!(context.get_edge(a, c), Some(&RelationKind::Temporal));
    assert_eq!(context.get_edge(b, c), None);
}

#[test]
fn test_get_edge_returns_none_for_unknown_node() {
    let context = get_context();
    assert_eq!(context.get_edge(0, 1), None);
}

/// A data contextoid whose payload is its own id, so a node read back names itself.
fn datum(id: u64) -> BaseContextoid {
    Contextoid::new(id, ContextoidType::Datoid(Data::new(id, id as f64)))
}

/// The index of the live node carrying `id`.
fn index_of(context: &BaseContext, id: u64) -> usize {
    context.get_node_index_by_id(id).expect("a live id")
}

/// The id of the node at `index`.
fn id_at(context: &BaseContext, index: usize) -> u64 {
    context.get_node(index).expect("a node at the index").id()
}

#[test]
fn test_add_node_refuses_a_duplicate_id() {
    let mut context = get_context();
    let first = context.add_node(datum(1)).unwrap();
    let duplicate = Contextoid::new(1, ContextoidType::Datoid(Data::new(1, 99.0)));

    let err = context.add_node(duplicate).unwrap_err();
    assert!(
        err.to_string().contains("ID 1 is already in the context"),
        "{err}"
    );
    // The first node keeps the id and the context is unchanged.
    assert_eq!(context.number_of_nodes(), 1);
    assert_eq!(context.get_node_index_by_id(1), Some(first));
    let payload = context
        .get_node(first)
        .unwrap()
        .vertex_type()
        .dataoid()
        .unwrap()
        .get_data();
    assert_eq!(payload, 1.0);
}

#[test]
fn test_update_node_refuses_an_id_held_by_another_node() {
    let mut context = get_context();
    let one = context.add_node(datum(1)).unwrap();
    let two = context.add_node(datum(2)).unwrap();

    let err = context.update_node(1, datum(2)).unwrap_err();
    assert!(
        err.to_string().contains("ID 2 is already in the context"),
        "{err}"
    );
    // Both ids still name their own nodes.
    assert_eq!(context.get_node_index_by_id(1), Some(one));
    assert_eq!(context.get_node_index_by_id(2), Some(two));
    assert_eq!(id_at(&context, one), 1);
    assert_eq!(id_at(&context, two), 2);

    // Keeping the id is allowed.
    assert!(context.update_node(1, datum(1)).is_ok());
}

#[test]
fn test_freeze_and_unfreeze_switch_the_state() {
    let mut context = get_context();
    context.add_node(datum(1)).unwrap();
    assert!(!context.is_frozen());
    context.freeze();
    assert!(context.is_frozen());
    context.freeze(); // already frozen: nothing changes
    assert!(context.is_frozen());
    assert_eq!(context.number_of_nodes(), 1);
    context.unfreeze();
    assert!(!context.is_frozen());
    context.unfreeze(); // already mutable: nothing changes
    assert!(!context.is_frozen());
    assert_eq!(context.number_of_nodes(), 1);
}

#[test]
fn test_neighbour_listings_agree_with_the_edges() {
    // Six nodes with a cycle, a fan-out, a fan-in and an isolated node.
    let mut context = get_context();
    let index: Vec<usize> = (1..=6)
        .map(|id| context.add_node(datum(id)).unwrap())
        .collect();
    let edges = [(0, 1), (1, 2), (2, 0), (0, 3), (0, 4), (3, 4), (2, 4)];
    for (a, b) in edges {
        context
            .add_edge(index[a], index[b], RelationKind::Datial)
            .unwrap();
    }
    context.freeze();

    let mut listed = 0;
    for &a in &index {
        let outbound: Vec<usize> = context.outbound_edges(a).unwrap().collect();
        let inbound: Vec<usize> = context.inbound_edges(a).unwrap().collect();
        listed += outbound.len();
        for &b in &index {
            let edge = context.contains_edge(a, b);
            assert_eq!(outbound.contains(&b), edge, "outbound {a} -> {b}");
            assert_eq!(
                context.inbound_edges(b).unwrap().any(|x| x == a),
                edge,
                "inbound {a} -> {b}"
            );
        }
        assert_eq!(
            inbound.len(),
            index
                .iter()
                .filter(|&&s| context.contains_edge(s, a))
                .count()
        );
    }
    assert_eq!(listed, edges.len());
    assert_eq!(context.number_of_edges(), edges.len());
    // The isolated node has no neighbours either way.
    assert_eq!(context.outbound_edges(index[5]).unwrap().count(), 0);
    assert_eq!(context.inbound_edges(index[5]).unwrap().count(), 0);
}

#[test]
fn test_neighbour_listing_needs_a_frozen_context() {
    let mut context = get_context();
    let a = context.add_node(datum(1)).unwrap();
    for err in [
        context.outbound_edges(a).err(),
        context.inbound_edges(a).err(),
    ] {
        let err = err.expect("a mutable context lists no neighbours");
        assert!(err.to_string().contains("not frozen"), "{err}");
    }
    context.freeze();
    assert!(context.outbound_edges(a).is_ok());
    context.unfreeze();
    assert!(context.outbound_edges(a).is_err());
}

#[test]
fn test_neighbour_listing_refuses_a_missing_node() {
    let mut context = get_context();
    context.add_node(datum(1)).unwrap();
    context.freeze();
    for err in [
        context.outbound_edges(99).err(),
        context.inbound_edges(99).err(),
    ] {
        let err = err.expect("no node at index 99");
        assert!(err.to_string().contains("No node at index 99"), "{err}");
    }
}

#[test]
fn test_a_frozen_context_refuses_changes_and_keeps_its_reads() {
    let mut context = get_context();
    let a = context.add_node(datum(1)).unwrap();
    let b = context.add_node(datum(2)).unwrap();
    context.add_edge(a, b, RelationKind::Temporal).unwrap();
    let before = context.snapshot().unwrap();
    context.freeze();

    assert!(context.add_node(datum(3)).is_err());
    assert!(context.update_node(1, datum(1)).is_err());
    assert!(context.remove_node(1).is_err());
    assert!(context.add_edge(b, a, RelationKind::Datial).is_err());
    assert!(context.remove_edge(a, b).is_err());

    // Nothing changed, and every read still answers.
    assert_eq!(context.number_of_nodes(), 2);
    assert_eq!(context.number_of_edges(), 1);
    assert_eq!(context.get_node_index_by_id(2), Some(b));
    assert_eq!(id_at(&context, b), 2);
    assert!(context.contains_edge(a, b));
    assert_eq!(context.get_edge(a, b), Some(&RelationKind::Temporal));
    assert_eq!(context.snapshot().unwrap(), before);

    // Unfrozen, the same changes succeed.
    context.unfreeze();
    assert!(context.add_node(datum(3)).is_ok());
    assert!(context.add_edge(b, a, RelationKind::Datial).is_ok());
}

#[test]
fn test_freezing_after_a_removal_keeps_every_index_on_its_node() {
    // Freezing compacts away the removed node 20, so the nodes after it move down one index.
    let mut context = get_context();
    let [i10, i20, i30, i40] = [10, 20, 30, 40].map(|id| context.add_node(datum(id)).unwrap());
    context.add_edge(i10, i30, RelationKind::Datial).unwrap();
    context.add_edge(i30, i40, RelationKind::Datial).unwrap();
    context.set_data_index(1, i40, true);
    context.set_data_index(2, i20, false);
    context.set_time_index(3, i30, true);
    context.set_time_index(4, i20, false);
    context.remove_node(20).unwrap();

    context.freeze();

    let index = |id| index_of(&context, id);
    for id in [10, 30, 40] {
        assert_eq!(id_at(&context, index(id)), id);
    }
    assert_eq!(context.get_node_index_by_id(20), None);
    assert_ne!(index(40), i40, "the index after the removal moved down");
    // The edges follow their nodes.
    assert_eq!(
        context
            .outbound_edges(index(10))
            .unwrap()
            .collect::<Vec<_>>(),
        [index(30)]
    );
    assert_eq!(
        context
            .inbound_edges(index(40))
            .unwrap()
            .collect::<Vec<_>>(),
        [index(30)]
    );
    // Stored data and time indices follow their nodes; the ones naming the removed node go.
    assert_eq!(
        context
            .get_data_index(&1, true)
            .map(|&i| id_at(&context, i)),
        Some(40)
    );
    assert_eq!(
        context
            .get_time_index(&3, true)
            .map(|&i| id_at(&context, i)),
        Some(30)
    );
    assert_eq!(context.get_data_index(&2, false), None);
    assert_eq!(context.get_time_index(&4, false), None);

    // Unfreezing keeps every index.
    let frozen = [index(10), index(30), index(40)];
    context.unfreeze();
    assert_eq!([10, 30, 40].map(|id| index_of(&context, id)), frozen);
    assert!(context.add_node(datum(50)).is_ok());
}

#[test]
fn test_a_clone_of_a_frozen_context_is_frozen() {
    let mut context = get_context();
    let a = context.add_node(datum(1)).unwrap();
    context.freeze();
    let copy = context.clone();
    assert!(copy.is_frozen());
    assert!(copy.outbound_edges(a).is_ok());
}
