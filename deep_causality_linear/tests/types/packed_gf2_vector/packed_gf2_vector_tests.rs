/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The bit-packed 𝔽₂ vector.
//!
//! Two things shape these tests. Every length here crosses a word boundary, because a vector that
//! fits in one word cannot tell a word index apart from a bit index. And the padding bits past
//! `len` in the last word are checked directly, because `weight` and `inner` count whole words and
//! would count padding as data.

use deep_causality_linear::{
    LinearError, LinearErrorEnum, MatrixBuild, PackedGf2, PackedGf2Vector,
};
use deep_causality_num::Gf2;

type V = PackedGf2Vector<u64>;

fn v(len: usize, support: &[usize]) -> V {
    V::from_support(len, support).unwrap()
}

/// A support round-trips through the bits and back.
#[test]
fn test_a_support_round_trips() {
    let s = [0usize, 1, 63, 64, 65, 127, 128, 199];
    let x = v(200, &s);
    assert_eq!(x.len(), 200);
    assert_eq!(x.weight(), s.len());
    assert_eq!(x.support().collect::<Vec<_>>(), s.to_vec());
    for i in 0..200 {
        let want = Gf2::new(s.contains(&i));
        assert_eq!(x.get(i).unwrap(), want, "entry {i}");
    }
}

/// A repeated index cancels, because addition in 𝔽₂ is exclusive or.
///
/// This makes `from_support` the same function as summing the basis vectors named, which is what a
/// caller building a chain from a list of cells wants.
#[test]
fn test_a_repeated_index_cancels() {
    let x = v(100, &[7, 70, 7]);
    assert_eq!(x.support().collect::<Vec<_>>(), vec![70]);
    assert_eq!(x.weight(), 1);

    let thrice = v(100, &[7, 7, 7]);
    assert_eq!(thrice.support().collect::<Vec<_>>(), vec![7]);
}

/// The padding past `len` is not data.
///
/// A 130-bit vector occupies three 64-bit words, so 62 bits of the last word are padding. Setting
/// every entry must give weight 130 and not 192.
#[test]
fn test_the_padding_bits_are_not_counted() {
    let all: Vec<usize> = (0..130).collect();
    let x = v(130, &all);
    assert_eq!(x.as_words().len(), 3, "130 bits is three 64-bit words");
    assert_eq!(x.weight(), 130, "the 62 padding bits must not be counted");
    assert_eq!(x.support().count(), 130);

    // The same through `set`, which is the other way in.
    let mut y = V::zeros(130);
    for i in 0..130 {
        y.set(i, Gf2::ONE).unwrap();
    }
    assert_eq!(y.weight(), 130);
    assert_eq!(x, y);

    // And clearing one entry is visible.
    y.set(129, Gf2::ZERO).unwrap();
    assert_eq!(y.weight(), 129);
}

/// Addition is exclusive or; intersection is conjunction. They are different functions.
#[test]
fn test_addition_and_intersection_differ_where_the_supports_meet() {
    let a = v(140, &[1, 5, 70, 100, 139]);
    let b = v(140, &[5, 70, 101]);

    let sum = a.add(&b).unwrap();
    assert_eq!(
        sum.support().collect::<Vec<_>>(),
        vec![1, 100, 101, 139],
        "5 and 70 are in both and must cancel"
    );

    let meet = a.intersect(&b).unwrap();
    assert_eq!(
        meet.support().collect::<Vec<_>>(),
        vec![5, 70],
        "5 and 70 are in both and must survive"
    );

    // Addition is its own inverse over 𝔽₂.
    assert_eq!(sum.add(&b).unwrap(), a);
}

/// `⟨γ₁, γ₂⟩` is the parity of the intersection, checked against the entrywise definition.
///
/// The word-parallel form never builds the intersection, so the reference here sums
/// `γ₁ⁱ · γ₂ⁱ` one entry at a time.
#[test]
fn test_the_pairing_matches_its_entrywise_definition() {
    let cases: [(&[usize], &[usize]); 5] = [
        (&[], &[]),
        (&[3], &[3]),
        (&[3, 70], &[3, 70]),
        (&[1, 5, 70, 100, 139], &[5, 70, 101]),
        (&[0, 63, 64, 127, 128], &[63, 64, 128]),
    ];
    for (sa, sb) in cases {
        let a = v(140, sa);
        let b = v(140, sb);
        let mut ones = 0usize;
        for i in 0..140 {
            if a.get(i).unwrap().bit() && b.get(i).unwrap().bit() {
                ones += 1;
            }
        }
        assert_eq!(
            a.inner(&b).unwrap(),
            Gf2::new(ones % 2 == 1),
            "⟨{sa:?}, {sb:?}⟩ with {ones} shared entries"
        );
        assert_eq!(a.inner(&b).unwrap(), b.inner(&a).unwrap(), "symmetry");
        assert_eq!(
            a.inner(&b).unwrap(),
            Gf2::new(a.intersect(&b).unwrap().weight() % 2 == 1),
            "must agree with the weight of the intersection"
        );
    }
}

