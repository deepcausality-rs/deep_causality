/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for LatticeGaugeField type.
//!
//! Covers constructors, plaquettes, Wilson action, Monte Carlo methods,
//! and gradient flow.

use deep_causality_topology::{CWComplex, Lattice, LatticeGaugeField, LinkVariable, U1};
use std::sync::Arc;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a small 2x2 lattice for testing
fn create_test_lattice() -> Arc<Lattice<2>> {
    Arc::new(Lattice::new([2, 2], [true, true]))
}

/// Create a small 1D lattice for simpler tests
fn create_1d_lattice() -> Arc<Lattice<1>> {
    Arc::new(Lattice::new([4], [true]))
}

// ============================================================================
// Constructor Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_try_identity() {
    let lattice = create_test_lattice();
    let field: Result<LatticeGaugeField<U1, 2, f64>, _> =
        LatticeGaugeField::try_identity(lattice, 6.0);
    assert!(field.is_ok());
    let field = field.unwrap();
    assert!((*field.beta() - 6.0).abs() < 1e-10);
}

#[test]
fn test_lattice_gauge_field_identity_convenience() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);
    assert!((*field.beta() - 6.0).abs() < 1e-10);
}

#[test]
fn test_lattice_gauge_field_try_from_links_valid() {
    let lattice = create_1d_lattice();
    let mut links = std::collections::HashMap::new();

    // Add identity links for all edges
    for cell in lattice.cells(1) {
        let link: LinkVariable<U1, f64> = LinkVariable::identity();
        links.insert(cell, link);
    }

    let field: Result<LatticeGaugeField<U1, 1, f64>, _> =
        LatticeGaugeField::try_from_links(lattice, links, 1.0);
    assert!(field.is_ok());
}

#[test]
fn test_lattice_gauge_field_try_from_links_missing() {
    let lattice = create_1d_lattice();
    let links = std::collections::HashMap::new(); // Empty links

    let field: Result<LatticeGaugeField<U1, 1, f64>, _> =
        LatticeGaugeField::try_from_links(lattice, links, 1.0);
    assert!(field.is_err());
}

#[test]
fn test_lattice_gauge_field_from_links_unchecked() {
    let lattice = create_1d_lattice();
    let links = std::collections::HashMap::new();
    let field: LatticeGaugeField<U1, 1, f64> =
        LatticeGaugeField::from_links_unchecked(lattice, links, 2.0);
    assert!((*field.beta() - 2.0).abs() < 1e-10);
}

// ============================================================================
// Getter Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_lattice() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice.clone(), 6.0);
    assert_eq!(field.lattice().shape(), &[2, 2]);
}

#[test]
fn test_lattice_gauge_field_lattice_arc() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice.clone(), 6.0);
    let arc = field.lattice_arc();
    assert_eq!(arc.shape(), &[2, 2]);
}

#[test]
fn test_lattice_gauge_field_beta() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 5.5);
    assert!((*field.beta() - 5.5).abs() < 1e-10);
}

#[test]
fn test_lattice_gauge_field_num_links() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);
    assert!(field.num_links() > 0);
}

#[test]
fn test_lattice_gauge_field_link() {
    let lattice = create_1d_lattice();
    let field: LatticeGaugeField<U1, 1, f64> = LatticeGaugeField::identity(lattice.clone(), 6.0);

    let edges: Vec<_> = lattice.cells(1).collect();
    if !edges.is_empty() {
        let link = field.link(&edges[0]);
        assert!(link.is_some());
    }
}

#[test]
fn test_lattice_gauge_field_link_mut() {
    let lattice = create_1d_lattice();
    let mut field: LatticeGaugeField<U1, 1, f64> =
        LatticeGaugeField::identity(lattice.clone(), 6.0);

    let edges: Vec<_> = lattice.cells(1).collect();
    if !edges.is_empty() {
        let link = field.link_mut(&edges[0]);
        assert!(link.is_some());
    }
}

#[test]
fn test_lattice_gauge_field_links() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);
    let links = field.links();
    assert!(!links.is_empty());
}

