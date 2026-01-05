/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_metric::Metric;
use deep_causality_physics::theories::{ElectroweakField, ElectroweakOps, ElectroweakParams};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{
    BaseTopology, GaugeField, Manifold, Simplex, SimplicialComplexBuilder,
};

fn create_dummy_ew_field() -> ElectroweakField {
    let mut builder = SimplicialComplexBuilder::new(0);
    let _ = builder.add_simplex(Simplex::new(vec![0]));
    let complex = builder.build().expect("Failed to build complex");

    let data = CausalTensor::new(vec![0.0], vec![1]).unwrap();

    let base = Manifold::new(complex, data, 0).expect("Failed to create manifold");

    let num_points = base.len();
    // Electroweak has 4 generators (3 SU2 + 1 U1)
    let conn = CausalTensor::zeros(&[num_points, 4, 4]);
    let strength = CausalTensor::zeros(&[num_points, 4, 4, 4]);

    GaugeField::new(base, Metric::Minkowski(4), conn, strength)
        .expect("Failed to create ElectroweakField")
}

#[test]
fn test_ew_params() {
    let params = ElectroweakParams::standard_model();

    // Check relations
    // e = g * sin(theta)
    let e = params.em_coupling();
    let g = params.g_coupling();
    let sin = params.sin_theta_w();

    assert!((e - g * sin).abs() < 1e-6);

    // M_W = M_Z * cos(theta)
    let mw = params.w_mass_computed();
    let mz = params.z_mass_computed();
    let cos = params.cos_theta_w();

    assert!((mw - mz * cos).abs() < 1e-6);
}

#[test]
fn test_ew_extraction() {
    let ew = create_dummy_ew_field();

    // This calls the mixing logic we implemented
    let photon = ew.extract_photon();
    assert!(photon.is_ok());

    let z = ew.extract_z();
    assert!(z.is_ok());

    // Check extracted field structure
    let qed = photon.unwrap();
    assert!(qed.gauge_group_name().contains("U(1)"));
}
