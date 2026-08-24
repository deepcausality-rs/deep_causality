/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_linear::utils_tests::fixtures_matrix::*;
use deep_causality_linear::{LinearError, MatrixBuild, MatrixView, PackedGf2};
use deep_causality_num::Gf2;

#[test]
fn test_bits_per_word_matches_the_word_type() {
    assert_eq!(PackedGf2::<u8>::bits_per_word(), 8);
    assert_eq!(PackedGf2::<u16>::bits_per_word(), 16);
    assert_eq!(PackedGf2::<u32>::bits_per_word(), 32);
    assert_eq!(PackedGf2::<u64>::bits_per_word(), 64);
}

#[test]
fn test_allocation_matches_the_bit_count() {
    // r * ceil(c / w) words, which is the storage claim the packing rests on.
    let m: PackedGf2<u8> = PackedGf2::zeros(3, 20);
    assert_eq!(m.words_per_row(), 3, "ceil(20 / 8)");
    assert_eq!(m.as_words().len(), 9, "3 rows * 3 words");
}

#[test]
fn test_a_column_count_that_is_not_a_multiple_of_the_word_width() {
    let m: PackedGf2<u8> = PackedGf2::zeros(1, 5);
    assert_eq!(m.words_per_row(), 1);
    assert_eq!(m.cols(), 5);
    // The three trailing bits exist in the word and must read as outside the shape.
    assert!(m.get(0, 5).is_err());
}

#[test]
fn test_the_trailing_padding_bits_stay_zero() {
    let mut m: PackedGf2<u8> = PackedGf2::zeros(1, 5);
    for j in 0..5 {
        m.set(0, j, Gf2::ONE).unwrap();
    }
    // Only the low five bits may be set; the padding must not leak into a whole-word update.
    assert_eq!(m.as_words()[0], 0b0001_1111);
}

#[test]
fn test_from_slice_round_trips() {
    let data = [Gf2::ONE, Gf2::ZERO, Gf2::ONE, Gf2::ZERO];
    let m: PackedGf2<u64> = PackedGf2::from_slice(&data, 2, 2).unwrap();
    assert_eq!(m.get(0, 0).unwrap(), Gf2::ONE);
    assert_eq!(m.get(0, 1).unwrap(), Gf2::ZERO);
    assert_eq!(m.get(1, 0).unwrap(), Gf2::ONE);
}

#[test]
fn test_from_slice_rejects_a_buffer_that_does_not_match_the_shape() {
    let data = [Gf2::ONE; 3];
    let e = PackedGf2::<u64>::from_slice(&data, 2, 2).unwrap_err();
    assert!(matches!(e, LinearError::ShapeMismatch { .. }), "got {e:?}");
}

#[test]
fn test_the_boundary_operator_alphabet_reduces_mod_two() {
    // -1 and 1 are both the F2 one; 0 is zero.
    let (d, r, c) = boundary_alphabet_3x3();
    let m: PackedGf2<u64> = PackedGf2::from_i64_mod2(&d, r, c).unwrap();
    for i in 0..r {
        for j in 0..c {
            let expected = if d[i * c + j] % 2 == 0 {
                Gf2::ZERO
            } else {
                Gf2::ONE
            };
            assert_eq!(m.get(i, j).unwrap(), expected, "at ({i}, {j})");
        }
    }
    assert_eq!(m.get(0, 1).unwrap(), Gf2::ONE, "-1 must map to the F2 one");
}

#[test]
fn test_the_same_matrix_at_two_word_widths_reads_identically() {
    let (d, r, c) = ranks_disagree_3x3();
    let narrow: PackedGf2<u8> = PackedGf2::from_i64_mod2(&d, r, c).unwrap();
    let wide: PackedGf2<u64> = PackedGf2::from_i64_mod2(&d, r, c).unwrap();
    for i in 0..r {
        for j in 0..c {
            assert_eq!(
                narrow.get(i, j).unwrap(),
                wide.get(i, j).unwrap(),
                "at ({i}, {j})"
            );
        }
    }
}

#[test]
fn test_out_of_shape_access_is_rejected() {
    let m: PackedGf2<u64> = PackedGf2::zeros(2, 2);
    assert!(m.get(2, 0).is_err());
    assert!(m.get(0, 2).is_err());
}

#[test]
fn test_identity_over_gf2() {
    let m: PackedGf2<u8> = PackedGf2::identity(3);
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { Gf2::ONE } else { Gf2::ZERO };
            assert_eq!(m.get(i, j).unwrap(), expected);
        }
    }
}

#[test]
fn test_empty_shapes() {
    let m: PackedGf2<u64> = PackedGf2::zeros(0, 0);
    assert!(m.is_empty());
    assert_eq!(m.as_words().len(), 0);
}