/// The pairs and triples of the support, in count and in content.
#[test]
fn test_the_support_pairs_and_triples() {
    let x = v(200, &[2, 64, 130, 199]);
    let pairs: Vec<_> = x.support_pairs().collect();
    assert_eq!(
        pairs,
        vec![
            (2, 64),
            (2, 130),
            (2, 199),
            (64, 130),
            (64, 199),
            (130, 199)
        ]
    );
    assert_eq!(pairs.len(), 4 * 3 / 2);

    let triples: Vec<_> = x.support_triples().collect();
    assert_eq!(
        triples,
        vec![(2, 64, 130), (2, 64, 199), (2, 130, 199), (64, 130, 199)]
    );
    assert_eq!(triples.len(), 4 * 3 * 2 / 6);

    // Strictly ascending within each tuple, and the counts follow C(w, k).
    for w in 0..=6usize {
        let y = v(300, &(0..w).map(|i| i * 37).collect::<Vec<_>>());
        assert_eq!(y.support_pairs().count(), w * w.saturating_sub(1) / 2);
        assert_eq!(
            y.support_triples().count(),
            w * w.saturating_sub(1) * w.saturating_sub(2) / 6
        );
        assert!(y.support_pairs().all(|(a, b)| a < b));
        assert!(y.support_triples().all(|(a, b, c)| a < b && b < c));
    }
}

/// A row of a packed matrix becomes a vector.
///
/// This is how a homology generator gets out of `kernel_basis_gf2`, whose basis is the rows of a
/// [`PackedGf2`].
#[test]
fn test_a_matrix_row_becomes_a_vector() {
    let mut m: PackedGf2<u64> = PackedGf2::zeros(3, 130);
    for (r, cols) in [(0usize, vec![0usize, 129]), (1, vec![64]), (2, vec![])].into_iter() {
        for c in cols {
            m.set(r, c, Gf2::ONE).unwrap();
        }
    }
    assert_eq!(
        V::from_row(&m, 0).unwrap().support().collect::<Vec<_>>(),
        vec![0, 129]
    );
    assert_eq!(
        V::from_row(&m, 1).unwrap().support().collect::<Vec<_>>(),
        vec![64]
    );
    assert!(V::from_row(&m, 2).unwrap().is_zero());
    assert_eq!(V::from_row(&m, 0).unwrap().len(), 130);

    assert!(matches!(
        V::from_row(&m, 3),
        Err(LinearError(LinearErrorEnum::IndexOutOfBounds { .. }))
    ));
}

/// An empty vector is not the zero vector, and both are handled.
#[test]
fn test_the_empty_and_zero_vectors() {
    let e = V::zeros(0);
    assert!(e.is_empty());
    assert!(e.is_zero(), "vacuously");
    assert_eq!(e.weight(), 0);
    assert_eq!(e.support().count(), 0);
    assert_eq!(e.inner(&e).unwrap(), Gf2::ZERO);

    let z = V::zeros(130);
    assert!(!z.is_empty());
    assert!(z.is_zero());
    assert_eq!(z.support_pairs().count(), 0);
}

/// Out-of-range indices and mismatched lengths are refused.
#[test]
fn test_the_bounds_and_shape_checks() {
    let mut a = V::zeros(10);
    assert!(matches!(
        a.get(10),
        Err(LinearError(LinearErrorEnum::IndexOutOfBounds { .. }))
    ));
    assert!(matches!(
        a.set(10, Gf2::ONE),
        Err(LinearError(LinearErrorEnum::IndexOutOfBounds { .. }))
    ));
    assert!(matches!(
        V::from_support(10, &[3, 10]),
        Err(LinearError(LinearErrorEnum::IndexOutOfBounds { .. }))
    ));

    let b = V::zeros(11);
    assert!(matches!(
        a.add(&b),
        Err(LinearError(LinearErrorEnum::ShapeMismatch { .. }))
    ));
    assert!(matches!(
        a.intersect(&b),
        Err(LinearError(LinearErrorEnum::ShapeMismatch { .. }))
    ));
    assert!(matches!(
        a.inner(&b),
        Err(LinearError(LinearErrorEnum::ShapeMismatch { .. }))
    ));
}