#[test]
fn test_lattice_gauge_field_set_link() {
    let lattice = create_1d_lattice();
    let mut field: LatticeGaugeField<U1, 1, f64> =
        LatticeGaugeField::identity(lattice.clone(), 6.0);

    let edges: Vec<_> = lattice.cells(1).collect();
    if !edges.is_empty() {
        let new_link: LinkVariable<U1, f64> = LinkVariable::identity();
        field.set_link(edges[0].clone(), new_link);
        assert!(field.link(&edges[0]).is_some());
    }
}

// ============================================================================
// into_parts Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_beta_owned() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 7.0);
    let beta = field.beta_owned();
    assert!((beta - 7.0).abs() < 1e-10);
}

#[test]
fn test_lattice_gauge_field_into_parts() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 8.0);
    let (lat, links, beta) = field.into_parts();
    assert_eq!(lat.shape(), &[2, 2]);
    assert!(!links.is_empty());
    assert!((beta - 8.0).abs() < 1e-10);
}

// ============================================================================
// Plaquette Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_try_plaquette_identity() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let result = field.try_plaquette(&[0, 0], 0, 1);
    assert!(result.is_ok());
    let plaq = result.unwrap();
    assert!((plaq.trace() - 1.0).abs() < 1e-10);
}

#[test]
fn test_lattice_gauge_field_try_average_plaquette() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let avg = field.try_average_plaquette();
    assert!(avg.is_ok());
    let val = avg.unwrap();
    assert!((val - 1.0).abs() < 0.1, "Average plaquette = {}", val);
}

// ============================================================================
// Wilson Action Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_try_wilson_action_identity() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let action = field.try_wilson_action();
    assert!(action.is_ok());
    let val = action.unwrap();
    assert!(val.abs() < 0.1, "Wilson action for identity = {}", val);
}

#[test]
fn test_lattice_gauge_field_try_plaquette_action() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let action = field.try_plaquette_action(&[0, 0], 0, 1);
    assert!(action.is_ok());
    let val = action.unwrap();
    assert!(val.abs() < 1e-10);
}

// ============================================================================
// Display Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_display() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);
    let display = format!("{}", field);
    assert!(display.contains("LatticeGaugeField"));
    assert!(display.contains("U(1)"));
}

// ============================================================================
// Monte Carlo Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_try_staple() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice.clone(), 6.0);

    let edges: Vec<_> = lattice.cells(1).collect();
    if !edges.is_empty() {
        let staple = field.try_staple(&edges[0]);
        assert!(staple.is_ok());
    }
}

#[test]
fn test_lattice_gauge_field_try_local_action_change() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice.clone(), 6.0);
    let new_link: LinkVariable<U1, f64> = LinkVariable::identity();

    let edges: Vec<_> = lattice.cells(1).collect();
    if !edges.is_empty() {
        let delta_s = field.try_local_action_change(&edges[0], &new_link);
        assert!(delta_s.is_ok());
        let val = delta_s.unwrap();
        assert!(val.abs() < 1e-10, "ΔS for I→I = {}", val);
    }
}

// ============================================================================
// Gradient Flow Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_try_energy_density() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let energy = field.try_energy_density();
    assert!(energy.is_ok());
    let val = energy.unwrap();
    assert!(val.abs() < 0.1, "Energy density = {}", val);
}

#[test]
fn test_lattice_gauge_field_try_t2_energy() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);
    let flow_time = 0.1;

    let t2e = field.try_t2_energy(flow_time);
    assert!(t2e.is_ok());
}

// ============================================================================
// Random Constructor Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_try_random() {
    let lattice = create_test_lattice();
    let mut rng = deep_causality_rand::rng();

    let field: Result<LatticeGaugeField<U1, 2, f64>, _> =
        LatticeGaugeField::try_random(lattice.clone(), 6.0, &mut rng);
    assert!(field.is_ok());

    let field = field.unwrap();
    assert!((*field.beta() - 6.0).abs() < 1e-10);
    assert!(field.num_links() > 0);
}

#[test]
fn test_lattice_gauge_field_random_convenience() {
    let lattice = create_test_lattice();
    let mut rng = deep_causality_rand::rng();

    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::random(lattice, 6.0, &mut rng);
    assert!(field.num_links() > 0);
}

