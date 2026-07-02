/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::FloatType;
use crate::constants::{CAP, COMMS_BAND_RAD_S, L};
use deep_causality_cfd::{BlackoutTrigger, CfdScalar, CoupledField};
use deep_causality_num::{FromPrimitive, Zero};
use deep_causality_tensor::Truncation;
use std::process::exit;

pub(crate) fn stop(e: &deep_causality_cfd::PhysicsError) -> ! {
    eprintln!("corridor setup failed: {e}");
    exit(2)
}

/// Lift an exact `f64` specification constant into the working precision.
pub fn ft(x: f64) -> FloatType {
    FromPrimitive::from_f64(x).expect("specification lifts into FloatType")
}

pub(crate) fn spacing() -> FloatType {
    ft(2.0 * std::f64::consts::PI) / ft((1usize << L) as f64)
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
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

pub(crate) fn scalar0(field: &CoupledField<FloatType>, name: &str) -> FloatType {
    field
        .scalar(name)
        .and_then(|s| s.first().copied())
        .unwrap_or_else(|| ft(0.0))
}
