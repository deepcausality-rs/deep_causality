/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The Cayley–Dickson ladder: ℂ, ℍ, 𝕆
//!
//! Doubling a real number gives a complex number, doubling that gives a quaternion, and doubling
//! again gives an octonion. Three witnesses carry the three rungs:
//!
//! ```text
//! ComplexWitness     Complex<T>      2 slots    Functor, Foldable, Semigroupal,
//! QuaternionWitness  Quaternion<T>   4 slots    LaxMonoidal, MonoidalApplicative,
//! OctonionWitness    Octonion<T>     8 slots    Convolutional
//! ```
//!
//! All three carry the same six traits, so a scaling law, a slot-wise combine and a norm are each
//! written once and run on every rung. The euclidean norm in section 2 is the clearest case: one
//! generic function, bounded by `Foldable`, serving 2, 4 and 8 slots.
//!
//! What the rungs keep differs, and sections 4 and 5 measure it rather than assert it:
//!
//! ```text
//! |ab| = |a||b|     holds on every rung — all four are composition algebras
//! ab = ba           holds through ℂ
//! (ab)c = a(bc)     holds through ℍ
//! ```
//!
//! This is why quaternions run rigid-body rotation in robotics and graphics — a rotation composes
//! by multiplication, and composition of rotations is associative — and why octonions appear where
//! the product itself is the object of study, as in the Standard Model constructions the
//! `standard_model_symmetry` application builds on.

use deep_causality_algebra::Real;
use deep_causality_haft::{Foldable, Functor, HKT, LaxMonoidal, MonoidalApplicative, Semigroupal};
use deep_causality_num::{lift, lower};
use deep_causality_num_complex::{
    Complex, ComplexWitness, Octonion, OctonionWitness, Quaternion, QuaternionWitness,
};

/// A complex number, a quaternion and an octonion to work with.
const Z: [f64; 2] = [3.0, 4.0];
const Q: [f64; 4] = [1.0, 2.0, -2.0, 1.0];
const O: [f64; 8] = [1.0, 1.0, -1.0, 2.0, 0.0, 1.0, 1.0, -1.0];

/// The factor the scaling law in section 3 multiplies by.
const SCALE: f64 = 2.5;

/// What counts as zero when a floating-point identity is checked.
const TOLERANCE: f64 = 1e-12;

