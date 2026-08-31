/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{CoMonad, Functor, Pure};
use deep_causality_multivector::{CausalMultiField, CausalMultiFieldWitness, Metric};

fn create_test_field() -> CausalMultiField<f32> {
    let shape = [2, 2, 2];
    let metric = Metric::from_signature(2, 0, 0);
    let dx = [0.1f32, 0.1, 0.1];
    CausalMultiField::zeros(shape, metric, dx)
}

#[test]
fn test_witness_new() {
    let _witness = CausalMultiFieldWitness::<f32>::new();
}

#[test]
fn test_functor_fmap() {
    let field = create_test_field();

    // Test mapping f32 -> f32
    // We must use the same type for A and C in CausalMultiField because of the HKT limitations
    // documented in CausalMultiFieldWitness.
    let mapped = CausalMultiFieldWitness::<f32>::fmap(field, |x: f32| x + 1.0);

    let data = mapped.data();
    // zeros + 1.0 = all ones
    for val in data.as_slice() {
        assert!((*val - 1.0f32).abs() < 1e-6f32);
    }
}

#[test]
fn test_pure_lifts_into_a_single_cell() {
    let val = 42.0f32;
    let field = CausalMultiFieldWitness::<f32>::pure(val);

    // The defining property: `pure` is handed one value and `A` has no `Clone`, so the
    // context it builds must hold exactly one coefficient. Asserted as a count, not as a
    // loop over the slice, which would pass vacuously on an empty field.
    assert_eq!(
        field.data().as_slice().len(),
        1,
        "pure must build exactly one cell; more would require duplicating the value"
    );
    assert_eq!(field.data().as_slice()[0], val);

    // The minimal algebra, Cl(0,0,0), is what makes the cell count 1.
    assert_eq!(field.metric().dimension(), 0);
    assert_eq!(field.matrix_dim(), 1);
    assert_eq!(field.shape(), &[1, 1, 1]);
    assert_eq!(field.num_cells(), 1);
}

#[test]
fn test_comonad_left_counit_on_pure() {
    // extract(pure(a)) == a. An independent oracle: it names no internal shape, only the
    // comonad/pure agreement that any lawful pair must satisfy.
    for val in [0.0f32, 42.0, -7.5, f32::MIN_POSITIVE] {
        let field = CausalMultiFieldWitness::<f32>::pure(val);
        assert_eq!(CausalMultiFieldWitness::<f32>::extract(&field), val);
    }
}

#[test]
fn test_comonad_extract() {
    // Let's use `ones`. first element is 1.0.
    let field = CausalMultiFieldWitness::<f32>::pure(5.0); // Simple 1-element(ish) field

    let val = CausalMultiFieldWitness::<f32>::extract(&field);
    assert!((val - 5.0f32).abs() < 1e-6f32);
}

#[test]
fn test_comonad_extend() {
    let field = CausalMultiFieldWitness::<f32>::pure(10.0);

    // extend takes (&Field<A>) -> C
    // and produces Field<C> where every element is the result of applying that function to the *whole* field.
    // The implementation of `extend` does:
    // 1. apply f(fa) -> c_val
    // 2. create new field of same shape filled with c_val.

    let extended = CausalMultiFieldWitness::<f32>::extend(&field, |f| {
        let val = CausalMultiFieldWitness::<f32>::extract(f);
        val + 1.0
    });

    // Extract(field) is 10.0. +1.0 = 11.0.
    // Extended field should be all 11.0.
    let data = extended.data();
    for v in data.as_slice() {
        assert!((*v - 11.0f32).abs() < 1e-6f32);
    }
}

#[test]
fn test_multifield_zeros_creates_correct_shape() {
    let field = create_test_field();
    let data = field.data();

    // Shape should be [Nx, Ny, Nz, D, D] = [2, 2, 2, 2, 2]
    assert_eq!(data.shape(), &[2, 2, 2, 2, 2]);
}

#[test]
fn test_multifield_ones_creates_identity_matrices() {
    let shape = [1, 1, 1];
    let metric = Metric::from_signature(2, 0, 0);
    let dx = [0.1f32, 0.1, 0.1];

    let field = CausalMultiField::ones(shape, metric, dx);
    let data_vec = field.data().clone().to_vec();

    // For a single cell with 2x2 matrix, should be identity
    // [1, 0, 0, 1] in row-major order
    assert!((data_vec[0] - 1.0).abs() < 1e-6);
    assert!(data_vec[1].abs() < 1e-6);
    assert!(data_vec[2].abs() < 1e-6);
    assert!((data_vec[3] - 1.0).abs() < 1e-6);
}

#[test]
fn test_multifield_num_cells() {
    let field = create_test_field();
    assert_eq!(field.num_cells(), 8); // 2 * 2 * 2
}

#[test]
fn test_multifield_metric() {
    let field = create_test_field();
    let metric = field.metric();
    assert_eq!(metric.dimension(), 2);
}

#[test]
fn test_multifield_clone() {
    let field1 = create_test_field();
    let field2 = field1.clone();

    assert_eq!(field1.data().shape(), field2.data().shape());
    assert_eq!(field1.metric(), field2.metric());
}

#[test]
fn test_multifield_add() {
    let field1 = create_test_field();
    let field2 = create_test_field();

    let result = &field1 + &field2;

    // zeros + zeros = zeros
    let data = result.data().clone().to_vec();
    for val in data {
        assert!(val.abs() < 1e-6);
    }
}

#[test]
fn test_multifield_sub() {
    let shape = [1, 1, 1];
    let metric = Metric::from_signature(2, 0, 0);
    let dx = [0.1f32, 0.1, 0.1];

    let field1 = CausalMultiField::ones(shape, metric, dx);
    let field2 = CausalMultiField::zeros(shape, metric, dx);

    let result = field1 - field2;

    // ones - zeros = ones (identity matrices)
    let data = result.data().clone().to_vec();
    assert!((data[0] - 1.0).abs() < 1e-6);
    assert!((data[3] - 1.0).abs() < 1e-6);
}
