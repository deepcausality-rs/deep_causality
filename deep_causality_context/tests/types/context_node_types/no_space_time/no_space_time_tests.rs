/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    Coordinate, FloatType, Identifiable, NoSpaceTime, SpaceTemporal, Spatial, Temporal, TimeScale,
};
use deep_causality_num::BFloat16;

// =============================================================================
// It is zero-sized, at every scalar
// =============================================================================

#[test]
fn test_is_zero_sized_at_every_scalar() {
    // The point of the type: naming the absence of a spatial part must cost nothing. If the
    // scalar were stored rather than held in PhantomData, these would be 8, 4 and 2.
    assert_eq!(size_of::<NoSpaceTime<f64>>(), 0);
    assert_eq!(size_of::<NoSpaceTime<f32>>(), 0);
    assert_eq!(size_of::<NoSpaceTime<BFloat16>>(), 0);
}

#[test]
fn test_new_and_default_agree() {
    let made: NoSpaceTime<FloatType> = NoSpaceTime::new();
    let defaulted: NoSpaceTime<FloatType> = NoSpaceTime::default();
    assert_eq!(made, defaulted);
}

// =============================================================================
// Coordinate: an empty coordinate system
// =============================================================================

#[test]
fn test_dimension_is_zero() {
    let empty: NoSpaceTime<FloatType> = NoSpaceTime::new();
    assert_eq!(empty.dimension(), 0);
}

#[test]
fn test_every_index_is_out_of_bounds() {
    let empty: NoSpaceTime<FloatType> = NoSpaceTime::new();

    // Index 0 is the one a caller reaches for first, and it must fail like any other: with no
    // axes there is no zeroth axis either.
    for index in [0, 1, 2, 3, usize::MAX] {
        let result = empty.coordinate(index);
        assert!(result.is_err(), "index {index} should be out of bounds");
        let message = result.unwrap_err().to_string();
        assert!(message.contains("out of bounds"));
    }
}

// =============================================================================
// Identifiable, Temporal, SpaceTemporal
// =============================================================================

#[test]
fn test_id_is_zero() {
    let empty: NoSpaceTime<FloatType> = NoSpaceTime::new();
    assert_eq!(empty.id(), 0);
}

#[test]
fn test_time_scale_is_no_scale() {
    let empty: NoSpaceTime<FloatType> = NoSpaceTime::new();
    assert_eq!(empty.time_scale(), TimeScale::NoScale);
}

#[test]
fn test_time_unit_is_the_unit_type() {
    let empty: NoSpaceTime<FloatType> = NoSpaceTime::new();
    assert_eq!(empty.time_unit(), ());
    assert_eq!(*empty.t(), ());
}

// =============================================================================
// The trait impls a frame needs
// =============================================================================

#[test]
fn test_implements_spatial_and_space_temporal_at_the_frame_scalar() {
    // A frame binds its Space and SpaceTime members to its scalar, so the impls have to resolve
    // with `Coord` equal to that scalar rather than to a placeholder.
    fn assert_spatial<T: Spatial<Coord = FloatType>>() {}
    fn assert_space_temporal<T: SpaceTemporal<Coord = FloatType>>() {}

    assert_spatial::<NoSpaceTime<FloatType>>();
    assert_space_temporal::<NoSpaceTime<FloatType>>();
}

#[test]
fn test_display_names_the_type() {
    let empty: NoSpaceTime<FloatType> = NoSpaceTime::new();
    assert_eq!(format!("{empty}"), "NoSpaceTime");
}

#[test]
fn test_debug_is_available() {
    let empty: NoSpaceTime<FloatType> = NoSpaceTime::new();
    assert!(format!("{empty:?}").contains("NoSpaceTime"));
}

#[test]
fn test_is_copy() {
    let a: NoSpaceTime<FloatType> = NoSpaceTime::new();
    let b = a;
    assert_eq!(a, b);
}
