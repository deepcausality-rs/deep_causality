/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    Context, Data, EuclideanSpace, Identifiable, NoSpaceTime, NoTime, Temporal, TimeScale,
};

#[test]
fn test_is_zero_sized() {
    assert_eq!(size_of::<NoTime>(), 0);
}

#[test]
fn test_new_and_default_agree() {
    // Through a generic bound, so the test exercises the `Default` impl the context slot uses.
    fn made<T: Default>() -> T {
        T::default()
    }
    assert_eq!(NoTime::new(), made::<NoTime>());
}

#[test]
fn test_reports_no_instant() {
    let none = NoTime::new();
    assert_eq!(none.time_scale(), TimeScale::NoScale);
    assert_eq!(none.time_unit(), ());
}

#[test]
fn test_id_display_debug_and_copy() {
    let a = NoTime::new();
    let b = a;
    assert_eq!(a, b);
    assert_eq!(a.id(), 0);
    assert_eq!(format!("{a}"), "NoTime");
    assert!(format!("{a:?}").contains("NoTime"));
}

#[test]
fn test_fills_the_temporal_slot_of_a_context() {
    type Layout = Context<Data<f64>, EuclideanSpace<f64>, NoTime, NoSpaceTime<f64>>;
    let context: Layout = Context::with_capacity(1, "positions only", 4);
    assert_eq!(context.name(), "positions only");
}
