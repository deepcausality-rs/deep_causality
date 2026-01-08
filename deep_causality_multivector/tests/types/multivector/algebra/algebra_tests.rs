/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorError, Metric, MultiVector};
use deep_causality_num::{Complex, RealField, Zero};

// Helper for approximate float comparison
const EPSILON: f64 = 1e-9;

// ============================================================================
// Tier 1: The Container (Storage & Linear Combinations) - using f64
// Requirements: AddGroup (Add, Sub, Neg, Zero)
// ============================================================================

#[test]
fn test_zero() {
    let metric = Metric::Euclidean(2);
    let mv_zero = CausalMultiVector::<f64>::zero(metric);
    assert_eq!(mv_zero.data(), &vec![0.0, 0.0, 0.0, 0.0]);
    assert_eq!(mv_zero.metric(), metric);

    let metric_3d = Metric::Euclidean(3);
    let mv_zero_3d = CausalMultiVector::<f64>::zero(metric_3d);
    assert_eq!(mv_zero_3d.data().len(), 8); // 2^3 = 8
    assert!(mv_zero_3d.data().iter().all(|&x| x == 0.0));
}

#[test]
fn test_add() {
    let metric = Metric::Euclidean(2);
    let mv1 = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], metric).unwrap();
    let mv2 = CausalMultiVector::new(vec![5.0, 6.0, 7.0, 8.0], metric).unwrap();
    let expected = vec![6.0, 8.0, 10.0, 12.0];
    let result = mv1.add(&mv2);
    assert_eq!(result.data(), &expected);
    assert_eq!(result.metric(), metric);
}

#[test]
fn test_add_with_negative_numbers() {
    let metric = Metric::Euclidean(2);
    let mv1 = CausalMultiVector::new(vec![1.0, -2.0, 3.0, -4.0], metric).unwrap();
    let mv2 = CausalMultiVector::new(vec![-5.0, 6.0, -7.0, 8.0], metric).unwrap();
    let expected = vec![-4.0, 4.0, -4.0, 4.0];
    let result = mv1.add(&mv2);
    assert_eq!(result.data(), &expected);
}

#[test]
#[should_panic(expected = "Metric mismatch in add")]
fn test_add_metric_mismatch_panics() {
    let metric1 = Metric::Euclidean(2);
    let metric2 = Metric::Euclidean(3);
    let mv1 = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], metric1).unwrap();
    let mv2 =
        CausalMultiVector::new(vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0], metric2).unwrap();
    mv1.add(&mv2);
}

#[test]
fn test_sub() {
    let metric = Metric::Euclidean(2);
    let mv1 = CausalMultiVector::new(vec![5.0, 6.0, 7.0, 8.0], metric).unwrap();
    let mv2 = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], metric).unwrap();
    let expected = vec![4.0, 4.0, 4.0, 4.0];
    let result = mv1.sub(&mv2);
    assert_eq!(result.data(), &expected);
    assert_eq!(result.metric(), metric);
}

#[test]
fn test_sub_resulting_in_negative_numbers() {
    let metric = Metric::Euclidean(2);
    let mv1 = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], metric).unwrap();
    let mv2 = CausalMultiVector::new(vec![5.0, 6.0, 7.0, 8.0], metric).unwrap();
    let expected = vec![-4.0, -4.0, -4.0, -4.0];
    let result = mv1.sub(&mv2);
    assert_eq!(result.data(), &expected);
}

#[test]
#[should_panic(expected = "Metric mismatch in sub")]
fn test_sub_metric_mismatch_panics() {
    let metric1 = Metric::Euclidean(2);
    let metric2 = Metric::Euclidean(3);
    let mv1 = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], metric1).unwrap();
    let mv2 =
        CausalMultiVector::new(vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0], metric2).unwrap(); // Different dimension
    mv1.sub(&mv2);
}

// ============================================================================
// Tier 2: The Vector Space (Scaling) - using f64
// Requirements: Module<S> (Vector Space over Scalar S)
// ============================================================================

#[test]
fn test_scale_positive_scalar() {
    let metric = Metric::Euclidean(2);
    let mv = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], metric).unwrap();
    let scalar = 2.0f64;
    let expected = vec![2.0, 4.0, 6.0, 8.0];
    let result = mv.scale(scalar);
    assert_eq!(result.data(), &expected);
}

#[test]
fn test_scale_negative_scalar() {
    let metric = Metric::Euclidean(2);
    let mv = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], metric).unwrap();
    let scalar = -1.0f64;
    let expected = vec![-1.0, -2.0, -3.0, -4.0];
    let result = mv.scale(scalar);
    assert_eq!(result.data(), &expected);
}

