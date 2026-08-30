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
    LinearError, LinearErrorEnum, MatrixBuild, MatrixView, PackedGf2, PackedGf2Vector,
    image_basis_gf2, kernel_basis_gf2, rank_gf2,
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

// ------------------------------------------------------------------ basis orientation
//
// `kernel_basis_gf2` allocates `zeros(cols, free.len())` and sets `(f, k)` for basis vector `k`;
// `image_basis_gf2` allocates `zeros(rows, pivots.len())` and does the same. Both write basis
// vectors down **columns**. `from_row` copies a contiguous run of words, so reading a basis with it
// returns a vector whose length is the number of basis vectors rather than the dimension they live
// in.
//
// The shapes and their ranks come from `openspec/notes/archive/homology/reference/reference.py`, where each
// rank is derived from the matrix's structure rather than measured. That script imports nothing
// from this workspace.
//
// Suite audit, run before the implementation existed
// --------------------------------------------------
// No circularity. The vector under test comes out of `from_column`; the rows it is paired against
// come out of `from_row`, and the pairing is `inner`. Three separate routines, so an orientation
// error in one is not cancelled by the same error in another. No expected value was read off a run
// of `kernel_basis_gf2`.
//
// Where the annihilation assertion is vacuous, and what covers it. `A · v = 0` holds for every `v`
// when `A` is the zero matrix, and the loop body does not run at all when the nullity is zero. Of
// the 28 shapes, 12 exercise it with a non-zero `A` and a non-empty kernel: `all_ones`, `wide` and
// `circulant`, at each of the four sizes. The remaining shapes are carried by the two assertions
// that are never vacuous — the basis dimensions, and the entrywise agreement in
// `test_from_column_agrees_with_entrywise_reads`.
//
// Range rather than a point. Sizes are 63, 64, 65 and 129, so a word index and a bit index cannot
// be confused, and the last word is partly padding in half of them. Shapes are square, wide and
// tall, so a `rows`/`cols` swap cannot pass: a wide matrix has a kernel basis of different height
// and width, and `test_reading_a_kernel_basis_as_rows_gives_the_wrong_length` pins both numbers.

/// (name, rows, cols, rank over 𝔽₂, nullity over 𝔽₂, rank over ℚ).
const GF2_SHAPES: &[(&str, usize, usize, usize, usize, usize)] = &[
    ("identity_63", 63, 63, 63, 0, 63),
    ("zero_63x63", 63, 63, 0, 63, 0),
    ("all_ones_63x63", 63, 63, 1, 62, 1),
    ("upper_unitriangular_63", 63, 63, 63, 0, 63),
    ("wide_63x126", 63, 126, 63, 63, 63),
    ("tall_126x63", 126, 63, 63, 0, 63),
    ("identity_64", 64, 64, 64, 0, 64),
    ("zero_64x64", 64, 64, 0, 64, 0),
    ("all_ones_64x64", 64, 64, 1, 63, 1),
    ("upper_unitriangular_64", 64, 64, 64, 0, 64),
    ("wide_64x128", 64, 128, 64, 64, 64),
    ("tall_128x64", 128, 64, 64, 0, 64),
    ("identity_65", 65, 65, 65, 0, 65),
    ("zero_65x65", 65, 65, 0, 65, 0),
    ("all_ones_65x65", 65, 65, 1, 64, 1),
    ("upper_unitriangular_65", 65, 65, 65, 0, 65),
    ("wide_65x130", 65, 130, 65, 65, 65),
    ("tall_130x65", 130, 65, 65, 0, 65),
    ("identity_129", 129, 129, 129, 0, 129),
    ("zero_129x129", 129, 129, 0, 129, 0),
    ("all_ones_129x129", 129, 129, 1, 128, 1),
    ("upper_unitriangular_129", 129, 129, 129, 0, 129),
    ("wide_129x258", 129, 258, 129, 129, 129),
    ("tall_258x129", 258, 129, 129, 0, 129),
    ("circulant_63", 63, 63, 62, 1, 63),
    ("circulant_64", 64, 64, 63, 1, 63),
    ("circulant_65", 65, 65, 64, 1, 65),
    ("circulant_129", 129, 129, 128, 1, 129),
];

