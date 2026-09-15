/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::rng;
use deep_causality_uncertain::{
    BernoulliParams, DistributionEnum, LeafOrdinals, NormalDistributionParams, Sample, Uncertain,
    UncertainBool, UncertainError, UniformDistributionParams,
};

#[test]
fn test_distribution_enum_debug_clone_copy() {
    let point_f64 = DistributionEnum::Point(1.0);
    let normal: DistributionEnum<f64> =
        DistributionEnum::Normal(NormalDistributionParams::new(0.0, 1.0));
    let uniform: DistributionEnum<f64> =
        DistributionEnum::Uniform(UniformDistributionParams::new(0.0, 1.0));
    let bernoulli: DistributionEnum<f64> = DistributionEnum::Bernoulli(BernoulliParams::new(0.5));

    // Test Debug
    assert_eq!(format!("{:?}", point_f64), "Point(1.0)");
    assert_eq!(
        format!("{:?}", normal),
        "Normal(NormalDistributionParams { mean: 0.0, std_dev: 1.0 })"
    );
    assert_eq!(
        format!("{:?}", uniform),
        "Uniform(UniformDistributionParams { low: 0.0, high: 1.0 })"
    );
    assert_eq!(
        format!("{:?}", bernoulli),
        "Bernoulli(BernoulliParams { p: 0.5 })"
    );

    // Test Clone
    let cloned_point_f64 = point_f64;
    let cloned_normal = normal;
    let cloned_uniform = uniform;
    let cloned_bernoulli = bernoulli;

    assert!(matches!(cloned_point_f64, DistributionEnum::Point(1.0)));
    assert!(matches!(cloned_normal, DistributionEnum::Normal(_)));
    assert!(matches!(cloned_uniform, DistributionEnum::Uniform(_)));
    assert!(matches!(cloned_bernoulli, DistributionEnum::Bernoulli(_)));

    // Test Copy (by assignment)
    let copied_point_f64 = point_f64;
    let copied_normal = normal;
    let copied_uniform = uniform;
    let copied_bernoulli = bernoulli;

    assert!(matches!(copied_point_f64, DistributionEnum::Point(1.0)));
    assert!(matches!(copied_normal, DistributionEnum::Normal(_)));
    assert!(matches!(copied_uniform, DistributionEnum::Uniform(_)));
    assert!(matches!(copied_bernoulli, DistributionEnum::Bernoulli(_)));
}

#[test]
fn test_distribution_enum_f64_sample_point() {
    let dist = DistributionEnum::Point(42.0);
    let mut rng = rng();
    let sample = dist.sample(&mut rng).unwrap();
    assert_eq!(sample, Sample::Real(42.0));
}

#[test]
fn test_distribution_enum_f64_sample_normal() {
    let dist: DistributionEnum<f64> =
        DistributionEnum::Normal(NormalDistributionParams::new(10.0, 1.0));
    let mut rng = rng();
    let sample = real(dist.sample(&mut rng).unwrap());

    // Check if sample is within a reasonable range (e.g., mean +/- 5*std_dev)
    assert!(sample > 5.0 && sample < 15.0);
}

#[test]
fn test_distribution_enum_f64_sample_uniform() {
    let dist: DistributionEnum<f64> =
        DistributionEnum::Uniform(UniformDistributionParams::new(0.0, 1.0));
    let mut rng = rng();
    let sample = real(dist.sample(&mut rng).unwrap());

    assert!((0.0..=1.0).contains(&sample));
}

/// A Bernoulli draws a truth value out of a distribution enum carrying a real scalar.
///
/// This was an `UnsupportedTypeError`, back when there was a `DistributionEnum<bool>` for the
/// Boolean case to live in. There is not: `bool` is not a scalar, a Bernoulli is parameterised by
/// a probability rather than by what it produces, and what it produces is a `Sample::Bool` from a
/// graph of any scalar. The error had nothing left to report.
#[test]
fn test_distribution_enum_bernoulli_draws_a_boolean_at_a_real_scalar() {
    let mut rng = rng();
    let dist: DistributionEnum<f64> = DistributionEnum::Bernoulli(BernoulliParams::new(0.8));
    assert!(matches!(dist.sample(&mut rng).unwrap(), Sample::Bool(_)));

    // The endpoints are exact, which is what holding `p` as fixed point buys.
    let never: DistributionEnum<f64> = DistributionEnum::Bernoulli(BernoulliParams::new(0.0));
    let always: DistributionEnum<f64> = DistributionEnum::Bernoulli(BernoulliParams::new(1.0));
    for _ in 0..64 {
        assert_eq!(never.sample(&mut rng).unwrap(), Sample::Bool(false));
        assert_eq!(always.sample(&mut rng).unwrap(), Sample::Bool(true));
    }
}

