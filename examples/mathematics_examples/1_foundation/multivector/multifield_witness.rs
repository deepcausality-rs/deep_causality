/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `CausalMultiFieldWitness`: the HKT layer over a geometric-algebra field
//!
//! `CausalMultiField` holds one multivector per grid cell. Its inherent operators — `gradient`,
//! `curl`, `divergence`, the products — are what field work runs on. This example is about the
//! layer above them:
//!
//! ```text
//! HKT       Type<A> = CausalMultiField<A, T>
//! Functor   fmap     maps every coefficient, carrying metric, spacing and shape across
//! Pure      pure     lifts one value into the smallest field holding it
//! CoMonad   extract  the coefficient at flat position 0
//!           extend   runs a closure at every focus
//! ```
//!
//! ## What a focus is here
//!
//! `extend` rotates the flat coefficient buffer so that cell `i` lands where `extract` reads. That
//! buffer interleaves grid cells with the matrix coefficients inside each cell, because a
//! `CausalMultiField` stores the matrix isomorphism of each multivector, shaped
//! `[Nx, Ny, Nz, D, D]`. So a focus is a rotation of that buffer, and it is what makes
//! `extend(extract) == id` hold. Grid neighbourhoods come from the inherent differential
//! operators, which is what section 4 uses.
//!
//! `extend` is also quadratic in the coefficient count, since each focus is a real field rather
//! than a borrowed view. That suits the laws and small fields.

use deep_causality_algebra::Real;
use deep_causality_haft::{CoMonad, Functor, Pure};
use deep_causality_multivector::{
    CausalMultiField, CausalMultiFieldWitness, CausalMultiVector, Metric, matrix_dim,
};
use deep_causality_num::{const_scalar_from_float, const_scalar_from_int, lift_usize, lower};

/// `Cl(3, 0)`: three positive basis vectors.
const N: usize = 3;
/// Blade indices in `Cl(3, 0)`, each axis owning one bit.
const E1: usize = 1;
const E2: usize = 2;

/// A `2 x 2 x 1` grid at this spacing.
const SHAPE: [usize; 3] = [2, 2, 1];
const CELLS: usize = SHAPE[0] * SHAPE[1] * SHAPE[2];
const SPACING: FloatType = const_scalar_from_float!(FloatType, 0.5);

/// The gain section 2 applies to every coefficient.
const GAIN: FloatType = const_scalar_from_int!(FloatType, 3);

/// What counts as zero when a linearity check is made.
const TOLERANCE: FloatType = const_scalar_from_float!(FloatType, 1e-12);

/// The working scalar. Every coefficient in the field carries it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const FIVE: FloatType = const_scalar_from_int!(FloatType, 5);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let metric = Metric::Euclidean(N);
    let dx = [SPACING; 3];

    // Cell k holds k·e1 + e2, so the field varies across the grid and the gradient has something
    // to find.
    let cells: Vec<CausalMultiVector<FloatType>> = (0..CELLS)
        .map(|k| multivector(&[(E1, lift_usize(k)), (E2, ONE)], metric))
        .collect::<Result<_, _>>()?;
    let field = CausalMultiField::from_coefficients(&cells, SHAPE, dx);

    // ---------------------------------------------------------------------
    // 1. What the carrier holds.
    // ---------------------------------------------------------------------
    print_storage(
        field.data().shape(),
        field.data().as_slice().len(),
        matrix_dim(N),
    );

    // ---------------------------------------------------------------------
    // 2. Functor: map the coefficients, keep the field.
    // ---------------------------------------------------------------------
    let gain = GAIN;
    let amplified = CausalMultiFieldWitness::<FloatType>::fmap(field.clone(), move |v| v * gain);
    print_functor(
        field.data().shape(),
        amplified.data().shape(),
        *amplified.dx(),
        amplified.metric(),
    );

    // The metric, the spacing and the shape travelled across untouched.
    assert_eq!(amplified.data().shape(), field.data().shape());
    assert_eq!(amplified.dx(), field.dx());

    // ---------------------------------------------------------------------
    // 3. CoMonad: extract, and the law extend is arranged to satisfy.
    // ---------------------------------------------------------------------
    let focus = CausalMultiFieldWitness::<FloatType>::extract(&field);
    let round_trip = CausalMultiFieldWitness::<FloatType>::extend(
        &field,
        CausalMultiFieldWitness::<FloatType>::extract,
    );
    let law_holds = round_trip.data().as_slice() == field.data().as_slice();
    print_comonad(focus, law_holds);

    assert!(law_holds);

    // ---------------------------------------------------------------------
    // 4. The functor and the field operators commute.
    // ---------------------------------------------------------------------
    // `gradient` is linear, so scaling the coefficients and then differentiating gives the same
    // answer as differentiating and then scaling. That is what lets a unit change be applied at
    // whichever end of a pipeline is convenient.
    let scaled_then_differentiated = amplified.gradient();
    let differentiated_then_scaled =
        CausalMultiFieldWitness::<FloatType>::fmap(field.gradient(), move |v| v * gain);
    let gap = max_difference(
        scaled_then_differentiated.data().as_slice(),
        differentiated_then_scaled.data().as_slice(),
    );
    print_commuting(gap);

    assert!(gap < TOLERANCE);

    // ---------------------------------------------------------------------
    // 5. Pure: the smallest field holding one value.
    // ---------------------------------------------------------------------
    let single = CausalMultiFieldWitness::<FloatType>::pure(FIVE);
    print_pure(single.data().shape(), single.data().as_slice().len());

    print_footer();
    Ok(())
}

