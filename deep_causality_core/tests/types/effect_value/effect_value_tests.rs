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

#[test]
fn test_display_relay_to() {
    use deep_causality_core::PropagatingEffect;

    let effect = PropagatingEffect::pure(42);
    let val: EffectValue<i32> = EffectValue::RelayTo(7, Box::new(effect));
    let display = format!("{}", val);
    assert_eq!(display, "RelayTo(7)");
}

#[cfg(feature = "std")]
#[test]
fn test_display_map() {
    use deep_causality_core::PropagatingEffect;
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(1, Box::new(PropagatingEffect::pure(42)));

    let val: EffectValue<i32> = EffectValue::Map(map);
    let display = format!("{}", val);
    assert_eq!(display, "Map(...)");
}

#[test]
fn test_eq_none() {
    let a: EffectValue<i32> = EffectValue::None;
    let b: EffectValue<i32> = EffectValue::None;
    assert_eq!(a, b);
}

#[test]
fn test_eq_value() {
    let a = EffectValue::Value(42);
    let b = EffectValue::Value(42);
    let c = EffectValue::Value(7);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_eq_contextual_link() {
    let a: EffectValue<i32> = EffectValue::ContextualLink(1, 2);
    let b: EffectValue<i32> = EffectValue::ContextualLink(1, 2);
    let c: EffectValue<i32> = EffectValue::ContextualLink(1, 3);
    let d: EffectValue<i32> = EffectValue::ContextualLink(9, 2);
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_ne!(a, d);
}

#[test]
fn test_eq_relay_to_compares_target_only() {
    use deep_causality_core::PropagatingEffect;

    // Same target, different boxed effects: still equal (target-only comparison).
    let a: EffectValue<i32> = EffectValue::RelayTo(0, Box::new(PropagatingEffect::pure(1)));
    let b: EffectValue<i32> = EffectValue::RelayTo(0, Box::new(PropagatingEffect::pure(99)));
    assert_eq!(a, b);

    // Different target: not equal.
    let c: EffectValue<i32> = EffectValue::RelayTo(1, Box::new(PropagatingEffect::pure(1)));
    assert_ne!(a, c);
}

#[cfg(feature = "std")]
#[test]
fn test_eq_map_is_never_equal() {
    use deep_causality_core::PropagatingEffect;
    use std::collections::HashMap;

    let mut map_a = HashMap::new();
    map_a.insert(1, Box::new(PropagatingEffect::pure(42)));
    let a: EffectValue<i32> = EffectValue::Map(map_a);

    let mut map_b = HashMap::new();
    map_b.insert(1, Box::new(PropagatingEffect::pure(42)));
    let b: EffectValue<i32> = EffectValue::Map(map_b);

    // Maps are documented as not comparable, so eq is always false.
    assert_ne!(a, b);
}

#[test]
fn test_eq_different_variants() {
    let none: EffectValue<i32> = EffectValue::None;
    let value = EffectValue::Value(42);
    let link: EffectValue<i32> = EffectValue::ContextualLink(1, 2);
    assert_ne!(none, value);
    assert_ne!(value, link);
    assert_ne!(none, link);
}
