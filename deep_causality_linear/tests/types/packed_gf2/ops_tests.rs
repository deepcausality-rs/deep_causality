/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The operator impls for the bit-packed 𝔽₂ matrix, where arithmetic is word-parallel.

use deep_causality_linear::{MatrixBuild, MatrixView, PackedGf2, RowOps};
use deep_causality_num::{Gf2, One, Zero};

fn packed(bits: &[u8], r: usize, c: usize) -> PackedGf2<u8> {
    let cells: Vec<Gf2> = bits.iter().map(|&b| Gf2::new(b != 0)).collect();
    PackedGf2::from_slice(&cells, r, c).unwrap()
}

#[test]
fn test_zero_and_is_zero() {
    let z: PackedGf2<u8> = PackedGf2::zero();
    assert!(z.is_zero());
    assert!(packed(&[0, 0, 0, 0], 2, 2).is_zero());
    assert!(!packed(&[0, 0, 0, 1], 2, 2).is_zero());
}

#[test]
fn test_one_and_is_one() {
    let o: PackedGf2<u8> = PackedGf2::one();
    assert_eq!(o.shape(), (1, 1));
    assert!(o.is_one());
    let i3: PackedGf2<u8> = PackedGf2::identity(3);
    assert!(i3.is_one());
    assert!(!packed(&[1, 1, 0, 1], 2, 2).is_one());
}

#[test]
fn test_addition_is_exclusive_or() {
    let a = packed(&[1, 1, 0, 0], 2, 2);
    let b = packed(&[1, 0, 1, 0], 2, 2);
    let sum = a + b;
    // 1^1=0, 1^0=1, 0^1=1, 0^0=0
    assert_eq!(sum.get(0, 0).unwrap(), Gf2::ZERO);
    assert_eq!(sum.get(0, 1).unwrap(), Gf2::ONE);
    assert_eq!(sum.get(1, 0).unwrap(), Gf2::ONE);
    assert_eq!(sum.get(1, 1).unwrap(), Gf2::ZERO);
}

#[test]
fn test_subtraction_coincides_with_addition() {
    let a = packed(&[1, 1, 0, 1], 2, 2);
    let b = packed(&[1, 0, 1, 1], 2, 2);
    let sum = a.clone() + b.clone();
    let diff = a - b;
    for i in 0..2 {
        for j in 0..2 {
            assert_eq!(sum.get(i, j).unwrap(), diff.get(i, j).unwrap());
        }
    }
}

#[test]
fn test_negation_is_the_identity() {
    let a = packed(&[1, 0, 1, 1], 2, 2);
    let n = -a.clone();
    for i in 0..2 {
        for j in 0..2 {
            assert_eq!(n.get(i, j).unwrap(), a.get(i, j).unwrap());
        }
    }
}

#[test]
fn test_every_matrix_is_its_own_additive_inverse() {
    let a = packed(&[1, 0, 1, 1], 2, 2);
    assert!((a.clone() + a).is_zero());
}

#[test]
fn test_multiplication_is_the_parity_of_shared_ones() {
    // [1 1; 0 1] * [1 0; 1 1] over F2 = [0 1; 1 1]
    let a = packed(&[1, 1, 0, 1], 2, 2);
    let b = packed(&[1, 0, 1, 1], 2, 2);
    let p = a * b;
    assert_eq!(p.get(0, 0).unwrap(), Gf2::ZERO, "1*1 + 1*1 = 0 over F2");
    assert_eq!(p.get(0, 1).unwrap(), Gf2::ONE);
    assert_eq!(p.get(1, 0).unwrap(), Gf2::ONE);
    assert_eq!(p.get(1, 1).unwrap(), Gf2::ONE);
}

#[test]
fn test_multiplication_does_not_commute_over_gf2_either() {
    let a = packed(&[1, 1, 0, 1], 2, 2);
    let b = packed(&[1, 0, 1, 1], 2, 2);
    let ab = a.clone() * b.clone();
    let ba = b * a;
    assert_ne!(ab.get(0, 0).unwrap(), ba.get(0, 0).unwrap());
}