/// Only `Point` draws nothing, and both pre-passes ask this one question.
#[test]
fn test_only_point_draws_nothing() {
    let point: DistributionEnum<f64> = DistributionEnum::Point(1.0);
    let normal: DistributionEnum<f64> =
        DistributionEnum::Normal(NormalDistributionParams::new(0.0, 1.0));
    let uniform: DistributionEnum<f64> =
        DistributionEnum::Uniform(UniformDistributionParams::new(0.0, 1.0));
    let bernoulli: DistributionEnum<f64> = DistributionEnum::Bernoulli(BernoulliParams::new(0.5));

    // `draws()` is crate-internal; what it decides is observable as the leaf-ordinal count.
    assert!(LeafOrdinals::new(&Uncertain::<f64>::point(1.0)).is_empty());
    assert_eq!(
        LeafOrdinals::new(&Uncertain::<f64>::normal(0.0, 1.0)).len(),
        1
    );
    assert_eq!(
        LeafOrdinals::new(&Uncertain::<f64>::uniform(0.0, 1.0)).len(),
        1
    );
    assert_eq!(
        LeafOrdinals::for_bool(&UncertainBool::<f64>::bernoulli(0.5)).len(),
        1
    );

    // And the four values above are the ones those graphs were built from.
    assert!(matches!(point, DistributionEnum::Point(_)));
    assert!(matches!(normal, DistributionEnum::Normal(_)));
    assert!(matches!(uniform, DistributionEnum::Uniform(_)));
    assert!(matches!(bernoulli, DistributionEnum::Bernoulli(_)));
}

#[test]
fn test_distribution_enum_display() {
    let point_f64 = DistributionEnum::Point(1.23);
    let normal: DistributionEnum<f64> =
        DistributionEnum::Normal(NormalDistributionParams::new(4.56, 0.1));
    let uniform: DistributionEnum<f64> =
        DistributionEnum::Uniform(UniformDistributionParams::new(7.89, 9.01));
    let bernoulli: DistributionEnum<f64> = DistributionEnum::Bernoulli(BernoulliParams::new(0.7));

    assert_eq!(format!("{}", point_f64), "Distribution: Point { D: 1.23 }");
    assert_eq!(
        format!("{}", normal),
        "Distribution: Normal { D: NormalDistributionParams { mean:  4.5600 , std_dev:  0.1000  } }"
    );
    assert_eq!(
        format!("{}", uniform),
        "Distribution: Uniform { D: UniformDistributionParams { low: 7.8900 , high: 9.0100 } }"
    );
    assert_eq!(
        format!("{}", bernoulli),
        "Distribution: Bernoulli { D: BernoulliParams { p: 0.70 } }"
    );
}

/// A construction refusal still reports, and says which parameter it refused.
#[test]
fn test_construction_refusals_are_carried_through() {
    let mut rng = rng();

    // `Normal::new` requires a *finite* standard deviation, not a positive one: a negative sigma
    // is the same symmetric distribution reflected, so it is accepted and draws normally.
    let non_finite_sd: DistributionEnum<f64> =
        DistributionEnum::Normal(NormalDistributionParams::new(0.0, f64::INFINITY));
    assert!(matches!(
        non_finite_sd.sample(&mut rng),
        Err(UncertainError::NormalDistributionError(_))
    ));

    let negative_sd: DistributionEnum<f64> =
        DistributionEnum::Normal(NormalDistributionParams::new(0.0, -1.0));
    assert!(matches!(
        negative_sd.sample(&mut rng).unwrap(),
        Sample::Real(_)
    ));

    let empty_range: DistributionEnum<f64> =
        DistributionEnum::Uniform(UniformDistributionParams::new(1.0, 1.0));
    assert!(matches!(
        empty_range.sample(&mut rng),
        Err(UncertainError::UniformDistributionError(_))
    ));

    let bad_p: DistributionEnum<f64> = DistributionEnum::Bernoulli(BernoulliParams::new(2.0));
    assert!(matches!(
        bad_p.sample(&mut rng),
        Err(UncertainError::BernoulliDistributionError(_))
    ));
}

/// The real half of a sample, for an assertion that wants a number.
fn real(sample: Sample<f64>) -> f64 {
    match sample {
        Sample::Real(v) => v,
        Sample::Bool(b) => panic!("expected a real sample, found Bool({b})"),
    }
}
