/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::EffectValue;

#[test]
fn test_none_variant() {
    let val: EffectValue<i32> = EffectValue::None;
    assert!(val.is_none());
    assert!(!val.is_value());
    assert!(!val.is_contextual_link());
    assert!(!val.is_relay_to());
}

#[test]
fn test_value_variant() {
    let val = EffectValue::Value(42);
    assert!(!val.is_none());
    assert!(val.is_value());
    assert!(!val.is_contextual_link());
    assert!(!val.is_relay_to());
}

#[test]
fn test_contextual_link_variant() {
    let val: EffectValue<i32> = EffectValue::ContextualLink(1, 2);
    assert!(!val.is_none());
    assert!(!val.is_value());
    assert!(val.is_contextual_link());
    assert!(!val.is_relay_to());
}

#[test]
fn test_relay_to_variant() {
    use deep_causality_core::PropagatingEffect;

    let effect = PropagatingEffect::pure(42);
    let val: EffectValue<i32> = EffectValue::RelayTo(0, Box::new(effect));

    assert!(!val.is_none());
    assert!(!val.is_value());
    assert!(!val.is_contextual_link());
    assert!(val.is_relay_to());
}

#[cfg(feature = "std")]
#[test]
fn test_map_variant() {
    use deep_causality_core::PropagatingEffect;
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(1, Box::new(PropagatingEffect::pure(42)));

    let val: EffectValue<i32> = EffectValue::Map(map);

    assert!(!val.is_none());
    assert!(!val.is_value());
    assert!(!val.is_contextual_link());
    assert!(!val.is_relay_to());
    assert!(val.is_map());
}

#[test]
fn test_as_value() {
    let val = EffectValue::Value(42);
    assert_eq!(val.as_value(), Some(&42));

    let none_val: EffectValue<i32> = EffectValue::None;
    assert_eq!(none_val.as_value(), None);
}

#[test]
fn test_into_value() {
    let val = EffectValue::Value(42);
    assert_eq!(val.into_value(), Some(42));

    let none_val: EffectValue<i32> = EffectValue::None;
    assert_eq!(none_val.into_value(), None);
}

#[test]
fn test_from_conversion() {
    let val: EffectValue<i32> = 42.into();
    assert!(val.is_value());
    assert_eq!(val.into_value(), Some(42));
}

#[test]
fn test_default() {
    let val: EffectValue<i32> = EffectValue::default();
    assert!(val.is_none());
}

#[test]
fn test_display_none() {
    let val: EffectValue<i32> = EffectValue::None;
    let display = format!("{}", val);
    assert_eq!(display, "None");
}

#[test]
fn test_display_value() {
    let val = EffectValue::Value(42);
    let display = format!("{}", val);
    assert_eq!(display, "Value(42)");
}

#[test]
fn test_display_contextual_link() {
    let val: EffectValue<i32> = EffectValue::ContextualLink(1, 2);
    let display = format!("{}", val);
    assert_eq!(display, "ContextualLink(1, 2)");
}