/// Rebuilds a reference shape from the same closed-form description the script uses, so no packed
/// data crosses the language boundary.
fn shape(name: &str, rows: usize, cols: usize) -> PackedGf2<u64> {
    let mut m: PackedGf2<u64> = PackedGf2::zeros(rows, cols);
    let on = |m: &mut PackedGf2<u64>, i: usize, j: usize| m.set(i, j, Gf2::ONE).unwrap();
    let base = name.split(['_', 'x']).next().unwrap();
    match base {
        "identity" => (0..rows).for_each(|i| on(&mut m, i, i)),
        "zero" => {}
        "all" => (0..rows).for_each(|i| (0..cols).for_each(|j| on(&mut m, i, j))),
        "upper" => (0..rows).for_each(|i| (i..cols).for_each(|j| on(&mut m, i, j))),
        "wide" => (0..rows).for_each(|i| {
            (0..cols)
                .filter(|j| j % rows == i)
                .for_each(|j| on(&mut m, i, j))
        }),
        "tall" => (0..rows).for_each(|i| on(&mut m, i, i % cols)),
        "circulant" => (0..rows).for_each(|i| {
            on(&mut m, i, i);
            on(&mut m, i, (i + 1) % cols);
        }),
        other => panic!("unknown reference shape {other}"),
    }
    m
}

/// `A · v`, computed row by row through `inner`.
///
/// Deliberately a different routine from the one under test: the vector comes out of
/// `from_column`, and the rows it is paired against come out of `from_row`. An orientation bug in
/// one is not cancelled by the same bug in the other.
fn a_times_v(a: &PackedGf2<u64>, v: &V, rows: usize) -> bool {
    (0..rows).all(|i| {
        V::from_row(a, i)
            .unwrap()
            .inner(v)
            .map(|x| x == Gf2::ZERO)
            .unwrap_or(false)
    })
}

/// Every kernel basis vector is a column, has the matrix's column count as its length, and is
/// annihilated by the matrix.
#[test]
fn test_kernel_basis_vectors_are_columns_that_annihilate() {
    for &(name, rows, cols, _rank, nullity, _) in GF2_SHAPES {
        let a = shape(name, rows, cols);
        let basis = kernel_basis_gf2(&a).unwrap();

        assert_eq!(
            MatrixView::cols(&basis),
            nullity,
            "{name}: kernel basis should have one column per free variable"
        );
        assert_eq!(
            MatrixView::rows(&basis),
            cols,
            "{name}: each kernel vector lives in the domain, of dimension cols"
        );

        for k in 0..nullity {
            let vk = V::from_column(&basis, k).unwrap();
            assert_eq!(
                vk.len(),
                cols,
                "{name}: kernel vector {k} has the wrong length"
            );
            assert!(a_times_v(&a, &vk, rows), "{name}: A · v_{k} is not zero");
        }
    }
}

/// The kernel basis is independent, so its rank is its column count. Without this a routine
/// returning `nullity` copies of the zero vector would pass the annihilation test.
#[test]
fn test_kernel_basis_is_independent() {
    for &(name, rows, cols, _, nullity, _) in GF2_SHAPES {
        let a = shape(name, rows, cols);
        let basis = kernel_basis_gf2(&a).unwrap();
        assert_eq!(
            rank_gf2(&basis).unwrap(),
            nullity,
            "{name}: the kernel basis is not independent"
        );
    }
}

/// Every image basis vector is a column of length equal to the matrix's row count, and the basis
/// has one vector per pivot.
#[test]
fn test_image_basis_vectors_are_columns_in_the_codomain() {
    for &(name, rows, cols, rank, _, _) in GF2_SHAPES {
        let a = shape(name, rows, cols);
        let basis = image_basis_gf2(&a).unwrap();

        assert_eq!(
            MatrixView::cols(&basis),
            rank,
            "{name}: one image vector per pivot"
        );
        assert_eq!(
            MatrixView::rows(&basis),
            rows,
            "{name}: each image vector lives in the codomain, of dimension rows"
        );
        assert_eq!(
            rank_gf2(&basis).unwrap(),
            rank,
            "{name}: image basis is not independent"
        );

        for k in 0..rank {
            assert_eq!(
                V::from_column(&basis, k).unwrap().len(),
                rows,
                "{name}: image vector {k} has the wrong length"
            );
        }
    }
}

