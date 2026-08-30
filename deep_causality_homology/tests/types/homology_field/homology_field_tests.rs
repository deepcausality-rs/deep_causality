/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The coefficient field, and the rank it produces.
//!
//! Ranks come from `openspec/notes/homology/reference/reference.py`, where each is derived from the
//! matrix's structure rather than measured.

use deep_causality_homology::HomologyField;
use deep_causality_linear::CsrMatrix;

/// A circulant with ones at `(i, i)` and `(i, i+1 mod n)` has determinant `1 − (−1)ⁿ`.
///
/// For odd `n` that is 2: invertible over ℚ, singular over 𝔽₂. This is the smallest matrix family
/// where the field argument changes the rank, and an implementation ignoring the field gets one of
/// the two wrong.
fn circulant(n: usize) -> CsrMatrix<i8> {
    let t: Vec<(usize, usize, i8)> = (0..n)
        .flat_map(|i| [(i, i, 1i8), (i, (i + 1) % n, 1i8)])
        .collect();
    CsrMatrix::from_triplets(n, n, &t).unwrap()
}

#[test]
fn test_the_field_changes_the_rank_of_an_odd_circulant() {
    for n in [3usize, 5, 63, 65] {
        let m = circulant(n);
        assert_eq!(
            HomologyField::Rational.rank_of(&m).unwrap(),
            n,
            "odd circulant of size {n} is invertible over ℚ"
        );
        assert_eq!(
            HomologyField::Gf2.rank_of(&m).unwrap(),
            n - 1,
            "odd circulant of size {n} is singular over 𝔽₂"
        );
    }
}

#[test]
fn test_an_even_circulant_is_singular_over_both_fields() {
    // Kept beside the odd cases so the suite cannot mistake "this shape does not separate the
    // fields" for "the fields agree".
    for n in [4usize, 64] {
        let m = circulant(n);
        assert_eq!(HomologyField::Rational.rank_of(&m).unwrap(), n - 1);
        assert_eq!(HomologyField::Gf2.rank_of(&m).unwrap(), n - 1);
    }
}

/// A negative incidence number is `1` over 𝔽₂ and stays `−1` over ℚ.
#[test]
fn test_negative_entries_reduce_mod_two_but_survive_over_the_rationals() {
    // The boundary of a triangle: rank 2 over both, but the sign matters to the ℚ path.
    let d = CsrMatrix::from_triplets(
        3,
        3,
        &[
            (0, 0, -1i8),
            (1, 0, 1),
            (1, 1, -1),
            (2, 1, 1),
            (0, 2, -1),
            (2, 2, 1),
        ],
    )
    .unwrap();
    assert_eq!(HomologyField::Rational.rank_of(&d).unwrap(), 2);
    assert_eq!(HomologyField::Gf2.rank_of(&d).unwrap(), 2);

    // Dropping the signs changes the ℚ rank and leaves the 𝔽₂ rank alone, which is why the two
    // paths cannot share a conversion.
    let unsigned = CsrMatrix::from_triplets(
        3,
        3,
        &[
            (0, 0, 1i8),
            (1, 0, 1),
            (1, 1, 1),
            (2, 1, 1),
            (0, 2, 1),
            (2, 2, 1),
        ],
    )
    .unwrap();
    assert_eq!(HomologyField::Rational.rank_of(&unsigned).unwrap(), 3);
    assert_eq!(HomologyField::Gf2.rank_of(&unsigned).unwrap(), 2);
}

/// A degenerate shape has rank zero over either field, with no elimination run.
#[test]
fn test_a_degenerate_shape_has_rank_zero() {
    for (r, c) in [(0usize, 5usize), (5, 0), (0, 0)] {
        let m: CsrMatrix<i8> = CsrMatrix::from_triplets(r, c, &[]).unwrap();
        assert_eq!(m.shape(), (r, c));
        assert_eq!(HomologyField::Rational.rank_of(&m).unwrap(), 0);
        assert_eq!(HomologyField::Gf2.rank_of(&m).unwrap(), 0);
    }
}

/// The enum is a plain value: copyable, comparable, and hashable, so a caller can key on it.
#[test]
fn test_the_field_is_a_plain_value() {
    let a = HomologyField::Rational;
    let b = a;
    assert_eq!(a, b);
    assert_ne!(HomologyField::Rational, HomologyField::Gf2);
}
