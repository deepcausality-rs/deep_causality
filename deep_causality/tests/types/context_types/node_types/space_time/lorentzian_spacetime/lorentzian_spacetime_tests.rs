/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::prelude::*;

#[test]
fn test_identifiable_trait() {
    let s = LorentzianSpacetime::new(7, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    assert_eq!(s.id(), 7);
}

#[test]
fn test_coordinate_trait() {
    let s = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 0.0, TimeScale::Second);

    assert_eq!(s.dimension(), 4);
    assert_eq!(*s.coordinate(0).unwrap(), 1.0);
    assert_eq!(*s.coordinate(1).unwrap(), 2.0);
    assert_eq!(*s.coordinate(2).unwrap(), 3.0);
}

#[test]
fn test_coordinate_trait_out_of_bounds() {
    let s = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 0.0, TimeScale::Second);
    let res = s.coordinate(4);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let s = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 42.0, TimeScale::Millisecond);
    let formatted = format!("{s}");
    dbg!(&formatted);
    assert!(formatted.contains("LorentzianSpacetime"));
    assert!(formatted.contains("x=1.0"));
    assert!(formatted.contains("t=42.0"));
}

#[test]
fn test_temporal_trait() {
    let s = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 123456.0, TimeScale::Second);

    assert_eq!(s.time_scale(), TimeScale::Second);
    assert_eq!(s.time_unit(), 123456.0);
}

#[test]
fn test_space_temporal_trait() {
    let s = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 999999.0, TimeScale::Second);

    // dbg!(*s.t());
    assert_eq!(s.t(), &999999.0);
    // dbg!(*s.coordinate(0));
    assert_eq!(*s.coordinate(0).unwrap(), 1.0);
}
