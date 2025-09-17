/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_rand::{Distribution, Rng, RngCore, Uniform, UniformDistributionError};

// Mock Rng for integer tests
struct MockIntRng {
    val: u64,
}

impl RngCore for MockIntRng {
    fn next_u32(&mut self) -> u32 {
        self.val as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.val
    }
    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        unimplemented!()
    }
}

impl Rng for MockIntRng {}

macro_rules! uniform_int_tests {
    ($ty:ty, $mod_name:ident) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn test_new() {
                let uniform = Uniform::<$ty>::new(10, 20).unwrap();
                let mut rng = MockIntRng { val: 5 };
                let sample = uniform.sample(&mut rng);
                assert!((10..20).contains(&sample));

                let res = Uniform::<$ty>::new(20, 10);
                assert_eq!(res.unwrap_err(), UniformDistributionError::InvalidRange);
            }

            #[test]
            fn test_new_inclusive() {
                let uniform = Uniform::<$ty>::new_inclusive(10, 20).unwrap();
                let mut rng = MockIntRng { val: 10 };
                let sample = uniform.sample(&mut rng);
                assert!((10..=20).contains(&sample));

                let res = Uniform::<$ty>::new_inclusive(21, 20);
                assert_eq!(res.unwrap_err(), UniformDistributionError::InvalidRange);
            }

            #[test]
            fn test_sample_range() {
                let mut rng = MockIntRng { val: 0 };
                let sample = Uniform::<$ty>::new(10, 20).unwrap().sample(&mut rng);
                assert_eq!(sample, 10);

                let mut rng = MockIntRng { val: 9 };
                let sample = Uniform::<$ty>::new(10, 20).unwrap().sample(&mut rng);
                assert_eq!(sample, 19);

                let mut rng = MockIntRng { val: 10 };
                let sample = Uniform::<$ty>::new(10, 20).unwrap().sample(&mut rng);
                assert_eq!(sample, 10);
            }
        }
    };
}

uniform_int_tests!(u32, u32_tests);
uniform_int_tests!(u64, u64_tests);

macro_rules! uniform_float_tests {
    ($ty:ty, $mod_name:ident, $tolerance:expr) => {
        mod $mod_name {
            use super::*;
            use deep_causality_rand::rng;

            #[test]
            fn test_new() {
                let uniform = Uniform::<$ty>::new(10.0, 20.0).unwrap();
                let mut rng = rng();
                let sample = uniform.sample(&mut rng);
                assert!((10.0..20.0).contains(&sample));

                let res = Uniform::<$ty>::new(20.0, 10.0);
                assert_eq!(res.unwrap_err(), UniformDistributionError::EmptyRange);

                let res = Uniform::<$ty>::new(10.0, <$ty>::INFINITY);
                assert_eq!(res.unwrap_err(), UniformDistributionError::NonFinite);
            }

            #[test]
            fn test_new_inclusive() {
                let uniform = Uniform::<$ty>::new_inclusive(10.0, 20.0).unwrap();
                let mut rng = rng();
                let sample = uniform.sample(&mut rng);
                assert!((10.0..=20.0).contains(&sample));

                let res = Uniform::<$ty>::new_inclusive(21.0, 20.0);
                assert_eq!(res.unwrap_err(), UniformDistributionError::EmptyRange);
            }

            #[test]
            fn test_sample_distribution() {
                let low = 10.0;
                let high = 20.0;
                let uniform = Uniform::<$ty>::new(low, high).unwrap();
                let mut rng = rng();
                const NUM_SAMPLES: usize = 100_000;
                let mut sum = 0.0;
                for _ in 0..NUM_SAMPLES {
                    let sample = uniform.sample(&mut rng);
                    assert!((low..high).contains(&sample));
                    sum += sample as f64;
                }
                let mean = sum / NUM_SAMPLES as f64;
                let expected_mean = (low + high) as f64 / 2.0;
                assert!((mean - expected_mean).abs() < $tolerance);
            }
        }
    };
}

uniform_float_tests!(f32, f32_tests, 0.05);
uniform_float_tests!(f64, f64_tests, 0.05);
