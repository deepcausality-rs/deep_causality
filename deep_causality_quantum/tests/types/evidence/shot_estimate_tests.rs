/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shot statistics at the pipeline's scalar, and the read-out decision as a margin over shots.

use deep_causality_quantum::{
    CheckItem, CheckVerdict, CountHistogram, QuantumErrorEnum, ShotEstimate, Tolerance,
};

fn histogram(ones: u64, zeros: u64) -> CountHistogram {
    let mut h = CountHistogram::new(1);
    h.record_n(1, ones);
    h.record_n(0, zeros);
    h
}

#[test]
fn test_the_estimate_and_its_standard_error() {
    let h = histogram(640, 360);
    let e = ShotEstimate::<f64>::of_outcome(&h, 1).unwrap();
    assert_eq!(e.shots(), 1000);
    assert!((e.estimate() - 0.64).abs() < 1e-15);
    let se = (0.64 * 0.36 / 1000.0_f64).sqrt();
    assert!((e.standard_error() - se).abs() < 1e-15);
    // The complementary outcome has the complementary estimate and the same width.
    let z = ShotEstimate::<f64>::of_outcome(&h, 0).unwrap();
    assert!((z.estimate() - 0.36).abs() < 1e-15);
    assert!((z.standard_error() - se).abs() < 1e-15);
    // of_bit reads the same thing on a one-bit histogram.
    let b = ShotEstimate::<f64>::of_bit(&h, 0).unwrap();
    assert_eq!(b, e);
}

#[test]
fn test_an_empty_histogram_is_not_a_probability() {
    let h = CountHistogram::new(1);
    let err = ShotEstimate::<f64>::of_outcome(&h, 1).unwrap_err();
    match err.0 {
        QuantumErrorEnum::NormalizationError(msg) => {
            assert_eq!(msg, "cannot bridge an empty shot histogram")
        }
        other => panic!("expected NormalizationError, got {other:?}"),
    }
    assert!(matches!(
        ShotEstimate::<f64>::of_bit(&histogram(1, 1), 3)
            .unwrap_err()
            .0,
        QuantumErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_an_accepted_read_out_reports_how_it_was_accepted() {
    // 1024 shots estimating 0.9991 against at_least(0.999): the shortfall is negative, the
    // threshold is the shot-noise width at 1024, and the count is 1024.
    let ones = (0.9991 * 1024.0_f64).round() as u64;
    let h = histogram(ones, 1024 - ones);
    let e = ShotEstimate::<f64>::of_outcome(&h, 1).unwrap();
    let report = e.at_least(0.999);
    assert_eq!(report.examined(), 1024);
    assert_eq!(report.verdict(), CheckVerdict::Accepted);
    let record = &report.checks()[0];
    assert_eq!(record.item, CheckItem::Whole);
    assert!((record.measured - (0.999 - e.estimate())).abs() < 1e-15);
    let width = Tolerance::<f64>::shot_noise()
        .shot_noise_width(e.estimate(), 1024)
        .unwrap();
    assert_eq!(record.threshold, width);
    assert!(
        record.margin < 0.0,
        "above the spec reads as a negative margin"
    );
}

#[test]
fn test_a_shortfall_beyond_the_noise_rejects_and_within_it_accepts() {
    // 1024 shots at 0.990 against at_least(0.999): shortfall 0.009 against a width ≈ 0.0031.
    let h = histogram(1014, 10);
    let e = ShotEstimate::<f64>::of_outcome(&h, 1).unwrap();
    let report = e.at_least(0.999);
    assert_eq!(report.verdict(), CheckVerdict::Rejected);
    assert!(report.worst_margin().unwrap() > 1.0);
    // A shortfall inside one standard error accepts: 1000 shots at 0.500 against 0.510.
    let h = histogram(500, 500);
    let e = ShotEstimate::<f64>::of_outcome(&h, 1).unwrap();
    let report = e.at_least(0.510);
    assert_eq!(report.verdict(), CheckVerdict::Accepted);
    assert!(report.worst_margin().unwrap() <= 1.0 && report.worst_margin().unwrap() > 0.0);
    // And at_most mirrors it.
    assert_eq!(e.at_most(0.490).verdict(), CheckVerdict::Accepted);
    assert_eq!(e.at_most(0.400).verdict(), CheckVerdict::Rejected);
}

#[test]
fn test_separation_in_bits_is_zero_for_equal_estimates_and_grows_with_distance() {
    let a = ShotEstimate::<f64>::of_outcome(&histogram(500, 500), 1).unwrap();
    let b = ShotEstimate::<f64>::of_outcome(&histogram(500, 500), 1).unwrap();
    assert!(a.separation_bits(&b).abs() < 1e-12);
    let c = ShotEstimate::<f64>::of_outcome(&histogram(900, 100), 1).unwrap();
    let d = ShotEstimate::<f64>::of_outcome(&histogram(990, 10), 1).unwrap();
    let near = a.separation_bits(&c);
    let far = a.separation_bits(&d);
    assert!(near > 0.0 && far > near);
    assert!((a.separation_bits(&c) - c.separation_bits(&a)).abs() < 1e-12);
}

#[test]
fn test_the_same_histogram_summarised_at_two_precisions() {
    // The estimate follows the scalar: both compile, the f32 statistics agree with the f64 ones
    // to f32 precision, and the validation tolerances the pipeline would use differ by scalar,
    // which is what §10.4's precision sweep records.
    let h = histogram(640, 360);
    let wide = ShotEstimate::<f64>::of_outcome(&h, 1).unwrap();
    let narrow = ShotEstimate::<f32>::of_outcome(&h, 1).unwrap();
    assert!((wide.estimate() - narrow.estimate() as f64).abs() < f32::EPSILON as f64 * 4.0);
    assert!(
        (wide.standard_error() - narrow.standard_error() as f64).abs() < f32::EPSILON as f64 * 4.0
    );
    let tol_wide = Tolerance::<f64>::state().threshold(2, 1.0).unwrap();
    let tol_narrow = Tolerance::<f32>::state().threshold(2, 1.0).unwrap() as f64;
    assert!(tol_wide < tol_narrow, "the tolerance moved with the scalar");
    // The shot-noise width does not: it reads from the budget, not from epsilon.
    let w64 = Tolerance::<f64>::shot_noise()
        .shot_noise_width(0.64, 1000)
        .unwrap();
    let w32 = Tolerance::<f32>::shot_noise()
        .shot_noise_width(0.64, 1000)
        .unwrap() as f64;
    assert!((w64 - w32).abs() < 1e-6);
    assert_eq!(
        Tolerance::<f64>::shot_noise().shot_noise_width(0.5, 0),
        None
    );
    assert_eq!(Tolerance::<f64>::shot_noise().threshold(2, 1.0), None);
}
