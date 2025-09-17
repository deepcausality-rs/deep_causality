/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::Distribution;
use deep_causality_rand::types::distr::normal::standard_normal::StandardNormal;

macro_rules! standard_normal_tests {
    ($float_type:ty, $tolerance:expr) => {
        #[test]
        fn test_mean_and_std_dev() {
            let mut rng = rng();
            let distr = StandardNormal;
            const NUM_SAMPLES: usize = 100_000;
            let mut samples = Vec::with_capacity(NUM_SAMPLES);
            for _ in 0..NUM_SAMPLES {
                let sample: $float_type = distr.sample(&mut rng);
                samples.push(sample);
            }

            let sum: $float_type = samples.iter().sum();
            let mean = sum / (NUM_SAMPLES as $float_type);

            let variance: $float_type = samples.iter().map(|&x| (x - mean).powi(2)).sum();
            let std_dev = (variance / (NUM_SAMPLES as $float_type)).sqrt();

            assert!(mean.abs() < $tolerance, "Mean {} is not close to 0", mean);
            assert!(
                (std_dev - 1.0).abs() < $tolerance,
                "Std dev {} is not close to 1",
                std_dev
            );
        }

        #[test]
        fn test_symmetry() {
            let mut rng = rng();
            let distr = StandardNormal;
            const NUM_SAMPLES: usize = 10_000;
            let mut pos_count = 0;
            let mut neg_count = 0;
            for _ in 0..NUM_SAMPLES {
                let sample: $float_type = distr.sample(&mut rng);
                if sample > (0.0 as $float_type) {
                    pos_count += 1;
                } else {
                    neg_count += 1;
                }
            }

            // Check if the number of positive and negative samples are roughly equal.
            // The difference should be within a few standard deviations of a binomial distribution.
            let diff = (pos_count as isize - neg_count as isize).abs() as f64;
            let std_dev = (NUM_SAMPLES as f64 * 0.5 * 0.5).sqrt();
            assert!(diff < 5.0 * std_dev, "Distribution is not symmetric");
        }
    };
}

mod f32_tests {
    use super::*;
    use deep_causality_rand::rng;
    standard_normal_tests!(f32, 0.01);
}

mod f64_tests {
    use super::*;
    use deep_causality_rand::rng;
    standard_normal_tests!(f64, 0.01);
}

#[test]
fn test_derived_traits() {
    let sn1 = StandardNormal;
    let sn2 = sn1; // Clone
    let sn3 = sn1; // Copy
    assert_eq!(format!("{:?}", sn1), "StandardNormal");
    assert_eq!(format!("{:?}", sn2), "StandardNormal");
    assert_eq!(format!("{:?}", sn3), "StandardNormal");
}
