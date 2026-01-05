/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;
use deep_causality_physics::theories::{WeakField, WeakIsospin, WeakOps};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;
use deep_causality_topology::{BaseTopology, GaugeField, Simplex, SimplicialComplexBuilder};

fn create_dummy_weak_field() -> WeakField {
    // Need to create a valid GaugeField<SU2>
    let mut builder = SimplicialComplexBuilder::new(0);
    let _ = builder.add_simplex(Simplex::new(vec![0]));
    let complex = builder.build().expect("Failed to build complex");

    // Manifold data must match complex size (1 simplex)
    let data = CausalTensor::new(vec![0.0], vec![1]).unwrap();

    let base = Manifold::new(complex, data, 0).expect("Failed to create manifold");

    // Default metric for SU(2) might be implied or explicit.
    // SU(2) has WestCoast.

    // Connection: dim 4, lie_algebra 3
    let num_points = base.len();
    let conn = CausalTensor::zeros(&[num_points, 4, 3]);
    let strength = CausalTensor::zeros(&[num_points, 4, 4, 3]);

    GaugeField::new(base, Metric::Minkowski(4), conn, strength).expect("Failed to create WeakField")
}

#[test]
fn test_weak_constants_integrity() {
    let weak = create_dummy_weak_field();

    // Check masses (PDG values roughly)
    assert!(weak.w_mass() > 80.0 && weak.w_mass() < 81.0);
    assert!(weak.z_mass() > 91.0 && weak.z_mass() < 92.0);
    assert!(weak.sin2_theta_w() > 0.23 && weak.sin2_theta_w() < 0.24);
}

#[test]
fn test_weak_decay() {
    // Muon decay
    let lifetime = WeakField::muon_lifetime();
    // Lifetime roughly 2.2e-6 seconds
    assert!(lifetime > 2.0e-6 && lifetime < 2.3e-6);

    // W width
    let w_width = WeakField::w_boson_width();
    assert!(w_width > 0.0);
}

#[test]
fn test_propagators() {
    // Low energy limit q^2 << M_W^2
    let prop = WeakField::charged_current_propagator(0.1).unwrap();
    // Should be approx -1/M_W^2
    let mw = 80.377;
    let expected = -1.0 / (mw * mw);

    // Use relative error
    let diff = (prop - expected).abs();
    assert!(diff < 1e-6);

    // Neutral current
    let nu = WeakIsospin::neutrino();
    let prop_z = WeakField::neutral_current_propagator(0.1, &nu).unwrap();
    assert!(prop_z.abs() > 0.0);
}
