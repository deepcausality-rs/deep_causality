/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The two Stirling triangles, against published values and against each other.

use deep_causality_num::{stirling_first_unsigned, stirling_second};

fn s2(n: usize, k: usize) -> u128 {
    let mut buf = [0u128; 64];
    stirling_second(n, k, &mut buf).expect("representable")
}

fn s1(n: usize, k: usize) -> u128 {
    let mut buf = [0u128; 64];
    stirling_first_unsigned(n, k, &mut buf).expect("representable")
}

/// Rows 0 to 5 of the second-kind triangle, from OEIS A008277.
#[test]
fn test_the_second_kind_triangle_matches_its_published_rows() {
    let rows: [&[u128]; 6] = [
        &[1],
        &[0, 1],
        &[0, 1, 1],
        &[0, 1, 3, 1],
        &[0, 1, 7, 6, 1],
        &[0, 1, 15, 25, 10, 1],
    ];
    for (n, row) in rows.iter().enumerate() {
        for (k, want) in row.iter().enumerate() {
            assert_eq!(s2(n, k), *want, "S({n}, {k})");
        }
    }
}

/// Rows 0 to 5 of the unsigned first-kind triangle, from OEIS A132393.
#[test]
fn test_the_first_kind_triangle_matches_its_published_rows() {
    let rows: [&[u128]; 6] = [
        &[1],
        &[0, 1],
        &[0, 1, 1],
        &[0, 2, 3, 1],
        &[0, 6, 11, 6, 1],
        &[0, 24, 50, 35, 10, 1],
    ];
    for (n, row) in rows.iter().enumerate() {
        for (k, want) in row.iter().enumerate() {
            assert_eq!(s1(n, k), *want, "c({n}, {k})");
        }
    }
}

/// Each row of the second kind sums to a Bell number, and each row of the first
/// kind to a factorial.
///
/// Two whole-row identities, so an error at one entry that the tabulated rows
/// above happen not to reach still shows up here.
#[test]
fn test_the_rows_sum_to_the_bell_numbers_and_the_factorials() {
    let bell = [1u128, 1, 2, 5, 15, 52, 203, 877, 4140, 21147];
    let mut fact = 1u128;
    for (n, want) in bell.iter().enumerate() {
        let sum2: u128 = (0..=n).map(|k| s2(n, k)).sum();
        assert_eq!(sum2, *want, "Σ_k S({n}, k) must be Bell({n})");

        let sum1: u128 = (0..=n).map(|k| s1(n, k)).sum();
        assert_eq!(sum1, fact, "Σ_k c({n}, k) must be {n}!");
        fact *= (n + 1) as u128;
    }
}

/// Two closed forms along the edges of the triangles.
///
/// `S(n, 2) = 2^{n−1} − 1`, `c(n, 1) = (n−1)!`, and both diagonals are 1. The
/// diagonal is what a wrong multiplier leaves alone, and the second column is
/// what it does not.
#[test]
fn test_the_edges_of_both_triangles_match_their_closed_forms() {
    let mut fact = 1u128;
    for n in 1..=15 {
        assert_eq!(s2(n, n), 1, "S({n}, {n})");
        assert_eq!(s1(n, n), 1, "c({n}, {n})");
        assert_eq!(s2(n, 1), 1, "S({n}, 1)");
        assert_eq!(s1(n, 1), fact, "c({n}, 1) must be {}!", n - 1);
        if n >= 2 {
            assert_eq!(s2(n, 2), (1u128 << (n - 1)) - 1, "S({n}, 2)");
        }
        fact *= n as u128;
    }
}

/// The two triangles are inverse to each other, up to the alternating sign.
///
/// `Σ_k (−1)^{n−k}·c(n,k)·S(k,m) = δ_{n,m}`. This is the identity that makes
/// Haruna's A.12 and A.14 a matched pair: one expands a power of `a(γ)` over the
/// elementary products, the other inverts it, and they can only be inverses if
/// this holds. It also pins the sign convention — the unsigned first-kind
/// numbers with the sign written out, which is what A.14 uses.
#[test]
fn test_the_two_triangles_invert_each_other() {
    for n in 0..=9usize {
        for m in 0..=n {
            let mut acc: i128 = 0;
            for k in m..=n {
                let term = (s1(n, k) as i128) * (s2(k, m) as i128);
                acc += if (n - k) % 2 == 0 { term } else { -term };
            }
            let want = i128::from(n == m);
            assert_eq!(acc, want, "Σ_k (−1)^(n−k) c({n},k) S(k,{m})");
        }
    }
}

/// Off the triangle, and at the corner.
#[test]
fn test_the_boundary_values() {
    assert_eq!(s2(0, 0), 1, "the empty set has one partition into no parts");
    assert_eq!(s1(0, 0), 1);
    for n in 1..=6 {
        assert_eq!(s2(n, 0), 0, "S({n}, 0)");
        assert_eq!(s1(n, 0), 0, "c({n}, 0)");
        assert_eq!(s2(n, n + 1), 0, "S({n}, {})", n + 1);
        assert_eq!(s1(n, n + 3), 0, "c({n}, {})", n + 3);
    }
}

/// A scratch row shorter than `k + 1` is refused rather than truncated.
#[test]
fn test_a_short_scratch_row_is_refused() {
    let mut buf = [0u64; 3];
    assert_eq!(stirling_second::<u64>(9, 4, &mut buf), None);
    assert_eq!(stirling_first_unsigned::<u64>(9, 4, &mut buf), None);
    // Exactly k + 1 is enough.
    let mut ok = [0u64; 5];
    assert_eq!(stirling_second::<u64>(9, 4, &mut ok), Some(7770));
}

/// Overflow is reported, not wrapped.
///
/// `c(n, 1) = (n−1)!` leaves `u64` at `n = 22`, and `S(n, 2) = 2^{n−1} − 1`
/// leaves it at `n = 66`. Both are checked at the last value that fits and the
/// first that does not.
#[test]
fn test_overflow_is_reported_rather_than_wrapped() {
    let mut buf = [0u64; 8];
    assert_eq!(
        stirling_first_unsigned::<u64>(21, 1, &mut buf),
        Some(2432902008176640000)
    );
    assert_eq!(stirling_first_unsigned::<u64>(22, 1, &mut buf), None);

    assert_eq!(
        stirling_second::<u64>(64, 2, &mut buf),
        Some((1u64 << 63) - 1)
    );
    assert_eq!(stirling_second::<u64>(66, 2, &mut buf), None);
}
