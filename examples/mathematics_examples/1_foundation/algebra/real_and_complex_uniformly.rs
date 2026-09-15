/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # One norm kernel for reals and complex numbers
//!
//! Real and complex scalars are incomparable in the trait tower: reals are ordered and live
//! under `Real` / `RealField`, complex numbers are unordered and live under `ComplexField`.
//! No stock bound covers both, which is why so much numerical code ends up written twice.
//!
//! `Normed` is the bridge, and the reason it works is that the real type is **associated**
//! rather than a parameter:
//!
//! ```text
//! impl<T: RealField> Normed for T             Real = T          |x|  = |x|
//! impl<T: RealField> Normed for Complex<T>    Real = T          |z|  = sqrt(re² + im²)
//! ```
//!
//! `f64::Real` and `Complex<f64>::Real` are both `f64`, so a generic function can say
//! `-> T::Real` without threading a second type parameter. One body then serves both.
//!
//! `ConjugateScalar` goes one further and spans three families -- real fields, `Complex`, and
//! `Dual` for forward-mode AD -- by tying its real type to the weaker `Scalar`. That is what
//! lets a differentiable SVD exist: the magnitude of a dual has to stay a dual, or the
//! derivative is lost on the way through.
//!
//! The payoff at the end is not abstraction for its own sake. `Normed::modulus` is implemented
//! by the scaled form rather than `sqrt(re² + im²)`, and that is the difference between a
//! right answer and an infinity.

use deep_causality_algebra::{ConjugateScalar, Normed, NormedScalar};
use deep_causality_num::{lift, lower};
use deep_causality_num_complex::Complex;
use deep_causality_num_dual::Dual;

/// The working scalar. Both the reals below and the components of the complex carry it.
pub type FloatType = f64;

fn main() {
    print_header();

    // ---------------------------------------------------------------------
    // 1. One function, a real vector and a complex vector.
    // ---------------------------------------------------------------------
    // `norm` is written once. The element type decides what `modulus_squared` means and what
    // the return type is; the body does not branch on either.
    let reals: Vec<FloatType> = [3.0, 4.0].iter().map(|&x| lift(x)).collect();
    let complexes: Vec<Complex<FloatType>> = vec![
        Complex::new(lift::<FloatType>(3.0), lift(4.0)),
        Complex::new(lift::<FloatType>(0.0), lift(0.0)),
    ];

    print_norms(norm(&reals), norm(&complexes));
    // |(3, 4)| = 5 either way: as two reals, or as one complex with those components.
    assert_eq!(lower(norm(&reals)), 5.0);
    assert_eq!(lower(norm(&complexes)), 5.0);

    // ---------------------------------------------------------------------
    // 2. The associated type is what removes the second parameter.
    // ---------------------------------------------------------------------
    let r: FloatType = lift(-2.5);
    let z = Complex::new(lift::<FloatType>(3.0), lift::<FloatType>(4.0));
    // `Normed` and `ConjugateScalar` both offer `modulus`, so the trait is named at the
    // call site. For a real or a complex the two agree; only `Dual` separates them.
    print_associated(
        Normed::modulus(&r),
        Normed::modulus(&z),
        Normed::modulus_squared(&z),
    );

    // ---------------------------------------------------------------------
    // 3. `ConjugateScalar` reaches one family further: `Dual`.
    // ---------------------------------------------------------------------
    // Conjugation is the identity on reals, flips the sign of `im` on complex, and leaves a
    // dual alone -- so one inner-product kernel covers all three. A dual's modulus stays a
    // dual, which is exactly why `ConjugateScalar` does not require `Normed`.
    let d = Dual::variable(lift::<FloatType>(3.0));
    print_conjugates(
        ConjugateScalar::conjugate(&r),
        ConjugateScalar::conjugate(&z),
        ConjugateScalar::conjugate(&d).re,
        ConjugateScalar::modulus(&d).du,
    );

    // ---------------------------------------------------------------------
    // 4. Why `modulus` is a member and not `modulus_squared().sqrt()`.
    // ---------------------------------------------------------------------
    // `|z|` for a component near the top of the range is representable; `re²` is not. The
    // naive form reaches infinity and the square root stays there. The same happens at the
    // bottom: the square underflows to zero and the modulus comes back zero.
    let huge = Complex::new(lift::<FloatType>(1e308), lift::<FloatType>(0.0));
    let tiny = Complex::new(lift::<FloatType>(1e-200), lift::<FloatType>(0.0));

    print_overflow(
        naive_modulus(&huge),
        Normed::modulus(&huge),
        naive_modulus(&tiny),
        Normed::modulus(&tiny),
    );

    // The scaled form factors the larger component out, so the ratio it squares is in [0, 1].
    assert!(lower(naive_modulus(&huge)).is_infinite());
    assert_eq!(lower(Normed::modulus(&huge)), 1e308);
    assert_eq!(lower(naive_modulus(&tiny)), 0.0);
    assert_eq!(lower(Normed::modulus(&tiny)), 1e-200);
}

