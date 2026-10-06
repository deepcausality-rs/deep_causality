/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    Context, Coordinate, Data, DiscreteTime, FloatType, Identifiable, NoSpace, NoSpaceTime, Spatial,
};
use deep_causality_num::BFloat16;

#[test]
fn test_is_zero_sized_at_every_scalar() {
    assert_eq!(size_of::<NoSpace<f64>>(), 0);
    assert_eq!(size_of::<NoSpace<f32>>(), 0);
    assert_eq!(size_of::<NoSpace<BFloat16>>(), 0);
}

#[test]
fn test_new_and_default_agree() {
    let made: NoSpace<FloatType> = NoSpace::new();
    let defaulted: NoSpace<FloatType> = NoSpace::default();
    assert_eq!(made, defaulted);
}

#[test]
fn test_every_index_is_out_of_bounds() {
    let empty: NoSpace<FloatType> = NoSpace::new();
    assert_eq!(empty.dimension(), 0);
    for index in [0, 1, 2, usize::MAX] {
        let message = empty.coordinate(index).unwrap_err().to_string();
        assert!(message.contains("out of bounds"), "index {index}");
        assert!(message.contains("NoSpace"));
    }
}

#[test]
fn test_id_display_debug_and_copy() {
    let a: NoSpace<FloatType> = NoSpace::new();
    let b = a;
    assert_eq!(a, b);
    assert_eq!(a.id(), 0);
    assert_eq!(format!("{a}"), "NoSpace");
    assert!(format!("{a:?}").contains("NoSpace"));
}

#[test]
fn test_fills_the_spatial_slot_of_a_context() {
    fn assert_spatial<T: Spatial<Coord = FloatType>>() {}
    assert_spatial::<NoSpace<FloatType>>();

    type Clockwork = Context<Data<f64>, NoSpace<f64>, DiscreteTime, NoSpaceTime<f64>>;
    let context: Clockwork = Context::with_capacity(1, "clock only", 4);
    assert_eq!(context.name(), "clock only");
}
