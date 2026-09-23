/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context_store::SubstrateRef;
use std::collections::HashSet;

#[test]
fn test_new_and_getters() {
    let r = SubstrateRef::new("series".to_string(), "sensor-7".to_string());
    assert_eq!(r.source(), "series");
    assert_eq!(r.key(), "sensor-7");
}

#[test]
fn test_display_joins_source_and_key() {
    let r = SubstrateRef::new("series".to_string(), "sensor-7".to_string());
    assert_eq!(r.to_string(), "series/sensor-7");
}

#[test]
fn test_default_is_empty() {
    let r = SubstrateRef::default();
    assert_eq!(r.source(), "");
    assert_eq!(r.key(), "");
    assert_eq!(r.to_string(), "/");
}

#[test]
fn test_equality_is_by_both_fields() {
    let a = SubstrateRef::new("series".to_string(), "k".to_string());
    let b = SubstrateRef::new("series".to_string(), "k".to_string());
    let other_source = SubstrateRef::new("file".to_string(), "k".to_string());
    let other_key = SubstrateRef::new("series".to_string(), "j".to_string());
    assert_eq!(a, b);
    assert_ne!(a, other_source);
    assert_ne!(a, other_key);
}

#[test]
fn test_clone_and_hash() {
    let a = SubstrateRef::new("series".to_string(), "k".to_string());
    let b = a.clone();
    let mut set = HashSet::new();
    set.insert(a);
    assert!(set.contains(&b));
    assert_eq!(set.len(), 1);
}

#[test]
fn test_debug() {
    let r = SubstrateRef::new("s".to_string(), "k".to_string());
    let dbg = format!("{r:?}");
    assert!(dbg.contains("SubstrateRef"));
    assert!(dbg.contains("\"s\""));
    assert!(dbg.contains("\"k\""));
}
