/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `DiagonalPhase`, the exact-rational phase of a diagonal logical gate.
//!
//! # Oracle provenance
//!
//! The polynomials are Junichi Haruna, arXiv:2511.15224, Table 1, normalised so the phase reads
//! `exp(2πi·Q(n)/M)`:
//!
//! | Gate | `Q(n)` | `M` |
//! |---|---|---|
//! | `Z̄` | `n` | 2 |
//! | `S̄` | `n²` | 4 |
//! | `T̄` | `2n³ − 3n² + 2n` | 8 |
//!
//! Expected values below are those polynomials evaluated by hand, not by re-running the
//! accumulator under test.
//!
//! `class_invariance_tests.rs` already pins `phase_at(0)` and `phase_at(1)`. Neither separates the
//! coefficients from each other: at `n = 1` every power is 1, so `Q(1)` is the plain SUM of the
//! coefficients, and any permutation of `T̄`'s `{2, −3, 2}` reproduces it. Its `Q(1) = 1` and
//! `Q(0) = 0` survive the reordering `[0, −3, 2, 2]`, which is a different gate at every other
//! overlap. That is what the `n ≥ 2` cases here are for.

use deep_causality_homology::Gf2Chain;
use deep_causality_quantum::{DiagonalPhase, QuantumErrorEnum};

type W = u64;
type Chain = Gf2Chain<W>;

fn chain(support: &[usize]) -> Chain {
    Chain::from_support(4, 1, support).unwrap()
}

#[allow(non_snake_case)]
fn Rational(n: i64, d: i64) -> deep_causality_num_rational::Rational<i64> {
    deep_causality_num_rational::Rational::new(n, d)
}

// ---------------------------------------------------------------------------
// The polynomials, away from the degenerate overlaps.
// ---------------------------------------------------------------------------

#[test]
fn test_z_phase_is_the_overlap_over_two() {
    // Q(n) = n, M = 2.
    let z = DiagonalPhase::z(chain(&[0]));
    assert_eq!(z.phase_at(2), Rational(1, 1)); // 2/2, a whole turn
    assert_eq!(z.phase_at(3), Rational(3, 2));
    assert_eq!(z.phase_at(5), Rational(5, 2));
}

#[test]
fn test_s_phase_is_the_squared_overlap_over_four() {
    // Q(n) = n², M = 4. The square is what separates S̄ from Z̄ away from n ∈ {0, 1},
    // where n and n² agree.
    let s = DiagonalPhase::s(chain(&[0]));
    assert_eq!(s.phase_at(2), Rational(1, 1)); // 4/4
    assert_eq!(s.phase_at(3), Rational(9, 4));
    assert_eq!(s.phase_at(5), Rational(25, 4));
}

#[test]
fn test_t_phase_follows_the_cubic_at_every_coefficient() {
    // Q(n) = 2n³ − 3n² + 2n, M = 8, evaluated by hand:
    //   n = 2:  16 − 12 + 4 = 8   → 8/8 = 1
    //   n = 3:  54 − 27 + 6 = 33  → 33/8
    //   n = 4: 128 − 48 + 8 = 88  → 88/8 = 11
    //   n = 5: 250 − 75 + 10 = 185 → 185/8
    // The reordering [0, −3, 2, 2], which Q(0) and Q(1) both accept, gives
    // −6 + 8 + 16 = 18 at n = 2 instead of 8.
    let t = DiagonalPhase::t(chain(&[0]));
    assert_eq!(t.phase_at(2), Rational(1, 1));
    assert_eq!(t.phase_at(3), Rational(33, 8));
    assert_eq!(t.phase_at(4), Rational(11, 1));
    assert_eq!(t.phase_at(5), Rational(185, 8));
}

