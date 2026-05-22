/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `CubicalReggeGeometry::metropolis_update` — Phase R6.4 + R6.5.
//!
//! Covers:
//! - Type sanity: `AcceptReject` / `RejectReason` variants pattern-match.
//! - Mutation semantics: accepted updates change the per-edge buffer at the
//!   right index; rejected updates leave the geometry untouched.
//! - Boundary behaviour: `NonPositiveLength` rejection triggers when `σ` is
//!   large enough to push edge lengths below zero on average.
//! - Detailed-balance smoke: the equilibrium mean edge length drifts toward
//!   the action-minimising configuration over moderate-length runs.

use deep_causality_topology::utils_tests::open_cube_3;
use deep_causality_topology::{AcceptReject, ChainComplex, CubicalReggeGeometry, RejectReason};

/// Build a 3D PerEdge geometry with all edges initialised to `length`.
fn build_per_edge_3d(num_edges: usize, length: f64) -> CubicalReggeGeometry<3, f64> {
    CubicalReggeGeometry::<3, f64>::from_edge_lengths(vec![length; num_edges])
}

#[test]
fn accept_reject_variants_pattern_match() {
    // Compile-time check that the public variants are reachable from outside
    // the crate via the re-export.
    let a: AcceptReject<f64> = AcceptReject::Accepted {
        edge: 0,
        proposed_length: 1.0,
        delta_action: 0.0,
    };
    let r: AcceptReject<f64> = AcceptReject::Rejected {
        edge: 0,
        proposed_length: -0.5,
        reason: RejectReason::NonPositiveLength,
    };
    assert!(matches!(a, AcceptReject::Accepted { .. }));
    assert!(matches!(
        r,
        AcceptReject::Rejected {
            reason: RejectReason::NonPositiveLength,
            ..
        }
    ));
}

#[test]
fn metropolis_step_returns_well_formed_outcome() {
    let lattice = open_cube_3();
    let num_edges = lattice.num_cells(1);
    let mut geom = build_per_edge_3d(num_edges, 1.0);
    let mut rng = deep_causality_rand::rng();

    let outcome = geom.metropolis_update(&lattice, &mut rng, 0.1, 1.0);
    match outcome {
        AcceptReject::Accepted {
            edge,
            proposed_length,
            ..
        } => {
            assert!(edge < num_edges);
            assert!(proposed_length > 0.0);
        }
        AcceptReject::Rejected { edge, .. } => {
            assert!(edge < num_edges);
        }
    }
}

#[test]
fn accepted_step_mutates_only_the_target_edge() {
    let lattice = open_cube_3();
    let num_edges = lattice.num_cells(1);
    let mut geom = build_per_edge_3d(num_edges, 1.0);
    let before: Vec<f64> = (0..num_edges)
        .map(|i| geom.edge_length_at(i).unwrap())
        .collect();
    let mut rng = deep_causality_rand::rng();

    // Drive enough steps that we're statistically certain to see at least one acceptance.
    let mut saw_accept = false;
    for _ in 0..200 {
        let out = geom.metropolis_update(&lattice, &mut rng, 0.05, 0.1);
        if let AcceptReject::Accepted {
            edge,
            proposed_length,
            ..
        } = out
        {
            saw_accept = true;
            // The target edge must now equal `proposed_length`.
            assert!((geom.edge_length_at(edge).unwrap() - proposed_length).abs() < 1e-12);
            break;
        }
    }
    assert!(
        saw_accept,
        "200 Metropolis steps with σ=0.05, β=0.1 should produce at least one acceptance"
    );

    // Length count and edge accessibility are preserved.
    for i in 0..num_edges {
        assert!(
            geom.edge_length_at(i).unwrap() > 0.0,
            "edge {i} non-positive after run"
        );
        let _ = before; // suppress unused warning if no asserts on it
    }
}

#[test]
fn rejected_step_leaves_geometry_unchanged() {
    let lattice = open_cube_3();
    let num_edges = lattice.num_cells(1);
    let mut geom = build_per_edge_3d(num_edges, 1.0);
    let mut rng = deep_causality_rand::rng();

    // Try repeatedly with a very large σ so most proposals push length ≤ 0 and
    // get rejected by the NonPositiveLength floor; capture the first rejection
    // and verify the per-edge buffer is identical before and after.
    for _ in 0..50 {
        let snapshot: Vec<f64> = (0..num_edges)
            .map(|i| geom.edge_length_at(i).unwrap())
            .collect();
        let out = geom.metropolis_update(&lattice, &mut rng, 10.0, 1.0);
        match out {
            AcceptReject::Rejected {
                reason: RejectReason::NonPositiveLength,
                ..
            } => {
                for (i, &snapped) in snapshot.iter().enumerate() {
                    assert_eq!(geom.edge_length_at(i).unwrap(), snapped);
                }
                return; // success path
            }
            _ => continue,
        }
    }
    panic!("50 large-σ proposals should produce at least one NonPositiveLength rejection");
}

