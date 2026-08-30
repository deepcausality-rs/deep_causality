/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `core::ops` operators on `CsrMatrix`, ported from `deep_causality_sparse`.
//!
//! The source file covered every ownership form of each operator: owned, borrowed, and the two
//! mixed forms, plus `AddAssign`, `SubAssign` and `MulAssign<Self>`. `deep_causality_linear`
//! implements the owned form of `Add`, `Sub`, `Neg` and `Mul`, the owned scalar `Mul<S>`, and
//! `MulAssign<S>` for a scalar. The forms this crate carries are exercised below; the ones it does
//! not carry are named in the port's skip list rather than approximated here.
//!
//! Where a skipped test asserted values no surviving test covered, those assertions were kept and
//! restated over the owned operator.

use deep_causality_linear::CsrMatrix;

/// The 2x2 diagonal `[1 0; 0 2]`.
fn a() -> CsrMatrix<f64> {
    CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap()
}

/// The 2x2 anti-diagonal `[0 3; 4 0]`, whose pattern is disjoint from `a`'s.
fn b() -> CsrMatrix<f64> {
    CsrMatrix::from_triplets(2, 2, &[(0, 1, 3.0), (1, 0, 4.0)]).unwrap()
}

/// A 2x3 matrix, `[1 0 2; 0 3 0]`.
fn mm_a() -> CsrMatrix<f64> {
    CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap()
}

/// A 3x2 matrix, `[4 0; 0 5; 6 0]`.
fn mm_b() -> CsrMatrix<f64> {
    CsrMatrix::from_triplets(3, 2, &[(0, 0, 4.0), (1, 1, 5.0), (2, 0, 6.0)]).unwrap()
}

// ----- Add -----

#[test]
fn test_add_unions_two_disjoint_patterns_entrywise() {
    let c = a() + b();
    assert_eq!(c.get_value_at(0, 0), 1.0);
    assert_eq!(c.get_value_at(0, 1), 3.0);
    assert_eq!(c.get_value_at(1, 0), 4.0);
    assert_eq!(c.get_value_at(1, 1), 2.0);
    assert_eq!(c.shape(), (2, 2));
}

// ----- Sub -----

#[test]
fn test_sub_of_a_matrix_with_itself_stores_nothing() {
    let c = a() - a();
    assert!(
        c.values().is_empty(),
        "a cancelled entry leaves the stored pattern, it does not stay as a zero"
    );
}

#[test]
fn test_sub_subtracts_entrywise() {
    let c = a() - b();
    assert_eq!(c.get_value_at(0, 0), 1.0);
    assert_eq!(c.get_value_at(0, 1), -3.0);
    assert_eq!(c.get_value_at(1, 0), -4.0);
    assert_eq!(c.get_value_at(1, 1), 2.0);
}

// ----- Neg -----

#[test]
fn test_neg_flips_the_sign_of_every_stored_entry() {
    let c = -a();
    assert_eq!(c.get_value_at(0, 0), -1.0);
    assert_eq!(c.get_value_at(1, 1), -2.0);
}

// ----- Matrix Mul -----

#[test]
fn test_mul_is_the_matrix_product_and_takes_the_outer_shape() {
    // [1 0 2; 0 3 0] * [4 0; 0 5; 6 0] = [16 0; 0 15]
    let c = mm_a() * mm_b();
    assert_eq!(c.get_value_at(0, 0), 16.0);
    assert_eq!(c.get_value_at(1, 1), 15.0);
    assert_eq!(c.shape(), (2, 2));
}

// ----- Scalar Mul -----

#[test]
fn test_scalar_mul_scales_every_stored_entry() {
    let c = a() * 3.0f64;
    assert_eq!(c.get_value_at(0, 0), 3.0);
    assert_eq!(c.get_value_at(1, 1), 6.0);
}

#[test]
fn test_scalar_mul_assign_scales_in_place() {
    let mut x = a();
    x *= 4.0f64;
    assert_eq!(x.get_value_at(0, 0), 4.0);
    assert_eq!(x.get_value_at(1, 1), 8.0);
}

// ---- restored: the borrowing and assigning forms ------------------------------------------------
//
// The port skipped sixteen tests because only the owned operator forms existed. The borrowing and
// assigning forms are now present, so these come back. Phase 5 repoints 102 import sites, and
// `&a + &b` is the common shape at a call site that consumes neither operand.

#[cfg(test)]
mod restored_reference_forms {
    use deep_causality_linear::CsrMatrix;

    fn a() -> CsrMatrix<f64> {
        CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap()
    }
    fn b() -> CsrMatrix<f64> {
        CsrMatrix::from_triplets(2, 2, &[(0, 0, 10.0), (0, 1, 5.0)]).unwrap()
    }

    #[test]
    fn test_every_add_combination_agrees() {
        let expected = a().add_matrix(&b()).unwrap();
        assert_eq!(&a() + &b(), expected);
        assert_eq!(a() + &b(), expected);
        assert_eq!(&a() + b(), expected);
        assert_eq!(a() + b(), expected);
    }

    #[test]
    fn test_add_by_reference_leaves_both_operands_usable() {
        let (x, y) = (a(), b());
        let _ = &x + &y;
        // Neither was consumed.
        assert_eq!(x.get_value_at(0, 0), 1.0);
        assert_eq!(y.get_value_at(0, 0), 10.0);
    }

    #[test]
    fn test_every_sub_combination_agrees() {
        let expected = &b() - &a();
        assert_eq!(b() - &a(), expected);
        assert_eq!(&b() - a(), expected);
        assert_eq!(b() - a(), expected);
        assert_eq!(expected.get_value_at(0, 0), 9.0);
    }

    #[test]
    fn test_add_assign_owned_and_borrowed() {
        let mut x = a();
        x += b();
        assert_eq!(x.get_value_at(0, 0), 11.0);
        let mut y = a();
        y += &b();
        assert_eq!(y.get_value_at(0, 0), 11.0);
    }

    #[test]
    fn test_sub_assign_owned_and_borrowed() {
        let mut x = b();
        x -= a();
        assert_eq!(x.get_value_at(0, 0), 9.0);
        let mut y = b();
        y -= &a();
        assert_eq!(y.get_value_at(0, 0), 9.0);
    }

    #[test]
    fn test_neg_by_reference() {
        let x = a();
        let n = -&x;
        assert_eq!(n.get_value_at(1, 1), -2.0);
        assert_eq!(x.get_value_at(1, 1), 2.0, "the operand survives");
    }

    #[test]
    fn test_mul_by_reference() {
        let i = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 1.0)]).unwrap();
        let p = &a() * &i;
        assert_eq!(p.get_value_at(0, 0), 1.0);
        assert_eq!(p.get_value_at(1, 1), 2.0);
    }
}
