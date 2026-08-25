/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The inherent surface ported from `deep_causality_sparse`, so that repointing a caller is a
//! change of `use` line and nothing else.
//!
//! Each test checks the answer, not merely that the call resolves: a wrapper that dispatches to the
//! wrong operation compiles and returns a matrix.

use deep_causality_linear::{CsrMatrix, LinearError, LinearErrorEnum, MatrixView};
use deep_causality_num::{One, Zero};

fn m(triplets: &[(usize, usize, f64)], r: usize, c: usize) -> CsrMatrix<f64> {
    CsrMatrix::from_triplets(r, c, triplets).unwrap()
}

// ---- shaped constructors, which the trait forms cannot express -----------------------------------

#[test]
fn test_the_shaped_zero_carries_its_shape_and_an_empty_structure() {
    let z: CsrMatrix<f64> = CsrMatrix::zero(3, 4);
    assert_eq!(z.shape(), (3, 4));
    assert!(z.values().is_empty(), "every entry is structural");
    assert_eq!(
        z.row_indices().len(),
        4,
        "the row pointer has rows + 1 entries even when nothing is stored"
    );
    for i in 0..3 {
        for j in 0..4 {
            assert_eq!(z.get(i, j).unwrap(), 0.0, "at ({i}, {j})");
        }
    }
}

#[test]
fn test_the_shaped_zero_differs_from_the_trait_zero() {
    // `Zero::zero` takes no shape and gives the 0x0 matrix; the inherent one takes a shape. Both
    // exist, and the inherent wins the bare name — which is the crate this replaces' behaviour.
    let shaped: CsrMatrix<f64> = CsrMatrix::zero(2, 2);
    let trait_zero: CsrMatrix<f64> = <CsrMatrix<f64> as Zero>::zero();
    assert_eq!(shaped.shape(), (2, 2));
    assert_eq!(trait_zero.shape(), (0, 0));
}

#[test]
fn test_the_sized_identity_is_the_identity() {
    let i3: CsrMatrix<f64> = CsrMatrix::one(3);
    assert_eq!(i3.shape(), (3, 3));
    for i in 0..3 {
        for j in 0..3 {
            let want = if i == j { 1.0 } else { 0.0 };
            assert_eq!(i3.get(i, j).unwrap(), want, "at ({i}, {j})");
        }
    }
    assert_eq!(i3.values().len(), 3, "only the diagonal is stored");
}

// ---- the &self arithmetic Rust's method probe needs ---------------------------------------------

#[test]
fn test_add_takes_both_operands_by_reference() {
    // The point of these existing: `a.add(&b)` on a borrowed field cannot resolve to `Add`, whose
    // receiver is by value.
    let a = m(&[(0, 0, 1.0), (1, 1, 2.0)], 2, 2);
    let b = m(&[(0, 0, 3.0), (0, 1, 5.0)], 2, 2);
    let s = a.add(&b);
    assert_eq!(s.get(0, 0).unwrap(), 4.0);
    assert_eq!(s.get(0, 1).unwrap(), 5.0);
    assert_eq!(s.get(1, 1).unwrap(), 2.0);
    // Both operands survive, which is what taking them by reference means.
    assert_eq!(a.get(0, 0).unwrap(), 1.0);
    assert_eq!(b.get(0, 0).unwrap(), 3.0);
}

#[test]
fn test_sub_subtracts_entrywise_and_drops_a_cancellation() {
    let a = m(&[(0, 0, 5.0), (1, 1, 2.0)], 2, 2);
    let b = m(&[(0, 0, 5.0), (0, 1, 1.0)], 2, 2);
    let d = a.sub(&b);
    assert_eq!(d.get(0, 0).unwrap(), 0.0);
    assert_eq!(d.get(0, 1).unwrap(), -1.0);
    assert_eq!(d.get(1, 1).unwrap(), 2.0);
    assert_eq!(
        d.values().len(),
        2,
        "the cancelled entry is structurally absent, not a stored zero"
    );
}

