/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{One, Zero};
use deep_causality_sparse::CsrMatrix;

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

#[test]
fn test_from_triplets_with_zero() {
    // Use 1.0 as "zero" - elements with value 1.0 should be excluded
    let triplets = vec![
        (0, 0, 1.0), // Should be excluded
        (0, 1, 2.0), // Should be included
        (1, 0, 3.0), // Should be included
        (1, 1, 1.0), // Should be excluded
    ];

    let matrix = CsrMatrix::from_triplets_with_zero(2, 2, &triplets, 1.0).unwrap();

    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.values(), &vec![2.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![1, 0]);
    // Row indices:
    // Row 0: 1 element (2.0 at col 1) -> starts at 0, ends at 1
    // Row 1: 1 element (3.0 at col 0) -> starts at 1, ends at 2
    assert_eq!(matrix.row_indices(), &vec![0, 1, 2]);
}

#[test]
fn test_add_with_zero() {
    // Use 0.0 as zero for standard addition check
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 2, &[(0, 0, 2.0), (1, 1, 3.0)]).unwrap();

    // 1.0 + 2.0 = 3.0
    // 2.0 + 3.0 = 5.0
    let c = a.add_with_zero(&b, 0.0).unwrap();

    assert_eq!(c.values(), &vec![3.0, 5.0]);

    // Test cancellation with explicit zero
    // A = [1.0, 0.0]
    // B = [-1.0, 0.0]
    // Sum = [0.0, 0.0] -> Empty if zero is 0.0
    let a2 = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b2 = CsrMatrix::from_triplets(2, 2, &[(0, 0, -1.0)]).unwrap();
    let c2 = a2.add_with_zero(&b2, 0.0).unwrap();
    assert!(c2.values().is_empty());

    // Test with non-standard zero
    // Treat 3.0 as zero.
    // A = [1.0, 2.0]
    // B = [2.0, 1.0]
    // Sum = [3.0, 3.0] -> Both should be filtered out if zero is 3.0
    let a3 = CsrMatrix::from_triplets_with_zero(1, 2, &[(0, 0, 1.0), (0, 1, 2.0)], 0.0).unwrap();
    let b3 = CsrMatrix::from_triplets_with_zero(1, 2, &[(0, 0, 2.0), (0, 1, 1.0)], 0.0).unwrap();
    let c3 = a3.add_with_zero(&b3, 3.0).unwrap();
    assert!(c3.values().is_empty());
}

#[test]
fn test_module_trait() {
    let matrix = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();

    // Scale by 2.0
    let scaled = matrix.scale(2.0);

    assert_eq!(scaled.values(), &vec![2.0, 4.0]);

    // Verify it's a new matrix
    assert_ne!(matrix.values(), scaled.values());
}

// ---------------------------------------------------------------------------
// Where CsrMatrix sits in the algebra tower.
//
// Each admission witness compiles only if the type reaches the bound, so calling it is the
// assertion. `Ring` is the rung the two marker impls `Distributive` and `Annihilating` closed;
// `CommutativeRing` must stay refused, because matrix multiplication does not commute.
// ---------------------------------------------------------------------------

mod tower {
    use deep_causality_algebra::{
        AbelianGroup, Additive, Annihilating, Associative, Distributive, Module, Multiplicative,
        Ring,
    };
    use deep_causality_sparse::CsrMatrix;

    fn admits_abelian_group<T: AbelianGroup>() {}
    fn admits_ring<T: Ring>() {}
    fn admits_module<M: Module<R>, R: Ring>() {}
    fn admits_distributive<T: Distributive>() {}
    fn admits_annihilating<T: Annihilating>() {}
    fn admits_associative_add<T: Associative<Additive>>() {}
    fn admits_associative_mul<T: Associative<Multiplicative>>() {}

    #[test]
    fn test_csr_matrix_carries_the_markers_that_reach_ring() {
        admits_associative_add::<CsrMatrix<f64>>();
        admits_associative_mul::<CsrMatrix<f64>>();
        admits_distributive::<CsrMatrix<f64>>();
        admits_annihilating::<CsrMatrix<f64>>();
    }

    #[test]
    fn test_csr_matrix_is_an_abelian_group() {
        admits_abelian_group::<CsrMatrix<f64>>();
        admits_abelian_group::<CsrMatrix<i64>>();
    }

    #[test]
    fn test_csr_matrix_reaches_the_ring_rung() {
        admits_ring::<CsrMatrix<f64>>();
        admits_ring::<CsrMatrix<i64>>();
    }

    #[test]
    fn test_csr_matrix_is_a_module_over_its_scalar_ring() {
        // Reached through the blanket at `algebra/module.rs:65`, which needs only `AbelianGroup`
        // and the scalar multiplication -- not `Ring`.
        admits_module::<CsrMatrix<f64>, f64>();
        admits_module::<CsrMatrix<i64>, i64>();
    }

    #[test]
    fn test_the_module_scaling_is_the_operation_the_tower_names() {
        let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 2.0), (1, 1, 3.0)]).unwrap();
        let scaled = m.clone() * 2.0_f64;
        assert_eq!(scaled.get_value_at(0, 0), 4.0);
        assert_eq!(scaled.get_value_at(1, 1), 6.0);
    }

    #[test]
    fn test_ring_distributivity_holds_on_a_worked_example() {
        // The law `Distributive` promises: A(B + C) = AB + AC.
        let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 1, 2.0), (1, 1, 3.0)]).unwrap();
        let b = CsrMatrix::from_triplets(2, 2, &[(0, 0, 4.0), (1, 0, 5.0)]).unwrap();
        let c = CsrMatrix::from_triplets(2, 2, &[(0, 1, 6.0), (1, 1, 7.0)]).unwrap();

        let lhs = a.mul(&(b.clone() + c.clone()));
        let rhs = a.mul(&b) + a.mul(&c);

        for i in 0..2 {
            for j in 0..2 {
                assert_eq!(
                    lhs.get_value_at(i, j),
                    rhs.get_value_at(i, j),
                    "mismatch at ({i}, {j})"
                );
            }
        }
    }

    #[test]
    fn test_ring_annihilation_holds_on_a_worked_example() {
        // The law `Annihilating` promises: 0 * A = 0.
        let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
        let zero = CsrMatrix::<f64>::zero(2, 2);
        let product = zero.mul(&a);
        for i in 0..2 {
            for j in 0..2 {
                assert_eq!(product.get_value_at(i, j), 0.0);
            }
        }
    }

    #[test]
    fn test_matrix_multiplication_does_not_commute() {
        // Why `Commutative<Multiplicative>` is deliberately absent, and why a `CommutativeRing`
        // bound must keep refusing this type.
        let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 1, 1.0)]).unwrap();
        let b = CsrMatrix::from_triplets(2, 2, &[(1, 0, 1.0)]).unwrap();
        let ab = a.mul(&b);
        let ba = b.mul(&a);
        assert_ne!(ab.get_value_at(0, 0), ba.get_value_at(0, 0));
    }
}