#[test]
fn test_the_three_gates_disagree_away_from_the_shared_points() {
    // The three polynomials coincide at exactly two overlaps, and both are degenerate:
    //
    //   n = 0:  0/2 = 0/4 = 0/8 = 0
    //   n = 2:  2/2 = 4/4 = 8/8 = 1, one whole turn, so all three act as the identity
    //
    // `class_invariance_tests` pins n = 0, one of those two. Everywhere else the gates must
    // be distinguishable, or Z̄, S̄ and T̄ are interchangeable in any test that avoids n = 1.
    let g = chain(&[0]);
    let (z, s, t) = (
        DiagonalPhase::z(g.clone()),
        DiagonalPhase::s(g.clone()),
        DiagonalPhase::t(g),
    );
    for n in [1u64, 3, 4, 5, 6] {
        assert_ne!(z.phase_at(n), s.phase_at(n), "Z and S agree at n = {n}");
        assert_ne!(s.phase_at(n), t.phase_at(n), "S and T agree at n = {n}");
        assert_ne!(z.phase_at(n), t.phase_at(n), "Z and T agree at n = {n}");
    }

    // And the two coincidences are real, not an accident of this fixture.
    for n in [0u64, 2] {
        assert_eq!(z.phase_at(n), s.phase_at(n));
        assert_eq!(s.phase_at(n), t.phase_at(n));
    }
}

#[test]
fn test_the_modulus_is_the_declared_power_of_two() {
    let g = chain(&[0]);
    assert_eq!(DiagonalPhase::z(g.clone()).modulus(), 2);
    assert_eq!(DiagonalPhase::s(g.clone()).modulus(), 4);
    assert_eq!(DiagonalPhase::t(g.clone()).modulus(), 8);
    // And an explicitly built gate reports 2^k, not k.
    let custom = DiagonalPhase::new(g, alloc_vec(), 5).expect("valid polynomial");
    assert_eq!(custom.modulus(), 32);
}

fn alloc_vec() -> Vec<i64> {
    vec![0, 1]
}

// ---------------------------------------------------------------------------
// Construction, including both refusals.
// ---------------------------------------------------------------------------

#[test]
fn test_an_empty_polynomial_is_refused() {
    let err = DiagonalPhase::new(chain(&[0]), Vec::new(), 1).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)),
        "expected DimensionMismatch, got {err:?}"
    );
}

#[test]
fn test_a_modulus_that_would_not_fit_is_refused_at_its_boundary() {
    // The guard is `log2_modulus >= 62`, so 61 is accepted and 62 is not.
    assert!(DiagonalPhase::new(chain(&[0]), vec![0, 1], 61).is_ok());

    let err = DiagonalPhase::new(chain(&[0]), vec![0, 1], 62).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)),
        "expected DimensionMismatch, got {err:?}"
    );
}

#[test]
fn test_a_custom_polynomial_is_evaluated_in_ascending_order() {
    // coeffs are ascending, so [1, 0, 3] is Q(n) = 1 + 3n², not 3 + n².
    // At n = 2 those are 13 and 7; at n = 1 both are 4, which is why the
    // check uses n = 2.
    let g = DiagonalPhase::new(chain(&[0]), vec![1, 0, 3], 1).expect("valid polynomial");
    assert_eq!(g.phase_at(1), Rational(4, 2));
    assert_eq!(g.phase_at(2), Rational(13, 2));
}

// ---------------------------------------------------------------------------
// Moving to another representative.
// ---------------------------------------------------------------------------

#[test]
fn test_shifting_moves_the_chain_and_leaves_the_gate_alone() {
    // γ ↦ γ ⊕ b over F₂: {0,1} ⊕ {1,2} = {0,2}. The polynomial and modulus do not move,
    // which is what makes the class invariance question well posed.
    let t = DiagonalPhase::t(chain(&[0, 1]));
    let shifted = t.shifted_by(&chain(&[1, 2])).expect("same register width");

    assert_eq!(shifted.chain(), &chain(&[0, 2]));
    assert_eq!(shifted.modulus(), t.modulus());
    for n in [0u64, 1, 2, 3, 5] {
        assert_eq!(shifted.phase_at(n), t.phase_at(n), "phase moved at n = {n}");
    }

    // Shifting by the same chain twice returns to the original: b ⊕ b = 0.
    let back = shifted.shifted_by(&chain(&[1, 2])).expect("same width");
    assert_eq!(back.chain(), t.chain());
}

#[test]
fn test_shifting_by_another_register_is_refused() {
    let t = DiagonalPhase::t(chain(&[0]));
    let wrong = Chain::from_support(8, 1, &[0]).unwrap();
    let err = t.shifted_by(&wrong).unwrap_err();
    assert!(
        matches!(err.0, QuantumErrorEnum::DimensionMismatch(_)),
        "expected DimensionMismatch, got {err:?}"
    );
}
