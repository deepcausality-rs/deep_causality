/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `CountHistogram`: the width is bounded by the bits an outcome carries, an outcome must fit the
//! width, and the counts add in checked arithmetic. A refused record changes nothing.

use deep_causality_quantum::{CountHistogram, QuantumErrorEnum, ShotHistogram};

#[test]
fn test_the_width_is_bounded_by_the_bits_an_outcome_carries() {
    let bits = usize::BITS as usize;
    assert!(CountHistogram::new(0).is_ok());
    assert_eq!(CountHistogram::new(bits).unwrap().num_bits(), bits);
    let err = CountHistogram::new(bits + 1).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)));
}

#[test]
fn test_an_outcome_must_fit_the_width_and_a_refusal_records_nothing() {
    let mut h = CountHistogram::new(2).unwrap();
    h.record(3).unwrap();
    assert!(matches!(
        h.record(4).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    // Even zero shots of an outcome that does not fit are refused.
    assert!(matches!(
        h.record_n(4, 0).unwrap_err().0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
    assert_eq!(h.total(), 1);
    assert_eq!(h.entries(), vec![(3, 1)]);

    // A zero-width histogram carries the one outcome, 0.
    let mut z = CountHistogram::new(0).unwrap();
    z.record(0).unwrap();
    assert!(z.record(1).is_err());
    assert_eq!(z.total(), 1);

    // At the full width every outcome fits.
    let mut full = CountHistogram::new(usize::BITS as usize).unwrap();
    full.record(usize::MAX).unwrap();
    assert_eq!(full.count(usize::MAX), 1);
}

#[test]
fn test_the_counts_add_in_checked_arithmetic_and_an_overflow_records_nothing() {
    let mut h = CountHistogram::new(1).unwrap();
    h.record_n(1, u64::MAX - 1).unwrap();
    h.record(1).unwrap();
    assert_eq!(h.count(1), u64::MAX);
    assert_eq!(h.total(), u64::MAX);
    // The outcome's own count would overflow.
    assert!(matches!(
        h.record(1).unwrap_err().0,
        QuantumErrorEnum::CalculationError(_)
    ));
    assert_eq!(h.count(1), u64::MAX);
    assert_eq!(h.total(), u64::MAX);
    // The total would overflow while the outcome's own count would not.
    assert!(matches!(
        h.record(0).unwrap_err().0,
        QuantumErrorEnum::CalculationError(_)
    ));
    assert_eq!(h.count(0), 0);
    assert_eq!(h.total(), u64::MAX);
    assert_eq!(h.entries(), vec![(1, u64::MAX)]);
}

#[test]
fn test_zero_shots_record_no_entry() {
    let mut h = CountHistogram::new(2).unwrap();
    h.record_n(1, 0).unwrap();
    assert_eq!(h.total(), 0);
    assert!(h.entries().is_empty());
    assert_eq!(h.count(1), 0);
}
