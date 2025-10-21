/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::PropagatingEffect;
use std::collections::HashMap;

#[test]
fn test_map_variant() {
    let mut map1 = HashMap::new();
    map1.insert(1, Box::new(PropagatingEffect::Numerical(1.0)));
    map1.insert(2, Box::new(PropagatingEffect::Deterministic(true)));
    let effect1 = PropagatingEffect::Map(map1.clone());

    let mut map2 = HashMap::new();
    map2.insert(1, Box::new(PropagatingEffect::Numerical(1.0)));
    map2.insert(2, Box::new(PropagatingEffect::Deterministic(true)));
    let effect2 = PropagatingEffect::Map(map2.clone());

    let mut map3 = HashMap::new();
    map3.insert(1, Box::new(PropagatingEffect::Numerical(1.0)));
    map3.insert(3, Box::new(PropagatingEffect::Deterministic(false))); // Different key and value
    let effect3 = PropagatingEffect::Map(map3.clone());

    assert!(effect1.is_map());
    assert_eq!(effect1, effect2);
    assert_ne!(effect1, effect3);
    assert_eq!(
        format!("{effect1:?}"),
        format!("PropagatingEffect::Map({:?})", map1)
    );

    // Test map specific methods
    let mut new_map = PropagatingEffect::new_map();
    new_map.insert(10, PropagatingEffect::Numerical(100.0));
    assert_eq!(new_map.get_numerical_from_map(10).unwrap(), 100.0);
    assert!(new_map.get_numerical_from_map(99).is_err());

    new_map.insert(11, PropagatingEffect::Deterministic(true));
    assert!(new_map.get_deterministic_from_map(11).unwrap());
    assert!(new_map.get_deterministic_from_map(99).is_err());
}

#[test]
#[should_panic(expected = "Cannot insert into PropagatingEffect that is not a Map variant")]
fn test_insert_panic() {
    let mut effect = PropagatingEffect::None;
    effect.insert(1, PropagatingEffect::Numerical(1.0));
}

#[test]
fn test_get_numerical_from_map_error_not_map() {
    let effect = PropagatingEffect::None;
    let result = effect.get_numerical_from_map(1);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "CausalityError: Cannot get value by key from PropagatingEffect that is not a Map variant"
    );
}

#[test]
fn test_get_numerical_from_map_error_wrong_type() {
    let mut map = PropagatingEffect::new_map();
    map.insert(1, PropagatingEffect::Deterministic(true));
    let result = map.get_numerical_from_map(1);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "CausalityError: Effect for key '1' is not of type Numerical"
    );
}

#[test]
fn test_get_deterministic_from_map_error_not_map() {
    let effect = PropagatingEffect::None;
    let result = effect.get_deterministic_from_map(1);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "CausalityError: Cannot get value by key from PropagatingEffect that is not a Map variant"
    );
}

#[test]
fn test_get_deterministic_from_map_error_wrong_type() {
    let mut map = PropagatingEffect::new_map();
    map.insert(1, PropagatingEffect::Numerical(1.0));
    let result = map.get_deterministic_from_map(1);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "CausalityError: Effect for key '1' is not of type Deterministic"
    );
}
