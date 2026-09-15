/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Foldable` for `ZipDenseVectorWitness`.
//!
//! # What this instance is actually for
//!
//! It is **not** for reaching the elements. `ZipDenseVectorWitness` and `DenseVectorWitness`
//! project to the same `DenseVector<T>`, so `DenseVectorWitness::fold` already accepts the output
//! of `ZipDenseVectorWitness::zip_with` with no conversion of any kind —
//! `test_plain_witness_already_folds_a_zipped_vector` pins that, so nobody adds this instance
//! twice for a reason that was never true.
//!
//! It is for **generic code bounded on one witness**. A function written as
//! `W: Semigroupal<W> + Foldable<W>` cannot be instantiated at the zip witness without it, and
//! the workaround — two witness parameters plus an unexpressible `W::Type<T> == F::Type<T>`
//! constraint — is not writable in Rust. `test_generic_over_one_witness_zips_then_folds` is that
//! case, and it is the test that does not compile before this instance exists.
//!
//! The fold itself must agree with the plain witness element for element and in order. A vector
//! of equal values, or a commutative fold, would hide a reversal; every case below uses distinct
//! values and a non-commutative operation where order is the thing under test.

use deep_causality_haft::{Foldable, Functor, HKT, Semigroupal};
use deep_causality_linear::{DenseVector, DenseVectorWitness, ZipDenseVectorWitness as Zip};

fn dv(xs: &[i64]) -> DenseVector<i64> {
    DenseVector::from_vec(xs.to_vec())
}

// ---------------------------------------------------------------------------
// The reason the instance exists
// ---------------------------------------------------------------------------

/// Zips two structures through one witness and reduces the result through the same one.
///
/// This is the whole point. Without `Foldable<ZipDenseVectorWitness>` the bound cannot be
/// satisfied and this function cannot be instantiated at the zip witness at all.
fn zip_then_fold<W>(a: W::Type<i64>, b: W::Type<i64>) -> i64
where
    W: HKT + Semigroupal<W> + Foldable<W>,
{
    W::fold(W::zip_with(a, b, |x, y| x * y), 0, |acc, x| acc + x)
}

#[test]
fn test_generic_over_one_witness_zips_then_folds() {
    // 1*10 + 2*20 + 3*30 + 4*40 = 10 + 40 + 90 + 160 = 300
    let got = zip_then_fold::<Zip>(dv(&[1, 2, 3, 4]), dv(&[10, 20, 30, 40]));
    assert_eq!(got, 300);
}

#[test]
fn test_plain_witness_already_folds_a_zipped_vector() {
    // The negative control for the claim above: this compiles without the new instance, because
    // both witnesses project to `DenseVector<T>`. So the instance buys the *bound*, not access to
    // the elements, and a note claiming otherwise is wrong.
    let zipped = Zip::zip_with(dv(&[1, 2, 3, 4]), dv(&[10, 20, 30, 40]), |x, y| x * y);
    assert_eq!(DenseVectorWitness::fold(zipped, 0, |acc, x| acc + x), 300);
}

// ---------------------------------------------------------------------------
// The fold itself
// ---------------------------------------------------------------------------

#[test]
fn test_fold_visits_left_to_right() {
    // Subtraction is non-commutative and non-associative: ((0-1)-2)-3-4 = -10, while a
    // right-to-left visit gives ((0-4)-3)-2-1 = -10 as well, so subtraction alone is not enough.
    // Division of the accumulator by each element separates them: left-to-right only.
    assert_eq!(Zip::fold(dv(&[1, 2, 3, 4]), 0, |acc, x| acc - x), -10);
    assert_eq!(
        Zip::fold(dv(&[1, 2, 3, 4]), 0, |acc, x| acc * 10 + x),
        1234,
        "digit accumulation pins the visit order; a reversal yields 4321"
    );
}

#[test]
fn test_fold_agrees_with_the_plain_witness_element_for_element() {
    // The two witnesses differ only in what `apply` means. `fold` involves no applicative, so they
    // must agree exactly — including on order, which the digit accumulator pins.
    for xs in [
        vec![],
        vec![7],
        vec![1, 2, 3, 4],
        vec![9, 0, 5, 3, 8, 2],
        vec![-3, 11, -7],
    ] {
        let via_zip = Zip::fold(dv(&xs), 0, |acc, x| acc * 10 + x);
        let via_plain = DenseVectorWitness::fold(dv(&xs), 0, |acc, x| acc * 10 + x);
        assert_eq!(via_zip, via_plain, "witnesses disagreed on {xs:?}");
    }
}

#[test]
fn test_fold_on_an_empty_vector_returns_the_seed() {
    // The seed must come back untouched rather than a default: 42 is chosen so a `0` default shows.
    assert_eq!(Zip::fold(dv(&[]), 42, |acc, x| acc + x), 42);
}

#[test]
fn test_fold_changes_the_accumulator_type() {
    // `fold` is generic in `B`; the accumulator need not be the element type.
    let joined = Zip::fold(dv(&[1, 2, 3]), String::new(), |mut acc, x| {
        acc.push_str(&x.to_string());
        acc
    });
    assert_eq!(joined, "123");
}

// ---------------------------------------------------------------------------
// Interaction with the zip's own truncation
// ---------------------------------------------------------------------------

#[test]
fn test_fold_sees_only_what_the_zip_left() {
    // `zip_with` truncates to the shorter side. The fold must run over the survivors and no
    // further — a fold that walked the longer operand's length would read past the data.
    let zipped = Zip::zip_with(dv(&[1, 2, 3, 4, 5]), dv(&[10, 20]), |x, y| x * y);
    assert_eq!(zipped.len(), 2, "zip_with should truncate to the shorter");
    assert_eq!(
        Zip::fold(zipped, 0, |acc, x| acc * 1000 + x),
        10 * 1000 + 40
    );
}

#[test]
fn test_fold_after_fmap_composes() {
    // `Functor` then `Foldable` on the same witness: the fold sees the mapped values, in order.
    let mapped = Zip::fmap(dv(&[1, 2, 3]), |x| x * 2);
    assert_eq!(Zip::fold(mapped, 0, |acc, x| acc * 10 + x), 246);
}
