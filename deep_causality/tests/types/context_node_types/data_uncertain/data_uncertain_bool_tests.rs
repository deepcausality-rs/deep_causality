/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    Datable, Identifiable, UncertainAdjustable, UncertainBool, UncertainBooleanData,
};

#[test]
fn test_new() {
    let id = 1;
    let data = UncertainBool::point(true);
    let ubd = UncertainBooleanData::new(id, data.clone());
    assert_eq!(ubd.id(), id);
    assert!(ubd.get_data().sample_from_entropy().unwrap());
}

#[test]
fn test_id() {
    let id = 42;
    let data = UncertainBool::point(true);
    let ubd = UncertainBooleanData::new(id, data);
    assert_eq!(ubd.id(), id);
}

#[test]
fn test_get_data() {
    let id = 1;
    let data = UncertainBool::point(true);
    let ubd = UncertainBooleanData::new(id, data.clone());
    assert!(ubd.get_data().sample_from_entropy().unwrap());
}

#[test]
fn test_set_data() {
    let id = 1;
    let initial_data = UncertainBool::point(true);
    let mut ubd = UncertainBooleanData::new(id, initial_data);
    assert!(ubd.get_data().sample_from_entropy().unwrap());

    let new_data = UncertainBool::point(false);
    ubd.set_data(new_data.clone());
    assert!(!ubd.get_data().sample_from_entropy().unwrap());
}

#[test]
fn test_display() {
    let id = 1;
    let data = UncertainBool::bernoulli(0.75);
    let ubd = UncertainBooleanData::new(id, data.clone());
    let display_str = format!("{}", ubd);
    // The debug format of Uncertain is not stable for testing, so we check for key components.
    // As with the real node: the struct is `UncertainBoolData<R>`, and the `Display` names it.
    assert!(display_str.contains("UncertainBoolData: id: 1"));
    assert!(display_str.contains("data:"));
}

#[test]
fn test_update() {
    let id = 1;
    let initial_data = UncertainBool::point(true);
    let mut ubd = UncertainBooleanData::new(id, initial_data);
    assert!(ubd.get_data().sample_from_entropy().unwrap());

    let update_data = UncertainBool::point(false);
    let res = ubd.update(update_data.clone());
    assert!(res.is_ok());
    assert!(!ubd.get_data().sample_from_entropy().unwrap());
}

#[test]
fn test_adjust() {
    let id = 1;
    let initial_data = UncertainBool::point(true);
    let mut ubd = UncertainBooleanData::new(id, initial_data);
    assert!(ubd.get_data().sample_from_entropy().unwrap());

    // Identity used to be checked through `Uncertain::id()`, a process-wide counter that existed
    // to key the sample cache. With the cache gone the value's own equality says the same thing
    // and says it structurally.
    let adjust_data = UncertainBool::bernoulli(0.9);
    let expected = adjust_data.clone();
    let res = ubd.adjust(adjust_data);
    assert!(res.is_ok());
    assert_eq!(ubd.get_data(), expected, "adjust replaces the held value");
}
