/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_quantum::{InstrumentTime, QuantumErrorEnum};

#[test]
fn test_the_time_keeps_its_values_and_bounds_the_shots() {
    let t = InstrumentTime::new(0.5, 1000.0).unwrap();
    assert_eq!((t.shot_time(), t.white_noise_range()), (0.5, 1000.0));
    assert_eq!(t.max_shots(), 2000);
    // The floor: 1000.9 s holds 2001 whole shots of 0.5 s.
    assert_eq!(InstrumentTime::new(0.5, 1000.9).unwrap().max_shots(), 2001);
    // A range of exactly one shot holds one.
    assert_eq!(InstrumentTime::new(0.5, 0.5).unwrap().max_shots(), 1);
    // A ratio past u64 saturates.
    assert_eq!(
        InstrumentTime::new(1.0e-30, 1.0e10).unwrap().max_shots(),
        u64::MAX
    );
}

#[test]
fn test_a_time_that_holds_no_shot_is_refused() {
    for (shot, range) in [(0.0, 1.0), (-1.0, 1.0), (1.0, 0.5)] {
        assert!(matches!(
            InstrumentTime::new(shot, range).unwrap_err().0,
            QuantumErrorEnum::CalculationError(_)
        ));
    }
    for (shot, range) in [(f64::NAN, 1.0), (1.0, f64::INFINITY)] {
        assert!(matches!(
            InstrumentTime::new(shot, range).unwrap_err().0,
            QuantumErrorEnum::NonFiniteValue(_)
        ));
    }
}