/// A multivector in `Cl(3, 0)` with the given blade coefficients set.
fn multivector(
    blades: &[(usize, FloatType)],
    metric: Metric,
) -> Result<CausalMultiVector<FloatType>, Box<dyn std::error::Error>> {
    let mut data = vec![ZERO; 1 << N];
    for &(index, value) in blades {
        data[index] = value;
    }

    Ok(CausalMultiVector::new(data, metric)?)
}

/// The largest absolute difference between two buffers.
fn max_difference(a: &[FloatType], b: &[FloatType]) -> FloatType {
    a.iter().zip(b.iter()).fold(ZERO, |acc, (&x, &y)| {
        let gap = Real::abs(x - y);
        if gap > acc { gap } else { acc }
    })
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== `CausalMultiFieldWitness`: the HKT layer over a geometric-algebra field ===\n");
    println!(
        "  A {}x{}x{} grid in Cl({N}, 0), one multivector per cell.\n",
        SHAPE[0], SHAPE[1], SHAPE[2]
    );
}

fn print_storage(shape: &[usize], coefficients: usize, dim: usize) {
    println!("--- 1. What the carrier holds ---");
    println!("  tensor shape     {shape:?}");
    println!("  coefficients     {coefficients}");
    println!("  matrix_dim({N})    {dim}    the field stores the matrix isomorphism per cell,");
    println!("                        so the buffer interleaves cells with their coefficients");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_functor(before: &[usize], after: &[usize], dx: [FloatType; 3], metric: Metric) {
    println!("\n--- 2. Functor::fmap, gain {GAIN} on every coefficient ---");
    println!("  shape before     {before:?}");
    println!("  shape after      {after:?}");
    println!(
        "  spacing after    [{:.2}, {:.2}, {:.2}]",
        lower(dx[0]),
        lower(dx[1]),
        lower(dx[2])
    );
    println!("  metric after     {metric}");
    println!("  The coefficients moved; the geometry around them was carried across.");
}

fn print_comonad(focus: FloatType, law_holds: bool) {
    println!("\n--- 3. CoMonad ---");
    println!("  extract                  {:.4}", lower(focus));
    println!("  extend(extract) == id    {law_holds}");
    println!("  A focus is the flat buffer rotated so that cell i sits where extract reads.");
    println!("  Grid neighbourhoods come from the differential operators below.");
}

fn print_commuting(gap: FloatType) {
    println!("\n--- 4. fmap and gradient commute ---");
    println!("  gradient(gain * F)  against  gain * gradient(F)");
    println!("  largest difference       {:.2e}", lower(gap));
    println!("  `gradient` is linear, so a unit change applies at either end of a pipeline.");
}

fn print_pure(shape: &[usize], coefficients: usize) {
    println!("\n--- 5. Pure ---");
    println!("  pure(5.0)   shape {shape:?}, {coefficients} coefficient");
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  The witness supplies the categorical layer: `fmap` moves the payload and leaves");
    println!("  the metric, the spacing and the shape in place, and `extend` satisfies the");
    println!("  comonad laws. The differential operators supply the physics, and section 4");
    println!("  shows the two agree where they overlap.");
}
