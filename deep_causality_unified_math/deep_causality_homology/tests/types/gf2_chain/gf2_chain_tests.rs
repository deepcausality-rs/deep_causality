/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The mod-2 chain.
//!
//! The bit arithmetic belongs to `PackedGf2Vector` and is tested where it lives. What is tested
//! here is the part this type adds: that the degree travels with the data, and that an operation
//! between two degrees is refused rather than computed.

use deep_causality_homology::{Gf2Chain, HomologyError, HomologyErrorEnum};
use deep_causality_linear::{MatrixBuild, PackedGf2};
use deep_causality_num::Gf2;

type C = Gf2Chain<u64>;

fn c(len: usize, degree: usize, support: &[usize]) -> C {
    C::from_support(len, degree, support).unwrap()
}

/// The degree is carried, and the data with it.
#[test]
fn test_a_chain_carries_its_degree() {
    let x = c(100, 1, &[3, 70, 99]);
    assert_eq!(x.degree(), 1);
    assert_eq!(x.len(), 100, "the cell count, not the weight");
    assert_eq!(x.weight(), 3);
    assert_eq!(x.support().collect::<Vec<_>>(), vec![3, 70, 99]);
    assert!(!x.is_zero());
    assert!(!x.is_empty());

    let z = C::zeros(100, 2);
    assert_eq!(z.degree(), 2);
    assert!(z.is_zero());
    assert!(!z.is_empty(), "a zero chain still ranges over 100 cells");
}

/// Two chains of the same degree add, intersect and pair.
///
/// The arithmetic itself is the vector's; what this checks is that the degree survives the
/// operation, since a sum of two `1`-chains is a `1`-chain.
#[test]
fn test_operations_between_equal_degrees_keep_the_degree() {
    let a = c(140, 1, &[1, 5, 70, 139]);
    let b = c(140, 1, &[5, 70, 101]);

    let sum = a.add(&b).unwrap();
    assert_eq!(sum.degree(), 1);
    assert_eq!(sum.support().collect::<Vec<_>>(), vec![1, 101, 139]);

    let meet = a.intersect(&b).unwrap();
    assert_eq!(meet.degree(), 1);
    assert_eq!(meet.support().collect::<Vec<_>>(), vec![5, 70]);

    assert_eq!(a.inner(&b).unwrap(), Gf2::ZERO, "two shared cells is even");
    let d = c(140, 1, &[5]);
    assert_eq!(a.inner(&d).unwrap(), Gf2::ONE, "one shared cell is odd");
}

/// An operation between two degrees is refused.
///
/// This is what the type is for. A `1`-chain and a `2`-chain have no sum, no intersection and no
/// pairing, and a bare bit vector with the degree passed alongside cannot say so.
#[test]
fn test_operations_across_degrees_are_refused() {
    let one = c(140, 1, &[1, 5]);
    let two = c(140, 2, &[1, 5]);

    for r in [
        one.add(&two).err(),
        one.intersect(&two).err(),
        one.inner(&two).err(),
    ] {
        assert!(
            matches!(
                r,
                Some(HomologyError(HomologyErrorEnum::ChainGroupMismatch(_)))
            ),
            "a cross-degree operation must be refused, got {r:?}"
        );
    }

    // Identical data, different degree: not the same chain.
    assert_ne!(one, two);
}

/// Equal degrees but unequal lengths are refused too.
#[test]
fn test_operations_across_lengths_are_refused() {
    let short = c(100, 1, &[1]);
    let long = c(140, 1, &[1]);
    assert!(short.add(&long).is_err());
    assert!(short.intersect(&long).is_err());
    assert!(short.inner(&long).is_err());
}

/// A row of a packed 𝔽₂ matrix becomes a chain of a stated degree.
///
/// This reads a row of an arbitrary matrix. It is not the path a homology generator takes: a basis
/// from `kernel_basis_gf2` is stored down columns, which
/// `test_a_basis_column_becomes_a_chain` covers.
#[test]
fn test_a_basis_row_becomes_a_chain() {
    let mut m: PackedGf2<u64> = PackedGf2::zeros(2, 130);
    m.set(0, 0, Gf2::ONE).unwrap();
    m.set(0, 129, Gf2::ONE).unwrap();
    m.set(1, 64, Gf2::ONE).unwrap();

    let g0 = C::from_row(&m, 0, 1).unwrap();
    assert_eq!(g0.degree(), 1);
    assert_eq!(g0.len(), 130);
    assert_eq!(g0.support().collect::<Vec<_>>(), vec![0, 129]);

    let g1 = C::from_row(&m, 1, 1).unwrap();
    assert_eq!(g1.support().collect::<Vec<_>>(), vec![64]);
    assert_eq!(g0.inner(&g1).unwrap(), Gf2::ZERO, "disjoint supports");

    assert!(C::from_row(&m, 2, 1).is_err());
}

/// The pairs and triples reach through to the vector.
#[test]
fn test_the_support_tuples_reach_through() {
    let x = c(200, 1, &[2, 64, 130]);
    assert_eq!(
        x.support_pairs().collect::<Vec<_>>(),
        vec![(2, 64), (2, 130), (64, 130)]
    );
    assert_eq!(x.support_triples().collect::<Vec<_>>(), vec![(2, 64, 130)]);
}

/// A repeated cell cancels, because the coefficients are in 𝔽₂.
#[test]
fn test_a_repeated_cell_cancels() {
    let x = c(100, 1, &[7, 70, 7]);
    assert_eq!(x.support().collect::<Vec<_>>(), vec![70]);
    assert!(C::from_support(100, 1, &[7, 100]).is_err(), "out of range");
}

