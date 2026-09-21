/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::FloatType;
use deep_causality_context::{Datable, Identifiable, UncertainAdjustable, UncertainData};
use deep_causality_uncertain::Uncertain;

#[test]
fn test_new() {
    let id = 1;
    let data = Uncertain::<f64>::point(42.0);
    let ufd = UncertainData::<FloatType>::new(id, data.clone());
    assert_eq!(ufd.id(), id);
    assert!((ufd.get_data().sample_from_entropy().unwrap() - 42.0).abs() < f64::EPSILON);
}

#[test]
fn test_id() {
    let id = 42;
    let data = Uncertain::<f64>::point(1.0);
    let ufd = UncertainData::<FloatType>::new(id, data);
    assert_eq!(ufd.id(), id);
}

#[test]
fn test_get_data() {
    let id = 1;
    let data = Uncertain::<f64>::point(std::f64::consts::PI);
    let ufd = UncertainData::<FloatType>::new(id, data.clone());
    assert!(
        (ufd.get_data().sample_from_entropy().unwrap() - std::f64::consts::PI).abs() < f64::EPSILON
    );
}

#[test]
fn test_set_data() {
    let id = 1;
    let initial_data = Uncertain::<f64>::point(1.0);
    let mut ufd = UncertainData::<FloatType>::new(id, initial_data);
    assert!((ufd.get_data().sample_from_entropy().unwrap() - 1.0).abs() < f64::EPSILON);

    let new_data = Uncertain::<f64>::point(2.0);
    ufd.set_data(new_data.clone());
    assert!((ufd.get_data().sample_from_entropy().unwrap() - 2.0).abs() < f64::EPSILON);
}

#[test]
fn test_display() {
    let id = 1;
    let data = Uncertain::normal(0.0, 1.0);
    let ufd = UncertainData::<FloatType>::new(id, data.clone());
    let display_str = format!("{}", ufd);
    // A generic `Display` must name the type rather than one instantiation of it — printing a
    // scalar-specific name for an `UncertainData<f32>` would be a lie.
    assert!(display_str.contains("UncertainData: id: 1"));
    assert!(display_str.contains("data:"));
}

#[test]
fn test_update() {
    let id = 1;
    let mut ufd = UncertainData::<FloatType>::new(id, Uncertain::<FloatType>::point(1.0));
    assert_eq!(ufd.get_data().sample_from_entropy().unwrap(), 1.0);

    // `UncertainAdjustable::update` has a default body that returns `Ok(())` and changes nothing,
    // so a refusal check alone would pass against an impl that never ran. The held value is what
    // says the update happened.
    let update_data = Uncertain::<FloatType>::point(2.0);
    let res = ufd.update(update_data.clone());
    assert!(res.is_ok());
    assert_eq!(ufd.get_data().sample_from_entropy().unwrap(), 2.0);
    assert_eq!(
        ufd.get_data(),
        update_data,
        "update replaces the held value"
    );
}

#[test]
fn test_update_replaces_the_distribution_rather_than_the_sample() {
    let id = 7;
    let mut ufd = UncertainData::<FloatType>::new(id, Uncertain::<FloatType>::point(0.0));

    // A distribution, not a point: replacing it has to carry the whole node across, so an impl
    // that collapsed the new value to a sample would not compare equal.
    let update_data = Uncertain::<FloatType>::uniform(-3.0, 5.0);
    let expected = update_data.clone();
    ufd.update(update_data).expect("update succeeds");

    assert_eq!(ufd.get_data(), expected);
    assert_eq!(ufd.id(), id, "adjusting the data leaves the identity alone");
}

#[test]
fn test_adjust() {
    let id = 1;
    let mut ufd = UncertainData::<FloatType>::new(id, Uncertain::<FloatType>::point(1.0));
    assert_eq!(ufd.get_data().sample_from_entropy().unwrap(), 1.0);

    let adjust_data = Uncertain::<FloatType>::point(-4.5);
    let res = ufd.adjust(adjust_data.clone());
    assert!(res.is_ok());
    assert_eq!(ufd.get_data().sample_from_entropy().unwrap(), -4.5);
    assert_eq!(
        ufd.get_data(),
        adjust_data,
        "adjust replaces the held value"
    );
}

#[test]
fn test_adjust_and_update_agree() {
    // Replacing a distribution has no partial form, so the two channels have to land on the same
    // state. A `update` wired to one field and `adjust` to another separates them here.
    let replacement = Uncertain::<FloatType>::normal(2.0, 0.5);

    let mut updated = UncertainData::<FloatType>::new(1, Uncertain::<FloatType>::point(0.0));
    updated
        .update(replacement.clone())
        .expect("update succeeds");

    let mut adjusted = UncertainData::<FloatType>::new(1, Uncertain::<FloatType>::point(0.0));
    adjusted
        .adjust(replacement.clone())
        .expect("adjust succeeds");

    assert_eq!(updated.get_data(), adjusted.get_data());
    assert_eq!(updated.get_data(), replacement);
}

#[test]
fn test_repeated_update_keeps_only_the_last_distribution() {
    let mut ufd = UncertainData::<FloatType>::new(3, Uncertain::<FloatType>::point(1.0));

    for value in [2.0, 3.0, 4.0] {
        ufd.update(Uncertain::<FloatType>::point(value))
            .expect("update succeeds");
        assert_eq!(ufd.get_data().sample_from_entropy().unwrap(), value);
    }

    // Supersession, not accumulation: the node holds one distribution at a time.
    assert_eq!(ufd.get_data(), Uncertain::<FloatType>::point(4.0));
}