#[test]
fn test_scale_zero_scalar() {
    let metric = Metric::Euclidean(2);
    let mv = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], metric).unwrap();
    let scalar = 0.0f64;
    let expected = vec![0.0, 0.0, 0.0, 0.0];
    let result = mv.scale(scalar);
    assert_eq!(result.data(), &expected);
}

#[test]
fn test_normalize_non_zero_multivector() {
    let metric = Metric::Euclidean(1); // G(1) -> e0^2 = 1
    let mv = CausalMultiVector::new(vec![0.0, 3.0], metric).unwrap(); // A vector 3e1
    let result = mv.normalize();
    // Magnitude of 3e1 is 3. Normalized should be 1e1
    assert!((result.data()[1] - 1.0).abs() < EPSILON);
    assert!((result.squared_magnitude() - 1.0).abs() < EPSILON); // Check unit magnitude
}

#[test]
fn test_normalize_zero_multivector() {
    let metric = Metric::Euclidean(2);
    let mv = CausalMultiVector::new(vec![0.0, 0.0, 0.0, 0.0], metric).unwrap();
    let result = mv.normalize();
    // Normalizing a zero vector should return itself
    assert_eq!(result.data(), &vec![0.0, 0.0, 0.0, 0.0]);
}

#[test]
fn test_normalize_scalar() {
    let metric = Metric::Euclidean(0); // Scalar only
    let mv = CausalMultiVector::new(vec![5.0], metric).unwrap();
    let result = mv.normalize();
    assert!((result.data()[0] - 1.0).abs() < EPSILON);
    assert!((result.squared_magnitude() - 1.0).abs() < EPSILON);
}

#[test]
fn test_normalize_with_negative_components() {
    let metric = Metric::Euclidean(1);
    let mv = CausalMultiVector::new(vec![0.0, -4.0], metric).unwrap(); // -4e1
    let result = mv.normalize();
    assert!((result.data()[1] - (-1.0)).abs() < EPSILON);
    assert!((result.squared_magnitude() - 1.0).abs() < EPSILON);
}

// ============================================================================
// Tier 3: The Standard Clifford Algebra (Commutative Coefficients) - using f64
// Requirements: Field (Commutative Ring + Division)
// ============================================================================

#[test]
fn test_commutator_scalars() {
    let metric = Metric::Euclidean(0);
    let mv1 = CausalMultiVector::new(vec![5.0], metric).unwrap();
    let mv2 = CausalMultiVector::new(vec![3.0], metric).unwrap();
    // [A, B] = AB - BA. For scalars, AB = BA, so commutator is 0.
    let result = mv1.commutator(&mv2);
    assert!((result.data()[0]).abs() < EPSILON);
}

#[test]
fn test_commutator_vectors_euclidean() {
    let metric = Metric::Euclidean(2); // e1^2=1, e2^2=1
    let mv_e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], metric).unwrap(); // e1
    let mv_e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], metric).unwrap(); // e2

    // For euclidean vectors, e_i e_j = e_ij (i!=j) and e_j e_i = -e_ij
    // So [e1, e2] = e1e2 - e2e1 = e12 - (-e12) = 2e12
    let result = mv_e1.commutator(&mv_e2);
    assert!((result.data()[3] - 2.0).abs() < EPSILON); // e12 component
    assert!((result.data()[0]).abs() < EPSILON);
    assert!((result.data()[1]).abs() < EPSILON);
    assert!((result.data()[2]).abs() < EPSILON);
}

#[test]
fn test_inverse_scalar_non_zero() {
    let metric = Metric::Euclidean(0);
    let mv = CausalMultiVector::new(vec![2.0], metric).unwrap();
    let result = mv.inverse().unwrap();
    assert!((result.data()[0] - 0.5).abs() < EPSILON);
}

#[test]
fn test_inverse_scalar_zero_error() {
    let metric = Metric::Euclidean(0);
    let mv = CausalMultiVector::new(vec![0.0], metric).unwrap();
    let err = mv.inverse().unwrap_err();
    assert_eq!(err, CausalMultiVectorError::zero_magnitude());
}

#[test]
fn test_inverse_vector_euclidean() {
    let metric = Metric::Euclidean(1); // e1^2 = 1
    let mv_e1 = CausalMultiVector::new(vec![0.0, 2.0], metric).unwrap(); // 2e1
    // (2e1)^-1 = (1/2)e1 (because e1^2 = 1)
    let result = mv_e1.inverse().unwrap();
    assert!((result.data()[1] - 0.5).abs() < EPSILON);
}

