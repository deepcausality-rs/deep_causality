/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `DenseMatrix` as a comonad: a stencil on a periodic grid
//!
//! `DenseMatrixWitness` carries `Functor`, `Pure`, `Applicative`, `CoMonad` and `Foldable`. The
//! comonad is the interesting one, and it is what a stencil computation is:
//!
//! ```text
//! extract   the (0, 0) entry
//! extend    run a closure at every position, each seeing the whole matrix focused there
//! ```
//!
//! `DenseMatrix` holds no cursor, so the focus is expressed by moving the data: `extend` hands the
//! closure a view with the focused entry rotated to `(0, 0)` and the rest wrapped around it. The
//! grid is a torus, and a neighbourhood is read at fixed offsets from the origin whichever cell is
//! in focus. A stencil is then written once, with no edge cases at the borders.
//!
//! The setting is a temperature map across a chip die with one hotspot, smoothed by a nine-point
//! box filter. Two properties are checked rather than asserted in prose:
//!
//! ```text
//! extend(extract) == id         the comonad law the shifted view is arranged to satisfy
//! sum is preserved              every cell is counted nine times across the whole grid,
//!                               so a normalised box filter moves heat without adding any
//! ```

use deep_causality_algebra::Real;
use deep_causality_haft::{Applicative, CoMonad, Foldable, Functor, Pure};
use deep_causality_linear::{DenseMatrix, DenseMatrixWitness, MatrixView};
use deep_causality_num::{lift, lift_count, lower};

/// The die is a `ROWS x COLS` grid of temperature sensors, in °C.
const ROWS: usize = 4;
const COLS: usize = 4;
const TEMPERATURES: [f64; ROWS * COLS] = [
    42.0, 43.0, 44.0, 42.0, //
    43.0, 91.0, 46.0, 43.0, // the hotspot sits here
    44.0, 47.0, 45.0, 44.0, //
    42.0, 43.0, 44.0, 43.0,
];

/// A nine-point box filter: the focused cell and the eight around it.
const STENCIL_CELLS: u64 = 9;

/// Kelvin is the same scale with this offset, which is the unit change in section 2.
const KELVIN_OFFSET: f64 = 273.15;

/// What counts as zero when a conservation law is checked.
const TOLERANCE: f64 = 1e-9;

/// The working scalar. Every temperature carries it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let field = DenseMatrix::from_vec(
        TEMPERATURES.iter().map(|&t| lift::<FloatType>(t)).collect(),
        ROWS,
        COLS,
    )?;
    print_grid("measured, in C", field.as_slice());

    // ---------------------------------------------------------------------
    // 1. The comonad law the shifted view exists to satisfy.
    // ---------------------------------------------------------------------
    // `extract` reads the (0, 0) entry, and `extend` focuses each position there in turn, so
    // extending with `extract` returns the matrix unchanged.
    let identity = DenseMatrixWitness::extend(&field, DenseMatrixWitness::extract);
    print_law(identity.as_slice() == field.as_slice());

    assert_eq!(identity.as_slice(), field.as_slice());

    // ---------------------------------------------------------------------
    // 2. Functor: relabel every entry, shape untouched.
    // ---------------------------------------------------------------------
    let offset = lift::<FloatType>(KELVIN_OFFSET);
    let kelvin = DenseMatrixWitness::fmap(field.clone(), move |c| c + offset);
    print_grid("the same field, in K", kelvin.as_slice());

    // ---------------------------------------------------------------------
    // 3. CoMonad: the stencil.
    // ---------------------------------------------------------------------
    // In the view the focused cell sits at (0, 0), so its eight neighbours are at rows
    // {0, 1, ROWS-1} and columns {0, 1, COLS-1}. The grid wraps, so this reads nine cells wherever
    // the focus is and the border needs no special case.
    let smoothed = DenseMatrixWitness::extend(&field, |view| {
        let (rows, cols) = (view.rows(), view.cols());
        let data = view.as_slice();
        let sum = [0, 1, rows - 1]
            .iter()
            .flat_map(|&i| [0, 1, cols - 1].map(move |j| (i, j)))
            .fold(lift::<FloatType>(0.0), |acc, (i, j)| {
                acc + data[i * cols + j]
            });

        sum / lift_count::<FloatType>(STENCIL_CELLS)
    });
    print_grid("after one box filter", smoothed.as_slice());

    // ---------------------------------------------------------------------
    // 4. Foldable: the conservation law.
    // ---------------------------------------------------------------------
    // Across the whole grid every cell is read by exactly nine neighbourhoods, so a filter that
    // divides by nine moves heat around and adds none.
    let zero = lift::<FloatType>(0.0);
    let before = DenseMatrixWitness::fold(field.clone(), zero, |acc, v| acc + v);
    let after = DenseMatrixWitness::fold(smoothed.clone(), zero, |acc, v| acc + v);
    let peak_before = peak(field.as_slice());
    let peak_after = peak(smoothed.as_slice());
    print_conservation(before, after, peak_before, peak_after);

    assert!(Real::abs(after - before) < lift::<FloatType>(TOLERANCE));
    assert!(peak_after < peak_before);

    // ---------------------------------------------------------------------
    // 5. Pure and Applicative: the 1x1 broadcast.
    // ---------------------------------------------------------------------
    // `pure` builds the smallest container holding one value, and `apply` broadcasts a `1 x 1`
    // side across the other, so one function reaches every entry.
    let to_fahrenheit = DenseMatrixWitness::pure(|c: FloatType| {
        c * lift::<FloatType>(9.0) / lift::<FloatType>(5.0) + lift::<FloatType>(32.0)
    });
    let fahrenheit = DenseMatrixWitness::apply(to_fahrenheit, smoothed);
    print_broadcast(fahrenheit.rows(), fahrenheit.cols(), fahrenheit.as_slice());

    print_footer();
    Ok(())
}

