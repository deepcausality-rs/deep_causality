/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::ReactionRate;

#[test]
fn test_reaction_rate_valid() {
    let k = ReactionRate::<f64>::new(1.0e9).unwrap();
    assert_eq!(k.value(), 1.0e9);
    assert_eq!(ReactionRate::<f64>::new(0.0).unwrap().value(), 0.0);
}

#[test]
fn test_reaction_rate_rejects_negative() {
    assert!(ReactionRate::<f64>::new(-1.0).is_err());
}

#[test]
fn test_reaction_rate_rejects_nonfinite() {
    assert!(ReactionRate::<f64>::new(f64::NAN).is_err());
    assert!(ReactionRate::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_reaction_rate_new_unchecked() {
    let k = ReactionRate::<f64>::new_unchecked(3.3e8);
    assert_eq!(k.value(), 3.3e8);
}

#[test]
fn test_reaction_rate_default() {
    let k: ReactionRate<f64> = Default::default();
    assert_eq!(k.value(), 0.0);
}

#[test]
fn test_reaction_rate_into_f64() {
    let k = ReactionRate::<f64>::new(8.8e8).unwrap();
    let v: f64 = k.into();
    assert_eq!(v, 8.8e8);
}