/// The working scalar. Every component on every rung carries it.
pub type FloatType = f64;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_header();

    let z = complex(&Z);
    let q = quaternion(&Q);
    let o = octonion(&O);

    // ---------------------------------------------------------------------
    // 1. The ladder, counted by the witnesses themselves.
    // ---------------------------------------------------------------------
    // `fold` visits every slot, so counting slots is the same call on all three.
    print_ladder(
        slots::<ComplexWitness>(z),
        slots::<QuaternionWitness>(q),
        slots::<OctonionWitness>(o),
    );

    // ---------------------------------------------------------------------
    // 2. One norm, three algebras.
    // ---------------------------------------------------------------------
    // `euclidean_norm` is written once against `Foldable` and knows nothing about which rung it
    // is on. |z| for [3, 4] is 5 exactly, which is the check that the generic path is right.
    let norms = (
        euclidean_norm::<ComplexWitness>(z),
        euclidean_norm::<QuaternionWitness>(q),
        euclidean_norm::<OctonionWitness>(o),
    );
    print_norms(norms.0, norms.1, norms.2);

    assert_eq!(norms.0, lift::<FloatType>(5.0));

    // ---------------------------------------------------------------------
    // 3. Functor and Semigroupal: one law, three carriers.
    // ---------------------------------------------------------------------
    // `fmap` scales every slot; `zip_with` pairs slot with slot. Neither names a rung, and
    // `unit` is the empty structure the zip has as its identity.
    let scale = lift::<FloatType>(SCALE);
    let scaled = ComplexWitness::fmap(z, move |v| v * scale);
    let doubled = QuaternionWitness::zip_with(q, q, |a, b| a + b);
    let picked = OctonionWitness::apply(slot_functions(), o);

    print_structure(
        scaled.re,
        scaled.im,
        doubled.w,
        picked.s,
        picked.e1,
        unit_slots(),
    );

    // Scaling every slot scales the norm by the same factor.
    let scaled_norm = euclidean_norm::<ComplexWitness>(scaled);
    assert!(Real::abs(scaled_norm - norms.0 * scale) < lift::<FloatType>(TOLERANCE));

    // ---------------------------------------------------------------------
    // 4. What every rung keeps: the norm is multiplicative.
    // ---------------------------------------------------------------------
    // |ab| = |a||b| is what makes these composition algebras, and it survives every doubling.
    let z2 = complex(&[1.0, -2.0]);
    let q2 = quaternion(&[0.0, 1.0, 1.0, 0.0]);
    let o2 = octonion(&[2.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 1.0]);

    let multiplicative = (
        norm_gap::<ComplexWitness>(z * z2, z, z2),
        norm_gap::<QuaternionWitness>(q * q2, q, q2),
        norm_gap::<OctonionWitness>(o * o2, o, o2),
    );
    print_multiplicative(multiplicative.0, multiplicative.1, multiplicative.2);

    let tolerance = lift::<FloatType>(TOLERANCE);
    assert!(multiplicative.0 < tolerance);
    assert!(multiplicative.1 < tolerance);
    assert!(multiplicative.2 < tolerance);

    // ---------------------------------------------------------------------
    // 5. Where each rung stops.
    // ---------------------------------------------------------------------
    // The commutator ab − ba measures commutativity; the associator (ab)c − a(bc) measures
    // associativity. Both are read through the same generic norm.
    let i = quaternion(&[0.0, 1.0, 0.0, 0.0]);
    let j = quaternion(&[0.0, 0.0, 1.0, 0.0]);
    let complex_commutator = euclidean_norm::<ComplexWitness>(z * z2 - z2 * z);
    let quaternion_commutator = euclidean_norm::<QuaternionWitness>(i * j - j * i);

    let e1 = octonion(&[0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let e2 = octonion(&[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let e4 = octonion(&[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]);
    let quaternion_associator = euclidean_norm::<QuaternionWitness>((i * j) * q - i * (j * q));
    let octonion_associator = euclidean_norm::<OctonionWitness>((e1 * e2) * e4 - e1 * (e2 * e4));

    print_limits(
        complex_commutator,
        quaternion_commutator,
        quaternion_associator,
        octonion_associator,
    );

    assert!(complex_commutator < tolerance);
    assert!(quaternion_commutator > tolerance);
    assert!(quaternion_associator < tolerance);
    assert!(octonion_associator > tolerance);

    print_footer();
    Ok(())
}

/// The euclidean norm over a carrier's slots, written once against `Foldable`.
///
/// Every rung of the ladder stores its components in a fixed number of slots, and a fold visits
/// each of them once, so this serves 2, 4 and 8 slots through the same body.
fn euclidean_norm<W>(carrier: W::Type<FloatType>) -> FloatType
where
    W: HKT + Foldable<W>,
{
    Real::sqrt(W::fold(carrier, lift::<FloatType>(0.0), |acc, v| {
        acc + v * v
    }))
}

/// How many slots a carrier holds, counted by the fold that visits them.
fn slots<W>(carrier: W::Type<FloatType>) -> usize
where
    W: HKT + Foldable<W>,
{
    W::fold(carrier, 0usize, |acc, _| acc + 1)
}

/// `| |ab| − |a||b| |`, the gap in the multiplicative law.
fn norm_gap<W>(
    product: W::Type<FloatType>,
    a: W::Type<FloatType>,
    b: W::Type<FloatType>,
) -> FloatType
where
    W: HKT + Foldable<W>,
{
    Real::abs(euclidean_norm::<W>(product) - euclidean_norm::<W>(a) * euclidean_norm::<W>(b))
}

/// One function per slot, which is what `MonoidalApplicative::apply` takes.
fn slot_functions() -> Octonion<fn(FloatType) -> FloatType> {
    fn keep(v: FloatType) -> FloatType {
        v
    }
    fn negate(v: FloatType) -> FloatType {
        -v
    }

    // A struct literal, because `Octonion::new` asks for a `RealField` and a function is not one.
    // The witness places no such bound: `Octonion<A>` holds whatever `A` the functor is at.
    Octonion {
        s: keep,
        e1: negate,
        e2: keep,
        e3: negate,
        e4: keep,
        e5: negate,
        e6: keep,
        e7: negate,
    }
}

/// The slot count of the unit, the empty structure `zip_with` has as its identity.
fn unit_slots() -> (usize, usize, usize) {
    let complex = <ComplexWitness as LaxMonoidal<ComplexWitness>>::unit();
    let quaternion = <QuaternionWitness as LaxMonoidal<QuaternionWitness>>::unit();
    let octonion = <OctonionWitness as LaxMonoidal<OctonionWitness>>::unit();

    (
        ComplexWitness::fold(complex, 0usize, |acc, _| acc + 1),
        QuaternionWitness::fold(quaternion, 0usize, |acc, _| acc + 1),
        OctonionWitness::fold(octonion, 0usize, |acc, _| acc + 1),
    )
}

/// The three carriers, lifted from their `f64` tables.
fn complex(values: &[f64; 2]) -> Complex<FloatType> {
    Complex::new(lift::<FloatType>(values[0]), lift::<FloatType>(values[1]))
}

fn quaternion(values: &[f64; 4]) -> Quaternion<FloatType> {
    Quaternion::new(
        lift::<FloatType>(values[0]),
        lift::<FloatType>(values[1]),
        lift::<FloatType>(values[2]),
        lift::<FloatType>(values[3]),
    )
}

fn octonion(values: &[f64; 8]) -> Octonion<FloatType> {
    Octonion::new(
        lift::<FloatType>(values[0]),
        lift::<FloatType>(values[1]),
        lift::<FloatType>(values[2]),
        lift::<FloatType>(values[3]),
        lift::<FloatType>(values[4]),
        lift::<FloatType>(values[5]),
        lift::<FloatType>(values[6]),
        lift::<FloatType>(values[7]),
    )
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== The Cayley-Dickson ladder: C, H, O ===\n");
    println!("  Each rung doubles the one below it. The six HKT traits are the same on all");
    println!("  three, so one norm, one scaling law and one zip serve every rung.\n");
}

fn print_ladder(complex: usize, quaternion: usize, octonion: usize) {
    println!("--- 1. The ladder, counted by Foldable ---");
    println!("  ComplexWitness      Complex<T>      {complex} slots");
    println!("  QuaternionWitness   Quaternion<T>   {quaternion} slots");
    println!("  OctonionWitness     Octonion<T>     {octonion} slots");
}

/// The display boundary: `f64` appears here and nowhere else.
fn print_norms(complex: FloatType, quaternion: FloatType, octonion: FloatType) {
    println!("\n--- 2. One generic norm, bounded by Foldable ---");
    println!("  |z| over {Z:?}   {:.6}", lower(complex));
    println!("  |q| over {Q:?}   {:.6}", lower(quaternion));
    println!("  |o| over 8 slots                {:.6}", lower(octonion));
}

fn print_structure(
    scaled_re: FloatType,
    scaled_im: FloatType,
    doubled_w: FloatType,
    picked_s: FloatType,
    picked_e1: FloatType,
    unit: (usize, usize, usize),
) {
    println!("\n--- 3. Functor, Semigroupal, LaxMonoidal, MonoidalApplicative ---");
    println!(
        "  fmap    {SCALE} * z              re {:.2}, im {:.2}",
        lower(scaled_re),
        lower(scaled_im)
    );
    println!("  zip     q + q, slot by slot    w {:.2}", lower(doubled_w));
    println!(
        "  apply   one function per slot  s {:.2}, e1 {:.2}   (keep, negate, keep, ...)",
        lower(picked_s),
        lower(picked_e1)
    );
    println!(
        "  unit    the zip identity       {} / {} / {} slots",
        unit.0, unit.1, unit.2
    );
}

fn print_multiplicative(complex: FloatType, quaternion: FloatType, octonion: FloatType) {
    println!("\n--- 4. What every rung keeps: |ab| = |a||b| ---");
    println!("  gap on C   {:.2e}", lower(complex));
    println!("  gap on H   {:.2e}", lower(quaternion));
    println!("  gap on O   {:.2e}", lower(octonion));
    println!("  All four normed division algebras are composition algebras, and the norm");
    println!("  law survives every doubling.");
}

fn print_limits(
    complex_commutator: FloatType,
    quaternion_commutator: FloatType,
    quaternion_associator: FloatType,
    octonion_associator: FloatType,
) {
    println!("\n--- 5. Where each rung stops ---");
    println!(
        "  |z w - w z|         on C   {:.2e}   commutativity holds through C",
        lower(complex_commutator)
    );
    println!(
        "  |i j - j i|         on H   {:.2e}   the commutator is 2k, so H is where it ends",
        lower(quaternion_commutator)
    );
    println!(
        "  |(ij)q - i(jq)|     on H   {:.2e}   associativity holds through H",
        lower(quaternion_associator)
    );
    println!(
        "  |(e1e2)e4 - e1(e2e4)| on O {:.2e}   the associator is what O trades away",
        lower(octonion_associator)
    );
}

fn print_footer() {
    println!("\n--- The point ---");
    println!("  The witnesses give all three rungs one container interface, so the norm, the");
    println!("  scaling and the zip are written once. The product underneath is what changes,");
    println!("  and section 5 measures exactly where. A rotation composes by multiplication,");
    println!("  which is why rigid-body work sits on H: the composition is associative there.");
}
