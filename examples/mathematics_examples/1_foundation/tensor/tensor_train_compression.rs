/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `CausalTensorTrain`: a tensor stored as its factors
//!
//! A dense tensor of order `d` with `n` values per axis holds `n^d` entries, and that count is
//! what puts high-dimensional problems out of reach. A tensor train stores the same object as a
//! chain of order-3 cores, so the count falls to `d·n·r²` for a bond dimension `r`. Where the data
//! has low rank, `r` stays small and the saving is the whole difference between feasible and not.
//!
//! `CausalTensorTrainWitness` carries three traits, and what they act on is the point of this
//! example:
//!
//! ```text
//! HKT       Type<T> = CausalTensorTrain<T>
//! Functor   fmap    maps every entry of every core
//! Foldable  fold    folds every entry of every core
//! Pure      pure    the rank-1 boundary train holding one value
//! ```
//!
//! `fmap` and `fold` reach the **factors**, not the tensor the factors represent. Section 4
//! measures the gap: scaling every core entry by `s` scales the logical tensor by `s^d`, because
//! the tensor is the product of `d` cores and each one picks up the factor. That is the functor
//! behaving correctly on the container it is a functor over, and it is worth seeing once.
//!
//! The setting is a 4-dimensional lookup table built from two separable terms, which is exactly
//! the shape a tensor train compresses well.

use deep_causality_algebra::Real;
use deep_causality_haft::{Foldable, Functor, Pure};
use deep_causality_num::{lift, lift_count, lower};
use deep_causality_tensor::{
    CausalTensor, CausalTensorTrain, CausalTensorTrainWitness, TensorTrain, Truncation,
};

/// The lookup table is `ORDER` axes of `AXIS_LEN` values each.
const ORDER: usize = 4;
const AXIS_LEN: usize = 4;

/// The relative tolerance the TT-SVD truncates singular values at.
const REL_TOL: f64 = 1e-10;

/// The factor section 4 scales every core entry by.
const CORE_SCALE: f64 = 2.0;

/// What counts as zero when a reconstruction is checked.
const TOLERANCE: f64 = 1e-9;

/// The working scalar. The table, the cores and every reconstruction carry it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // ---------------------------------------------------------------------
    // 1. The dense table, and the train that factors it.
    // ---------------------------------------------------------------------
    // The table is the sum of two separable terms, so its TT rank is at most two at every bond.
    let shape = vec![AXIS_LEN; ORDER];
    let dense = CausalTensor::from_shape_fn(&shape, table_entry);
    let truncation = Truncation::<FloatType>::by_tol(lift::<FloatType>(REL_TOL))?;
    let train = CausalTensorTrain::from_dense(&dense, &truncation)?;

    let dense_entries = dense.as_slice().len();
    let core_entries = core_entry_count(&train);
    print_shapes(
        &shape,
        dense_entries,
        train.order(),
        train.bond_dims(),
        train.max_bond(),
        core_entries,
    );

    // ---------------------------------------------------------------------
    // 2. The factorisation is exact for data this shape.
    // ---------------------------------------------------------------------
    let rebuilt = train.to_dense()?;
    let error = max_difference(dense.as_slice(), rebuilt.as_slice());
    print_reconstruction(error);

    assert!(error < lift::<FloatType>(TOLERANCE));

    // ---------------------------------------------------------------------
    // 3. Foldable folds the cores.
    // ---------------------------------------------------------------------
    // The sum over the cores counts `core_entries` numbers; the sum over the table counts
    // `dense_entries` of them. They answer different questions, and the fold answers the first.
    let zero = lift::<FloatType>(0.0);
    let core_sum = CausalTensorTrainWitness::fold(train.clone(), zero, |acc, v| acc + v);
    let table_sum = dense
        .as_slice()
        .iter()
        .fold(zero, |acc: FloatType, &v| acc + v);
    print_folds(core_sum, core_entries, table_sum, dense_entries);

    // ---------------------------------------------------------------------
    // 4. Functor maps the cores, and the logical tensor feels it once per core.
    // ---------------------------------------------------------------------
    // Multiplying every core entry by `s` multiplies the tensor by `s^d`, because the tensor is
    // the contraction of `d` cores and each contributes one factor.
    let scale = lift::<FloatType>(CORE_SCALE);
    let scaled = CausalTensorTrainWitness::fmap(train.clone(), move |v| v * scale);
    let scaled_dense = scaled.to_dense()?;

    let expected = (0..ORDER).fold(lift::<FloatType>(1.0), |acc, _| acc * scale);
    let observed = scaled_dense.as_slice()[0] / rebuilt.as_slice()[0];
    print_scaling(scale, ORDER, expected, observed);

    assert!(Real::abs(observed - expected) < lift::<FloatType>(TOLERANCE));

    // ---------------------------------------------------------------------
    // 5. Pure: the smallest train there is.
    // ---------------------------------------------------------------------
    let single = CausalTensorTrainWitness::pure(lift::<FloatType>(7.0));
    print_pure(
        single.order(),
        single.bond_dims(),
        core_entry_count(&single),
    );

    print_footer();
    Ok(())
}

