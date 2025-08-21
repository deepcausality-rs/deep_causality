/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    BaseContext, BaseSymbol, BaseTeloidStore, Data, EuclideanSpace, EuclideanSpacetime,
    EuclideanTime, FloatType, NumericalValue, ProposedAction, Teloid, TeloidID, TeloidModal,
    TeloidStorable, TeloidStore,
};

fn always_true_predicate(_context: &BaseContext, _action: &ProposedAction) -> bool {
    true
}

fn create_test_teloid(
    id: TeloidID,
    action_id: &str,
) -> Teloid<
    Data<NumericalValue>,
    EuclideanSpace,
    EuclideanTime,
    EuclideanSpacetime,
    BaseSymbol,
    FloatType,
    FloatType,
> {
    Teloid::new(
        id,
        action_id.to_string(),
        always_true_predicate,
        TeloidModal::Obligatory,
        id as u64 * 100,
        id as u32 * 10,
        id as u32 * 5,
        vec![],
        None,
    )
}

#[test]
fn test_teloid_store_new() {
    let store = TeloidStore::<
        Data<NumericalValue>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >::new();

    // There is an equivalent type alias "BaseTeloidStore" in DeepCausality
    // that reduces the type signature boilerplate for the basic use case.
    // let mut store = BaseTeloidStore::new();

    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}

#[test]
fn test_teloid_store_with_capacity() {
    let store = TeloidStore::<
        Data<NumericalValue>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >::with_capacity(10);
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
}

#[test]
fn test_teloid_store_insert_new() {
    // For all subsequent tests, we use the basic type alias
    let mut store = BaseTeloidStore::new();

    let teloid1 = create_test_teloid(1, "action1");

    let old_teloid = store.insert(teloid1.clone());
    assert!(old_teloid.is_none());
    assert_eq!(store.len(), 1);
    assert!(!store.is_empty());
    assert!(store.contains_key(&1));
    assert_eq!(store.get(&1).unwrap().id(), 1);
}

#[test]
fn test_teloid_store_insert_update() {
    let mut store = BaseTeloidStore::new();

    let teloid1 = create_test_teloid(1, "action1");
    store.insert(teloid1.clone());

    let teloid1_updated = create_test_teloid(1, "action1_updated");
    let old_teloid = store.insert(teloid1_updated.clone());

    assert!(old_teloid.is_some());
    assert_eq!(old_teloid.unwrap().action_identifier(), "action1");
    assert_eq!(store.len(), 1);
    assert_eq!(
        store.get(&1).unwrap().action_identifier(),
        "action1_updated"
    );
}

#[test]
fn test_teloid_store_get() {
    let mut store = BaseTeloidStore::new();

    let teloid1 = create_test_teloid(1, "action1");
    store.insert(teloid1.clone());

    let retrieved_teloid = store.get(&1);
    assert!(retrieved_teloid.is_some());
    assert_eq!(retrieved_teloid.unwrap().id(), 1);
    assert_eq!(retrieved_teloid.unwrap().action_identifier(), "action1");

    let non_existent_teloid = store.get(&99);
    assert!(non_existent_teloid.is_none());
}

#[test]
fn test_teloid_store_remove() {
    let mut store = BaseTeloidStore::new();

    let teloid1 = create_test_teloid(1, "action1");
    store.insert(teloid1.clone());

    let removed_teloid = store.remove(&1);
    assert!(removed_teloid.is_some());
    assert_eq!(removed_teloid.unwrap().id(), 1);
    assert_eq!(store.len(), 0);
    assert!(store.is_empty());
    assert!(!store.contains_key(&1));

    let non_existent_removed = store.remove(&99);
    assert!(non_existent_removed.is_none());
}

#[test]
fn test_teloid_store_update() {
    let mut store = BaseTeloidStore::new();

    let teloid1 = create_test_teloid(1, "action1");
    store.insert(teloid1.clone());

    let teloid1_updated = create_test_teloid(1, "action1_updated");
    let old_teloid = store.update(teloid1_updated.clone());

    assert!(old_teloid.is_some());
    assert_eq!(old_teloid.unwrap().action_identifier(), "action1");
    assert_eq!(store.len(), 1);
    assert_eq!(
        store.get(&1).unwrap().action_identifier(),
        "action1_updated"
    );
}

#[test]
fn test_teloid_store_contains_key() {
    let mut store = BaseTeloidStore::new();

    let teloid1 = create_test_teloid(1, "action1");
    store.insert(teloid1.clone());

    assert!(store.contains_key(&1));
    assert!(!store.contains_key(&99));
}

#[test]
fn test_teloid_store_len() {
    let mut store = BaseTeloidStore::new();

    assert_eq!(store.len(), 0);

    store.insert(create_test_teloid(1, "action1"));
    assert_eq!(store.len(), 1);

    store.insert(create_test_teloid(2, "action2"));
    assert_eq!(store.len(), 2);

    store.remove(&1);
    assert_eq!(store.len(), 1);
}

#[test]
fn test_teloid_store_is_empty() {
    let mut store = BaseTeloidStore::new();
    assert!(store.is_empty());

    store.insert(create_test_teloid(1, "action1"));
    assert!(!store.is_empty());

    store.remove(&1);
    assert!(store.is_empty());
}

#[test]
fn test_teloid_store_clear() {
    let mut store = BaseTeloidStore::new();

    store.insert(create_test_teloid(1, "action1"));
    store.insert(create_test_teloid(2, "action2"));
    assert_eq!(store.len(), 2);

    store.clear();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
    assert!(!store.contains_key(&1));
}