#[test]
fn test_inverse_bivector_euclidean() {
    let metric = Metric::Euclidean(2); // e1^2=1, e2^2=1 -> e12^2 = -1
    let mv_e12 = CausalMultiVector::new(vec![0.0, 0.0, 0.0, 2.0], metric).unwrap(); // 2e12
    // (2e12)^-1 = (1/2) * e12 / (e12^2) = (1/2) * e12 / -1 = (-1/2)e12
    let result = mv_e12.inverse().unwrap();
    assert!((result.data()[3] - (-0.5)).abs() < EPSILON);
}

#[test]
fn test_geometric_product_scalars() {
    let metric = Metric::Euclidean(0);
    let mv1 = CausalMultiVector::new(vec![2.0], metric).unwrap();
    let mv2 = CausalMultiVector::new(vec![3.0], metric).unwrap();
    let result = mv1.geometric_product(&mv2);
    assert_eq!(result.data(), &vec![6.0]);
}

#[test]
fn test_geometric_product_e1_e2_euclidean() {
    let metric = Metric::Euclidean(2); // e1^2=1, e2^2=1
    let mv_e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], metric).unwrap(); // e1
    let mv_e2 = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], metric).unwrap(); // e2
    // e1 * e2 = e12
    let result = mv_e1.geometric_product(&mv_e2);
    assert!((result.data()[3] - 1.0).abs() < EPSILON); // e12 component
    assert_eq!(result.data().iter().filter(|&&x| x != 0.0).count(), 1);
}

#[test]
fn test_geometric_product_e1_e1_euclidean() {
    let metric = Metric::Euclidean(2); // e1^2=1
    let mv_e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], metric).unwrap(); // e1
    // e1 * e1 = 1
    let result = mv_e1.geometric_product(&mv_e1);
    assert!((result.data()[0] - 1.0).abs() < EPSILON); // scalar component
    assert_eq!(result.data().iter().filter(|&&x| x != 0.0).count(), 1);
}

#[test]
fn test_geometric_product_complex_case() {
    // (1 + 2e1) * (3 + 4e2) in G(2) Euclidean
    // = 3 + 4e2 + 6e1 + 8e1e2
    let metric = Metric::Euclidean(2);
    let mv1 = CausalMultiVector::new(vec![1.0, 2.0, 0.0, 0.0], metric).unwrap(); // 1 + 2e1
    let mv2 = CausalMultiVector::new(vec![3.0, 0.0, 4.0, 0.0], metric).unwrap(); // 3 + 4e2

    let result = mv1.geometric_product(&mv2);
    // Scalar: 1*3 = 3
    // e1: 2*3 = 6
    // e2: 1*4 = 4
    // e12: 2*4 = 8
    let expected_data = [3.0, 6.0, 4.0, 8.0];

    for (i, &expected_val) in expected_data.iter().enumerate() {
        assert!((result.data()[i] - expected_val).abs() < EPSILON);
    }
}

// ============================================================================
// Tier 4: The Generalized Algebra (Non-Commutative Coefficients) - using Complex<f64>
// Requirements: AssociativeRing (No Commutativity guaranteed)
// ============================================================================

#[test]
fn test_geometric_product_general_scalars_complex() {
    let metric = Metric::Euclidean(0);
    let mv1 = CausalMultiVector::new(vec![Complex::new(2.0, 1.0)], metric).unwrap(); // 2+i
    let mv2 = CausalMultiVector::new(vec![Complex::new(3.0, 2.0)], metric).unwrap(); // 3+2i
    // (2+i) * (3+2i) = 6 + 4i + 3i - 2 = 4 + 7i
    let expected = Complex::new(4.0, 7.0);

    let result = mv1.geometric_product_general(&mv2);
    assert!((result.data()[0].re - expected.re).abs() < EPSILON);
    assert!((result.data()[0].im - expected.im).abs() < EPSILON);
}

#[test]
fn test_geometric_product_general_e1_e2_euclidean_complex() {
    let metric = Metric::Euclidean(2); // e1^2=1, e2^2=1
    let mv_e1 = CausalMultiVector::new(
        vec![
            Complex::zero(),
            Complex::new(1.0, 0.0),
            Complex::zero(),
            Complex::zero(),
        ],
        metric,
    )
    .unwrap(); // e1
    let mv_e2 = CausalMultiVector::new(
        vec![
            Complex::zero(),
            Complex::zero(),
            Complex::new(1.0, 0.0),
            Complex::zero(),
        ],
        metric,
    )
    .unwrap(); // e2
    // e1 * e2 = e12 (coefficients are 1.0, so same as regular GP)
    let result = mv_e1.geometric_product_general(&mv_e2);
    assert!((result.data()[3].re - 1.0).abs() < EPSILON); // e12 component
    assert!((result.data()[3].im).abs() < EPSILON);
    assert_eq!(
        result
            .data()
            .iter()
            .filter(|&&x| x != Complex::zero())
            .count(),
        1
    );
}

