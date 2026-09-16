/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # Alignment: pairing two structures position by position
//!
//! A container can be read as an applicative in more than one way, and the reading decides what
//! "combine these two" means. Four crates supply the pieces:
//!
//! ```text
//! Semigroupal            zip_with: pair slot with slot, combining in one pass
//! Convolutional          the promise that zip reassociates: zip(zip(a,b),c) = zip(a,zip(b,c))
//! MonoidalApplicative    apply, derived from zip_with, needing no Clone on the payload
//! LaxMonoidal            unit: the empty structure zip has as its identity
//! ```
//!
//! `CausalTensorWitness` and `DenseVectorWitness` broadcast: a one-element function container
//! is applied to every slot. `ZipTensorWitness` and `ZipDenseVectorWitness` project to the same
//! containers and align instead: slot `i` with slot `i`. Same data, two lawful readings, and the
//! witness at the call site is what picks one.
//!
//! `ComplexWitness` and `DualWitness` carry the whole family including `LaxMonoidal`, so the
//! same vocabulary reaches a number type with two components as readily as a vector with many.

use deep_causality_haft::{
    Applicative, Convolutional, HKT, LaxMonoidal, MonoidalApplicative, Pure, Semigroupal,
};
use deep_causality_linear::{DenseVector, ZipDenseVectorWitness};
use deep_causality_num::{const_scalar_from_int, lift, lower};
use deep_causality_num_complex::{Complex, ComplexWitness};
use deep_causality_num_dual::{Dual, DualWitness};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness, ZipTensorWitness};

/// The working scalar. Every measurement below carries it.
pub type FloatType = f64;

/// Small numbers, declared once at the working type rather than lifted at each use.
const ZERO: FloatType = const_scalar_from_int!(FloatType, 0);
const ONE: FloatType = const_scalar_from_int!(FloatType, 1);
const TWO: FloatType = const_scalar_from_int!(FloatType, 2);
const THREE: FloatType = const_scalar_from_int!(FloatType, 3);
const FOUR: FloatType = const_scalar_from_int!(FloatType, 4);
const FIVE: FloatType = const_scalar_from_int!(FloatType, 5);
const HUNDRED: FloatType = const_scalar_from_int!(FloatType, 100);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    // ---------------------------------------------------------------------
    // 1. Two readings of the same container.
    // ---------------------------------------------------------------------
    // Two sensor channels sampled at the same four instants. Pairing them is positional:
    // reading i of one belongs with reading i of the other.
    let a = tensor(&[1.0, 2.0, 3.0, 4.0])?;
    let b = tensor(&[10.0, 20.0, 30.0, 40.0])?;

    // The zip witness aligns.
    let summed = ZipTensorWitness::zip_with(a.clone(), b.clone(), |x, y| x + y);
    let paired = ZipTensorWitness::zip(a.clone(), b.clone());
    print_alignment(summed.as_slice(), paired.as_slice());

    // The plain witness broadcasts: one function reaches every slot.
    let scale: fn(FloatType) -> FloatType = |x| x * HUNDRED;
    let broadcast = CausalTensorWitness::apply(CausalTensorWitness::pure(scale), a.clone());
    print_broadcast(broadcast.as_slice());

    // ---------------------------------------------------------------------
    // 2. `apply` through the monoidal reading.
    // ---------------------------------------------------------------------
    // One function per slot rather than one function for all slots. `zip_with` pairs each
    // function with its own argument exactly once, so the payload needs no `Clone`.
    let ops: CausalTensor<fn(FloatType) -> FloatType> = CausalTensor::new(
        vec![
            (|x| x + ONE) as fn(FloatType) -> FloatType,
            |x| x * TWO,
            |x| x - ONE,
            |x| x / TWO,
        ],
        vec![4],
    )?;
    let per_slot =
        <ZipTensorWitness as MonoidalApplicative<ZipTensorWitness>>::apply(ops, a.clone());
    print_per_slot(per_slot.as_slice());

    // ---------------------------------------------------------------------
    // 3. `Convolutional`: the promise that zip reassociates.
    // ---------------------------------------------------------------------
    // The compiler cannot check associativity, so the trait is the author's assertion. It is
    // checkable on values, and here it holds.
    let c = tensor(&[100.0, 200.0, 300.0, 400.0])?;
    let left = ZipTensorWitness::zip_with(
        ZipTensorWitness::zip_with(a.clone(), b.clone(), |x, y| x + y),
        c.clone(),
        |xy, z| xy + z,
    );
    let right = ZipTensorWitness::zip_with(
        a.clone(),
        ZipTensorWitness::zip_with(b.clone(), c.clone(), |y, z| y + z),
        |x, yz| x + yz,
    );
    print_associativity(left.as_slice(), right.as_slice(), is_convolutional());

    // ---------------------------------------------------------------------
    // 4. The same vocabulary over a dense vector.
    // ---------------------------------------------------------------------
    let u = DenseVector::from_vec(vec![ONE, lift(2.0), lift(3.0)]);
    let v = DenseVector::from_vec(vec![FOUR, lift(5.0), lift(6.0)]);
    let dot_terms = ZipDenseVectorWitness::zip_with(u, v, |x, y| x * y);
    print_dense(dot_terms.as_slice());

    // ---------------------------------------------------------------------
    // 5. `LaxMonoidal`: the unit that zip has as its identity.
    // ---------------------------------------------------------------------
    // A number type with a fixed number of components carries the same family. `unit` is the
    // structure holding `()`, which is what makes the monoid a monoid rather than a semigroup.
    let z1 = Complex::new(THREE, FOUR);
    let z2 = Complex::new(ONE, TWO);
    let z_sum = ComplexWitness::zip_with(z1, z2, |x, y| x + y);
    let complex_unit = <ComplexWitness as LaxMonoidal<ComplexWitness>>::unit();

    // A dual number carries a value and its derivative; zipping pairs both components.
    let d1 = Dual::new(TWO, ONE);
    let d2 = Dual::new(FIVE, ZERO);
    let d_sum = DualWitness::zip_with(d1, d2, |x, y| x + y);
    let dual_unit = <DualWitness as LaxMonoidal<DualWitness>>::unit();

    print_lax_monoidal(&z_sum, &complex_unit, &d_sum, &dual_unit);

    Ok(())
}

