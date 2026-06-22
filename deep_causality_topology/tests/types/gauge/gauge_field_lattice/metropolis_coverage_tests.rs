/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage for the `try_metropolis_update` non-finite-action rejection branch.
//!
//! When the local action change `ΔS` is not finite (NaN/Inf), the Metropolis
//! step must reject the proposal unconditionally (`Ok(false)`), guarding against
//! pathological numerics. This is the `if !delta_s.is_finite() { false }` branch
//! in `ops_metropolis.rs`.

use deep_causality_num::Complex;
use deep_causality_rand::types::Xoshiro256;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{ChainComplex, LatticeComplex, LatticeGaugeField, LinkVariable, U1};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// try_metropolis_update: a field whose links carry non-finite (Inf) matrix
// entries produces a non-finite ΔS. Because old_tr and new_tr are both +Inf,
// ΔS = β·(old_tr - new_tr)/N = β·(Inf - Inf)/N = NaN. NaN is not `< 0`, so the
// accept/reject `else` branch runs, hits `!delta_s.is_finite()`, and rejects.
// ============================================================================

#[test]
fn test_metropolis_update_rejects_non_finite_action() {
    let lattice = Arc::new(LatticeComplex::new([2, 2], [true, true]));

    // 1x1 U(1) link holding an infinite matrix entry (bypasses validation via the
    // unchecked matrix constructor; shape [1, 1] still matches U(1)::matrix_dim()).
    let inf_tensor =
        CausalTensor::new(vec![Complex::new(f64::INFINITY, 0.0)], vec![1, 1]).expect("1x1 tensor");
    let make_inf_link =
        || LinkVariable::<U1, Complex<f64>, f64>::from_matrix_unchecked(inf_tensor.clone());

    let mut links: HashMap<_, LinkVariable<U1, Complex<f64>, f64>> = HashMap::new();
    let edges: Vec<_> = lattice.cells(1).collect();
    assert!(!edges.is_empty(), "expected at least one edge");
    for edge in &edges {
        links.insert(edge.clone(), make_inf_link());
    }

    let mut field: LatticeGaugeField<U1, 2, Complex<f64>, f64> =
        LatticeGaugeField::from_links_unchecked(lattice, links, 6.0, ());

    let mut rng = Xoshiro256::new();

    // The proposal must be rejected because ΔS is non-finite, regardless of the
    // random perturbation drawn.
    let accepted = field
        .try_metropolis_update(&edges[0], 0.3, &mut rng)
        .expect("metropolis update should not error on non-finite action");
    assert!(
        !accepted,
        "non-finite ΔS must lead to rejection (Ok(false))"
    );
}
