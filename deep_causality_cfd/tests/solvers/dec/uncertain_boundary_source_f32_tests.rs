/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The uncertain boundary source at `f32` — the compile that is the test.
//!
//! This crate's uncertain surface used to carry `+ ProbabilisticType` on sixteen bounds, and that
//! trait was implemented for `f64` and `Float106` and for nothing else. `f32` was therefore not a
//! scalar the uncertain march could be instantiated at — not because any of the arithmetic below
//! is hard at `f32`, but because a trait in another crate had no impl for it.
//!
//! The bound is gone and the scalar is a parameter, so these instantiate. The assertions that
//! follow the construction are ordinary, and deliberately so: what is under test is that the code
//! **exists** at this scalar. A green assertion on a march that could not previously be typed is
//! the whole of the claim.

use deep_causality_cfd::{DropoutVerbosity, UncertainBoundarySource, UncertainInflowZone};
use deep_causality_uncertain::{MaybeUncertain, Uncertain};

/// A gate small enough to settle quickly, stated at `f32` — as the three probabilities now are.
fn fast_source(default_value: f32) -> UncertainBoundarySource<f32> {
    UncertainBoundarySource::new(default_value)
        .with_presence_gate(0.5, 0.9, 0.1, 64)
        .with_collapse_samples(16)
        .with_gate_seed(0x5EED)
}

#[test]
fn a_present_f32_sample_resolves_to_its_value() {
    let source = fast_source(0.0);
    let mut last_good = 0.0f32;
    let sample = MaybeUncertain::<f32>::from_uncertain(Uncertain::normal(3.0, 0.01));

    let (value, dropout) = source
        .resolve(&sample, &mut last_good, 0)
        .expect("a certainly-present f32 sample resolves");

    assert!(!dropout, "a present sample must not be a dropout");
    assert!(
        (value - 3.0).abs() < 0.1,
        "collapsed to {value}, expected about 3.0"
    );
    assert!(
        (last_good - 3.0).abs() < 0.1,
        "last-good is {last_good}, expected about 3.0"
    );
}

#[test]
fn an_absent_f32_sample_falls_back_to_the_last_good() {
    let source = fast_source(7.0);
    let mut last_good = 7.0f32;
    let sample = MaybeUncertain::<f32>::always_none();

    let (value, dropout) = source
        .resolve(&sample, &mut last_good, 0)
        .expect("a dropout falls back rather than failing");

    assert!(dropout, "an absent sample must be a dropout");
    assert_eq!(value, 7.0);
    assert_eq!(last_good, 7.0);
}

/// The zone, too — the fluid application of the same source, at the same scalar.
#[test]
fn an_inflow_zone_types_at_f32() {
    let zone: UncertainInflowZone<f32> = UncertainInflowZone::new(1, false, 0, 1.5)
        .with_presence_gate(0.5, 0.9, 0.1, 64)
        .with_collapse_samples(16)
        .with_verbosity(DropoutVerbosity::Silent);

    // Constructed and configured at `f32`; the accessors confirm the axes came through unchanged.
    assert_eq!(zone.wall_axis(), 1);
    assert_eq!(zone.flow_axis(), 0);
    assert!(!zone.max_side());
}