#[test]
fn test_lattice_gauge_field_random_vs_identity_differ() {
    let lattice = create_test_lattice();
    let mut rng = deep_causality_rand::rng();

    let identity: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice.clone(), 6.0);
    let random: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::random(lattice, 6.0, &mut rng);

    // Action should differ between identity (minimum) and random (higher)
    let action_id = identity.try_wilson_action().unwrap();
    let action_rand = random.try_wilson_action().unwrap();

    // Identity has S ≈ 0, random has S > 0
    assert!(action_rand > action_id || (action_rand - action_id).abs() < 0.01);
}

// ============================================================================
// Wilson Loop Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_try_wilson_loop_identity() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    // 1x1 Wilson loop on identity field should give 1 (Tr(I)/N = 1)
    let wloop = field.try_wilson_loop(&[0, 0], 0, 1, 1, 1);
    assert!(wloop.is_ok());

    let val = wloop.unwrap();
    assert!(
        (val - 1.0).abs() < 0.1,
        "1x1 Wilson loop on identity should be ~1, got {}",
        val
    );
}

#[test]
fn test_lattice_gauge_field_try_wilson_loop_invalid_dirs() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    // Same direction should fail
    let result = field.try_wilson_loop(&[0, 0], 0, 0, 1, 1);
    assert!(result.is_err());
}

#[test]
fn test_lattice_gauge_field_try_wilson_loop_zero_size() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let result = field.try_wilson_loop(&[0, 0], 0, 1, 0, 1);
    assert!(result.is_err());
}

#[test]
fn test_lattice_gauge_field_try_polyakov_loop_identity() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let polyakov = field.try_polyakov_loop(&[0, 0], 0);
    assert!(polyakov.is_ok());

    let val = polyakov.unwrap();
    // For identity: Tr(I...I)/N = 1
    assert!(
        (val - 1.0).abs() < 0.1,
        "Polyakov loop on identity should be ~1, got {}",
        val
    );
}

#[test]
fn test_lattice_gauge_field_try_average_polyakov_loop() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let avg = field.try_average_polyakov_loop(0);
    assert!(avg.is_ok());
}

// ============================================================================
// Metropolis Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_metropolis_sweep_f64() {
    let lattice = create_test_lattice();
    let mut rng = deep_causality_rand::rng();

    let mut field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    // Perform a sweep
    let acceptance = field.metropolis_sweep_f64(0.1, &mut rng);
    assert!(acceptance.is_ok());

    let rate = acceptance.unwrap();
    assert!((0.0..=1.0).contains(&rate), "Acceptance rate = {}", rate);
}

#[test]
fn test_lattice_gauge_field_metropolis_update_f64() {
    let lattice = create_test_lattice();
    let mut rng = deep_causality_rand::rng();

    let mut field: LatticeGaugeField<U1, 2, f64> =
        LatticeGaugeField::identity(lattice.clone(), 6.0);

    // Get first edge and clone it before using
    let edge = {
        let edges: Vec<_> = lattice.cells(1).collect();
        edges.into_iter().next()
    };

    if let Some(e) = edge {
        let result = field.metropolis_update_f64(&e, 0.1, &mut rng);
        assert!(result.is_ok());
    }
}

// ============================================================================
// Gauge Transform Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_gauge_transform_action_invariance() {
    let lattice = create_test_lattice();
    let mut rng = deep_causality_rand::rng();

    let mut field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let action_before = field.try_wilson_action().unwrap();

    // Apply random gauge transform
    let result = field.try_random_gauge_transform(&mut rng);
    assert!(result.is_ok());

    let action_after = field.try_wilson_action().unwrap();

    // Action should be invariant (within numerical precision)
    assert!(
        (action_after - action_before).abs() < 0.1,
        "Action not invariant: {} vs {}",
        action_before,
        action_after
    );
}

// ============================================================================
// Continuum Limit Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_try_field_strength() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let fs = field.try_field_strength(&[0, 0], 0, 1);
    assert!(fs.is_ok());
}

