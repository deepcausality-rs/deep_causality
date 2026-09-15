/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! What one evaluation of a node produces.

use deep_causality_num::BFloat16;
use deep_causality_uncertain::Sample;

#[test]
fn display_renders_the_underlying_value_for_both_kinds() {
    assert_eq!(format!("{}", Sample::Real(1.5f64)), "1.5");
    assert_eq!(format!("{}", Sample::Real(-0.25f64)), "-0.25");
    assert_eq!(format!("{}", Sample::Bool::<f64>(true)), "true");
    assert_eq!(format!("{}", Sample::Bool::<f64>(false)), "false");
}

/// The display is the scalar's own, so a narrow scalar renders as itself rather than widened.
#[test]
fn display_is_the_scalars_own() {
    let at_bf16 = Sample::Real(BFloat16::from(2.5));
    assert_eq!(format!("{}", at_bf16), format!("{}", BFloat16::from(2.5)));
}

#[test]
fn debug_distinguishes_the_two_kinds() {
    assert!(format!("{:?}", Sample::Real(1.0f64)).starts_with("Real"));
    assert!(format!("{:?}", Sample::Bool::<f64>(true)).starts_with("Bool"));
}

#[test]
fn equality_is_on_the_kind_as_well_as_the_value() {
    assert_eq!(Sample::Real(1.0f64), Sample::Real(1.0));
    assert_ne!(Sample::Real(1.0f64), Sample::Real(2.0));
    assert_ne!(Sample::Bool::<f64>(true), Sample::Bool(false));

    // A real and a Boolean are never equal, whatever they hold.
    let real: Sample<f64> = Sample::Real(1.0);
    let boolean: Sample<f64> = Sample::Bool(true);
    assert_ne!(real, boolean);
}

/// A non-finite real is carried rather than rejected — the sample is what the draw produced, and
/// judging it is the caller's business.
#[test]
fn a_non_finite_real_is_carried() {
    assert_eq!(format!("{}", Sample::Real(f64::INFINITY)), "inf");
    assert_eq!(format!("{}", Sample::Real(f64::NEG_INFINITY)), "-inf");
    assert_eq!(format!("{}", Sample::Real(f64::NAN)), "NaN");

    // And NaN is equal to nothing, including another NaN sample.
    assert_ne!(Sample::Real(f64::NAN), Sample::Real(f64::NAN));
}

/// Negative zero keeps its sign through the sample, which is what a sum needs in order to
/// preserve it.
#[test]
fn negative_zero_keeps_its_sign() {
    match Sample::Real(-0.0f64) {
        Sample::Real(v) => assert!(v.is_sign_negative(), "the sample must not normalise -0.0"),
        Sample::Bool(_) => panic!("built a real"),
    }
}

#[test]
fn a_sample_is_copy_and_a_copy_compares_equal() {
    let s = Sample::Real(3.0f64);
    let copied = s;
    assert_eq!(s, copied);
}
