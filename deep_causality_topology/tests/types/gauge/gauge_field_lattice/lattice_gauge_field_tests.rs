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
