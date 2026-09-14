/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The diagonal traversal on [`CausalTensorWitness`].
//!
//! What separates this from `Traversable::sequence` is which pairing it uses, and a count alone
//! cannot see the difference between "paired by index" and "paired by index, off by one". Every
//! assertion here is on values.

use deep_causality_haft::{DiagonalTraversable, Traversable};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness, ZipTensorWitness};

const DRAWS: usize = 50;

/// A 2×2 field whose cell `k` holds the run `100k .. 100k + DRAWS`.
///
/// The stride of 100 makes every value say which cell and which position it came from, so a
/// mispairing is legible in the assertion message rather than merely failing.
fn field() -> CausalTensor<CausalTensor<i32>> {
    let cells: Vec<CausalTensor<i32>> = (0..4)
        .map(|k| {
            CausalTensor::from_vec((0..DRAWS).map(|i| (k * 100 + i) as i32).collect(), &[DRAWS])
        })
        .collect();
    CausalTensor::from_vec(cells, &[2, 2])
}

/// An ensemble of `n` empty fields: the accumulator the traversal builds into.
fn seed(n: usize) -> CausalTensor<CausalTensor<i32>> {
    CausalTensor::from_vec(
        (0..n)
            .map(|_| CausalTensor::from_vec(Vec::new(), &[0]))
            .collect(),
        &[n],
    )
}

#[test]
fn the_diagonal_pairs_index_with_index() {
    let out = CausalTensorWitness::sequence_zip::<i32, ZipTensorWitness>(field(), seed(DRAWS));

    assert_eq!(out.shape(), &[DRAWS], "one field per draw");
    for (i, f) in out.as_slice().iter().enumerate() {
        assert_eq!(f.shape(), &[2, 2], "field {i} lost the structure's shape");
        let want: Vec<i32> = (0..4).map(|k| (k * 100 + i) as i32).collect();
        assert_eq!(
            f.as_slice(),
            want.as_slice(),
            "field {i} does not hold draw {i} of every cell"
        );
    }
}

#[test]
fn the_cartesian_traversal_is_a_different_operation() {
    // Not a test of the diagonal so much as the measurement that motivates it: the same field
    // through `Traversable::sequence` takes the cartesian applicative and returns every
    // combination of the four cells.
    //
    // 50^4 is 6 250 000. Kept small here — 3^4 = 81 against the diagonal's 3 — because the point
    // is the shape of the answer, not how long it takes to build.
    let n = 3;
    let cells: Vec<CausalTensor<i32>> = (0..4)
        .map(|k| CausalTensor::from_vec((0..n).map(|i| (k * 100 + i) as i32).collect(), &[n]))
        .collect();
    let small = CausalTensor::from_vec(cells, &[2, 2]);

    let cartesian = CausalTensorWitness::sequence::<i32, CausalTensorWitness>(small.clone());
    assert_eq!(cartesian.len(), 81, "the cartesian traversal returns n^4");

    let diagonal = CausalTensorWitness::sequence_zip::<i32, ZipTensorWitness>(
        small,
        CausalTensor::from_vec(
            (0..n)
                .map(|_| CausalTensor::from_vec(Vec::new(), &[0]))
                .collect(),
            &[n],
        ),
    );
    assert_eq!(diagonal.len(), n, "the diagonal traversal returns n");
}

#[test]
fn an_empty_structure_returns_the_seed() {
    // With no cells there is nothing to zip, and the absence of a `Pure` means there is nothing to
    // derive an answer from either. The seed is the answer.
    let empty: CausalTensor<CausalTensor<i32>> = CausalTensor::from_vec(Vec::new(), &[0]);
    let out = CausalTensorWitness::sequence_zip::<i32, ZipTensorWitness>(empty, seed(7));
    assert_eq!(out.shape(), &[7], "the seed's length survived");
    assert!(
        out.as_slice().iter().all(|f| f.as_slice().is_empty()),
        "the seed's fields came back changed"
    );
}