/// A basis column becomes a chain, which is the path a homology generator actually takes.
///
/// `kernel_basis_gf2` allocates `zeros(cols, free.len())` and writes vector `k` down column `k`, so
/// the generator's length is the matrix's column count. Reading it as a row gives the number of
/// generators instead, which is a different number whenever the two differ.
#[test]
fn test_a_basis_column_becomes_a_chain() {
    // 3 columns, and column 1 has a kernel vector supported on {0, 129}.
    let mut basis: PackedGf2<u64> = PackedGf2::zeros(130, 3);
    basis.set(0, 1, Gf2::ONE).unwrap();
    basis.set(129, 1, Gf2::ONE).unwrap();
    basis.set(64, 2, Gf2::ONE).unwrap();

    let g = C::from_column(&basis, 1, 1).unwrap();
    assert_eq!(g.degree(), 1);
    assert_eq!(
        g.len(),
        130,
        "a generator lives in the domain, of dimension 130"
    );
    assert_eq!(g.support().collect::<Vec<_>>(), vec![0, 129]);

    let h = C::from_column(&basis, 2, 1).unwrap();
    assert_eq!(h.support().collect::<Vec<_>>(), vec![64]);
    assert_eq!(g.inner(&h).unwrap(), Gf2::ZERO, "disjoint supports");

    // The row reading of the same basis is 3 wide, not 130. That is the defect this pins.
    assert_eq!(C::from_row(&basis, 0, 1).unwrap().len(), 3);

    assert!(C::from_column(&basis, 3, 1).is_err());
}

// -------------------------------------------------------------- the chain group is one guard
//
// `C_k` is identified by `(degree, len)`. Both halves are checked in `same_group`, so one condition
// raises one error. Before that, the degree was checked in the chain and the length in the packed
// vector underneath, and a caller saw two different error types for a single question.
//
// The table below varies the two halves independently. A test that changed degree and length
// together could not tell which guard fired.

/// Chains from different chain groups are refused, whichever half differs, with one error variant.
#[test]
fn test_operands_from_different_chain_groups_are_refused() {
    // (name, left degree, left len, right degree, right len)
    let cases: &[(&str, usize, usize, usize, usize)] = &[
        ("degree differs, length equal", 1, 130, 2, 130),
        ("degree equal, length differs", 1, 130, 1, 131),
        ("both differ", 1, 130, 2, 131),
        ("degree differs at grade zero", 0, 64, 1, 64),
        ("length differs across a word boundary", 1, 64, 1, 65),
    ];
    for &(what, dl, ll, dr, lr) in cases {
        let a: C = Gf2Chain::zeros(ll, dl);
        let b: C = Gf2Chain::zeros(lr, dr);
        for (op, result) in [
            ("add", a.add(&b).map(|_| ())),
            ("intersect", a.intersect(&b).map(|_| ())),
            ("inner", a.inner(&b).map(|_| ())),
        ] {
            assert!(
                matches!(
                    result,
                    Err(HomologyError(HomologyErrorEnum::ChainGroupMismatch(_)))
                ),
                "{what}: {op} should raise ChainGroupMismatch"
            );
        }
    }
}

/// Operands in the same chain group are accepted, so the guard is not simply refusing everything.
#[test]
fn test_operands_from_the_same_chain_group_are_accepted() {
    for (degree, len) in [(0usize, 1usize), (1, 64), (2, 65), (3, 130)] {
        let a = Gf2Chain::<u64>::from_support(len, degree, &[0]).unwrap();
        let b = Gf2Chain::<u64>::from_support(len, degree, &[len - 1]).unwrap();
        assert!(a.add(&b).is_ok(), "C_{degree} of dimension {len}: add");
        assert!(
            a.intersect(&b).is_ok(),
            "C_{degree} of dimension {len}: intersect"
        );
        assert!(a.inner(&b).is_ok(), "C_{degree} of dimension {len}: inner");
    }
}

/// The error names both chain groups, so a caller can see which half disagreed.
#[test]
fn test_the_mismatch_error_names_both_chain_groups() {
    let a: C = Gf2Chain::zeros(130, 1);
    let b: C = Gf2Chain::zeros(131, 2);
    let err = a.add(&b).unwrap_err();
    let msg = format!("{err}");
    for part in ["C_1", "130", "C_2", "131"] {
        assert!(msg.contains(part), "the message should name {part}: {msg}");
    }
}

/// A chain over no cells is empty; a chain over some cells is not, whatever its coefficients.
///
/// `is_empty` was asserted only where it is false, so a body returning a constant `false` passed
/// the whole suite. The two readings it distinguishes are "ranges over no cells" and "every
/// coefficient is zero", and conflating them is the mistake the pair below rules out.
#[test]
fn test_a_chain_over_no_cells_is_empty() {
    let nothing: C = Gf2Chain::zeros(0, 1);
    assert!(
        nothing.is_empty(),
        "C_1 of dimension 0 has no cells to range over"
    );
    assert!(
        nothing.is_zero(),
        "and it is also the zero chain, vacuously"
    );
    assert_eq!(nothing.len(), 0);
    assert_eq!(nothing.weight(), 0);
    assert_eq!(nothing.support().count(), 0);

    let zero_but_not_empty: C = Gf2Chain::zeros(64, 1);
    assert!(!zero_but_not_empty.is_empty(), "64 cells is not no cells");
    assert!(
        zero_but_not_empty.is_zero(),
        "but every coefficient is zero"
    );

    let neither = c(64, 1, &[7]);
    assert!(!neither.is_empty());
    assert!(!neither.is_zero());

    // The empty chain is in its own chain group, and pairs with itself.
    assert!(nothing.add(&nothing).is_ok());
    assert!(
        nothing.add(&zero_but_not_empty).is_err(),
        "C_1 of dimension 0 is not C_1 of 64"
    );
}
