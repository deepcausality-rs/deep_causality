/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Foldable` for `ZipTensorWitness`.
//!
//! # What this instance is for
//!
//! Not for reaching the elements. `ZipTensorWitness` and `CausalTensorWitness` project to the same
//! `CausalTensor<T>`, so `CausalTensorWitness::fold` already accepts whatever `zip_with` returns,
//! with no conversion — `test_plain_witness_already_folds_a_zipped_tensor` pins that so the
//! instance is not added twice for a reason that was never true.
//!
//! It is for **generic code bounded on one witness**: `W: Semigroupal<W> + Foldable<W>`, which
//! zips and then reduces through the same parameter, cannot be instantiated at the zip witness
//! without it. The workaround needs two witness parameters plus a `W::Type<T> == F::Type<T>`
//! constraint Rust cannot express. `test_generic_over_one_witness_zips_then_folds` is that case,
//! and it is what does not compile before this instance exists.
//!
//! # What is tensor-specific
//!
//! A `CausalTensor` carries a shape, and `zip_with` keeps it when both operands agree and
//! flattens to `[len]` when they do not. `fold` reduces to a scalar over the flat data, so it is
//! insensitive to that distinction — which is a claim to pin rather than assume, because a fold
//! written against the shape rather than the data would pass every one-dimensional test.

use deep_causality_haft::{Foldable, Functor, HKT, Semigroupal};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness, ZipTensorWitness as Zip};

fn t(data: &[i64], shape: &[usize]) -> CausalTensor<i64> {
    CausalTensor::new(data.to_vec(), shape.to_vec()).expect("shape must match the data length")
}

// ---------------------------------------------------------------------------
// The reason the instance exists
// ---------------------------------------------------------------------------

/// Zips two structures through one witness and reduces the result through the same one.
fn zip_then_fold<W>(a: W::Type<i64>, b: W::Type<i64>) -> i64
where
    W: HKT + Semigroupal<W> + Foldable<W>,
{
    W::fold(W::zip_with(a, b, |x, y| x * y), 0, |acc, x| acc + x)
}

#[test]
fn test_generic_over_one_witness_zips_then_folds() {
    // 1*10 + 2*20 + 3*30 + 4*40 + 5*50 + 6*60 = 10+40+90+160+250+360 = 910
    let got = zip_then_fold::<Zip>(
        t(&[1, 2, 3, 4, 5, 6], &[2, 3]),
        t(&[10, 20, 30, 40, 50, 60], &[2, 3]),
    );
    assert_eq!(got, 910);
}

#[test]
fn test_plain_witness_already_folds_a_zipped_tensor() {
    // The negative control: this compiles without the new instance, because both witnesses project
    // to `CausalTensor<T>`. The instance buys the bound, not access to the elements.
    let zipped = Zip::zip_with(
        t(&[1, 2, 3, 4, 5, 6], &[2, 3]),
        t(&[10, 20, 30, 40, 50, 60], &[2, 3]),
        |x, y| x * y,
    );
    assert_eq!(CausalTensorWitness::fold(zipped, 0, |acc, x| acc + x), 910);
}

// ---------------------------------------------------------------------------
// The fold itself
// ---------------------------------------------------------------------------

#[test]
fn test_fold_visits_in_row_major_order() {
    // Digit accumulation over a 2x3: row-major gives 123456, any other visit order gives something
    // else. Plain subtraction would not separate a reversal here, so the base-10 accumulator is
    // the operative check.
    assert_eq!(
        Zip::fold(t(&[1, 2, 3, 4, 5, 6], &[2, 3]), 0, |acc, x| acc * 10 + x),
        123456
    );
}

#[test]
fn test_fold_agrees_with_the_plain_witness_across_shapes() {
    // The two witnesses differ only in what `apply` means, and `fold` involves no applicative, so
    // they must agree exactly — over rank 1, rank 2 and rank 3, and over the empty tensor.
    for (data, shape) in [
        (vec![], vec![0]),
        (vec![7], vec![1]),
        (vec![1, 2, 3, 4], vec![4]),
        (vec![1, 2, 3, 4, 5, 6], vec![2, 3]),
        (vec![1, 2, 3, 4, 5, 6, 7, 8], vec![2, 2, 2]),
    ] {
        let via_zip = Zip::fold(t(&data, &shape), 0, |acc, x| acc * 10 + x);
        let via_plain = CausalTensorWitness::fold(t(&data, &shape), 0, |acc, x| acc * 10 + x);
        assert_eq!(via_zip, via_plain, "witnesses disagreed on shape {shape:?}");
    }
}

#[test]
fn test_fold_ignores_the_shape_and_reads_the_data() {
    // The same six values as a flat [6], a [2,3] and a [3,2] must fold identically. A fold written
    // against the shape rather than the flat data passes every rank-1 test and fails here.
    let flat = Zip::fold(t(&[1, 2, 3, 4, 5, 6], &[6]), 0, |acc, x| acc * 10 + x);
    let two_by_three = Zip::fold(t(&[1, 2, 3, 4, 5, 6], &[2, 3]), 0, |acc, x| acc * 10 + x);
    let three_by_two = Zip::fold(t(&[1, 2, 3, 4, 5, 6], &[3, 2]), 0, |acc, x| acc * 10 + x);
    assert_eq!(flat, 123456);
    assert_eq!(two_by_three, 123456);
    assert_eq!(three_by_two, 123456);
}

#[test]
fn test_fold_on_an_empty_tensor_returns_the_seed() {
    // 42 rather than 0, so a default-returning implementation shows.
    assert_eq!(Zip::fold(t(&[], &[0]), 42, |acc, x| acc + x), 42);
}

#[test]
fn test_fold_changes_the_accumulator_type() {
    let joined = Zip::fold(t(&[1, 2, 3], &[3]), String::new(), |mut acc, x| {
        acc.push_str(&x.to_string());
        acc
    });
    assert_eq!(joined, "123");
}

// ---------------------------------------------------------------------------
// Interaction with the zip's shape rule
// ---------------------------------------------------------------------------

#[test]
fn test_fold_after_a_shape_preserving_zip() {
    // Equal shapes: `zip_with` keeps the shape, and the fold reads all of it in row-major order.
    let zipped = Zip::zip_with(
        t(&[1, 2, 3, 4], &[2, 2]),
        t(&[1, 1, 1, 1], &[2, 2]),
        |x, y| x + y,
    );
    assert_eq!(
        zipped.shape(),
        &[2, 2],
        "equal shapes should survive the zip"
    );
    assert_eq!(Zip::fold(zipped, 0, |acc, x| acc * 10 + x), 2345);
}

#[test]
fn test_fold_after_a_shape_dropping_zip_sees_only_the_overlap() {
    // Differing shapes: `zip_with` truncates to the shorter and reports a flat `[len]`. The fold
    // must run over the survivors and no further.
    let zipped = Zip::zip_with(t(&[1, 2, 3, 4, 5], &[5]), t(&[10, 20], &[2]), |x, y| x * y);
    assert_eq!(zipped.shape(), &[2], "the overlap is reported flat");
    assert_eq!(
        Zip::fold(zipped, 0, |acc, x| acc * 1000 + x),
        10 * 1000 + 40
    );
}

#[test]
fn test_fold_after_fmap_composes() {
    let mapped = Zip::fmap(t(&[1, 2, 3], &[3]), |x| x * 2);
    assert_eq!(Zip::fold(mapped, 0, |acc, x| acc * 10 + x), 246);
}