/// The rank the elimination reaches is the rank the reference derives in closed form.
#[test]
fn test_rank_matches_the_closed_form_reference() {
    for &(name, rows, cols, rank, nullity, _) in GF2_SHAPES {
        let a = shape(name, rows, cols);
        assert_eq!(rank_gf2(&a).unwrap(), rank, "{name}: rank over 𝔽₂");
        assert_eq!(
            rank + nullity,
            cols,
            "{name}: rank–nullity over the column count"
        );
    }
}

/// The reference set separates the two coefficient fields.
///
/// An odd circulant is invertible over ℚ and singular over 𝔽₂, because its determinant is
/// `1 − (−1)ⁿ = 2`. The even one is singular over both, and is kept next to them so the suite
/// cannot mistake "no shape separates the fields" for "the fields agree".
#[test]
fn test_the_reference_set_separates_the_coefficient_fields() {
    let separating = GF2_SHAPES
        .iter()
        .filter(|(_, _, _, r2, _, rq)| r2 != rq)
        .count();
    assert!(
        separating >= 3,
        "the reference set no longer distinguishes 𝔽₂ from ℚ; the field parameter is untested"
    );
    for &(name, rows, cols, rank, _, rank_q) in GF2_SHAPES {
        if rank != rank_q {
            let a = shape(name, rows, cols);
            assert_eq!(
                rank_gf2(&a).unwrap(),
                rank,
                "{name}: 𝔽₂ rank should be below the ℚ rank of {rank_q}"
            );
        }
    }
}

/// Reading a basis through the row constructor gives the wrong length.
///
/// This is the defect four docstrings described. It is asserted rather than described so that a
/// future change of orientation cannot pass silently.
#[test]
fn test_reading_a_kernel_basis_as_rows_gives_the_wrong_length() {
    // 63 columns, 63 free variables, so the basis is 63×63 and rows and columns have equal length.
    // Pick a shape where they differ: `wide_64x128` has a 128×64 kernel basis.
    let a = shape("wide_64x128", 64, 128);
    let basis = kernel_basis_gf2(&a).unwrap();
    assert_eq!(
        (MatrixView::rows(&basis), MatrixView::cols(&basis)),
        (128, 64)
    );

    let by_column = V::from_column(&basis, 0).unwrap();
    let by_row = V::from_row(&basis, 0).unwrap();

    assert_eq!(
        by_column.len(),
        128,
        "a kernel vector lives in the 128-dimensional domain"
    );
    assert_eq!(
        by_row.len(),
        64,
        "a row of the basis is 64 wide, which is not a kernel vector"
    );
    assert!(a_times_v(&a, &by_column, 64), "the column is annihilated");
}

/// A column out of bounds is refused rather than read.
#[test]
fn test_from_column_rejects_an_out_of_range_index() {
    let m: PackedGf2<u64> = PackedGf2::zeros(130, 3);
    assert!(V::from_column(&m, 2).is_ok());
    assert!(matches!(
        V::from_column(&m, 3),
        Err(LinearError(LinearErrorEnum::IndexOutOfBounds { .. }))
    ));
}

/// A column read back entry by entry agrees with `get` on the matrix, at every index.
#[test]
fn test_from_column_agrees_with_entrywise_reads() {
    let mut m: PackedGf2<u64> = PackedGf2::zeros(130, 5);
    for i in 0..130 {
        for j in 0..5 {
            if (i * 7 + j * 3) % 5 == 0 {
                m.set(i, j, Gf2::ONE).unwrap();
            }
        }
    }
    for j in 0..5 {
        let col = V::from_column(&m, j).unwrap();
        assert_eq!(col.len(), 130);
        for i in 0..130 {
            assert_eq!(
                col.get(i).unwrap(),
                m.get(i, j).unwrap(),
                "entry ({i}, {j})"
            );
        }
    }
}