#[test]
fn test_geometric_product_general_complex_case_complex_coefficients() {
    let metric = Metric::Euclidean(2);
    // (C1 + C2 * e1) * (C3 + C4 * e2)
    // C1=1+i, C2=2-i, C3=3+2i, C4=1-2i
    let c1 = Complex::new(1.0, 1.0);
    let c2 = Complex::new(2.0, -1.0);
    let c3 = Complex::new(3.0, 2.0);
    let c4 = Complex::new(1.0, -2.0);

    let mv1 =
        CausalMultiVector::new(vec![c1, c2, Complex::zero(), Complex::zero()], metric).unwrap(); // (1+i) + (2-i)e1
    let mv2 =
        CausalMultiVector::new(vec![c3, Complex::zero(), c4, Complex::zero()], metric).unwrap(); // (3+2i) + (1-2i)e2

    let result = mv1.geometric_product_general(&mv2);

    // Scalar part: C1*C3 = (1+i)*(3+2i) = 3+2i+3i-2 = 1+5i
    let expected_scalar = Complex::new(1.0, 5.0);
    assert!((result.data()[0].re - expected_scalar.re).abs() < EPSILON);
    assert!((result.data()[0].im - expected_scalar.im).abs() < EPSILON);

    // e1 part: C2*C3 = (2-i)*(3+2i) = 6+4i-3i+2 = 8+i
    let expected_e1 = Complex::new(8.0, 1.0);
    assert!((result.data()[1].re - expected_e1.re).abs() < EPSILON);
    assert!((result.data()[1].im - expected_e1.im).abs() < EPSILON);

    // e2 part: C1*C4 = (1+i)*(1-2i) = 1-2i+i+2 = 3-i
    let expected_e2 = Complex::new(3.0, -1.0);
    assert!((result.data()[2].re - expected_e2.re).abs() < EPSILON);
    assert!((result.data()[2].im - expected_e2.im).abs() < EPSILON);

    // e12 part: C2*C4 = (2-i)*(1-2i) = 2-4i-i-2 = -5i
    let expected_e12 = Complex::new(0.0, -5.0);
    assert!((result.data()[3].re - expected_e12.re).abs() < EPSILON);
    assert!((result.data()[3].im - expected_e12.im).abs() < EPSILON);
}

#[test]
#[should_panic(expected = "Metric mismatch")]
fn test_geometric_product_general_metric_mismatch_panics() {
    let metric1 = Metric::Euclidean(2);
    let metric2 = Metric::Euclidean(3);
    let mv1 = CausalMultiVector::new(vec![Complex::new(1.0, 0.0); 4], metric1).unwrap();
    let mv2 = CausalMultiVector::new(vec![Complex::new(1.0, 0.0); 8], metric2).unwrap();

    mv1.geometric_product_general(&mv2);
}

// Additional test for geometric_product_general for sign > 0 and sign < 0
#[test]
fn test_geometric_product_general_sign_handling() {
    let metric = Metric::Euclidean(2); // e1^2=1, e2^2=1, e12^2 = -1
    let c = Complex::new(1.0, 1.0); // 1+i
    let mv_e1 = CausalMultiVector::new(
        vec![Complex::zero(), c, Complex::zero(), Complex::zero()],
        metric,
    )
    .unwrap(); // (1+i)e1
    let mv_e2 = CausalMultiVector::new(
        vec![Complex::zero(), Complex::zero(), c, Complex::zero()],
        metric,
    )
    .unwrap(); // (1+i)e2

    let result = mv_e1.geometric_product_general(&mv_e2);
    // Expected result: (1+i)e1 * (1+i)e2 = (1+i)*(1+i)e12 = (1+2i-1)e12 = 2ie12
    let expected_e12 = Complex::new(0.0, 2.0);

    assert!((result.data()[3].re - expected_e12.re).abs() < EPSILON);
    assert!((result.data()[3].im - expected_e12.im).abs() < EPSILON);

    // Now test a product that results in a negative sign in basis_product, like e2 * e1 = -e12
    let result_rev = mv_e2.geometric_product_general(&mv_e1);
    // Expected result: (1+i)e2 * (1+i)e1 = (1+i)*(1+i)(-e12) = -2ie12
    let expected_e12_rev = Complex::new(0.0, -2.0);

    assert!((result_rev.data()[3].re - expected_e12_rev.re).abs() < EPSILON);
    assert!((result_rev.data()[3].im - expected_e12_rev.im).abs() < EPSILON);
}