/// The lookup table: two separable terms added together.
///
/// Each term is a product across the axes, so each has TT rank 1 and the sum has rank at most 2.
fn table_entry(index: &[usize]) -> FloatType {
    let [i, j, k, l] = [index[0], index[1], index[2], index[3]];
    let at = |n: usize| lift_count::<FloatType>(n as u64);
    let one = lift::<FloatType>(1.0);
    let two = lift::<FloatType>(2.0);
    let three = lift::<FloatType>(3.0);
    let four = lift::<FloatType>(4.0);

    (one + at(i)) * (two + at(j)) * (one + at(k)) * (three + at(l))
        + (four - at(i)) * (one + at(j)) * (three - at(k)) * (one + at(l))
}

/// How many numbers the cores hold between them.
fn core_entry_count(train: &CausalTensorTrain<FloatType>) -> usize {
    train.cores().iter().map(|c| c.as_slice().len()).sum()
}

/// The largest absolute difference between two buffers of the same length.
fn max_difference(a: &[FloatType], b: &[FloatType]) -> FloatType {
    a.iter()
        .zip(b.iter())
        .fold(lift::<FloatType>(0.0), |acc, (&x, &y)| {
            let gap = Real::abs(x - y);
            if gap > acc { gap } else { acc }
        })
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== `CausalTensorTrain`: a tensor stored as its factors ===\n");
    println!("  An order-{ORDER} lookup table with {AXIS_LEN} values per axis, built from two");
    println!("  separable terms and factored by the TT-SVD.\n");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_shapes(
    shape: &[usize],
    dense_entries: usize,
    order: usize,
    bonds: Vec<usize>,
    max_bond: usize,
    core_entries: usize,
) {
    println!("--- 1. Dense, and the train that factors it ---");
    println!("  dense shape       {shape:?}     {dense_entries} entries");
    println!("  train order       {order}");
    println!("  bond dimensions   {bonds:?}   max {max_bond}");
    println!("  core entries      {core_entries}");
    println!(
        "  storage ratio     {:.2}x fewer numbers held",
        dense_entries as f64 / core_entries as f64
    );
}

fn print_reconstruction(error: FloatType) {
    println!("\n--- 2. Contracting the cores returns the table ---");
    println!("  largest entry-wise error   {:.2e}", lower(error));
    println!("  The table has rank 2 at every bond, so the factorisation loses nothing.");
}

fn print_folds(
    core_sum: FloatType,
    core_entries: usize,
    table_sum: FloatType,
    dense_entries: usize,
) {
    println!("\n--- 3. Foldable visits the cores ---");
    println!(
        "  fold over the cores    {:12.4}   over {core_entries} numbers",
        lower(core_sum)
    );
    println!(
        "  sum over the table     {:12.4}   over {dense_entries} numbers",
        lower(table_sum)
    );
    println!("  The witness is a functor over the train, so its fold reaches the factors.");
    println!("  Summing the tensor the factors stand for is the other question, and");
    println!("  `to_dense` is what asks it.");
}

fn print_scaling(scale: FloatType, order: usize, expected: FloatType, observed: FloatType) {
    println!("\n--- 4. Functor maps the cores ---");
    println!(
        "  every core entry x {:.1}, across {order} cores",
        lower(scale)
    );
    println!("  tensor scaled by       {:.4}   observed", lower(observed));
    println!("  s^d, the closed form   {:.4}   expected", lower(expected));
}

fn print_pure(order: usize, bonds: Vec<usize>, entries: usize) {
    println!("\n--- 5. Pure ---");
    println!("  pure(7.0)   order {order}, bonds {bonds:?}, {entries} entry");
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  A tensor train holds `d*n*r^2` numbers where the dense form holds `n^d`. The");
    println!("  witness is a functor over the train, so `fmap` and `fold` act on the cores and");
    println!("  `to_dense` is the step that moves back to the tensor they represent.");
}
