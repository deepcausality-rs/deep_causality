/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `LogicalPauli`, a physical Pauli in symplectic form.
//!
//! # Risk, and the inputs that separate it
//!
//! The type carries two chains over one register, so the standing risk is that they are
//! exchanged: a `compose` that added `other.z` into `x`, or an `x()` accessor returning `z`,
//! is invisible whenever the two supports are equal or the register is symmetric. Every
//! fixture here therefore gives `X` and `Z` DISJOINT and unequal supports.
//!
//! Composition is addition over 𝔽₂, so the oracle is set symmetric difference, computed by
//! hand and stated as an explicit support.

use deep_causality_homology::Gf2Chain;
use deep_causality_quantum::{LogicalPauli, QuantumErrorEnum};

type W = u64;
type Chain = Gf2Chain<W>;

fn chain(support: &[usize]) -> Chain {
    Chain::from_support(6, 1, support).unwrap()
}

fn pauli(x: &[usize], z: &[usize]) -> LogicalPauli<W> {
    LogicalPauli::new(chain(x), chain(z)).expect("one register")
}

#[test]
fn test_the_two_supports_are_kept_apart() {
    // X on {0, 1} and Z on {3}: disjoint and different sizes, so an accessor returning the
    // wrong half, or a constructor storing them the wrong way round, shows immediately.
    let p = pauli(&[0, 1], &[3]);
    assert_eq!(p.x(), &chain(&[0, 1]));
    assert_eq!(p.z(), &chain(&[3]));
    assert_ne!(p.x(), p.z());
    assert_eq!(p.len(), 6);
    assert!(!p.is_empty());
    assert!(!p.is_identity());
}

#[test]
fn test_compose_adds_each_half_into_its_own() {
    // 𝔽₂ addition is symmetric difference, per half and independently:
    //   X: {0, 1} ⊕ {1, 2} = {0, 2}
    //   Z: {3}    ⊕ {4}    = {3, 4}
    // The four supports are pairwise different, so a composition that crossed the halves —
    // adding other.z into x — produces {0,1,4} and {1,2,3} instead.
    let a = pauli(&[0, 1], &[3]);
    let b = pauli(&[1, 2], &[4]);
    let c = a.compose(&b).expect("one register");

    assert_eq!(c.x(), &chain(&[0, 2]));
    assert_eq!(c.z(), &chain(&[3, 4]));
}

#[test]
fn test_composition_is_commutative_and_self_inverse_over_f2() {
    // P ⊕ P = identity, and P ⊕ Q = Q ⊕ P. Both follow from 𝔽₂ addition and hold for any
    // operand, so they catch a composition that is subtraction, or that keeps one side.
    let a = pauli(&[0, 1], &[3]);
    let b = pauli(&[1, 2], &[4]);

    let self_composed = a.compose(&a).expect("one register");
    assert!(
        self_composed.is_identity(),
        "P ⊕ P must act nowhere, got x={:?} z={:?}",
        self_composed.x(),
        self_composed.z()
    );

    assert_eq!(
        a.compose(&b).expect("one register"),
        b.compose(&a).expect("one register")
    );

    // Composing with the identity returns the operand unchanged.
    let identity = pauli(&[], &[]);
    assert!(identity.is_identity());
    assert_eq!(a.compose(&identity).expect("one register"), a);
}

#[test]
fn test_identity_requires_both_halves_to_be_empty() {
    // A Pauli acting only through Z is not the identity. Testing `is_identity` on the
    // all-empty case alone would accept a predicate that looked at `x` only.
    assert!(
        !pauli(&[], &[2]).is_identity(),
        "Z-only is not the identity"
    );
    assert!(
        !pauli(&[2], &[]).is_identity(),
        "X-only is not the identity"
    );
    assert!(pauli(&[], &[]).is_identity());
}

#[test]
fn test_a_pauli_over_two_registers_is_refused() {
    // The X and Z halves must describe one operator, so mismatched widths are rejected at
    // construction rather than producing a Pauli that cannot be composed.
    let err = LogicalPauli::new(chain(&[0]), Chain::from_support(8, 1, &[0]).unwrap()).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)),
        "expected DimensionMismatch, got {err:?}"
    );
}

#[test]
fn test_composing_across_registers_is_refused() {
    let a = pauli(&[0], &[1]);
    let wide = LogicalPauli::new(
        Chain::from_support(8, 1, &[0]).unwrap(),
        Chain::from_support(8, 1, &[1]).unwrap(),
    )
    .expect("one register");

    let err = a.compose(&wide).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)),
        "expected DimensionMismatch, got {err:?}"
    );
}

#[test]
fn test_an_empty_register_reports_itself_empty() {
    let empty = LogicalPauli::new(
        Chain::from_support(0, 1, &[]).unwrap(),
        Chain::from_support(0, 1, &[]).unwrap(),
    )
    .expect("one register");
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);
    // Width zero and "acts nowhere" are different questions that coincide here; the
    // six-qubit identity above separates them.
    assert!(empty.is_identity());
}