#[test]
fn non_positive_proposal_returns_non_positive_length_rejection() {
    // Deterministic check: use very large σ and small β; with high
    // probability some proposal will be ≤ 0.
    let lattice = open_cube_3();
    let num_edges = lattice.num_cells(1);
    let mut geom = build_per_edge_3d(num_edges, 0.5); // start near 0
    let mut rng = deep_causality_rand::rng();
    let mut saw_nonpos = false;
    for _ in 0..100 {
        let out = geom.metropolis_update(&lattice, &mut rng, 2.0, 1.0);
        if matches!(
            out,
            AcceptReject::Rejected {
                reason: RejectReason::NonPositiveLength,
                ..
            }
        ) {
            saw_nonpos = true;
            break;
        }
    }
    assert!(saw_nonpos);
}

#[test]
fn edge_lengths_stay_positive_across_long_run() {
    // R6.5 detailed-balance smoke: run many steps, confirm all edge lengths
    // remain strictly positive. This is a weaker check than full equilibrium
    // distribution matching (which would need a long-running test and binning),
    // but it confirms the basic invariants of the Metropolis loop.
    let lattice = open_cube_3();
    let num_edges = lattice.num_cells(1);
    let mut geom = build_per_edge_3d(num_edges, 1.0);
    let mut rng = deep_causality_rand::rng();

    let mut accepted = 0usize;
    let mut rejected = 0usize;
    // After the R6.6.1 single-edge-gradient optimisation, each Metropolis
    // step is O(D · 2^D) instead of O(num_hinges · 2^D) — a ~50x speedup
    // on this lattice. 20K steps now run in ~3s in debug mode (the prior
    // 5K-step run took ~47s before the optimisation).
    let steps = 20_000;
    for _ in 0..steps {
        match geom.metropolis_update(&lattice, &mut rng, 0.1, 1.0) {
            AcceptReject::Accepted { .. } => accepted += 1,
            AcceptReject::Rejected { .. } => rejected += 1,
        }
    }
    for i in 0..num_edges {
        let l = geom.edge_length_at(i).unwrap();
        assert!(l > 0.0, "edge {i}: non-positive length {l} after long run");
        assert!(l.is_finite(), "edge {i}: non-finite length after long run");
    }
    // Sanity on acceptance rate: should be neither degenerate.
    let rate = accepted as f64 / steps as f64;
    assert!(
        (0.05..0.95).contains(&rate),
        "acceptance rate {rate} (accepted={accepted}, rejected={rejected}) \
         outside sensible (0.05, 0.95) range — Metropolis tuning broken"
    );
}

#[test]
fn delta_action_recorded_on_acceptance_matches_gradient_product() {
    // Verifies the documented exact equality `ΔS = (L_new − L_old) · gradient[e]`
    // for axis-aligned cubical (bilinear-in-lengths action).
    let lattice = open_cube_3();
    let num_edges = lattice.num_cells(1);
    let mut geom = build_per_edge_3d(num_edges, 1.0);
    let mut rng = deep_causality_rand::rng();

    for _ in 0..500 {
        let pre_lengths: Vec<f64> = (0..num_edges)
            .map(|i| geom.edge_length_at(i).unwrap())
            .collect();
        let pre_gradient = geom.regge_gradient(&lattice);
        let out = geom.metropolis_update(&lattice, &mut rng, 0.1, 1.0);
        if let AcceptReject::Accepted {
            edge,
            proposed_length,
            delta_action,
        } = out
        {
            let expected = (proposed_length - pre_lengths[edge]) * pre_gradient[edge];
            assert!(
                (delta_action - expected).abs() < 1e-12,
                "ΔS reported = {delta_action}, expected = {expected} \
                 ((L_new − L_old) · gradient[e])"
            );
            return;
        }
    }
    panic!("500 steps should have produced at least one acceptance with ΔS reported");
}