/// A rank-1 tensor of the working scalar.
fn tensor(values: &[f64]) -> Result<CausalTensor<FloatType>, Box<dyn std::error::Error>> {
    let data: Vec<FloatType> = values.iter().map(|&x| lift(x)).collect();
    let n = data.len();
    Ok(CausalTensor::new(data, vec![n])?)
}

/// Compiles only because `ZipTensorWitness` asserts `Convolutional`; the call is the proof.
fn is_convolutional() -> bool {
    fn requires<W: Convolutional<W> + HKT>() {}
    requires::<ZipTensorWitness>();
    requires::<ZipDenseVectorWitness>();
    true
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

/// The display boundary: `f64` appears here and nowhere else.
fn shown(xs: &[FloatType]) -> Vec<f64> {
    xs.iter().map(|&x| lower(x)).collect()
}

fn print_header() {
    println!("=== Alignment: pairing two structures position by position ===\n");
    println!("a = [1, 2, 3, 4]   b = [10, 20, 30, 40]\n");
    println!("Precision: {}\n", core::any::type_name::<FloatType>());
}

fn print_alignment(summed: &[FloatType], paired: &[(FloatType, FloatType)]) {
    let pairs: Vec<(f64, f64)> = paired.iter().map(|&(x, y)| (lower(x), lower(y))).collect();
    println!("--- 1. The zip witness aligns ---");
    println!("  zip_with(a, b, +) = {:?}", shown(summed));
    println!("  zip(a, b)         = {pairs:?}");
}

fn print_broadcast(values: &[FloatType]) {
    println!("\n  The plain witness broadcasts instead: one function reaches every slot.");
    println!("  apply(pure(*100), a) = {:?}", shown(values));
}

fn print_per_slot(values: &[FloatType]) {
    println!("\n--- 2. MonoidalApplicative: one function per slot ---");
    println!("  ops = [+1, *2, -1, /2] applied to a");
    println!("  apply(ops, a) = {:?}", shown(values));
}

fn print_associativity(left: &[FloatType], right: &[FloatType], asserted: bool) {
    println!("\n--- 3. Convolutional: zip reassociates ---");
    println!("  zip(zip(a, b), c) = {:?}", shown(left));
    println!("  zip(a, zip(b, c)) = {:?}", shown(right));
    println!("  equal: {}", shown(left) == shown(right));
    println!("  both zip witnesses assert Convolutional: {asserted}");
}

fn print_dense(terms: &[FloatType]) {
    println!("\n--- 4. The same vocabulary over a dense vector ---");
    println!("  u = [1, 2, 3]  v = [4, 5, 6]");
    println!(
        "  zip_with(u, v, *) = {:?}   (the dot product's terms)",
        shown(terms)
    );
    println!("  their sum = {}", shown(terms).iter().sum::<f64>());
}

fn print_lax_monoidal(
    z_sum: &Complex<FloatType>,
    complex_unit: &Complex<()>,
    d_sum: &Dual<FloatType>,
    dual_unit: &Dual<()>,
) {
    println!("\n--- 5. LaxMonoidal: zip's unit ---");
    println!(
        "  (3 + 4i) zip (1 + 2i) under + = {} + {}i",
        lower(z_sum.re),
        lower(z_sum.im)
    );
    println!("  ComplexWitness::unit() = {complex_unit:?}");
    println!(
        "  dual(2, 1) zip dual(5, 0) under + = value {} , derivative {}",
        lower(d_sum.re),
        lower(d_sum.du)
    );
    println!("  DualWitness::unit()    = {dual_unit:?}");
}
