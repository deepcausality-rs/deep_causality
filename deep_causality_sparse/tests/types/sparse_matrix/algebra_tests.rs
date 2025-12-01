use deep_causality_num::{One, Zero};
use deep_causality_sparse::CsrMatrix; // Removed AbelianGroup

#[test]
fn test_zero_matrix() {
    let z: CsrMatrix<f64> = CsrMatrix::zero(3, 3);
    assert_eq!(z.shape(), (3, 3));
    assert_eq!(z.values().len(), 0); // No non-zero elements
}

#[test]
fn test_additive_identity() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let z: CsrMatrix<f64> = CsrMatrix::zero(2, 2); // Added type annotation
    let b = &a + &z;
    assert_eq!(a, b); // A + 0 = A
}

#[test]
fn test_additive_inverse() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let neg_a = a.neg();
    let z = &a + &neg_a;
    // For sparse matrices, A + (-A) should result in an empty matrix (no non-zero elements)
    // with the same shape.
    assert_eq!(z.values().len(), 0);
    assert_eq!(z.shape(), a.shape());
}

#[test]
fn test_commutativity() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 2, &[(1, 1, 2.0)]).unwrap();
    let ab = &a + &b;
    let ba = &b + &a;
    assert_eq!(ab, ba); // A + B = B + A
}

#[test]
fn test_associativity() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 2, &[(1, 1, 2.0)]).unwrap();
    let c = CsrMatrix::from_triplets(2, 2, &[(0, 1, 3.0)]).unwrap();
    let ab_c = (&a + &b) + &c;
    let a_bc = &a + (&b + &c);
    assert_eq!(ab_c, a_bc); // (A + B) + C = A + (B + C)
}

#[test]
fn test_scalar_multiplication() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let b = a.scale(3.0);
    assert_eq!(b.get_value_at(0, 0), 3.0);
    assert_eq!(b.get_value_at(1, 1), 6.0);
}

#[test]
fn test_matrix_multiplication_identity() {
    let i: CsrMatrix<f64> = CsrMatrix::one(3); // Added type annotation
    let a = CsrMatrix::from_triplets(3, 3, &[(0, 0, 1.0), (1, 1, 2.0), (2, 2, 3.0)]).unwrap();
    let b = &i * &a;
    assert_eq!(a, b); // I * A = A
}

#[test]
#[should_panic(expected = "shape mismatch")]
fn test_add_shape_mismatch() {
    let a: CsrMatrix<f64> = CsrMatrix::zero(2, 2); // Added type annotation
    let b: CsrMatrix<f64> = CsrMatrix::zero(3, 3); // Added type annotation
    let _c = a + b; // Should panic
}

#[test]
fn test_scalar_zero_trait() {
    let scalar_zero: CsrMatrix<f64> = Zero::zero();
    assert_eq!(scalar_zero.shape(), (0, 0));
    assert!(scalar_zero.is_zero());

    let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    assert!(!m.is_zero());
}

#[test]
fn test_scalar_one_trait() {
    let scalar_one: CsrMatrix<f64> = One::one();
    assert_eq!(scalar_one.shape(), (1, 1));
    assert!(scalar_one.is_one());

    let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    assert!(!m.is_one());
}
