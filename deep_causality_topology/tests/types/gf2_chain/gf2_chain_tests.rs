/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The mod-2 chain.
//!
//! The bit arithmetic belongs to `PackedGf2Vector` and is tested where it lives. What is tested
//! here is the part this type adds: that the degree travels with the data, and that an operation
//! between two degrees is refused rather than computed.

use deep_causality_linear::{MatrixBuild, PackedGf2};
use deep_causality_num::Gf2;
use deep_causality_topology::{Gf2Chain, TopologyError, TopologyErrorEnum};

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
                Some(TopologyError(TopologyErrorEnum::DimensionMismatch(_)))
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
/// This is the path a homology generator takes: `kernel_basis_gf2` returns its basis as rows, and
/// the degree is what the caller knows and the matrix does not.
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