/// The Euclidean norm of a slice, for real *or* complex elements.
///
/// `T::Real` is the associated real type, so the signature needs no second parameter and the
/// body no branch. At `T = f64` this is the ordinary norm; at `T = Complex<f64>` it is the
/// Hermitian one, because `modulus_squared` already means `re² + im²` there.
fn norm<T: NormedScalar>(xs: &[T]) -> T::Real {
    let zero = <T::Real as deep_causality_num::Zero>::zero();
    let total = xs
        .iter()
        .fold(zero, |acc, x| acc + Normed::modulus_squared(x));
    deep_causality_algebra::Real::sqrt(total)
}

/// `|z|` the direct way, which is what this trait member exists to avoid.
fn naive_modulus(z: &Complex<FloatType>) -> FloatType {
    deep_causality_algebra::Real::sqrt(z.re * z.re + z.im * z.im)
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== One norm kernel for reals and complex numbers ===\n");
    println!("  `Normed::Real` is an associated type, so f64::Real and Complex<f64>::Real");
    println!("  are both f64 and one signature covers both.\n");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_norms(real_norm: FloatType, complex_norm: FloatType) {
    println!("--- 1. The same `norm` over two element types ---");
    println!("  norm([3.0, 4.0])                 = {}", lower(real_norm));
    println!(
        "  norm([3 + 4i, 0])                = {}",
        lower(complex_norm)
    );
    println!("  Same body, same return type, no branch on the element.");
}

fn print_associated(r: FloatType, z: FloatType, z_sq: FloatType) {
    println!("\n--- 2. modulus and modulus_squared ---");
    println!("  |-2.5|          = {}        (a real)", lower(r));
    println!("  |3 + 4i|        = {}          (a complex)", lower(z));
    println!("  |3 + 4i|^2      = {}         (re^2 + im^2)", lower(z_sq));
}

fn print_conjugates(
    r: FloatType,
    z: Complex<FloatType>,
    dual_re: FloatType,
    dual_modulus_du: FloatType,
) {
    println!("\n--- 3. ConjugateScalar spans reals, complex and duals ---");
    println!(
        "  conj(-2.5)      = {}        (identity on reals)",
        lower(r)
    );
    println!(
        "  conj(3 + 4i)    = {} + {}i     (the sign of im flips)",
        lower(z.re),
        lower(z.im)
    );
    println!(
        "  conj(dual 3)    = {}          (identity again)",
        lower(dual_re)
    );
    println!(
        "  |dual 3|.du     = {}          (the modulus keeps its derivative)",
        lower(dual_modulus_du)
    );
}

fn print_overflow(
    naive_huge: FloatType,
    scaled_huge: FloatType,
    naive_tiny: FloatType,
    scaled_tiny: FloatType,
) {
    println!("\n--- 4. Why `modulus` is a member, not a default ---");
    println!(
        "  {:<22} {:>14} {:>14}",
        "z", "sqrt(re^2+im^2)", "modulus()"
    );
    println!(
        "  {:<22} {:>14} {:>14}",
        "1e308 + 0i",
        format!("{:e}", lower(naive_huge)),
        format!("{:e}", lower(scaled_huge))
    );
    println!(
        "  {:<22} {:>14} {:>14}",
        "1e-200 + 0i",
        format!("{:e}", lower(naive_tiny)),
        format!("{:e}", lower(scaled_tiny))
    );
    println!("\n  Both naive answers are wrong for a modulus that is representable: the square");
    println!("  overflows at the top and underflows at the bottom. `modulus` factors the larger");
    println!("  component out first, so the ratio it squares is in [0, 1] and cannot do either.");
}
