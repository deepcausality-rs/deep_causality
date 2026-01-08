/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{Datable, Identifiable, UncertainAdjustable, UncertainBooleanData};
use deep_causality_uncertain::Uncertain;

#[test]
fn test_new() {
    let id = 1;
    let data = Uncertain::<bool>::point(true);
    let ubd = UncertainBooleanData::new(id, data.clone());
    assert_eq!(ubd.id(), id);
    assert!(ubd.get_data().sample().unwrap());
}

#[test]
fn test_id() {
    let id = 42;
    let data = Uncertain::<bool>::point(true);
    let ubd = UncertainBooleanData::new(id, data);
    assert_eq!(ubd.id(), id);
}

#[test]
fn test_get_data() {
    let id = 1;
    let data = Uncertain::<bool>::point(true);
    let ubd = UncertainBooleanData::new(id, data.clone());
    assert!(ubd.get_data().sample().unwrap());
}

#[test]
fn test_set_data() {
    let id = 1;
    let initial_data = Uncertain::<bool>::point(true);
    let mut ubd = UncertainBooleanData::new(id, initial_data);
    assert!(ubd.get_data().sample().unwrap());

    let new_data = Uncertain::<bool>::point(false);
    ubd.set_data(new_data.clone());
    assert!(!ubd.get_data().sample().unwrap());
}

#[test]
fn test_display() {
    let id = 1;
    let data = Uncertain::bernoulli(0.75);
    let ubd = UncertainBooleanData::new(id, data.clone());
    let display_str = format!("{}", ubd);
    // The debug format of Uncertain is not stable for testing, so we check for key components.
    assert!(display_str.contains("UncertainBooleanData: id: 1"));
    assert!(display_str.contains("data:"));
}

#[test]
fn test_update() {
    let id = 1;
    let initial_data = Uncertain::<bool>::point(true);
    let mut ubd = UncertainBooleanData::new(id, initial_data);
    assert!(ubd.get_data().sample().unwrap());

    let update_data = Uncertain::<bool>::point(false);
    let res = ubd.update(update_data.clone());
    assert!(res.is_ok());
    assert!(!ubd.get_data().sample().unwrap());
}

#[test]
fn test_adjust() {
    let id = 1;
    let initial_data = Uncertain::<bool>::point(true);
    let mut ubd = UncertainBooleanData::new(id, initial_data);
    assert!(ubd.get_data().sample().unwrap());

    let adjust_data = Uncertain::bernoulli(0.9);
    let adjust_data_id = adjust_data.id();
    let res = ubd.adjust(adjust_data);
    assert!(res.is_ok());
    assert_eq!(ubd.get_data().id(), adjust_data_id);
}