/// The largest value in the grid.
fn peak(values: &[FloatType]) -> FloatType {
    values.iter().fold(
        lift::<FloatType>(f64::MIN),
        |acc, &v| if v > acc { v } else { acc },
    )
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== `DenseMatrix` as a comonad: a stencil on a periodic grid ===\n");
    println!("  A {ROWS}x{COLS} die temperature map with one hotspot, smoothed by a nine-point");
    println!("  box filter written as a single `extend`.\n");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_grid(title: &str, values: &[FloatType]) {
    println!("--- {title} ---");
    for row in 0..ROWS {
        let cells: Vec<String> = (0..COLS)
            .map(|col| format!("{:7.2}", lower(values[row * COLS + col])))
            .collect();
        println!("  {}", cells.join(" "));
    }
    println!();
}

fn print_law(holds: bool) {
    println!("--- the comonad law ---");
    println!("  extend(extract) == id    {holds}");
    println!("  Each position is focused at (0, 0) in its own view, so extending with the");
    println!("  counit reproduces the matrix entry for entry.\n");
}

fn print_conservation(
    before: FloatType,
    after: FloatType,
    peak_before: FloatType,
    peak_after: FloatType,
) {
    println!("--- Foldable: what the filter moved, and what it kept ---");
    println!("  total before   {:9.4} C", lower(before));
    println!("  total after    {:9.4} C", lower(after));
    println!("  difference     {:9.2e}", lower(Real::abs(after - before)));
    println!(
        "  peak           {:7.2} C  ->  {:7.2} C",
        lower(peak_before),
        lower(peak_after)
    );
    println!("  The sum is conserved and the hotspot is spread, which is what a smoothing");
    println!("  filter is for: the heat is still on the die, and it is no longer in one cell.");
}

fn print_broadcast(rows: usize, cols: usize, values: &[FloatType]) {
    println!("\n--- Pure and Applicative: one function over every entry ---");
    println!("  pure built a 1x1 and apply broadcast it to {rows}x{cols}");
    println!("  first row, in F:  {:?}", to_two(&values[0..cols]));
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  A stencil is a comonad: every output cell is a function of the whole grid");
    println!("  focused at that cell. `extend` supplies the focus, the wrap makes the grid a");
    println!("  torus, and the filter is written once with the border handled by arithmetic.");
}

fn to_two(values: &[FloatType]) -> Vec<f64> {
    values
        .iter()
        .map(|&v| (lower(v) * 100.0).round() / 100.0)
        .collect()
}
