/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::FloatType;
use crate::constants::{CAP, COMMS_BAND_RAD_S};
use deep_causality_cfd::{BlackoutTrigger, CfdScalar, CoupledField};
use deep_causality_num::{FromPrimitive, Real};
use deep_causality_tensor::Truncation;
use std::process::exit;

pub(crate) fn stop(e: &deep_causality_cfd::PhysicsError) -> ! {
    eprintln!("corridor setup failed: {e}");
    exit(2)
}

/// Lift an exact `f64` specification constant into the working precision. The constants in
/// `constants.rs` are written as `f64` literals, which any precision at least as wide (`f64`,
/// `Float106`) represents exactly; every *derived* number is computed in [`FloatType`] after
/// this one lossless lift, so switching the alias changes the arithmetic precision of the whole
/// corridor.
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}

/// A commanded angle in degrees, converted to radians in the working precision (the conversion
/// uses [`FloatType`]'s own value of pi, not the `f64` one).
pub fn rad(deg: f64) -> FloatType {
    ft(deg) * FloatType::pi() / ft(180.0)
}

pub(crate) fn trunc() -> Truncation<FloatType> {
    Truncation::<FloatType>::by_bond(CAP).expect("bond cap is valid")
}

/// The blackout trigger at the GPS L1 band.
pub fn trigger() -> BlackoutTrigger<FloatType> {
    BlackoutTrigger::new(ft(COMMS_BAND_RAD_S))
}

pub(crate) fn peak<R: CfdScalar>(xs: &[R]) -> R {
    xs.iter()
        .copied()
        .fold(R::zero(), |a, x| if x > a { x } else { a })
}

pub(crate) fn norm3(v: [FloatType; 3]) -> FloatType {
    Real::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
}

pub(crate) fn scalar0(field: &CoupledField<FloatType>, name: &str) -> FloatType {
    field
        .scalar(name)
        .and_then(|s| s.first().copied())
        .unwrap_or_else(|| ft(0.0))
}
