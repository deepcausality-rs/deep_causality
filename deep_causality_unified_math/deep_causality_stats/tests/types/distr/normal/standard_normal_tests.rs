/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::Distribution;
use deep_causality_stats::StandardNormal;

macro_rules! standard_normal_tests {
    ($float_type:ty, $tolerance:expr) => {
        #[test]
        #[cfg_attr(miri, ignore)]
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
        // Disabled under Miri: soft-float emulation skews the statistical
        // sampling enough to exceed the symmetry tolerance. Test is correct
        // under normal CI.
        #[cfg_attr(miri, ignore)]
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
            assert!(diff < 8.0 * std_dev, "Distribution is not symmetric");
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

// `Float106` cannot ride the `as $float_type` macro above (no primitive cast),
// so its standard-normal draw is exercised by hand here.
mod f106_tests {
    use super::*;
    use deep_causality_algebra::Real;
    use deep_causality_num::Float106;
    use deep_causality_rand::rng;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn test_mean_and_std_dev() {
        let mut rng = rng();
        let distr = StandardNormal;
        const NUM_SAMPLES: usize = 100_000;
        let samples: Vec<Float106> = (0..NUM_SAMPLES).map(|_| distr.sample(&mut rng)).collect();

        let n = Float106::from(NUM_SAMPLES as f64);
        let zero = Float106::from(0.0);
        let mean = samples.iter().copied().fold(zero, |a, b| a + b) / n;
        let variance = samples
            .iter()
            .copied()
            .fold(zero, |a, x| a + (x - mean) * (x - mean))
            / n;
        let std_dev = variance.sqrt();

        let tol = Float106::from(0.02);
        assert!(
            mean.abs() < tol,
            "Float106 normal mean {mean:?} not close to 0"
        );
        assert!(
            (std_dev - Float106::from(1.0)).abs() < tol,
            "Float106 normal std dev {std_dev:?} not close to 1"
        );
    }

    #[test]
    fn test_double_double_entropy() {
        // Honesty check: a genuine double-double draw carries sub-f64 precision, so
        // some samples have a nonzero low limb. An f64 draw widened to double-double
        // would always have lo == 0.
        let mut rng = rng();
        let distr = StandardNormal;
        let any_low = (0..2000).any(|_| {
            let s: Float106 = distr.sample(&mut rng);
            s.lo() != 0.0
        });
        assert!(any_low, "Float106 normal draws never populate the low limb");
    }
}

// Precision-generic sampling: one function, several scalars.
//
// This exercised the `RealRng` capability bound, which is removed — it had no consumer in the
// workspace and its documented extension path was forbidden by the orphan rule. The property it
// pinned is worth keeping, so it is restated against what exists today.
//
// Note the shape of the bound. A caller generic in its scalar must currently name two concrete
// distribution types in a `where` clause, which no other crate in unified math asks of anyone.
// Group 3 replaces both clauses with a single scalar bound; until then this test records the cost
// in the form a reader can see.
mod precision_generic_tests {
    use deep_causality_algebra::RealField;
    use deep_causality_num::Float106;
    use deep_causality_rand::{Distribution, rng};
    use deep_causality_stats::{StandardNormal, StandardUniform};

    fn exercise<R: RealField>()
    where
        StandardUniform: Distribution<R>,
        StandardNormal: Distribution<R>,
    {
        let mut g = rng();
        for _ in 0..1000 {
            let u: R = StandardUniform.sample(&mut g);
            assert!(u >= R::zero() && u < R::one(), "uniform_01 out of [0, 1)");
        }
        for _ in 0..1000 {
            let z: R = StandardNormal.sample(&mut g);
            assert!(z.is_finite(), "standard normal draw not finite");
        }
    }

    #[test]
    fn the_same_body_samples_f64_and_f106() {
        exercise::<f64>();
        exercise::<Float106>();
    }
}
