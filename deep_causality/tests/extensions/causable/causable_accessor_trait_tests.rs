/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Trait-qualified `CausableCollectionAccessor` coverage for the slice / `VecDeque` / `HashMap` /
//! `BTreeMap` impls.
//!
//! The per-collection tests call `.len()` / `.is_empty()` / `.to_vec()` on the concrete type, which
//! resolve to the *inherent* std methods rather than the trait impls in
//! `extensions/causable/mod.rs`. These tests invoke the methods through
//! `CausableCollectionAccessor` explicitly so the trait impls themselves are exercised.

use std::collections::{BTreeMap, HashMap, VecDeque};

use deep_causality::utils_test::test_utils::*;
use deep_causality::*;

type Item = BaseCausaloid<NumericalValue, bool>;

fn items() -> Vec<Item> {
    vec![
        get_test_causaloid_deterministic(1),
        get_test_causaloid_deterministic(2),
        get_test_causaloid_deterministic(3),
    ]
}

#[test]
fn slice_accessor_trait_methods() {
    let v = items();
    let slice: &[Item] = &v;

    assert_eq!(
        CausableCollectionAccessor::<NumericalValue, bool, _>::len(slice),
        3
    );
    assert!(!CausableCollectionAccessor::<NumericalValue, bool, _>::is_empty(slice));
    assert_eq!(
        CausableCollectionAccessor::<NumericalValue, bool, _>::to_vec(slice).len(),
        3
    );
    assert_eq!(
        CausableCollectionAccessor::<NumericalValue, bool, _>::get_all_items(slice).len(),
        3
    );
    assert!(
        CausableCollectionAccessor::<NumericalValue, bool, _>::get_item_by_id(slice, 2).is_some()
    );

    let empty: &[Item] = &[];
    assert!(CausableCollectionAccessor::<NumericalValue, bool, _>::is_empty(empty));
}

#[test]
fn vec_deque_accessor_trait_methods() {
    let dq: VecDeque<Item> = VecDeque::from_iter(items());

    assert_eq!(
        CausableCollectionAccessor::<NumericalValue, bool, _>::len(&dq),
        3
    );
    assert!(!CausableCollectionAccessor::<NumericalValue, bool, _>::is_empty(&dq));
    assert_eq!(
        CausableCollectionAccessor::<NumericalValue, bool, _>::to_vec(&dq).len(),
        3
    );
}

#[test]
fn hash_map_accessor_trait_methods() {
    let map: HashMap<i8, Item> =
        HashMap::from_iter(items().into_iter().enumerate().map(|(i, c)| (i as i8, c)));

    assert_eq!(
        CausableCollectionAccessor::<NumericalValue, bool, _>::len(&map),
        3
    );
    assert!(!CausableCollectionAccessor::<NumericalValue, bool, _>::is_empty(&map));
    assert_eq!(
        CausableCollectionAccessor::<NumericalValue, bool, _>::to_vec(&map).len(),
        3
    );
}

#[test]
fn btree_map_accessor_trait_methods() {
    let map: BTreeMap<i8, Item> =
        BTreeMap::from_iter(items().into_iter().enumerate().map(|(i, c)| (i as i8, c)));

    assert_eq!(
        CausableCollectionAccessor::<NumericalValue, bool, _>::len(&map),
        3
    );
    assert!(!CausableCollectionAccessor::<NumericalValue, bool, _>::is_empty(&map));
    assert_eq!(
        CausableCollectionAccessor::<NumericalValue, bool, _>::to_vec(&map).len(),
        3
    );
}