#[test]
fn test_neg_negates_every_stored_entry_and_keeps_the_pattern() {
    let a = m(&[(0, 0, 1.0), (1, 1, -2.0)], 2, 2);
    let n = a.neg();
    assert_eq!(n.get(0, 0).unwrap(), -1.0);
    assert_eq!(n.get(1, 1).unwrap(), 2.0);
    assert_eq!(n.values().len(), a.values().len());
    assert_eq!(n.col_indices(), a.col_indices());
    assert_eq!(n.row_indices(), a.row_indices());
}

#[test]
fn test_mul_is_the_matrix_product() {
    // [[1,2],[3,4]] * [[0,1],[1,0]] = [[2,1],[4,3]]
    let a = m(&[(0, 0, 1.0), (0, 1, 2.0), (1, 0, 3.0), (1, 1, 4.0)], 2, 2);
    let b = m(&[(0, 1, 1.0), (1, 0, 1.0)], 2, 2);
    let p = a.mul(&b);
    assert_eq!(p.get(0, 0).unwrap(), 2.0);
    assert_eq!(p.get(0, 1).unwrap(), 1.0);
    assert_eq!(p.get(1, 0).unwrap(), 4.0);
    assert_eq!(p.get(1, 1).unwrap(), 3.0);
}

#[test]
fn test_scale_multiplies_every_stored_entry() {
    let a = m(&[(0, 0, 1.0), (1, 1, -2.0)], 2, 2);
    let s = a.scale(3.0);
    assert_eq!(s.get(0, 0).unwrap(), 3.0);
    assert_eq!(s.get(1, 1).unwrap(), -6.0);
    assert_eq!(s.shape(), (2, 2));
}

// ---- the fallible forms -------------------------------------------------------------------------

#[test]
fn test_sub_matrix_reports_a_shape_mismatch_where_sub_would_panic() {
    let a = m(&[(0, 0, 1.0)], 2, 2);
    let b = m(&[(0, 0, 1.0)], 3, 3);
    assert!(matches!(
        a.sub_matrix(&b),
        Err(LinearError(LinearErrorEnum::ShapeMismatch {
            left: (2, 2),
            right: (3, 3)
        }))
    ));
}

#[test]
#[should_panic(expected = "shape mismatch")]
fn test_sub_panics_on_a_shape_mismatch() {
    let a = m(&[(0, 0, 1.0)], 2, 2);
    let b = m(&[(0, 0, 1.0)], 3, 3);
    let _ = a.sub(&b);
}

#[test]
#[should_panic(expected = "shape mismatch")]
fn test_add_panics_on_a_shape_mismatch() {
    let a = m(&[(0, 0, 1.0)], 2, 2);
    let b = m(&[(0, 0, 1.0)], 3, 3);
    let _ = a.add(&b);
}

// ---- the contextual zero ------------------------------------------------------------------------

#[test]
fn test_a_contextual_zero_decides_what_is_stored() {
    // With `zero = 1.0`, the entries equal to 1.0 are the absent ones and 0.0 is a stored value.
    // That inverts what `from_triplets` would keep, which is the whole point of the variant.
    let with_one =
        CsrMatrix::from_triplets_with_zero(1, 3, &[(0, 0, 1.0), (0, 1, 2.0), (0, 2, 0.0)], 1.0)
            .unwrap();
    assert_eq!(
        with_one.values().len(),
        2,
        "1.0 is the absent value here, so only 2.0 and 0.0 are stored"
    );
    assert_eq!(with_one.value_at_or(0, 0, 1.0), 1.0);
    assert_eq!(with_one.value_at_or(0, 1, 1.0), 2.0);
    assert_eq!(with_one.value_at_or(0, 2, 1.0), 0.0);

    // The plain constructor on the same triplets keeps a different set.
    let plain = m(&[(0, 0, 1.0), (0, 1, 2.0), (0, 2, 0.0)], 1, 3);
    assert_eq!(plain.values().len(), 2, "0.0 is the absent value there");
}

#[test]
fn test_a_contextual_zero_is_rejected_outside_the_shape() {
    assert!(matches!(
        CsrMatrix::from_triplets_with_zero(1, 2, &[(0, 5, 3.0)], 1.0),
        Err(LinearError(LinearErrorEnum::IndexOutOfBounds {
            index: (0, 5),
            shape: (1, 2)
        }))
    ));
}