#[test]
fn ragged_cells_truncate_the_ensemble_and_not_the_structure() {
    // Cell 1 holds 20 draws where the others hold 50. Twenty is what the ensemble can support, so
    // the result holds 20 fields — and each of those 20 still holds all four cells. Truncating the
    // other way would change what each field *is*; truncating this way only changes how many.
    let cells: Vec<CausalTensor<i32>> = (0..4)
        .map(|k| {
            let n = if k == 1 { 20 } else { DRAWS };
            CausalTensor::from_vec((0..n).map(|i| (k * 100 + i) as i32).collect(), &[n])
        })
        .collect();
    let ragged = CausalTensor::from_vec(cells, &[2, 2]);

    let out = CausalTensorWitness::sequence_zip::<i32, ZipTensorWitness>(ragged, seed(DRAWS));

    assert_eq!(
        out.shape(),
        &[20],
        "the ensemble truncated to the shortest cell"
    );
    for (i, f) in out.as_slice().iter().enumerate() {
        assert_eq!(
            f.shape(),
            &[2, 2],
            "field {i} lost a cell to the truncation"
        );
        let want: Vec<i32> = (0..4).map(|k| (k * 100 + i) as i32).collect();
        assert_eq!(f.as_slice(), want.as_slice(), "field {i} mispaired");
    }
}

#[test]
fn a_seed_shorter_than_the_cells_bounds_the_ensemble() {
    // The seed is the caller declaring how many they want. Asking for fewer than the cells can
    // supply gives fewer, with the same pairing.
    let out = CausalTensorWitness::sequence_zip::<i32, ZipTensorWitness>(field(), seed(5));
    assert_eq!(out.shape(), &[5]);
    for (i, f) in out.as_slice().iter().enumerate() {
        let want: Vec<i32> = (0..4).map(|k| (k * 100 + i) as i32).collect();
        assert_eq!(f.as_slice(), want.as_slice(), "field {i} mispaired");
    }
}

#[test]
fn a_single_cell_field_is_the_ensemble_itself() {
    // The degenerate structure: one cell, so each field holds one value and the result is the
    // original run wrapped one deep.
    let one = CausalTensor::from_vec(vec![CausalTensor::from_vec(vec![7, 8, 9], &[3])], &[1]);
    let out = CausalTensorWitness::sequence_zip::<i32, ZipTensorWitness>(
        one,
        CausalTensor::from_vec(
            (0..3)
                .map(|_| CausalTensor::from_vec(Vec::new(), &[0]))
                .collect(),
            &[3],
        ),
    );
    assert_eq!(out.len(), 3);
    for (i, want) in [7, 8, 9].iter().enumerate() {
        assert_eq!(out.as_slice()[i].as_slice(), &[*want]);
        assert_eq!(out.as_slice()[i].shape(), &[1]);
    }
}

// ---------------------------------------------------------------------------------------------
// Defect (a): the cartesian applicative in place of the diagonal
// ---------------------------------------------------------------------------------------------
//
// This defect cannot be written. `sequence` is bounded on `M: Applicative<M>`, and the zip
// witness has no `Pure` and so no `Applicative`, so substituting it for the diagonal does not
// compile:
//
// ```compile_fail
// CausalTensorWitness::sequence::<i32, ZipTensorWitness>(field());
// // error[E0277]: the trait bound `ZipTensorWitness: Applicative<ZipTensorWitness>`
// //               is not satisfied
// ```
//
// Going the other way — driving `sequence_zip` with the cartesian witness — fails for the mirror
// reason: `CausalTensorWitness` carries `Monad` and therefore the cartesian `apply`, but it
// implements no `Semigroupal`, which is `sequence_zip`'s bound.
//
// So the audit for (a) is not a test that fails. It is that the two operations cannot be
// confused: each traversal accepts exactly the witness whose pairing it means, and the compiler
// refuses the other. `the_cartesian_traversal_is_a_different_operation` above measures what the
// cartesian one does instead, which is the fact worth keeping — 3^4 against 3, and 50^4 against
// 50 at the size sampling actually uses.
