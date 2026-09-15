/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The laws that pin octonion multiplication.
//!
//! These replace a set of hand-written basis-product expectations. Those expectations named four
//! products and said nothing about the rest, and a table can satisfy them while still failing the
//! law that defines the algebra. The tests here check the laws instead: they hold for every pair,
//! and they fix the table up to a relabelling of the basis.

use deep_causality_num_complex::Octonion;

fn unit(i: usize) -> Octonion<f64> {
    let mut c = [0.0f64; 8];
    c[i] = 1.0;
    Octonion::new(c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7])
}

fn norm(o: &Octonion<f64>) -> f64 {
    (o.s * o.s
        + o.e1 * o.e1
        + o.e2 * o.e2
        + o.e3 * o.e3
        + o.e4 * o.e4
        + o.e5 * o.e5
        + o.e6 * o.e6
        + o.e7 * o.e7)
        .sqrt()
}

/// `|x y| = |x| |y|` for every pair.
///
/// This is what makes the octonions a composition algebra, and it is the one law a written-out
/// multiplication table gets wrong quietly: a table whose seven Fano lines carry incompatible
/// orientations still squares every unit to `-1` and still anticommutes, so the basis products all
/// look right while this law fails by an amount that depends on the operands.
#[test]
fn test_norm_is_multiplicative() {
    let samples = [
        Octonion::new(1.0, 1.0, -1.0, 2.0, 0.0, 1.0, 1.0, -1.0),
        Octonion::new(2.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0, 1.0),
        Octonion::new(-1.0, 0.5, 0.25, -2.0, 3.0, 1.0, -0.5, 0.75),
        Octonion::new(0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
    ];

    for a in samples.iter() {
        for b in samples.iter() {
            let gap = (norm(&(*a * *b)) - norm(a) * norm(b)).abs();
            assert!(
                gap < 1e-9,
                "|ab| = |a||b| failed by {gap} for {a:?} and {b:?}"
            );
        }
    }
}

/// Every imaginary unit squares to `-1`, and distinct units anticommute.
#[test]
fn test_units_square_to_minus_one_and_anticommute() {
    let minus_one = Octonion::new(-1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    for i in 1..8 {
        let ei = unit(i);
        assert_eq!(ei * ei, minus_one, "e{i} squared");

        for j in 1..8 {
            if i == j {
                continue;
            }
            let ej = unit(j);
            assert_eq!(ei * ej, -(ej * ei), "e{i} and e{j} anticommute");
        }
    }
}

/// The octonions are alternative: `(xx)y = x(xy)` and `(yx)x = y(xx)`.
///
/// Alternativity is the strongest associativity the octonions keep, and it follows from the
/// composition law. A table with an incompatible line orientation breaks it.
#[test]
fn test_multiplication_is_alternative() {
    let x = Octonion::new(1.0, 2.0, -1.0, 0.5, 3.0, -2.0, 1.0, 0.25);
    let y = Octonion::new(-2.0, 1.0, 1.5, -3.0, 0.0, 1.0, -1.0, 2.0);

    let left = (x * x) * y - x * (x * y);
    let right = (y * x) * x - y * (x * x);

    assert!(norm(&left) < 1e-9, "left alternativity failed: {left:?}");
    assert!(norm(&right) < 1e-9, "right alternativity failed: {right:?}");
}

/// A product of two distinct basis units is again a basis unit, up to sign.
#[test]
fn test_basis_products_stay_on_the_basis() {
    for i in 1..8 {
        for j in 1..8 {
            if i == j {
                continue;
            }
            let p = unit(i) * unit(j);
            let components = [p.s, p.e1, p.e2, p.e3, p.e4, p.e5, p.e6, p.e7];
            let nonzero = components.iter().filter(|v| v.abs() > 1e-12).count();

            assert_eq!(nonzero, 1, "e{i} * e{j} landed on {nonzero} components");
            assert!(
                (norm(&p) - 1.0).abs() < 1e-12,
                "e{i} * e{j} changed the norm"
            );
        }
    }
}
