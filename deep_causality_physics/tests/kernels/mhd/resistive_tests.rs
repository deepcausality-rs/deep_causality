/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AlfvenSpeed, Diffusivity, PhysicsErrorEnum, magnetic_reconnection_rate_kernel,
    resistive_diffusion_kernel,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{Manifold, PointCloud, ReggeGeometry, SimplicialManifold};

/// The dummy manifold with caller-supplied data, so the diffusion term is not identically zero.
fn manifold_with_data(data: Vec<f64>) -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    assert_eq!(data.len(), num, "data must match the simplex count");
    let metric =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    Manifold::with_metric(
        complex,
        CausalTensor::new(data, vec![num]).unwrap(),
        Some(metric),
        0,
    )
    .unwrap()
}

fn create_dummy_manifold() -> SimplicialManifold<f64, f64> {
    let points = CausalTensor::new(vec![0.0, 0.0, 1.0, 0.0, 0.5, 0.866], vec![3, 2]).unwrap();
    let point_cloud =
        PointCloud::new(points, CausalTensor::new(vec![0.0; 3], vec![3]).unwrap(), 0).unwrap();
    let complex = point_cloud.triangulate(1.1).unwrap();
    let num = complex.total_simplices();
    let num_edges = complex.skeletons()[1].simplices().len();
    let metric =
        ReggeGeometry::new(CausalTensor::new(vec![1.0; num_edges], vec![num_edges]).unwrap());
    Manifold::with_metric(
        complex,
        CausalTensor::new(vec![0.0; num], vec![num]).unwrap(),
        Some(metric),
        0,
    )
    .unwrap()
}

#[test]
fn test_resistive_diffusion_vanishes_for_a_zero_field() {
    // The flat case. The comment used to concede that the answer is zero whatever the algebra
    // does, so on its own this pins nothing; it sits beside the two tests below.
    let m = create_dummy_manifold();
    let eta = Diffusivity::<f64>::new(0.1).unwrap();

    let rate = resistive_diffusion_kernel(&m, eta).unwrap();
    for (i, v) in rate.as_slice().iter().enumerate() {
        assert!(v.abs() < 1e-12, "component {i} = {v} must vanish");
    }
}

#[test]
fn test_resistive_diffusion_is_linear_in_the_diffusivity() {
    // rate = -eta * laplacian(B), so eta -> k eta scales the answer by k for any field. No
    // oracle needed, and it fails if the diffusivity is dropped or squared.
    let m = manifold_with_data(vec![1.0, -2.0, 3.0, 0.5, -1.5, 2.5, 4.0]);
    let base = resistive_diffusion_kernel(&m, Diffusivity::<f64>::new(0.1).unwrap()).unwrap();

    for k in [0.5_f64, 2.0, 7.0] {
        let scaled =
            resistive_diffusion_kernel(&m, Diffusivity::<f64>::new(0.1 * k).unwrap()).unwrap();
        let a: &[f64] = base.as_slice();
        let b: &[f64] = scaled.as_slice();
        for (i, (x, y)) in a.iter().zip(b).enumerate() {
            assert!(
                (y - k * x).abs() < 1e-12,
                "k = {k}, component {i}: {y} against {}",
                k * x
            );
        }
    }
}

#[test]
fn test_resistive_diffusion_is_non_zero_and_linear_in_the_field() {
    // A non-zero field must produce a non-zero rate, and doubling the field doubles it. The
    // previous fixture was an all-zero field, where every implementation returns zero.
    let data = vec![1.0, -2.0, 3.0, 0.5, -1.5, 2.5, 4.0];
    let eta = Diffusivity::<f64>::new(0.25).unwrap();

    let single = resistive_diffusion_kernel(&manifold_with_data(data.clone()), eta).unwrap();
    assert!(
        single.as_slice().iter().any(|v: &f64| v.abs() > 1e-9),
        "a non-zero field must give a non-zero diffusion rate"
    );

    let doubled_data: Vec<f64> = data.iter().map(|v| v * 2.0).collect();
    let doubled = resistive_diffusion_kernel(&manifold_with_data(doubled_data), eta).unwrap();
    let a: &[f64] = single.as_slice();
    let b: &[f64] = doubled.as_slice();
    for (i, (x, y)) in a.iter().zip(b).enumerate() {
        assert!(
            (y - 2.0 * x).abs() < 1e-12,
            "component {i}: {y} against {}",
            2.0 * x
        );
    }
}

#[test]
fn test_resistive_diffusion_negative_diffusivity_error() {
    // Diffusivity::new rejects negatives, so use new_unchecked to feed a
    // negative eta straight into the kernel and trip its PhysicalInvariantBroken
    // guard (resistive.rs:25-29).
    let m = create_dummy_manifold();
    let eta = Diffusivity::<f64>::new_unchecked(-0.5);
    let res = resistive_diffusion_kernel(&m, eta);
    assert!(
        matches!(
            res.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_reconnection_rate_non_positive_lundquist_error() {
    // lundquist <= 0 -> Singularity (resistive.rs:51-55).
    let va = AlfvenSpeed::<f64>::new(100.0).unwrap();
    assert!(
        matches!(
            magnetic_reconnection_rate_kernel(va, 0.0).unwrap_err().0,
            PhysicsErrorEnum::Singularity { .. }
        ),
        "expected a Singularity refusal"
    );
    let va2 = AlfvenSpeed::<f64>::new(100.0).unwrap();
    assert!(
        matches!(
            magnetic_reconnection_rate_kernel(va2, -1.0).unwrap_err().0,
            PhysicsErrorEnum::Singularity { .. }
        ),
        "expected a Singularity refusal"
    );
}

#[test]
fn test_reconnection_rate() {
    let va = AlfvenSpeed::<f64>::new(100.0).unwrap();
    let s = 100.0; // Lundquist

    let res = magnetic_reconnection_rate_kernel(va, s);
    assert!(res.is_ok());
    // vin = va / sqrt(S) = 100 / 10 = 10
    assert!((res.unwrap().value() - 10.0).abs() < 1e-10);
}

#[test]
fn test_reconnection_rate_follows_the_sweet_parker_exponent() {
    // The Sweet-Parker model's content is the exponent: the dimensionless rate is
    // M = v_in/v_A = S^(-1/2). A single pinned quotient fixes one point and cannot tell that
    // exponent from -1 or -1/3, all of which agree somewhere.
    //
    // Provenance: Sweet, *Electromagnetic Phenomena in Cosmical Physics*, 1958; Parker,
    // J. Geophys. Res. 62, 509 (1957). Contrast Petschek, which scales as 1/ln S.
    let va = AlfvenSpeed::<f64>::new(1.0e6).unwrap();

    let at = |s: f64| {
        magnetic_reconnection_rate_kernel(AlfvenSpeed::<f64>::new(1.0e6).unwrap(), s)
            .unwrap()
            .value()
    };

    // Quadrupling S halves the inflow speed.
    assert!(
        (at(4.0e6) * 2.0 - at(1.0e6)).abs() / at(1.0e6) < 1e-12,
        "4x Lundquist gave {}, expected {}",
        at(4.0e6),
        at(1.0e6) / 2.0
    );

    // A millionfold Lundquist number gives a rate of exactly 1e-3 of the Alfven speed, the
    // figure quoted for Sweet-Parker reconnection in the solar corona.
    assert!(
        (at(1.0e6) / va.value() - 1.0e-3).abs() < 1e-15,
        "M = {}, expected 1e-3",
        at(1.0e6) / va.value()
    );
}
