/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the HKT law generators and the comparator they are asserted with.
//!
//! A defect here is silent. The generators and `approx_eq` are what every law suite in
//! `tests/extensions` measures against, so a comparator that accepts too much or a sweep that
//! covers less than its doc claims weakens those suites without failing anything itself.

use deep_causality_topology::utils_tests::{
    LawRng, approx_eq, approx_eq_slice, chain_cases, graph_cases, manifold_cases, path_complex,
    path_complex_len,
};

const TOL: f64 = 1e-9;

// ----------------------------------------------------------------------------
// approx_eq
// ----------------------------------------------------------------------------

#[test]
fn approx_eq_accepts_a_difference_inside_the_tolerance() {
    assert!(approx_eq(1.0, 1.0 + 1e-12, TOL));
    assert!(approx_eq(0.0, 0.0, TOL));
    assert!(approx_eq(0.0, -0.0, TOL));
    // The relative arm: the absolute difference is well above `tol`, the relative one is not.
    assert!(approx_eq(1e6, 1e6 + 1e-4, TOL));
}

#[test]
fn approx_eq_rejects_a_difference_outside_the_tolerance() {
    assert!(!approx_eq(1.0, 1.1, TOL));
    assert!(!approx_eq(1e300, 1.0, TOL));
}

#[test]
fn approx_eq_rejects_nan_on_either_side() {
    assert!(!approx_eq(f64::NAN, f64::NAN, TOL));
    assert!(!approx_eq(f64::NAN, 1.0, TOL));
    assert!(!approx_eq(1.0, f64::NAN, TOL));
}

#[test]
fn approx_eq_rejects_an_infinity_against_a_finite_value() {
    // Both tolerance terms evaluate to infinity when either operand is infinite, and `inf <= inf`
    // holds, so a comparator without the finiteness check accepts every pair below. That made a
    // law that overflowed on one side indistinguishable from one that held.
    assert!(!approx_eq(f64::INFINITY, 1.0, TOL));
    assert!(!approx_eq(1.0, f64::INFINITY, TOL));
    assert!(!approx_eq(f64::NEG_INFINITY, 0.0, TOL));
    assert!(!approx_eq(f64::INFINITY, f64::NEG_INFINITY, TOL));
}

#[test]
fn approx_eq_accepts_an_infinity_against_itself() {
    // Exact equality still settles it, so two sides that overflow the same way still agree.
    assert!(approx_eq(f64::INFINITY, f64::INFINITY, TOL));
    assert!(approx_eq(f64::NEG_INFINITY, f64::NEG_INFINITY, TOL));
}

#[test]
fn approx_eq_slice_compares_length_and_elements() {
    assert!(approx_eq_slice(&[1.0, 2.0], &[1.0, 2.0 + 1e-12], TOL));
    assert!(approx_eq_slice(&[], &[], TOL));
    assert!(!approx_eq_slice(&[1.0, 2.0], &[1.0], TOL));
    assert!(!approx_eq_slice(&[1.0, 2.0], &[1.0, 2.5], TOL));
    assert!(!approx_eq_slice(&[1.0, f64::INFINITY], &[1.0, 2.0], TOL));
}

// ----------------------------------------------------------------------------
// LawRng
// ----------------------------------------------------------------------------

#[test]
fn a_seed_reproduces_its_own_sequence() {
    let mut first = LawRng::new(0x0D15_EA5E);
    let mut second = LawRng::new(0x0D15_EA5E);
    let a: Vec<u64> = (0..16).map(|_| first.next_u64()).collect();
    let b: Vec<u64> = (0..16).map(|_| second.next_u64()).collect();
    assert_eq!(
        a, b,
        "a reported counterexample has to replay from its seed"
    );
}

#[test]
fn below_stays_in_range_and_answers_a_zero_bound() {
    let mut rng = LawRng::new(0x0B_E107);
    for _ in 0..256 {
        assert!(rng.below(5) < 5);
    }
    // A zero bound has no legal answer, so the generator clamps instead of dividing by zero.
    assert_eq!(rng.below(0), 0);
}