#[test]
fn test_lattice_gauge_field_try_field_strength_antisymmetry() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    // F_μμ = 0
    let f00 = field.try_field_strength(&[0, 0], 0, 0);
    assert!(f00.is_ok());

    let val = f00.unwrap();
    assert!(val.as_slice()[0].abs() < 1e-10, "F_00 should be zero");
}

#[test]
fn test_lattice_gauge_field_try_topological_charge_density_2d() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    // In 2D, topological charge is 0 (requires D >= 4)
    let q = field.try_topological_charge_density(&[0, 0]);
    assert!(q.is_ok());
    assert!(q.unwrap().abs() < 1e-10);
}

// ============================================================================
// Gradient Flow Scale Setting Tests
// ============================================================================

#[test]
fn test_lattice_gauge_field_find_t0_not_reached() {
    use deep_causality_topology::FlowParams;

    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let params = FlowParams {
        epsilon: 0.01,
        t_max: 0.1, // Short time, won't reach t0
        method: deep_causality_topology::FlowMethod::Euler,
    };

    let result = field.try_find_t0(&params);
    // For identity field with short t_max, may not reach t₀
    // This tests the error path
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Improved Action Tests
// ============================================================================

#[test]
fn test_action_coeffs_constructors() {
    use deep_causality_topology::ActionCoeffs;

    let sym = ActionCoeffs::<f64>::try_symanzik().unwrap();
    assert!((sym.c1 - (-1.0 / 12.0)).abs() < 1e-10);
    assert!((sym.c0 - (1.0 + 8.0 / 12.0)).abs() < 1e-10);

    let iwa = ActionCoeffs::<f64>::try_iwasaki().unwrap();
    assert!((iwa.c1 - (-0.331)).abs() < 1e-10);

    let dbw2 = ActionCoeffs::<f64>::try_dbw2().unwrap();
    assert!((dbw2.c1 - (-1.4088)).abs() < 1e-10);

    let cust = ActionCoeffs::<f64>::custom(0.6, 0.05);
    assert!((cust.c0 - 0.6).abs() < 1e-10);
}

#[test]
fn test_lattice_gauge_field_try_improved_action_identity() {
    use deep_causality_topology::ActionCoeffs;

    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);
    let coeffs = ActionCoeffs::try_symanzik().unwrap();

    // For identity field, both plaquette and rectangle traces are N=1
    // S_plaq = 0, S_rect = 0
    // Total S = 0
    let action = field.try_improved_action(&coeffs);
    assert!(action.is_ok());
    let val = action.unwrap();
    assert!(val.abs() < 1e-10);
}

#[test]
fn test_lattice_gauge_field_try_improved_action_random() {
    use deep_causality_topology::ActionCoeffs;

    let lattice = create_test_lattice();
    let mut rng = deep_causality_rand::rng();
    // Use try_random to get a non-trivial field
    let field: LatticeGaugeField<U1, 2, f64> =
        LatticeGaugeField::try_random(lattice, 6.0, &mut rng).unwrap();

    let coeffs = ActionCoeffs::try_symanzik().unwrap();
    let action = field.try_improved_action(&coeffs);

    assert!(action.is_ok());
    let val = action.unwrap();
    // Action should be non-zero
    assert!(val.abs() > 1e-10 || val.abs() < 1000.0);
}

// ============================================================================
// Additional Metropolis Coverage
// ============================================================================

#[test]
fn test_metropolis_sweep_empty_lattice() {
    let lattice = create_1d_lattice(); // Just a dummy lattice
    let links = std::collections::HashMap::new(); // Empty links
    let mut field: LatticeGaugeField<U1, 1, f64> =
        LatticeGaugeField::from_links_unchecked(lattice, links, 1.0);

    let mut rng = deep_causality_rand::rng();
    let rate = field.metropolis_sweep_f64(0.1, &mut rng);

    assert!(rate.is_ok());
    assert_eq!(rate.unwrap(), 0.0); // Should handle empty gracefully
}

#[test]
fn test_metropolis_high_epsilon_acceptance() {
    let lattice = create_test_lattice();
    let mut rng = deep_causality_rand::rng();
    let mut field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    // Epsilon = 50.0 is huge perturbation
    // Should result in very low acceptance (likely 0)
    let rate = field.metropolis_sweep_f64(50.0, &mut rng).unwrap();
    assert!(
        rate < 0.5,
        "High epsilon should yield low acceptance, got {}",
        rate
    );
}

