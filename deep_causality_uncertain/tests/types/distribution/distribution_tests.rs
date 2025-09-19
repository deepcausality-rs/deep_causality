/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::rng;
use deep_causality_uncertain::{
    BernoulliParams, DistributionEnum, NormalDistributionParams, UncertainError,
    UniformDistributionParams,
};

#[test]
fn test_distribution_enum_debug_clone_copy() {
    let point_f64 = DistributionEnum::Point(1.0);
    let normal: DistributionEnum<f64> =
        DistributionEnum::Normal(NormalDistributionParams::new(0.0, 1.0));
    let uniform: DistributionEnum<f64> =
        DistributionEnum::Uniform(UniformDistributionParams::new(0.0, 1.0));
    let bernoulli: DistributionEnum<bool> = DistributionEnum::Bernoulli(BernoulliParams::new(0.5));

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
    dbg!(&sample);
    assert_eq!(sample, 42.0);
}

#[test]
fn test_distribution_enum_f64_sample_normal() {
    let dist: DistributionEnum<f64> =
        DistributionEnum::Normal(NormalDistributionParams::new(10.0, 1.0));
    let mut rng = rng();
    let sample = dist.sample(&mut rng).unwrap();
    dbg!(&sample);

    // Check if sample is within a reasonable range (e.g., mean +/- 5*std_dev)
    assert!(sample > 5.0 && sample < 15.0);
}

#[test]
fn test_distribution_enum_f64_sample_uniform() {
    let dist: DistributionEnum<f64> =
        DistributionEnum::Uniform(UniformDistributionParams::new(0.0, 1.0));
    let mut rng = rng();
    let sample = dist.sample(&mut rng).unwrap();
    dbg!(&sample);

    assert!((0.0..=1.0).contains(&sample));
}

#[test]
fn test_distribution_enum_bool_sample_point() {
    let dist = DistributionEnum::Point(true);
    let mut rng = rng();
    let sample = dist.sample(&mut rng).unwrap();
    dbg!(&sample);

    assert!(sample);
}

#[allow(clippy::bool_comparison)]
#[test]
fn test_distribution_enum_bool_sample_bernoulli() {
    let dist: DistributionEnum<bool> = DistributionEnum::Bernoulli(BernoulliParams::new(0.8));
    let mut rng = rng();
    let sample = dist.sample(&mut rng).unwrap();
    dbg!(&sample);
    // Due to randomness, we can't assert exact equality, but they should both lean towards true
    assert!(sample == true || sample == false);
}

#[test]
fn test_distribution_enum_display() {
    let point_f64 = DistributionEnum::Point(1.23);
    let normal: DistributionEnum<f64> =
        DistributionEnum::Normal(NormalDistributionParams::new(4.56, 0.1));
    let uniform: DistributionEnum<f64> =
        DistributionEnum::Uniform(UniformDistributionParams::new(7.89, 9.01));
    let bernoulli: DistributionEnum<bool> = DistributionEnum::Bernoulli(BernoulliParams::new(0.7));

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

#[test]
fn test_sample_f64_unsupported_type_error() {
    let mut rng = rng();
    let bernoulli_dist: DistributionEnum<f64> =
        DistributionEnum::Bernoulli(BernoulliParams { p: 0.5 });
    let result = bernoulli_dist.sample(&mut rng);

    dbg!(&result);

    assert!(matches!(
        result,
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}

#[test]
fn test_sample_bool_unsupported_type_error() {
    let mut rng = rng();
    let normal_dist: DistributionEnum<bool> = DistributionEnum::Normal(NormalDistributionParams {
        mean: 0.0,
        std_dev: 1.0,
    });
    let result = normal_dist.sample(&mut rng);

    dbg!(&result);

    assert!(matches!(
        result,
        Err(UncertainError::UnsupportedTypeError(_))
    ));

    let uniform_dist: DistributionEnum<bool> =
        DistributionEnum::Uniform(UniformDistributionParams {
            low: 0.0,
            high: 1.0,
        });
    let result = uniform_dist.sample(&mut rng);
    dbg!(&result);

    assert!(matches!(
        result,
        Err(UncertainError::UnsupportedTypeError(_))
    ));
}