#[test]
fn test_add_with_zero_merges_the_stored_entries_only() {
    // The case topology's `Chain::add_with_zero` is built on, and the one that pins what the
    // contextual zero does *not* do.
    //
    // `zero = 1.0` makes 1.0 the absent value, so building `a` drops its (0, 0, 1.0) and `b` drops
    // its only entry entirely. The sum is a merge of what remains stored -- it does not visit the
    // positions neither operand stores and add their absent values together. Those stay absent.
    //
    // Checked against `deep_causality_sparse`, whose merge this is: both give `values = [2.0]` at
    // `col_indices = [1]`. An implementation that walked every position would give three entries
    // and diverge silently.
    let a = CsrMatrix::from_triplets_with_zero(1, 3, &[(0, 0, 1.0), (0, 1, 2.0)], 1.0).unwrap();
    let b = CsrMatrix::from_triplets_with_zero(1, 3, &[(0, 1, 1.0)], 1.0).unwrap();
    assert_eq!(
        a.values(),
        &vec![2.0],
        "1.0 is absent, so only 2.0 is stored"
    );
    assert!(b.values().is_empty(), "b's only entry was the absent value");

    let s = a.add_with_zero(&b, 1.0).unwrap();
    assert_eq!(s.values(), &vec![2.0]);
    assert_eq!(s.col_indices(), &vec![1]);
    assert_eq!(s.row_indices(), &vec![0, 1]);
}

#[test]
fn test_add_with_zero_drops_a_sum_that_lands_on_the_absent_value() {
    // Two stored entries whose sum is the contextual zero: the position becomes absent, exactly as
    // a sum cancelling to `T::zero()` does in the plain path.
    let a = CsrMatrix::from_triplets_with_zero(1, 2, &[(0, 0, 3.0), (0, 1, 5.0)], 1.0).unwrap();
    let b = CsrMatrix::from_triplets_with_zero(1, 2, &[(0, 0, -2.0)], 1.0).unwrap();
    let s = a.add_with_zero(&b, 1.0).unwrap();
    assert_eq!(
        s.values(),
        &vec![5.0],
        "3 + (-2) = 1, which is the absent value"
    );
    assert_eq!(s.col_indices(), &vec![1]);
}

#[test]
fn test_add_matches_the_merge_on_the_plain_zero() {
    // The ordinary path is the same merge with `T::zero()`, so a shared column sums and a
    // cancellation drops.
    let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 4.0), (1, 1, 7.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 3, &[(0, 0, 2.0), (0, 2, -4.0)]).unwrap();
    let s = a.add(&b);
    assert_eq!(s.get(0, 0).unwrap(), 3.0);
    assert_eq!(s.get(0, 2).unwrap(), 0.0, "cancelled");
    assert_eq!(s.get(1, 1).unwrap(), 7.0);
    assert_eq!(s.values().len(), 2, "the cancellation is not stored");
    assert_eq!(
        s.row_indices(),
        &vec![0, 1, 2],
        "the row pointer tracks the drop"
    );
}

#[test]
fn test_add_with_zero_reports_a_shape_mismatch() {
    let a = m(&[(0, 0, 1.0)], 2, 2);
    let b = m(&[(0, 0, 1.0)], 3, 3);
    assert!(matches!(
        a.add_with_zero(&b, 0.0),
        Err(LinearError(LinearErrorEnum::ShapeMismatch {
            left: (2, 2),
            right: (3, 3)
        }))
    ));
}

#[test]
fn test_value_at_or_returns_the_named_absent_value() {
    let a = m(&[(0, 0, 7.0)], 2, 2);
    assert_eq!(a.value_at_or(0, 0, -1.0), 7.0, "a stored entry is itself");
    assert_eq!(
        a.value_at_or(1, 1, -1.0),
        -1.0,
        "an absent one is the named value"
    );
    assert_eq!(
        a.value_at_or(9, 9, -1.0),
        -1.0,
        "and so is one outside the shape"
    );
}

#[test]
fn test_one_and_the_trait_identity_agree_at_size_one() {
    let inherent: CsrMatrix<f64> = CsrMatrix::one(1);
    let from_trait: CsrMatrix<f64> = <CsrMatrix<f64> as One>::one();
    assert_eq!(inherent.shape(), from_trait.shape());
    assert_eq!(inherent.get(0, 0).unwrap(), from_trait.get(0, 0).unwrap());
}