#[test]
fn scalar_draws_the_awkward_values_it_advertises() {
    let mut rng = LawRng::new(0x5CA1);
    let drawn: Vec<f64> = (0..512).map(|_| rng.scalar(8.0)).collect();

    assert!(
        drawn.iter().any(|x| x.is_subnormal()),
        "no subnormal drawn: `f64::MIN_POSITIVE` is the smallest positive normal, so reaching for \
         it leaves the subnormal path untested"
    );
    assert!(
        drawn.iter().any(|x| x.abs() > 1e11),
        "no large magnitude drawn"
    );
    assert!(drawn.contains(&0.0), "no zero drawn");
    assert!(drawn.iter().all(|x| x.is_finite()), "an infinity was drawn");
}

#[test]
fn well_scaled_stays_in_range_and_leaves_the_extremes_out() {
    let mut rng = LawRng::new(0x5CA1 ^ 1);
    for x in rng.well_scaled_vec(512, 4.0) {
        assert!(x.abs() <= 4.0, "well_scaled left its range: {x}");
        assert!(
            x == 0.0 || x.is_normal(),
            "well_scaled drew a subnormal, which is what `scalar` is for: {x}"
        );
    }
}

#[test]
fn scalars_returns_the_requested_count() {
    let mut rng = LawRng::new(0x5CA1 ^ 2);
    assert_eq!(rng.scalars(7, 3.0).len(), 7);
    assert!(rng.scalars(0, 3.0).is_empty());
}

// ----------------------------------------------------------------------------
// Generators
// ----------------------------------------------------------------------------

#[test]
fn path_complex_len_matches_the_complex_it_describes() {
    for n in [2usize, 3, 5, 9] {
        let complex = path_complex::<f64>(n);
        let simplices: usize = complex
            .skeletons()
            .iter()
            .map(|s| s.simplices().len())
            .sum();
        assert_eq!(
            path_complex_len(n),
            simplices,
            "the advertised data length disagrees with the complex at n={n}"
        );
    }
}

#[test]
#[should_panic(expected = "a path complex needs at least two vertices")]
fn path_complex_rejects_a_degenerate_vertex_count() {
    let _ = path_complex::<f64>(1);
}

#[test]
#[should_panic(expected = "a path complex needs at least two vertices")]
fn path_complex_len_rejects_a_degenerate_vertex_count() {
    // The same precondition, because `2 * n - 1` describes no complex below two vertices and
    // underflows at zero.
    let _ = path_complex_len(1);
}

#[test]
fn manifold_cases_sweep_every_legal_cursor() {
    let cases = manifold_cases(0xC4A1);
    assert!(!cases.is_empty());
    for case in &cases {
        assert!(
            case.value.cursor() < case.value.data().len(),
            "generated an out-of-range focus for {}",
            case.label
        );
    }
    assert!(
        cases.iter().any(|c| c.value.cursor() > 0),
        "a sweep pinned at cursor 0 cannot see a dropped focus"
    );
}

#[test]
fn graph_cases_sweep_widths_and_cursors() {
    let cases = graph_cases(0xC4A1 ^ 1);
    // One case per legal cursor at each of the four widths.
    assert_eq!(cases.len(), 1 + 2 + 3 + 6);
    for case in &cases {
        assert_eq!(
            case.value.data().len(),
            case.value.num_vertices(),
            "payload and vertex count disagree for {}",
            case.label
        );
        assert!(case.value.cursor() < case.value.num_vertices());
    }
}

#[test]
fn chain_cases_never_generate_an_empty_chain() {
    // Load-bearing for the adjunction law suite: every assertion there reads a stored weight, and
    // `right_adjunct` and `counit` answer an empty chain with an `Err` rather than a value. CSR
    // drops explicit zeros, so an empty chain is one generated `0.0` away.
    let cases = chain_cases(0xC4A1 ^ 2);
    assert!(!cases.is_empty());
    for case in &cases {
        assert!(
            !case.value.weights().values().is_empty(),
            "{} generated a chain that stores nothing",
            case.label
        );
    }
}

#[test]
fn chain_cases_vary_grade_and_sparsity() {
    let cases = chain_cases(0xC4A1 ^ 3);
    assert!(
        cases.iter().any(|c| c.value.grade() == 0) && cases.iter().any(|c| c.value.grade() == 1),
        "the sweep covers both grades"
    );
    assert!(
        cases.iter().any(|c| c.label.contains("gapped")),
        "the sweep covers a chain whose columns are not a 0..k run"
    );
}