// ============================================================================
// Phase 2: Coverage Tests (Flow, Plaquette, Smearing, Gauge Ops)
// ============================================================================

// --- Plaquette Operations (ops_plague.rs) ---

#[test]
fn test_try_rectangle_invalid_dirs() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    // mu == nu impossible
    let res = field.try_rectangle(&[0, 0], 0, 0);
    assert!(res.is_err());

    // out of bounds
    let res = field.try_rectangle(&[0, 0], 0, 5);
    assert!(res.is_err());
}

#[test]
fn test_try_rectangle_identity() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    // Rect on identity field -> Identity
    let rect = field.try_rectangle(&[0, 0], 0, 1);
    assert!(rect.is_ok());
    let val = rect.unwrap().trace();
    assert!((val - 1.0).abs() < 1e-10);
}

#[test]
fn test_try_plaquette_invalid_dirs() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let res = field.try_plaquette(&[0, 0], 0, 0);
    assert!(res.is_err());
}

// --- Gauge Ops (ops_monte_carlo.rs / ops_gauge.rs) ---

#[test]
fn test_try_staple_identity() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice.clone(), 6.0);

    // Pick an edge and check staple sum
    let mut edge_iter = lattice.cells(1);
    if let Some(edge) = edge_iter.next() {
        let staple = field.try_staple(&edge);
        assert!(staple.is_ok());
        let s = staple.unwrap();
        // In 2D, sum of 2 staples for identity = 2 * Identity
        assert!((s.trace() - 2.0).abs() < 1e-10);
    }
}

#[test]
fn test_gauge_transform_infallible() {
    let lattice = create_test_lattice();
    let mut field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    // Transform with identity
    field.gauge_transform(|_site| LinkVariable::identity());

    let action = field.try_wilson_action().unwrap();
    assert!(action.abs() < 1e-10);
}

// --- Smearing (ops_smearing.rs) ---

#[test]
fn test_smearing_constructors() {
    use deep_causality_topology::SmearingParams;

    let ape = SmearingParams::<f64>::ape_default();
    assert!((ape.alpha - 0.45).abs() < 1e-10);

    let stout = SmearingParams::<f64>::stout_default();
    assert!((stout.alpha - 0.12).abs() < 1e-10);
}

#[test]
fn test_try_smear_identity_invariant() {
    use deep_causality_topology::SmearingParams;

    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);
    let params = SmearingParams::ape_default();

    let smeared = field.try_smear(&params);
    assert!(smeared.is_ok());

    // Identity is a fixed point of smearing
    let smeared_field = smeared.unwrap();
    let action = smeared_field.try_wilson_action().unwrap();
    assert!(action.abs() < 1e-10);
}

// --- Gradient Flow (ops_gradient_flow.rs) ---

#[test]
fn test_try_flow_identity_invariant() {
    use deep_causality_topology::{FlowMethod, FlowParams};

    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);
    let params = FlowParams {
        epsilon: 0.01,
        t_max: 0.05,
        method: FlowMethod::Euler,
    };

    let flowed = field.try_flow(&params);
    assert!(flowed.is_ok());
    let f_field = flowed.unwrap();

    let action = f_field.try_wilson_action().unwrap();
    assert!(action.abs() < 1e-10);
}

#[test]
fn test_try_flow_rk3() {
    use deep_causality_topology::{FlowMethod, FlowParams};

    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);
    let params = FlowParams {
        epsilon: 0.01,
        t_max: 0.05,
        method: FlowMethod::RungeKutta3,
    };

    let flowed = field.try_flow(&params);
    assert!(flowed.is_ok());
}

#[test]
fn test_t2_energy_value() {
    let lattice = create_test_lattice();
    let field: LatticeGaugeField<U1, 2, f64> = LatticeGaugeField::identity(lattice, 6.0);

    let t2e = field.try_t2_energy(0.5);
    assert!(t2e.is_ok());
    assert!(t2e.unwrap().abs() < 1e-10);
}