#[test]
fn test_the_identity_is_the_multiplicative_unit() {
    let a = packed(&[1, 1, 0, 1], 2, 2);
    let i: PackedGf2<u8> = PackedGf2::identity(2);
    let p = a.clone() * i;
    for r in 0..2 {
        for c in 0..2 {
            assert_eq!(p.get(r, c).unwrap(), a.get(r, c).unwrap());
        }
    }
}

#[test]
fn test_scaling_by_a_ring_element() {
    let a = packed(&[1, 1, 0, 1], 2, 2);
    // Zero clears; anything else leaves it, since the only units of F2 are 0 and 1.
    assert!((a.clone() * Gf2::ZERO).is_zero());
    let kept = a.clone() * Gf2::ONE;
    assert_eq!(kept.get(0, 0).unwrap(), Gf2::ONE);

    let mut b = a;
    b *= Gf2::ZERO;
    assert!(b.is_zero());
}

#[test]
fn test_the_word_parallel_row_update() {
    // axpy over F2 is dst ^= src when the factor is one, and a no-op when it is zero.
    let mut a = packed(&[1, 1, 0, 0, 1, 0, 1, 0], 2, 4);
    a.axpy_rows(1, 0, &Gf2::ZERO, 0).unwrap();
    assert_eq!(
        a.get(1, 0).unwrap(),
        Gf2::ONE,
        "a zero factor changes nothing"
    );
    a.axpy_rows(1, 0, &Gf2::ONE, 0).unwrap();
    // row1 ^= row0: [1,0,1,0] ^ [1,1,0,0] = [0,1,1,0]
    assert_eq!(a.get(1, 0).unwrap(), Gf2::ZERO);
    assert_eq!(a.get(1, 1).unwrap(), Gf2::ONE);
    assert_eq!(a.get(1, 2).unwrap(), Gf2::ONE);
    assert_eq!(a.get(1, 3).unwrap(), Gf2::ZERO);
}

#[test]
fn test_scale_row_is_degenerate_over_gf2() {
    let mut a = packed(&[1, 1, 1, 1], 2, 2);
    a.scale_row(0, &Gf2::ONE, 0).unwrap();
    assert_eq!(
        a.get(0, 0).unwrap(),
        Gf2::ONE,
        "scaling by the only unit leaves the row"
    );
    a.scale_row(0, &Gf2::ZERO, 0).unwrap();
    assert_eq!(a.get(0, 0).unwrap(), Gf2::ZERO, "scaling by zero clears it");
    assert_eq!(
        a.get(1, 0).unwrap(),
        Gf2::ONE,
        "and leaves other rows alone"
    );
}

#[test]
fn test_swap_rows_and_the_pivot_search() {
    let mut a = packed(&[0, 1, 1, 0], 2, 2);
    assert_eq!(
        a.pivot_in_column(0, 0),
        Some(1),
        "the search must skip the zero at (0,0)"
    );
    a.swap_rows(0, 1).unwrap();
    assert_eq!(a.get(0, 0).unwrap(), Gf2::ONE);
    // The swap turned [0 1; 1 0] into the identity, so column 0 below row 0 is all zero.
    assert_eq!(
        a.pivot_in_column(0, 1),
        None,
        "an all-zero tail has no pivot"
    );
    assert_eq!(
        a.pivot_in_column(1, 1),
        Some(1),
        "and (1,1) is a pivot in the identity"
    );
}

#[test]
fn test_the_row_operations_reject_an_out_of_range_row() {
    let mut a = packed(&[1, 0, 0, 1], 2, 2);
    assert!(a.swap_rows(0, 5).is_err());
    assert!(a.scale_row(5, &Gf2::ONE, 0).is_err());
    assert!(a.axpy_rows(5, 0, &Gf2::ONE, 0).is_err());
}

#[test]
fn test_set_clears_as_well_as_sets() {
    let mut a: PackedGf2<u8> = PackedGf2::zeros(2, 2);
    a.set(0, 1, Gf2::ONE).unwrap();
    assert_eq!(a.get(0, 1).unwrap(), Gf2::ONE);
    a.set(0, 1, Gf2::ZERO).unwrap();
    assert_eq!(a.get(0, 1).unwrap(), Gf2::ZERO);
}
