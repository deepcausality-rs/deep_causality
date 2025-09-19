/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::{SampleBorrow, SampleUniform};

// Mock SampleUniform type for testing
#[derive(Debug, PartialEq)]
struct MockSampleUniform(u32);

// Implement SampleUniform for our mock type
impl SampleUniform for MockSampleUniform {
    type Sampler = MockSampler;
}

// Dummy Sampler for MockSampleUniform (not used in these tests, but required by trait)
struct MockSampler;

impl deep_causality_rand::UniformSampler for MockSampler {
    type X = MockSampleUniform;

    fn new<B1, B2>(
        _low: B1,
        _high: B2,
    ) -> Result<Self, deep_causality_rand::UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        unimplemented!()
    }

    fn new_inclusive<B1, B2>(
        _low: B1,
        _high: B2,
    ) -> Result<Self, deep_causality_rand::UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        unimplemented!()
    }

    fn sample<R: deep_causality_rand::Rng + ?Sized>(&self, _rng: &mut R) -> Self::X {
        unimplemented!()
    }
}

#[test]
fn test_sample_borrow_for_direct_type() {
    let value = MockSampleUniform(42);
    let borrowed_ref = value.borrow();
    assert_eq!(borrowed_ref, &value);
    assert_eq!(borrowed_ref.0, 42);
}

#[test]
fn test_sample_borrow_for_reference_type() {
    let value = MockSampleUniform(100);
    let value_ref = &value;
    let borrowed_ref = value_ref.borrow();
    assert_eq!(borrowed_ref, &value);
    assert_eq!(borrowed_ref.0, 100);
}
